import type {
  StudioWorkerRequest,
  StudioWorkerResponse,
} from "./types";

type Pending = {
  resolve: (response: StudioWorkerResponse) => void;
  reject: (error: Error) => void;
};

export class StudioWorkerClient {
  private readonly worker = new Worker(
    new URL("./m2a.worker.ts", import.meta.url),
    { type: "module" },
  );
  private readonly pending = new Map<string, Pending>();
  private animationPreviewRequestId: string | null = null;

  constructor() {
    this.worker.addEventListener("message", (event: MessageEvent<StudioWorkerResponse>) => {
      const pending = this.pending.get(event.data.requestId);
      if (!pending) return;
      this.pending.delete(event.data.requestId);
      if (event.data.ok) pending.resolve(event.data);
      else pending.reject(new Error(event.data.message));
    });
    this.worker.addEventListener("error", (event) => {
      for (const pending of this.pending.values()) {
        pending.reject(new Error(event.message || "Studio Worker failed"));
      }
      this.pending.clear();
    });
  }

  request(request: StudioWorkerRequest, transfer: Transferable[] = []) {
    return new Promise<StudioWorkerResponse>((resolve, reject) => {
      this.pending.set(request.requestId, { resolve, reject });
      this.worker.postMessage(request, transfer);
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
    if (this.animationPreviewRequestId) this.cancel(this.animationPreviewRequestId);
    this.animationPreviewRequestId = requestId;
    return this.request({
      requestId,
      type: "MATERIALIZE_ANIMATION_STUDIO_DOCUMENT",
      sourceGlb,
      animationStudioDocumentJson,
    }, [sourceGlb]).finally(() => {
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
    if (this.animationPreviewRequestId) this.cancel(this.animationPreviewRequestId);
    this.animationPreviewRequestId = requestId;
    return this.request({
      requestId,
      type: "PREVIEW_AUTHORED_ANIMATION_CLIP",
      animationStudioDocumentJson,
      clipId,
    }).finally(() => {
      if (this.animationPreviewRequestId === requestId) {
        this.animationPreviewRequestId = null;
      }
    });
  }

  buildAuthoredCreatureModelPackage(
    sourceGlb: ArrayBuffer,
    appearanceTwoDa: ArrayBuffer,
    animationAuthoringJson: string,
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
      eventAuthoringJson,
    }, [sourceGlb, appearanceTwoDa]);
  }

  buildEditedCreatureModelPackage(
    sourceGlb: ArrayBuffer,
    appearanceTwoDa: ArrayBuffer,
    animationAuthoringJson: string,
    animationStudioDocumentJson: string,
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
      eventAuthoringJson,
    }, [sourceGlb, appearanceTwoDa]);
  }

  cancelAnimationStudioPreviewBuild() {
    if (!this.animationPreviewRequestId) return false;
    return this.cancel(this.animationPreviewRequestId);
  }

  cancel(requestId: string) {
    const pending = this.pending.get(requestId);
    if (!pending) return false;
    this.pending.delete(requestId);
    pending.reject(new DOMException("Studio Worker request cancelled", "AbortError"));
    return true;
  }

  dispose() {
    this.worker.terminate();
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
