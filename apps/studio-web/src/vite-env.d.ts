/// <reference types="vite/client" />

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
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
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
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    takeTextureBytes(): Uint8Array;
    takeProofModuleBytes(): Uint8Array;
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
