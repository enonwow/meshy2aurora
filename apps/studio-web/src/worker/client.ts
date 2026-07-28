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
