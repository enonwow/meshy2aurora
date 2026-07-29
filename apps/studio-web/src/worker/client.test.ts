// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { StudioWorkerClient } from "./client";
import type { StudioWorkerRequest, StudioWorkerResponse } from "./types";

type MessageListener = (event: MessageEvent<StudioWorkerResponse>) => void;
type ErrorListener = (event: ErrorEvent) => void;

class FakeWorker {
  static instances: FakeWorker[] = [];
  readonly requests: StudioWorkerRequest[] = [];
  readonly messageListeners = new Set<MessageListener>();
  readonly errorListeners = new Set<ErrorListener>();
  terminated = false;

  constructor() {
    FakeWorker.instances.push(this);
  }

  addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
  ) {
    if (type === "message") {
      this.messageListeners.add(listener as MessageListener);
    } else if (type === "error") {
      this.errorListeners.add(listener as ErrorListener);
    }
  }

  postMessage(request: StudioWorkerRequest) {
    this.requests.push(request);
  }

  terminate() {
    this.terminated = true;
  }

  emit(response: StudioWorkerResponse) {
    this.messageListeners.forEach((listener) => listener({
      data: response,
    } as MessageEvent<StudioWorkerResponse>));
  }
}

beforeEach(() => {
  FakeWorker.instances = [];
  vi.stubGlobal("Worker", FakeWorker);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("StudioWorkerClient preview cancellation", () => {
  it("runs preview on a dedicated Worker and terminates stale computation", async () => {
    const client = new StudioWorkerClient();
    const mainWorker = FakeWorker.instances[0]!;
    const first = client.previewAuthoredAnimationClip(
      "{}",
      "clip-first",
      "preview-first",
    );
    const firstRejected = expect(first).rejects.toMatchObject({
      name: "AbortError",
    });
    const firstPreviewWorker = FakeWorker.instances[1]!;

    const second = client.previewAuthoredAnimationClip(
      "{}",
      "clip-second",
      "preview-second",
    );
    const secondPreviewWorker = FakeWorker.instances[2]!;

    await firstRejected;
    expect(firstPreviewWorker.terminated).toBe(true);
    expect(mainWorker.terminated).toBe(false);
    expect(secondPreviewWorker.requests).toEqual([
      expect.objectContaining({
        requestId: "preview-second",
        type: "PREVIEW_AUTHORED_ANIMATION_CLIP",
        clipId: "clip-second",
      }),
    ]);

    secondPreviewWorker.emit({
      requestId: "preview-second",
      ok: true,
      type: "AUTHORED_ANIMATION_CLIP_PREVIEWED",
      previewJson: "{}",
    });
    await expect(second).resolves.toMatchObject({
      requestId: "preview-second",
      ok: true,
    });
    expect(client.cancelAnimationStudioPreviewBuild()).toBe(false);
    client.dispose();
    expect(mainWorker.terminated).toBe(true);
    expect(secondPreviewWorker.terminated).toBe(true);
  });

  it("reports cancellation only when it really terminates the preview Worker", async () => {
    const client = new StudioWorkerClient();
    const preview = client.materializeAnimationStudioDocument(
      new Uint8Array([1, 2, 3]).buffer,
      "{}",
      "materialize-preview",
    );
    const rejected = expect(preview).rejects.toMatchObject({
      name: "AbortError",
    });
    const previewWorker = FakeWorker.instances[1]!;

    expect(client.cancelAnimationStudioPreviewBuild()).toBe(true);
    expect(previewWorker.terminated).toBe(true);
    await rejected;
    expect(client.cancelAnimationStudioPreviewBuild()).toBe(false);
    client.dispose();
  });
});
