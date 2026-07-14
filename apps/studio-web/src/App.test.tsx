// @vitest-environment jsdom

import { StrictMode, act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import type {
  StudioWorkerRequest,
  StudioWorkerResponse,
  WorkerArtifact,
} from "./worker/types";

type WorkerEventListener = (event: MessageEvent<StudioWorkerResponse>) => void;

class FakeWorker {
  static instances: FakeWorker[] = [];

  readonly requests: StudioWorkerRequest[] = [];
  readonly listeners = new Set<WorkerEventListener>();
  terminated = false;

  constructor() {
    FakeWorker.instances.push(this);
  }

  addEventListener(type: string, listener: EventListenerOrEventListenerObject) {
    if (type === "message") this.listeners.add(listener as WorkerEventListener);
  }

  postMessage(request: StudioWorkerRequest) {
    this.requests.push(request);
  }

  terminate() {
    this.terminated = true;
  }

  emit(response: StudioWorkerResponse) {
    this.listeners.forEach((listener) => listener({ data: response } as MessageEvent<StudioWorkerResponse>));
  }
}

const roots: Root[] = [];

async function renderApp(strict = false) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  await act(async () => {
    root.render(strict ? <StrictMode><App /></StrictMode> : <App />);
  });
  return container;
}

function localFile(name: string, marker: number): File {
  return {
    name,
    size: 1,
    arrayBuffer: async () => new Uint8Array([marker]).buffer,
  } as File;
}

async function selectFile(input: HTMLInputElement, file: File) {
  Object.defineProperty(input, "files", { configurable: true, value: [file] });
  await act(async () => {
    input.dispatchEvent(new window.Event("change", { bubbles: true }));
  });
}

describe("Studio session lifecycle", () => {
  beforeEach(() => {
    FakeWorker.instances = [];
    vi.stubGlobal("Worker", FakeWorker);
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
  });

  afterEach(async () => {
    while (roots.length) {
      const root = roots.pop();
      if (root) await act(async () => root.unmount());
    }
    document.body.replaceChildren();
    vi.unstubAllGlobals();
  });

  it("recreates the Worker after the StrictMode cleanup cycle", async () => {
    await renderApp(true);

    expect(FakeWorker.instances).toHaveLength(2);
    expect(FakeWorker.instances[0].terminated).toBe(true);
    expect(FakeWorker.instances[1].terminated).toBe(false);
  });

  it.each([
    ["source", 0, localFile("second.glb", 3)],
    ["appearance", 1, localFile("second.2da", 4)],
  ] as const)("ignores an old build response after the %s input changes", async (_label, changedIndex, replacement) => {
    const container = await renderApp();
    const [sourceInput, appearanceInput] = Array.from(
      container.querySelectorAll<HTMLInputElement>('input[type="file"]'),
    );
    await selectFile(sourceInput, localFile("first.glb", 1));
    await selectFile(appearanceInput, localFile("appearance.2da", 2));

    const button = container.querySelector<HTMLButtonElement>("button");
    expect(button?.disabled).toBe(false);
    await act(async () => {
      button?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });

    const worker = FakeWorker.instances[0];
    const buildRequest = worker.requests.find((request) => request.type === "BUILD_MODEL_PACKAGE");
    expect(buildRequest).toBeDefined();

    await selectFile([sourceInput, appearanceInput][changedIndex], replacement);
    const staleArtifact: WorkerArtifact = {
      artifactId: "stale",
      kind: "HAK",
      fileName: "stale.hak",
      mediaType: "application/octet-stream",
      byteLength: 1,
      sha256: "0".repeat(64),
      bytes: new Uint8Array([9]).buffer,
      provenance: "M2A_WASM_WORKER",
    };
    await act(async () => {
      worker.emit({
        requestId: buildRequest!.requestId,
        ok: true,
        type: "MODEL_PACKAGE_BUILT",
        artifacts: [staleArtifact],
        reportJson: "{}",
        manifestJson: "{}",
        summaryJson: "{}",
        readbackJson: "not-json-because-this-response-must-be-ignored",
      });
      await Promise.resolve();
    });

    expect(container.textContent).not.toContain("stale.hak");
    expect(container.querySelector(".status strong")?.textContent).toBe("READY");
  });
});
