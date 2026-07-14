/// <reference lib="webworker" />

import init, {
  buildM7CorpusBatchV1,
  buildM6ModelPackageV1,
  ingestGlbJson,
  inspectM7CorpusIntakeV1Json,
  validateM7CorpusManifestV1Json,
} from "@m2a-wasm";
import type {
  StudioWorkerRequest,
  StudioWorkerResponse,
  WorkerArtifact,
} from "./types";

let initialized: Promise<unknown> | undefined;
const ensureInitialized = () => (initialized ??= init());
const encoder = new TextEncoder();

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
    return {
      requestId: request.requestId,
      ok: true,
      type: "SOURCE_INSPECTED",
      ingestJson: ingestGlbJson(new Uint8Array(request.sourceGlb)),
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

  const result = buildM6ModelPackageV1(
    new Uint8Array(request.sourceGlb),
    new Uint8Array(request.appearanceTwoDa),
  );
  try {
    const hak = exactBuffer(result.takeHakBytes());
    const model = exactBuffer(result.takeModelBytes());
    const report = encoder.encode(result.reportJson).buffer;
    const manifest = encoder.encode(result.manifestJson).buffer;
    const summary = encoder.encode(result.summaryJson).buffer;
    const artifacts = await Promise.all([
      artifact("package-hak", "HAK", "meshy2aurora.hak", "application/octet-stream", hak),
      artifact("model-mdl", "MODEL", "meshy2aurora.mdl", "application/octet-stream", model),
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
