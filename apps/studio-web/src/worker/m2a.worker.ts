/// <reference lib="webworker" />

import init, {
  buildItemEquippedProofModuleV2,
  buildM7CorpusBatchV1,
  buildItemProofModuleV1,
  buildMeshyItemPartWithOptionsV2,
  buildMeshyH1ModelPackageV2,
  buildMeshyH1ModelPackageV3,
  buildMeshyProceduralHumanoidProductWithOptionsV3,
  buildMeshyProceduralHumanoidP100kExperimentWithOptionsV2,
  buildMeshyProceduralHumanoidP300kExperimentWithOptionsV2,
  buildMeshyM0StaticRigidPackageV1,
  buildMeshyStaticPlaceablePackageV1,
  buildMeshyStaticPlaceablePackageV2,
  buildMeshyStaticTilePackageV1,
  ingestGlbJson,
  ingestMeshyP100kExperimentJson,
  ingestMeshyP300kExperimentJson,
  ingestStaticRigidGlbJson,
  inspectItemBaseitemsV1Json,
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
    type Catalog = {
      rows: Array<{
        baseItem: number;
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
      modelResref: string;
      iconResref: string;
      textureResref: string;
      mdl: ArrayBuffer;
      texture: ArrayBuffer;
      icon: ArrayBuffer;
      report: Record<string, unknown> & {
        triangleCount: number;
        textureFormat: string;
        mdlSha256: string;
        textureSha256: string;
        iconProjectionBounds?: IconProjectionBounds;
        iconOpaquePixelCount?: number;
      };
      readback: unknown;
    };
    const catalog = JSON.parse(
      inspectItemBaseitemsV1Json(new Uint8Array(request.baseitemsTwoDa)),
    ) as Catalog;
    const selected = catalog.rows.find((row) => row.baseItem === request.baseItem);
    if (!selected) throw new Error(`ITEM-BASEITEM-NOT-FOUND: ${request.baseItem}`);
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
    }
    const orderedParts = selected.partSlots.map((slot) => request.parts.find(
      (part) => part.field.toLowerCase() === slot.field.toLowerCase(),
    )!);
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
      const resolved = JSON.parse(resolveItemPartResourceV1Json(
        new Uint8Array(request.baseitemsTwoDa),
        request.baseItem,
        part.field,
        part.variant,
        part.modelResref,
        part.iconResref,
      )) as { modelResref: string; iconResref: string };
      if (
        resolved.modelResref !== part.modelResref
        || resolved.iconResref !== part.iconResref
      ) {
        throw new Error(
          `ITEM-RESOURCE-NAME-MISMATCH: ${part.field} must resolve to ${resolved.modelResref} / ${resolved.iconResref}`,
        );
      }
    }

    const outputs: PartOutput[] = [];
    const emitsCustomIcons = selected.capability.iconProfile === "STANDARD"
      || selected.capability.iconProfile === "LAYERED";
    const meshyParts = orderedParts.filter(
      (candidate) => candidate.sourceKind === "MESHY_GLB",
    );
    const partOptionsJson = (part: typeof meshyParts[number]) => JSON.stringify({
      schemaVersion: 1,
      transform: JSON.parse(part.transformJson),
      sourceNode: part.sourceNode,
      textureEncoding: part.textureEncoding,
      iconSize: null,
      iconProjectionBounds: null,
    });
    const seamResults: Array<{
      firstField: string;
      secondField: string;
      status: "TOUCHING" | "GAP" | "OVERLAP";
      gap: number;
      overlap: boolean;
      requiredRelation: "ADJACENT_TOUCH" | "NON_ADJACENT_NO_OVERLAP";
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
          ? "ADJACENT_TOUCH" as const
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
      (result) => result.overlap
        || (
          result.requiredRelation === "ADJACENT_TOUCH"
          && (
            result.status !== "TOUCHING"
            || result.gap > request.seamValidation.tolerance
          )
        ),
    );
    if (failedSeams.length > 0) {
      throw new Error(
        `ITEM-SEAM-VALIDATION-FAILED: ${failedSeams.map((result) => (
          `${result.firstField}/${result.secondField}=${result.status}`
        )).join(", ")}`,
      );
    }
    for (const part of meshyParts) {
      const result = buildMeshyItemPartWithOptionsV2(
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
      const projectionBounds = outputs.reduce<IconProjectionBounds | undefined>(
        (combined, output) => {
          const bounds = output.report.iconProjectionBounds;
          if (!bounds) {
            throw new Error(
              `ITEM-ICON-PROJECTION-MISSING: ${output.field} emitted no geometry projection bounds`,
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
      for (let index = 0; index < meshyParts.length; index += 1) {
        const part = meshyParts[index];
        const output = outputs[index];
        const result = buildMeshyItemPartWithOptionsV2(
          new Uint8Array(part.sourceGlb!),
          part.modelResref,
          part.textureResref,
          JSON.stringify({
            schemaVersion: 1,
            transform: JSON.parse(part.transformJson),
            sourceNode: part.sourceNode,
            textureEncoding: part.textureEncoding,
            iconSize: [width, height],
            iconProjectionBounds: projectionBounds,
          }),
        );
        try {
          const report = JSON.parse(result.reportJson) as PartOutput["report"];
          const icon = exactBuffer(result.takeIconBytes());
          if (
            icon.byteLength === 0
            || !report.iconOpaquePixelCount
            || report.mdlSha256 !== output.report.mdlSha256
            || report.textureSha256 !== output.report.textureSha256
          ) {
            throw new Error(
              `ITEM-ICON-LAYER-MISMATCH: ${part.field} failed deterministic shared-frame rasterization`,
            );
          }
          result.takeMdlBytes();
          result.takeTextureBytes();
          output.icon = icon;
          output.report = report;
          output.readback = JSON.parse(result.readbackJson);
        } finally {
          result.free();
        }
      }
    }
    const utiResult = writeItemUtiV1(
      new Uint8Array(request.baseitemsTwoDa),
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
        "ITEM-EQUIPPED-PROOF-CONTEXT-MISSING: CAPART and Cloak require a creature resref and Appearance row",
      );
    }
    if (!equippedProofProfile && request.equippedProofContext) {
      throw new Error(
        "ITEM-EQUIPPED-PROOF-CONTEXT-UNEXPECTED: equipped proof context is valid only for CAPART and Cloak",
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
      ...outputs.map((output) => output.report.triangleCount),
      ...referenceTriangleCounts,
    ]))) as {
      triangleCount: number;
      triangleBudget: number;
      warning: boolean;
      warningAbove: number;
    };

    const resources = outputs.flatMap((output) => [
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
    const resourceKeys = new Set(
      [...occupiedResourceKeys].filter((key) => /^\d+:/.test(key)),
    );
    for (const resource of resources) {
      const key = `${resource.resourceType}:${resource.resref.toLowerCase()}`;
      if (resourceKeys.has(key)) {
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
    const report = {
      schemaVersion: 1,
      status: "OFFLINE_ITEM_PACKAGE_PASSED",
      profile: "ITEM",
      baseItem: request.baseItem,
      partCount: request.parts.length,
      meshyPartCount: outputs.length,
      referenceSelectorCount: request.parts.length - outputs.length,
      iconLayerCount: emitsCustomIcons ? outputs.length : 0,
      iconLayerMode: selected.capability.iconProfile === "IPRP_SPELL"
        ? "IPRP_SPELLS_ICON_RESOLUTION"
        : selected.capability.iconProfile === "CAPART_COMPOSITE"
          ? "RETAIL_CAPART_COMPOSITION_PLAN_V1"
          : selected.capability.iconProfile === "CLOAK_MODEL"
            ? "RETAIL_CLOAKMODEL_ICON"
            : "GEOMETRY_RASTER_TGA_V2",
      iconRuntimeParity: "offline_semantic_readback_only",
      triangleBudget: budget,
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
      partReadbacksJson: JSON.stringify(outputs.map((output) => ({
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
