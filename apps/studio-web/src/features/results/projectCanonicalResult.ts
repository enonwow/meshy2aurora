import type { WorkerArtifact } from "../../worker/types";
import { FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 } from "../source/directCreatureAnimationProfile";

export interface CanonicalResultSnapshot {
  status: string;
  sourceMetrics: CanonicalModelMetrics;
  convertedMetrics: CanonicalModelMetrics;
  geometry: { vertices: number; triangles: number; joints: number; deformation: string };
  animation: { sourceName: string; outputName: string; durationSeconds: number; hasMotion: boolean };
  texture: { width: number; height: number; pixelFormat: string; byteLength: number };
  resrefs: { model: string; texture: string };
  appearance: { appendedRow: number; sourcePrefixPreserved: boolean; policy: string };
  hak: { byteLength: number; sha256: string; entryCount: number };
  outputs: Record<string, { byteLength: number; sha256: string }>;
  resources: Array<{ role: string; resref: string; type: number; byteLength: number; sha256: string }>;
  semanticEvidence: {
    semanticDiff: string[];
    deviations: Array<{ code: string; path: string; message: string }>;
  };
  conversionEvidence: CanonicalConversionEvidence;
  packageAssemblyEvidence: {
    strictReconciled: true;
    resourceCount: number;
    artifactCount: number;
  };
  animationEventEvidence?: CanonicalAnimationEventEvidence;
  animationMappingEvidence?: CanonicalAnimationMappingEvidenceV1;
  animationStudioEvidence?: CanonicalAnimationStudioEvidenceV1;
  runtimeFixtureContract?: CanonicalM0RuntimeFixtureContract;
  artifacts: WorkerArtifact[];
  reportJson: string;
  summaryJson: string;
  manifestJson: string;
}

export interface CanonicalConversionGate {
  schemaVersion: 1;
  code: string;
  severity: string;
  path: string;
  expected: string;
  actual: string;
  message: string;
}

export interface CanonicalConversionDiagnostic {
  schemaVersion: 1;
  code: string;
  severity: string;
  path: string;
  message: string;
}

export interface CanonicalConversionEvidence {
  schemaVersion: 1;
  conversionEligible: boolean;
  policies: {
    engineFacingProof: string;
    uvRuntimeProof: string;
  };
  gates: CanonicalConversionGate[];
  diagnostics: CanonicalConversionDiagnostic[];
}

export interface CanonicalModelMetrics {
  nodes: number;
  meshes: number;
  vertices: number;
  triangles: number;
  animations: number;
}

export interface CanonicalAnimationEventEvidence {
  schemaVersion: 1;
  profile: string;
  requiredPairCount: number;
  satisfiedPairCount: number;
  totalEventCount: number;
  unknownEventNames: string[];
  missingPairs: string[];
  complete: true;
  authoringCanonical: { byteLength: number; sha256: string };
}

export interface CanonicalAnimationMappingEvidenceV1 {
  schemaVersion: 1;
  profile: string;
  sourceRevision: string;
  authoringRevision: number;
  authoringFingerprintSha256: string;
  baseAnimations: Array<{
    targetSlot: string;
    resolvedSourceSlot: string;
    sourceKind: string;
    sourceClipName: string | null;
    customAnimationId: string | null;
    provider: string;
    assetId: string;
    ownership: string;
    viaFallbackSlots: string[];
  }>;
  customAnimations: Array<{
    id: string;
    playback: "ONE_SHOT" | "LOOPING_PHASED";
    phases: Array<"START" | "LOOP" | "END">;
    outputClipNames: string[];
    sourceClipNames: string[];
    provider: string;
    assetId: string;
    ownership: string;
  }>;
  conformance: {
    status: "READY";
    expectedBaseSlotCount: 42;
    materializedBaseSlotCount: 42;
    expectedCustomClipNames: string[];
    materializedCustomClipNames: string[];
  };
}

export interface CanonicalAnimationStudioUsageV1 {
  authoredClipId: string;
  outputClipName: string;
  usageKind: "BASE_SLOT" | "CUSTOM_ONE_SHOT" | "CUSTOM_PHASE";
  baseSlot: string | null;
  customAnimationId: string | null;
  phase: "START" | "LOOP" | "END" | null;
}

export interface CanonicalAnimationStudioEvidenceV1 {
  animationStudioSchemaVersion: 1;
  animationStudioFingerprintSha256: string;
  animationStudioRevision: number;
  creatureAnimationAuthoringSchemaVersion: 2;
  creatureAnimationAuthoringFingerprintSha256: string;
  authoredClipCount: number;
  authoredClipIds: string[];
  authoredClipOutputNames: string[];
  authoredEventCount: number;
  customAssignmentCount: number;
  sourceRevision: string;
  readbackStatus: "MATCH";
  animationStudioReadback: CanonicalAnimationStudioReadbackV1;
  sourceGlbUnchanged: true;
  authoredClips: Array<{
    id: string;
    outputName: string;
    kind: "MOTION" | "STATIC_POSE";
    status: "VALID";
    revision: number;
    source: {
      kind: "BLANK_POSE" | "SOURCE_CLIP_COPY" | "PROCEDURAL_TEMPLATE";
      sourceRevision: string;
      sourceClipName: string | null;
      sourceClipFingerprint: string | null;
      proceduralTemplate: string | null;
    };
    keyframeCount: number;
    eventCount: number;
    usages: CanonicalAnimationStudioUsageV1[];
  }>;
}

export interface CanonicalAnimationStudioReadbackV1 {
  schemaVersion: 1;
  studioFingerprint: string;
  sourceRevision: string;
  status: "MATCH";
  clips: Array<{
    authoredClipId: string;
    outputClipName: string;
    materializedFingerprint: string;
  }>;
  diagnostics: [];
}

export interface CanonicalM0RuntimeResource {
  resref: string;
  byteLength: number;
  sha256: string;
}

export interface CanonicalM0RuntimeFixtureContract {
  schemaVersion: 1;
  lane: "M0_BINARY_VERTICAL_SLICE";
  module: CanonicalM0RuntimeResource;
  hak: CanonicalM0RuntimeResource;
  model: CanonicalM0RuntimeResource;
  texture: CanonicalM0RuntimeResource;
  appearanceTwoDa: CanonicalM0RuntimeResource;
  appearance: { physicalRow: number; label: string; modelType: string; race: string };
  binaryScene: {
    moduleResref: string;
    areaResref: string;
    orderedHakResrefs: string[];
    entryPosition: { x: number; y: number; z: number };
    entryDirection: { x: number; y: number };
    fixture: {
      templateResref: string;
      appearanceRow: number;
      position: { x: number; y: number; z: number };
      orientation: { x: number; y: number };
    };
  };
  meshEligibility: {
    eligible: boolean;
    checkedMeshCount: number;
    eligibleMeshCount: number;
    meshTypes: number[];
    textureResrefs: string[];
    rule: string;
  };
}

type JsonRecord = Record<string, unknown>;

const fail = (path: string): never => { throw new Error(`Canonical result field ${path} is missing or has the wrong type`); };
const record = (value: unknown, path: string): JsonRecord => value !== null && typeof value === "object" && !Array.isArray(value) ? value as JsonRecord : fail(path);
const array = (value: unknown, path: string): unknown[] => Array.isArray(value) ? value : fail(path);
const string = (value: unknown, path: string): string => typeof value === "string" && value.length > 0 ? value : fail(path);
const boolean = (value: unknown, path: string): boolean => typeof value === "boolean" ? value : fail(path);
const number = (value: unknown, path: string): number => typeof value === "number" && Number.isFinite(value) ? value : fail(path);
const integer = (value: unknown, path: string): number => Number.isSafeInteger(value) && (value as number) >= 0 ? value as number : fail(path);
const stringArray = (value: unknown, path: string): string[] => array(value, path).map((entry, index) => string(entry, `${path}[${index}]`));
const sha256 = (value: unknown, path: string): string => {
  const result = string(value, path);
  return /^[0-9a-f]{64}$/.test(result) ? result : fail(path);
};

function identity(value: unknown, path: string) {
  const item = record(value, path);
  return { byteLength: integer(item.byteLength, `${path}.byteLength`), sha256: sha256(item.sha256, `${path}.sha256`) };
}

function parseJson(json: string, path: string) {
  try { return record(JSON.parse(json), path); } catch { return fail(path); }
}

function runtimeResource(value: unknown, path: string): CanonicalM0RuntimeResource {
  const item = record(value, path);
  return { resref: string(item.resref, `${path}.resref`), ...identity(item, path) };
}

function position3(value: unknown, path: string) {
  const item = record(value, path);
  return {
    x: number(item.x, `${path}.x`),
    y: number(item.y, `${path}.y`),
    z: number(item.z, `${path}.z`),
  };
}

function direction2(value: unknown, path: string) {
  const item = record(value, path);
  return { x: number(item.x, `${path}.x`), y: number(item.y, `${path}.y`) };
}

function runtimeFixtureContractParser(value: unknown, path: string): CanonicalM0RuntimeFixtureContract {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  if (string(item.lane, `${path}.lane`) !== "M0_BINARY_VERTICAL_SLICE") fail(`${path}.lane`);
  const appearance = record(item.appearance, `${path}.appearance`);
  const binaryScene = record(item.binaryScene, `${path}.binaryScene`);
  const fixture = record(binaryScene.fixture, `${path}.binaryScene.fixture`);
  const meshEligibility = record(item.meshEligibility, `${path}.meshEligibility`);
  const contract: CanonicalM0RuntimeFixtureContract = {
    schemaVersion: 1,
    lane: "M0_BINARY_VERTICAL_SLICE",
    module: runtimeResource(item.module, `${path}.module`),
    hak: runtimeResource(item.hak, `${path}.hak`),
    model: runtimeResource(item.model, `${path}.model`),
    texture: runtimeResource(item.texture, `${path}.texture`),
    appearanceTwoDa: runtimeResource(item.appearanceTwoDa, `${path}.appearanceTwoDa`),
    appearance: {
      physicalRow: integer(appearance.physicalRow, `${path}.appearance.physicalRow`),
      label: string(appearance.label, `${path}.appearance.label`),
      modelType: string(appearance.modelType, `${path}.appearance.modelType`),
      race: string(appearance.race, `${path}.appearance.race`),
    },
    binaryScene: {
      moduleResref: string(binaryScene.moduleResref, `${path}.binaryScene.moduleResref`),
      areaResref: string(binaryScene.areaResref, `${path}.binaryScene.areaResref`),
      orderedHakResrefs: stringArray(binaryScene.orderedHakResrefs, `${path}.binaryScene.orderedHakResrefs`),
      entryPosition: position3(binaryScene.entryPosition, `${path}.binaryScene.entryPosition`),
      entryDirection: direction2(binaryScene.entryDirection, `${path}.binaryScene.entryDirection`),
      fixture: {
        templateResref: string(fixture.templateResref, `${path}.binaryScene.fixture.templateResref`),
        appearanceRow: integer(fixture.appearanceRow, `${path}.binaryScene.fixture.appearanceRow`),
        position: position3(fixture.position, `${path}.binaryScene.fixture.position`),
        orientation: direction2(fixture.orientation, `${path}.binaryScene.fixture.orientation`),
      },
    },
    meshEligibility: {
      eligible: boolean(meshEligibility.eligible, `${path}.meshEligibility.eligible`),
      checkedMeshCount: integer(meshEligibility.checkedMeshCount, `${path}.meshEligibility.checkedMeshCount`),
      eligibleMeshCount: integer(meshEligibility.eligibleMeshCount, `${path}.meshEligibility.eligibleMeshCount`),
      meshTypes: array(meshEligibility.meshTypes, `${path}.meshEligibility.meshTypes`).map((entry, index) => integer(entry, `${path}.meshEligibility.meshTypes[${index}]`)),
      textureResrefs: stringArray(meshEligibility.textureResrefs, `${path}.meshEligibility.textureResrefs`),
      rule: string(meshEligibility.rule, `${path}.meshEligibility.rule`),
    },
  };
  if (contract.appearance.physicalRow !== contract.binaryScene.fixture.appearanceRow) {
    throw new Error(`Canonical result identity mismatch at ${path}.appearance.physicalRow`);
  }
  if (contract.module.resref !== contract.binaryScene.moduleResref) {
    throw new Error(`Canonical result identity mismatch at ${path}.module.resref`);
  }
  if (contract.binaryScene.orderedHakResrefs.length !== 1
    || contract.hak.resref !== contract.binaryScene.orderedHakResrefs[0]) {
    throw new Error(`Canonical result identity mismatch at ${path}.binaryScene.orderedHakResrefs`);
  }
  if (contract.meshEligibility.eligibleMeshCount > contract.meshEligibility.checkedMeshCount) {
    throw new Error(`Canonical result identity mismatch at ${path}.meshEligibility.eligibleMeshCount`);
  }
  return contract;
}

function equal(actual: unknown, expected: unknown, path: string) {
  if (actual !== expected) throw new Error(`Canonical result identity mismatch at ${path}`);
}

function conversionGate(value: unknown, path: string): CanonicalConversionGate {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  return {
    schemaVersion: 1,
    code: string(item.code, `${path}.code`),
    severity: string(item.severity, `${path}.severity`),
    path: string(item.path, `${path}.path`),
    expected: string(item.expected, `${path}.expected`),
    actual: string(item.actual, `${path}.actual`),
    message: string(item.message, `${path}.message`),
  };
}

function conversionDiagnostic(value: unknown, path: string): CanonicalConversionDiagnostic {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  return {
    schemaVersion: 1,
    code: string(item.code, `${path}.code`),
    severity: string(item.severity, `${path}.severity`),
    path: string(item.path, `${path}.path`),
    message: string(item.message, `${path}.message`),
  };
}

function nullableString(value: unknown, path: string): string | null {
  return value === null ? null : string(value, path);
}

function animationProvenance(value: unknown, path: string) {
  const item = record(value, path);
  return {
    provider: string(item.provider, `${path}.provider`),
    assetId: string(item.assetId, `${path}.assetId`),
    ownership: string(item.ownership, `${path}.ownership`),
  };
}

function animationMappingEvidenceParser(
  authoringValue: unknown,
  conformanceValue: unknown,
  path: string,
): CanonicalAnimationMappingEvidenceV1 {
  const authoring = record(authoringValue, `${path}.animationAuthoring`);
  const conformance = record(conformanceValue, `${path}.authoredAnimationConformance`);
  if (integer(authoring.schemaVersion, `${path}.animationAuthoring.schemaVersion`) !== 1) {
    fail(`${path}.animationAuthoring.schemaVersion`);
  }
  if (integer(conformance.schemaVersion, `${path}.authoredAnimationConformance.schemaVersion`) !== 1) {
    fail(`${path}.authoredAnimationConformance.schemaVersion`);
  }
  const baseAnimations = array(
    authoring.baseAnimations,
    `${path}.animationAuthoring.baseAnimations`,
  ).map((value, index) => {
    const itemPath = `${path}.animationAuthoring.baseAnimations[${index}]`;
    const item = record(value, itemPath);
    const assignment = record(item.assignment, `${itemPath}.assignment`);
    const targetSlot = string(item.targetSlot, `${itemPath}.targetSlot`);
    const assignmentTargetSlot = string(
      assignment.targetSlot,
      `${itemPath}.assignment.targetSlot`,
    );
    if (assignmentTargetSlot !== targetSlot) {
      fail(`${itemPath}.assignment.targetSlot`);
    }
    return {
      targetSlot,
      resolvedSourceSlot: string(
        item.resolvedSourceSlot,
        `${itemPath}.resolvedSourceSlot`,
      ),
      sourceKind: string(assignment.sourceKind, `${itemPath}.assignment.sourceKind`),
      sourceClipName: nullableString(
        assignment.sourceClipName,
        `${itemPath}.assignment.sourceClipName`,
      ),
      customAnimationId: nullableString(
        assignment.customAnimationId,
        `${itemPath}.assignment.customAnimationId`,
      ),
      ...animationProvenance(
        assignment.provenance,
        `${itemPath}.assignment.provenance`,
      ),
      viaFallbackSlots: stringArray(
        item.viaFallbackSlots,
        `${itemPath}.viaFallbackSlots`,
      ),
    };
  });
  const canonicalSlots = new Set<string>(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1);
  if (
    baseAnimations.length !== FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.length
    || baseAnimations.some(({ targetSlot, resolvedSourceSlot, viaFallbackSlots }, index) => (
      targetSlot !== FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1[index]
      || !canonicalSlots.has(resolvedSourceSlot)
      || viaFallbackSlots.some((slot) => !canonicalSlots.has(slot))
    ))
  ) {
    fail(`${path}.animationAuthoring.baseAnimations`);
  }
  const customAnimations = array(
    authoring.customAnimations,
    `${path}.animationAuthoring.customAnimations`,
  ).map((value, index) => {
    const itemPath = `${path}.animationAuthoring.customAnimations[${index}]`;
    const item = record(value, itemPath);
    const playbackValue = string(item.playback, `${itemPath}.playback`);
    if (playbackValue !== "ONE_SHOT" && playbackValue !== "LOOPING_PHASED") {
      fail(`${itemPath}.playback`);
    }
    const playback = playbackValue as "ONE_SHOT" | "LOOPING_PHASED";
    const phases = stringArray(item.phases, `${itemPath}.phases`);
    if (
      phases.some((phase) => !["START", "LOOP", "END"].includes(phase))
      || new Set(phases).size !== phases.length
      || (
        playback === "ONE_SHOT"
          ? phases.length !== 0
          : phases.filter((phase) => phase === "LOOP").length !== 1
      )
    ) {
      fail(`${itemPath}.phases`);
    }
    const outputClipNames = stringArray(
      item.outputClipNames,
      `${itemPath}.outputClipNames`,
    );
    const sourceClipNames = stringArray(
      item.sourceClipNames,
      `${itemPath}.sourceClipNames`,
    );
    if (
      outputClipNames.length !== sourceClipNames.length
      || outputClipNames.length === 0
      || (
        playback === "ONE_SHOT"
          ? outputClipNames.length !== 1
          : outputClipNames.length !== phases.length
      )
    ) {
      fail(`${itemPath}.outputClipNames`);
    }
    return {
      id: string(item.id, `${itemPath}.id`),
      playback,
      phases: phases as Array<"START" | "LOOP" | "END">,
      outputClipNames,
      sourceClipNames,
      ...animationProvenance(item.provenance, `${itemPath}.provenance`),
    };
  });
  const status = string(
    conformance.status,
    `${path}.authoredAnimationConformance.status`,
  );
  const expectedBaseSlotCount = integer(
    conformance.expectedBaseSlotCount,
    `${path}.authoredAnimationConformance.expectedBaseSlotCount`,
  );
  const materializedBaseSlotCount = integer(
    conformance.materializedBaseSlotCount,
    `${path}.authoredAnimationConformance.materializedBaseSlotCount`,
  );
  const conformanceDiagnostics = array(
    conformance.diagnostics,
    `${path}.authoredAnimationConformance.diagnostics`,
  );
  if (
    status !== "READY"
    || expectedBaseSlotCount !== 42
    || materializedBaseSlotCount !== 42
    || conformanceDiagnostics.length !== 0
  ) {
    fail(`${path}.authoredAnimationConformance`);
  }
  const expectedCustomClipNames = stringArray(
    conformance.expectedCustomClipNames,
    `${path}.authoredAnimationConformance.expectedCustomClipNames`,
  );
  const materializedCustomClipNames = stringArray(
    conformance.materializedCustomClipNames,
    `${path}.authoredAnimationConformance.materializedCustomClipNames`,
  );
  const declaredCustomClipNames = customAnimations.flatMap(
    ({ outputClipNames }) => outputClipNames,
  );
  const allOutputNames = [
    ...FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    ...declaredCustomClipNames,
  ].map((name) => name.toLowerCase());
  if (
    JSON.stringify(expectedCustomClipNames) !== JSON.stringify(declaredCustomClipNames)
    || JSON.stringify(materializedCustomClipNames) !== JSON.stringify(expectedCustomClipNames)
    || new Set(allOutputNames).size !== allOutputNames.length
  ) {
    fail(`${path}.authoredAnimationConformance.materializedCustomClipNames`);
  }
  return {
    schemaVersion: 1,
    profile: string(authoring.profile, `${path}.animationAuthoring.profile`),
    sourceRevision: string(
      authoring.sourceRevision,
      `${path}.animationAuthoring.sourceRevision`,
    ),
    authoringRevision: integer(
      authoring.authoringRevision,
      `${path}.animationAuthoring.authoringRevision`,
    ),
    authoringFingerprintSha256: sha256(
      authoring.authoringFingerprintSha256,
      `${path}.animationAuthoring.authoringFingerprintSha256`,
    ),
    baseAnimations,
    customAnimations,
    conformance: {
      status: "READY",
      expectedBaseSlotCount: 42,
      materializedBaseSlotCount: 42,
      expectedCustomClipNames,
      materializedCustomClipNames,
    },
  };
}

function animationStudioEvidenceParser(
  value: unknown,
  path: string,
): CanonicalAnimationStudioEvidenceV1 {
  const item = record(value, path);
  if (
    integer(
      item.animationStudioSchemaVersion,
      `${path}.animationStudioSchemaVersion`,
    ) !== 1
  ) fail(`${path}.animationStudioSchemaVersion`);
  if (
    integer(
      item.creatureAnimationAuthoringSchemaVersion,
      `${path}.creatureAnimationAuthoringSchemaVersion`,
    ) !== 2
  ) fail(`${path}.creatureAnimationAuthoringSchemaVersion`);
  const authoredClipIds = stringArray(
    item.authoredClipIds,
    `${path}.authoredClipIds`,
  );
  const authoredClipOutputNames = stringArray(
    item.authoredClipOutputNames,
    `${path}.authoredClipOutputNames`,
  );
  const authoredClips = array(
    item.authoredClips,
    `${path}.authoredClips`,
  ).map((value, index) => {
    const clipPath = `${path}.authoredClips[${index}]`;
    const clip = record(value, clipPath);
    const source = record(clip.source, `${clipPath}.source`);
    const kindValue = string(clip.kind, `${clipPath}.kind`);
    if (kindValue !== "MOTION" && kindValue !== "STATIC_POSE") {
      fail(`${clipPath}.kind`);
    }
    const kind = kindValue as "MOTION" | "STATIC_POSE";
    if (string(clip.status, `${clipPath}.status`) !== "VALID") {
      fail(`${clipPath}.status`);
    }
    const sourceKindValue = string(source.kind, `${clipPath}.source.kind`);
    if (
      sourceKindValue !== "BLANK_POSE"
      && sourceKindValue !== "SOURCE_CLIP_COPY"
      && sourceKindValue !== "PROCEDURAL_TEMPLATE"
    ) fail(`${clipPath}.source.kind`);
    const sourceKind = sourceKindValue as
      | "BLANK_POSE"
      | "SOURCE_CLIP_COPY"
      | "PROCEDURAL_TEMPLATE";
    const usages = array(clip.usages, `${clipPath}.usages`).map(
      (usageValue, usageIndex): CanonicalAnimationStudioUsageV1 => {
        const usagePath = `${clipPath}.usages[${usageIndex}]`;
        const usage = record(usageValue, usagePath);
        const usageKindValue = string(usage.usageKind, `${usagePath}.usageKind`);
        if (
          usageKindValue !== "BASE_SLOT"
          && usageKindValue !== "CUSTOM_ONE_SHOT"
          && usageKindValue !== "CUSTOM_PHASE"
        ) fail(`${usagePath}.usageKind`);
        const usageKind = usageKindValue as
          | "BASE_SLOT"
          | "CUSTOM_ONE_SHOT"
          | "CUSTOM_PHASE";
        const phaseCandidate = usage.phase === null
          ? null
          : string(usage.phase, `${usagePath}.phase`);
        if (
          phaseCandidate !== null
          && phaseCandidate !== "START"
          && phaseCandidate !== "LOOP"
          && phaseCandidate !== "END"
        ) fail(`${usagePath}.phase`);
        const phaseValue = phaseCandidate as "START" | "LOOP" | "END" | null;
        return {
          authoredClipId: string(
            usage.authoredClipId,
            `${usagePath}.authoredClipId`,
          ),
          outputClipName: string(
            usage.outputClipName,
            `${usagePath}.outputClipName`,
          ),
          usageKind,
          baseSlot: nullableString(usage.baseSlot, `${usagePath}.baseSlot`),
          customAnimationId: nullableString(
            usage.customAnimationId,
            `${usagePath}.customAnimationId`,
          ),
          phase: phaseValue,
        };
      },
    );
    return {
      id: string(clip.id, `${clipPath}.id`),
      outputName: string(clip.outputName, `${clipPath}.outputName`),
      kind,
      status: "VALID" as const,
      revision: integer(clip.revision, `${clipPath}.revision`),
      source: {
        kind: sourceKind,
        sourceRevision: sha256(
          source.sourceRevision,
          `${clipPath}.source.sourceRevision`,
        ),
        sourceClipName: nullableString(
          source.sourceClipName,
          `${clipPath}.source.sourceClipName`,
        ),
        sourceClipFingerprint: nullableString(
          source.sourceClipFingerprint,
          `${clipPath}.source.sourceClipFingerprint`,
        ),
        proceduralTemplate: nullableString(
          source.proceduralTemplate,
          `${clipPath}.source.proceduralTemplate`,
        ),
      },
      keyframeCount: integer(clip.keyframeCount, `${clipPath}.keyframeCount`),
      eventCount: integer(clip.eventCount, `${clipPath}.eventCount`),
      usages,
    };
  });
  const authoredClipCount = integer(
    item.authoredClipCount,
    `${path}.authoredClipCount`,
  );
  const authoredEventCount = integer(
    item.authoredEventCount,
    `${path}.authoredEventCount`,
  );
  if (
    authoredClipCount !== authoredClips.length
    || authoredClipIds.length !== authoredClips.length
    || authoredClipOutputNames.length !== authoredClips.length
    || authoredClips.some((clip, index) => (
      clip.id !== authoredClipIds[index]
      || clip.outputName !== authoredClipOutputNames[index]
    ))
    || authoredClips.reduce((total, clip) => total + clip.eventCount, 0)
      !== authoredEventCount
  ) {
    fail(`${path}.authoredClips`);
  }
  if (string(item.readbackStatus, `${path}.readbackStatus`) !== "MATCH") {
    fail(`${path}.readbackStatus`);
  }
  if (!boolean(item.sourceGlbUnchanged, `${path}.sourceGlbUnchanged`)) {
    fail(`${path}.sourceGlbUnchanged`);
  }
  const sourceRevision = sha256(item.sourceRevision, `${path}.sourceRevision`);
  if (authoredClips.some((clip) => clip.source.sourceRevision !== sourceRevision)) {
    fail(`${path}.authoredClips.sourceRevision`);
  }
  const animationStudioFingerprintSha256 = sha256(
    item.animationStudioFingerprintSha256,
    `${path}.animationStudioFingerprintSha256`,
  );
  const readbackPath = `${path}.animationStudioReadback`;
  const readbackItem = record(item.animationStudioReadback, readbackPath);
  if (
    integer(readbackItem.schemaVersion, `${readbackPath}.schemaVersion`) !== 1
    || string(readbackItem.status, `${readbackPath}.status`) !== "MATCH"
  ) {
    fail(readbackPath);
  }
  const readbackDiagnostics = array(
    readbackItem.diagnostics,
    `${readbackPath}.diagnostics`,
  );
  if (readbackDiagnostics.length !== 0) {
    fail(`${readbackPath}.diagnostics`);
  }
  const readbackClips = array(
    readbackItem.clips,
    `${readbackPath}.clips`,
  ).map((value, index) => {
    const clipPath = `${readbackPath}.clips[${index}]`;
    const clip = record(value, clipPath);
    return {
      authoredClipId: string(
        clip.authoredClipId,
        `${clipPath}.authoredClipId`,
      ),
      outputClipName: string(
        clip.outputClipName,
        `${clipPath}.outputClipName`,
      ),
      materializedFingerprint: sha256(
        clip.materializedFingerprint,
        `${clipPath}.materializedFingerprint`,
      ),
    };
  });
  const expectedUsageKeys = authoredClips
    .flatMap(({ usages }) => usages)
    .map(({ authoredClipId, outputClipName }) => (
      `${authoredClipId}\0${outputClipName}`
    ))
    .sort();
  const readbackUsageKeys = readbackClips
    .map(({ authoredClipId, outputClipName }) => (
      `${authoredClipId}\0${outputClipName}`
    ))
    .sort();
  if (
    sha256(
      readbackItem.studioFingerprint,
      `${readbackPath}.studioFingerprint`,
    ) !== animationStudioFingerprintSha256
    || sha256(
      readbackItem.sourceRevision,
      `${readbackPath}.sourceRevision`,
    ) !== sourceRevision
    || JSON.stringify(readbackUsageKeys) !== JSON.stringify(expectedUsageKeys)
  ) {
    fail(readbackPath);
  }
  const animationStudioReadback: CanonicalAnimationStudioReadbackV1 = {
    schemaVersion: 1,
    studioFingerprint: animationStudioFingerprintSha256,
    sourceRevision,
    status: "MATCH",
    clips: readbackClips,
    diagnostics: [],
  };
  return {
    animationStudioSchemaVersion: 1,
    animationStudioFingerprintSha256,
    animationStudioRevision: integer(
      item.animationStudioRevision,
      `${path}.animationStudioRevision`,
    ),
    creatureAnimationAuthoringSchemaVersion: 2,
    creatureAnimationAuthoringFingerprintSha256: sha256(
      item.creatureAnimationAuthoringFingerprintSha256,
      `${path}.creatureAnimationAuthoringFingerprintSha256`,
    ),
    authoredClipCount,
    authoredClipIds,
    authoredClipOutputNames,
    authoredEventCount,
    customAssignmentCount: integer(
      item.customAssignmentCount,
      `${path}.customAssignmentCount`,
    ),
    sourceRevision,
    readbackStatus: "MATCH",
    animationStudioReadback,
    sourceGlbUnchanged: true,
    authoredClips,
  };
}

export function projectCanonicalResult(
  reportJson: string,
  summaryJson: string,
  manifestJson: string,
  artifacts: readonly WorkerArtifact[],
): CanonicalResultSnapshot {
  const report = parseJson(reportJson, "reportJson");
  const summary = parseJson(summaryJson, "summaryJson");
  const manifest = parseJson(manifestJson, "manifestJson");
  for (const [value, path] of [[report, "report"], [summary, "summary"], [manifest, "manifest"]] as const) {
    if (integer(value.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  }
  const status = string(summary.status, "summary.status");
  if (status !== "M6_MODEL_PACKAGE_MATERIALIZED" && status !== "M0_MESHY_STATIC_RIGID_PACKAGE_MATERIALIZED") {
    fail("summary.status");
  }
  equal(string(manifest.status, "manifest.status"), status, "manifest.status");

  const ingest = record(report.ingest, "report.ingest");
  if (integer(ingest.schemaVersion, "report.ingest.schemaVersion") !== 1) fail("report.ingest.schemaVersion");
  const sourceInventory = record(ingest.inventory, "report.ingest.inventory");
  const sourceStatistics = record(ingest.statistics, "report.ingest.statistics");
  const sourceMetrics: CanonicalModelMetrics = {
    nodes: integer(sourceInventory.nodeCount, "report.ingest.inventory.nodeCount"),
    meshes: integer(sourceInventory.meshCount, "report.ingest.inventory.meshCount"),
    vertices: integer(sourceStatistics.vertexCount, "report.ingest.statistics.vertexCount"),
    triangles: integer(sourceStatistics.triangleCount, "report.ingest.statistics.triangleCount"),
    animations: integer(sourceInventory.animationCount, "report.ingest.inventory.animationCount"),
  };

  const conversion = record(report.conversion, "report.conversion");
  if (integer(conversion.schemaVersion, "report.conversion.schemaVersion") !== 1) fail("report.conversion.schemaVersion");
  const conversionPolicies = record(conversion.policies, "report.conversion.policies");
  const conversionEvidence: CanonicalConversionEvidence = {
    schemaVersion: 1,
    conversionEligible: boolean(conversion.conversionEligible, "report.conversion.conversionEligible"),
    policies: {
      engineFacingProof: string(conversionPolicies.engineFacingProof, "report.conversion.policies.engineFacingProof"),
      uvRuntimeProof: string(conversionPolicies.uvRuntimeProof, "report.conversion.policies.uvRuntimeProof"),
    },
    gates: array(conversion.gates, "report.conversion.gates").map((value, index) =>
      conversionGate(value, `report.conversion.gates[${index}]`)),
    diagnostics: array(conversion.diagnostics, "report.conversion.diagnostics").map((value, index) =>
      conversionDiagnostic(value, `report.conversion.diagnostics[${index}]`)),
  };

  const geometryJson = record(report.geometry, "report.geometry");
  const geometry = {
    vertices: integer(geometryJson.vertexCount, "report.geometry.vertexCount"),
    triangles: integer(geometryJson.triangleCount, "report.geometry.triangleCount"),
    joints: integer(geometryJson.activeJointCount, "report.geometry.activeJointCount"),
    deformation: string(geometryJson.outputSegmentDeformation, "report.geometry.outputSegmentDeformation"),
  };
  const animationJson = record(summary.animation, "summary.animation");
  const animation = {
    sourceName: string(animationJson.sourceName, "summary.animation.sourceName"),
    outputName: string(animationJson.outputName, "summary.animation.outputName"),
    durationSeconds: number(animationJson.durationSeconds, "summary.animation.durationSeconds"),
    hasMotion: boolean(animationJson.hasMotion, "summary.animation.hasMotion"),
  };
  const textureJson = record(report.texture, "report.texture");
  const texture = {
    width: integer(textureJson.width, "report.texture.width"),
    height: integer(textureJson.height, "report.texture.height"),
    pixelFormat: string(textureJson.pixelFormat, "report.texture.pixelFormat"),
    byteLength: integer(textureJson.byteLength, "report.texture.byteLength"),
  };
  const appearanceJson = record(report.appearance, "report.appearance");
  const appendedRow = integer(appearanceJson.appendedRowIndex, "report.appearance.appendedRowIndex");
  equal(integer(summary.appendedPhysicalRow, "summary.appendedPhysicalRow"), appendedRow, "summary.appendedPhysicalRow");
  equal(integer(manifest.appendedPhysicalRow, "manifest.appendedPhysicalRow"), appendedRow, "manifest.appendedPhysicalRow");
  const policy = string(summary.appearancePayloadPolicy, "summary.appearancePayloadPolicy");
  equal(string(manifest.appearancePayloadPolicy, "manifest.appearancePayloadPolicy"), policy, "manifest.appearancePayloadPolicy");

  const outputJson = record(summary.outputs, "summary.outputs");
  const outputs = Object.fromEntries(["model", "texture", "appearanceTwoDa", "hak", "proofModule", "report"].map((name) => [name, identity(outputJson[name], `summary.outputs.${name}`)]));
  const hakJson = record(report.hak, "report.hak");
  const hak = {
    byteLength: integer(hakJson.byteLength, "report.hak.byteLength"),
    sha256: sha256(hakJson.archiveSha256, "report.hak.archiveSha256"),
    entryCount: integer(hakJson.entryCount, "report.hak.entryCount"),
  };
  equal(outputs.hak.byteLength, hak.byteLength, "summary.outputs.hak.byteLength");
  equal(outputs.hak.sha256, hak.sha256, "summary.outputs.hak.sha256");
  equal(texture.byteLength, outputs.texture.byteLength, "report.texture.byteLength");
  equal(sha256(textureJson.outputSha256, "report.texture.outputSha256"), outputs.texture.sha256, "report.texture.outputSha256");
  const modelJson = record(report.model, "report.model");
  const projection = record(modelJson.projection, "report.model.projection");
  const convertedMetrics: CanonicalModelMetrics = {
    nodes: integer(projection.rigNodeCount, "report.model.projection.rigNodeCount")
      + integer(projection.meshNodeCount, "report.model.projection.meshNodeCount"),
    meshes: integer(projection.meshNodeCount, "report.model.projection.meshNodeCount"),
    vertices: geometry.vertices,
    triangles: integer(projection.triangleCount, "report.model.projection.triangleCount"),
    animations: integer(projection.animationCount, "report.model.projection.animationCount"),
  };
  equal(convertedMetrics.triangles, geometry.triangles, "report.model.projection.triangleCount");
  const semanticDiff = stringArray(modelJson.semanticDiff, "report.model.semanticDiff");
  const deviations = array(modelJson.deviations, "report.model.deviations").map((value, index) => {
    const item = record(value, `report.model.deviations[${index}]`);
    return {
      code: string(item.code, `report.model.deviations[${index}].code`),
      path: string(item.path, `report.model.deviations[${index}].path`),
      message: string(item.message, `report.model.deviations[${index}].message`),
    };
  });
  equal(sha256(modelJson.payloadSha256, "report.model.payloadSha256"), outputs.model.sha256, "report.model.payloadSha256");
  equal(integer(record(modelJson.layout, "report.model.layout").fileLength, "report.model.layout.fileLength"), outputs.model.byteLength, "report.model.layout.fileLength");
  equal(integer(appearanceJson.outputByteLength, "report.appearance.outputByteLength"), outputs.appearanceTwoDa.byteLength, "report.appearance.outputByteLength");
  equal(sha256(appearanceJson.outputSha256, "report.appearance.outputSha256"), outputs.appearanceTwoDa.sha256, "report.appearance.outputSha256");
  const proofModuleJson = record(report.proofModule, "report.proofModule");
  equal(integer(proofModuleJson.byteLength, "report.proofModule.byteLength"), outputs.proofModule.byteLength, "report.proofModule.byteLength");
  equal(sha256(proofModuleJson.sha256, "report.proofModule.sha256"), outputs.proofModule.sha256, "report.proofModule.sha256");
  equal(integer(proofModuleJson.appearanceRow, "report.proofModule.appearanceRow"), appendedRow, "report.proofModule.appearanceRow");
  if (string(proofModuleJson.semanticReadbackStatus, "report.proofModule.semanticReadbackStatus") !== "PASS") fail("report.proofModule.semanticReadbackStatus");

  let animationEventEvidence: CanonicalAnimationEventEvidence | undefined;
  if (
    report.animationEventConformance !== undefined
    || report.animationEventAuthoringCanonical !== undefined
  ) {
    const eventConformance = record(
      report.animationEventConformance,
      "report.animationEventConformance",
    );
    if (
      integer(
        eventConformance.schemaVersion,
        "report.animationEventConformance.schemaVersion",
      ) !== 1
    ) {
      fail("report.animationEventConformance.schemaVersion");
    }
    const requiredPairCount = integer(
      eventConformance.requiredPairCount,
      "report.animationEventConformance.requiredPairCount",
    );
    const satisfiedPairCount = integer(
      eventConformance.satisfiedPairCount,
      "report.animationEventConformance.satisfiedPairCount",
    );
    const missingPairs = stringArray(
      eventConformance.missingPairs,
      "report.animationEventConformance.missingPairs",
    );
    if (
      !boolean(eventConformance.complete, "report.animationEventConformance.complete")
      || satisfiedPairCount !== requiredPairCount
      || missingPairs.length !== 0
    ) {
      throw new Error("Canonical result identity mismatch at report.animationEventConformance");
    }
    animationEventEvidence = {
      schemaVersion: 1,
      profile: string(
        eventConformance.profile,
        "report.animationEventConformance.profile",
      ),
      requiredPairCount,
      satisfiedPairCount,
      totalEventCount: integer(
        eventConformance.totalEventCount,
        "report.animationEventConformance.totalEventCount",
      ),
      unknownEventNames: stringArray(
        eventConformance.unknownEventNames,
        "report.animationEventConformance.unknownEventNames",
      ),
      missingPairs,
      complete: true,
      authoringCanonical: identity(
        report.animationEventAuthoringCanonical,
        "report.animationEventAuthoringCanonical",
      ),
    };
  }

  const packageManifest = record(manifest.packageManifest, "manifest.packageManifest");
  equal(sha256(packageManifest.packageSha256, "manifest.packageManifest.packageSha256"), hak.sha256, "manifest.packageManifest.packageSha256");
  const resources = array(packageManifest.resources, "manifest.packageManifest.resources").map((value, index) => {
    const item = record(value, `manifest.packageManifest.resources[${index}]`);
    return {
      role: string(item.role, `manifest.packageManifest.resources[${index}].role`),
      resref: string(item.resref, `manifest.packageManifest.resources[${index}].resref`),
      type: integer(item.type, `manifest.packageManifest.resources[${index}].type`),
      byteLength: integer(item.byteLength, `manifest.packageManifest.resources[${index}].byteLength`),
      sha256: sha256(item.sha256, `manifest.packageManifest.resources[${index}].sha256`),
    };
  });
  if (resources.length !== 3 || resources.length !== hak.entryCount) throw new Error("Canonical result identity mismatch at HAK resource count");
  const resourcesByRole = new Map(resources.map((resource) => [resource.role, resource]));
  if (resourcesByRole.size !== resources.length) throw new Error("Canonical result identity mismatch at duplicate resource role");
  const resource = (role: string) => resourcesByRole.get(role) ?? fail(`manifest.packageManifest.resources.${role}`);
  const modelResource = resource("MODEL");
  const textureResource = resource("TEXTURE");
  const appearanceResource = resource("APPEARANCE_TABLE");
  if (resourcesByRole.size !== 3) fail("manifest.packageManifest.resources.roles");
  const reconcile = (actual: { byteLength: number; sha256: string }, expected: { byteLength: number; sha256: string }, path: string) => {
    equal(actual.byteLength, expected.byteLength, `${path}.byteLength`);
    equal(actual.sha256, expected.sha256, `${path}.sha256`);
  };
  reconcile(modelResource, outputs.model, "manifest.packageManifest.resources.MODEL");
  reconcile(textureResource, outputs.texture, "manifest.packageManifest.resources.TEXTURE");
  reconcile(appearanceResource, outputs.appearanceTwoDa, "manifest.packageManifest.resources.APPEARANCE_TABLE");
  equal(modelResource.resref, string(summary.modelResref, "summary.modelResref"), "manifest.packageManifest.resources.MODEL.resref");
  equal(string(projection.modelResourceResref, "report.model.projection.modelResourceResref"), modelResource.resref, "report.model.projection.modelResourceResref");
  equal(textureResource.resref, string(summary.textureResref, "summary.textureResref"), "manifest.packageManifest.resources.TEXTURE.resref");
  equal(appearanceResource.resref, "appearance", "manifest.packageManifest.resources.APPEARANCE_TABLE.resref");

  let animationMappingEvidence: CanonicalAnimationMappingEvidenceV1 | undefined;
  if (
    report.animationAuthoring !== undefined
    || report.authoredAnimationConformance !== undefined
    || manifest.animationAuthoring !== undefined
    || manifest.authoredAnimationConformance !== undefined
  ) {
    const reportEvidence = animationMappingEvidenceParser(
      report.animationAuthoring,
      report.authoredAnimationConformance,
      "report",
    );
    const manifestEvidence = animationMappingEvidenceParser(
      manifest.animationAuthoring,
      manifest.authoredAnimationConformance,
      "manifest",
    );
    if (JSON.stringify(reportEvidence) !== JSON.stringify(manifestEvidence)) {
      throw new Error("Canonical result identity mismatch at animationMappingEvidence");
    }
    if (
      reportEvidence.conformance.expectedCustomClipNames.length
      !== reportEvidence.conformance.materializedCustomClipNames.length
    ) {
      fail("report.authoredAnimationConformance.materializedCustomClipNames");
    }
    animationMappingEvidence = reportEvidence;
  }

  let animationStudioEvidence: CanonicalAnimationStudioEvidenceV1 | undefined;
  if (
    report.animationStudioSchemaVersion !== undefined
    || summary.animationStudioSchemaVersion !== undefined
    || manifest.animationStudioSchemaVersion !== undefined
  ) {
    const reportEvidence = animationStudioEvidenceParser(report, "report");
    const summaryEvidence = animationStudioEvidenceParser(summary, "summary");
    const manifestEvidence = animationStudioEvidenceParser(manifest, "manifest");
    if (
      JSON.stringify(reportEvidence) !== JSON.stringify(summaryEvidence)
      || JSON.stringify(reportEvidence) !== JSON.stringify(manifestEvidence)
    ) {
      throw new Error(
        "Canonical result identity mismatch at animationStudioEvidence",
      );
    }
    if (
      reportEvidence.sourceRevision
        !== sha256(
          record(manifest.inputGlb, "manifest.inputGlb").sha256,
          "manifest.inputGlb.sha256",
        )
    ) {
      fail("animationStudioEvidence.sourceRevision");
    }
    animationStudioEvidence = reportEvidence;
  }

  let runtimeFixtureContract: CanonicalM0RuntimeFixtureContract | undefined;
  if (status === "M0_MESHY_STATIC_RIGID_PACKAGE_MATERIALIZED") {
    const reportContract = runtimeFixtureContractParser(report.m0RuntimeFixtureContract, "report.m0RuntimeFixtureContract");
    const summaryContract = runtimeFixtureContractParser(summary.m0RuntimeFixtureContract, "summary.m0RuntimeFixtureContract");
    const manifestContract = runtimeFixtureContractParser(manifest.m0RuntimeFixtureContract, "manifest.m0RuntimeFixtureContract");
    if (JSON.stringify(reportContract) !== JSON.stringify(summaryContract)
      || JSON.stringify(reportContract) !== JSON.stringify(manifestContract)) {
      throw new Error("Canonical result identity mismatch at m0RuntimeFixtureContract");
    }
    reconcile(reportContract.module, outputs.proofModule, "m0RuntimeFixtureContract.module");
    reconcile(reportContract.hak, outputs.hak, "m0RuntimeFixtureContract.hak");
    reconcile(reportContract.model, outputs.model, "m0RuntimeFixtureContract.model");
    reconcile(reportContract.texture, outputs.texture, "m0RuntimeFixtureContract.texture");
    reconcile(reportContract.appearanceTwoDa, outputs.appearanceTwoDa, "m0RuntimeFixtureContract.appearanceTwoDa");
    equal(reportContract.model.resref, modelResource.resref, "m0RuntimeFixtureContract.model.resref");
    equal(reportContract.texture.resref, textureResource.resref, "m0RuntimeFixtureContract.texture.resref");
    equal(reportContract.appearanceTwoDa.resref, appearanceResource.resref, "m0RuntimeFixtureContract.appearanceTwoDa.resref");
    equal(reportContract.appearance.physicalRow, appendedRow, "m0RuntimeFixtureContract.appearance.physicalRow");
    if (!reportContract.meshEligibility.eligible) {
      throw new Error("Canonical result identity mismatch at m0RuntimeFixtureContract.meshEligibility.eligible");
    }
    runtimeFixtureContract = reportContract;
  } else if (report.m0RuntimeFixtureContract !== undefined
    || summary.m0RuntimeFixtureContract !== undefined
    || manifest.m0RuntimeFixtureContract !== undefined) {
    fail("m0RuntimeFixtureContract");
  }

  if (artifacts.length !== 6 || new Set(artifacts.map(({ artifactId }) => artifactId)).size !== artifacts.length) {
    throw new Error("Canonical result identity mismatch at artifact inventory");
  }
  const requiredArtifacts = [
    ["package-hak", "HAK", outputs.hak],
    ["model-mdl", "MODEL", outputs.model],
    ["proof-module", "MODULE", outputs.proofModule],
    ["report-json", "JSON_REPORT", outputs.report],
  ] as const;
  for (const [artifactId, kind, expected] of requiredArtifacts) {
    const artifact = artifacts.find((item) => item.artifactId === artifactId)
      ?? fail(`artifacts.${artifactId}`);
    if (artifact.kind !== kind) fail(`artifacts.${artifactId}.kind`);
    equal(artifact.byteLength, expected.byteLength, `artifacts.${artifactId}.byteLength`);
    equal(artifact.sha256, expected.sha256, `artifacts.${artifactId}.sha256`);
  }
  for (const [artifactId, exactJson] of [
    ["report-json", reportJson],
    ["manifest-json", manifestJson],
    ["summary-json", summaryJson],
  ] as const) {
    const artifact = artifacts.find((item) => item.artifactId === artifactId)
      ?? fail(`artifacts.${artifactId}`);
    if (artifact.kind !== "JSON_REPORT") fail(`artifacts.${artifactId}.kind`);
    if (new TextDecoder().decode(artifact.bytes) !== exactJson) {
      throw new Error(`Canonical result identity mismatch at artifacts.${artifactId}.bytes`);
    }
  }
  for (const artifact of artifacts) {
    string(artifact.fileName, "artifacts.fileName");
    integer(artifact.byteLength, "artifacts.byteLength");
    sha256(artifact.sha256, "artifacts.sha256");
    if (artifact.provenance !== "M2A_WASM_WORKER") fail(`artifacts.${artifact.artifactId}.provenance`);
    if (artifact.bytes.byteLength !== artifact.byteLength) throw new Error(`Canonical result identity mismatch at artifact ${artifact.artifactId} bytes`);
  }

  return {
    status,
    sourceMetrics,
    convertedMetrics,
    geometry,
    animation,
    texture,
    resrefs: {
      model: string(summary.modelResref, "summary.modelResref"),
      texture: string(summary.textureResref, "summary.textureResref"),
    },
    appearance: {
      appendedRow,
      sourcePrefixPreserved: boolean(appearanceJson.sourcePrefixPreserved, "report.appearance.sourcePrefixPreserved"),
      policy,
    },
    hak,
    outputs,
    resources,
    semanticEvidence: { semanticDiff, deviations },
    conversionEvidence,
    packageAssemblyEvidence: {
      strictReconciled: true,
      resourceCount: resources.length,
      artifactCount: artifacts.length,
    },
    animationEventEvidence,
    animationMappingEvidence,
    animationStudioEvidence,
    runtimeFixtureContract,
    artifacts: [...artifacts],
    reportJson,
    summaryJson,
    manifestJson,
  };
}
