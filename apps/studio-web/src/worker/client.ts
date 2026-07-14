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

  dispose() {
    this.worker.terminate();
    for (const pending of this.pending.values()) {
      pending.reject(new Error("Studio Worker disposed"));
    }
    this.pending.clear();
  }
}
