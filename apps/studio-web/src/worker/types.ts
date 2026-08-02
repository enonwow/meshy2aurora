export type ModelPackageLaneV1 = "M0_STATIC_RIGID";
export type CreatureSourceForwardV1 =
  | "POSITIVE_Z"
  | "NEGATIVE_Z"
  | "POSITIVE_X"
  | "NEGATIVE_X";

export type ModelMaterialTargetV1 = "CREATURE" | "PLACEABLE" | "TILE" | "MODEL_PART";

export type SkinAccessoryStabilizationOptionsV2 = {
  mode: "AUTO" | "KEEP_SOURCE_WEIGHTS" | "SELECT_BONE";
  selectedBoneName?: string;
  componentBoneOverrides?: readonly {
    segmentIndex: number;
    componentIndex: number;
    boneName: string;
  }[];
};

/** @deprecated Use SkinAccessoryStabilizationOptionsV2. */
export type SkinAccessoryStabilizationOptionsV1 = SkinAccessoryStabilizationOptionsV2;

export type StudioWorkerRequest =
  | { requestId: string; type: "INITIALIZE" }
  | {
      requestId: string;
      type: "INSPECT_MODEL_COMPONENTS";
      sourceGlb: ArrayBuffer;
      target: ModelMaterialTargetV1;
      sourceStateId: string;
    }
  | {
      requestId: string;
      type: "RESOLVE_MODEL_MATERIALS";
      sourceGlb: ArrayBuffer;
      target: ModelMaterialTargetV1;
      documentJson: string;
      sourceStateId: string;
      recipeStateId: string;
    }
  | {
      requestId: string;
      type: "INSPECT_SOURCE";
      sourceGlb: ArrayBuffer;
      target?: "CREATURE" | "PLACEABLE" | "TILE";
      creatureProfile?: "PRODUCT_300K" | "EXPERIMENTAL_P100K" | "EXPERIMENTAL_P300K";
      experimentalAggressiveGeometryCleanup?: boolean;
      modelResref?: string;
    }
  | {
      requestId: string;
      type: "RESOLVE_PLACEABLE_COLLISION";
      sourceGlb: ArrayBuffer;
      modelResref: string;
      authoringJson: string;
      experimentalAggressiveGeometryCleanup?: boolean;
    }
  | {
      requestId: string;
      type: "RESOLVE_PLACEABLE_TEXTURES";
      sourceGlb: ArrayBuffer;
      baseTextureResref: string;
      geometryAuthoringJson: string;
      textureAuthoringJson: string;
      texturePayloadBlob: ArrayBuffer;
      texturePayloadDescriptorsJson: string;
      experimentalAggressiveGeometryCleanup?: boolean;
    }
  | { requestId: string; type: "INSPECT_APPEARANCE"; appearanceTwoDa: ArrayBuffer }
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
      demoModuleIdentityJson: string;
      demoCreatureResref: string;
      textureArtifactCleanup: boolean;
      sourceForward: CreatureSourceForwardV1;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV2;
      materialSeparationJson?: string;
      modelTextureAuthoringJson?: string;
      modelTexturePayloadBlob?: ArrayBuffer;
      modelTexturePayloadDescriptorsJson?: string;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT";
      identityJson: string;
      textureArtifactCleanup: boolean;
      sourceForward: CreatureSourceForwardV1;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV2;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT";
      identityJson: string;
      textureArtifactCleanup: boolean;
      sourceForward: CreatureSourceForwardV1;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV2;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "H1_SKINNED_FULL_42";
      identityJson: string;
      demoModuleIdentityJson: string;
      demoCreatureResref: string;
      textureArtifactCleanup: boolean;
      sourceForward: CreatureSourceForwardV1;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV2;
      materialSeparationJson?: string;
      modelTextureAuthoringJson?: string;
      modelTexturePayloadBlob?: ArrayBuffer;
      modelTexturePayloadDescriptorsJson?: string;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "H1_SKINNED_FULL_42_EVENTS";
      eventAuthoringJson: string;
      identityJson: string;
      demoModuleIdentityJson: string;
      demoCreatureResref: string;
      textureArtifactCleanup: boolean;
      sourceForward: CreatureSourceForwardV1;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV2;
      materialSeparationJson?: string;
      modelTextureAuthoringJson?: string;
      modelTexturePayloadBlob?: ArrayBuffer;
      modelTexturePayloadDescriptorsJson?: string;
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
      textureAuthoringJson?: string;
      texturePayloadBlob?: ArrayBuffer;
      texturePayloadDescriptorsJson?: string;
      materialSeparationJson?: string;
      modelTextureAuthoringJson?: string;
      modelTexturePayloadBlob?: ArrayBuffer;
      modelTexturePayloadDescriptorsJson?: string;
      experimentalAggressiveGeometryCleanup?: boolean;
    }
  | {
      requestId: string;
      type: "BUILD_TILE_PACKAGE";
      sourceGlb: ArrayBuffer;
      optionsJson: string;
      materialSeparationJson?: string;
      modelTextureAuthoringJson?: string;
      modelTexturePayloadBlob?: ArrayBuffer;
      modelTexturePayloadDescriptorsJson?: string;
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
  kind: "HAK" | "MODEL" | "MODULE" | "PWK" | "WOK" | "SET" | "TEXTURE" | "JSON_REPORT";
  fileName: string;
  mediaType: string;
  byteLength: number;
  sha256: string;
  bytes: ArrayBuffer;
  provenance: "M2A_WASM_WORKER";
}

export type StudioWorkerSuccess =
  | {
      requestId: string;
      ok: true;
      type: "INITIALIZED";
      runtimeCapabilities: {
        schemaVersion: 1;
        runtimeContract: "M2A_STUDIO_WASM_2026_07_31_V1";
        creatureSourceForward: "CARDINAL_XZ_TO_AURORA_NEGATIVE_Y_V1";
        creatureTriangleBudget: 300000;
      };
    }
  | {
      requestId: string;
      ok: true;
      type: "MODEL_COMPONENTS_INSPECTED";
      sourceStateId: string;
      inspectionJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "MODEL_MATERIALS_RESOLVED";
      sourceStateId: string;
      recipeStateId: string;
      resolutionJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "SOURCE_INSPECTED";
      ingestJson: string;
      placeableAuthoringJson?: string;
      placeableCollisionJson?: string;
      placeableTexturesJson?: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "PLACEABLE_COLLISION_RESOLVED";
      collisionJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "PLACEABLE_TEXTURES_RESOLVED";
      texturesJson: string;
    }
  | { requestId: string; ok: true; type: "APPEARANCE_INSPECTED"; inspectionJson: string }
  | {
      requestId: string;
      ok: true;
      type: "MODEL_PACKAGE_BUILT";
      artifacts: WorkerArtifact[];
      reportJson: string;
      manifestJson: string;
      summaryJson: string;
      readbackJson: string;
      demoReportJson?: string;
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
