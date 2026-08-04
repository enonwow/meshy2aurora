/// <reference lib="webworker" />

import init, {
  buildItemEquippedProofModuleV2,
  buildM7CorpusBatchV1,
  buildItemProofModuleV1,
  buildMeshyItemPartWithOptionsV2,
  buildMeshyItemPartWithOptionsV3,
  buildMeshyH1ModelPackageV2,
  buildMeshyH1ModelPackageV3,
  buildMeshyProceduralHumanoidProductWithOptionsV3,
  buildMeshyProceduralHumanoidP100kExperimentWithOptionsV2,
  buildMeshyProceduralHumanoidP300kExperimentWithOptionsV2,
  buildMeshyM0StaticRigidPackageV1,
  buildMeshyStaticPlaceablePackageV1,
  buildMeshyStaticPlaceablePackageV2,
  buildMeshyStaticTilePackageV1,
  buildItemAttachmentProfileV1Json,
  fitMeshyItemPartsV3Json,
  fitMeshyItemPartsV4Json,
  ingestGlbJson,
  ingestMeshyP100kExperimentJson,
  ingestMeshyP300kExperimentJson,
  ingestStaticRigidGlbJson,
  inspectItemBaseitemsV1Json,
  extendItemBaseitemModelRangeV1,
  extendItemBaseitemModelRangeV1ReportJson,
  inspectItemReferenceResourceV1Json,
  inspectItemReferenceTwoDaV1Json,
  inspectTwoDaV2Json,
  inspectM7CorpusIntakeV1Json,
  inspectMeshyStaticPlaceableAuthoringV1,
  measureMeshyItemSeamV1Json,
  resolveItemCastSpellIconV1Json,
  resolveItemCapartPartV1Json,
  resolveItemCapartPartV2Json,
  resolveItemCloakV4Json,
  resolveItemEquippedAppearanceV1Json,
  resolveItemPartResourceV1Json,
  validateM7CorpusManifestV1Json,
  validateItemFitReportV3Json,
  validateItemFitReportV4Json,
  validateItemModelType2ComposerV3Json,
  validateItemModelType2ComposerV4Json,
  validateItemModelType2IconLayersV3Json,
  validateItemTriangleBudgetV1Json,
  writeHakV1,
  writeItemUtiV1,
} from "@m2a-wasm";
import type {
  StudioWorkerRequest,
  StudioWorkerResponse,
  WorkerArtifact,
} from "./types";

function proceduralBuildOptionsJson(
  textureArtifactCleanup: boolean,
  skinAccessoryStabilization?: {
    mode: "AUTO" | "KEEP_SOURCE_WEIGHTS" | "SELECT_BONE";
    selectedBoneName?: string;
  },
): string {
  const stabilization = skinAccessoryStabilization ?? { mode: "AUTO" as const };
  return JSON.stringify({
    schemaVersion: 1,
    textureArtifactCleanup,
    skinAccessoryStabilization: {
      schemaVersion: 1,
      mode: stabilization.mode,
      ...(stabilization.mode === "SELECT_BONE"
        ? { selectedBoneName: stabilization.selectedBoneName?.trim() }
        : {}),
    },
  });
}

let initialized: Promise<unknown> | undefined;
const ensureInitialized = () => (initialized ??= init());
const encoder = new TextEncoder();
const twoDaInspectionLimitsJson = JSON.stringify({
  maxInputBytes: 16_777_216,
  maxColumns: 4_096,
  maxRows: 65_536,
  maxTokenBytes: 1_048_576,
  maxDiagnostics: 2_048,
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

function matchesItemReferenceResourceType(value: unknown): value is 6 | 2002 {
  return value === 6 || value === 2002;
}

async function handle(request: StudioWorkerRequest): Promise<StudioWorkerResponse> {
  await ensureInitialized();
  if (request.type === "INITIALIZE") {
    return { requestId: request.requestId, ok: true, type: "INITIALIZED" };
  }
  if (request.type === "INSPECT_SOURCE") {
    const ingestJson = request.target === "PLACEABLE" || request.target === "ITEM" || request.target === "TILE"
      ? ingestStaticRigidGlbJson(new Uint8Array(request.sourceGlb))
      : request.creatureProfile === "EXPERIMENTAL_P300K"
        ? ingestMeshyP300kExperimentJson(new Uint8Array(request.sourceGlb))
      : request.creatureProfile === "EXPERIMENTAL_P100K"
        ? ingestMeshyP100kExperimentJson(new Uint8Array(request.sourceGlb))
      : ingestGlbJson(new Uint8Array(request.sourceGlb));
    return {
      requestId: request.requestId,
      ok: true,
      type: "SOURCE_INSPECTED",
      ingestJson,
      placeableAuthoringJson: request.target === "PLACEABLE"
        ? inspectMeshyStaticPlaceableAuthoringV1(new Uint8Array(request.sourceGlb))
        : undefined,
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
  if (request.type === "INSPECT_ITEM_BASEITEMS") {
    return {
      requestId: request.requestId,
      ok: true,
      type: "ITEM_BASEITEMS_INSPECTED",
      catalogJson: inspectItemBaseitemsV1Json(new Uint8Array(request.baseitemsTwoDa)),
    };
  }

  if (request.type === "BUILD_ITEM_ATTACHMENT_PROFILE") {
    const baseitemsHash = await sha256(request.baseitemsTwoDa);
    const modelIdentity = [];
    const byteLength = request.models.reduce((total, model) => total + model.bytes.byteLength, 0);
    const mdlBundle = new Uint8Array(byteLength);
    let byteOffset = 0;
    const models = [];
    for (const model of request.models) {
      const bytes = new Uint8Array(model.bytes);
      mdlBundle.set(bytes, byteOffset);
      models.push({
        field: model.field,
        modelResref: model.modelResref,
        byteOffset,
        byteLength: bytes.byteLength,
      });
      modelIdentity.push({
        field: model.field,
        modelResref: model.modelResref,
        sha256: await sha256(model.bytes),
        byteLength: model.bytes.byteLength,
      });
      byteOffset += bytes.byteLength;
    }
    const resourceContextSha256 = await sha256(encoder.encode(JSON.stringify({
      schemaVersion: 1,
      policy: "DIRECT_READONLY_REFERENCE_FILES_V1",
      baseitemsSha256: baseitemsHash,
      models: modelIdentity,
    })).buffer);
    return {
      requestId: request.requestId,
      ok: true,
      type: "ITEM_ATTACHMENT_PROFILE_BUILT",
      attachmentProfileJson: buildItemAttachmentProfileV1Json(
        new Uint8Array(request.baseitemsTwoDa),
        request.baseItem,
        mdlBundle,
        JSON.stringify({
          schemaVersion: 1,
          resourceContextSha256,
          referenceKind: request.referenceKind,
          referenceId: request.referenceId,
          models,
        }),
      ),
    };
  }

  if (request.type === "FIT_ITEM_PARTS") {
    const totalBytes = request.parts.reduce((total, part) => total + part.sourceGlb.byteLength, 0);
    const sourceBundle = new Uint8Array(totalBytes);
    let byteOffset = 0;
    const descriptors = request.parts.map((part) => {
      const bytes = new Uint8Array(part.sourceGlb);
      sourceBundle.set(bytes, byteOffset);
      const descriptor = {
        field: part.field,
        modelResref: part.modelResref,
        sourceNode: part.sourceNode,
        byteOffset,
        byteLength: bytes.byteLength,
      };
      byteOffset += bytes.byteLength;
      return descriptor;
    });
    return {
      requestId: request.requestId,
      ok: true,
      type: "ITEM_PARTS_FITTED",
          fitReportJson: request.attachmentProfileJson
        ? fitMeshyItemPartsV4Json(sourceBundle, JSON.stringify({
            schemaVersion: 4,
            tolerance: request.tolerance,
            targetAxialLengths: null,
            targetAxialScaleFactors: request.targetAxialScaleFactors ?? null,
            parts: descriptors,
          }), request.attachmentProfileJson)
        : fitMeshyItemPartsV3Json(sourceBundle, JSON.stringify({
            schemaVersion: 3,
            tolerance: request.tolerance,
            targetAxialLengths: request.targetAxialLengths ?? null,
            parts: descriptors,
          })),
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
    const result = request.authoringJson
      ? buildMeshyStaticPlaceablePackageV2(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.placeablesTwoDa),
          request.identityJson,
          request.placementJson,
          request.paletteId,
          request.authoringJson,
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
      const hak = exactBuffer(result.takeHakBytes());
      const model = exactBuffer(result.takeModelBytes());
      const module = exactBuffer(result.takeProofModuleBytes());
      const reportBytes = encoder.encode(result.reportJson).buffer;
      const artifacts = await Promise.all([
        artifact("placeable-package-hak", "HAK", report.hakFileName, "application/octet-stream", hak),
        artifact("placeable-model-mdl", "MODEL", `${report.modelResref}.mdl`, "application/octet-stream", model),
        artifact("placeable-proof-module", "MODULE", report.moduleFileName, "application/octet-stream", module),
        artifact(
          "placeable-report-json",
          "JSON_REPORT",
          "placeable-materialization-report.json",
          "application/json",
          reportBytes,
        ),
      ]);
      return {
        requestId: request.requestId,
        ok: true,
        type: "PLACEABLE_PACKAGE_BUILT",
        artifacts,
        reportJson: result.reportJson,
        readbackJson: result.readbackJson,
      };
    } finally {
      result.free();
    }
  }
  if (request.type === "BUILD_ITEM_PACKAGE") {
    const attachmentProfileJson = request.attachmentProfileJson ?? null;
    type Catalog = {
      sourceSha256: string;
      rows: Array<{
        baseItem: number;
        itemClass: string;
        modelType: number;
        minRange: number | null;
        maxRange: number | null;
        equipableSlots: number;
        invSlotWidth: number;
        invSlotHeight: number;
        capability: {
          iconProfile: "STANDARD" | "LAYERED" | "IPRP_SPELL" | "CAPART_COMPOSITE" | "CLOAK_MODEL";
          requiredReferenceTables: string[];
        };
        partSlots: Array<{
          field: string;
          sourceKind: "MESHY_GLB" | "CAPART_SELECTION" | "CLOAK_MODEL_SELECTION";
          referenceTable: string | null;
        }>;
      }>;
    };
    type IconProjectionBounds = {
      min: [number, number];
      max: [number, number];
    };
    type PartOutput = {
      field: string;
      variant: number;
      weaponColor: number | null;
      modelResref: string;
      iconResref: string;
      textureResref: string;
      mdl: ArrayBuffer;
      texture: ArrayBuffer;
      icon: ArrayBuffer;
      report: Record<string, unknown> & {
        triangleCount: number;
        degenerateTriangleCountRemoved: number;
        textureFormat: string;
        mdlSha256: string;
        textureSha256: string;
        iconSha256?: string;
        iconProjectionBounds?: IconProjectionBounds;
        iconOpaquePixelCount?: number;
      };
      readback: {
        model: {
          boundsMin: { x: number; y: number; z: number };
          boundsMax: { x: number; y: number; z: number };
        };
      };
    };
    const catalog = JSON.parse(
      inspectItemBaseitemsV1Json(new Uint8Array(request.baseitemsTwoDa)),
    ) as Catalog;
    const selected = catalog.rows.find((row) => row.baseItem === request.baseItem);
    if (!selected) throw new Error(`ITEM-BASEITEM-NOT-FOUND: ${request.baseItem}`);
    if (
      selected.modelType === 2
      && request.parts.some((part) => part.textureEncoding !== "DIRECT_COLOR")
    ) {
      throw new Error(
        "ITEM-MODELTYPE2-PLT-UNSUPPORTED: weapon colors select concrete MDL/texture variants and do not serialize standalone PLT color fields",
      );
    }
    const expectedFields = selected.partSlots.map((slot) => slot.field);
    if (
      request.parts.length !== expectedFields.length
      || expectedFields.some((field) => request.parts.filter(
        (part) => part.field.toLowerCase() === field.toLowerCase(),
      ).length !== 1)
    ) {
      throw new Error("ITEM-PARTS-INCOMPLETE: build request must cover every resolved slot exactly once");
    }
    for (const slot of selected.partSlots) {
      const part = request.parts.find((candidate) => (
        candidate.field.toLowerCase() === slot.field.toLowerCase()
      ));
      if (!part || part.sourceKind !== slot.sourceKind) {
        throw new Error(`ITEM-PART-SOURCE-KIND-MISMATCH: ${slot.field} must use ${slot.sourceKind}`);
      }
      if (slot.sourceKind === "MESHY_GLB" && !part.sourceGlb) {
        throw new Error(`ITEM-PART-SOURCE-MISSING: ${slot.field} requires one Meshy GLB`);
      }
      if (slot.sourceKind !== "MESHY_GLB" && part.sourceGlb) {
        throw new Error(`ITEM-PART-SOURCE-UNEXPECTED: ${slot.field} is a numeric reference selector`);
      }
      if (selected.modelType === 2) {
        const weaponColorways = part.weaponColorways ?? [];
        const colors = weaponColorways.map((colorway) => colorway.color);
        if (
          part.sourceKind !== "MESHY_GLB"
          || weaponColorways.length !== 4
          || new Set(colors).size !== 4
          || ![1, 2, 3, 4].every((color) => colors.includes(color as 1 | 2 | 3 | 4))
          || !weaponColorways.some((colorway) => (
            colorway.color === part.variant % 10
            && colorway.variant === part.variant
            && colorway.modelResref === part.modelResref
            && colorway.iconResref === part.iconResref
            && colorway.textureResref === part.textureResref
          ))
        ) {
          throw new Error(
            `ITEM-WEAPON-COLORWAY-COVERAGE-INCOMPLETE: ${slot.field} must bind colors 1..4 and its selected UTI value`,
          );
        }
      } else if ((part.weaponColorways?.length ?? 0) !== 0) {
        throw new Error(
          `ITEM-WEAPON-COLORWAY-UNEXPECTED: ${slot.field} is not a ModelType 2 weapon part`,
        );
      }
    }
    const orderedParts = selected.partSlots.map((slot) => request.parts.find(
      (part) => part.field.toLowerCase() === slot.field.toLowerCase(),
    )!);
    let effectiveBaseitemsTwoDa = request.baseitemsTwoDa;
    type BaseitemsModelRangeReport = {
      status: "PATCHED" | "NOT_REQUIRED";
      baseItem: number;
      model: number;
      modelBucket: number;
      sourceMinRange: number;
      sourceMaxRange: number;
      effectiveMaxRange: number;
      sourceSha256: string;
      outputSha256: string;
    };
    let baseitemsModelRange: BaseitemsModelRangeReport | null = null;
    if (selected.modelType === 2) {
      const selectedModels = orderedParts.map((part) => Math.floor(part.variant / 10));
      const highestModel = Math.max(...selectedModels);
      const rangeReport = JSON.parse(extendItemBaseitemModelRangeV1ReportJson(
        new Uint8Array(request.baseitemsTwoDa),
        request.baseItem,
        highestModel,
      )) as BaseitemsModelRangeReport;
      baseitemsModelRange = rangeReport;
      effectiveBaseitemsTwoDa = exactBuffer(extendItemBaseitemModelRangeV1(
        new Uint8Array(request.baseitemsTwoDa),
        request.baseItem,
        highestModel,
      ));
      if (
        rangeReport.baseItem !== request.baseItem
        || rangeReport.model !== highestModel
        || !["PATCHED", "NOT_REQUIRED"].includes(rangeReport.status)
      ) {
        throw new Error("ITEM-BASEITEM-RANGE-REPORT-MISMATCH: override is not bound to the selected BaseItem/model");
      }
    }
    if (
      !Number.isFinite(request.seamValidation.tolerance)
      || request.seamValidation.tolerance < 0
    ) {
      throw new Error(
        "ITEM-SEAM-TOLERANCE-INVALID: tolerance must be finite and non-negative",
      );
    }
    if (request.occupiedResourceKeys.some((key) => (
      typeof key !== "string"
      || key.trim().length === 0
      || !/^(?:[0-9]+|HAK|MOD):[a-z0-9_]{1,16}$/i.test(key.trim())
    ))) {
      throw new Error(
        "ITEM-OCCUPIED-RESOURCE-KEY-INVALID: occupied entries must use numeric-type:resref, HAK:resref or MOD:resref",
      );
    }
    const occupiedResourceKeys = new Set(
      request.occupiedResourceKeys.map((key) => key.trim().toLowerCase()),
    );
    for (const [kind, resref] of [
      ["hak", request.hakResref],
      ["mod", request.moduleResref],
    ] as const) {
      const key = `${kind}:${resref.toLowerCase()}`;
      if (occupiedResourceKeys.has(key)) {
        throw new Error(`ITEM-RESOURCE-COLLISION: occupied output resource ${key}`);
      }
    }
    const referenceTables = new Map<string, ArrayBuffer>();
    for (const table of request.referenceTables) {
      const normalized = table.tableName.trim().toUpperCase();
      const expectedFileName = `${normalized.toLowerCase()}.2da`;
      if (
        table.fileName.toLowerCase() !== expectedFileName
        || referenceTables.has(normalized)
      ) {
        throw new Error(
          `ITEM-REFERENCE-TABLE-IDENTITY-INVALID: ${normalized} must come from one exact ${expectedFileName} file`,
        );
      }
      referenceTables.set(normalized, table.bytes);
    }
    for (const tableName of selected.capability.requiredReferenceTables) {
      const normalized = tableName.toUpperCase();
      if (!referenceTables.has(normalized)) {
        throw new Error(`ITEM-REFERENCE-TABLE-MISSING: ${tableName}.2da is required by this composer`);
      }
    }
    const referenceTableReports = request.referenceTables.map((table) => ({
      fileName: table.fileName,
      ...JSON.parse(inspectItemReferenceTwoDaV1Json(
        table.tableName,
        new Uint8Array(table.bytes),
      )),
    }));
    const requiresRetailResources = ["CAPART_COMPOSITE", "CLOAK_MODEL"].includes(
      selected.capability.iconProfile,
    );
    if (requiresRetailResources && !request.referenceResourceManifest) {
      throw new Error(
        "ITEM-RESOURCE-MANIFEST-MISSING: CAPART and Cloak require one exact pinned retail resource manifest",
      );
    }
    if (!requiresRetailResources && (
      request.referenceResourceManifest
      || request.referenceResources.length > 0
    )) {
      throw new Error(
        "ITEM-RESOURCE-MANIFEST-UNEXPECTED: retail resources are valid only for CAPART and Cloak",
      );
    }
    type RetailManifest = {
      schemaVersion?: number;
      sourceKind?: string;
      sourceContainerFileName?: string;
      sourceContainerSha256?: string;
      resources?: Array<{
        resourceType?: number;
        resref?: string;
        fileName?: string;
        sha256?: string;
        locator?: string;
      }>;
    };
    let retailManifest: RetailManifest | null = null;
    let retailManifestSha256: string | null = null;
    if (request.referenceResourceManifest) {
      const manifest = request.referenceResourceManifest;
      if (
        !manifest.fileName.toLowerCase().endsWith(".json")
        || manifest.bytes.byteLength === 0
      ) {
        throw new Error(
          "ITEM-RESOURCE-MANIFEST-FILE-INVALID: retail manifest must be one non-empty JSON file",
        );
      }
      try {
        retailManifest = JSON.parse(
          new TextDecoder("utf-8", { fatal: true }).decode(manifest.bytes),
        ) as RetailManifest;
      } catch {
        throw new Error(
          "ITEM-RESOURCE-MANIFEST-JSON-INVALID: retail resource manifest is not valid UTF-8 JSON",
        );
      }
      retailManifestSha256 = await sha256(manifest.bytes);
      if (
        retailManifest.schemaVersion !== 1
        || retailManifest.sourceKind !== "NWN_BASE_KEY"
        || retailManifest.sourceContainerFileName?.toLowerCase() !== "nwn_base.key"
        || retailManifest.sourceContainerSha256
          !== "09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935"
        || !Array.isArray(retailManifest.resources)
        || retailManifest.resources.length !== request.referenceResources.length
      ) {
        throw new Error(
          "ITEM-RESOURCE-MANIFEST-IDENTITY-INVALID: manifest must bind every supplied resource to the pinned retail nwn_base.key",
        );
      }
    }
    const manifestEntries = new Map<string, NonNullable<RetailManifest["resources"]>[number]>();
    for (const [index, entry] of (retailManifest?.resources ?? []).entries()) {
      const normalizedResref = entry.resref?.trim().toLowerCase() ?? "";
      const key = `${entry.resourceType}:${normalizedResref}`;
      if (
        !matchesItemReferenceResourceType(entry.resourceType)
        || !/^[a-z0-9_]{1,16}$/.test(normalizedResref)
        || typeof entry.fileName !== "string"
        || typeof entry.sha256 !== "string"
        || !/^[0-9a-f]{64}$/.test(entry.sha256)
        || typeof entry.locator !== "string"
        || !/^bif:\d+:resource:\d+$/.test(entry.locator)
        || manifestEntries.has(key)
      ) {
        throw new Error(
          `ITEM-RESOURCE-MANIFEST-ENTRY-INVALID: manifest resource ${index} has invalid identity, hash or locator`,
        );
      }
      manifestEntries.set(key, entry);
    }
    const inventoryKeys = new Set<string>();
    const resourceInventory = request.referenceResources.map((resource, index) => {
      const expectedExtension = resource.resourceType === 2002 ? ".mdl" : ".plt";
      const normalizedResref = resource.resref.trim().toLowerCase();
      const key = `${resource.resourceType}:${normalizedResref}`;
      const manifestEntry = manifestEntries.get(key);
      if (
        resource.bytes.byteLength === 0
        || resource.fileName.toLowerCase() !== `${normalizedResref}${expectedExtension}`
        || !/^[a-z0-9_]{1,16}$/.test(normalizedResref)
        || inventoryKeys.has(key)
        || manifestEntry?.fileName?.toLowerCase() !== resource.fileName.toLowerCase()
      ) {
        throw new Error(
          `ITEM-RESOURCE-INVENTORY-FILE-INVALID: entry ${index} must be one unique manifest-declared exact resref${expectedExtension} payload`,
        );
      }
      inventoryKeys.add(key);
      return JSON.parse(inspectItemReferenceResourceV1Json(
        resource.resourceType,
        normalizedResref,
        new Uint8Array(resource.bytes),
        JSON.stringify({
          schemaVersion: 1,
          sourceKind: retailManifest!.sourceKind,
          sourceContainerFileName: retailManifest!.sourceContainerFileName,
          sourceContainerSha256: retailManifest!.sourceContainerSha256,
          resourceLocator: manifestEntry.locator,
          expectedSha256: manifestEntry.sha256,
          manifestSha256: retailManifestSha256,
        }),
      ));
    });
    const blueprint = JSON.parse(request.blueprintJson) as {
      templateResref?: string;
      parts?: Array<{ field?: string; value?: number }>;
    };
    if (
      blueprint.templateResref !== request.blueprintResref
      || !Array.isArray(blueprint.parts)
      || blueprint.parts.length !== request.parts.length
      || request.parts.some((part) => blueprint.parts?.filter(
        (value) => value.field?.toLowerCase() === part.field.toLowerCase()
          && value.value === part.variant,
      ).length !== 1)
    ) {
      throw new Error(
        "ITEM-BLUEPRINT-RESOURCE-MISMATCH: UTI identity and numeric part values must match the packaged resources",
      );
    }
    for (const part of orderedParts.filter((candidate) => candidate.sourceKind === "MESHY_GLB")) {
      const appearances = selected.modelType === 2
        ? part.weaponColorways ?? []
        : [{
            variant: part.variant,
            modelResref: part.modelResref,
            iconResref: part.iconResref,
          }];
      for (const appearance of appearances) {
        const resolved = JSON.parse(resolveItemPartResourceV1Json(
          new Uint8Array(effectiveBaseitemsTwoDa),
          request.baseItem,
          part.field,
          appearance.variant,
          appearance.modelResref,
          appearance.iconResref,
        )) as { modelResref: string; iconResref: string };
        if (
          resolved.modelResref !== appearance.modelResref
          || resolved.iconResref !== appearance.iconResref
        ) {
          throw new Error(
            `ITEM-RESOURCE-NAME-MISMATCH: ${part.field} must resolve to ${resolved.modelResref} / ${resolved.iconResref}`,
          );
        }
      }
    }

    const outputs: PartOutput[] = [];
    const emitsCustomIcons = selected.capability.iconProfile === "STANDARD"
      || selected.capability.iconProfile === "LAYERED";
    const meshyParts = orderedParts.filter(
      (candidate) => candidate.sourceKind === "MESHY_GLB",
    );
    const buildParts = selected.modelType === 2
      ? meshyParts.flatMap((part) => (part.weaponColorways ?? []).map((colorway) => ({
          ...part,
          ...colorway,
          weaponColor: colorway.color as number | null,
        })))
      : meshyParts.map((part) => ({ ...part, weaponColor: null as number | null }));
    const sourceHashes = new Map<string, string>();
    for (const part of meshyParts) {
      sourceHashes.set(part.field, await sha256(part.sourceGlb!));
    }
    const exactKeys = (value: unknown, keys: readonly string[], path: string) => {
      if (!value || typeof value !== "object" || Array.isArray(value)) {
        throw new Error(`ITEM-GENERATION-PROVENANCE-INVALID: ${path} must be an object`);
      }
      const record = value as Record<string, unknown>;
      const actual = Object.keys(record);
      const unknown = actual.find((key) => !keys.includes(key));
      if (unknown || keys.some((key) => !(key in record))) {
        throw new Error(`ITEM-GENERATION-PROVENANCE-INVALID: ${path} has an unknown or missing field`);
      }
      return record;
    };
    type GenerationArtifactEvidence = {
      field: string;
      profileId: string;
      bridgeProtocolVersion: number;
      sha256: string;
      byteLength: number;
      taskIds: { PREVIEW: string };
      consumedCredits: number;
      createdAt: string;
      finishedAt: string;
    };
    const generationArtifacts = new Map<string, GenerationArtifactEvidence>();
    if ((request.generationSessionJson === null) !== (request.generationArtifactsJson === null)) {
      throw new Error("ITEM-GENERATION-PROVENANCE-INCOMPLETE: session and exact Bridge artifact evidence must be supplied together");
    }
    if (request.generationArtifactsJson !== null) {
      if (/https?:\/\/|authorization|api[_-]?key|signedUrl/i.test(request.generationArtifactsJson)) {
        throw new Error("ITEM-GENERATION-PROVENANCE-SECRET: Bridge artifact evidence contains a URL or credential-like field");
      }
      const artifactRoot = exactKeys(
        JSON.parse(request.generationArtifactsJson),
        ["schemaVersion", "artifacts"],
        "generationArtifacts",
      ) as Record<string, unknown> & { artifacts: unknown[] };
      if (
        artifactRoot.schemaVersion !== 1
        || !Array.isArray(artifactRoot.artifacts)
        || artifactRoot.artifacts.length !== meshyParts.length
      ) {
        throw new Error("ITEM-GENERATION-PROVENANCE-ARTIFACTS-INVALID: exact Bridge artifact evidence must cover every Meshy slot");
      }
      for (const [index, value] of artifactRoot.artifacts.entries()) {
        const evidence = exactKeys(value, [
          "field", "profileId", "bridgeProtocolVersion", "sha256", "byteLength",
          "taskIds", "consumedCredits", "createdAt", "finishedAt",
        ], `generationArtifacts.artifacts[${index}]`) as unknown as GenerationArtifactEvidence;
        const taskIds = exactKeys(
          evidence.taskIds,
          ["PREVIEW"],
          `generationArtifacts.artifacts[${index}].taskIds`,
        );
        if (
          typeof evidence.field !== "string"
          || evidence.field.trim().length === 0
          || generationArtifacts.has(evidence.field)
          || !["S1-static-prop/v1", "RECOVERED-image-to-3d/v1"].includes(evidence.profileId)
          || evidence.bridgeProtocolVersion !== 1
          || !/^[a-f0-9]{64}$/.test(evidence.sha256)
          || !Number.isSafeInteger(evidence.byteLength)
          || evidence.byteLength <= 0
          || typeof taskIds.PREVIEW !== "string"
          || taskIds.PREVIEW.trim().length === 0
          || !Number.isSafeInteger(evidence.consumedCredits)
          || evidence.consumedCredits < 0
          || typeof evidence.createdAt !== "string"
          || evidence.createdAt.trim().length === 0
          || typeof evidence.finishedAt !== "string"
          || evidence.finishedAt.trim().length === 0
        ) {
          throw new Error(`ITEM-GENERATION-PROVENANCE-ARTIFACT-INVALID: evidence ${index} has invalid identity, task, cost or hash`);
        }
        generationArtifacts.set(evidence.field, {
          ...evidence,
          taskIds: { PREVIEW: taskIds.PREVIEW as string },
        });
      }
    }
    type GenerationSlot = {
      field: string;
      label: string;
      token: string | null;
      role: string;
      profileId: string;
      targetPolycount: number;
      status: string;
      concept: { sha256: string } | null;
      run: { runId: string; taskId: string | null; createdAt: string } | null;
      artifact: { sha256: string; byteLength: number; consumedCredits: number; finishedAt: string } | null;
    };
    let generationReport: null | {
      sessionId: string;
      createdAt: string;
      status: string;
      ownerCreditCap: number;
      balanceAtReview: number;
      maximumCredits: number;
      actualConsumedCredits: number;
      slots: Array<{
        field: string;
        label: string;
        token: string | null;
        role: string;
        profileId: string;
        targetPolycount: number;
        conceptSha256: string;
        runId: string;
        taskId: string;
        runCreatedAt: string;
        taskCreatedAt: string;
        glbSha256: string;
        byteLength: number;
        consumedCredits: number;
        finishedAt: string;
      }>;
    } = null;
    if (request.generationSessionJson !== null) {
      if (/https?:\/\/|authorization|api[_-]?key|signedUrl/i.test(request.generationSessionJson)) {
        throw new Error("ITEM-GENERATION-PROVENANCE-SECRET: generation snapshot contains a URL or credential-like field");
      }
      const root = exactKeys(JSON.parse(request.generationSessionJson), [
        "schemaVersion", "sessionId", "createdAt", "baseitemsSha256", "baseItem", "itemClass",
        "modelType", "status", "ownerCreditCap", "balanceAtReview", "maximumCredits", "slots",
      ], "generationSession") as Record<string, unknown> & { slots: unknown[] };
      if (
        root.schemaVersion !== 1
        || root.baseitemsSha256 !== catalog.sourceSha256
        || root.baseItem !== request.baseItem
        || root.itemClass !== selected.itemClass
        || root.modelType !== selected.modelType
        || root.status !== "ARTIFACTS_VERIFIED"
        || typeof root.sessionId !== "string"
        || root.sessionId.trim().length === 0
        || typeof root.createdAt !== "string"
        || root.createdAt.trim().length === 0
        || typeof root.ownerCreditCap !== "number"
        || !Number.isSafeInteger(root.ownerCreditCap)
        || root.ownerCreditCap < 0
        || typeof root.balanceAtReview !== "number"
        || !Number.isSafeInteger(root.balanceAtReview)
        || root.balanceAtReview < 0
        || typeof root.maximumCredits !== "number"
        || !Number.isSafeInteger(root.maximumCredits)
        || root.maximumCredits < 0
        || !Array.isArray(root.slots)
        || root.slots.length !== meshyParts.length
      ) {
        throw new Error("ITEM-GENERATION-PROVENANCE-IDENTITY-MISMATCH: session does not bind the exact BaseItem and completed Meshy slots");
      }
      const sessionSlots = root.slots.map((value, index) => {
        const slot = exactKeys(value, [
          "field", "label", "token", "role", "profileId", "targetPolycount", "status",
          "concept", "preview", "run", "artifact",
        ], `generationSession.slots[${index}]`) as unknown as GenerationSlot & Record<string, unknown>;
        if (slot.concept) exactKeys(slot.concept, ["fileName", "mimeType", "byteLength", "sha256"], `generationSession.slots[${index}].concept`);
        if (slot.run) exactKeys(slot.run, ["runId", "taskId", "createdAt"], `generationSession.slots[${index}].run`);
        if (slot.artifact) exactKeys(slot.artifact, ["sha256", "byteLength", "consumedCredits", "finishedAt"], `generationSession.slots[${index}].artifact`);
        return slot;
      });
      const normalizedSlots = meshyParts.map((part, index) => {
        const slot = sessionSlots.find((candidate) => candidate.field === part.field);
        const artifactEvidence = generationArtifacts.get(part.field);
        const expectedRole = meshyParts.length === 1 ? "MODEL" : ["BOTTOM", "MIDDLE", "TOP"][index];
        if (
          !slot
          || slot.role !== expectedRole
          || typeof slot.label !== "string"
          || slot.label.trim().length === 0
          || !(slot.token === null || typeof slot.token === "string")
          || slot.profileId !== "S1-static-prop/v1"
          || !Number.isSafeInteger(slot.targetPolycount)
          || slot.targetPolycount < 100
          || slot.targetPolycount > 300_000
          || slot.status !== "ARTIFACT_VERIFIED"
          || !slot.concept
          || !/^[a-f0-9]{64}$/.test(slot.concept.sha256)
          || !slot.run?.taskId
          || !slot.run.runId
          || !slot.run.createdAt
          || !slot.artifact
          || slot.artifact.sha256 !== sourceHashes.get(part.field)
          || slot.artifact.byteLength !== part.sourceGlb!.byteLength
          || !Number.isSafeInteger(slot.artifact.consumedCredits)
          || slot.artifact.consumedCredits < 0
          || !slot.artifact.finishedAt
        ) {
          throw new Error(`ITEM-GENERATION-PROVENANCE-SLOT-MISMATCH: ${part.field} is not bound to its exact concept, task and GLB`);
        }
        if (artifactEvidence?.taskIds.PREVIEW !== slot.run.taskId) {
          throw new Error(`ITEM-GENERATION-PROVENANCE-TASK-MISMATCH: ${part.field} session task differs from exact Bridge artifact evidence`);
        }
        if (
          artifactEvidence.sha256 !== slot.artifact.sha256
          || artifactEvidence.byteLength !== slot.artifact.byteLength
          || artifactEvidence.consumedCredits !== slot.artifact.consumedCredits
          || artifactEvidence.finishedAt !== slot.artifact.finishedAt
        ) {
          throw new Error(`ITEM-GENERATION-PROVENANCE-ARTIFACT-MISMATCH: ${part.field} session artifact differs from exact Bridge evidence`);
        }
        return {
          field: part.field,
          label: slot.label,
          token: slot.token,
          role: slot.role,
          profileId: slot.profileId,
          targetPolycount: slot.targetPolycount,
          conceptSha256: slot.concept.sha256,
          runId: slot.run.runId,
          taskId: slot.run.taskId,
          runCreatedAt: slot.run.createdAt,
          taskCreatedAt: artifactEvidence.createdAt,
          glbSha256: slot.artifact.sha256,
          byteLength: slot.artifact.byteLength,
          consumedCredits: slot.artifact.consumedCredits,
          finishedAt: slot.artifact.finishedAt,
        };
      });
      const actualConsumedCredits = normalizedSlots.reduce(
        (sum, slot) => sum + slot.consumedCredits,
        0,
      );
      if (
        root.maximumCredits > root.ownerCreditCap
        || root.maximumCredits > root.balanceAtReview
        || actualConsumedCredits > root.maximumCredits
      ) {
        throw new Error("ITEM-GENERATION-PROVENANCE-CREDIT-MISMATCH: reviewed cap, balance, maximum and consumed credits are inconsistent");
      }
      generationReport = {
        sessionId: String(root.sessionId),
        createdAt: String(root.createdAt),
        status: String(root.status),
        ownerCreditCap: Number(root.ownerCreditCap),
        balanceAtReview: Number(root.balanceAtReview),
        maximumCredits: Number(root.maximumCredits),
        actualConsumedCredits,
        slots: normalizedSlots,
      };
    }
    type FitPartContract = {
      field: string;
      sourceSha256: string;
      sourceNode: string | null;
      axialTargetAxis: number;
      targetAxialLength: number;
      outputBoundsMin: [number, number, number];
      outputBoundsMax: [number, number, number];
      bottomConnector: null | {
        kind: "BOTTOM";
        axialAxis: 1;
        position: [number, number, number];
      };
      topConnector: null | {
        kind: "TOP";
        axialAxis: 1;
        position: [number, number, number];
      };
      transform: unknown;
      targetSpaceScaleXyz: [number, number, number];
      transformSha256: string;
    };
    const fitPartContracts = new Map<string, FitPartContract>();
    let fitBindingReport: null | {
      status: string;
      algorithm: string;
      solutionSha256: string;
      referenceProfileSha256: string | null;
      parts: Array<{ field: string; sourceSha256: string; transformSha256: string }>;
      adjacentConnectors: Array<{
        firstField: string;
        secondField: string;
        axialOverlap: number;
        status: string;
      }>;
    } = null;
    let iconPresentationFit: null | {
      status: "PASSED";
      algorithm: string;
      solutionSha256: string;
      targetAxialLengths: [number, number, number];
      worldAttachmentUnaffected: true;
      parts: Array<{
        field: string;
        sourceSha256: string;
        transformSha256: string;
      }>;
    } = null;
    const iconPresentationTransforms = new Map<string, unknown>();
    let validatedFitReportJson: string | null = null;
    if (request.fitReportJson !== null) {
      if (/https?:\/\/|authorization|api[_-]?key|signedUrl/i.test(request.fitReportJson)) {
        throw new Error("ITEM-FIT-PROVENANCE-SECRET: fit snapshot contains a URL or credential-like field");
      }
      const rawFit = JSON.parse(request.fitReportJson) as { schemaVersion?: number };
      validatedFitReportJson = rawFit.schemaVersion === 4
        ? validateItemFitReportV4Json(request.fitReportJson)
        : validateItemFitReportV3Json(request.fitReportJson);
      const fit = JSON.parse(validatedFitReportJson) as {
        schemaVersion?: number;
        status?: string;
        algorithm?: string;
        solutionSha256?: string;
        parts?: FitPartContract[];
        adjacentConnectors?: Array<{
          firstField: string;
          secondField: string;
          axialOverlap: number;
          requiredMinOverlap: number;
          requiredMaxOverlap: number;
          surfaceStatus: string;
          status: string;
        }>;
        orientationFrame?: {
          targetAxialAxis: number;
          targetWidthAxis: number;
          targetDepthAxis: number;
          widthToDepthRatio: number;
          handednessDeterminant: number;
          evidence: string;
          status: string;
        };
        referenceProfileSha256?: string;
      };
      if (
        ![3, 4].includes(fit.schemaVersion ?? -1)
        || fit.status !== "PASSED"
        || ![
          "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX",
          "ITEM_REFERENCE_SLOT_FRAME_FIT_V1",
        ].includes(fit.algorithm ?? "")
        || (fit.schemaVersion === 4 && (
          attachmentProfileJson === null
          || (JSON.parse(attachmentProfileJson) as { profileSha256?: string }).profileSha256
            !== fit.referenceProfileSha256
        ))
        || (fit.schemaVersion === 3 && attachmentProfileJson !== null)
        || !fit.orientationFrame
        || fit.orientationFrame.status !== "PASSED"
        || fit.orientationFrame.targetAxialAxis !== 1
        || fit.orientationFrame.targetWidthAxis !== 2
        || fit.orientationFrame.targetDepthAxis !== 0
        || fit.orientationFrame.widthToDepthRatio < 1.10
        || Math.abs(fit.orientationFrame.handednessDeterminant - 1) > 1e-5
        || !Array.isArray(fit.parts)
        || fit.parts.length !== meshyParts.length
        || !Array.isArray(fit.adjacentConnectors)
        || fit.adjacentConnectors.length !== Math.max(0, meshyParts.length - 1)
        || fit.adjacentConnectors.some((connector) => (
          connector.status !== "OVERLAPPING"
          || connector.surfaceStatus === "GAP"
          || !Number.isFinite(connector.axialOverlap)
          || connector.axialOverlap <= 0
          || connector.axialOverlap < connector.requiredMinOverlap
          || connector.axialOverlap > connector.requiredMaxOverlap
        ))
        || fit.parts.some((part, index) => (
          part.axialTargetAxis !== 1
          || !Number.isFinite(part.targetAxialLength)
          || part.targetAxialLength <= 0
          || part.outputBoundsMin.length !== 3
          || part.outputBoundsMax.length !== 3
          || !Array.isArray(part.targetSpaceScaleXyz)
          || part.targetSpaceScaleXyz.length !== 3
          || part.targetSpaceScaleXyz.some((value) => !Number.isFinite(value) || value <= 0)
          || [...part.outputBoundsMin, ...part.outputBoundsMax].some((value) => !Number.isFinite(value))
          || Math.abs(
            part.outputBoundsMax[1]
            - part.outputBoundsMin[1]
            - part.targetAxialLength
          ) > 1e-4
          || Boolean(part.bottomConnector) !== (index > 0)
          || Boolean(part.topConnector) !== (index + 1 < fit.parts!.length)
        ))
      ) {
        throw new Error("ITEM-FIT-PROVENANCE-INVALID: fit snapshot is incomplete, changed, or not seam-qualified");
      }
      const boundParts = [];
      for (const part of meshyParts) {
        const fitted = fit.parts.find((candidate) => candidate.field === part.field);
        const transform = JSON.parse(part.transformJson);
        if (
          !fitted
          || fitted.sourceSha256 !== sourceHashes.get(part.field)
          || fitted.sourceNode !== part.sourceNode
          || JSON.stringify(fitted.transform) !== JSON.stringify(transform)
          || JSON.stringify(fitted.targetSpaceScaleXyz)
            !== JSON.stringify(part.targetSpaceScaleXyz ?? [1, 1, 1])
        ) {
          throw new Error(`ITEM-FIT-PROVENANCE-MISMATCH: ${part.field} source, sourceNode or transform changed after fit`);
        }
        boundParts.push({
          field: part.field,
          sourceSha256: fitted.sourceSha256,
          transformSha256: fitted.transformSha256,
        });
        fitPartContracts.set(part.field, fitted);
      }
      fitBindingReport = {
        status: fit.status!,
        algorithm: fit.algorithm!,
        solutionSha256: fit.solutionSha256!,
        referenceProfileSha256: fit.referenceProfileSha256 ?? null,
        parts: boundParts,
        adjacentConnectors: fit.adjacentConnectors!.map((connector) => ({
          firstField: connector.firstField,
          secondField: connector.secondField,
          axialOverlap: connector.axialOverlap,
          status: connector.status,
        })),
      };
    }
    if (selected.modelType === 2 && fitPartContracts.size !== meshyParts.length) {
      throw new Error(
        "ITEM-MODELTYPE2-FIT-REQUIRED: Item Properties output requires one +Y fit contract per Meshy part",
      );
    }
    if (selected.modelType === 2 && attachmentProfileJson !== null && emitsCustomIcons) {
      const targetAxialLengths: [number, number, number] = [0.22, 0.08, 0.90];
      const totalBytes = meshyParts.reduce(
        (total, part) => total + part.sourceGlb!.byteLength,
        0,
      );
      const sourceBundle = new Uint8Array(totalBytes);
      let byteOffset = 0;
      const descriptors = meshyParts.map((part) => {
        const bytes = new Uint8Array(part.sourceGlb!);
        sourceBundle.set(bytes, byteOffset);
        const descriptor = {
          field: part.field,
          modelResref: part.modelResref,
          sourceNode: part.sourceNode,
          byteOffset,
          byteLength: bytes.byteLength,
        };
        byteOffset += bytes.byteLength;
        return descriptor;
      });
      const validatedIconFitJson = validateItemFitReportV3Json(
        fitMeshyItemPartsV3Json(sourceBundle, JSON.stringify({
          schemaVersion: 3,
          tolerance: request.seamValidation.tolerance,
          targetAxialLengths,
          parts: descriptors,
        })),
      );
      const iconFit = JSON.parse(validatedIconFitJson) as {
        schemaVersion?: number;
        status?: string;
        algorithm?: string;
        solutionSha256?: string;
        parts?: Array<{
          field: string;
          sourceSha256: string;
          sourceNode: string | null;
          transform: unknown;
          transformSha256: string;
        }>;
      };
      if (
        iconFit.schemaVersion !== 3
        || iconFit.status !== "PASSED"
        || iconFit.algorithm !== "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX"
        || !/^[a-f0-9]{64}$/.test(iconFit.solutionSha256 ?? "")
        || !Array.isArray(iconFit.parts)
        || iconFit.parts.length !== meshyParts.length
      ) {
        throw new Error(
          "ITEM-ICON-PRESENTATION-FIT-INVALID: the presentation-only fit is incomplete or changed",
        );
      }
      for (const part of meshyParts) {
        const fitted = iconFit.parts.find((candidate) => candidate.field === part.field);
        if (
          !fitted
          || fitted.sourceSha256 !== sourceHashes.get(part.field)
          || fitted.sourceNode !== part.sourceNode
          || !/^[a-f0-9]{64}$/.test(fitted.transformSha256)
        ) {
          throw new Error(
            `ITEM-ICON-PRESENTATION-FIT-MISMATCH: ${part.field} is not bound to its exact source`,
          );
        }
        iconPresentationTransforms.set(part.field, fitted.transform);
      }
      iconPresentationFit = {
        status: "PASSED",
        algorithm: iconFit.algorithm,
        solutionSha256: iconFit.solutionSha256!,
        targetAxialLengths,
        worldAttachmentUnaffected: true,
        parts: iconFit.parts.map((part) => ({
          field: part.field,
          sourceSha256: part.sourceSha256,
          transformSha256: part.transformSha256,
        })),
      };
    }
    const partOptionsJson = (part: typeof meshyParts[number] & { weaponColor?: number | null }) => JSON.stringify({
      schemaVersion: 1,
      transform: JSON.parse(part.transformJson),
      sourceNode: part.sourceNode,
      textureEncoding: part.textureEncoding,
      iconSize: null,
      iconProjectionBounds: null,
      weaponColor: part.weaponColor ?? null,
      targetSpaceScaleXyz: part.targetSpaceScaleXyz ?? [1, 1, 1],
    });
    const seamResults: Array<{
      firstField: string;
      secondField: string;
      status: "TOUCHING" | "GAP" | "OVERLAP";
      gap: number;
      overlap: boolean;
      requiredRelation: "ADJACENT_CONNECTED" | "NON_ADJACENT_NO_OVERLAP";
    }> = [];
    const logicalPartIndex = new Map(
      orderedParts.map((part, index) => [part.field.toLowerCase(), index]),
    );
    for (let firstIndex = 0; firstIndex < meshyParts.length; firstIndex += 1) {
      for (let secondIndex = firstIndex + 1; secondIndex < meshyParts.length; secondIndex += 1) {
        const first = meshyParts[firstIndex];
        const second = meshyParts[secondIndex];
        const firstLogicalIndex = logicalPartIndex.get(first.field.toLowerCase())!;
        const secondLogicalIndex = logicalPartIndex.get(second.field.toLowerCase())!;
        const requiredRelation = Math.abs(firstLogicalIndex - secondLogicalIndex) === 1
          ? "ADJACENT_CONNECTED" as const
          : "NON_ADJACENT_NO_OVERLAP" as const;
        seamResults.push({
          ...(JSON.parse(measureMeshyItemSeamV1Json(
            first.field,
            new Uint8Array(first.sourceGlb!),
            first.modelResref,
            partOptionsJson(first),
            second.field,
            new Uint8Array(second.sourceGlb!),
            second.modelResref,
            partOptionsJson(second),
            request.seamValidation.tolerance,
          )) as {
            firstField: string;
            secondField: string;
            status: "TOUCHING" | "GAP" | "OVERLAP";
            gap: number;
            overlap: boolean;
          }),
          requiredRelation,
        });
      }
    }
    const failedSeams = seamResults.filter(
      (result) => result.requiredRelation === "ADJACENT_CONNECTED"
        ? result.status === "GAP" || result.gap > request.seamValidation.tolerance
        : result.overlap,
    );
    if (failedSeams.length > 0) {
      throw new Error(
        `ITEM-SEAM-VALIDATION-FAILED: ${failedSeams.map((result) => (
          `${result.firstField}/${result.secondField}=${result.status}`
        )).join(", ")}`,
      );
    }
    const buildItemPart = selected.modelType === 2
      ? buildMeshyItemPartWithOptionsV3
      : buildMeshyItemPartWithOptionsV2;
    for (const part of buildParts) {
      const result = buildItemPart(
        new Uint8Array(part.sourceGlb!),
        part.modelResref,
        part.textureResref,
        partOptionsJson(part),
      );
      try {
        const report = JSON.parse(result.reportJson) as PartOutput["report"];
        const icon = exactBuffer(result.takeIconBytes());
        if (icon.byteLength !== 0) {
          throw new Error(`ITEM-ICON-PREFLIGHT-UNEXPECTED: ${part.field} emitted icon bytes without an icon size`);
        }
        outputs.push({
          field: part.field,
          variant: part.variant,
          weaponColor: part.weaponColor,
          modelResref: part.modelResref,
          iconResref: part.iconResref,
          textureResref: part.textureResref,
          mdl: exactBuffer(result.takeMdlBytes()),
          texture: exactBuffer(result.takeTextureBytes()),
          icon,
          report,
          readback: JSON.parse(result.readbackJson),
        });
      } finally {
        result.free();
      }
    }
    if (emitsCustomIcons && outputs.length > 0) {
      const iconProjectionReports = iconPresentationFit
        ? buildParts.map((part, index) => {
            const transform = iconPresentationTransforms.get(part.field);
            if (!transform) {
              throw new Error(
                `ITEM-ICON-PRESENTATION-FIT-MISSING: ${part.field} has no presentation transform`,
              );
            }
            const result = buildItemPart(
              new Uint8Array(part.sourceGlb!),
              part.modelResref,
              part.textureResref,
              JSON.stringify({
                schemaVersion: 1,
                transform,
                sourceNode: part.sourceNode,
                textureEncoding: part.textureEncoding,
                iconSize: null,
                iconProjectionBounds: null,
                weaponColor: part.weaponColor,
                targetSpaceScaleXyz: [1, 1, 1],
              }),
            );
            try {
              const report = JSON.parse(result.reportJson) as PartOutput["report"];
              if (report.textureSha256 !== outputs[index].report.textureSha256) {
                throw new Error(
                  `ITEM-ICON-PRESENTATION-TEXTURE-MISMATCH: ${part.field} changed texture bytes`,
                );
              }
              result.takeMdlBytes();
              result.takeTextureBytes();
              result.takeIconBytes();
              return report;
            } finally {
              result.free();
            }
          })
        : outputs.map((output) => output.report);
      const projectionBounds = iconProjectionReports.reduce<IconProjectionBounds | undefined>(
        (combined, report, index) => {
          const bounds = report.iconProjectionBounds;
          if (!bounds) {
            throw new Error(
              `ITEM-ICON-PROJECTION-MISSING: ${buildParts[index].field} emitted no geometry projection bounds`,
            );
          }
          return combined
            ? {
                min: [
                  Math.min(combined.min[0], bounds.min[0]),
                  Math.min(combined.min[1], bounds.min[1]),
                ],
                max: [
                  Math.max(combined.max[0], bounds.max[0]),
                  Math.max(combined.max[1], bounds.max[1]),
                ],
              }
            : {
                min: [...bounds.min] as [number, number],
                max: [...bounds.max] as [number, number],
              };
        },
        undefined,
      );
      if (!projectionBounds) {
        throw new Error("ITEM-ICON-PROJECTION-MISSING: no Meshy geometry was available");
      }
      const width = Math.max(1, selected.invSlotWidth * 32);
      const height = Math.max(1, selected.invSlotHeight * 32);
      for (let index = 0; index < buildParts.length; index += 1) {
        const part = buildParts[index];
        const output = outputs[index];
        const iconTransform = iconPresentationTransforms.get(part.field)
          ?? JSON.parse(part.transformJson);
        const result = buildItemPart(
          new Uint8Array(part.sourceGlb!),
          part.modelResref,
          part.textureResref,
          JSON.stringify({
            schemaVersion: 1,
            transform: iconTransform,
            sourceNode: part.sourceNode,
            textureEncoding: part.textureEncoding,
            iconSize: [width, height],
            iconProjectionBounds: projectionBounds,
            weaponColor: part.weaponColor,
            targetSpaceScaleXyz: iconPresentationFit
              ? [1, 1, 1]
              : part.targetSpaceScaleXyz ?? [1, 1, 1],
          }),
        );
        try {
          const report = JSON.parse(result.reportJson) as PartOutput["report"];
          const icon = exactBuffer(result.takeIconBytes());
          if (
            icon.byteLength === 0
            || !report.iconOpaquePixelCount
            || report.textureSha256 !== output.report.textureSha256
          ) {
            throw new Error(
              `ITEM-ICON-LAYER-MISMATCH: ${part.field} failed deterministic shared-frame rasterization`,
            );
          }
          result.takeMdlBytes();
          result.takeTextureBytes();
          output.icon = icon;
          output.report.iconSha256 = report.iconSha256;
          output.report.iconProjectionBounds = report.iconProjectionBounds;
          output.report.iconOpaquePixelCount = report.iconOpaquePixelCount;
        } finally {
          result.free();
        }
      }
    }
    let itemIconConformance: {
      status: "PASSED" | "NOT_APPLICABLE";
      algorithm: "AURORA_MODELTYPE2_ICON_LAYER_COMPOSITE_V3" | null;
      layoutProfile: "LONG_VERTICAL_PART_ORDER_V2" | null;
      colorways: Array<{
        color: number;
        opaquePixelCount: number;
        boundsMin: [number, number];
        boundsMaxExclusive: [number, number];
        axialFillRatio: number;
        occupiedFillRatio: number;
        partOrderStatus: "PASSED";
        compositeRgbaSha256: string;
      }>;
    } = {
      status: "NOT_APPLICABLE",
      algorithm: null,
      layoutProfile: null,
      colorways: [],
    };
    if (selected.modelType === 2 && emitsCustomIcons) {
      const width = Math.max(1, selected.invSlotWidth * 32);
      const height = Math.max(1, selected.invSlotHeight * 32);
      if (height < width * 2) {
        throw new Error(
          "ITEM-ICON-LAYOUT-MANUAL_REQUIRED: ModelType 2 canvas is not a long vertical profile",
        );
      }
      const colorways = [];
      for (const color of [1, 2, 3, 4]) {
        const group = meshyParts.map((part) => {
          const output = outputs.find((candidate) => (
            candidate.field === part.field && candidate.weaponColor === color
          ));
          if (!output || output.icon.byteLength === 0) {
            throw new Error(`ITEM-ICON-V3-COLORWAY-INCOMPLETE: color ${color} has no ${part.field} layer`);
          }
          return output;
        });
        const byteLength = group.reduce((sum, output) => sum + output.icon.byteLength, 0);
        const iconBundle = new Uint8Array(byteLength);
        let iconOffset = 0;
        const layers = group.map((output) => {
          const icon = new Uint8Array(output.icon);
          iconBundle.set(icon, iconOffset);
          const layer = {
            field: output.field,
            iconResref: output.iconResref,
            byteOffset: iconOffset,
            byteLength: icon.byteLength,
          };
          iconOffset += icon.byteLength;
          return layer;
        });
        const composed = JSON.parse(validateItemModelType2IconLayersV3Json(
          iconBundle,
          JSON.stringify({
            schemaVersion: 3,
            width,
            height,
            layoutProfile: "LONG_VERTICAL_PART_ORDER_V2",
            layers,
          }),
        )) as {
          status: "PASSED";
          algorithm: "AURORA_MODELTYPE2_ICON_LAYER_COMPOSITE_V3";
          opaquePixelCount: number;
          boundsMin: [number, number];
          boundsMaxExclusive: [number, number];
          axialFillRatio: number;
          occupiedFillRatio: number;
          partOrderStatus: "PASSED";
          compositeRgbaSha256: string;
        };
        colorways.push({ color, ...composed });
      }
      itemIconConformance = {
        status: "PASSED",
        algorithm: "AURORA_MODELTYPE2_ICON_LAYER_COMPOSITE_V3",
        layoutProfile: "LONG_VERTICAL_PART_ORDER_V2",
        colorways: colorways.map((colorway) => ({
          color: colorway.color,
          opaquePixelCount: colorway.opaquePixelCount,
          boundsMin: colorway.boundsMin,
          boundsMaxExclusive: colorway.boundsMaxExclusive,
          axialFillRatio: colorway.axialFillRatio,
          occupiedFillRatio: colorway.occupiedFillRatio,
          partOrderStatus: colorway.partOrderStatus,
          compositeRgbaSha256: colorway.compositeRgbaSha256,
        })),
      };
    }
    let itemPropertiesModelConformance: {
      status: "PASSED" | "NOT_APPLICABLE";
      algorithm: "ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V2" | null;
      axialTargetAxis: 1 | null;
      checkedMdlCount: number;
      appendOrder: string[];
      colorways: Array<{
        color: number;
        compositeBoundsMin: [number, number, number];
        compositeBoundsMax: [number, number, number];
        totalTriangleCount: number;
        parts: Array<{
          field: string;
          rootControllerOwner: string;
          transformControllerOwner: string;
          meshNodeNames: string[];
        }>;
      }>;
    } = {
      status: "NOT_APPLICABLE",
      algorithm: null,
      axialTargetAxis: null,
      checkedMdlCount: 0,
      appendOrder: [],
      colorways: [],
    };
    if (selected.modelType === 2) {
      if (!validatedFitReportJson) {
        throw new Error("ITEM-MODELTYPE2-COMPOSER-FIT-MISSING: node-aware validation requires an exact full-frame or reference-frame fit report");
      }
      const colorways = [];
      for (const color of [1, 2, 3, 4]) {
        const group = meshyParts.map((part) => {
          const output = outputs.find((candidate) => (
            candidate.field === part.field && candidate.weaponColor === color
          ));
          if (!output) {
            throw new Error(
              `ITEM-MODELTYPE2-COLORWAY-INCOMPLETE: color ${color} has no ${part.field} MDL`,
            );
          }
          return output;
        });
        const byteLength = group.reduce((sum, output) => sum + output.mdl.byteLength, 0);
        const mdlBundle = new Uint8Array(byteLength);
        let mdlOffset = 0;
        const descriptors = group.map((output) => {
          const mdl = new Uint8Array(output.mdl);
          mdlBundle.set(mdl, mdlOffset);
          const descriptor = {
            field: output.field,
            modelResref: output.modelResref,
            byteOffset: mdlOffset,
            byteLength: mdl.byteLength,
          };
          mdlOffset += mdl.byteLength;
          return descriptor;
        });
        const fitSchemaVersion = (JSON.parse(validatedFitReportJson) as { schemaVersion: number }).schemaVersion;
        const composerJson = fitSchemaVersion === 4
          ? validateItemModelType2ComposerV4Json(
              mdlBundle,
              JSON.stringify({ schemaVersion: 4, parts: descriptors }),
              validatedFitReportJson,
            )
          : validateItemModelType2ComposerV3Json(
              mdlBundle,
              JSON.stringify({ schemaVersion: 3, parts: descriptors }),
              validatedFitReportJson,
            );
        const composer = JSON.parse(composerJson) as {
          status: "PASSED";
          algorithm: "ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V2";
          appendOrder: string[];
          compositeBoundsMin: [number, number, number];
          compositeBoundsMax: [number, number, number];
          totalTriangleCount: number;
          parts: Array<{
            field: string;
            rootControllerOwner: string;
            transformControllerOwner: string;
            meshNodeNames: string[];
          }>;
        };
        colorways.push({ color, ...composer });
      }
      itemPropertiesModelConformance = {
        status: "PASSED",
        algorithm: "ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V2",
        axialTargetAxis: 1,
        checkedMdlCount: outputs.length,
        appendOrder: colorways[0].appendOrder,
        colorways: colorways.map((colorway) => ({
          color: colorway.color,
          compositeBoundsMin: colorway.compositeBoundsMin,
          compositeBoundsMax: colorway.compositeBoundsMax,
          totalTriangleCount: colorway.totalTriangleCount,
          parts: colorway.parts,
        })),
      };
    }
    const selectedOutputs = meshyParts.map((part) => {
      const output = outputs.find((candidate) => (
        candidate.field === part.field && candidate.variant === part.variant
      ));
      if (!output) {
        throw new Error(
          `ITEM-WEAPON-COLORWAY-SELECTION-MISSING: ${part.field} has no built resource for UTI value ${part.variant}`,
        );
      }
      return output;
    });
    const utiResult = writeItemUtiV1(
      new Uint8Array(effectiveBaseitemsTwoDa),
      request.baseItem,
      request.blueprintJson,
    );
    let uti: ArrayBuffer;
    let utiReportJson: string;
    try {
      uti = exactBuffer(utiResult.takeUtiBytes());
      utiReportJson = utiResult.reportJson;
    } finally {
      utiResult.free();
    }
    const equippedProofProfile = selected.capability.iconProfile === "CAPART_COMPOSITE"
      ? "CAPART_ARMOR"
      : selected.capability.iconProfile === "CLOAK_MODEL"
        ? "CLOAK"
        : null;
    if (equippedProofProfile && !request.equippedProofContext) {
      throw new Error(
        "ITEM-EQUIPPED-PROOF-CONTEXT-MISSING: this equipped Item proof requires a creature resref and Appearance row",
      );
    }
    if (!equippedProofProfile && request.equippedProofContext) {
      throw new Error(
        "ITEM-EQUIPPED-PROOF-CONTEXT-UNEXPECTED: this Item does not use an equipped proof profile",
      );
    }
    if (
      equippedProofProfile
      && (
        !Number.isInteger(request.equippedProofContext!.appearanceRow)
        || request.equippedProofContext!.appearanceRow < 0
        || request.equippedProofContext!.appearanceRow > 0xffff
        || !Number.isInteger(request.equippedProofContext!.race)
        || request.equippedProofContext!.race < 0
        || request.equippedProofContext!.race > 0xff
        || !Number.isInteger(request.equippedProofContext!.gender)
        || request.equippedProofContext!.gender < 0
        || request.equippedProofContext!.gender > 0xff
        || !Number.isInteger(request.equippedProofContext!.phenotype)
        || request.equippedProofContext!.phenotype < 0
        || request.equippedProofContext!.phenotype > 0xff
      )
    ) {
      throw new Error(
        "ITEM-EQUIPPED-PROOF-CONTEXT-INVALID: Appearance must fit WORD and Race/Gender/Phenotype must fit the supported 0..255 context",
      );
    }
    const moduleResourceKeys = [
      `2012:${request.areaResref.toLowerCase()}`,
      `2023:${request.areaResref.toLowerCase()}`,
      `2046:${request.areaResref.toLowerCase()}`,
      ...(equippedProofProfile
        ? [`2027:${request.equippedProofContext!.creatureResref.toLowerCase()}`]
        : []),
    ];
    const occupiedModuleResource = moduleResourceKeys.find(
      (key) => occupiedResourceKeys.has(key),
    );
    if (occupiedModuleResource) {
      throw new Error(
        `ITEM-RESOURCE-COLLISION: occupied proof-module resource ${occupiedModuleResource}`,
      );
    }
    const appearanceBinding = equippedProofProfile
      ? JSON.parse(resolveItemEquippedAppearanceV1Json(
          new Uint8Array(referenceTables.get("APPEARANCE")!),
          request.equippedProofContext!.appearanceRow,
          request.equippedProofContext!.race,
          request.equippedProofContext!.gender,
          request.equippedProofContext!.phenotype,
        )) as {
          modelPrefix: string;
          appearanceTableSha256: string;
        }
      : null;
    const equipmentSlot = selected.equipableSlots;
    if (equippedProofProfile === "CAPART_ARMOR") {
      if (
        !request.capartContext
        || request.capartContext.genderCode
        || request.capartContext.modelPrefix !== appearanceBinding!.modelPrefix
      ) {
        throw new Error(
          `ITEM-EQUIPPED-PROOF-APPEARANCE-CONTEXT-MISMATCH: CAPART modelPrefix must be ${appearanceBinding!.modelPrefix} and genderCode must be empty`,
        );
      }
    }
    const placementJson = JSON.stringify({
      schemaVersion: 1,
      x: 10,
      y: 14.5,
      z: 0,
      orientationX: 0,
      orientationY: -1,
    });
    const moduleResult = equippedProofProfile
      ? buildItemEquippedProofModuleV2(
          new Uint8Array(uti),
          JSON.stringify({
            schemaVersion: 2,
            moduleResref: request.moduleResref,
            areaResref: request.areaResref,
            hakResref: request.hakResref,
            blueprintResref: request.blueprintResref,
            moduleName: request.moduleName,
            areaName: request.areaName,
            creatureResref: request.equippedProofContext!.creatureResref,
            creatureDisplayName: equippedProofProfile === "CAPART_ARMOR"
              ? "Meshy2Aurora CAPART proof wearer"
              : "Meshy2Aurora Cloak proof wearer",
            appearanceRow: request.equippedProofContext!.appearanceRow,
            race: request.equippedProofContext!.race,
            gender: request.equippedProofContext!.gender,
            phenotype: request.equippedProofContext!.phenotype,
            modelPrefix: appearanceBinding!.modelPrefix,
            appearanceTableSha256: appearanceBinding!.appearanceTableSha256,
            fixtureProfile: equippedProofProfile,
            equipmentSlot,
          }),
          placementJson,
        )
      : buildItemProofModuleV1(
          new Uint8Array(uti),
          JSON.stringify({
            schemaVersion: 1,
            moduleResref: request.moduleResref,
            areaResref: request.areaResref,
            hakResref: request.hakResref,
            blueprintResref: request.blueprintResref,
            moduleName: request.moduleName,
            areaName: request.areaName,
          }),
          placementJson,
        );
    let module: ArrayBuffer;
    let moduleReport: Record<string, unknown>;
    try {
      module = exactBuffer(moduleResult.takeModuleBytes());
      moduleReport = JSON.parse(moduleResult.reportJson) as Record<string, unknown>;
    } finally {
      moduleResult.free();
    }

    const properties = (blueprint as {
      properties?: Array<Record<string, number>>;
    }).properties ?? [];
    const capartBaseResolution = selected.capability.iconProfile === "CAPART_COMPOSITE"
      ? orderedParts.map((part) => {
          const slot = selected.partSlots.find(
            (candidate) => candidate.field.toLowerCase() === part.field.toLowerCase(),
          );
          if (
            part.sourceKind !== "CAPART_SELECTION"
            || !slot?.referenceTable
          ) {
            throw new Error(
              `ITEM-CAPART-SLOT-INVALID: ${part.field} has no exact parts_* table mapping`,
            );
          }
          const partsTable = referenceTables.get(slot.referenceTable.toUpperCase());
          if (!partsTable) {
            throw new Error(
              `ITEM-REFERENCE-TABLE-MISSING: ${slot.referenceTable}.2da is required by ${part.field}`,
            );
          }
          return JSON.parse(resolveItemCapartPartV1Json(
            part.field,
            part.variant,
            new Uint8Array(referenceTables.get("CAPART")!),
            slot.referenceTable,
            new Uint8Array(partsTable),
          ));
        })
      : null;
    const capartResolution = capartBaseResolution
      ? (() => {
          if (!request.capartContext) {
            throw new Error(
              "ITEM-CAPART-CONTEXT-MISSING: CAPART requires an exact modelPrefix and optional genderCode",
            );
          }
          const robeHidden = new Set<string>(
            (capartBaseResolution.find((part) => part.mdlName === "ROBE")
              ?.robeHiddenMdlNames ?? []) as string[],
          );
          return orderedParts.map((part, index) => {
            const slot = selected.partSlots.find(
              (candidate) => candidate.field.toLowerCase() === part.field.toLowerCase(),
            )!;
            const base = capartBaseResolution[index] as { mdlName: string };
            return JSON.parse(resolveItemCapartPartV2Json(
              part.field,
              part.variant,
              new Uint8Array(referenceTables.get("CAPART")!),
              slot.referenceTable!,
              new Uint8Array(referenceTables.get(slot.referenceTable!.toUpperCase())!),
              JSON.stringify(request.capartContext),
              JSON.stringify(resourceInventory),
              base.mdlName !== "ROBE" && robeHidden.has(base.mdlName),
            ));
          });
        })()
      : null;
    const specialResolution = selected.capability.iconProfile === "IPRP_SPELL"
      ? JSON.parse(resolveItemCastSpellIconV1Json(
          request.baseItem,
          JSON.stringify(properties),
          new Uint8Array(referenceTables.get("IPRP_SPELLS")!),
        ))
      : selected.capability.iconProfile === "CLOAK_MODEL"
        ? JSON.parse(resolveItemCloakV4Json(
            orderedParts[0]?.variant ?? 0,
            appearanceBinding!.modelPrefix,
            new Uint8Array(referenceTables.get("CLOAKMODEL")!),
            JSON.stringify(resourceInventory),
          ))
        : capartResolution;
    const referenceTriangleCounts = selected.capability.iconProfile === "CAPART_COMPOSITE"
      ? (capartResolution as Array<{
          modelResource?: { triangleCount?: number } | null;
        }>).flatMap((resolution) => (
          typeof resolution.modelResource?.triangleCount === "number"
            ? [resolution.modelResource.triangleCount]
            : []
        ))
      : selected.capability.iconProfile === "CLOAK_MODEL"
        ? [Number((specialResolution as {
            modelResource?: { triangleCount?: number };
          }).modelResource?.triangleCount)]
        : [];
    if (referenceTriangleCounts.some((count) => !Number.isSafeInteger(count) || count < 0)) {
      throw new Error(
        "ITEM-REFERENCE-TRIANGLE-COUNT-INVALID: every resolved retail MDL must report a non-negative safe triangle count",
      );
    }
    const budget = JSON.parse(validateItemTriangleBudgetV1Json(JSON.stringify([
      ...selectedOutputs.map((output) => output.report.triangleCount),
      ...referenceTriangleCounts,
    ]))) as {
      triangleCount: number;
      triangleBudget: number;
      warning: boolean;
      warningAbove: number;
    };

    const resources: Array<{
      resref: string;
      resourceType: number;
      payload: ArrayBuffer;
      allowOccupiedOverride?: boolean;
    }> = outputs.flatMap((output) => [
      { resref: output.modelResref, resourceType: 2002, payload: output.mdl },
      {
        resref: output.textureResref,
        resourceType: output.report.textureFormat === "PLT_V1" ? 6 : 3,
        payload: output.texture,
      },
      ...(emitsCustomIcons
        ? [{ resref: output.iconResref, resourceType: 3, payload: output.icon }]
        : []),
    ]);
    resources.push({ resref: request.blueprintResref, resourceType: 2025, payload: uti });
    if (baseitemsModelRange?.status === "PATCHED") {
      resources.push({
        resref: "baseitems",
        resourceType: 2017,
        payload: effectiveBaseitemsTwoDa,
        allowOccupiedOverride: true,
      });
    }
    const resourceKeys = new Set(
      [...occupiedResourceKeys].filter((key) => /^\d+:/.test(key)),
    );
    for (const resource of resources) {
      const key = `${resource.resourceType}:${resource.resref.toLowerCase()}`;
      if (resourceKeys.has(key) && !resource.allowOccupiedOverride) {
        throw new Error(`ITEM-RESOURCE-COLLISION: duplicate package resource ${key}`);
      }
      resourceKeys.add(key);
    }
    let payloadOffset = 0;
    const descriptors = resources.map((resource) => {
      const descriptor = {
        resref: resource.resref,
        resourceType: resource.resourceType,
        payloadOffset,
        payloadSize: resource.payload.byteLength,
      };
      payloadOffset += resource.payload.byteLength;
      return descriptor;
    });
    const payloadBlob = new Uint8Array(payloadOffset);
    let cursor = 0;
    for (const resource of resources) {
      payloadBlob.set(new Uint8Array(resource.payload), cursor);
      cursor += resource.payload.byteLength;
    }
    const hak = exactBuffer(writeHakV1(
      payloadBlob,
      JSON.stringify({ schemaVersion: 1, resources: descriptors }),
      JSON.stringify({
        schemaVersion: 1,
        limits: {
          maxEntryCount: 262_144,
          maxOutputBytes: 268_435_456,
        },
      }),
    ));
    const resourceManifest = await Promise.all(resources.map(async (resource) => ({
      resref: resource.resref,
      resourceType: resource.resourceType,
      byteLength: resource.payload.byteLength,
      sha256: await sha256(resource.payload),
    })));
    const yamlScalar = (value: string) => (
      /^[a-z0-9][a-z0-9._/-]*$/i.test(value)
      && !/^(?:true|false|null|~)$/i.test(value)
      && !/^[0-9]+$/.test(value)
        ? value
        : JSON.stringify(value)
    );
    const normalizedSessionId = generationReport?.sessionId
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 64);
    const sourceAssetId = meshyParts.length > 0
      ? `item-${request.baseItem}-${normalizedSessionId || sourceHashes.get(meshyParts[0].field)!.slice(0, 16)}`
      : null;
    const sourceBundleFiles = meshyParts.map((part, index) => {
      const generationSlot = generationReport?.slots.find((slot) => slot.field === part.field);
      const slotName = meshyParts.length === 1
        ? "model"
        : generationSlot?.role.toLowerCase()
          ?? ["bottom", "middle", "top"][index]
          ?? `part-${index + 1}`;
      return {
        field: part.field,
        role: meshyParts.length === 1 ? "source-model" : `source-modelpart-${slotName}`,
        fileName: `${slotName}.glb`,
        byteLength: part.sourceGlb!.byteLength,
        sha256: sourceHashes.get(part.field)!,
      };
    });
    let sourceManifestBytes: ArrayBuffer | null = null;
    let sourceBundleReport: null | {
      schema: "meshy2aurora.sample-3d/v1";
      assetId: string;
      manifestFileName: "manifest.yaml";
      manifestSha256: string;
      files: typeof sourceBundleFiles;
    } = null;
    if (sourceAssetId) {
      const lines = [
        "schema: meshy2aurora.sample-3d/v1",
        `asset_id: ${yamlScalar(sourceAssetId)}`,
        `provider: ${generationReport ? "meshy" : "owner-import"}`,
        "local_only: true",
        `qualification: ${generationReport ? "generated-unqualified" : "imported-unqualified"}`,
        "files:",
      ];
      for (const file of sourceBundleFiles) {
        lines.push(
          `  - role: ${yamlScalar(file.role)}`,
          `    path: ${yamlScalar(file.fileName)}`,
          `    field: ${yamlScalar(file.field)}`,
          `    size_bytes: ${file.byteLength}`,
          `    sha256: ${yamlScalar(file.sha256)}`,
        );
      }
      lines.push(
        `provenance_note: ${JSON.stringify(generationReport
          ? "Meshy image-to-3D Item generation; exact concepts, tasks, source GLBs and fit are bound below"
          : "Owner-imported Item source GLB payloads; no Meshy task lineage was supplied")}`,
        "item_identity:",
        `  baseitems_sha256: ${yamlScalar(catalog.sourceSha256)}`,
        `  base_item: ${request.baseItem}`,
        `  item_class: ${yamlScalar(selected.itemClass)}`,
        `  model_type: ${selected.modelType}`,
      );
      if (generationReport) {
        lines.push(
          "generation_session:",
          `  session_id: ${yamlScalar(generationReport.sessionId)}`,
          `  created_at: ${yamlScalar(generationReport.createdAt)}`,
          `  status: ${yamlScalar(generationReport.status)}`,
          `  owner_credit_cap: ${generationReport.ownerCreditCap}`,
          `  balance_at_review: ${generationReport.balanceAtReview}`,
          `  maximum_credits: ${generationReport.maximumCredits}`,
          `  actual_consumed_credits: ${generationReport.actualConsumedCredits}`,
          "  parts:",
        );
        for (const slot of generationReport.slots) {
          lines.push(
            `    - field: ${yamlScalar(slot.field)}`,
            `      role: ${yamlScalar(slot.role.toLowerCase())}`,
            `      profile_id: ${yamlScalar(slot.profileId)}`,
            `      target_polycount: ${slot.targetPolycount}`,
            `      concept_sha256: ${yamlScalar(slot.conceptSha256)}`,
            `      run_id: ${yamlScalar(slot.runId)}`,
            `      task_id: ${yamlScalar(slot.taskId)}`,
            `      run_created_at: ${yamlScalar(slot.runCreatedAt)}`,
            `      task_created_at: ${yamlScalar(slot.taskCreatedAt)}`,
            `      output: ${yamlScalar(sourceBundleFiles.find((file) => file.field === slot.field)!.fileName)}`,
            `      output_sha256: ${yamlScalar(slot.glbSha256)}`,
            `      size_bytes: ${slot.byteLength}`,
            `      consumed_credits: ${slot.consumedCredits}`,
            `      finished_at: ${yamlScalar(slot.finishedAt)}`,
          );
        }
      }
      if (fitBindingReport) {
        lines.push(
          "fit:",
          `  status: ${yamlScalar(fitBindingReport.status)}`,
          `  algorithm: ${yamlScalar(fitBindingReport.algorithm)}`,
          `  solution_sha256: ${yamlScalar(fitBindingReport.solutionSha256)}`,
          "  parts:",
        );
        for (const part of fitBindingReport.parts) {
          lines.push(
            `    - field: ${yamlScalar(part.field)}`,
            `      source_sha256: ${yamlScalar(part.sourceSha256)}`,
            `      transform_sha256: ${yamlScalar(part.transformSha256)}`,
          );
        }
        lines.push("  adjacent_connectors:");
        for (const connector of fitBindingReport.adjacentConnectors) {
          lines.push(
            `    - first_field: ${yamlScalar(connector.firstField)}`,
            `      second_field: ${yamlScalar(connector.secondField)}`,
            `      axial_overlap: ${connector.axialOverlap}`,
            `      status: ${yamlScalar(connector.status)}`,
          );
        }
      }
      sourceManifestBytes = exactBuffer(encoder.encode(`${lines.join("\n")}\n`));
      sourceBundleReport = {
        schema: "meshy2aurora.sample-3d/v1",
        assetId: sourceAssetId,
        manifestFileName: "manifest.yaml",
        manifestSha256: await sha256(sourceManifestBytes),
        files: sourceBundleFiles,
      };
    }
    const report = {
      schemaVersion: 1,
      status: "OFFLINE_ITEM_PACKAGE_PASSED",
      profile: "ITEM",
      baseItem: request.baseItem,
      partCount: request.parts.length,
      meshyPartCount: meshyParts.length,
      referenceSelectorCount: request.parts.length - meshyParts.length,
      iconLayerCount: emitsCustomIcons ? outputs.length : 0,
      weaponColorwayCoverage: {
        status: selected.modelType === 2 ? "COMPLETE" : "NOT_APPLICABLE",
        expectedResourceCount: selected.modelType === 2 ? meshyParts.length * 4 : 0,
        emittedResourceCount: selected.modelType === 2 ? outputs.length : 0,
        colors: selected.modelType === 2 ? [1, 2, 3, 4] : [],
        geometryReuse: selected.modelType === 2 ? "ONE_MESHY_GLB_PER_PART" : null,
      },
      iconLayerMode: selected.capability.iconProfile === "IPRP_SPELL"
        ? "IPRP_SPELLS_ICON_RESOLUTION"
        : selected.capability.iconProfile === "CAPART_COMPOSITE"
          ? "RETAIL_CAPART_COMPOSITION_PLAN_V1"
          : selected.capability.iconProfile === "CLOAK_MODEL"
            ? "RETAIL_CLOAKMODEL_ICON"
            : "AURORA_MODELTYPE2_ICON_LAYERS_V3",
      iconRuntimeParity: itemIconConformance.status === "PASSED"
        ? "offline_native_layer_composite_validated"
        : "offline_semantic_readback_only",
      triangleBudget: budget,
      generation: generationReport,
      fit: fitBindingReport,
      iconPresentationFit,
      attachmentProfile: attachmentProfileJson
        ? JSON.parse(attachmentProfileJson)
        : null,
      baseitemsModelRange,
      itemPropertiesModelConformance,
      itemIconConformance,
      sourceBundle: sourceBundleReport,
      seamValidation: {
        tolerance: request.seamValidation.tolerance,
        status: seamResults.length > 0 ? "PASSED" : "NOT_APPLICABLE",
        results: seamResults,
      },
      uti: JSON.parse(utiReportJson),
      partReports: outputs.map((output) => output.report),
      referenceTables: referenceTableReports,
      referenceResourceManifest: request.referenceResourceManifest
        ? {
            fileName: request.referenceResourceManifest.fileName,
            sha256: retailManifestSha256,
            sourceContainerSha256: retailManifest?.sourceContainerSha256,
          }
        : null,
      referenceResourceInventory: resourceInventory,
      specialResolution,
      resources: resourceManifest,
      hakResref: request.hakResref,
      hakFileName: request.hakFileName,
      hakSha256: await sha256(hak),
      proofModule: moduleReport,
      moduleFileName: request.moduleFileName,
      moduleSha256: await sha256(module),
      modelVisibility: "not_tested",
      proofCompleteness: "missing",
      readyForOwnerProof: false,
      proofBlocker: "The exact candidate MOD/HAK pair has been emitted but has not yet been installed and byte-verified in the native NWN user directories.",
    };
    const reportJson = JSON.stringify(report);
    const artifactList = await Promise.all([
      artifact(
        "item-package-hak",
        "HAK",
        request.hakFileName,
        "application/octet-stream",
        hak,
      ),
      artifact(
        "item-proof-module",
        "MODULE",
        request.moduleFileName,
        "application/octet-stream",
        module,
      ),
      artifact(
        "item-blueprint-uti",
        "ITEM_BLUEPRINT",
        `${request.blueprintResref}.uti`,
        "application/octet-stream",
        uti,
      ),
      ...outputs.flatMap((output, index) => [
        artifact(
          `item-part-${index}-mdl`,
          "MODEL" as const,
          `${output.modelResref}.mdl`,
          "application/octet-stream",
          output.mdl,
        ),
        artifact(
          `item-part-${index}-texture`,
          "TEXTURE" as const,
          `${output.textureResref}.${output.report.textureFormat === "PLT_V1" ? "plt" : "tga"}`,
          output.report.textureFormat === "PLT_V1" ? "application/octet-stream" : "image/x-tga",
          output.texture,
        ),
        ...(emitsCustomIcons ? [artifact(
          `item-part-${index}-icon`,
          "TEXTURE" as const,
          `${output.iconResref}.tga`,
          "image/x-tga",
          output.icon,
        )] : []),
      ]),
      ...sourceBundleFiles.map((file, index) => artifact(
        `item-source-${file.role.replace(/^source-model(?:part-)?/, "") || index + 1}`,
        "SOURCE_MODEL" as const,
        file.fileName,
        "model/gltf-binary",
        meshyParts[index].sourceGlb!.slice(0),
      )),
      ...(sourceManifestBytes ? [artifact(
        "item-source-manifest",
        "SOURCE_MANIFEST" as const,
        "manifest.yaml",
        "application/yaml",
        sourceManifestBytes,
      )] : []),
      ...(attachmentProfileJson ? [artifact(
        "item-attachment-profile",
        "JSON_REPORT" as const,
        "item-attachment-profile.json",
        "application/json",
        encoder.encode(attachmentProfileJson).buffer,
      )] : []),
      ...(validatedFitReportJson ? [artifact(
        "item-reference-fit-report",
        "JSON_REPORT" as const,
        "item-reference-fit-report.json",
        "application/json",
        encoder.encode(validatedFitReportJson).buffer,
      )] : []),
      artifact(
        "item-build-report",
        "JSON_REPORT",
        "item-build-report.json",
        "application/json",
        encoder.encode(reportJson).buffer,
      ),
    ]);
    return {
      requestId: request.requestId,
      ok: true,
      type: "ITEM_PACKAGE_BUILT",
      artifacts: artifactList,
      reportJson,
      partReadbacksJson: JSON.stringify(selectedOutputs.map((output) => ({
        field: output.field,
        variant: output.variant,
        modelResref: output.modelResref,
        readback: output.readback,
      }))),
      utiReportJson,
    };
  }
  if (request.type === "BUILD_TILE_PACKAGE") {
    const result = buildMeshyStaticTilePackageV1(
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
        artifact(
          "tile-texture-tga",
          "TEXTURE",
          `${report.textureResref}.tga`,
          "image/x-tga",
          exactBuffer(result.takeTextureBytes()),
        ),
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
    && request.packageLane === "SKINNED_PROCEDURAL_HUMANOID_42"
  ) {
    const result = buildMeshyProceduralHumanoidProductWithOptionsV3(
      new Uint8Array(request.sourceGlb),
      new Uint8Array(request.appearanceTwoDa),
      request.identityJson,
      proceduralBuildOptionsJson(
        request.textureArtifactCleanup,
        request.skinAccessoryStabilization,
      ),
    );
    try {
      const summary = JSON.parse(result.summaryJson) as {
        identity?: { modelResref?: unknown; textureResref?: unknown; hakResref?: unknown };
      };
      const modelResref = summary.identity?.modelResref;
      const textureResref = summary.identity?.textureResref;
      const hakResref = summary.identity?.hakResref;
      if (
        typeof modelResref !== "string"
        || typeof textureResref !== "string"
        || typeof hakResref !== "string"
      ) {
        throw new Error("Procedural product summary has no exact resource identity");
      }
      const hak = exactBuffer(result.takeHakBytes());
      const model = exactBuffer(result.takeModelBytes());
      const texture = exactBuffer(result.takeTextureBytes());
      const report = encoder.encode(result.reportJson).buffer;
      const manifest = encoder.encode(result.manifestJson).buffer;
      const summaryBytes = encoder.encode(result.summaryJson).buffer;
      const artifacts = await Promise.all([
        artifact("package-hak", "HAK", `${hakResref}.hak`, "application/octet-stream", hak),
        artifact("model-mdl", "MODEL", `${modelResref}.mdl`, "application/octet-stream", model),
        artifact("texture-tga", "TEXTURE", `${textureResref}.tga`, "image/x-tga", texture),
        artifact("report-json", "JSON_REPORT", "inspection.json", "application/json", report),
        artifact("manifest-json", "JSON_REPORT", "conversion-manifest.json", "application/json", manifest),
        artifact("summary-json", "JSON_REPORT", "summary.json", "application/json", summaryBytes),
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

  if (
    request.type === "BUILD_MODEL_PACKAGE"
    && (
      request.packageLane === "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT"
      || request.packageLane === "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT"
    )
  ) {
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
            request.skinAccessoryStabilization,
          ),
        )
      : buildMeshyProceduralHumanoidP100kExperimentWithOptionsV2(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.appearanceTwoDa),
          request.identityJson,
          proceduralBuildOptionsJson(
            request.textureArtifactCleanup,
            request.skinAccessoryStabilization,
          ),
        );
    try {
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
      case "H1_SKINNED_FULL_42_EVENTS":
        return buildMeshyH1ModelPackageV3(
          new Uint8Array(request.sourceGlb),
          new Uint8Array(request.appearanceTwoDa),
          request.eventAuthoringJson,
        );
      case "H1_SKINNED_FULL_42":
        return buildMeshyH1ModelPackageV2(
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
