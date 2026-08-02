/// <reference types="vite/client" />

declare const __M2A_CANONICAL_REPOSITORY_ROOT__: string;

declare module "@m2a-wasm" {
  export default function init(): Promise<unknown>;
  export function ingestGlbJson(bytes: Uint8Array): string;
  export function ingestStaticRigidGlbJson(bytes: Uint8Array): string;
  export function inspectTwoDaV2Json(bytes: Uint8Array, limitsJson: string): string;
  export function validateM7CorpusManifestV1Json(manifestJson: string): string;
  export function inspectM7CorpusIntakeV1Json(
    manifestJson: string,
    payloadBlob: Uint8Array,
    descriptorsJson: string,
  ): string;
  export function buildM7CorpusBatchV1(
    manifestJson: string,
    payloadBlob: Uint8Array,
    descriptorsJson: string,
  ): string;
  export function buildM6ModelPackageV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyH1ModelPackageV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyH1ModelPackageV2(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyH1ModelPackageV3(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    eventAuthoringJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function directCreatureAnimationCatalogV1Json(): string;
  export function validateCreatureAnimationAuthoringV1(
    animationAuthoringJson: string,
  ): string;
  export function resolveCreatureAnimationMappingV1(
    animationAuthoringJson: string,
  ): string;
  export function inspectEditableAnimationSourceV1(
    sourceGlb: Uint8Array,
    clipName?: string,
  ): string;
  export function inspectAnimationTransferCompatibilityV1(
    targetGlb: Uint8Array,
    donorGlb: Uint8Array,
  ): string;
  export function retargetAnimationClipBetweenModelsV1(
    targetGlb: Uint8Array,
    donorGlb: Uint8Array,
    clipName: string,
    optionsJson: string,
  ): string;
  export function inspectHumanoidRetargetCompatibilityV2(
    targetGlb: Uint8Array,
    donorGlb: Uint8Array,
    overridesJson: string,
    manualMappingConfirmed: boolean,
  ): string;
  export function retargetAnimationClipHumanoidV2(
    targetGlb: Uint8Array,
    donorGlb: Uint8Array,
    clipName: string,
    semanticMapJson: string,
    clipId: string,
    outputName: string,
  ): string;
  export function buildAnimationSequencePreviewV1(
    requestJson: string,
    availableClipsJson: string,
  ): string;
  export function resampleAnimationCurveToLinearV1(
    curveJson: string,
    policyJson: string,
  ): string;
  export function bakeAnimationLayersV1(
    layersJson: string,
    rigJson: string,
  ): string;
  export function applyAnimationWorkbenchOperationV1(requestJson: string): string;
  export function prepareAnimationTransferBatchV2(
    targetGlb: Uint8Array,
    donorGlb: Uint8Array,
    requestJson: string,
  ): string;
  export function validateAnimationStudioDocumentV1(
    animationStudioDocumentJson: string,
    sourceGlb: Uint8Array,
  ): string;
  export function materializeAnimationStudioDocumentV1(
    animationStudioDocumentJson: string,
    sourceGlb: Uint8Array,
  ): string;
  export function previewAuthoredAnimationClipV1(
    animationStudioDocumentJson: string,
    clipId: string,
  ): string;
  export function applyAnimationEditCommandBatchV1(
    animationStudioDocumentJson: string,
    commandBatchJson: string,
    sourceGlb: Uint8Array,
  ): string;
  export function analyzeAnimationMotionQualityV1(
    authoredClipJson: string,
    policyJson: string,
    contextJson: string,
    sourceGlb: Uint8Array,
  ): string;
  export function applyAnimationAuthoringToolV1(
    authoredClipJson: string,
    toolRequestJson: string,
    sourceGlb: Uint8Array,
  ): string;
  export function inspectHeldWeaponSourceV1(
    filename: string,
    provenance: string,
    weaponGlb: Uint8Array,
  ): string;
  export function composeHeldWeaponAttachmentV1(
    weaponGlb: Uint8Array,
    filename: string,
    provenance: string,
    rigJson: string,
    primaryHand: "RIGHT" | "LEFT",
    targetNodeId: number,
    localTransformJson: string,
    attachmentRevision: bigint,
  ): string;
  export function evaluateAnimationPoseParityV1(
    expectedClipJson: string,
    actualClipJson: string,
    policyJson: string,
    sourceGlb: Uint8Array,
  ): string;
  export function validateAnimationPresetV1(
    manifestJson: string,
    animationJson: string,
    catalogSha256: string,
  ): string;
  export function inspectAnimationPresetCompatibilityV1(
    manifestJson: string,
    animationJson: string,
    catalogSha256: string,
    sourceGlb: Uint8Array,
  ): string;
  export function instantiateAnimationPresetV1(
    manifestJson: string,
    animationJson: string,
    catalogSha256: string,
    sourceGlb: Uint8Array,
    clipId: string,
    outputName: string,
  ): string;
  export function exportAnimationContributionV1(
    authoredClipJson: string,
    sourceGlb: Uint8Array,
    metadataJson: string,
  ): string;
  export function buildMeshyH1ModelPackageV4(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    animationAuthoringJson: string,
    eventAuthoringJson?: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyH1ModelPackageV4ProjectV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    animationAuthoringJson: string,
    projectIdentityJson: string,
    eventAuthoringJson?: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyH1ModelPackageV5(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    animationAuthoringJson: string,
    animationStudioDocumentJson: string,
    eventAuthoringJson?: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyH1ModelPackageV5ProjectV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    animationAuthoringJson: string,
    animationStudioDocumentJson: string,
    projectIdentityJson: string,
    eventAuthoringJson?: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyH1ModelPackageV6HeldWeaponProjectV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    animationAuthoringJson: string,
    animationStudioDocumentJson: string,
    projectIdentityJson: string,
    weaponGlb: Uint8Array,
    heldWeaponAttachmentJson: string,
    eventAuthoringJson?: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyProceduralHumanoidModelPackageV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyProceduralHumanoidProductV2(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyProceduralHumanoidProductWithOptionsV3(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
    buildOptionsJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    free(): void;
  };
  export function ingestMeshyP100kExperimentJson(sourceGlb: Uint8Array): string;
  export function ingestMeshyP300kExperimentJson(sourceGlb: Uint8Array): string;
  export function buildMeshyProceduralHumanoidP100kExperimentV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeAppearanceTwoDaBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyProceduralHumanoidP100kExperimentWithOptionsV2(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
    buildOptionsJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeAppearanceTwoDaBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyProceduralHumanoidP300kExperimentV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeAppearanceTwoDaBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyProceduralHumanoidP300kExperimentWithOptionsV2(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
    buildOptionsJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeAppearanceTwoDaBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyM0StaticRigidPackageV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyStaticPlaceablePackageV1(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    identityJson: string,
    placementJson: string,
    paletteId: number,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function inspectMeshyStaticPlaceableAuthoringV1(
    sourceGlb: Uint8Array,
  ): string;
  export function buildMeshyStaticPlaceablePackageV2(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    identityJson: string,
    placementJson: string,
    paletteId: number,
    authoringJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyStaticPlaceablePackageV3ProjectV1(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    projectIdentityJson: string,
    placementJson: string,
    paletteId: number,
    authoringJson?: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyStaticTilePackageV1(
    sourceGlb: Uint8Array,
    optionsJson: string,
  ): {
    readonly reportJson: string;
    readonly modelReadbackJson: string;
    readonly wokReadbackJson: string;
    readonly setReadbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModuleBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeWokBytes(): Uint8Array;
    takeSetBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeImageMapBytes(): Uint8Array;
    free(): void;
  };
}
