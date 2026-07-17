// @vitest-environment jsdom

import { StrictMode, act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { InMemoryMeshyBridgeClient, type MeshyBridgeClient } from "./features/meshy/bridge";
import type { StudioWorkerRequest, StudioWorkerResponse, WorkerArtifact } from "./worker/types";

vi.mock("./features/preview/SceneViewport", () => ({
  SceneViewport: ({ provenance }: { provenance: string }) => <div data-testid={`viewport-${provenance}`} />,
}));

type WorkerListener = (event: MessageEvent<StudioWorkerResponse>) => void;

class FakeWorker {
  static instances: FakeWorker[] = [];
  readonly requests: StudioWorkerRequest[] = [];
  readonly listeners = new Set<WorkerListener>();
  terminated = false;

  constructor() { FakeWorker.instances.push(this); }
  addEventListener(type: string, listener: EventListenerOrEventListenerObject) {
    if (type === "message") this.listeners.add(listener as WorkerListener);
  }
  postMessage(request: StudioWorkerRequest) { this.requests.push(request); }
  terminate() { this.terminated = true; }
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

function sourceInspectionJson() {
  const sha256 = "a".repeat(64);
  return JSON.stringify({
    schemaVersion: 1,
    ir: {
      schemaVersion: 1,
      source: { format: "GLB_2_0", byteLength: 1, sha256, assetVersion: "2.0", generator: null },
      scenes: [], nodes: [], meshes: [], primitives: [], materials: [], textures: [], samplers: [], images: [], skins: [], animations: [],
    },
    report: {
      schemaVersion: 1, format: "GLB_2_0", input: { byteLength: 1, sha256 },
      inventory: { sceneCount: 0, nodeCount: 0, meshCount: 0, primitiveCount: 0, materialCount: 0, textureCount: 0, samplerCount: 0, imageCount: 0, skinCount: 0, jointReferenceCount: 0, animationCount: 0, keyframeCount: 0 },
      statistics: { vertexCount: 0, indexCount: 0, triangleCount: 0, boundsMin: null, boundsMax: null, primitivesMissingNormals: 0, primitivesMissingUv0: 0, nonTrianglePrimitives: 0 },
      gates: [], diagnostics: [], conversionEligible: true,
    },
  });
}

function appearanceInspectionJson() {
  return JSON.stringify({
    schemaVersion: 1, format: "2DA", version: "V2.0", sourceSha256: "b".repeat(64), byteLength: 1,
    newline: "LF", terminalNewline: true, defaultValue: null, columns: ["LABEL"], physicalRowCount: 0, nextAppendIndex: 0, rowLabelMismatchCount: 0, diagnostics: [],
  });
}

function artifact(id: string, kind: WorkerArtifact["kind"], bytes: number[], sha256: string): WorkerArtifact {
  return {
    artifactId: id, kind, fileName: `${id}.bin`, mediaType: kind === "JSON_REPORT" ? "application/json" : "application/octet-stream",
    byteLength: bytes.length, sha256, bytes: new Uint8Array(bytes).buffer, provenance: "M2A_WASM_WORKER",
  };
}

function builtResponse(requestId: string, format = "nwn1-binary-mdl"): StudioWorkerResponse {
  const reportJson = JSON.stringify({
    schemaVersion: 1,
    geometry: { vertexCount: 24, triangleCount: 12, activeJointCount: 2, outputSegmentDeformation: "SKIN" },
    ingest: { schemaVersion: 1, inventory: { nodeCount: 3, meshCount: 1, jointReferenceCount: 2, animationCount: 1 }, statistics: { vertexCount: 24, triangleCount: 12 } },
    conversion: { schemaVersion: 1, conversionEligible: true, policies: { engineFacingProof: "OPEN_M6", uvRuntimeProof: "OPEN_M6" }, gates: [], diagnostics: [] },
    model: { payloadSha256: "b".repeat(64), layout: { fileLength: 2 }, projection: { modelResourceResref: "m2a_model", animationCount: 1, rigNodeCount: 2, meshNodeCount: 1, triangleCount: 12 }, semanticDiff: [], deviations: [] },
    texture: { width: 2, height: 2, pixelFormat: "RGBA8", byteLength: 60, outputSha256: "d".repeat(64) },
    appearance: { appendedRowIndex: 1, sourcePrefixPreserved: true, outputByteLength: 7, outputSha256: "e".repeat(64) },
    hak: { byteLength: 3, archiveSha256: "a".repeat(64), entryCount: 3 },
    proofModule: { byteLength: 4, sha256: "7".repeat(64), appearanceRow: 1, semanticReadbackStatus: "PASS" },
  });
  const summaryJson = JSON.stringify({ schemaVersion: 1, status: "M6_MODEL_PACKAGE_MATERIALIZED", outputs: { model: { byteLength: 2, sha256: "b".repeat(64) }, texture: { byteLength: 60, sha256: "d".repeat(64) }, appearanceTwoDa: { byteLength: 7, sha256: "e".repeat(64) }, hak: { byteLength: 3, sha256: "a".repeat(64) }, proofModule: { byteLength: 4, sha256: "7".repeat(64) }, report: { byteLength: new TextEncoder().encode(reportJson).byteLength, sha256: "c".repeat(64) } }, appendedPhysicalRow: 1, modelResref: "m2a_model", textureResref: "m2a_texture", animation: { sourceName: "walk", outputName: "cwalk", durationSeconds: 1.25, hasMotion: true }, appearancePayloadPolicy: "PRESERVED_AND_APPENDED" });
  const manifestJson = JSON.stringify({ schemaVersion: 1, status: "M6_MODEL_PACKAGE_MATERIALIZED", appendedPhysicalRow: 1, appearancePayloadPolicy: "PRESERVED_AND_APPENDED", packageManifest: { packageSha256: "a".repeat(64), resources: [{ role: "APPEARANCE_TABLE", resref: "appearance", type: 2017, byteLength: 7, sha256: "e".repeat(64) }, { role: "MODEL", resref: "m2a_model", type: 2002, byteLength: 2, sha256: "b".repeat(64) }, { role: "TEXTURE", resref: "m2a_texture", type: 3, byteLength: 60, sha256: "d".repeat(64) }] } });
  return {
    requestId, ok: true, type: "MODEL_PACKAGE_BUILT", reportJson,
    summaryJson,
    manifestJson,
    readbackJson: JSON.stringify({ schemaVersion: 1, format, nodeTree: { roots: [{ offset: 12, number: 1, name: "root", controllers: [], children: [] }] }, animations: [], diagnostics: [] }),
    artifacts: [
      artifact("package-hak", "HAK", [1, 2, 3], "a".repeat(64)),
      artifact("model-mdl", "MODEL", [1, 2], "b".repeat(64)),
      artifact("proof-module", "MODULE", [4, 5, 6, 7], "7".repeat(64)),
      artifact("report-json", "JSON_REPORT", [...new TextEncoder().encode(reportJson)], "c".repeat(64)),
      artifact("manifest-json", "JSON_REPORT", [...new TextEncoder().encode(manifestJson)], "f".repeat(64)),
      artifact("summary-json", "JSON_REPORT", [...new TextEncoder().encode(summaryJson)], "9".repeat(64)),
    ],
  };
}

async function settle() {
  await act(async () => { await new Promise((resolve) => window.setTimeout(resolve, 0)); });
}

async function selectFile(input: HTMLInputElement, file: File) {
  Object.defineProperty(input, "files", { configurable: true, value: [file] });
  await act(async () => { input.dispatchEvent(new window.Event("change", { bubbles: true })); });
  await settle();
}

function button(container: HTMLElement, label: string) {
  return Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
    .find((candidate) => candidate.textContent?.trim() === label);
}

function setValue(element: HTMLInputElement | HTMLTextAreaElement, value: string) {
  const prototype = element instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
  Object.getOwnPropertyDescriptor(prototype, "value")?.set?.call(element, value);
  element.dispatchEvent(new Event("input", { bubbles: true }));
}

async function renderApp(strict = false, options: { meshyBridge?: MeshyBridgeClient; meshyLabEnabled?: boolean } = {}) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  const app = <App {...options} />;
  await act(async () => root.render(strict ? <StrictMode>{app}</StrictMode> : app));
  return container;
}

async function driveToBuild(container: HTMLElement) {
  const [sourceInput, appearanceInput] = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="file"]'));
  await selectFile(sourceInput, localFile("source.glb", 1));
  await selectFile(appearanceInput, localFile("appearance.2da", 2));
  const worker = FakeWorker.instances.at(-1)!;
  const sourceRequest = worker.requests.filter((request) => request.type === "INSPECT_SOURCE").at(-1)!;
  const appearanceRequest = worker.requests.filter((request) => request.type === "INSPECT_APPEARANCE").at(-1)!;
  await act(async () => {
    worker.emit({ requestId: sourceRequest.requestId, ok: true, type: "SOURCE_INSPECTED", ingestJson: sourceInspectionJson() });
    worker.emit({ requestId: appearanceRequest.requestId, ok: true, type: "APPEARANCE_INSPECTED", inspectionJson: appearanceInspectionJson() });
    await Promise.resolve();
  });
  await act(async () => button(container, "Continue to Inspect")?.click());
  await act(async () => button(container, "Continue to Build")?.click());
  await act(async () => button(container, "Build Package")?.click());
  await settle();
  return { sourceInput, worker, build: worker.requests.find((request) => request.type === "BUILD_MODEL_PACKAGE")! };
}

describe("Studio workflow", () => {
  beforeEach(() => {
    FakeWorker.instances = [];
    vi.stubGlobal("Worker", FakeWorker);
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
  });

  afterEach(async () => {
    while (roots.length) await act(async () => roots.pop()?.unmount());
    document.body.replaceChildren();
    vi.unstubAllGlobals();
  });

  it("recreates the Worker after StrictMode cleanup", async () => {
    await renderApp(true);
    expect(FakeWorker.instances).toHaveLength(2);
    expect(FakeWorker.instances[0].terminated).toBe(true);
    expect(FakeWorker.instances[1].terminated).toBe(false);
  });

  it("moves inspected local inputs through Build into a readback-verified review", async () => {
    const container = await renderApp();
    const { build, worker } = await driveToBuild(container);
    await act(async () => { worker.emit(builtResponse(build.requestId)); await Promise.resolve(); });

    expect(container.querySelector("#review-model-heading")?.textContent).toBe("Model Details");
    expect(container.textContent).toContain("Verified by binary readback");
    expect(container.textContent).toContain("Conversion Readiness");
    expect(container.querySelector('[aria-label="Canonical Worker artifact downloads"]')?.textContent)
      .toContain("package-hak.bin");
  });

  it("rejects an unknown readback contract without a partial review", async () => {
    const container = await renderApp();
    const { build, worker } = await driveToBuild(container);
    await act(async () => { worker.emit(builtResponse(build.requestId, "BINARY_MDL")); await Promise.resolve(); });

    expect(container.querySelector("#review-model-heading")).toBeNull();
    expect(container.textContent).toContain("Build failed");
    expect(container.textContent).toContain("readbackJson.format");
  });

  it("drops a stale build response after replacing the source", async () => {
    const container = await renderApp();
    const { build, sourceInput, worker } = await driveToBuild(container);
    await selectFile(sourceInput, localFile("replacement.glb", 3));
    await act(async () => { worker.emit(builtResponse(build.requestId)); await Promise.resolve(); });

    expect(worker.terminated).toBe(true);
    expect(container.querySelector("#review-model-heading")).toBeNull();
  });

  it("imports a verified Meshy Lab GLB through the same Source intake as a local file", async () => {
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 120 });
    const container = await renderApp(false, { meshyBridge: bridge, meshyLabEnabled: true });
    await act(async () => button(container, "Open Meshy Lab")?.click());
    await act(async () => {
      setValue(container.querySelector<HTMLInputElement>("#meshy-pairing-code")!, "local-proof");
      button(container, "Connect local bridge")?.click();
    });
    await settle();
    await act(async () => {
      setValue(container.querySelector<HTMLTextAreaElement>("#meshy-asset-prompt")!, "A weathered stone lantern");
      Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
        .find((candidate) => candidate.textContent?.includes("S1 · Static Prop"))?.click();
    });
    await settle();
    await act(async () => button(container, "Review generation")?.click());
    await settle();
    await act(async () => button(container, "Generate S1 asset")?.click());
    await settle();
    await bridge.completeRunForTest(bridge.latestRunIdForTest()!, new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
    await act(async () => button(container, "Refresh status")?.click());
    await settle();
    await act(async () => button(container, "Import verified GLB to Source")?.click());
    await settle();

    expect(container.textContent).toContain("meshy-s1-static-prop.glb");
    expect(container.textContent).toContain("Imported from Meshy Lab: S1-static-prop/v1");
    expect(FakeWorker.instances.at(-1)?.requests.some((request) => request.type === "INSPECT_SOURCE")).toBe(true);
  });
});
