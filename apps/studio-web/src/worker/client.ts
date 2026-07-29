import type {
  StudioWorkerRequest,
  StudioWorkerResponse,
} from "./types";

type Pending = {
  resolve: (response: StudioWorkerResponse) => void;
  reject: (error: Error) => void;
  lane: "MAIN" | "PREVIEW";
};

export class StudioWorkerClient {
  private readonly pending = new Map<string, Pending>();
  private readonly worker: Worker;
  private previewWorker: Worker | null = null;
  private animationPreviewRequestId: string | null = null;

  constructor() {
    this.worker = this.createWorker("MAIN");
  }

  private createWorker(lane: Pending["lane"]) {
    const worker = new Worker(
      new URL("./m2a.worker.ts", import.meta.url),
      { type: "module" },
    );
    worker.addEventListener("message", (event: MessageEvent<StudioWorkerResponse>) => {
      const pending = this.pending.get(event.data.requestId);
      if (!pending || pending.lane !== lane) return;
      this.pending.delete(event.data.requestId);
      if (event.data.ok) pending.resolve(event.data);
      else pending.reject(new Error(event.data.message));
    });
    worker.addEventListener("error", (event) => {
      for (const [requestId, pending] of this.pending) {
        if (pending.lane !== lane) continue;
        pending.reject(new Error(event.message || "Studio Worker failed"));
        this.pending.delete(requestId);
      }
      if (lane === "PREVIEW" && this.previewWorker === worker) {
        this.previewWorker = null;
        this.animationPreviewRequestId = null;
      }
    });
    return worker;
  }

  request(
    request: StudioWorkerRequest,
    transfer: Transferable[] = [],
    lane: Pending["lane"] = "MAIN",
  ) {
    const worker = lane === "PREVIEW"
      ? (this.previewWorker ??= this.createWorker("PREVIEW"))
      : this.worker;
    return new Promise<StudioWorkerResponse>((resolve, reject) => {
      this.pending.set(request.requestId, { resolve, reject, lane });
      worker.postMessage(request, transfer);
    });
  }

  validateCreatureAnimationMapping(
    animationAuthoringJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "VALIDATE_CREATURE_ANIMATION_MAPPING",
      animationAuthoringJson,
    });
  }

  inspectEditableAnimationSource(
    sourceGlb: ArrayBuffer,
    clipName?: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "INSPECT_EDITABLE_ANIMATION_SOURCE",
      sourceGlb,
      clipName,
    }, [sourceGlb]);
  }

  validateAnimationStudioDocument(
    sourceGlb: ArrayBuffer,
    animationStudioDocumentJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "VALIDATE_ANIMATION_STUDIO_DOCUMENT",
      sourceGlb,
      animationStudioDocumentJson,
    }, [sourceGlb]);
  }

  materializeAnimationStudioDocument(
    sourceGlb: ArrayBuffer,
    animationStudioDocumentJson: string,
    requestId = workerRequestId(),
  ) {
    if (this.animationPreviewRequestId) {
      this.cancelAnimationStudioPreviewBuild();
    }
    this.animationPreviewRequestId = requestId;
    return this.request({
      requestId,
      type: "MATERIALIZE_ANIMATION_STUDIO_DOCUMENT",
      sourceGlb,
      animationStudioDocumentJson,
    }, [sourceGlb], "PREVIEW").finally(() => {
      if (this.animationPreviewRequestId === requestId) {
        this.animationPreviewRequestId = null;
      }
    });
  }

  previewAuthoredAnimationClip(
    animationStudioDocumentJson: string,
    clipId: string,
    requestId = workerRequestId(),
  ) {
    if (this.animationPreviewRequestId) {
      this.cancelAnimationStudioPreviewBuild();
    }
    this.animationPreviewRequestId = requestId;
    return this.request({
      requestId,
      type: "PREVIEW_AUTHORED_ANIMATION_CLIP",
      animationStudioDocumentJson,
      clipId,
    }, [], "PREVIEW").finally(() => {
      if (this.animationPreviewRequestId === requestId) {
        this.animationPreviewRequestId = null;
      }
    });
  }

  buildAuthoredCreatureModelPackage(
    sourceGlb: ArrayBuffer,
    appearanceTwoDa: ArrayBuffer,
    animationAuthoringJson: string,
    projectIdentityJson: string,
    eventAuthoringJson?: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "BUILD_MODEL_PACKAGE",
      sourceGlb,
      appearanceTwoDa,
      packageLane: "H1_SKINNED_FULL_42_AUTHORED",
      animationAuthoringJson,
      projectIdentityJson,
      eventAuthoringJson,
    }, [sourceGlb, appearanceTwoDa]);
  }

  buildEditedCreatureModelPackage(
    sourceGlb: ArrayBuffer,
    appearanceTwoDa: ArrayBuffer,
    animationAuthoringJson: string,
    animationStudioDocumentJson: string,
    projectIdentityJson: string,
    eventAuthoringJson?: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "BUILD_MODEL_PACKAGE",
      sourceGlb,
      appearanceTwoDa,
      packageLane: "H1_SKINNED_FULL_42_EDITED",
      animationAuthoringJson,
      animationStudioDocumentJson,
      projectIdentityJson,
      eventAuthoringJson,
    }, [sourceGlb, appearanceTwoDa]);
  }

  cancelAnimationStudioPreviewBuild() {
    const worker = this.previewWorker;
    const pendingPreviewIds = [...this.pending.entries()]
      .filter(([, pending]) => pending.lane === "PREVIEW")
      .map(([requestId]) => requestId);
    if (pendingPreviewIds.length === 0) return false;
    worker?.terminate();
    if (this.previewWorker === worker) this.previewWorker = null;
    this.animationPreviewRequestId = null;
    for (const requestId of pendingPreviewIds) {
      const pending = this.pending.get(requestId);
      this.pending.delete(requestId);
      pending?.reject(new DOMException(
        "Animation preview Worker was terminated",
        "AbortError",
      ));
    }
    return true;
  }

  cancel(requestId: string) {
    const pending = this.pending.get(requestId);
    if (!pending) return false;
    if (pending.lane === "PREVIEW") {
      return this.cancelAnimationStudioPreviewBuild();
    }
    this.pending.delete(requestId);
    pending.reject(new DOMException("Studio Worker request cancelled", "AbortError"));
    return true;
  }

  dispose() {
    this.worker.terminate();
    this.previewWorker?.terminate();
    this.previewWorker = null;
    this.animationPreviewRequestId = null;
    for (const pending of this.pending.values()) {
      pending.reject(new Error("Studio Worker disposed"));
    }
    this.pending.clear();
  }
}

function workerRequestId() {
  return typeof crypto.randomUUID === "function"
    ? crypto.randomUUID()
    : `worker-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}
