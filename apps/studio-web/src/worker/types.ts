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

export type CreatureWeaponGripOptionsV1 = {
  schemaVersion: 1;
  mode: "AUTO" | "AUTO_PLUS_OFFSETS";
  itemFamily?: "SWORD" | "AXE_MACE" | "SPEAR_POLEARM" | "BOW_CROSSBOW" | "SHIELD";
  rightHand: CreatureWeaponEulerOffsetV1;
  leftHand: CreatureWeaponEulerOffsetV1;
};

export type CreatureDemoAuthoringV1 = import("../features/source/heldWeapon").CreatureDemoAuthoringV1;
export type CreatureRuntimeEnvelopePolicyV1 =
  | { mode: "MEDIUM_HUMANOID" }
  | { mode: "BOUNDS_DERIVED" }
  | { mode: "EXPLICIT"; value: {
      schemaVersion: 1;
      height: number;
      hitDistance: number;
      personalSpace: number;
      creaturePersonalSpace: number;
      preferredAttackDistance: number;
      targetHeight: "LOW" | "MEDIUM" | "HIGH";
      perceptionDistance: number;
      walkDistance: number;
      runDistance: number;
      sizeCategory: number;
      footstepType: number;
    } };
export type CreatureMotionPackV1 = {
  schemaVersion: 1;
  sourceIdentity: string;
  skeletonProfile: "HUMANOID" | "QUADRUPED";
  semanticJoints: readonly string[];
  clips: readonly {
    sourceClipName: string;
    outputClipName: string;
    weaponFamily?: "SWORD" | "AXE_MACE" | "SPEAR_POLEARM" | "BOW_CROSSBOW" | "SHIELD";
  }[];
};
export type CreatureMaterialProfileV2 = {
  schemaVersion: 2;
  target: "AURORA_CLASSIC_SAFE" | "NWN_EE_MTR";
  normalMaps: boolean;
  tangentSpaceReady: boolean;
  metallicRoughnessToSpecularGloss: boolean;
  emissiveToSelfIllumination: boolean;
  alphaMode: "OPAQUE" | "MASK" | "BLEND";
  doubleSided: boolean;
};
export type CreaturePerformancePresetV1 = "COMPACT" | "STANDARD" | "HIGH" | "MAXIMUM";

export type CreatureWeaponEulerOffsetV1 = {
  rollDegrees: number;
  pitchDegrees: number;
  yawDegrees: number;
};

export type CreatureHeldWeaponOptionsV1 = {
  schemaVersion: 1;
  mode: "NONE" | "RIGHT_HAND" | "LEFT_HAND";
  itemResref?: string;
};

/** @deprecated Use SkinAccessoryStabilizationOptionsV2. */
export type SkinAccessoryStabilizationOptionsV1 = SkinAccessoryStabilizationOptionsV2;

export type StudioWorkerRequest =
  | { requestId: string; type: "INITIALIZE" }
  | {
      requestId: string;
      type: "INSPECT_ITEM_INPUTS";
      baseItemsTwoDa: ArrayBuffer;
      physicalRowIndex: number;
      expectedSourceSha256: string;
      sourceStateId: string;
    }
  | {
      requestId: string;
      type: "RESOLVE_ITEM_RECIPE";
      recipeJson: string;
      sourceStateId: string;
      recipeStateId: string;
    }
  | {
      requestId: string;
      type: "COMPILE_ITEM_PART";
      requestJson: string;
      sourceStateId: string;
      recipeStateId: string;
    }
  | {
      requestId: string;
      type: "BUILD_ITEM_ICONS";
      recipeJson: string;
      layersJson: string;
      sourceStateId: string;
      recipeStateId: string;
    }
  | {
      requestId: string;
      type: "BUILD_ITEM_PACKAGE";
      payloadBlob: ArrayBuffer;
      requestJson: string;
      outputStem: string;
      sourceStateId: string;
      recipeStateId: string;
    }
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
      unsafeHighPolyInspection?: boolean;
      experimentalAggressiveGeometryCleanup?: boolean;
      modelResref?: string;
    }
  | {
      requestId: string;
      type: "INSPECT_CREATURE_MOTION_PACK";
      sourceGlb: ArrayBuffer;
      motionPack: CreatureMotionPackV1;
    }
  | {
      requestId: string;
      type: "BUILD_EXACT_REFERENCE_SUPERMODEL_MOTION_CONTRACT";
      referenceMdl: ArrayBuffer;
      optionsJson: string;
    }
  | {
      requestId: string;
      type: "BUILD_REFERENCE_SUPERMODEL_CORRECTION";
      targetRigJson: string;
      motionContractJson: string;
    }
  | {
      requestId: string;
      type: "BUILD_EXACT_REFERENCE_SUPERMODEL_CARRIER";
      targetRigJson: string;
      motionContractJson: string;
      referenceMdl: ArrayBuffer;
    }
  | {
      requestId: string;
      type: "PREPARE_REFERENCE_SUPERMODEL_RIG";
      selectedSupermodelResref: string;
      sourceGlb: ArrayBuffer;
      referenceChainBlob: ArrayBuffer;
      referenceChainJson: string;
      sourceForward: CreatureSourceForwardV1;
    }
  | {
      requestId: string;
      type: "PREPARE_REFERENCE_SUPERMODEL_RIG_V2";
      selectedSupermodelResref: string;
      sourceGlb: ArrayBuffer;
      referenceChainBlob: ArrayBuffer;
      referenceChainJson: string;
      sourceForward: CreatureSourceForwardV1;
      authoringJson?: string;
      authoringIsSealed?: boolean;
      experimentalAllowExcessiveSkinBranchRepair?: boolean;
    }
  | {
      requestId: string;
      type: "BUILD_REFERENCE_SUPERMODEL_APPLIED_PREVIEW";
      selectedSupermodelResref: string;
      sourceGlb: ArrayBuffer;
      referenceChainBlob: ArrayBuffer;
      referenceChainJson: string;
      sourceForward: CreatureSourceForwardV1;
      experimentalAllowExcessiveSkinBranchRepair?: boolean;
    }
  | {
      requestId: string;
      type: "BUILD_REFERENCE_SUPERMODEL_AUTHORED_PREVIEW";
      selectedSupermodelResref: string;
      sourceGlb: ArrayBuffer;
      referenceChainBlob: ArrayBuffer;
      referenceChainJson: string;
      sourceForward: CreatureSourceForwardV1;
      authoringJson: string;
      experimentalAllowExcessiveSkinBranchRepair?: boolean;
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
  | { requestId: string; type: "INDEX_NWN_KEY_MODELS"; keyBytes: ArrayBuffer }
  | { requestId: string; type: "INDEX_HAK_MODELS"; hakBytes: ArrayBuffer }
  | { requestId: string; type: "PLAN_NWN_BIF_INDEX"; headerBytes: ArrayBuffer }
  | {
      requestId: string;
      type: "INDEX_NWN_BIF_TABLE";
      headerBytes: ArrayBuffer;
      tableBytes: ArrayBuffer;
    }
  | {
      requestId: string;
      type: "INSPECT_MDL_CATALOG_HEADERS";
      payloadBlob: ArrayBuffer;
      descriptorsJson: string;
    }
  | { requestId: string; type: "BUILD_SUPERMODEL_CATALOG"; inputJson: string }
  | { requestId: string; type: "INSPECT_BINARY_MDL"; mdlBytes: ArrayBuffer }
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
      packageLane: "REFERENCE_SUPERMODEL_CREATURE";
      selectedSupermodelResref: string;
      referenceChainBlob: ArrayBuffer;
      referenceChainJson: string;
      identityJson: string;
      sourceForward: CreatureSourceForwardV1;
      rigAuthoringJson?: string;
      experimentalAllowExcessiveSkinBranchRepair?: boolean;
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
      weaponGrip?: CreatureWeaponGripOptionsV1;
      heldWeapon?: CreatureHeldWeaponOptionsV1;
      demoAuthoring?: CreatureDemoAuthoringV1;
      runtimeEnvelope?: CreatureRuntimeEnvelopePolicyV1;
      motionPack?: CreatureMotionPackV1;
      materialProfile?: CreatureMaterialProfileV2;
      performancePreset?: CreaturePerformancePresetV1;
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
      weaponGrip?: CreatureWeaponGripOptionsV1;
      heldWeapon?: CreatureHeldWeaponOptionsV1;
      demoAuthoring?: CreatureDemoAuthoringV1;
      runtimeEnvelope?: CreatureRuntimeEnvelopePolicyV1;
      motionPack?: CreatureMotionPackV1;
      materialProfile?: CreatureMaterialProfileV2;
      performancePreset?: CreaturePerformancePresetV1;
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
      weaponGrip?: CreatureWeaponGripOptionsV1;
      heldWeapon?: CreatureHeldWeaponOptionsV1;
      demoAuthoring?: CreatureDemoAuthoringV1;
      runtimeEnvelope?: CreatureRuntimeEnvelopePolicyV1;
      motionPack?: CreatureMotionPackV1;
      materialProfile?: CreatureMaterialProfileV2;
      performancePreset?: CreaturePerformancePresetV1;
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
      weaponGrip?: CreatureWeaponGripOptionsV1;
      heldWeapon?: CreatureHeldWeaponOptionsV1;
      demoAuthoring?: CreatureDemoAuthoringV1;
      runtimeEnvelope?: CreatureRuntimeEnvelopePolicyV1;
      motionPack?: CreatureMotionPackV1;
      materialProfile?: CreatureMaterialProfileV2;
      performancePreset?: CreaturePerformancePresetV1;
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
      weaponGrip?: CreatureWeaponGripOptionsV1;
      heldWeapon?: CreatureHeldWeaponOptionsV1;
      demoAuthoring?: CreatureDemoAuthoringV1;
      runtimeEnvelope?: CreatureRuntimeEnvelopePolicyV1;
      motionPack?: CreatureMotionPackV1;
      materialProfile?: CreatureMaterialProfileV2;
      performancePreset?: CreaturePerformancePresetV1;
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
      materialUvProjectionJson?: string;
      materialProfile?: "AURORA_CLASSIC_SAFE" | "NWN_EE_MTR";
      modelTextureAuthoringJson?: string;
      modelTexturePayloadBlob?: ArrayBuffer;
      modelTexturePayloadDescriptorsJson?: string;
      compatibilityPipeline?: "PLACEABLE_V1_V8";
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
  kind: "HAK" | "MODEL" | "MODULE" | "ITEM_BLUEPRINT" | "PWK" | "WOK" | "SET" | "TEXTURE" | "MATERIAL" | "TEXTURE_INFO" | "TWO_DA" | "JSON_REPORT";
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
        runtimeContract: "M2A_STUDIO_WASM_2026_08_19_V3";
        creatureSourceForward: "CARDINAL_XZ_TO_AURORA_POSITIVE_Y_V2";
        creatureTriangleBudget: 300000;
        creatureEquipment: "COMPLETE_EMBEDDED_GIT_UTC_V2";
        creatureMotionPack: "SOURCE_BOUND_HUMANOID_QUADRUPED_V1";
        creatureMaterials: "ANIMATED_CLASSIC_OR_NWN_EE_MTR_V2";
        referenceSupermodelMotion: "EXACT_REFERENCE_BIND_AND_WEIGHTED_ANCHORS_V3_WITH_MATERIAL_LEDGER";
      };
    }
  | {
      requestId: string;
      ok: true;
      type: "ITEM_INPUTS_INSPECTED";
      sourceStateId: string;
      baseItemJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "ITEM_RECIPE_RESOLVED";
      sourceStateId: string;
      recipeStateId: string;
      resolutionJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "ITEM_PART_COMPILED";
      sourceStateId: string;
      recipeStateId: string;
      reportJson: string;
      readbackJson: string;
      artifacts: WorkerArtifact[];
    }
  | {
      requestId: string;
      ok: true;
      type: "ITEM_ICONS_BUILT";
      sourceStateId: string;
      recipeStateId: string;
      payloadBlob: ArrayBuffer;
      descriptorsJson: string;
      reportsJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "ITEM_PACKAGE_BUILT";
      sourceStateId: string;
      recipeStateId: string;
      artifacts: WorkerArtifact[];
      utiReportJson: string;
      utiReadbackJson: string;
      manifestSha256: string;
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
      type: "CREATURE_MOTION_PACK_INSPECTED";
      reportJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "EXACT_REFERENCE_SUPERMODEL_MOTION_CONTRACT_BUILT";
      motionContractJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "REFERENCE_SUPERMODEL_CORRECTION_BUILT";
      correctionJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "EXACT_REFERENCE_SUPERMODEL_CARRIER_BUILT";
      correctionJson: string;
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
  | { requestId: string; ok: true; type: "NWN_KEY_MODELS_INDEXED"; indexJson: string }
  | { requestId: string; ok: true; type: "HAK_MODELS_INDEXED"; indexJson: string }
  | { requestId: string; ok: true; type: "NWN_BIF_INDEX_PLANNED"; planJson: string }
  | { requestId: string; ok: true; type: "NWN_BIF_TABLE_INDEXED"; indexJson: string }
  | { requestId: string; ok: true; type: "MDL_CATALOG_HEADERS_INSPECTED"; reportJson: string }
  | { requestId: string; ok: true; type: "SUPERMODEL_CATALOG_BUILT"; catalogJson: string }
  | { requestId: string; ok: true; type: "BINARY_MDL_INSPECTED"; reportJson: string }
  | {
      requestId: string;
      ok: true;
      type: "REFERENCE_SUPERMODEL_RIG_PREPARED";
      reportJson: string;
      authoringJson: string;
      targetRigJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "REFERENCE_SUPERMODEL_RIG_V2_PREPARED";
      reportJson: string;
      authoringJson: string;
      targetRigJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "REFERENCE_SUPERMODEL_APPLIED_PREVIEW_BUILT";
      readbackJson: string;
      applyReportJson: string;
      authoringJson: string;
      targetRigJson: string;
      artifacts: WorkerArtifact[];
    }
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
      resultKind?: "REFERENCE_SUPERMODEL";
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
