import type { CreatureAnimationAuthoringV1 } from "../animation-mapping/types";
import {
  CREATURE_ANIMATION_AUTHORING_PROFILE_V2,
  type AnimationKeyframeV1,
  type AnimationStudioDiagnosticV1,
  type AnimationStudioDocumentParseResultV1,
  type AnimationStudioDocumentV1,
  type AuthoredAnimationClipV1,
  type AuthoredAnimationEventV1,
  type AuthoredAnimationSourceV1,
  type AuthoredAnimationTrackV1,
  type CreatureAnimationAuthoringV2,
  type CustomAnimationClipReferenceV2,
  type CustomAnimationDefinitionV2,
} from "./types";
import { ANIMATION_STUDIO_PRODUCT_LIMITS_V1 } from "./limits";

const SHA_256_HEX = /^[a-f0-9]{64}$/i;

export function serializeAnimationStudioDocumentV1(
  document: AnimationStudioDocumentV1,
): string {
  const diagnostics = validateAnimationStudioSchemaV1(document);
  if (diagnostics.length > 0) {
    throw new AnimationStudioSchemaError(diagnostics);
  }
  // Field order intentionally mirrors the Rust serde struct order. This is
  // part of the cross-language fingerprint contract, while remaining
  // independent of the caller's object insertion order.
  return JSON.stringify({
    schemaVersion: document.schemaVersion,
    sourceRevision: document.sourceRevision,
    authoringRevision: document.authoringRevision,
    status: document.status,
    authoredClips: document.authoredClips.map((clip) => ({
      id: clip.id,
      name: clip.name,
      kind: clip.kind,
      status: clip.status,
      source: {
        kind: clip.source.kind,
        sourceRevision: clip.source.sourceRevision,
        sourceClipName: clip.source.sourceClipName,
        sourceClipFingerprint: clip.source.sourceClipFingerprint,
        proceduralTemplate: clip.source.proceduralTemplate,
      },
      lengthSeconds: canonicalFloat32ForWireV1(clip.lengthSeconds),
      transitionSeconds: canonicalFloat32ForWireV1(clip.transitionSeconds),
      animationRoot: clip.animationRoot,
      tracks: clip.tracks.map((track) => ({
        id: track.id,
        targetNodeId: track.targetNodeId,
        path: track.path,
        interpolation: track.interpolation,
        keyframes: track.keyframes.map((keyframe) => ({
          id: keyframe.id,
          timeSeconds: canonicalFloat32ForWireV1(keyframe.timeSeconds),
          value: keyframe.value.map(canonicalFloat32ForWireV1),
        })),
      })),
      events: clip.events.map((event) => ({
        id: event.id,
        timeSeconds: canonicalFloat32ForWireV1(event.timeSeconds),
        name: event.name,
      })),
      revision: clip.revision,
    })),
  });
}

/**
 * Rust owns the canonical Animation Studio wire model and stores transform and
 * time values as `f32`. JavaScript performs editor math as `number` (`f64`), so
 * serializing the raw value would make the TypeScript and Rust fingerprints
 * disagree even though they represent the same writer value. This projects a
 * finite JS number to the shortest decimal that round-trips to the exact same
 * IEEE-754 `f32`, matching serde_json/ryu's `f32` wire representation.
 */
export function canonicalFloat32ForWireV1(value: number): number {
  const projected = Math.fround(value);
  if (Object.is(projected, -0) || projected === 0) return 0;
  if (!Number.isFinite(projected)) return projected;
  for (let precision = 1; precision <= 9; precision += 1) {
    const candidate = Number(projected.toPrecision(precision));
    if (Math.fround(candidate) === projected) return candidate;
  }
  return Number(projected.toPrecision(9));
}

export function parseAnimationStudioDocumentV1(
  json: string,
): AnimationStudioDocumentParseResultV1 {
  let value: unknown;
  try {
    value = JSON.parse(json);
  } catch (error) {
    return {
      kind: "INVALID",
      diagnostics: [diagnostic(
        "M2A-ANIMATION-EDIT-SCHEMA",
        "$",
        `Animation Studio document is not valid JSON: ${errorMessage(error)}`,
        "Restore a valid Animation Studio V1 document.",
      )],
    };
  }
  const diagnostics = validateAnimationStudioSchemaV1(value);
  return diagnostics.length > 0
    ? { kind: "INVALID", diagnostics }
    : { kind: "VALID", value: value as AnimationStudioDocumentV1 };
}

export async function fingerprintAnimationStudioDocumentV1(
  document: AnimationStudioDocumentV1,
): Promise<string> {
  const bytes = new TextEncoder().encode(
    serializeAnimationStudioDocumentV1(document),
  );
  if (!globalThis.crypto?.subtle) {
    throw new Error("Web Crypto SHA-256 is unavailable");
  }
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

export function migrateCreatureAnimationAuthoringV1ToV2(
  v1: CreatureAnimationAuthoringV1,
): CreatureAnimationAuthoringV2 {
  return {
    schemaVersion: 2,
    profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V2,
    modelType: v1.modelType,
    sourceRevision: v1.sourceRevision,
    authoringRevision: v1.authoringRevision,
    assignments: v1.assignments.map((assignment) => structuredClone(assignment)),
    fallbacks: v1.fallbacks.map((fallback) => structuredClone(fallback)),
    customAnimations: v1.customAnimations.map((custom) => ({
      id: custom.id,
      name: custom.name,
      playback: custom.playback,
      clipReference: custom.sourceClipName === null
        ? null
        : {
            sourceKind: "SOURCE_CLIP",
            sourceClipName: custom.sourceClipName,
            authoredClipId: null,
          },
      phases: custom.phases.map((phase) => ({
        phase: phase.phase,
        clipReference: {
          sourceKind: "SOURCE_CLIP",
          sourceClipName: phase.sourceClipName,
          authoredClipId: null,
        },
      })),
      provenance: structuredClone(custom.provenance),
    })),
  };
}

export function validateAnimationStudioSchemaV1(
  document: unknown,
): AnimationStudioDiagnosticV1[] {
  const diagnostics: AnimationStudioDiagnosticV1[] = [];
  validateDocument(document, "$", diagnostics);
  if (diagnostics.length > 0 || !isRecord(document)) return diagnostics;

  const clips = document.authoredClips as AuthoredAnimationClipV1[];
  if (clips.length > ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxAuthoredClips) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-EDIT-ROW-LIMIT",
      "$.authoredClips",
      `Animation Studio supports at most ${
        ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxAuthoredClips
      } authored clips per project.`,
      "Remove or archive authored clips before adding another.",
    ));
  }
  let totalKeyframes = 0;
  reportDuplicates(
    clips.map(({ id }) => id),
    "$.authoredClips",
    "M2A-ANIMATION-EDIT-CLIP-DUPLICATE",
    "authored clip ID",
    diagnostics,
  );
  reportDuplicates(
    clips.map(({ name }) => name.toLocaleLowerCase("en-US")),
    "$.authoredClips",
    "M2A-ANIMATION-EDIT-CLIP-NAME",
    "authored output name",
    diagnostics,
  );
  clips.forEach((clip, clipIndex) => {
    const clipKeyframes = clip.tracks.reduce(
      (total, track) => total + track.keyframes.length,
      0,
    );
    totalKeyframes += clipKeyframes;
    if (
      clipKeyframes
      > ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxKeyframesPerClip
    ) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-EDIT-ROW-LIMIT",
        `$.authoredClips[${clipIndex}].tracks`,
        `Authored clip has ${clipKeyframes} keyframes; the product limit is ${
          ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxKeyframesPerClip
        }.`,
        "Trim, resample, or split the authored animation.",
      ));
    }
    if (
      clip.source.kind !== "IMPORTED_MODEL_COPY"
      && clip.source.sourceRevision !== document.sourceRevision
    ) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-EDIT-SOURCE-STALE",
        `$.authoredClips[${clipIndex}].source.sourceRevision`,
        "Authored clip source revision differs from its Studio document.",
        "Reconcile this clip with the exact current source GLB.",
      ));
    }
    reportDuplicates(
      clip.tracks.map(({ id }) => id),
      `$.authoredClips[${clipIndex}].tracks`,
      "M2A-ANIMATION-EDIT-SCHEMA",
      "track ID",
      diagnostics,
    );
    clip.tracks.forEach((track, trackIndex) => {
      reportDuplicates(
        track.keyframes.map(({ id }) => id),
        `$.authoredClips[${clipIndex}].tracks[${trackIndex}].keyframes`,
        "M2A-ANIMATION-EDIT-SCHEMA",
        "keyframe ID",
        diagnostics,
      );
    });
    reportDuplicates(
      clip.events.map(({ id }) => id),
      `$.authoredClips[${clipIndex}].events`,
      "M2A-ANIMATION-EDIT-SCHEMA",
      "event ID",
      diagnostics,
    );
  });
  if (
    totalKeyframes
    > ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxTotalKeyframes
  ) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-EDIT-ROW-LIMIT",
      "$.authoredClips",
      `Animation Studio document has ${totalKeyframes} keyframes; the project limit is ${
        ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxTotalKeyframes
      }.`,
      "Remove, resample, or split authored animations.",
    ));
  }
  return diagnostics;
}

export function validateCreatureAnimationAuthoringV2(
  authoring: CreatureAnimationAuthoringV2,
  studio: AnimationStudioDocumentV1,
): AnimationStudioDiagnosticV1[] {
  const diagnostics = validateAnimationStudioSchemaV1(studio);
  if (
    authoring.schemaVersion !== 2
    || authoring.profile !== CREATURE_ANIMATION_AUTHORING_PROFILE_V2
  ) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-EDIT-SCHEMA",
      "$.authoring.schemaVersion",
      "Creature animation authoring is not the supported V2 profile.",
      "Migrate the mapping explicitly from V1 to V2.",
    ));
    return diagnostics;
  }
  if (authoring.sourceRevision !== studio.sourceRevision) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-EDIT-SOURCE-STALE",
      "$.authoring.sourceRevision",
      "Animation mapping and Studio document use different source revisions.",
      "Reconcile the authored clips with the exact current source GLB.",
    ));
  }

  reportDuplicates(
    authoring.customAnimations.map(({ id }) => id),
    "$.authoring.customAnimations",
    "M2A-ANIMATION-EDIT-CLIP-DUPLICATE",
    "custom animation ID",
    diagnostics,
  );
  reportDuplicates(
    authoring.customAnimations.map(({ name }) => name.toLocaleLowerCase("en-US")),
    "$.authoring.customAnimations",
    "M2A-ANIMATION-EDIT-CLIP-NAME",
    "custom output name",
    diagnostics,
  );

  const authoredIds = new Set(studio.authoredClips.map(({ id }) => id));
  const customIds = new Set(authoring.customAnimations.map(({ id }) => id));
  authoring.customAnimations.forEach((custom, index) => {
    validateCustomDefinitionReferences(
      custom,
      authoredIds,
      `$.authoring.customAnimations[${index}]`,
      diagnostics,
    );
  });
  authoring.assignments.forEach((assignment, index) => {
    if (
      assignment.sourceKind === "CUSTOM"
      && (
        assignment.customAnimationId === null
        || !customIds.has(assignment.customAnimationId)
      )
    ) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-EDIT-SCHEMA",
        `$.authoring.assignments[${index}].customAnimationId`,
        "Custom assignment references an unknown custom animation ID.",
        "Choose an existing valid custom animation.",
      ));
    }
  });
  return diagnostics;
}

function validateDocument(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (!expectRecordWithKeys(
    value,
    path,
    ["schemaVersion", "sourceRevision", "authoringRevision", "status", "authoredClips"],
    [],
    diagnostics,
  )) return;
  if (value.schemaVersion !== 1) {
    schemaIssue(`${path}.schemaVersion`, "Expected schemaVersion 1.", diagnostics);
  }
  expectSha256(value.sourceRevision, `${path}.sourceRevision`, diagnostics);
  expectPositiveInteger(
    value.authoringRevision,
    `${path}.authoringRevision`,
    diagnostics,
  );
  expectEnum(
    value.status,
    ["DRAFT", "VALID", "INVALID"],
    `${path}.status`,
    diagnostics,
  );
  if (!Array.isArray(value.authoredClips)) {
    schemaIssue(`${path}.authoredClips`, "Expected an array.", diagnostics);
  } else {
    value.authoredClips.forEach((clip, index) => {
      validateClip(clip, `${path}.authoredClips[${index}]`, diagnostics);
    });
  }
}

function validateClip(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (!expectRecordWithKeys(
    value,
    path,
    [
      "id",
      "name",
      "kind",
      "status",
      "source",
      "lengthSeconds",
      "transitionSeconds",
      "animationRoot",
      "tracks",
      "events",
      "revision",
    ],
    [],
    diagnostics,
  )) return;
  expectNonEmptyString(value.id, `${path}.id`, diagnostics);
  expectNonEmptyString(value.name, `${path}.name`, diagnostics);
  expectEnum(value.kind, ["MOTION", "STATIC_POSE"], `${path}.kind`, diagnostics);
  expectEnum(value.status, ["DRAFT", "VALID", "INVALID"], `${path}.status`, diagnostics);
  validateSource(value.source, `${path}.source`, diagnostics);
  expectFiniteNonNegative(value.lengthSeconds, `${path}.lengthSeconds`, diagnostics);
  expectMaximum(
    value.lengthSeconds,
    ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxDurationSeconds,
    `${path}.lengthSeconds`,
    "clip duration",
    diagnostics,
  );
  expectFiniteNonNegative(
    value.transitionSeconds,
    `${path}.transitionSeconds`,
    diagnostics,
  );
  expectMaximum(
    value.transitionSeconds,
    ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxDurationSeconds,
    `${path}.transitionSeconds`,
    "transition duration",
    diagnostics,
  );
  expectNonEmptyString(value.animationRoot, `${path}.animationRoot`, diagnostics);
  expectPositiveInteger(value.revision, `${path}.revision`, diagnostics);
  if (!Array.isArray(value.tracks)) {
    schemaIssue(`${path}.tracks`, "Expected an array.", diagnostics);
  } else {
    value.tracks.forEach((track, index) => {
      validateTrack(track, `${path}.tracks[${index}]`, diagnostics);
    });
  }
  if (!Array.isArray(value.events)) {
    schemaIssue(`${path}.events`, "Expected an array.", diagnostics);
  } else {
    value.events.forEach((event, index) => {
      validateEvent(event, `${path}.events[${index}]`, diagnostics);
    });
  }
}

function validateSource(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (!expectRecordWithKeys(
    value,
    path,
    [
      "kind",
      "sourceRevision",
      "sourceClipName",
      "sourceClipFingerprint",
      "proceduralTemplate",
    ],
    [],
    diagnostics,
  )) return;
  expectEnum(
    value.kind,
    [
      "BLANK_POSE",
      "SOURCE_CLIP_COPY",
      "IMPORTED_MODEL_COPY",
      "PROCEDURAL_TEMPLATE",
    ],
    `${path}.kind`,
    diagnostics,
  );
  expectSha256(value.sourceRevision, `${path}.sourceRevision`, diagnostics);
  for (const key of [
    "sourceClipName",
    "sourceClipFingerprint",
    "proceduralTemplate",
  ] as const) {
    if (value[key] !== null) {
      expectNonEmptyString(value[key], `${path}.${key}`, diagnostics);
    }
  }
  if (
    value.sourceClipFingerprint !== null
    && !SHA_256_HEX.test(String(value.sourceClipFingerprint))
  ) {
    schemaIssue(
      `${path}.sourceClipFingerprint`,
      "Expected a SHA-256 hex fingerprint.",
      diagnostics,
    );
  }
  if (
    (value.kind === "SOURCE_CLIP_COPY" || value.kind === "IMPORTED_MODEL_COPY")
    && (
      typeof value.sourceClipName !== "string"
      || typeof value.sourceClipFingerprint !== "string"
    )
  ) {
    schemaIssue(
      path,
      `${String(value.kind)} requires sourceClipName and sourceClipFingerprint.`,
      diagnostics,
    );
  }
  if (
    (value.kind === "SOURCE_CLIP_COPY" || value.kind === "IMPORTED_MODEL_COPY")
    && value.proceduralTemplate !== null
  ) {
    schemaIssue(
      `${path}.proceduralTemplate`,
      `${String(value.kind)} cannot contain proceduralTemplate.`,
      diagnostics,
    );
  }
  if (
    value.kind === "PROCEDURAL_TEMPLATE"
    && typeof value.proceduralTemplate !== "string"
  ) {
    schemaIssue(
      path,
      "PROCEDURAL_TEMPLATE requires proceduralTemplate.",
      diagnostics,
    );
  }
  if (
    value.kind === "PROCEDURAL_TEMPLATE"
    && (value.sourceClipName !== null || value.sourceClipFingerprint !== null)
  ) {
    schemaIssue(
      path,
      "PROCEDURAL_TEMPLATE cannot contain source clip fields.",
      diagnostics,
    );
  }
  if (
    value.kind === "BLANK_POSE"
    && (
      value.sourceClipName !== null
      || value.sourceClipFingerprint !== null
      || value.proceduralTemplate !== null
    )
  ) {
    schemaIssue(
      path,
      "BLANK_POSE cannot contain source clip or procedural fields.",
      diagnostics,
    );
  }
}

function validateTrack(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (!expectRecordWithKeys(
    value,
    path,
    ["id", "targetNodeId", "path", "interpolation", "keyframes"],
    [],
    diagnostics,
  )) return;
  expectNonEmptyString(value.id, `${path}.id`, diagnostics);
  expectUnsignedInt32(value.targetNodeId, `${path}.targetNodeId`, diagnostics);
  expectEnum(value.path, ["TRANSLATION", "ROTATION"], `${path}.path`, diagnostics);
  expectEnum(value.interpolation, ["LINEAR"], `${path}.interpolation`, diagnostics);
  if (!Array.isArray(value.keyframes)) {
    schemaIssue(`${path}.keyframes`, "Expected an array.", diagnostics);
  } else {
    value.keyframes.forEach((keyframe, index) => {
      validateKeyframe(keyframe, `${path}.keyframes[${index}]`, diagnostics);
    });
  }
}

function validateKeyframe(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (!expectRecordWithKeys(
    value,
    path,
    ["id", "timeSeconds", "value"],
    [],
    diagnostics,
  )) return;
  expectNonEmptyString(value.id, `${path}.id`, diagnostics);
  expectFiniteNonNegative(value.timeSeconds, `${path}.timeSeconds`, diagnostics);
  expectMaximum(
    value.timeSeconds,
    ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxDurationSeconds,
    `${path}.timeSeconds`,
    "keyframe time",
    diagnostics,
  );
  if (
    !Array.isArray(value.value)
    || value.value.length === 0
    || value.value.some((component) => (
      typeof component !== "number" || !Number.isFinite(component)
    ))
  ) {
    schemaIssue(`${path}.value`, "Expected a non-empty finite number array.", diagnostics);
  } else if (value.value.some((component) => (
    Math.abs(component as number)
      > ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxAbsoluteTrackValue
  ))) {
    schemaIssue(
      `${path}.value`,
      `Transform components must be within +/-${
        ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxAbsoluteTrackValue
      }.`,
      diagnostics,
    );
  }
}

function validateEvent(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (!expectRecordWithKeys(
    value,
    path,
    ["id", "timeSeconds", "name"],
    [],
    diagnostics,
  )) return;
  expectNonEmptyString(value.id, `${path}.id`, diagnostics);
  expectFiniteNonNegative(value.timeSeconds, `${path}.timeSeconds`, diagnostics);
  expectMaximum(
    value.timeSeconds,
    ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxDurationSeconds,
    `${path}.timeSeconds`,
    "event time",
    diagnostics,
  );
  expectNonEmptyString(value.name, `${path}.name`, diagnostics);
}

function validateCustomDefinitionReferences(
  custom: CustomAnimationDefinitionV2,
  authoredIds: ReadonlySet<string>,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (custom.playback === "ONE_SHOT") {
    if (custom.clipReference === null) {
      schemaIssue(
        `${path}.clipReference`,
        "ONE_SHOT requires a clip reference.",
        diagnostics,
      );
    } else {
      validateReference(custom.clipReference, authoredIds, `${path}.clipReference`, diagnostics);
    }
    if (custom.phases.length > 0) {
      schemaIssue(`${path}.phases`, "ONE_SHOT cannot contain phases.", diagnostics);
    }
  } else {
    if (custom.clipReference !== null) {
      schemaIssue(
        `${path}.clipReference`,
        "LOOPING_PHASED uses phase references, not clipReference.",
        diagnostics,
      );
    }
    const phases = new Set(custom.phases.map(({ phase }) => phase));
    if (phases.size !== custom.phases.length) {
      schemaIssue(
        `${path}.phases`,
        "LOOPING_PHASED cannot contain duplicate phases.",
        diagnostics,
      );
    }
    for (const required of ["START", "LOOP", "END"] as const) {
      if (!phases.has(required)) {
        schemaIssue(`${path}.phases`, `Missing ${required} phase.`, diagnostics);
      }
    }
    custom.phases.forEach((phase, index) => {
      validateReference(
        phase.clipReference,
        authoredIds,
        `${path}.phases[${index}].clipReference`,
        diagnostics,
      );
    });
  }
}

function validateReference(
  reference: CustomAnimationClipReferenceV2,
  authoredIds: ReadonlySet<string>,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (reference.sourceKind === "SOURCE_CLIP") {
    if (!reference.sourceClipName || reference.authoredClipId !== null) {
      schemaIssue(
        path,
        "SOURCE_CLIP requires only sourceClipName.",
        diagnostics,
      );
    }
  } else if (
    !reference.authoredClipId
    || reference.sourceClipName !== null
    || !authoredIds.has(reference.authoredClipId)
  ) {
    schemaIssue(
      path,
      "AUTHORED_CLIP requires one existing authoredClipId.",
      diagnostics,
    );
  }
}

function reportDuplicates(
  values: readonly string[],
  path: string,
  code: string,
  label: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  const seen = new Set<string>();
  for (const value of values) {
    if (seen.has(value)) {
      diagnostics.push(diagnostic(
        code,
        path,
        `Duplicate ${label}: ${value}.`,
        `Use a unique ${label}; mappings use stable IDs rather than names.`,
      ));
    }
    seen.add(value);
  }
}

function expectRecordWithKeys(
  value: unknown,
  path: string,
  required: readonly string[],
  optional: readonly string[],
  diagnostics: AnimationStudioDiagnosticV1[],
): value is Record<string, unknown> {
  if (!isRecord(value)) {
    schemaIssue(path, "Expected an object.", diagnostics);
    return false;
  }
  const allowed = new Set([...required, ...optional]);
  for (const key of Object.keys(value)) {
    if (!allowed.has(key)) {
      schemaIssue(`${path}.${key}`, "Unknown field.", diagnostics);
    }
  }
  for (const key of required) {
    if (!(key in value)) schemaIssue(`${path}.${key}`, "Missing required field.", diagnostics);
  }
  return true;
}

function expectEnum(
  value: unknown,
  allowed: readonly string[],
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (typeof value !== "string" || !allowed.includes(value)) {
    schemaIssue(path, `Expected one of: ${allowed.join(", ")}.`, diagnostics);
  }
}

function expectSha256(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (typeof value !== "string" || !SHA_256_HEX.test(value)) {
    schemaIssue(path, "Expected a 64-character SHA-256 hex revision.", diagnostics);
  }
}

function expectNonEmptyString(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (typeof value !== "string" || value.trim().length === 0) {
    schemaIssue(path, "Expected a non-empty string.", diagnostics);
  }
}

function expectPositiveInteger(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (!Number.isSafeInteger(value) || (value as number) < 1) {
    schemaIssue(path, "Expected a positive integer.", diagnostics);
  }
}

function expectUnsignedInt32(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (
    !Number.isSafeInteger(value)
    || (value as number) < 0
    || (value as number) > 0xffff_ffff
  ) {
    schemaIssue(path, "Expected an unsigned 32-bit integer.", diagnostics);
  }
}

function expectFiniteNonNegative(
  value: unknown,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    schemaIssue(path, "Expected a finite non-negative number.", diagnostics);
  }
}

function expectMaximum(
  value: unknown,
  maximum: number,
  path: string,
  label: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  if (
    typeof value === "number"
    && Number.isFinite(value)
    && value > maximum
  ) {
    schemaIssue(
      path,
      `${label} exceeds the Animation Studio product limit ${maximum}.`,
      diagnostics,
    );
  }
}

function schemaIssue(
  path: string,
  message: string,
  diagnostics: AnimationStudioDiagnosticV1[],
): void {
  diagnostics.push(diagnostic(
    "M2A-ANIMATION-EDIT-SCHEMA",
    path,
    message,
    "Correct the document before opening or building it.",
  ));
}

function diagnostic(
  code: string,
  path: string,
  message: string,
  action: string,
): AnimationStudioDiagnosticV1 {
  return {
    schemaVersion: 1,
    code,
    path,
    level: "BLOCKING",
    message,
    action,
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export class AnimationStudioSchemaError extends Error {
  constructor(readonly diagnostics: AnimationStudioDiagnosticV1[]) {
    super(diagnostics.map(({ path, message }) => `${path}: ${message}`).join("; "));
    this.name = "AnimationStudioSchemaError";
  }
}

// Compile-time parity sentinels: changing these wire members is a deliberate
// schema change and must be mirrored in Rust fixtures.
void (null as unknown as AuthoredAnimationSourceV1);
void (null as unknown as AuthoredAnimationTrackV1);
void (null as unknown as AnimationKeyframeV1);
void (null as unknown as AuthoredAnimationEventV1);
