import type { DirectCreatureBaseSlotV1 } from "../source/directCreatureAnimationProfile";

export type { DirectCreatureBaseSlotV1 };

export const CREATURE_ANIMATION_AUTHORING_PROFILE_V1 =
  "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1" as const;

export type DirectCreatureModelTypeV1 = "S" | "L";
export type PlaybackPolicyV1 = "ENGINE_MANAGED";

export interface AuroraAnimationStateDefinitionV1 {
  readonly stateId: string;
  readonly label: string;
  readonly description: string;
  readonly slot: DirectCreatureBaseSlotV1;
  readonly supportedModelTypes: readonly DirectCreatureModelTypeV1[];
  readonly playbackPolicy: PlaybackPolicyV1;
}

export interface CreatureAnimationCatalogContractV1 {
  schemaVersion: 1;
  profile: "DIRECT_CREATURE_S_L_BASE_42_CATALOG_V1";
  supportedModelTypes: DirectCreatureModelTypeV1[];
  playbackPolicy: PlaybackPolicyV1;
  states: Array<Pick<
    AuroraAnimationStateDefinitionV1,
    "stateId" | "label" | "description" | "slot"
  >>;
}

export interface SourceAnimationClipV1 {
  clipId: string;
  name: string;
  durationSeconds: number;
  trackCount: number;
  targetNodeIds: number[];
  targetPaths: string[];
}

export interface SourceAnimationMeaningCandidateV1 {
  slot: DirectCreatureBaseSlotV1;
  score: number;
  reasonCode:
    | "EXACT_AURORA_SLOT"
    | "EXACT_SEMANTIC_ALIAS"
    | "AMBIGUOUS_SEMANTIC_ALIAS"
    | "MESHY_ACTION_CANDIDATE";
  reason: string;
}

export interface SourceAnimationRigV1 {
  readonly requiredNodeIds: readonly number[];
  readonly allowedNodeIds: readonly number[];
  readonly requiredTargetPaths: readonly string[];
  readonly allowedTargetPaths: readonly string[];
}

export interface SourceAnimationCoverageSummaryV1 {
  sourceClipCount: number;
  confidentlyCoveredSlotCount: number;
  confidentlyCoveredSlots: DirectCreatureBaseSlotV1[];
  missingBaseSlotCount: number;
  unknownClipCount: number;
  ambiguousClipCount: number;
  conflictCount: number;
}

export interface RankedSourceClipCandidateV1 {
  targetSlot: DirectCreatureBaseSlotV1;
  clipId: string;
  clipName: string;
  score: number;
  reasonCode: SourceAnimationMeaningCandidateV1["reasonCode"];
  reason: string;
}

export interface AvailableAnimationSourceV1 {
  clipId: string;
  clipName: string;
  score: number;
  requiresReview: boolean;
  reason: string;
}

export type AnimationMappingProposalDecisionV1 =
  | "AUTO_ASSIGN"
  | "REVIEW"
  | "UNRESOLVED";

export interface AnimationMappingProposalItemV1 {
  stateId: string;
  targetSlot: DirectCreatureBaseSlotV1;
  decision: AnimationMappingProposalDecisionV1;
  recommendedClipId: string | null;
  confidence: number;
  candidates: RankedSourceClipCandidateV1[];
  reason: string;
}

export interface CreatureAnimationMappingProposalV1 {
  schemaVersion: 1;
  items: AnimationMappingProposalItemV1[];
}

export type AnimationProviderV1 =
  | "SOURCE_GLB"
  | "COMPATIBLE_SUPERMODEL"
  | "PROCEDURAL_GENERATOR"
  | "USER_CUSTOM";

export type AnimationOwnershipV1 =
  | "USER_OWNED"
  | "ENGINE_INHERITED"
  | "PROJECT_GENERATED";

export interface AnimationMappingProvenanceV1 {
  provider: AnimationProviderV1;
  /** Portable asset identity. A filesystem path is never accepted here. */
  assetId: string;
  ownership: AnimationOwnershipV1;
}

export type AnimationSourceKindV1 =
  | "SOURCE_CLIP"
  | "INHERITED_SUPERMODEL"
  | "PROCEDURAL"
  | "CUSTOM";

export interface AnimationSourceAssignmentV1 {
  targetSlot: DirectCreatureBaseSlotV1;
  sourceKind: AnimationSourceKindV1;
  sourceClipName: string | null;
  customAnimationId: string | null;
  provenance: AnimationMappingProvenanceV1;
}

export type AnimationFallbackReviewV1 = "PENDING" | "ACCEPTED" | "REJECTED";

export interface AnimationFallbackDecisionV1 {
  id: string;
  targetSlot: DirectCreatureBaseSlotV1;
  sourceSlot: DirectCreatureBaseSlotV1;
  reason: string;
  review: AnimationFallbackReviewV1;
}

export type CustomAnimationPlaybackV1 = "ONE_SHOT" | "LOOPING_PHASED";
export type CustomAnimationPhaseKindV1 = "START" | "LOOP" | "END";

export interface CustomAnimationPhaseV1 {
  phase: CustomAnimationPhaseKindV1;
  sourceClipName: string;
}

export interface CustomAnimationDefinitionV1 {
  id: string;
  name: string;
  playback: CustomAnimationPlaybackV1;
  /** Used only by ONE_SHOT. Phased loops use phases. */
  sourceClipName: string | null;
  phases: CustomAnimationPhaseV1[];
  provenance: AnimationMappingProvenanceV1;
}

export interface CreatureAnimationAuthoringV1 {
  schemaVersion: 1;
  profile: typeof CREATURE_ANIMATION_AUTHORING_PROFILE_V1;
  modelType: DirectCreatureModelTypeV1;
  sourceRevision: string;
  authoringRevision: number;
  assignments: AnimationSourceAssignmentV1[];
  fallbacks: AnimationFallbackDecisionV1[];
  customAnimations: CustomAnimationDefinitionV1[];
}

export type AnimationMappingDiagnosticLevelV1 = "INFO" | "WARNING" | "BLOCKING";

export interface CreatureAnimationMappingDiagnosticV1 {
  schemaVersion: 1;
  code: string;
  path: string;
  level: AnimationMappingDiagnosticLevelV1;
  message: string;
  action: string;
}

export type AnimationMappingDiagnosticV1 =
  CreatureAnimationMappingDiagnosticV1;

export type CreatureAnimationMappingStatusV1 =
  | "READY"
  | "NEEDS_REVIEW"
  | "BLOCKED";

export interface CreatureAnimationInspectionV1 {
  readonly sourceClips: readonly SourceAnimationClipV1[];
}

export type AnimationCatalogFilterV1 =
  | "NEEDS_ATTENTION"
  | "BASE_42"
  | "CUSTOM";

export type AnimationCatalogRowKindV1 = "BASE" | "CUSTOM";
export type AnimationCatalogRowStatusV1 =
  | "MAPPED"
  | "NEEDS_REVIEW"
  | "BLOCKED";

export interface AnimationCatalogRowV1 {
  readonly id: string;
  readonly kind: AnimationCatalogRowKindV1;
  readonly stateId: string | null;
  readonly label: string;
  readonly description: string;
  readonly slot: DirectCreatureBaseSlotV1 | null;
  readonly modelType: DirectCreatureModelTypeV1;
  readonly playbackPolicy: PlaybackPolicyV1 | CustomAnimationPlaybackV1;
  readonly sourceLabel: string | null;
  readonly provenance: AnimationMappingProvenanceV1 | null;
  readonly status: AnimationCatalogRowStatusV1;
  readonly diagnosticCodes: readonly string[];
}
