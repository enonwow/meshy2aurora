/// <reference types="vite/client" />

declare module "@m2a-wasm" {
  export default function init(): Promise<unknown>;
  export function studioRuntimeCapabilitiesV1Json(): string;
  export function buildExactReferenceSupermodelMotionContractV3Json(
    referenceMdl: Uint8Array,
    optionsJson: string,
  ): string;
  export function buildMotionCorrectedRigV1Json(
    targetRigJson: string,
    motionContractJson: string,
  ): string;
  export function buildExactMotionCarrierRigV3Json(
    targetRigJson: string,
    motionContractJson: string,
    referenceMdl: Uint8Array,
  ): string;
  export function buildReferenceSupermodelAppliedPreviewV2(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
  ): {
    readonly readbackJson: string;
    readonly applyReportJson: string;
    readonly authoringJson: string;
    readonly targetRigJson: string;
    takeModelBytes(): Uint8Array;
    free(): void;
  };
  export function buildReferenceSupermodelAppliedPreviewV3(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
  ): ReturnType<typeof buildReferenceSupermodelAppliedPreviewV2>;
  export function buildReferenceSupermodelAppliedPreviewV4(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
    allowExcessiveBranchBoundaryRepair: boolean,
  ): ReturnType<typeof buildReferenceSupermodelAppliedPreviewV2>;
  export function prepareReferenceSupermodelRigV1(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
  ): {
    readonly reportJson: string;
    readonly authoringJson: string;
    readonly targetRigJson: string;
    free(): void;
  };
  export function prepareReferenceSupermodelRigV2(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
  ): ReturnType<typeof prepareReferenceSupermodelRigV1>;
  export function prepareReferenceSupermodelAuthoredRigV2(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
    authoringJson: string,
  ): ReturnType<typeof prepareReferenceSupermodelRigV1>;
  export function validateReferenceSupermodelSealedRigV2(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
    authoringJson: string,
  ): ReturnType<typeof prepareReferenceSupermodelRigV1>;
  export function prepareReferenceSupermodelRigV3(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
    allowExcessiveBranchBoundaryRepair: boolean,
  ): ReturnType<typeof prepareReferenceSupermodelRigV1>;
  export function prepareReferenceSupermodelAuthoredRigV3(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
    authoringJson: string,
    allowExcessiveBranchBoundaryRepair: boolean,
  ): ReturnType<typeof prepareReferenceSupermodelRigV1>;
  export function validateReferenceSupermodelSealedRigV3(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
    authoringJson: string,
    allowExcessiveBranchBoundaryRepair: boolean,
  ): ReturnType<typeof prepareReferenceSupermodelRigV1>;
  export function buildReferenceSupermodelAuthoredPreviewV1(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
    authoringJson: string,
  ): {
    readonly readbackJson: string;
    readonly applyReportJson: string;
    readonly authoringJson: string;
    readonly targetRigJson: string;
    takeModelBytes(): Uint8Array;
    free(): void;
  };
  export function buildReferenceSupermodelAuthoredPreviewV2(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
    authoringJson: string,
  ): ReturnType<typeof buildReferenceSupermodelAppliedPreviewV2>;
  export function buildReferenceSupermodelAuthoredPreviewV3(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    sourceForward: string,
    authoringJson: string,
    allowExcessiveBranchBoundaryRepair: boolean,
  ): ReturnType<typeof buildReferenceSupermodelAppliedPreviewV2>;
  export function buildReferenceSupermodelCreatureProductV2(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    identityJson: string,
    sourceForward: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeMaterialBytes(): Uint8Array;
    takeAppearanceTwoDaBytes(): Uint8Array;
    free(): void;
  };
  export function buildReferenceSupermodelCreatureProductV3(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    identityJson: string,
    sourceForward: string,
    authoringJson: string,
  ): ReturnType<typeof buildReferenceSupermodelCreatureProductV2>;
  export function buildReferenceSupermodelCreatureProductV4(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    identityJson: string,
    sourceForward: string,
    authoringJson: string,
  ): ReturnType<typeof buildReferenceSupermodelCreatureProductV2>;
  export function buildReferenceSupermodelCreatureProductV5(
    selectedSupermodelResref: string,
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    referenceChainBlob: Uint8Array,
    referenceChainJson: string,
    identityJson: string,
    sourceForward: string,
    authoringJson: string,
    allowExcessiveBranchBoundaryRepair: boolean,
  ): ReturnType<typeof buildReferenceSupermodelCreatureProductV2>;
  export function indexNwnKeyModelsV1Json(bytes: Uint8Array): string;
  export function indexHakModelsV1Json(bytes: Uint8Array): string;
  export function planNwnBifIndexV1Json(header: Uint8Array): string;
  export function indexNwnBifTableV1Json(header: Uint8Array, table: Uint8Array): string;
  export function inspectMdlCatalogHeadersV1Json(
    blob: Uint8Array,
    descriptorsJson: string,
  ): string;
  export function buildSupermodelCatalogV1Json(inputJson: string): string;
  export function inspectBinaryMdl(bytes: Uint8Array): string;
  export function inspectModelComponentsV1Json(
    bytes: Uint8Array,
    target: "CREATURE" | "PLACEABLE" | "TILE" | "MODEL_PART",
  ): string;
  export function resolveModelMaterialsV1Json(
    bytes: Uint8Array,
    target: "CREATURE" | "PLACEABLE" | "TILE" | "MODEL_PART",
    documentJson: string,
  ): string;
  export function inspectModelFacesV2Json(
    bytes: Uint8Array,
    target: "CREATURE" | "PLACEABLE" | "TILE" | "MODEL_PART",
  ): string;
  export function resolveModelMaterialsV2Json(
    bytes: Uint8Array,
    target: "CREATURE" | "PLACEABLE" | "TILE" | "MODEL_PART",
    documentJson: string,
  ): string;
  export function ingestGlbJson(bytes: Uint8Array): string;
  export function inspectHighPolyGlbJson(bytes: Uint8Array): string;
  export function inspectCreatureMotionPackSourceV1Json(
    bytes: Uint8Array,
    motionPackJson: string,
  ): string;
  export function ingestStaticRigidGlbJson(bytes: Uint8Array): string;
  export function inspectTwoDaV2Json(bytes: Uint8Array, limitsJson: string): string;
  export function resolveItemBaseRecordV1Json(
    bytes: Uint8Array,
    physicalRowIndex: number,
    expectedSourceSha256: string,
    limitsJson: string,
  ): string;
  export function resolveItemRecipeV1Json(recipeJson: string): string;
  export function compileItemPartV1(requestJson: string): {
    readonly reportJson: string;
    readonly readbackJson: string;
    takeMdlBytes(): Uint8Array;
    free(): void;
  };
  export function writeItemIconLayersV1(
    recipeJson: string,
    layersJson: string,
    optionsJson: string,
  ): {
    readonly descriptorsJson: string;
    readonly reportsJson: string;
    takePayloadBlob(): Uint8Array;
    free(): void;
  };
  export function writeItemPackageV1(
    payloadBlob: Uint8Array,
    requestJson: string,
    gffOptionsJson: string,
    archiveOptionsJson: string,
  ): {
    readonly manifestSha256: string;
    readonly utiReportJson: string;
    readonly utiReadbackJson: string;
    takeUtiBytes(): Uint8Array;
    takeHakBytes(): Uint8Array;
    takeModuleBytes(): Uint8Array;
    takeManifestBytes(): Uint8Array;
    free(): void;
  };
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
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takePwkBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
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
    readonly textureDescriptorsJson: string;
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
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
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
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    free(): void;
  };
  export function buildMeshyProceduralHumanoidProductDemoWithOptionsV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
    buildOptionsJson: string,
    moduleIdentityJson: string,
    creatureResref: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    readonly demoReportJson: string;
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyFullNativeH1PackageWithOptionsV4(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
    buildOptionsJson: string,
    eventAuthoringJson: string,
    moduleIdentityJson: string,
    creatureResref: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    readonly demoReportJson: string;
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyProceduralHumanoidProductDemoWithMaterialsV2(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
    buildOptionsJson: string,
    moduleIdentityJson: string,
    creatureResref: string,
    materialSeparationJson: string,
    modelTextureAuthoringJson: string,
    texturePayloadBlob: Uint8Array,
    texturePayloadDescriptorsJson: string,
  ): ReturnType<typeof buildMeshyProceduralHumanoidProductDemoWithOptionsV1>;
  export function buildMeshyFullNativeH1PackageWithMaterialsV5(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
    identityJson: string,
    buildOptionsJson: string,
    eventAuthoringJson: string,
    moduleIdentityJson: string,
    creatureResref: string,
    materialSeparationJson: string,
    modelTextureAuthoringJson: string,
    texturePayloadBlob: Uint8Array,
    texturePayloadDescriptorsJson: string,
  ): ReturnType<typeof buildMeshyFullNativeH1PackageWithOptionsV4>;
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
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takePwkBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function inspectMeshyStaticPlaceableAuthoringV1(
    sourceGlb: Uint8Array,
  ): string;
  export function inspectMeshyStaticPlaceableAuthoringV2(
    sourceGlb: Uint8Array,
    optionsJson: string,
  ): string;
  export function inspectMeshyStaticPlaceableAuthoringV3(
    sourceGlb: Uint8Array,
    optionsJson: string,
  ): string;
  export function inspectMeshyStaticPlaceableTexturesV1(
    sourceGlb: Uint8Array,
    optionsJson: string,
  ): string;
  export function resolveMeshyStaticPlaceableTexturesV1(
    sourceGlb: Uint8Array,
    baseTextureResref: string,
    geometryAuthoringJson: string,
    textureAuthoringJson: string,
    texturePayloadBlob: Uint8Array,
    texturePayloadDescriptorsJson: string,
    optionsJson: string,
  ): string;
  export function resolveMeshyStaticPlaceableCollisionV1(
    sourceGlb: Uint8Array,
    modelResref: string,
    authoringJson: string,
    optionsJson: string,
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
  export function buildMeshyStaticPlaceablePackageV3(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    identityJson: string,
    placementJson: string,
    paletteId: number,
    authoringJson: string,
    optionsJson: string,
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
  export function buildMeshyStaticPlaceablePackageV4(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    identityJson: string,
    placementJson: string,
    paletteId: number,
    authoringJson: string,
    optionsJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takePwkBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyStaticPlaceablePackageV5(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    identityJson: string,
    placementJson: string,
    paletteId: number,
    authoringJson: string,
    textureAuthoringJson: string,
    texturePayloadBlob: Uint8Array,
    texturePayloadDescriptorsJson: string,
    optionsJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takePwkBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyStaticPlaceablePackageV6(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    identityJson: string,
    placementJson: string,
    paletteId: number,
    authoringJson: string,
    materialSeparationJson: string,
    modelTextureAuthoringJson: string,
    texturePayloadBlob: Uint8Array,
    texturePayloadDescriptorsJson: string,
    optionsJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takePwkBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyStaticPlaceablePackageV7(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    identityJson: string,
    placementJson: string,
    paletteId: number,
    authoringJson: string,
    materialSeparationJson: string,
    modelTextureAuthoringJson: string,
    texturePayloadBlob: Uint8Array,
    texturePayloadDescriptorsJson: string,
    optionsJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takePwkBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyStaticPlaceablePackageV8(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    identityJson: string,
    placementJson: string,
    paletteId: number,
    authoringJson: string,
    materialSeparationJson: string,
    materialUvProjectionJson: string,
    modelTextureAuthoringJson: string,
    texturePayloadBlob: Uint8Array,
    texturePayloadDescriptorsJson: string,
    optionsJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takePwkBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyStaticPlaceablePackageV9(
    sourceGlb: Uint8Array,
    placeablesTwoDa: Uint8Array,
    identityJson: string,
    placementJson: string,
    paletteId: number,
    authoringJson: string,
    materialSeparationJson: string,
    materialUvProjectionJson: string,
    modelTextureAuthoringJson: string,
    texturePayloadBlob: Uint8Array,
    texturePayloadDescriptorsJson: string,
    materialProfileJson: string,
    optionsJson: string,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takePwkBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
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
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModuleBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeWokBytes(): Uint8Array;
    takeSetBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeImageMapBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyStaticTilePackageV2(
    sourceGlb: Uint8Array,
    optionsJson: string,
    materialSeparationJson: string,
    modelTextureAuthoringJson: string,
    texturePayloadBlob: Uint8Array,
    texturePayloadDescriptorsJson: string,
  ): {
    readonly reportJson: string;
    readonly modelReadbackJson: string;
    readonly wokReadbackJson: string;
    readonly setReadbackJson: string;
    readonly textureDescriptorsJson: string;
    takeHakBytes(): Uint8Array;
    takeModuleBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeWokBytes(): Uint8Array;
    takeSetBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeTexturePayloadBlob(): Uint8Array;
    takeImageMapBytes(): Uint8Array;
    free(): void;
  };
}
