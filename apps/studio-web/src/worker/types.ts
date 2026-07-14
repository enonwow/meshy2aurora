export type StudioWorkerRequest =
  | { requestId: string; type: "INITIALIZE" }
  | { requestId: string; type: "INSPECT_SOURCE"; sourceGlb: ArrayBuffer }
  | { requestId: string; type: "INSPECT_APPEARANCE"; appearanceTwoDa: ArrayBuffer }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
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
  kind: "HAK" | "MODEL" | "JSON_REPORT";
  fileName: string;
  mediaType: string;
  byteLength: number;
  sha256: string;
  bytes: ArrayBuffer;
  provenance: "M2A_WASM_WORKER";
}

export type StudioWorkerSuccess =
  | { requestId: string; ok: true; type: "INITIALIZED" }
  | { requestId: string; ok: true; type: "SOURCE_INSPECTED"; ingestJson: string }
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
