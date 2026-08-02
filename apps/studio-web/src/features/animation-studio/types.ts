import {
  CREATURE_ANIMATION_AUTHORING_PROFILE_V1,
  type AnimationFallbackDecisionV1,
  type AnimationMappingProvenanceV1,
  type AnimationSourceAssignmentV1,
  type CustomAnimationPhaseKindV1,
  type CustomAnimationPlaybackV1,
  type DirectCreatureModelTypeV1,
} from "../animation-mapping/types";

export const ANIMATION_STUDIO_SCHEMA_VERSION_V1 = 1 as const;
export const ANIMATION_STUDIO_SCHEMA_VERSION_V2 = 2 as const;
export const ANIMATION_STUDIO_SCHEMA_VERSION_V3 = 3 as const;
/**
 * The V2 schema changes reference shape, not the audited Base-42 profile.
 * Migration therefore preserves the V1 profile literal byte-for-byte.
 */
export const CREATURE_ANIMATION_AUTHORING_PROFILE_V2 =
  CREATURE_ANIMATION_AUTHORING_PROFILE_V1;

export type AnimationStudioDocumentStatusV1 =
  | "DRAFT"
  | "VALID"
  | "INVALID";

export type AuthoredAnimationClipKindV1 = "MOTION" | "STATIC_POSE";
export type AuthoredAnimationClipStatusV1 =
  | "DRAFT"
  | "VALID"
  | "INVALID";
export type AuthoredAnimationSourceKindV1 =
  | "BLANK_POSE"
  | "SOURCE_CLIP_COPY"
  | "IMPORTED_MODEL_COPY"
  | "PROCEDURAL_TEMPLATE"
  | "LIBRARY_PRESET_COPY"
  | "RETARGETED_MODEL_COPY";
export type AuthoredAnimationTrackPathV1 = "TRANSLATION" | "ROTATION";
export type AuthoredAnimationInterpolationV1 = "LINEAR";

export interface AuthoredAnimationSourceV1 {
  kind: AuthoredAnimationSourceKindV1;
  sourceRevision: string;
  sourceClipName: string | null;
  sourceClipFingerprint: string | null;
  proceduralTemplate: string | null;
  libraryPreset?: AnimationLibraryPresetProvenanceV1;
  retarget?: AnimationRetargetProvenanceV1;
}

export interface AnimationRetargetProvenanceV1 {
  donorSourceRevision: string;
  targetSourceRevision: string;
  donorClipName: string;
  donorClipFingerprint: string;
  donorRigSignatureSha256: string;
  targetRigSignatureSha256: string;
  compatibilityFingerprintSha256: string;
  mode: "SAME_HIERARCHY_RETARGET_V1" | "HUMANOID_SEMANTIC_RETARGET_V2";
  rootMotionScale: number;
  outputMotionFingerprintSha256: string;
  algorithmVersion: string;
  algorithmLimits: string;
}

export interface AnimationLibraryPresetProvenanceV1 {
  presetId: string;
  presetVersion: number;
  presetMotionSha256: string;
  catalogSha256: string;
  source: "BUILT_IN" | "COMMUNITY";
  authors: string[];
  license: string;
  rigSignatureSha256: string;
  instantiationMode: "STRICT_RIG_V1";
}

export interface AnimationKeyframeV1 {
  id: string;
  timeSeconds: number;
  value: number[];
}

export interface AuthoredAnimationTrackV1 {
  id: string;
  targetNodeId: number;
  path: AuthoredAnimationTrackPathV1;
  interpolation: AuthoredAnimationInterpolationV1;
  keyframes: AnimationKeyframeV1[];
}

export interface AuthoredAnimationEventV1 {
  id: string;
  timeSeconds: number;
  name: string;
}

export interface AuthoredAnimationClipV1 {
  id: string;
  name: string;
  kind: AuthoredAnimationClipKindV1;
  status: AuthoredAnimationClipStatusV1;
  source: AuthoredAnimationSourceV1;
  lengthSeconds: number;
  transitionSeconds: number;
  animationRoot: string;
  tracks: AuthoredAnimationTrackV1[];
  events: AuthoredAnimationEventV1[];
  revision: number;
}

export interface AnimationStudioDocumentV1 {
  schemaVersion:
    | typeof ANIMATION_STUDIO_SCHEMA_VERSION_V1
    | typeof ANIMATION_STUDIO_SCHEMA_VERSION_V2
    | typeof ANIMATION_STUDIO_SCHEMA_VERSION_V3;
  sourceRevision: string;
  authoringRevision: number;
  status: AnimationStudioDocumentStatusV1;
  authoredClips: AuthoredAnimationClipV1[];
}

export type CustomAnimationClipReferenceSourceKindV2 =
  | "SOURCE_CLIP"
  | "AUTHORED_CLIP";

export interface CustomAnimationClipReferenceV2 {
  sourceKind: CustomAnimationClipReferenceSourceKindV2;
  sourceClipName: string | null;
  authoredClipId: string | null;
}

export interface CustomAnimationPhaseV2 {
  phase: CustomAnimationPhaseKindV1;
  clipReference: CustomAnimationClipReferenceV2;
}

export interface CustomAnimationDefinitionV2 {
  id: string;
  name: string;
  playback: CustomAnimationPlaybackV1;
  clipReference: CustomAnimationClipReferenceV2 | null;
  phases: CustomAnimationPhaseV2[];
  provenance: AnimationMappingProvenanceV1;
}

/**
 * V2 deliberately keeps the Base-42 assignment wire shape. The stable
 * customAnimationId remains the relational key; output names never are.
 */
export type AnimationSourceAssignmentV2 = AnimationSourceAssignmentV1;

export interface CreatureAnimationAuthoringV2 {
  schemaVersion: 2;
  profile: typeof CREATURE_ANIMATION_AUTHORING_PROFILE_V2;
  modelType: DirectCreatureModelTypeV1;
  sourceRevision: string;
  authoringRevision: number;
  assignments: AnimationSourceAssignmentV2[];
  fallbacks: AnimationFallbackDecisionV1[];
  customAnimations: CustomAnimationDefinitionV2[];
}

export type AnimationStudioDiagnosticLevelV1 =
  | "INFO"
  | "WARNING"
  | "BLOCKING";

export interface AnimationStudioDiagnosticV1 {
  schemaVersion: 1;
  code: string;
  path: string;
  level: AnimationStudioDiagnosticLevelV1;
  message: string;
  action: string;
}

export type AnimationStudioReadbackStatusV1 =
  | "MATCH"
  | "MISMATCH";

export interface AnimationStudioReadbackClipV1 {
  authoredClipId: string;
  outputClipName: string;
  materializedFingerprint: string;
  materializedClip: {
    name: string;
    animationRoot: string;
    lengthSeconds: number;
    transitionSeconds: number;
    events: Array<{ timeSeconds: number; name: string }>;
    tracks: Array<{
      targetNodeId: number;
      path: "TRANSLATION" | "ROTATION" | "SCALE" | "WEIGHTS";
      interpolation: "LINEAR" | "STEP" | "CUBIC_SPLINE";
      timesSeconds: number[];
      values: number[][];
    }>;
  };
}

export interface AnimationStudioReadbackV1 {
  schemaVersion: 1;
  studioFingerprint: string;
  sourceRevision: string;
  status: AnimationStudioReadbackStatusV1;
  clips: AnimationStudioReadbackClipV1[];
  diagnostics: AnimationStudioDiagnosticV1[];
}

export interface CustomAnimationLibraryItemV1 {
  authoredClipId: string;
  name: string;
  status: AuthoredAnimationClipStatusV1;
  revision: number;
}

export type AnimationStudioDocumentParseResultV1 =
  | { kind: "VALID"; value: AnimationStudioDocumentV1 }
  | { kind: "INVALID"; diagnostics: AnimationStudioDiagnosticV1[] };
