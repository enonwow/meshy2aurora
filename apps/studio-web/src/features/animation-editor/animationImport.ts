import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";

export interface ExternalAnimationClipV1 {
  readonly name: string;
  readonly durationSeconds: number;
  readonly trackCount: number;
}

export interface ExternalAnimationSourceInspectionV1 {
  readonly sourceRevision: string;
  readonly rig: readonly AnimationRigNodeV1[];
  readonly clips: readonly ExternalAnimationClipV1[];
  readonly transferCompatibility: AnimationTransferCompatibilityV1;
  readonly semanticCompatibility?: HumanoidSemanticBoneMapV2;
}

export type AnimationTransferCompatibilityStatusV1 =
  | "EXACT_COPY"
  | "RETARGETABLE_SAME_HIERARCHY"
  | "INCOMPATIBLE";

export type AnimationTransferModeV1 =
  | "EXACT_RIG_COPY_V1"
  | "SAME_HIERARCHY_RETARGET_V1"
  | "HUMANOID_SEMANTIC_RETARGET_V2";

export type HumanoidRetargetCompatibilityStatusV2 =
  | "COMPATIBLE"
  | "MANUAL_CONFIRMATION_REQUIRED"
  | "INCOMPATIBLE";

export interface HumanoidSemanticBoneEntryV2 {
  readonly semantic: string;
  readonly required: boolean;
  readonly donorNodeId: number;
  readonly donorNodeName: string;
  readonly targetNodeId: number;
  readonly targetNodeName: string;
  readonly mappingSource: "VERSIONED_ALIAS" | "MANUAL";
}

export interface HumanoidSemanticOverrideV2 {
  readonly semantic: string;
  readonly donorNodeId: number;
  readonly targetNodeId: number;
}

export interface HumanoidSemanticBoneMapV2 {
  readonly schemaVersion: 2;
  readonly aliasDictionaryVersion: string;
  readonly status: HumanoidRetargetCompatibilityStatusV2;
  readonly donorSourceRevision: string;
  readonly targetSourceRevision: string;
  readonly manualMappingConfirmed: boolean;
  readonly entries: readonly HumanoidSemanticBoneEntryV2[];
  readonly diagnostics: readonly {
    readonly code: string;
    readonly path: string;
    readonly message: string;
    readonly action: string;
  }[];
  readonly fingerprintSha256: string;
}

export interface AnimationTransferCompatibilityV1 {
  readonly schemaVersion: 1;
  readonly status: AnimationTransferCompatibilityStatusV1;
  readonly donorSourceRevision: string;
  readonly targetSourceRevision: string;
  readonly donorRigSignatureSha256: string;
  readonly targetRigSignatureSha256: string;
  readonly compatibilityFingerprintSha256: string;
  readonly allowedModes: readonly AnimationTransferModeV1[];
  readonly mapping: {
    readonly schemaVersion: 1;
    readonly rootName: string;
    readonly entries: readonly {
      readonly boneName: string;
      readonly parentName: string | null;
      readonly donorNodeId: number;
      readonly targetNodeId: number;
    }[];
  } | null;
  readonly diagnostics: readonly {
    readonly code: string;
    readonly path: string;
    readonly message: string;
    readonly action: string;
  }[];
}

export interface AnimationTransferBatchResultV2 {
  readonly schemaVersion: 2;
  readonly commit: "ALL_OR_NOTHING";
  readonly donorSourceRevision: string;
  readonly targetSourceRevision: string;
  readonly semanticMapFingerprintSha256: string | null;
  readonly clips: readonly {
    readonly donorClipName: string;
    readonly mode: AnimationTransferModeV1;
    readonly status: "READY";
    readonly clip: AuthoredAnimationClipV1;
  }[];
  readonly batchFingerprintSha256: string;
}

export function parseAnimationTransferBatchResultV2(json: string): AnimationTransferBatchResultV2 {
  const value = JSON.parse(json) as Partial<AnimationTransferBatchResultV2>;
  if (value.schemaVersion !== 2 || value.commit !== "ALL_OR_NOTHING"
    || !Array.isArray(value.clips) || value.clips.length === 0
    || !value.clips.every((item) => item.status === "READY" && item.clip && typeof item.clip === "object")
    || typeof value.batchFingerprintSha256 !== "string" || !/^[0-9a-f]{64}$/iu.test(value.batchFingerprintSha256)) {
    throw new Error("Core returned an invalid transactional animation batch.");
  }
  return value as AnimationTransferBatchResultV2;
}

export function parseAnimationTransferCompatibilityV1(
  json: string,
): AnimationTransferCompatibilityV1 {
  const value = JSON.parse(json) as Partial<AnimationTransferCompatibilityV1>;
  const sha = /^[0-9a-f]{64}$/i;
  if (
    value.schemaVersion !== 1
    || ![
      "EXACT_COPY",
      "RETARGETABLE_SAME_HIERARCHY",
      "INCOMPATIBLE",
    ].includes(String(value.status))
    || typeof value.donorSourceRevision !== "string"
    || !sha.test(value.donorSourceRevision)
    || typeof value.targetSourceRevision !== "string"
    || !sha.test(value.targetSourceRevision)
    || typeof value.compatibilityFingerprintSha256 !== "string"
    || !sha.test(value.compatibilityFingerprintSha256)
    || !Array.isArray(value.allowedModes)
    || !value.allowedModes.every((mode) => (
      mode === "EXACT_RIG_COPY_V1"
      || mode === "SAME_HIERARCHY_RETARGET_V1"
      || mode === "HUMANOID_SEMANTIC_RETARGET_V2"
    ))
    || !Array.isArray(value.diagnostics)
    || typeof value.donorRigSignatureSha256 !== "string"
    || !sha.test(value.donorRigSignatureSha256)
    || typeof value.targetRigSignatureSha256 !== "string"
    || !sha.test(value.targetRigSignatureSha256)
    || !value.diagnostics.every((diagnostic) => (
      diagnostic !== null
      && typeof diagnostic === "object"
      && typeof diagnostic.code === "string"
      && typeof diagnostic.path === "string"
      && typeof diagnostic.message === "string"
      && typeof diagnostic.action === "string"
    ))
  ) {
    throw new Error("Core returned an invalid animation-transfer compatibility report.");
  }
  return value as AnimationTransferCompatibilityV1;
}

export function parseHumanoidSemanticBoneMapV2(
  json: string,
): HumanoidSemanticBoneMapV2 {
  const value = JSON.parse(json) as Partial<HumanoidSemanticBoneMapV2>;
  const sha = /^[0-9a-f]{64}$/i;
  if (
    value.schemaVersion !== 2
    || typeof value.aliasDictionaryVersion !== "string"
    || value.aliasDictionaryVersion.length === 0
    || !["COMPATIBLE", "MANUAL_CONFIRMATION_REQUIRED", "INCOMPATIBLE"]
      .includes(String(value.status))
    || typeof value.donorSourceRevision !== "string"
    || !sha.test(value.donorSourceRevision)
    || typeof value.targetSourceRevision !== "string"
    || !sha.test(value.targetSourceRevision)
    || typeof value.manualMappingConfirmed !== "boolean"
    || !Array.isArray(value.entries)
    || !value.entries.every((entry) => (
      entry !== null
      && typeof entry === "object"
      && typeof entry.semantic === "string"
      && entry.semantic.length > 0
      && typeof entry.required === "boolean"
      && Number.isSafeInteger(entry.donorNodeId)
      && typeof entry.donorNodeName === "string"
      && Number.isSafeInteger(entry.targetNodeId)
      && typeof entry.targetNodeName === "string"
      && (entry.mappingSource === "VERSIONED_ALIAS" || entry.mappingSource === "MANUAL")
    ))
    || !Array.isArray(value.diagnostics)
    || !value.diagnostics.every((diagnostic) => (
      diagnostic !== null
      && typeof diagnostic === "object"
      && typeof diagnostic.code === "string"
      && typeof diagnostic.path === "string"
      && typeof diagnostic.message === "string"
      && typeof diagnostic.action === "string"
    ))
    || typeof value.fingerprintSha256 !== "string"
    || !sha.test(value.fingerprintSha256)
  ) {
    throw new Error("Core returned an invalid humanoid semantic-retarget report.");
  }
  return value as HumanoidSemanticBoneMapV2;
}

export function createImportedModelClipV1(
  projected: AuthoredAnimationClipV1,
  input: {
    readonly id: string;
    readonly name: string;
    readonly donorSourceRevision: string;
  },
): AuthoredAnimationClipV1 {
  if (
    projected.source.sourceClipName === null
    || projected.source.sourceClipFingerprint === null
  ) {
    throw new Error("The donor clip has no exact source name and fingerprint.");
  }
  return {
    ...projected,
    id: input.id,
    name: input.name,
    status: "DRAFT",
    source: {
      kind: "IMPORTED_MODEL_COPY",
      sourceRevision: input.donorSourceRevision,
      sourceClipName: projected.source.sourceClipName,
      sourceClipFingerprint: projected.source.sourceClipFingerprint,
      proceduralTemplate: null,
      retarget: undefined,
    },
    revision: 1,
  };
}
