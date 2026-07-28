import type {
  AnimationStudioDiagnosticV1,
  CreatureAnimationAuthoringV2,
} from "./types";
import {
  CREATURE_ANIMATION_AUTHORING_PROFILE_V2,
} from "./types";
import {
  FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
} from "../source/directCreatureAnimationProfile";

const STORAGE_PREFIX = "m2a.creatureAnimationAuthoringV2.v1.";

export type CreatureAnimationAuthoringV2LoadResultV1 =
  | { kind: "EMPTY" }
  | { kind: "LOADED"; value: CreatureAnimationAuthoringV2 }
  | { kind: "ERROR"; diagnostic: AnimationStudioDiagnosticV1 };

export type CreatureAnimationAuthoringV2SaveResultV1 =
  | { kind: "SAVED" }
  | { kind: "ERROR"; diagnostic: AnimationStudioDiagnosticV1 };

export function serializeCreatureAnimationAuthoringV2(
  authoring: CreatureAnimationAuthoringV2,
): string {
  return JSON.stringify({
    schemaVersion: authoring.schemaVersion,
    profile: authoring.profile,
    modelType: authoring.modelType,
    sourceRevision: authoring.sourceRevision,
    authoringRevision: authoring.authoringRevision,
    assignments: authoring.assignments.map((assignment) => ({
      targetSlot: assignment.targetSlot,
      sourceKind: assignment.sourceKind,
      sourceClipName: assignment.sourceClipName,
      customAnimationId: assignment.customAnimationId,
      provenance: {
        provider: assignment.provenance.provider,
        assetId: assignment.provenance.assetId,
        ownership: assignment.provenance.ownership,
      },
    })),
    fallbacks: authoring.fallbacks.map((fallback) => ({
      id: fallback.id,
      targetSlot: fallback.targetSlot,
      sourceSlot: fallback.sourceSlot,
      reason: fallback.reason,
      review: fallback.review,
    })),
    customAnimations: authoring.customAnimations.map((custom) => ({
      id: custom.id,
      name: custom.name,
      playback: custom.playback,
      clipReference: custom.clipReference === null ? null : {
        sourceKind: custom.clipReference.sourceKind,
        sourceClipName: custom.clipReference.sourceClipName,
        authoredClipId: custom.clipReference.authoredClipId,
      },
      phases: custom.phases.map((phase) => ({
        phase: phase.phase,
        clipReference: {
          sourceKind: phase.clipReference.sourceKind,
          sourceClipName: phase.clipReference.sourceClipName,
          authoredClipId: phase.clipReference.authoredClipId,
        },
      })),
      provenance: {
        provider: custom.provenance.provider,
        assetId: custom.provenance.assetId,
        ownership: custom.provenance.ownership,
      },
    })),
  });
}

export function parseCreatureAnimationAuthoringV2(
  json: string,
): CreatureAnimationAuthoringV2 {
  const value: unknown = JSON.parse(json);
  assertExactObject(value, [
    "schemaVersion",
    "profile",
    "modelType",
    "sourceRevision",
    "authoringRevision",
    "assignments",
    "fallbacks",
    "customAnimations",
  ], "$");
  if (value.schemaVersion !== 2) throw new Error("$.schemaVersion must be 2");
  requireEnum(
    value.profile,
    [CREATURE_ANIMATION_AUTHORING_PROFILE_V2],
    "$.profile",
  );
  requireEnum(value.modelType, ["S", "L"], "$.modelType");
  requireSha256(value.sourceRevision, "$.sourceRevision");
  requirePositiveInteger(value.authoringRevision, "$.authoringRevision");
  requireArray(value.assignments, "$.assignments").forEach((assignment, index) => {
    const path = `$.assignments[${index}]`;
    assertExactObject(assignment, [
      "targetSlot",
      "sourceKind",
      "sourceClipName",
      "customAnimationId",
      "provenance",
    ], path);
    requireDirectCreatureBaseSlot(
      assignment.targetSlot,
      `${path}.targetSlot`,
    );
    requireEnum(
      assignment.sourceKind,
      ["SOURCE_CLIP", "INHERITED_SUPERMODEL", "PROCEDURAL", "CUSTOM"],
      `${path}.sourceKind`,
    );
    requireNullableString(assignment.sourceClipName, `${path}.sourceClipName`);
    requireNullableString(assignment.customAnimationId, `${path}.customAnimationId`);
    validateProvenance(assignment.provenance, `${path}.provenance`);
  });
  requireArray(value.fallbacks, "$.fallbacks").forEach((fallback, index) => {
    const path = `$.fallbacks[${index}]`;
    assertExactObject(fallback, [
      "id",
      "targetSlot",
      "sourceSlot",
      "reason",
      "review",
    ], path);
    requireString(fallback.id, `${path}.id`);
    requireDirectCreatureBaseSlot(fallback.targetSlot, `${path}.targetSlot`);
    requireDirectCreatureBaseSlot(fallback.sourceSlot, `${path}.sourceSlot`);
    requireString(fallback.reason, `${path}.reason`);
    requireEnum(
      fallback.review,
      ["PENDING", "ACCEPTED", "REJECTED"],
      `${path}.review`,
    );
  });
  requireArray(value.customAnimations, "$.customAnimations").forEach((custom, index) => {
    const path = `$.customAnimations[${index}]`;
    assertExactObject(custom, [
      "id",
      "name",
      "playback",
      "clipReference",
      "phases",
      "provenance",
    ], path);
    requireString(custom.id, `${path}.id`);
    requireString(custom.name, `${path}.name`);
    requireEnum(
      custom.playback,
      ["ONE_SHOT", "LOOPING_PHASED"],
      `${path}.playback`,
    );
    if (custom.clipReference !== null) {
      validateClipReference(custom.clipReference, `${path}.clipReference`);
    }
    requireArray(custom.phases, `${path}.phases`).forEach((phase, phaseIndex) => {
      const phasePath = `${path}.phases[${phaseIndex}]`;
      assertExactObject(phase, ["phase", "clipReference"], phasePath);
      requireEnum(
        phase.phase,
        ["START", "LOOP", "END"],
        `${phasePath}.phase`,
      );
      validateClipReference(phase.clipReference, `${phasePath}.clipReference`);
    });
    validateProvenance(custom.provenance, `${path}.provenance`);
  });
  return value as unknown as CreatureAnimationAuthoringV2;
}

export function saveCreatureAnimationAuthoringV2DraftV1(
  sourceRevision: string,
  authoring: CreatureAnimationAuthoringV2,
  storage: Pick<Storage, "setItem"> | undefined = globalThis.localStorage,
): CreatureAnimationAuthoringV2SaveResultV1 {
  try {
    requireSha256(sourceRevision, "sourceRevision");
    if (authoring.sourceRevision !== sourceRevision) {
      throw new Error("mapping sourceRevision does not match the storage key");
    }
    if (!storage) throw new Error("localStorage is unavailable");
    storage.setItem(`${STORAGE_PREFIX}${sourceRevision}`, serializeCreatureAnimationAuthoringV2(
      authoring,
    ));
    return { kind: "SAVED" };
  } catch (error) {
    return { kind: "ERROR", diagnostic: storageDiagnostic("SAVE", error) };
  }
}

export function loadCreatureAnimationAuthoringV2DraftV1(
  sourceRevision: string,
  storage: Pick<Storage, "getItem"> | undefined = globalThis.localStorage,
): CreatureAnimationAuthoringV2LoadResultV1 {
  try {
    requireSha256(sourceRevision, "sourceRevision");
    if (!storage) throw new Error("localStorage is unavailable");
    const json = storage.getItem(`${STORAGE_PREFIX}${sourceRevision}`);
    if (json === null) return { kind: "EMPTY" };
    const value = parseCreatureAnimationAuthoringV2(json);
    if (value.sourceRevision !== sourceRevision) {
      throw new Error("stored mapping sourceRevision does not match its key");
    }
    return { kind: "LOADED", value };
  } catch (error) {
    return { kind: "ERROR", diagnostic: storageDiagnostic("LOAD", error) };
  }
}

function validateClipReference(value: unknown, path: string) {
  assertExactObject(value, [
    "sourceKind",
    "sourceClipName",
    "authoredClipId",
  ], path);
  requireEnum(
    value.sourceKind,
    ["SOURCE_CLIP", "AUTHORED_CLIP"],
    `${path}.sourceKind`,
  );
  requireNullableString(value.sourceClipName, `${path}.sourceClipName`);
  requireNullableString(value.authoredClipId, `${path}.authoredClipId`);
}

function validateProvenance(value: unknown, path: string) {
  assertExactObject(value, ["provider", "assetId", "ownership"], path);
  requireEnum(
    value.provider,
    [
      "SOURCE_GLB",
      "COMPATIBLE_SUPERMODEL",
      "PROCEDURAL_GENERATOR",
      "USER_CUSTOM",
    ],
    `${path}.provider`,
  );
  requireString(value.assetId, `${path}.assetId`);
  requireEnum(
    value.ownership,
    ["USER_OWNED", "ENGINE_INHERITED", "PROJECT_GENERATED"],
    `${path}.ownership`,
  );
}

function assertExactObject(
  value: unknown,
  keys: readonly string[],
  path: string,
): asserts value is Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${path} must be an object`);
  }
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (
    actual.length !== expected.length
    || actual.some((key, index) => key !== expected[index])
  ) {
    throw new Error(`${path} has missing or unknown fields`);
  }
}

function requireArray(value: unknown, path: string): unknown[] {
  if (!Array.isArray(value)) throw new Error(`${path} must be an array`);
  return value;
}

function requireString(value: unknown, path: string): asserts value is string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${path} must be a non-empty string`);
  }
}

function requireEnum<const T extends string>(
  value: unknown,
  allowed: readonly T[],
  path: string,
): asserts value is T {
  if (typeof value !== "string" || !allowed.includes(value as T)) {
    throw new Error(`${path} must be one of ${allowed.join(", ")}`);
  }
}

function requireDirectCreatureBaseSlot(value: unknown, path: string) {
  requireEnum(value, FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1, path);
}

function requireNullableString(
  value: unknown,
  path: string,
): asserts value is string | null {
  if (value !== null && typeof value !== "string") {
    throw new Error(`${path} must be string or null`);
  }
}

function requireSha256(value: unknown, path: string): asserts value is string {
  if (typeof value !== "string" || !/^[0-9a-f]{64}$/i.test(value)) {
    throw new Error(`${path} must be a SHA-256`);
  }
}

function requirePositiveInteger(value: unknown, path: string) {
  if (!Number.isSafeInteger(value) || Number(value) < 1) {
    throw new Error(`${path} must be a positive integer`);
  }
}

function storageDiagnostic(
  operation: string,
  error: unknown,
): AnimationStudioDiagnosticV1 {
  return {
    schemaVersion: 1,
    code: `M2A-ANIMATION-STUDIO-MAPPING-V2-${operation}`,
    path: "creatureAnimationAuthoringV2",
    level: "BLOCKING",
    message: error instanceof Error ? error.message : String(error),
    action: "Keep the current tab open and retry local mapping persistence.",
  };
}
