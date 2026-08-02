/// <reference types="vite/client" />

declare module "@m2a-wasm" {
  export default function init(): Promise<unknown>;
  export function studioRuntimeCapabilitiesV1Json(): string;
  export function inspectModelComponentsV1Json(
    bytes: Uint8Array,
    target: "CREATURE" | "PLACEABLE" | "TILE" | "MODEL_PART",
  ): string;
  export function resolveModelMaterialsV1Json(
    bytes: Uint8Array,
    target: "CREATURE" | "PLACEABLE" | "TILE" | "MODEL_PART",
    documentJson: string,
  ): string;
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
