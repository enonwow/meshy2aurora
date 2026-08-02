/// <reference types="vite/client" />

declare module "@m2a-wasm" {
  export default function init(): Promise<unknown>;
  export function ingestGlbJson(bytes: Uint8Array): string;
  export function ingestStaticRigidGlbJson(bytes: Uint8Array): string;
  export function inspectItemBaseitemsV1Json(bytes: Uint8Array): string;
  export function inspectItemReferenceTwoDaV1Json(
    tableName: string,
    bytes: Uint8Array,
  ): string;
  export function inspectItemReferenceResourceV1Json(
    resourceType: number,
    resref: string,
    bytes: Uint8Array,
    provenanceJson: string,
  ): string;
  export function resolveItemPartResourceV1Json(
    baseitemsTwoDa: Uint8Array,
    baseItem: number,
    field: string,
    variant: number,
    explicitModelResref: string,
    explicitIconResref: string,
  ): string;
  export function validateItemTriangleBudgetV1Json(countsJson: string): string;
  export function buildMeshyItemPartV1(
    sourceGlb: Uint8Array,
    modelResref: string,
    textureResref: string,
    transformJson: string,
  ): {
    readonly readbackJson: string;
    readonly reportJson: string;
    takeMdlBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyItemPartWithOptionsV1(
    sourceGlb: Uint8Array,
    modelResref: string,
    textureResref: string,
    optionsJson: string,
  ): {
    readonly readbackJson: string;
    readonly reportJson: string;
    takeMdlBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeIconBytes(): Uint8Array;
    free(): void;
  };
  export function buildMeshyItemPartWithOptionsV2(
    sourceGlb: Uint8Array,
    modelResref: string,
    textureResref: string,
    optionsJson: string,
  ): {
    readonly readbackJson: string;
    readonly reportJson: string;
    takeMdlBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeIconBytes(): Uint8Array;
    free(): void;
  };
  export function measureMeshyItemSeamV1Json(
    firstField: string,
    firstSourceGlb: Uint8Array,
    firstModelResref: string,
    firstOptionsJson: string,
    secondField: string,
    secondSourceGlb: Uint8Array,
    secondModelResref: string,
    secondOptionsJson: string,
    tolerance: number,
  ): string;
  export function resolveItemCapartPartV1Json(
    field: string,
    selector: number,
    capartTwoDa: Uint8Array,
    partsTableName: string,
    partsTwoDa: Uint8Array,
  ): string;
  export function resolveItemCapartPartV2Json(
    field: string,
    selector: number,
    capartTwoDa: Uint8Array,
    partsTableName: string,
    partsTwoDa: Uint8Array,
    contextJson: string,
    resourceInventoryJson: string,
    hiddenByRobe: boolean,
  ): string;
  export function resolveItemCastSpellIconV1Json(
    baseItem: number,
    propertiesJson: string,
    iprpSpellsTwoDa: Uint8Array,
  ): string;
  export function resolveItemCloakV1Json(
    cloakModelRow: number,
    cloakModelTwoDa: Uint8Array,
  ): string;
  export function resolveItemCloakV2Json(
    cloakModelRow: number,
    cloakModelTwoDa: Uint8Array,
    availableResourceKeysJson: string,
  ): string;
  export function resolveItemCloakV3Json(
    cloakModelRow: number,
    cloakModelTwoDa: Uint8Array,
    resourceInventoryJson: string,
  ): string;
  export function resolveItemCloakV4Json(
    cloakModelRow: number,
    modelPrefix: string,
    cloakModelTwoDa: Uint8Array,
    resourceInventoryJson: string,
  ): string;
  export function resolveItemEquippedAppearanceV1Json(
    appearanceTwoDa: Uint8Array,
    appearanceRow: number,
    racialType: number,
    gender: number,
    phenotype: number,
  ): string;
  export function writeItemUtiV1(
    baseitemsTwoDa: Uint8Array,
    baseItem: number,
    blueprintJson: string,
  ): {
    readonly reportJson: string;
    takeUtiBytes(): Uint8Array;
    free(): void;
  };
  export function buildItemProofModuleV1(
    utiPayload: Uint8Array,
    identityJson: string,
    placementJson: string,
  ): {
    readonly reportJson: string;
    takeModuleBytes(): Uint8Array;
    free(): void;
  };
  export function buildItemEquippedProofModuleV2(
    utiPayload: Uint8Array,
    identityJson: string,
    placementJson: string,
  ): {
    readonly reportJson: string;
    takeModuleBytes(): Uint8Array;
    free(): void;
  };
  export function writeHakV1(
    payloadBlob: Uint8Array,
    resourcesJson: string,
    optionsJson: string,
  ): Uint8Array;
  export function writeTgaV1(imageJson: string, optionsJson: string): Uint8Array;
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
