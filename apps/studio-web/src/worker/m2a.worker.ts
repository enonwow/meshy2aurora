/// <reference lib="webworker" />

import init, {
  buildReferenceSupermodelAppliedPreviewV4,
  buildReferenceSupermodelAuthoredPreviewV3,
  buildReferenceSupermodelCreatureProductV2,
  buildReferenceSupermodelCreatureProductV3,
  buildReferenceSupermodelCreatureProductV5,
  prepareReferenceSupermodelRigV1,
  prepareReferenceSupermodelRigV3,
  prepareReferenceSupermodelAuthoredRigV3,
  validateReferenceSupermodelSealedRigV3,
  buildExactReferenceSupermodelMotionContractV3Json,
  buildExactMotionCarrierRigV3Json,
  buildMotionCorrectedRigV1Json,
  buildM7CorpusBatchV1,
  buildMeshyProceduralHumanoidProductDemoWithOptionsV1,
  buildMeshyProceduralHumanoidProductDemoWithMaterialsV2,
  buildMeshyFullNativeH1PackageWithOptionsV4,
  buildMeshyFullNativeH1PackageWithMaterialsV5,
  buildMeshyProceduralHumanoidP100kExperimentWithOptionsV2,
  buildMeshyProceduralHumanoidP300kExperimentWithOptionsV2,
  buildMeshyM0StaticRigidPackageV1,
  buildMeshyStaticPlaceablePackageV1,
  buildMeshyStaticPlaceablePackageV4,
  buildMeshyStaticPlaceablePackageV5,
  buildMeshyStaticPlaceablePackageV6,
  buildMeshyStaticPlaceablePackageV7,
  buildMeshyStaticPlaceablePackageV8,
  buildMeshyStaticPlaceablePackageV9,
  buildMeshyStaticTilePackageV1,
  buildMeshyStaticTilePackageV2,
  compileItemPartV1,
  ingestGlbJson,
  ingestMeshyP100kExperimentJson,
  ingestMeshyP300kExperimentJson,
  ingestStaticRigidGlbJson,
  inspectHighPolyGlbJson,
  inspectBinaryMdl,
  indexNwnKeyModelsV1Json,
  indexHakModelsV1Json,
  planNwnBifIndexV1Json,
  indexNwnBifTableV1Json,
  inspectMdlCatalogHeadersV1Json,
  buildSupermodelCatalogV1Json,
  inspectCreatureMotionPackSourceV1Json,
  resolveItemBaseRecordV1Json,
  resolveItemRecipeV1Json,
  inspectTwoDaV2Json,
  inspectM7CorpusIntakeV1Json,
  inspectMeshyStaticPlaceableAuthoringV3,
  inspectMeshyStaticPlaceableTexturesV1,
  inspectModelFacesV2Json,
  resolveMeshyStaticPlaceableCollisionV1,
  resolveMeshyStaticPlaceableTexturesV1,
  resolveModelMaterialsV1Json,
  resolveModelMaterialsV2Json,
  studioRuntimeCapabilitiesV1Json,
  validateM7CorpusManifestV1Json,
  writeItemPackageV1,
  writeItemIconLayersV1,
} from "@m2a-wasm";
import type {
  CreatureDemoAuthoringV1,
  CreatureMaterialProfileV2,
  CreatureMotionPackV1,
  CreaturePerformancePresetV1,
  CreatureRuntimeEnvelopePolicyV1,
  StudioWorkerRequest,
  StudioWorkerResponse,
  WorkerArtifact,
} from "./types";

function proceduralBuildOptionsJson(
  textureArtifactCleanup: boolean,
  sourceForward: "POSITIVE_Z" | "NEGATIVE_Z" | "POSITIVE_X" | "NEGATIVE_X",
  skinAccessoryStabilization?: {
    mode: "AUTO" | "KEEP_SOURCE_WEIGHTS" | "SELECT_BONE";
    selectedBoneName?: string;
    componentBoneOverrides?: readonly {
      segmentIndex: number;
      componentIndex: number;
      boneName: string;
    }[];
  },
  weaponGrip?: {
    schemaVersion: 1;
    mode: "AUTO" | "AUTO_PLUS_OFFSETS";
    itemFamily?: "SWORD" | "AXE_MACE" | "SPEAR_POLEARM" | "BOW_CROSSBOW" | "SHIELD";
    rightHand: { rollDegrees: number; pitchDegrees: number; yawDegrees: number };
    leftHand: { rollDegrees: number; pitchDegrees: number; yawDegrees: number };
  },
  heldWeapon?: {
    schemaVersion: 1;
    mode: "NONE" | "RIGHT_HAND" | "LEFT_HAND";
    itemResref?: string;
  },
  demoAuthoring?: CreatureDemoAuthoringV1,
  runtimeEnvelope?: CreatureRuntimeEnvelopePolicyV1,
  motionPack?: CreatureMotionPackV1,
  materialProfile?: CreatureMaterialProfileV2,
  performancePreset?: CreaturePerformancePresetV1,
): string {
  const stabilization = skinAccessoryStabilization ?? { mode: "AUTO" as const };
  const grip = weaponGrip ?? {
    schemaVersion: 1 as const,
    mode: "AUTO" as const,
    rightHand: { rollDegrees: 0, pitchDegrees: 0, yawDegrees: 0 },
    leftHand: { rollDegrees: 0, pitchDegrees: 0, yawDegrees: 0 },
  };
  return JSON.stringify({
    schemaVersion: 1,
    sourceForward,
    textureArtifactCleanup,
    skinAccessoryStabilization: {
      schemaVersion: 2,
      mode: stabilization.mode,
      ...(stabilization.mode === "SELECT_BONE"
        ? {
            ...(stabilization.selectedBoneName?.trim()
              ? { selectedBoneName: stabilization.selectedBoneName.trim() }
              : {}),
            componentBoneOverrides: stabilization.componentBoneOverrides ?? [],
          }
        : {}),
    },
    weaponGrip: grip,
    heldWeapon: heldWeapon ?? { schemaVersion: 1, mode: "NONE" as const },
    ...(demoAuthoring ? { demoAuthoring } : {}),
    ...(runtimeEnvelope ? { runtimeEnvelope } : {}),
    ...(motionPack ? { motionPack } : {}),
    ...(materialProfile ? { materialProfile } : {}),
    ...(performancePreset ? { performancePreset } : {}),
  });
}

function requireHeldWeaponReadback(
  report: {
    heldWeaponReadback?: unknown;
    heldStockWeaponReadback?: {
      weapon?: { resref?: unknown; resourceType?: unknown; resourceScope?: unknown };
      fixtures?: Array<{ hand?: unknown; equippedItemResref?: unknown }>;
    };
  },
  requested: {
    schemaVersion: 1;
    mode: "NONE" | "RIGHT_HAND" | "LEFT_HAND";
    itemResref?: string;
  } | undefined,
) {
  const heldWeapon = requested ?? { schemaVersion: 1 as const, mode: "NONE" as const };
  if (heldWeapon.mode === "NONE") {
    if (report.heldWeaponReadback !== undefined || report.heldStockWeaponReadback !== undefined) {
      throw new Error("Procedural demo unexpectedly contains held-item equipment");
    }
    return;
  }
  const expectedHand = heldWeapon.mode === "RIGHT_HAND" ? "right_hand" : "left_hand";
  const fixture = report.heldStockWeaponReadback?.fixtures?.[0];
  if (report.heldWeaponReadback !== undefined
    || report.heldStockWeaponReadback?.weapon?.resref !== heldWeapon.itemResref
    || report.heldStockWeaponReadback?.weapon?.resourceType !== 2025
    || report.heldStockWeaponReadback?.weapon?.resourceScope !== "NWN_BASE_GAME"
    || report.heldStockWeaponReadback?.fixtures?.length !== 1
    || fixture?.hand !== expectedHand
    || fixture.equippedItemResref !== heldWeapon.itemResref) {
    throw new Error("Procedural demo held-item readback differs from the requested hand slot");
  }
}

function requireCreatureDemoAuthoringReadback(
  report: {
    heldWeaponReadback?: unknown;
    heldStockWeaponReadback?: {
      weapon?: { resref?: unknown; resourceType?: unknown; resourceScope?: unknown };
      fixtures?: Array<{ hand?: unknown; equippedItemResref?: unknown }>;
    };
    authoredCreatureReadback?: {
      blueprint?: unknown;
      loadout?: unknown;
      loadoutReport?: {
        embeddedItemCount?: unknown;
        noEquippedResShortcuts?: unknown;
      };
    };
  },
  demoAuthoring: CreatureDemoAuthoringV1 | undefined,
  legacyHeldWeapon: Parameters<typeof requireHeldWeaponReadback>[1],
) {
  if (!demoAuthoring) {
    requireHeldWeaponReadback(report, legacyHeldWeapon);
    return;
  }
  const readback = report.authoredCreatureReadback;
  if (
    report.heldWeaponReadback !== undefined
    || report.heldStockWeaponReadback !== undefined
    || !readback
    || JSON.stringify(readback.blueprint) !== JSON.stringify(demoAuthoring.blueprint)
    || JSON.stringify(readback.loadout) !== JSON.stringify(demoAuthoring.equipmentLoadout)
    || readback.loadoutReport?.embeddedItemCount !== demoAuthoring.equipmentLoadout.items.length
    || readback.loadoutReport?.noEquippedResShortcuts !== true
  ) {
    throw new Error("Procedural demo authored Creature readback differs from the complete UTC/GIT request");
  }
}

let initialized: Promise<unknown> | undefined;
const ensureInitialized = () => (initialized ??= init());
const EXPECTED_RUNTIME_CAPABILITIES = {
  schemaVersion: 1,
  runtimeContract: "M2A_STUDIO_WASM_2026_08_19_V3",
  creatureSourceForward: "CARDINAL_XZ_TO_AURORA_POSITIVE_Y_V2",
  creatureTriangleBudget: 300000,
  creatureEquipment: "COMPLETE_EMBEDDED_GIT_UTC_V2",
  creatureMotionPack: "SOURCE_BOUND_HUMANOID_QUADRUPED_V1",
  creatureMaterials: "ANIMATED_CLASSIC_OR_NWN_EE_MTR_V2",
  referenceSupermodelMotion:
    "EXACT_REFERENCE_BIND_AND_WEIGHTED_ANCHORS_V3_WITH_MATERIAL_LEDGER",
} as const;

function requireRuntimeCapabilities() {
  const actual = JSON.parse(studioRuntimeCapabilitiesV1Json()) as unknown;
  if (JSON.stringify(actual) !== JSON.stringify(EXPECTED_RUNTIME_CAPABILITIES)) {
    throw new Error(
      `M2A-WASM-RUNTIME-CONTRACT-MISMATCH: expected ${JSON.stringify(EXPECTED_RUNTIME_CAPABILITIES)}, got ${JSON.stringify(actual)}`,
    );
  }
  return EXPECTED_RUNTIME_CAPABILITIES;
}
const encoder = new TextEncoder();
const nativeModelResourcePresentation = (resourceType: number) => {
  switch (resourceType) {
    case 3: return { extension: "tga", mimeType: "image/x-tga" };
    case 2022: return { extension: "txi", mimeType: "text/plain" };
    case 2072: return { extension: "mtr", mimeType: "text/plain" };
    default: return { extension: "bin", mimeType: "application/octet-stream" };
  }
};
const twoDaInspectionLimitsJson = JSON.stringify({
  maxInputBytes: 16_777_216,
  maxColumns: 4_096,
  maxRows: 65_536,
  maxTokenBytes: 1_048_576,
  maxDiagnostics: 2_048,
});
const gffWriterOptionsJson = JSON.stringify({
  schemaVersion: 1,
  limits: {
    maxGffBytes: 67_108_864,
    maxStructs: 65_536,
    maxFields: 262_144,
    maxLabels: 65_536,
    maxFieldsPerStruct: 65_536,
    maxListElements: 65_536,
    maxDepth: 64,
    maxStringBytes: 1_024,
    maxLocStringBytes: 1_048_576,
    maxVoidBytes: 16_777_216,
    maxDiagnostics: 2_048,
  },
});
const archiveWriterOptionsJson = JSON.stringify({
  schemaVersion: 1,
  limits: {
    maxEntryCount: 262_144,
    maxOutputBytes: 268_435_456,
  },
});
const tgaWriterOptionsJson = JSON.stringify({
  schemaVersion: 1,
  limits: {
    maxOutputBytes: 67_108_864,
  },
});

async function sha256(bytes: ArrayBuffer): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function artifact(
  artifactId: string,
  kind: WorkerArtifact["kind"],
  fileName: string,
  mediaType: string,
  bytes: ArrayBuffer,
): Promise<WorkerArtifact> {
  return {
    artifactId,
    kind,
    fileName,
    mediaType,
    byteLength: bytes.byteLength,
    sha256: await sha256(bytes),
    bytes,
    provenance: "M2A_WASM_WORKER",
  };
}

function exactBuffer(bytes: Uint8Array): ArrayBuffer {
  return bytes.slice().buffer;
}

function componentOnlyMaterialSeparationV1Json(documentJson: string): string {
  const document = JSON.parse(documentJson) as {
    schemaVersion?: number;
    sourceSha256?: string;
    materials?: unknown[];
    componentAssignments?: unknown[];
    faceAssignments?: unknown[];
  };
  if (document.schemaVersion === 1) return documentJson;
  if (document.schemaVersion !== 2 || !Array.isArray(document.componentAssignments)
    || !Array.isArray(document.faceAssignments) || document.faceAssignments.length) {
    throw new Error(
      "MODEL-MATERIAL-FACE-MODE-V2-TARGET-BUILD-UNSUPPORTED: this legacy build lane accepts V2 component assignments only",
    );
  }
  return JSON.stringify({
    schemaVersion: 1,
    sourceSha256: document.sourceSha256,
    materials: document.materials ?? [],
    assignments: document.componentAssignments,
  });
}

function componentOnlyLegacyMaterialInputs(
  sourceGlb: Uint8Array,
  target: "CREATURE" | "PLACEABLE" | "TILE" | "MODEL_PART",
  materialSeparationJson: string,
  modelTextureAuthoringJson: string,
) {
  const separation = componentOnlyMaterialSeparationV1Json(materialSeparationJson);
  if (separation === materialSeparationJson) {
    return { materialSeparationJson, modelTextureAuthoringJson };
  }
  const resolution = JSON.parse(resolveModelMaterialsV1Json(
    sourceGlb,
    target,
    separation,
  )) as { textureAuthoring: { separationSha256: string } };
  const textureAuthoring = JSON.parse(modelTextureAuthoringJson) as Record<string, unknown>;
  return {
    materialSeparationJson: separation,
    modelTextureAuthoringJson: JSON.stringify({
      ...textureAuthoring,
      separationSha256: resolution.textureAuthoring.separationSha256,
    }),
  };
}

function rejectCreatureExperimentMaterialInputs(request: StudioWorkerRequest): void {
  const values = request as unknown as Record<string, unknown>;
  const materialInputPresent = [
    "materialSeparationJson",
    "modelTextureAuthoringJson",
    "modelTexturePayloadBlob",
    "modelTexturePayloadDescriptorsJson",
  ].some((key) => values[key] !== undefined);
  if (materialInputPresent) {
    throw new Error(
      "CREATURE-MATERIALS-PROFILE-UNSUPPORTED: Material Separation is available only in the Product 300K profile",
    );
  }
}

async function handle(request: StudioWorkerRequest): Promise<StudioWorkerResponse> {
  await ensureInitialized();
  const runtimeCapabilities = requireRuntimeCapabilities();
  if (request.type === "INITIALIZE") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "INITIALIZED",
      runtimeCapabilities,
    };
  }
  if (request.type === "INSPECT_ITEM_INPUTS") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "ITEM_INPUTS_INSPECTED",
      sourceStateId: request.sourceStateId,
      baseItemJson: resolveItemBaseRecordV1Json(
        new Uint8Array(request.baseItemsTwoDa),
        request.physicalRowIndex,
        request.expectedSourceSha256,
        twoDaInspectionLimitsJson,
      ),
    };
  }
  if (request.type === "RESOLVE_ITEM_RECIPE") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "ITEM_RECIPE_RESOLVED",
      sourceStateId: request.sourceStateId,
      recipeStateId: request.recipeStateId,
      resolutionJson: resolveItemRecipeV1Json(request.recipeJson),
    };
  }
  if (request.type === "COMPILE_ITEM_PART") {
    const result = compileItemPartV1(request.requestJson);
    const reportJson = result.reportJson;
    const readbackJson = result.readbackJson;
    const mdlBytes = exactBuffer(result.takeMdlBytes());
    result.free();
    const report = JSON.parse(reportJson) as { modelResref?: string };
    if (!report.modelResref || !/^[a-z0-9_]{1,16}$/.test(report.modelResref)) {
      throw new Error("M2A-ITEM-PART-RESREF-INVALID");
    }
    return {
      requestId: request.requestId,
      ok: true,
      type: "ITEM_PART_COMPILED",
      sourceStateId: request.sourceStateId,
      recipeStateId: request.recipeStateId,
      reportJson,
      readbackJson,
      artifacts: await Promise.all([
        artifact(
          `item-part-${report.modelResref}`,
          "MODEL",
          `${report.modelResref}.mdl`,
          "application/octet-stream",
          mdlBytes,
        ),
        artifact(
          `item-part-${report.modelResref}-report`,
          "JSON_REPORT",
          `${report.modelResref}-report.json`,
          "application/json",
          exactBuffer(encoder.encode(reportJson)),
        ),
      ]),
    };
  }
  if (request.type === "BUILD_ITEM_ICONS") {
    const result = writeItemIconLayersV1(
      request.recipeJson,
      request.layersJson,
      tgaWriterOptionsJson,
    );
    const descriptorsJson = result.descriptorsJson;
    const reportsJson = result.reportsJson;
    const payloadBlob = exactBuffer(result.takePayloadBlob());
    result.free();
    return {
      requestId: request.requestId,
      ok: true,
      type: "ITEM_ICONS_BUILT",
      sourceStateId: request.sourceStateId,
      recipeStateId: request.recipeStateId,
      payloadBlob,
      descriptorsJson,
      reportsJson,
    };
  }
  if (request.type === "BUILD_ITEM_PACKAGE") {
    if (!/^[a-z0-9_]{1,16}$/.test(request.outputStem)) {
      throw new Error("M2A-ITEM-OUTPUT-STEM-INVALID");
    }
    const result = writeItemPackageV1(
      new Uint8Array(request.payloadBlob),
      request.requestJson,
      gffWriterOptionsJson,
      archiveWriterOptionsJson,
    );
    const utiReportJson = result.utiReportJson;
    const utiReadbackJson = result.utiReadbackJson;
    const manifestSha256 = result.manifestSha256;
    const utiBytes = exactBuffer(result.takeUtiBytes());
    const hakBytes = exactBuffer(result.takeHakBytes());
    const moduleBytes = exactBuffer(result.takeModuleBytes());
    const manifestBytes = exactBuffer(result.takeManifestBytes());
    result.free();
    const requestValue = JSON.parse(request.requestJson) as {
      recipe?: { identity?: { utiResref?: string } };
    };
    const utiResref = requestValue.recipe?.identity?.utiResref;
    if (!utiResref || !/^[a-z0-9_]{1,16}$/.test(utiResref)) {
      throw new Error("M2A-ITEM-UTI-RESREF-INVALID");
    }
    return {
      requestId: request.requestId,
      ok: true,
      type: "ITEM_PACKAGE_BUILT",
      sourceStateId: request.sourceStateId,
      recipeStateId: request.recipeStateId,
      utiReportJson,
      utiReadbackJson,
      manifestSha256,
      artifacts: await Promise.all([
        artifact(
          "item-blueprint-uti",
          "ITEM_BLUEPRINT",
          `${utiResref}.uti`,
          "application/octet-stream",
          utiBytes,
        ),
        artifact(
          "item-package-hak",
          "HAK",
          `${request.outputStem}.hak`,
          "application/octet-stream",
          hakBytes,
        ),
        artifact(
          "item-test-module",
          "MODULE",
          `${request.outputStem}.mod`,
          "application/octet-stream",
          moduleBytes,
        ),
        artifact(
          "item-package-manifest",
          "JSON_REPORT",
          `${request.outputStem}-manifest.json`,
          "application/json",
          manifestBytes,
        ),
      ]),
    };
  }
  if (request.type === "INSPECT_MODEL_COMPONENTS") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "MODEL_COMPONENTS_INSPECTED",
      sourceStateId: request.sourceStateId,
      inspectionJson: inspectModelFacesV2Json(
        new Uint8Array(request.sourceGlb),
        request.target,
      ),
    };
  }
  if (request.type === "RESOLVE_MODEL_MATERIALS") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "MODEL_MATERIALS_RESOLVED",
      sourceStateId: request.sourceStateId,
      recipeStateId: request.recipeStateId,
      resolutionJson: resolveModelMaterialsV2Json(
        new Uint8Array(request.sourceGlb),
        request.target,
        request.documentJson,
      ),
    };
  }
  if (request.type === "INSPECT_SOURCE") {
    const ingestJson = request.target === "PLACEABLE" || request.target === "TILE"
      ? ingestStaticRigidGlbJson(new Uint8Array(request.sourceGlb))
      : request.unsafeHighPolyInspection
        ? inspectHighPolyGlbJson(new Uint8Array(request.sourceGlb))
      : request.creatureProfile === "EXPERIMENTAL_P300K"
        ? ingestMeshyP300kExperimentJson(new Uint8Array(request.sourceGlb))
      : request.creatureProfile === "EXPERIMENTAL_P100K"
        ? ingestMeshyP100kExperimentJson(new Uint8Array(request.sourceGlb))
      : ingestGlbJson(new Uint8Array(request.sourceGlb));
    const optionsJson = JSON.stringify({
      schemaVersion: 1,
      experimentalAggressiveGeometryCleanup:
        request.experimentalAggressiveGeometryCleanup ?? false,
    });
    const placeableAuthoringJson = request.target === "PLACEABLE"
      ? inspectMeshyStaticPlaceableAuthoringV3(
          new Uint8Array(request.sourceGlb),
          optionsJson,
        )
      : undefined;
    const placeableTexturesJson = request.target === "PLACEABLE"
      ? inspectMeshyStaticPlaceableTexturesV1(
          new Uint8Array(request.sourceGlb),
          optionsJson,
        )
      : undefined;
    return {
      requestId: request.requestId,
      ok: true,
      type: "SOURCE_INSPECTED",
      ingestJson,
      placeableAuthoringJson,
      placeableTexturesJson,
      placeableCollisionJson: placeableAuthoringJson && request.modelResref
        ? resolveMeshyStaticPlaceableCollisionV1(
            new Uint8Array(request.sourceGlb),
            request.modelResref,
            JSON.stringify((JSON.parse(placeableAuthoringJson) as { document: unknown }).document),
            optionsJson,
          )
        : undefined,
    };
  }
  if (request.type === "INSPECT_CREATURE_MOTION_PACK") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "CREATURE_MOTION_PACK_INSPECTED",
      reportJson: inspectCreatureMotionPackSourceV1Json(
        new Uint8Array(request.sourceGlb),
        JSON.stringify(request.motionPack),
      ),
    };
  }
  if (request.type === "BUILD_EXACT_REFERENCE_SUPERMODEL_MOTION_CONTRACT") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "EXACT_REFERENCE_SUPERMODEL_MOTION_CONTRACT_BUILT",
      motionContractJson: buildExactReferenceSupermodelMotionContractV3Json(
        new Uint8Array(request.referenceMdl),
        request.optionsJson,
      ),
    };
  }
  if (request.type === "BUILD_REFERENCE_SUPERMODEL_CORRECTION") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "REFERENCE_SUPERMODEL_CORRECTION_BUILT",
      correctionJson: buildMotionCorrectedRigV1Json(
        request.targetRigJson,
        request.motionContractJson,
      ),
    };
  }
  if (request.type === "BUILD_EXACT_REFERENCE_SUPERMODEL_CARRIER") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "EXACT_REFERENCE_SUPERMODEL_CARRIER_BUILT",
      correctionJson: buildExactMotionCarrierRigV3Json(
        request.targetRigJson,
        request.motionContractJson,
        new Uint8Array(request.referenceMdl),
      ),
    };
  }
  if (request.type === "PREPARE_REFERENCE_SUPERMODEL_RIG") {
    const result = prepareReferenceSupermodelRigV1(
      request.selectedSupermodelResref,
      new Uint8Array(request.sourceGlb),
      new Uint8Array(request.referenceChainBlob),
      request.referenceChainJson,
      request.sourceForward,
    );
    try {
      return {
        requestId: request.requestId,
        ok: true,
        type: "REFERENCE_SUPERMODEL_RIG_PREPARED",
        reportJson: result.reportJson,
        authoringJson: result.authoringJson,
        targetRigJson: result.targetRigJson,
      };
    } finally {
      result.free();
    }
  }
  if (request.type === "PREPARE_REFERENCE_SUPERMODEL_RIG_V2") {
    const args = [
      request.selectedSupermodelResref,
      new Uint8Array(request.sourceGlb),
      new Uint8Array(request.referenceChainBlob),
      request.referenceChainJson,
      request.sourceForward,
    ] as const;
    const result = request.authoringJson
      ? request.authoringIsSealed
        ? validateReferenceSupermodelSealedRigV3(
            ...args,
            request.authoringJson,
            request.experimentalAllowExcessiveSkinBranchRepair === true,
          )
        : prepareReferenceSupermodelAuthoredRigV3(
            ...args,
            request.authoringJson,
            request.experimentalAllowExcessiveSkinBranchRepair === true,
          )
      : prepareReferenceSupermodelRigV3(
          ...args,
          request.experimentalAllowExcessiveSkinBranchRepair === true,
        );
    try {
      return {
        requestId: request.requestId,
        ok: true,
        type: "REFERENCE_SUPERMODEL_RIG_V2_PREPARED",
        reportJson: result.reportJson,
        authoringJson: result.authoringJson,
        targetRigJson: result.targetRigJson,
      };
    } finally {
      result.free();
    }
  }
  if (
    request.type === "BUILD_REFERENCE_SUPERMODEL_APPLIED_PREVIEW"
    || request.type === "BUILD_REFERENCE_SUPERMODEL_AUTHORED_PREVIEW"
  ) {
    const result = request.type === "BUILD_REFERENCE_SUPERMODEL_AUTHORED_PREVIEW"
      ? buildReferenceSupermodelAuthoredPreviewV3(
          request.selectedSupermodelResref,
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.referenceChainBlob),
          request.referenceChainJson,
          request.sourceForward,
          request.authoringJson,
          request.experimentalAllowExcessiveSkinBranchRepair === true,
        )
      : buildReferenceSupermodelAppliedPreviewV4(
          request.selectedSupermodelResref,
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.referenceChainBlob),
          request.referenceChainJson,
          request.sourceForward,
          request.experimentalAllowExcessiveSkinBranchRepair === true,
        );
    try {
      const modelBytes = exactBuffer(result.takeModelBytes());
      return {
        requestId: request.requestId,
        ok: true,
        type: "REFERENCE_SUPERMODEL_APPLIED_PREVIEW_BUILT",
        readbackJson: result.readbackJson,
        applyReportJson: result.applyReportJson,
        authoringJson: result.authoringJson,
        targetRigJson: result.targetRigJson,
        artifacts: [await artifact(
          "reference-supermodel-diagnostic-preview-model",
          "MODEL",
          "m2a_refpreview.mdl",
          "application/octet-stream",
          modelBytes,
        )],
      };
    } finally {
      result.free();
    }
  }
  if (request.type === "RESOLVE_PLACEABLE_COLLISION") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "PLACEABLE_COLLISION_RESOLVED",
      collisionJson: resolveMeshyStaticPlaceableCollisionV1(
        new Uint8Array(request.sourceGlb),
        request.modelResref,
        request.authoringJson,
        JSON.stringify({
          schemaVersion: 1,
          experimentalAggressiveGeometryCleanup:
            request.experimentalAggressiveGeometryCleanup ?? false,
        }),
      ),
    };
  }
  if (request.type === "RESOLVE_PLACEABLE_TEXTURES") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "PLACEABLE_TEXTURES_RESOLVED",
      texturesJson: resolveMeshyStaticPlaceableTexturesV1(
        new Uint8Array(request.sourceGlb),
        request.baseTextureResref,
        request.geometryAuthoringJson,
        request.textureAuthoringJson,
        new Uint8Array(request.texturePayloadBlob),
        request.texturePayloadDescriptorsJson,
        JSON.stringify({
          schemaVersion: 1,
          experimentalAggressiveGeometryCleanup:
            request.experimentalAggressiveGeometryCleanup ?? false,
        }),
      ),
    };
  }
  if (request.type === "INSPECT_APPEARANCE") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "APPEARANCE_INSPECTED",
      inspectionJson: inspectTwoDaV2Json(
        new Uint8Array(request.appearanceTwoDa),
        twoDaInspectionLimitsJson,
      ),
    };
  }
  if (request.type === "INDEX_NWN_KEY_MODELS") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "NWN_KEY_MODELS_INDEXED",
      indexJson: indexNwnKeyModelsV1Json(new Uint8Array(request.keyBytes)),
    };
  }
  if (request.type === "INDEX_HAK_MODELS") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "HAK_MODELS_INDEXED",
      indexJson: indexHakModelsV1Json(new Uint8Array(request.hakBytes)),
    };
  }
  if (request.type === "PLAN_NWN_BIF_INDEX") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "NWN_BIF_INDEX_PLANNED",
      planJson: planNwnBifIndexV1Json(new Uint8Array(request.headerBytes)),
    };
  }
  if (request.type === "INDEX_NWN_BIF_TABLE") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "NWN_BIF_TABLE_INDEXED",
      indexJson: indexNwnBifTableV1Json(
        new Uint8Array(request.headerBytes),
        new Uint8Array(request.tableBytes),
      ),
    };
  }
  if (request.type === "INSPECT_MDL_CATALOG_HEADERS") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "MDL_CATALOG_HEADERS_INSPECTED",
      reportJson: inspectMdlCatalogHeadersV1Json(
        new Uint8Array(request.payloadBlob),
        request.descriptorsJson,
      ),
    };
  }
  if (request.type === "BUILD_SUPERMODEL_CATALOG") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "SUPERMODEL_CATALOG_BUILT",
      catalogJson: buildSupermodelCatalogV1Json(request.inputJson),
    };
  }
  if (request.type === "INSPECT_BINARY_MDL") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "BINARY_MDL_INSPECTED",
      reportJson: inspectBinaryMdl(new Uint8Array(request.mdlBytes)),
    };
  }
  if (request.type === "VALIDATE_M7_CORPUS") {
    const manifestJson = validateM7CorpusManifestV1Json(request.manifestJson);
    return {
      requestId: request.requestId,
      ok: true,
      type: "M7_CORPUS_VALIDATED",
      manifestJson,
      artifacts: [await artifact(
        "m7-manifest-validation-json",
        "JSON_REPORT",
        "m7-manifest-validation.json",
        "application/json",
        encoder.encode(manifestJson).buffer,
      )],
    };
  }
  if (request.type === "INSPECT_M7_CORPUS_INTAKE") {
    const intakeJson = inspectM7CorpusIntakeV1Json(
      request.manifestJson,
      new Uint8Array(request.payloadBlob),
      request.descriptorsJson,
    );
    return {
      requestId: request.requestId,
      ok: true,
      type: "M7_CORPUS_INTAKE_INSPECTED",
      intakeJson,
      artifacts: [await artifact(
        "m7-intake-json",
        "JSON_REPORT",
        "m7-intake.json",
        "application/json",
        encoder.encode(intakeJson).buffer,
      )],
    };
  }
  if (request.type === "BUILD_M7_CORPUS_BATCH") {
    const batchJson = buildM7CorpusBatchV1(
      request.manifestJson,
      new Uint8Array(request.payloadBlob),
      request.descriptorsJson,
    );
    return {
      requestId: request.requestId,
      ok: true,
      type: "M7_CORPUS_BATCH_BUILT",
      batchJson,
      artifacts: [await artifact(
        "m7-batch-json",
        "JSON_REPORT",
        "m7-batch.json",
        "application/json",
        encoder.encode(batchJson).buffer,
      )],
    };
  }
  if (request.type === "BUILD_PLACEABLE_PACKAGE") {
    const textureInputsPresent = request.textureAuthoringJson !== undefined
      || request.texturePayloadBlob !== undefined
      || request.texturePayloadDescriptorsJson !== undefined;
    if (textureInputsPresent && (
      request.authoringJson === undefined
      || request.textureAuthoringJson === undefined
      || request.texturePayloadBlob === undefined
      || request.texturePayloadDescriptorsJson === undefined
    )) {
      throw new Error("PLACEABLE-TEXTURE-BUILD-INPUT-INCOMPLETE");
    }
    const materialInputsPresent = request.materialSeparationJson !== undefined
      || request.materialUvProjectionJson !== undefined
      || request.modelTextureAuthoringJson !== undefined
      || request.modelTexturePayloadBlob !== undefined
      || request.modelTexturePayloadDescriptorsJson !== undefined;
    if (materialInputsPresent && request.materialProfile === undefined && (
      request.authoringJson === undefined
      || request.materialSeparationJson === undefined
      || request.modelTextureAuthoringJson === undefined
      || request.modelTexturePayloadBlob === undefined
      || request.modelTexturePayloadDescriptorsJson === undefined
    )) {
      throw new Error("PLACEABLE-MATERIAL-SEPARATION-BUILD-INPUT-INCOMPLETE");
    }
    if (materialInputsPresent && request.textureAuthoringJson) {
      const legacyTextureDocument = JSON.parse(request.textureAuthoringJson) as {
        bindings?: { mode?: string }[];
      };
      if (legacyTextureDocument.bindings?.some((binding) => binding.mode === "OVERRIDE")) {
        throw new Error("PLACEABLE-MATERIAL-SEPARATION-LEGACY-TEXTURE-OVERRIDE-CONFLICT");
      }
    }
    const buildOptionsJson = JSON.stringify({
      schemaVersion: 1,
      experimentalAggressiveGeometryCleanup:
        request.experimentalAggressiveGeometryCleanup ?? false,
    });
    const v9InputsComplete = request.materialProfile !== undefined
      && request.materialSeparationJson !== undefined
      && request.authoringJson !== undefined;
    if (!v9InputsComplete && request.compatibilityPipeline !== "PLACEABLE_V1_V8") {
      throw new Error("PLACEABLE-V9-BUILD-INPUT-INCOMPLETE");
    }
    const result = v9InputsComplete
      ? buildMeshyStaticPlaceablePackageV9(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.placeablesTwoDa),
          request.identityJson,
          request.placementJson,
          request.paletteId,
          request.authoringJson!,
          request.materialSeparationJson!,
          request.materialUvProjectionJson ?? "",
          request.modelTextureAuthoringJson ?? "",
          new Uint8Array(request.modelTexturePayloadBlob ?? new ArrayBuffer(0)),
          request.modelTexturePayloadDescriptorsJson ?? "[]",
          JSON.stringify(request.materialProfile!),
          buildOptionsJson,
        )
      : request.materialSeparationJson !== undefined
      && request.modelTextureAuthoringJson !== undefined
      && request.modelTexturePayloadBlob !== undefined
      && request.modelTexturePayloadDescriptorsJson !== undefined
      && request.authoringJson !== undefined
      ? (JSON.parse(request.materialSeparationJson) as { schemaVersion?: number }).schemaVersion === 2
        ? request.materialUvProjectionJson !== undefined
          ? buildMeshyStaticPlaceablePackageV8(
            new Uint8Array(request.sourceGlb),
            new Uint8Array(request.placeablesTwoDa),
            request.identityJson,
            request.placementJson,
            request.paletteId,
            request.authoringJson,
            request.materialSeparationJson,
            request.materialUvProjectionJson,
            request.modelTextureAuthoringJson,
            new Uint8Array(request.modelTexturePayloadBlob),
            request.modelTexturePayloadDescriptorsJson,
            buildOptionsJson,
          )
          : buildMeshyStaticPlaceablePackageV7(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.placeablesTwoDa),
          request.identityJson,
          request.placementJson,
          request.paletteId,
          request.authoringJson,
          request.materialSeparationJson,
          request.modelTextureAuthoringJson,
          new Uint8Array(request.modelTexturePayloadBlob),
          request.modelTexturePayloadDescriptorsJson,
          buildOptionsJson,
          )
        : buildMeshyStaticPlaceablePackageV6(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.placeablesTwoDa),
          request.identityJson,
          request.placementJson,
          request.paletteId,
          request.authoringJson,
          request.materialSeparationJson,
          request.modelTextureAuthoringJson,
          new Uint8Array(request.modelTexturePayloadBlob),
          request.modelTexturePayloadDescriptorsJson,
          buildOptionsJson,
        )
      : request.textureAuthoringJson !== undefined
      && request.texturePayloadBlob !== undefined
      && request.texturePayloadDescriptorsJson !== undefined
      && request.authoringJson !== undefined
      ? buildMeshyStaticPlaceablePackageV5(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.placeablesTwoDa),
          request.identityJson,
          request.placementJson,
          request.paletteId,
          request.authoringJson,
          request.textureAuthoringJson,
          new Uint8Array(request.texturePayloadBlob),
          request.texturePayloadDescriptorsJson,
          buildOptionsJson,
        )
      : request.authoringJson
      ? buildMeshyStaticPlaceablePackageV4(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.placeablesTwoDa),
          request.identityJson,
          request.placementJson,
          request.paletteId,
          request.authoringJson,
          buildOptionsJson,
        )
      : buildMeshyStaticPlaceablePackageV1(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.placeablesTwoDa),
          request.identityJson,
          request.placementJson,
          request.paletteId,
        );
    try {
      const report = JSON.parse(result.reportJson) as {
        moduleFileName: string;
        hakFileName: string;
        modelResref: string;
      };
      const texturePayloadBlob = exactBuffer(result.takeTexturePayloadBlob());
      const textureDescriptors = JSON.parse(result.textureDescriptorsJson) as {
        schemaVersion: number;
        resref: string;
        resourceType: number;
        byteOffset: number;
        byteLength: number;
        sha256: string;
      }[];
      const hak = exactBuffer(result.takeHakBytes());
      const model = exactBuffer(result.takeModelBytes());
      const pwk = exactBuffer(result.takePwkBytes());
      const module = exactBuffer(result.takeProofModuleBytes());
      const reportBytes = encoder.encode(result.reportJson).buffer;
      const readbackJson = result.readbackJson;
      const readbackBytes = encoder.encode(readbackJson).buffer;
      const textureArtifacts = textureDescriptors.map((descriptor) => {
        const bytes = texturePayloadBlob.slice(
          descriptor.byteOffset,
          descriptor.byteOffset + descriptor.byteLength,
        );
        const format = descriptor.resourceType === 2072
          ? { kind: "MATERIAL" as const, extension: "mtr", mediaType: "text/plain" }
          : descriptor.resourceType === 2022
            ? { kind: "TEXTURE_INFO" as const, extension: "txi", mediaType: "text/plain" }
            : descriptor.resourceType === 2033
              ? { kind: "TEXTURE" as const, extension: "dds", mediaType: "image/vnd-ms.dds" }
              : { kind: "TEXTURE" as const, extension: "tga", mediaType: "image/x-tga" };
        const artifactId = descriptor.resourceType === 3
          ? `placeable-texture-${descriptor.resref}-tga`
          : `placeable-material-resource-${descriptor.resref}-${descriptor.resourceType}`;
        return artifact(
          artifactId,
          format.kind,
          `${descriptor.resref}.${format.extension}`,
          format.mediaType,
          bytes,
        ).then((item) => {
          if (item.sha256 !== descriptor.sha256) {
            throw new Error(`PLACEABLE-TEXTURE-ARTIFACT-HASH-MISMATCH: ${descriptor.resref}`);
          }
          return item;
        });
      });
      const artifacts = await Promise.all([
        artifact("placeable-package-hak", "HAK", report.hakFileName, "application/octet-stream", hak),
        artifact("placeable-model-mdl", "MODEL", `${report.modelResref}.mdl`, "application/octet-stream", model),
        artifact("placeable-walkmesh-pwk", "PWK", `${report.modelResref}.pwk`, "text/plain", pwk),
        artifact("placeable-proof-module", "MODULE", report.moduleFileName, "application/octet-stream", module),
        artifact(
          "placeable-report-json",
          "JSON_REPORT",
          "placeable-materialization-report.json",
          "application/json",
          reportBytes,
        ),
        artifact(
          "placeable-model-readback-json",
          "JSON_REPORT",
          "placeable-model-readback.json",
          "application/json",
          readbackBytes,
        ),
        ...(request.authoringJson ? [artifact(
          "placeable-authoring-v2-json",
          "JSON_REPORT",
          "placeable-authoring-v2.json",
          "application/json",
          encoder.encode(request.authoringJson).buffer,
        )] : []),
        ...(request.authoringJson && request.textureAuthoringJson ? [artifact(
          "placeable-authoring-v3-json",
          "JSON_REPORT",
          "placeable-authoring-v3.json",
          "application/json",
          encoder.encode(JSON.stringify({
            schemaVersion: 3,
            geometry: JSON.parse(request.authoringJson),
            textures: JSON.parse(request.textureAuthoringJson),
          }, null, 2)).buffer,
        )] : []),
        ...(request.materialSeparationJson ? [artifact(
          "model-material-separation-v1-json",
          "JSON_REPORT",
          "model-material-separation-v1.json",
          "application/json",
          encoder.encode(request.materialSeparationJson).buffer,
        )] : []),
        ...(request.materialUvProjectionJson ? [artifact(
          "model-material-uv-projection-v1-json",
          "JSON_REPORT",
          "model-material-uv-projection-v1.json",
          "application/json",
          encoder.encode(request.materialUvProjectionJson).buffer,
        )] : []),
        ...(request.modelTextureAuthoringJson ? [artifact(
          "model-texture-authoring-v1-json",
          "JSON_REPORT",
          "model-texture-authoring-v1.json",
          "application/json",
          encoder.encode(request.modelTextureAuthoringJson).buffer,
        )] : []),
        ...textureArtifacts,
      ]);
      return {
        requestId: request.requestId,
        ok: true,
        type: "PLACEABLE_PACKAGE_BUILT",
        artifacts,
        reportJson: result.reportJson,
        readbackJson,
      };
    } finally {
      result.free();
    }
  }
  if (request.type === "BUILD_TILE_PACKAGE") {
    const materialInputsPresent = request.materialSeparationJson !== undefined
      || request.modelTextureAuthoringJson !== undefined
      || request.modelTexturePayloadBlob !== undefined
      || request.modelTexturePayloadDescriptorsJson !== undefined;
    if (materialInputsPresent && (
      request.materialSeparationJson === undefined
      || request.modelTextureAuthoringJson === undefined
      || request.modelTexturePayloadBlob === undefined
      || request.modelTexturePayloadDescriptorsJson === undefined
    )) throw new Error("TILE-MATERIAL-SEPARATION-BUILD-INPUT-INCOMPLETE");
    const tileMaterialInputs = materialInputsPresent
      ? componentOnlyLegacyMaterialInputs(
          new Uint8Array(request.sourceGlb),
          "TILE",
          request.materialSeparationJson!,
          request.modelTextureAuthoringJson!,
        )
      : undefined;
    const result = materialInputsPresent
      ? buildMeshyStaticTilePackageV2(
          new Uint8Array(request.sourceGlb),
          request.optionsJson,
          tileMaterialInputs!.materialSeparationJson,
          tileMaterialInputs!.modelTextureAuthoringJson,
          new Uint8Array(request.modelTexturePayloadBlob!),
          request.modelTexturePayloadDescriptorsJson!,
        )
      : buildMeshyStaticTilePackageV1(
          new Uint8Array(request.sourceGlb),
          request.optionsJson,
        );
    try {
      const report = JSON.parse(result.reportJson) as {
        moduleFileName: string;
        hakFileName: string;
        modelResref: string;
        wokResref: string;
        tilesetResref: string;
        textureResref: string;
        imageMapResref: string;
      };
      const texturePayloadBlob = exactBuffer(result.takeTexturePayloadBlob());
      const textureDescriptors = JSON.parse(result.textureDescriptorsJson) as Array<{
        resref: string;
        byteOffset: number;
        byteLength: number;
      }>;
      const textureArtifacts = textureDescriptors.map((descriptor, index) => {
        const start = descriptor.byteOffset;
        const end = start + descriptor.byteLength;
        if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end)
          || start < 0 || end < start || end > texturePayloadBlob.byteLength) {
          throw new Error(`TILE-TEXTURE-DESCRIPTOR-RANGE-INVALID: textures[${index}]`);
        }
        return artifact(
          index === 0 ? "tile-texture-tga" : `tile-texture-${index}`,
          "TEXTURE",
          `${descriptor.resref}.tga`,
          "image/x-tga",
          texturePayloadBlob.slice(start, end),
        );
      });
      const artifacts = await Promise.all([
        artifact(
          "tile-package-hak",
          "HAK",
          report.hakFileName,
          "application/octet-stream",
          exactBuffer(result.takeHakBytes()),
        ),
        artifact(
          "tile-proof-module",
          "MODULE",
          report.moduleFileName,
          "application/octet-stream",
          exactBuffer(result.takeModuleBytes()),
        ),
        artifact(
          "tile-model-mdl",
          "MODEL",
          `${report.modelResref}.mdl`,
          "application/octet-stream",
          exactBuffer(result.takeModelBytes()),
        ),
        artifact(
          "tile-navigation-wok",
          "WOK",
          `${report.wokResref}.wok`,
          "text/plain",
          exactBuffer(result.takeWokBytes()),
        ),
        artifact(
          "tile-tileset-set",
          "SET",
          `${report.tilesetResref}.set`,
          "text/plain",
          exactBuffer(result.takeSetBytes()),
        ),
        ...textureArtifacts,
        artifact(
          "tile-image-map-tga",
          "TEXTURE",
          `${report.imageMapResref}.tga`,
          "image/x-tga",
          exactBuffer(result.takeImageMapBytes()),
        ),
        artifact(
          "tile-report-json",
          "JSON_REPORT",
          "tile-materialization-report.json",
          "application/json",
          encoder.encode(result.reportJson).buffer,
        ),
        artifact(
          "tile-model-readback-json",
          "JSON_REPORT",
          "tile-model-readback.json",
          "application/json",
          encoder.encode(result.modelReadbackJson).buffer,
        ),
        artifact(
          "tile-wok-readback-json",
          "JSON_REPORT",
          "tile-wok-readback.json",
          "application/json",
          encoder.encode(result.wokReadbackJson).buffer,
        ),
        artifact(
          "tile-set-readback-json",
          "JSON_REPORT",
          "tile-set-readback.json",
          "application/json",
          encoder.encode(result.setReadbackJson).buffer,
        ),
      ]);
      return {
        requestId: request.requestId,
        ok: true,
        type: "TILE_PACKAGE_BUILT",
        artifacts,
        reportJson: result.reportJson,
        modelReadbackJson: result.modelReadbackJson,
        wokReadbackJson: result.wokReadbackJson,
        setReadbackJson: result.setReadbackJson,
      };
    } finally {
      result.free();
    }
  }

  if (
    request.type === "BUILD_MODEL_PACKAGE"
    && request.packageLane === "REFERENCE_SUPERMODEL_CREATURE"
  ) {
    const identity = JSON.parse(request.identityJson) as {
      modelResref?: unknown;
      textureResref?: unknown;
      materialResref?: unknown;
      hakResref?: unknown;
    };
    if (
      typeof identity.modelResref !== "string"
      || typeof identity.textureResref !== "string"
      || typeof identity.materialResref !== "string"
      || typeof identity.hakResref !== "string"
    ) {
      throw new Error("M2A-REFERENCE-SUPERMODEL-PRODUCT-IDENTITY-INVALID");
    }
    const authoringSchemaVersion = request.rigAuthoringJson
      ? (JSON.parse(request.rigAuthoringJson) as { schemaVersion?: unknown }).schemaVersion
      : undefined;
    const result = request.rigAuthoringJson
      ? authoringSchemaVersion === 2
        ? buildReferenceSupermodelCreatureProductV5(
          request.selectedSupermodelResref,
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.appearanceTwoDa),
          new Uint8Array(request.referenceChainBlob),
          request.referenceChainJson,
          request.identityJson,
          request.sourceForward,
          request.rigAuthoringJson,
          request.experimentalAllowExcessiveSkinBranchRepair === true,
        )
        : buildReferenceSupermodelCreatureProductV3(
          request.selectedSupermodelResref,
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.appearanceTwoDa),
          new Uint8Array(request.referenceChainBlob),
          request.referenceChainJson,
          request.identityJson,
          request.sourceForward,
          request.rigAuthoringJson,
        )
      : buildReferenceSupermodelCreatureProductV2(
          request.selectedSupermodelResref,
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.appearanceTwoDa),
          new Uint8Array(request.referenceChainBlob),
          request.referenceChainJson,
          request.identityJson,
          request.sourceForward,
        );
    try {
      const artifacts = await Promise.all([
        artifact("package-hak", "HAK", `${identity.hakResref}.hak`, "application/octet-stream", exactBuffer(result.takeHakBytes())),
        artifact("model-mdl", "MODEL", `${identity.modelResref}.mdl`, "application/octet-stream", exactBuffer(result.takeModelBytes())),
        artifact("texture-tga", "TEXTURE", `${identity.textureResref}.tga`, "image/x-tga", exactBuffer(result.takeTextureBytes())),
        artifact("material-mtr", "MATERIAL", `${identity.materialResref}.mtr`, "text/plain", exactBuffer(result.takeMaterialBytes())),
        artifact("appearance-2da", "TWO_DA", "appearance.2da", "text/plain", exactBuffer(result.takeAppearanceTwoDaBytes())),
        artifact("report-json", "JSON_REPORT", "inspection.json", "application/json", encoder.encode(result.reportJson).buffer),
        artifact("manifest-json", "JSON_REPORT", "conversion-manifest.json", "application/json", encoder.encode(result.manifestJson).buffer),
        artifact("summary-json", "JSON_REPORT", "summary.json", "application/json", encoder.encode(result.summaryJson).buffer),
      ]);
      return {
        requestId: request.requestId,
        ok: true,
        type: "MODEL_PACKAGE_BUILT",
        resultKind: "REFERENCE_SUPERMODEL",
        artifacts,
        reportJson: result.reportJson,
        manifestJson: result.manifestJson,
        summaryJson: result.summaryJson,
        readbackJson: result.readbackJson,
      };
    } finally {
      result.free();
    }
  }

  if (
    request.type === "BUILD_MODEL_PACKAGE"
    && (
      request.packageLane === "SKINNED_PROCEDURAL_HUMANOID_42"
      || request.packageLane === "H1_SKINNED_FULL_42"
      || request.packageLane === "H1_SKINNED_FULL_42_EVENTS"
    )
  ) {
    const sourceGlb = new Uint8Array(request.sourceGlb);
    const appearanceTwoDa = new Uint8Array(request.appearanceTwoDa);
    const buildOptionsJson = proceduralBuildOptionsJson(
      request.textureArtifactCleanup,
      request.sourceForward,
      request.skinAccessoryStabilization,
      request.weaponGrip,
      request.heldWeapon,
      request.demoAuthoring,
      request.runtimeEnvelope,
      request.motionPack,
      request.materialProfile,
      request.performancePreset,
    );
    const materialInputs = [
      request.materialSeparationJson,
      request.modelTextureAuthoringJson,
      request.modelTexturePayloadBlob,
      request.modelTexturePayloadDescriptorsJson,
    ];
    const materialInputCount = materialInputs.filter((value) => value !== undefined).length;
    if (materialInputCount !== 0 && materialInputCount !== materialInputs.length) {
      throw new Error("MODEL-MATERIAL-INPUTS-INCOMPLETE: Creature requires all material and texture inputs together");
    }
    const materialInputsPresent = materialInputCount === materialInputs.length;
    const buildMaterialInputs = materialInputsPresent
      ? {
          materialSeparationJson: request.materialSeparationJson!,
          modelTextureAuthoringJson: request.modelTextureAuthoringJson!,
        }
      : undefined;
    const modelTexturePayload = new Uint8Array(request.modelTexturePayloadBlob ?? new ArrayBuffer(0));
    const result = request.packageLane === "H1_SKINNED_FULL_42"
      || request.packageLane === "H1_SKINNED_FULL_42_EVENTS"
      ? materialInputsPresent
        ? buildMeshyFullNativeH1PackageWithMaterialsV5(
          sourceGlb,
          appearanceTwoDa,
          request.identityJson,
          buildOptionsJson,
          request.packageLane === "H1_SKINNED_FULL_42_EVENTS"
            ? request.eventAuthoringJson
            : "",
          request.demoModuleIdentityJson,
          request.demoCreatureResref,
          buildMaterialInputs!.materialSeparationJson,
          buildMaterialInputs!.modelTextureAuthoringJson,
          modelTexturePayload,
          request.modelTexturePayloadDescriptorsJson!,
        )
        : buildMeshyFullNativeH1PackageWithOptionsV4(
          sourceGlb,
          appearanceTwoDa,
          request.identityJson,
          buildOptionsJson,
          request.packageLane === "H1_SKINNED_FULL_42_EVENTS"
            ? request.eventAuthoringJson
            : "",
          request.demoModuleIdentityJson,
          request.demoCreatureResref,
        )
      : materialInputsPresent
        ? buildMeshyProceduralHumanoidProductDemoWithMaterialsV2(
          sourceGlb,
          appearanceTwoDa,
          request.identityJson,
          buildOptionsJson,
          request.demoModuleIdentityJson,
          request.demoCreatureResref,
          buildMaterialInputs!.materialSeparationJson,
          buildMaterialInputs!.modelTextureAuthoringJson,
          modelTexturePayload,
          request.modelTexturePayloadDescriptorsJson!,
        )
        : buildMeshyProceduralHumanoidProductDemoWithOptionsV1(
          sourceGlb,
          appearanceTwoDa,
          request.identityJson,
          buildOptionsJson,
          request.demoModuleIdentityJson,
          request.demoCreatureResref,
        );
    try {
      const summary = JSON.parse(result.summaryJson) as {
        identity?: { modelResref?: unknown; textureResref?: unknown; hakResref?: unknown };
      };
      const requestedIdentity = JSON.parse(request.identityJson) as {
        modelResref?: unknown;
        textureResref?: unknown;
        hakResref?: unknown;
      };
      const fullNativeLane = request.packageLane === "H1_SKINNED_FULL_42"
        || request.packageLane === "H1_SKINNED_FULL_42_EVENTS";
      const modelResref = fullNativeLane
        ? requestedIdentity.modelResref
        : summary.identity?.modelResref;
      const textureResref = fullNativeLane
        ? requestedIdentity.textureResref
        : summary.identity?.textureResref;
      const hakResref = fullNativeLane
        ? requestedIdentity.hakResref
        : summary.identity?.hakResref;
      if (
        typeof modelResref !== "string"
        || typeof textureResref !== "string"
        || typeof hakResref !== "string"
      ) {
        throw new Error("Procedural product summary has no exact resource identity");
      }
      const hak = exactBuffer(result.takeHakBytes());
      const model = exactBuffer(result.takeModelBytes());
      const texturePayloadBlob = exactBuffer(result.takeTexturePayloadBlob());
      const textureDescriptors = JSON.parse(result.textureDescriptorsJson) as {
        resref: string;
        resourceType: number;
        byteOffset: number;
        byteLength: number;
        sha256: string;
      }[];
      const report = encoder.encode(result.reportJson).buffer;
      const manifest = encoder.encode(result.manifestJson).buffer;
      const summaryBytes = encoder.encode(result.summaryJson).buffer;
      const demoReport = JSON.parse(result.demoReportJson) as {
        moduleResref?: unknown;
        hakResref?: unknown;
        heldWeaponReadback?: unknown;
        heldStockWeaponReadback?: {
          weapon?: { resref?: unknown; resourceType?: unknown; resourceScope?: unknown };
          fixtures?: Array<{ hand?: unknown; equippedItemResref?: unknown }>;
        };
        authoredCreatureReadback?: {
          blueprint?: unknown;
          loadout?: unknown;
          loadoutReport?: {
            embeddedItemCount?: unknown;
            noEquippedResShortcuts?: unknown;
          };
        };
      };
      if (demoReport.moduleResref !== JSON.parse(request.demoModuleIdentityJson).moduleResref
        || demoReport.hakResref !== hakResref) {
        throw new Error("Procedural demo report has no exact module/HAK identity");
      }
      requireCreatureDemoAuthoringReadback(
        demoReport,
        request.demoAuthoring,
        request.heldWeapon,
      );
      const demoReportBytes = encoder.encode(result.demoReportJson).buffer;
      const textureArtifacts = textureDescriptors.map((descriptor, index) => {
        const start = descriptor.byteOffset;
        const end = start + descriptor.byteLength;
        if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end)
          || start < 0 || end < start || end > texturePayloadBlob.byteLength) {
          throw new Error(`MODEL-TEXTURE-DESCRIPTOR-RANGE-INVALID: textures[${index}]`);
        }
        const presentation = nativeModelResourcePresentation(descriptor.resourceType);
        return artifact(
          index === 0 && descriptor.resourceType === 3 ? "texture-tga" : `texture-${index}`,
          "TEXTURE",
          `${descriptor.resref}.${presentation.extension}`,
          presentation.mimeType,
          texturePayloadBlob.slice(start, end),
        );
      });
      const artifacts = await Promise.all([
        artifact("package-hak", "HAK", `${hakResref}.hak`, "application/octet-stream", hak),
        artifact("model-mdl", "MODEL", `${modelResref}.mdl`, "application/octet-stream", model),
        ...textureArtifacts,
        artifact(
          "proof-module",
          "MODULE",
          `${demoReport.moduleResref}.mod`,
          "application/octet-stream",
          exactBuffer(result.takeProofModuleBytes()),
        ),
        artifact("report-json", "JSON_REPORT", "inspection.json", "application/json", report),
        artifact("manifest-json", "JSON_REPORT", "conversion-manifest.json", "application/json", manifest),
        artifact("summary-json", "JSON_REPORT", "summary.json", "application/json", summaryBytes),
        artifact(
          "demo-report-json",
          "JSON_REPORT",
          "demo-report.json",
          "application/json",
          demoReportBytes,
        ),
        ...(materialInputsPresent ? [
          artifact(
            "material-separation-json",
            "JSON_REPORT",
            "material-separation.json",
            "application/json",
            encoder.encode(request.materialSeparationJson!).buffer,
          ),
          artifact(
            "model-texture-authoring-json",
            "JSON_REPORT",
            "model-texture-authoring.json",
            "application/json",
            encoder.encode(request.modelTextureAuthoringJson!).buffer,
          ),
        ] : []),
      ]);
      return {
        requestId: request.requestId,
        ok: true,
        type: "MODEL_PACKAGE_BUILT",
        artifacts,
        reportJson: result.reportJson,
        manifestJson: result.manifestJson,
        summaryJson: result.summaryJson,
        readbackJson: result.readbackJson,
        demoReportJson: result.demoReportJson,
      };
    } finally {
      result.free();
    }
  }

  if (
    request.type === "BUILD_MODEL_PACKAGE"
    && (
      request.packageLane === "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT"
      || request.packageLane === "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT"
    )
  ) {
    rejectCreatureExperimentMaterialInputs(request);
    const experimentLabel = request.packageLane === "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT"
      ? "P300K"
      : "P100K";
    const identity = JSON.parse(request.identityJson) as {
      modelResref?: unknown;
      textureResref?: unknown;
      module?: {
        moduleResref?: unknown;
        areaResref?: unknown;
        hakResref?: unknown;
      };
      creatureResref?: unknown;
    };
    const modelResref = identity.modelResref;
    const textureResref = identity.textureResref;
    const moduleResref = identity.module?.moduleResref;
    const hakResref = identity.module?.hakResref;
    if (
      typeof modelResref !== "string"
      || typeof textureResref !== "string"
      || typeof moduleResref !== "string"
      || typeof hakResref !== "string"
    ) {
      throw new Error(`${experimentLabel} package request has no exact resource identity`);
    }
    const result = request.packageLane === "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT"
      ? buildMeshyProceduralHumanoidP300kExperimentWithOptionsV2(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.appearanceTwoDa),
          request.identityJson,
          proceduralBuildOptionsJson(
            request.textureArtifactCleanup,
            request.sourceForward,
            request.skinAccessoryStabilization,
            request.weaponGrip,
            request.heldWeapon,
            request.demoAuthoring,
            request.runtimeEnvelope,
            request.motionPack,
            request.materialProfile,
            request.performancePreset,
          ),
        )
      : buildMeshyProceduralHumanoidP100kExperimentWithOptionsV2(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.appearanceTwoDa),
          request.identityJson,
          proceduralBuildOptionsJson(
            request.textureArtifactCleanup,
            request.sourceForward,
            request.skinAccessoryStabilization,
            request.weaponGrip,
            request.heldWeapon,
            request.demoAuthoring,
            request.runtimeEnvelope,
            request.motionPack,
            request.materialProfile,
            request.performancePreset,
          ),
        );
    try {
      const report = JSON.parse(result.reportJson) as {
        proofModule?: {
          heldWeaponReadback?: unknown;
          heldStockWeaponReadback?: {
            weapon?: { resref?: unknown; resourceType?: unknown; resourceScope?: unknown };
            fixtures?: Array<{ hand?: unknown; equippedItemResref?: unknown }>;
          };
        };
      };
      if (!report.proofModule) {
        throw new Error(`${experimentLabel} package report has no proof-module readback`);
      }
      requireCreatureDemoAuthoringReadback(
        report.proofModule,
        request.demoAuthoring,
        request.heldWeapon,
      );
      const artifacts = await Promise.all([
        artifact(
          "package-hak",
          "HAK",
          `${hakResref}.hak`,
          "application/octet-stream",
          exactBuffer(result.takeHakBytes()),
        ),
        artifact(
          "model-mdl",
          "MODEL",
          `${modelResref}.mdl`,
          "application/octet-stream",
          exactBuffer(result.takeModelBytes()),
        ),
        artifact(
          "texture-tga",
          "TEXTURE",
          `${textureResref}.tga`,
          "image/x-tga",
          exactBuffer(result.takeTextureBytes()),
        ),
        artifact(
          "proof-module",
          "MODULE",
          `${moduleResref}.mod`,
          "application/octet-stream",
          exactBuffer(result.takeProofModuleBytes()),
        ),
        artifact(
          "report-json",
          "JSON_REPORT",
          "inspection.json",
          "application/json",
          encoder.encode(result.reportJson).buffer,
        ),
        artifact(
          "manifest-json",
          "JSON_REPORT",
          "conversion-manifest.json",
          "application/json",
          encoder.encode(result.manifestJson).buffer,
        ),
        artifact(
          "summary-json",
          "JSON_REPORT",
          "summary.json",
          "application/json",
          encoder.encode(result.summaryJson).buffer,
        ),
      ]);
      return {
        requestId: request.requestId,
        ok: true,
        type: "MODEL_PACKAGE_BUILT",
        artifacts,
        reportJson: result.reportJson,
        manifestJson: result.manifestJson,
        summaryJson: result.summaryJson,
        readbackJson: result.readbackJson,
      };
    } finally {
      result.free();
    }
  }

  const result = (() => {
    switch (request.packageLane) {
      case "M0_STATIC_RIGID":
        return buildMeshyM0StaticRigidPackageV1(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.appearanceTwoDa),
        );
      default: {
        const unsupported: never = request;
        const lane = (unsupported as { packageLane?: unknown }).packageLane;
        throw new Error(`Unsupported model package lane: ${String(lane)}`);
      }
    }
  })();
  const artifactNames = request.packageLane === "M0_STATIC_RIGID"
    ? { hak: "m2a_m0_proof.hak", model: "m2a_m0p01.mdl", module: "m2a_bm0p1.mod" }
    : { hak: "m2a_codex_aproof.hak", model: "m2a_m6p01.mdl", module: "m2a_codex_aproof.mod" };
  try {
    const hak = exactBuffer(result.takeHakBytes());
    const model = exactBuffer(result.takeModelBytes());
    const proofModule = exactBuffer(result.takeProofModuleBytes());
    const report = encoder.encode(result.reportJson).buffer;
    const manifest = encoder.encode(result.manifestJson).buffer;
    const summary = encoder.encode(result.summaryJson).buffer;
    const artifacts = await Promise.all([
      artifact("package-hak", "HAK", artifactNames.hak, "application/octet-stream", hak),
      artifact("model-mdl", "MODEL", artifactNames.model, "application/octet-stream", model),
      artifact("proof-module", "MODULE", artifactNames.module, "application/octet-stream", proofModule),
      artifact("report-json", "JSON_REPORT", "inspection.json", "application/json", report),
      artifact("manifest-json", "JSON_REPORT", "conversion-manifest.json", "application/json", manifest),
      artifact("summary-json", "JSON_REPORT", "summary.json", "application/json", summary),
    ]);
    return {
      requestId: request.requestId,
      ok: true,
      type: "MODEL_PACKAGE_BUILT",
      artifacts,
      reportJson: result.reportJson,
      manifestJson: result.manifestJson,
      summaryJson: result.summaryJson,
      readbackJson: result.readbackJson,
    };
  } finally {
    result.free();
  }
}

self.addEventListener("message", (event: MessageEvent<StudioWorkerRequest>) => {
  void handle(event.data)
    .then((response) => {
      const transfer = response.ok && "artifacts" in response
        ? response.artifacts.map((item) => item.bytes)
        : [];
      self.postMessage(response, { transfer });
    })
    .catch((error: unknown) => {
      const response: StudioWorkerResponse = {
        requestId: event.data.requestId,
        ok: false,
        type: "FAILED",
        message: error instanceof Error ? error.message : String(error),
      };
      self.postMessage(response);
    });
});
