// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import type { StudioWorkerRequest, StudioWorkerResponse } from "./worker/types";

const projectionSpy = vi.hoisted(() => vi.fn());

vi.mock("./features/results/projectCanonicalResult", async (importOriginal) => {
  const actual = await importOriginal<typeof import("./features/results/projectCanonicalResult")>();
  return {
    ...actual,
    projectCanonicalResult: (...args: Parameters<typeof actual.projectCanonicalResult>) => {
      projectionSpy();
      return actual.projectCanonicalResult(...args);
    },
  };
});

vi.mock("./features/preview/SceneViewport", () => ({
  SceneViewport: ({ provenance }: { provenance: string }) => (
    <div data-testid={`viewport-${provenance}`} />
  ),
}));

type WorkerMessageListener = (event: MessageEvent<StudioWorkerResponse>) => void;

class FakeWorker {
  static instances: FakeWorker[] = [];

  readonly requests: StudioWorkerRequest[] = [];
  readonly listeners = new Set<WorkerMessageListener>();
  terminated = false;

  constructor() {
    FakeWorker.instances.push(this);
  }

  addEventListener(type: string, listener: EventListenerOrEventListenerObject) {
    if (type === "message") this.listeners.add(listener as WorkerMessageListener);
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

function localFile(name: string, marker: number): File {
  return {
    name,
    size: 1,
    type: name.endsWith(".glb") ? "model/gltf-binary" : "text/plain",
    lastModified: marker,
    arrayBuffer: async () => new Uint8Array([marker]).buffer,
  } as File;
}

async function settle() {
  await act(async () => {
    await new Promise((resolve) => window.setTimeout(resolve, 0));
  });
}

async function selectFile(input: HTMLInputElement, file: File) {
  Object.defineProperty(input, "files", { configurable: true, value: [file] });
  await act(async () => {
    input.dispatchEvent(new window.Event("change", { bubbles: true }));
  });
  await settle();
}

function button(container: HTMLElement, label: string) {
  return Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
    .find((candidate) => candidate.textContent?.trim() === label);
}

function sourceInspectionJson() {
  const sha256 = "a".repeat(64);
  return JSON.stringify({
    schemaVersion: 1,
    ir: {
      schemaVersion: 1,
      source: {
        format: "GLB_2_0",
        byteLength: 1,
        sha256,
        assetVersion: "2.0",
        generator: null,
      },
      scenes: [],
      nodes: [],
      meshes: [],
      primitives: [],
      materials: [],
      textures: [],
      samplers: [],
      images: [],
      skins: [],
      animations: [],
    },
    report: {
      schemaVersion: 1,
      format: "GLB_2_0",
      input: { byteLength: 1, sha256 },
      inventory: {
        sceneCount: 0,
        nodeCount: 0,
        meshCount: 0,
        primitiveCount: 0,
        materialCount: 0,
        textureCount: 0,
        samplerCount: 0,
        imageCount: 0,
        skinCount: 0,
        jointReferenceCount: 0,
        animationCount: 0,
        keyframeCount: 0,
      },
      statistics: {
        vertexCount: 0,
        indexCount: 0,
        triangleCount: 0,
        boundsMin: null,
        boundsMax: null,
        primitivesMissingNormals: 0,
        primitivesMissingUv0: 0,
        nonTrianglePrimitives: 0,
      },
      gates: [],
      diagnostics: [],
      conversionEligible: true,
    },
  });
}

function appearanceInspectionJson() {
  return JSON.stringify({
    schemaVersion: 1,
    format: "2DA",
    version: "V2.0",
    sourceSha256: "b".repeat(64),
    byteLength: 1,
    newline: "LF",
    terminalNewline: true,
    defaultValue: null,
    columns: ["LABEL"],
    physicalRowCount: 0,
    nextAppendIndex: 0,
    rowLabelMismatchCount: 0,
    diagnostics: [],
  });
}

async function renderRunningBuild() {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  await act(async () => root.render(<App />));

  const [sourceInput, appearanceInput] = Array.from(
    container.querySelectorAll<HTMLInputElement>('input[type="file"]'),
  );
  await selectFile(sourceInput, localFile("source.glb", 1));
  await selectFile(appearanceInput, localFile("appearance.2da", 2));

  const worker = FakeWorker.instances[0];
  const sourceRequest = worker.requests.filter((request) => request.type === "INSPECT_SOURCE").at(-1);
  const appearanceRequest = worker.requests.filter((request) => request.type === "INSPECT_APPEARANCE").at(-1);
  if (!sourceRequest || !appearanceRequest) throw new Error("inspection requests missing");

  await act(async () => {
    worker.emit({
      requestId: sourceRequest.requestId,
      ok: true,
      type: "SOURCE_INSPECTED",
      ingestJson: sourceInspectionJson(),
    });
    worker.emit({
      requestId: appearanceRequest.requestId,
      ok: true,
      type: "APPEARANCE_INSPECTED",
      inspectionJson: appearanceInspectionJson(),
    });
    await Promise.resolve();
  });

  await act(async () => button(container, "Continue to Inspect")?.click());
  await act(async () => button(container, "Continue to Build")?.click());
  await act(async () => button(container, "Build Package")?.click());
  await settle();

  const buildRequest = worker.requests.find((request) => request.type === "BUILD_MODEL_PACKAGE");
  if (!buildRequest) throw new Error("build request missing");
  return { buildRequest, container, worker };
}

describe("running Build invalidation", () => {
  beforeEach(() => {
    FakeWorker.instances = [];
    projectionSpy.mockClear();
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

  it("keeps Build and Cancel visible when stepper navigation is attempted", async () => {
    const { container, worker } = await renderRunningBuild();

    await act(async () => {
      container.querySelector<HTMLButtonElement>('[aria-label^="Source:"]')?.click();
    });

    expect(container.querySelector("#build-step-heading")?.textContent).toBe("Build");
    expect(button(container, "Cancel Build")).toBeDefined();
    expect(worker.terminated).toBe(false);
  });

  it("projects structured Worker failure evidence into the exact failed ledger stage", async () => {
    const { buildRequest, container, worker } = await renderRunningBuild();

    await act(async () => {
      worker.emit({
        requestId: buildRequest.requestId,
        ok: false,
        type: "FAILED",
        message: JSON.stringify({
          schemaVersion: 1,
          stage: "MODEL",
          code: "M4-WRITER-FAILED",
          path: "model.nodes[2]",
          message: "binary writer rejected node 2",
        }),
      });
      await Promise.resolve();
    });

    expect(container.querySelector('[data-status="FAILED"]')?.textContent)
      .toContain("Write binary MDL");
    expect(container.textContent).toContain("M4-WRITER-FAILED");
    expect(container.textContent).toContain("MODEL");
    expect(container.textContent).toContain("model.nodes[2]");
    expect(container.textContent).toContain("binary writer rejected node 2");
    expect(projectionSpy).not.toHaveBeenCalled();
  });

  it.each(["replace", "remove", "new-conversion"] as const)(
    "replaces the Worker immediately on %s and skips stale projections",
    async (action) => {
      const { buildRequest, container, worker } = await renderRunningBuild();
      const sourceInput = container.querySelector<HTMLInputElement>('.inputs-panel input[accept^=".glb"]');
      if (!sourceInput) throw new Error("source input missing");

      if (action === "replace") {
        await selectFile(sourceInput, localFile("replacement.glb", 3));
      } else if (action === "remove") {
        await act(async () => {
          container.querySelector<HTMLButtonElement>('[aria-label="Remove Meshy GLB model"]')?.click();
        });
      } else {
        await act(async () => button(container, "Clear")?.click());
      }

      expect(worker.terminated).toBe(true);
      expect(FakeWorker.instances).toHaveLength(2);
      expect(FakeWorker.instances[1].terminated).toBe(false);

      await act(async () => {
        worker.emit({
          requestId: buildRequest.requestId,
          ok: true,
          type: "MODEL_PACKAGE_BUILT",
          artifacts: [],
          reportJson: "stale-report",
          manifestJson: "stale-manifest",
          summaryJson: "stale-summary",
          readbackJson: "stale-readback",
        });
        await Promise.resolve();
      });

      expect(projectionSpy).not.toHaveBeenCalled();
      expect(container.querySelector("#review-model-heading")).toBeNull();
      expect(container.querySelector("#build-step-heading")).toBeNull();
    },
  );
});
