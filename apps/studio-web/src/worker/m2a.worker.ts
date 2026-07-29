/// <reference lib="webworker" />

import init, {
  buildM7CorpusBatchV1,
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
  inspectTwoDaV2Json,
  inspectM7CorpusIntakeV1Json,
  inspectMeshyStaticPlaceableAuthoringV1,
  validateM7CorpusManifestV1Json,
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

async function handle(request: StudioWorkerRequest): Promise<StudioWorkerResponse> {
  await ensureInitialized();
  if (request.type === "INITIALIZE") {
    return { requestId: request.requestId, ok: true, type: "INITIALIZED" };
  }
  if (request.type === "INSPECT_SOURCE") {
    const ingestJson = request.target === "PLACEABLE" || request.target === "TILE"
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
