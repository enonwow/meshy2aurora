export type ModelPackageLaneV1 =
  | "H1_SKINNED_FULL_42"
  | "M0_STATIC_RIGID";

export type SkinAccessoryStabilizationOptionsV1 = {
  mode: "AUTO" | "KEEP_SOURCE_WEIGHTS" | "SELECT_BONE";
  selectedBoneName?: string;
};

export type StudioWorkerRequest =
  | { requestId: string; type: "INITIALIZE" }
  | {
      requestId: string;
      type: "INSPECT_SOURCE";
      sourceGlb: ArrayBuffer;
      target?: "CREATURE" | "PLACEABLE" | "ITEM" | "TILE";
      creatureProfile?: "PRODUCT_300K" | "EXPERIMENTAL_P100K" | "EXPERIMENTAL_P300K";
    }
  | { requestId: string; type: "INSPECT_APPEARANCE"; appearanceTwoDa: ArrayBuffer }
  | { requestId: string; type: "INSPECT_ITEM_BASEITEMS"; baseitemsTwoDa: ArrayBuffer }
  | {
      requestId: string;
      type: "BUILD_ITEM_ATTACHMENT_PROFILE";
      baseitemsTwoDa: ArrayBuffer;
      baseItem: number;
      referenceKind: "REFERENCE_UTI" | "EXPLICIT_VARIANTS" | "APPROVED_PROFILE";
      referenceId: string;
      models: Array<{
        field: string;
        modelResref: string;
        bytes: ArrayBuffer;
      }>;
    }
  | {
      requestId: string;
      type: "FIT_ITEM_PARTS";
      tolerance: number;
      targetAxialLengths?: number[];
      targetAxialScaleFactors?: number[];
      attachmentProfileJson?: string;
      parts: Array<{
        field: string;
        modelResref: string;
        sourceGlb: ArrayBuffer;
        sourceNode: string | null;
      }>;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: ModelPackageLaneV1;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_42";
      identityJson: string;
      textureArtifactCleanup: boolean;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV1;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT";
      identityJson: string;
      textureArtifactCleanup: boolean;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV1;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT";
      identityJson: string;
      textureArtifactCleanup: boolean;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV1;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "H1_SKINNED_FULL_42_EVENTS";
      eventAuthoringJson: string;
    }
  | {
      requestId: string;
      type: "BUILD_PLACEABLE_PACKAGE";
      sourceGlb: ArrayBuffer;
      placeablesTwoDa: ArrayBuffer;
      identityJson: string;
      placementJson: string;
      paletteId: number;
      authoringJson?: string;
    }
  | {
      requestId: string;
      type: "BUILD_TILE_PACKAGE";
      sourceGlb: ArrayBuffer;
      optionsJson: string;
    }
  | {
      requestId: string;
      type: "BUILD_ITEM_PACKAGE";
      baseitemsTwoDa: ArrayBuffer;
      baseItem: number;
      hakResref: string;
      hakFileName: string;
      moduleResref: string;
      moduleFileName: string;
      moduleName: string;
      areaResref: string;
      areaName: string;
      blueprintResref: string;
      blueprintJson: string;
      generationSessionJson: string | null;
      generationArtifactsJson: string | null;
      fitReportJson: string | null;
      attachmentProfileJson?: string | null;
      occupiedResourceKeys: string[];
      seamValidation: {
        tolerance: number;
      };
      referenceTables: Array<{
        tableName: string;
        fileName: string;
        bytes: ArrayBuffer;
      }>;
      referenceResources: Array<{
        resourceType: 6 | 2002;
        resref: string;
        fileName: string;
        bytes: ArrayBuffer;
      }>;
      referenceResourceManifest: {
        fileName: string;
        bytes: ArrayBuffer;
      } | null;
      capartContext: {
        schemaVersion: 1;
        modelPrefix: string;
        genderCode: string | null;
      } | null;
      equippedProofContext: {
        creatureResref: string;
        appearanceRow: number;
        race: number;
        gender: number;
        phenotype: number;
      } | null;
      parts: Array<{
        field: string;
        variant: number;
        sourceKind: "MESHY_GLB" | "CAPART_SELECTION" | "CLOAK_MODEL_SELECTION";
        modelResref: string;
        iconResref: string;
        textureResref: string;
        weaponColorways?: Array<{
          color: 1 | 2 | 3 | 4;
          variant: number;
          modelResref: string;
          iconResref: string;
          textureResref: string;
        }>;
        sourceGlb?: ArrayBuffer;
        transformJson: string;
        targetSpaceScaleXyz?: [number, number, number];
        sourceNode: string | null;
        textureEncoding:
          | "DIRECT_COLOR"
          | "PLT_METAL1"
          | "PLT_METAL2"
          | "PLT_CLOTH1"
          | "PLT_CLOTH2"
          | "PLT_LEATHER1"
          | "PLT_LEATHER2";
      }>;
    }
  | { requestId: string; type: "VALIDATE_M7_CORPUS"; manifestJson: string }
  | {
      requestId: string;
      type: "INSPECT_M7_CORPUS_INTAKE";
      manifestJson: string;
      payloadBlob: ArrayBuffer;
      descriptorsJson: string;
    }
  | {
      requestId: string;
      type: "BUILD_M7_CORPUS_BATCH";
      manifestJson: string;
      payloadBlob: ArrayBuffer;
      descriptorsJson: string;
    };

export interface WorkerArtifact {
  artifactId: string;
  kind: "HAK" | "MODEL" | "MODULE" | "WOK" | "SET" | "TEXTURE" | "ITEM_BLUEPRINT" | "JSON_REPORT" | "SOURCE_MODEL" | "SOURCE_MANIFEST";
  fileName: string;
  mediaType: string;
  byteLength: number;
  sha256: string;
  bytes: ArrayBuffer;
  provenance: "M2A_WASM_WORKER";
}

export type StudioWorkerSuccess =
  | { requestId: string; ok: true; type: "INITIALIZED" }
  | {
      requestId: string;
      ok: true;
      type: "SOURCE_INSPECTED";
      ingestJson: string;
      placeableAuthoringJson?: string;
    }
  | { requestId: string; ok: true; type: "APPEARANCE_INSPECTED"; inspectionJson: string }
  | { requestId: string; ok: true; type: "ITEM_BASEITEMS_INSPECTED"; catalogJson: string }
  | {
      requestId: string;
      ok: true;
      type: "ITEM_ATTACHMENT_PROFILE_BUILT";
      attachmentProfileJson: string;
    }
  | { requestId: string; ok: true; type: "ITEM_PARTS_FITTED"; fitReportJson: string }
  | {
      requestId: string;
      ok: true;
      type: "MODEL_PACKAGE_BUILT";
      artifacts: WorkerArtifact[];
      reportJson: string;
      manifestJson: string;
      summaryJson: string;
      readbackJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "PLACEABLE_PACKAGE_BUILT";
      artifacts: WorkerArtifact[];
      reportJson: string;
      readbackJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "TILE_PACKAGE_BUILT";
      artifacts: WorkerArtifact[];
      reportJson: string;
      modelReadbackJson: string;
      wokReadbackJson: string;
      setReadbackJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "ITEM_PACKAGE_BUILT";
      artifacts: WorkerArtifact[];
      reportJson: string;
      partReadbacksJson: string;
      utiReportJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "M7_CORPUS_VALIDATED";
      manifestJson: string;
      artifacts: WorkerArtifact[];
    }
  | {
      requestId: string;
      ok: true;
      type: "M7_CORPUS_INTAKE_INSPECTED";
      intakeJson: string;
      artifacts: WorkerArtifact[];
    }
  | {
      requestId: string;
      ok: true;
      type: "M7_CORPUS_BATCH_BUILT";
      batchJson: string;
      artifacts: WorkerArtifact[];
    };

export interface StudioWorkerFailure {
  requestId: string;
  ok: false;
  type: "FAILED";
  message: string;
}

export type StudioWorkerResponse = StudioWorkerSuccess | StudioWorkerFailure;
