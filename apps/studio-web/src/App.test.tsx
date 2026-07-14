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

vi.mock("./features/preview/SceneViewport", () => ({
  SceneViewport: ({ provenance }: { provenance: string }) => <div data-testid={`viewport-${provenance}`} />,
}));

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

function jsonFile(name: string, value: unknown): File {
  const text = JSON.stringify(value);
  return {
    name,
    size: text.length,
    text: async () => text,
    arrayBuffer: async () => new TextEncoder().encode(text).buffer,
  } as File;
}

function jsonArtifact(fileName: string, json: string): WorkerArtifact {
  const bytes = new TextEncoder().encode(json).buffer;
  return {
    artifactId: fileName,
    kind: "JSON_REPORT",
    fileName,
    mediaType: "application/json",
    byteLength: bytes.byteLength,
    sha256: "0".repeat(64),
    bytes,
    provenance: "M2A_WASM_WORKER",
  };
}

function builtResponse(requestId: string, malformed?: "report" | "summary" | "manifest" | "readback" | "readback-shape"): StudioWorkerResponse {
  const report = {
    schemaVersion: 1,
    geometry: { vertexCount: 24, triangleCount: 12, activeJointCount: 2, outputSegmentDeformation: "SKIN" },
    model: { payloadSha256: "b".repeat(64), layout: { fileLength: 2 } },
    texture: { width: 2, height: 2, pixelFormat: "RGBA8", byteLength: 60, outputSha256: "d".repeat(64) },
    appearance: { appendedRowIndex: 1, sourcePrefixPreserved: true, outputByteLength: 7, outputSha256: "e".repeat(64) },
    hak: { byteLength: 3, archiveSha256: "a".repeat(64), entryCount: 3 },
  };
  let reportJson = JSON.stringify(report);
  const summary = {
    schemaVersion: 1,
    status: "M6_MODEL_PACKAGE_MATERIALIZED",
    outputs: {
      model: { byteLength: 2, sha256: "b".repeat(64) },
      texture: { byteLength: 60, sha256: "d".repeat(64) },
      appearanceTwoDa: { byteLength: 7, sha256: "e".repeat(64) },
      hak: { byteLength: 3, sha256: "a".repeat(64) },
      report: { byteLength: new TextEncoder().encode(reportJson).byteLength, sha256: "c".repeat(64) },
    },
    appendedPhysicalRow: 1,
    modelResref: "m2a_model",
    textureResref: "m2a_texture",
    animation: { sourceName: "walk", outputName: "cwalk", durationSeconds: 1.25, hasMotion: true },
    appearancePayloadPolicy: "PRESERVED_AND_APPENDED",
  };
  const manifest = {
    schemaVersion: 1,
    status: "M6_MODEL_PACKAGE_MATERIALIZED",
    appendedPhysicalRow: 1,
    appearancePayloadPolicy: "PRESERVED_AND_APPENDED",
    packageManifest: {
      packageSha256: "a".repeat(64),
      resources: [
        { role: "APPEARANCE_TABLE", resref: "appearance", type: 2017, byteLength: 7, sha256: "e".repeat(64) },
        { role: "MODEL", resref: "m2a_model", type: 2002, byteLength: 2, sha256: "b".repeat(64) },
        { role: "TEXTURE", resref: "m2a_texture", type: 3, byteLength: 60, sha256: "d".repeat(64) },
      ],
    },
  };
  let summaryJson = JSON.stringify(summary);
  let manifestJson = JSON.stringify(manifest);
  let readbackJson = JSON.stringify({ schemaVersion: 1, format: "BINARY_MDL", nodeTree: { roots: [] }, diagnostics: [] });
  if (malformed === "report") reportJson = "{}";
  if (malformed === "summary") summaryJson = "{}";
  if (malformed === "manifest") manifestJson = "{}";
  if (malformed === "readback") readbackJson = "{";
  if (malformed === "readback-shape") readbackJson = "{}";
  const artifact = (
    artifactId: string,
    kind: WorkerArtifact["kind"],
    content: ArrayBuffer,
    sha256: string,
  ): WorkerArtifact => ({
    artifactId,
    kind,
    fileName: `${artifactId}.bin`,
    mediaType: kind === "JSON_REPORT" ? "application/json" : "application/octet-stream",
    byteLength: content.byteLength,
    sha256,
    bytes: content,
    provenance: "M2A_WASM_WORKER",
  });
  return {
    requestId,
    ok: true,
    type: "MODEL_PACKAGE_BUILT",
    reportJson,
    summaryJson,
    manifestJson,
    readbackJson,
    artifacts: [
      artifact("package-hak", "HAK", new Uint8Array([1, 2, 3]).buffer, "a".repeat(64)),
      artifact("model-mdl", "MODEL", new Uint8Array([1, 2]).buffer, "b".repeat(64)),
      artifact("report-json", "JSON_REPORT", new TextEncoder().encode(reportJson).buffer, "c".repeat(64)),
      artifact("manifest-json", "JSON_REPORT", new TextEncoder().encode(manifestJson).buffer, "f".repeat(64)),
      artifact("summary-json", "JSON_REPORT", new TextEncoder().encode(summaryJson).buffer, "9".repeat(64)),
    ],
  };
}

function panelButton(panel: HTMLElement, label: string) {
  return Array.from(panel.querySelectorAll("button")).find(
    (candidate) => candidate.textContent === label,
  );
}

async function selectFile(input: HTMLInputElement, file: File) {
  Object.defineProperty(input, "files", { configurable: true, value: [file] });
  await act(async () => {
    input.dispatchEvent(new window.Event("change", { bubbles: true }));
  });
}

async function clearFile(input: HTMLInputElement) {
  Object.defineProperty(input, "files", { configurable: true, value: [] });
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

  it("commits an exact canonical result atomically, labels OPEN_M6, and resets it on input change", async () => {
    const container = await renderApp();
    const m6Panel = container.querySelector<HTMLElement>('[aria-label="Local file session"]')!;
    const [sourceInput, appearanceInput] = Array.from(m6Panel.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    await selectFile(sourceInput, localFile("source.glb", 1));
    await selectFile(appearanceInput, localFile("appearance.2da", 2));
    await act(async () => {
      panelButton(m6Panel, "Build model package")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const worker = FakeWorker.instances[0];
    const build = worker.requests.find((request) => request.type === "BUILD_MODEL_PACKAGE")!;
    await act(async () => {
      worker.emit(builtResponse(build.requestId));
      await Promise.resolve();
    });

    const workspace = container.querySelector<HTMLElement>('[aria-label="Canonical model result"]')!;
    expect(workspace.textContent).toContain("M6_MODEL_PACKAGE_MATERIALIZED");
    expect(workspace.textContent).toContain("24");
    expect(workspace.textContent).toContain("12");
    expect(workspace.textContent).toContain("walk → cwalk");
    expect(workspace.textContent).toContain("2 × 2 · RGBA8");
    expect(workspace.textContent).toContain("m2a_model");
    expect(workspace.textContent).toContain("OPEN_M6");
    expect(workspace.textContent).not.toContain("Runtime acceptance: PASS");
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("COMPLETE");
    const m7 = container.querySelector<HTMLElement>('[aria-label="M7 corpus session"]')!;
    expect(workspace.compareDocumentPosition(m7) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(container.textContent).toContain("report-json.bin");

    await selectFile(sourceInput, localFile("replacement.glb", 3));
    expect(container.querySelector('[aria-label="Canonical model result"]')).toBeNull();
    expect(container.textContent).not.toContain("report-json.bin");
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("READY");
  });

  it.each(["summary", "readback", "readback-shape"] as const)("turns malformed %s into ERROR with zero partial workspace or downloads", async (malformed) => {
    const container = await renderApp();
    const m6Panel = container.querySelector<HTMLElement>('[aria-label="Local file session"]')!;
    const [sourceInput, appearanceInput] = Array.from(m6Panel.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    await selectFile(sourceInput, localFile("source.glb", 1));
    await selectFile(appearanceInput, localFile("appearance.2da", 2));
    await act(async () => {
      panelButton(m6Panel, "Build model package")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const worker = FakeWorker.instances[0];
    const build = worker.requests.find((request) => request.type === "BUILD_MODEL_PACKAGE")!;
    await act(async () => {
      worker.emit(builtResponse(build.requestId, malformed));
      await Promise.resolve();
    });
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("ERROR");
    expect(m6Panel.textContent).toMatch(/Canonical (result|readback) field/);
    expect(container.querySelector('[aria-label="Canonical model result"]')).toBeNull();
    expect(container.textContent).not.toContain("package-hak.bin");
  });

  it("does not let a late source inspection overwrite WORKING or COMPLETE", async () => {
    const container = await renderApp();
    const m6Panel = container.querySelector<HTMLElement>('[aria-label="Local file session"]')!;
    const [sourceInput, appearanceInput] = Array.from(m6Panel.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    await selectFile(sourceInput, localFile("source.glb", 1));
    await selectFile(appearanceInput, localFile("appearance.2da", 2));
    await act(async () => {
      panelButton(m6Panel, "Build model package")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const worker = FakeWorker.instances[0];
    const inspect = worker.requests.find((request) => request.type === "INSPECT_SOURCE")!;
    const build = worker.requests.find((request) => request.type === "BUILD_MODEL_PACKAGE")!;
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("WORKING");
    await act(async () => {
      worker.emit(builtResponse(build.requestId));
      await Promise.resolve();
    });
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("COMPLETE");
    await act(async () => {
      worker.emit({ requestId: inspect.requestId, ok: true, type: "SOURCE_INSPECTED", ingestJson: "{" });
      await Promise.resolve();
    });
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("COMPLETE");
    expect(container.querySelector('[aria-label="Canonical model result"]')).not.toBeNull();
  });

  it("keeps a pending source inspection valid when only appearance changes", async () => {
    const container = await renderApp();
    const m6Panel = container.querySelector<HTMLElement>('[aria-label="Local file session"]')!;
    const [sourceInput, appearanceInput] = Array.from(m6Panel.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    await selectFile(sourceInput, localFile("source.glb", 1));
    const worker = FakeWorker.instances[0];
    const inspect = worker.requests.find((request) => request.type === "INSPECT_SOURCE")!;
    await selectFile(appearanceInput, localFile("appearance.2da", 2));
    await act(async () => {
      worker.emit({
        requestId: inspect.requestId,
        ok: true,
        type: "SOURCE_INSPECTED",
        ingestJson: JSON.stringify({ ir: { source: { sha256: "a".repeat(64) } } }),
      });
      await Promise.resolve();
    });
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("READY");
    expect(container.querySelector('[data-testid="viewport-SOURCE"]')).not.toBeNull();
  });

  it.each([
    ["FAILED", { ok: false, type: "FAILED", message: "SOURCE-INSPECT-FAILED" }],
    ["unexpected", { ok: true, type: "INITIALIZED" }],
  ] as const)("reports a guarded %s source inspection error", async (_label, outcome) => {
    const container = await renderApp();
    const m6Panel = container.querySelector<HTMLElement>('[aria-label="Local file session"]')!;
    const sourceInput = m6Panel.querySelector<HTMLInputElement>('input[type="file"]')!;
    await selectFile(sourceInput, localFile("source.glb", 1));
    const worker = FakeWorker.instances[0];
    const inspect = worker.requests.find((request) => request.type === "INSPECT_SOURCE")!;
    await act(async () => {
      worker.emit({ requestId: inspect.requestId, ...outcome } as StudioWorkerResponse);
      await Promise.resolve();
    });
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("ERROR");
    expect(m6Panel.textContent).toContain(
      outcome.type === "FAILED" ? "SOURCE-INSPECT-FAILED" : "Unexpected source inspection response",
    );
  });

  it("does not let a source inspection failure overwrite WORKING", async () => {
    const container = await renderApp();
    const m6Panel = container.querySelector<HTMLElement>('[aria-label="Local file session"]')!;
    const [sourceInput, appearanceInput] = Array.from(m6Panel.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    await selectFile(sourceInput, localFile("source.glb", 1));
    await selectFile(appearanceInput, localFile("appearance.2da", 2));
    await act(async () => {
      panelButton(m6Panel, "Build model package")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const worker = FakeWorker.instances[0];
    const inspect = worker.requests.find((request) => request.type === "INSPECT_SOURCE")!;
    await act(async () => {
      worker.emit({ requestId: inspect.requestId, ok: false, type: "FAILED", message: "LATE-INSPECT" });
      await Promise.resolve();
    });
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("WORKING");
    expect(m6Panel.textContent).not.toContain("LATE-INSPECT");
  });

  it("maps explicit M7 paths into one deterministic Worker envelope and renders deferred", async () => {
    const container = await renderApp();
    const panel = container.querySelector<HTMLElement>('[aria-label="M7 corpus session"]')!;
    const manifest = {
      schemaVersion: 1,
      corpusId: "ui-corpus",
      artDirectionApprovalId: null,
      samples: [
        { role: "RIGGED_HUMANOID_SOURCE_CLIPS", sampleId: "human", source: { relativePath: "models/h.glb" }, requiredSourceClipNames: ["walk"] },
        { role: "NON_HUMANOID_REFERENCE_SUPERMODEL", sampleId: "creature", source: { relativePath: "models/c.glb" }, referenceSupermodel: "c_dog" },
        { role: "STATIC_PLACEABLE_OR_ITEM", sampleId: "prop", source: { relativePath: "models/p.glb" }, resourceKind: "PLACEABLE" },
      ],
    };
    await selectFile(panel.querySelector<HTMLInputElement>('input[type="file"]')!, jsonFile("m7.json", manifest));
    await act(async () => {
      panelButton(panel, "Validate manifest")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const validationRequest = FakeWorker.instances[0].requests.find((candidate) => candidate.type === "VALIDATE_M7_CORPUS")!;
    const validatedJson = JSON.stringify(manifest);
    await act(async () => {
      FakeWorker.instances[0].emit({
        requestId: validationRequest.requestId,
        ok: true,
        type: "M7_CORPUS_VALIDATED",
        manifestJson: validatedJson,
        artifacts: [jsonArtifact("m7-manifest-validation.json", validatedJson)],
      });
      await Promise.resolve();
    });
    expect(panel.textContent).toContain("models/h.glb");
    expect(panel.textContent).toContain("NON_HUMANOID_REFERENCE_SUPERMODEL");

    const inputs = Array.from(panel.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    await selectFile(inputs[1], localFile("h.glb", 1));
    await selectFile(inputs[2], localFile("appearance.2da", 4));
    await selectFile(inputs[3], localFile("c.glb", 2));
    await selectFile(inputs[4], localFile("p.glb", 3));
    const button = panelButton(panel, "Build M7 reports");
    await act(async () => {
      button?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });

    const worker = FakeWorker.instances[0];
    const request = worker.requests.find((candidate) => candidate.type === "BUILD_M7_CORPUS_BATCH");
    expect(request?.type).toBe("BUILD_M7_CORPUS_BATCH");
    if (!request || request.type !== "BUILD_M7_CORPUS_BATCH") throw new Error("missing M7 request");
    expect([...new Uint8Array(request.payloadBlob)]).toEqual([1, 2, 3, 4]);
    expect(JSON.parse(request.descriptorsJson).payloads.map((item: { role: string; payloadOffset: number }) => [item.role, item.payloadOffset])).toEqual([
      ["SOURCE", 0], ["SOURCE", 1], ["SOURCE", 2], ["RIGGED_HUMANOID_APPEARANCE_2DA", 3],
    ]);
    const batchJson = JSON.stringify({ report: { status: "INPUT_DEFERRED" }, packets: [] });
    await act(async () => {
      worker.emit({
        requestId: request.requestId,
        ok: true,
        type: "M7_CORPUS_BATCH_BUILT",
        batchJson,
        artifacts: [jsonArtifact("m7-batch.json", batchJson)],
      });
      await Promise.resolve();
    });
    expect(panel.textContent).toContain("Batch: INPUT_DEFERRED");
    expect(panel.textContent).toContain("never claims M7 DONE");
    expect(panel.textContent).toContain("m7-batch.json");
    await clearFile(inputs[1]);
    expect(panel.textContent).not.toContain("m7-batch.json");
    await act(async () => {
      panelButton(panel, "Build M7 reports")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    expect(panel.textContent).not.toContain("m7-batch.json");
  });

  it("ignores a stale manifest text read", async () => {
    const container = await renderApp();
    const panel = container.querySelector<HTMLElement>('[aria-label="M7 corpus session"]')!;
    let resolveFirst!: (value: string) => void;
    const first = {
      name: "first.json",
      size: 1,
      text: () => new Promise<string>((resolve) => { resolveFirst = resolve; }),
    } as File;
    const secondValue = { schemaVersion: 1, corpusId: "second", samples: [] };
    const input = panel.querySelector<HTMLInputElement>('input[type="file"]')!;
    await selectFile(input, first);
    await selectFile(input, jsonFile("second.json", secondValue));
    await act(async () => { await Promise.resolve(); });
    resolveFirst(JSON.stringify({ schemaVersion: 1, corpusId: "first", samples: [] }));
    await act(async () => { await Promise.resolve(); });

    await act(async () => {
      panelButton(panel, "Validate manifest")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const request = FakeWorker.instances[0].requests.find((candidate) => candidate.type === "VALIDATE_M7_CORPUS");
    expect(request?.type === "VALIDATE_M7_CORPUS" && request.manifestJson).toBe(JSON.stringify(secondValue));
    expect(panel.textContent).toContain("second.json");
    expect(panel.textContent).not.toContain("first.json");
  });

  it("drops a stale M7 Worker response after the manifest changes", async () => {
    const container = await renderApp();
    const panel = container.querySelector<HTMLElement>('[aria-label="M7 corpus session"]')!;
    const input = panel.querySelector<HTMLInputElement>('input[type="file"]')!;
    await selectFile(input, jsonFile("first.json", { schemaVersion: 1, corpusId: "first", samples: [] }));
    await act(async () => {
      panelButton(panel, "Validate manifest")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const request = FakeWorker.instances[0].requests.find((candidate) => candidate.type === "VALIDATE_M7_CORPUS")!;
    await selectFile(input, jsonFile("second.json", { schemaVersion: 1, corpusId: "second", samples: [] }));
    const staleJson = JSON.stringify({ schemaVersion: 1, corpusId: "first", samples: [] });
    await act(async () => {
      FakeWorker.instances[0].emit({
        requestId: request.requestId,
        ok: true,
        type: "M7_CORPUS_VALIDATED",
        manifestJson: staleJson,
        artifacts: [jsonArtifact("stale-report.json", staleJson)],
      });
      await Promise.resolve();
    });
    expect(panel.textContent).toContain("second.json");
    expect(panel.textContent).not.toContain("stale-report.json");
    expect(panelButton(panel, "Inspect intake")?.disabled).toBe(true);
  });

  it("shows the exact WASM code for an invalid manifest shape without rendering samples", async () => {
    const container = await renderApp();
    const panel = container.querySelector<HTMLElement>('[aria-label="M7 corpus session"]')!;
    await selectFile(
      panel.querySelector<HTMLInputElement>('input[type="file"]')!,
      jsonFile("invalid.json", { schemaVersion: 1, corpusId: "invalid", samples: {} }),
    );
    await act(async () => {
      panelButton(panel, "Validate manifest")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const request = FakeWorker.instances[0].requests.find((candidate) => candidate.type === "VALIDATE_M7_CORPUS")!;
    const invalidJson = JSON.stringify({ schemaVersion: 1, code: "M7-MANIFEST-JSON-INVALID", path: "manifest" });
    await act(async () => {
      FakeWorker.instances[0].emit({
        requestId: request.requestId,
        ok: true,
        type: "M7_CORPUS_VALIDATED",
        manifestJson: invalidJson,
        artifacts: [jsonArtifact("invalid.json", invalidJson)],
      });
      await Promise.resolve();
    });
    expect(panel.textContent).toContain("Manifest: M7-MANIFEST-JSON-INVALID");
    expect(panelButton(panel, "Inspect intake")?.disabled).toBe(true);
    expect(panel.querySelectorAll("fieldset")).toHaveLength(0);
  });

  it("renders INPUT_INVALID intake as an exact JSON report", async () => {
    const container = await renderApp();
    const panel = container.querySelector<HTMLElement>('[aria-label="M7 corpus session"]')!;
    const manifest = {
      schemaVersion: 1,
      corpusId: "invalid-intake",
      artDirectionApprovalId: "approval",
      samples: [
        { role: "RIGGED_HUMANOID_SOURCE_CLIPS", sampleId: "human", source: null, requiredSourceClipNames: ["walk"] },
        { role: "NON_HUMANOID_REFERENCE_SUPERMODEL", sampleId: "creature", source: null, referenceSupermodel: "c_dog" },
        { role: "STATIC_PLACEABLE_OR_ITEM", sampleId: "prop", source: null, resourceKind: "PLACEABLE" },
      ],
    };
    await selectFile(panel.querySelector<HTMLInputElement>('input[type="file"]')!, jsonFile("intake.json", manifest));
    await act(async () => {
      panelButton(panel, "Validate manifest")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const worker = FakeWorker.instances[0];
    const validation = worker.requests.find((candidate) => candidate.type === "VALIDATE_M7_CORPUS")!;
    const manifestJson = JSON.stringify(manifest);
    await act(async () => {
      worker.emit({
        requestId: validation.requestId,
        ok: true,
        type: "M7_CORPUS_VALIDATED",
        manifestJson,
        artifacts: [jsonArtifact("validation.json", manifestJson)],
      });
      await Promise.resolve();
    });
    await act(async () => {
      panelButton(panel, "Inspect intake")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const intake = worker.requests.find((candidate) => candidate.type === "INSPECT_M7_CORPUS_INTAKE")!;
    const intakeJson = JSON.stringify({ status: "INPUT_INVALID", diagnostics: [{ code: "M7-SOURCE-IDENTITY-MISMATCH" }] });
    await act(async () => {
      worker.emit({
        requestId: intake.requestId,
        ok: true,
        type: "M7_CORPUS_INTAKE_INSPECTED",
        intakeJson,
        artifacts: [jsonArtifact("m7-intake.json", intakeJson)],
      });
      await Promise.resolve();
    });
    expect(panel.textContent).toContain("Intake: INPUT_INVALID");
    expect(panel.textContent).toContain("m7-intake.json");
  });

  it("propagates the exact Worker failure message", async () => {
    const container = await renderApp();
    const [m6Source, m6Appearance] = Array.from(
      container.querySelectorAll<HTMLInputElement>('[aria-label="Local file session"] input[type="file"]'),
    );
    await selectFile(m6Source, localFile("ready.glb", 1));
    await selectFile(m6Appearance, localFile("appearance.2da", 2));
    const panel = container.querySelector<HTMLElement>('[aria-label="M7 corpus session"]')!;
    await selectFile(
      panel.querySelector<HTMLInputElement>('input[type="file"]')!,
      jsonFile("failure.json", { schemaVersion: 1, corpusId: "failure", samples: [] }),
    );
    await act(async () => {
      panelButton(panel, "Validate manifest")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const request = FakeWorker.instances[0].requests.find((candidate) => candidate.type === "VALIDATE_M7_CORPUS")!;
    await act(async () => {
      FakeWorker.instances[0].emit({ requestId: request.requestId, ok: false, type: "FAILED", message: "M7-EXACT-WORKER-FAILURE" });
      await Promise.resolve();
    });
    expect(panel.textContent).toContain("M7 Worker error: M7-EXACT-WORKER-FAILURE");
    const m6Panel = container.querySelector<HTMLElement>('[aria-label="Local file session"]')!;
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("READY");
    expect(m6Panel.textContent).not.toContain("M7-EXACT-WORKER-FAILURE");
    expect(panelButton(m6Panel, "Build model package")?.disabled).toBe(false);
  });

  it("keeps a completed M7 report isolated from a later M6 result error", async () => {
    const container = await renderApp();
    const worker = FakeWorker.instances[0];
    const m7Panel = container.querySelector<HTMLElement>('[aria-label="M7 corpus session"]')!;
    const manifest = { schemaVersion: 1, corpusId: "isolated-m7", samples: [] };
    await selectFile(m7Panel.querySelector<HTMLInputElement>('input[type="file"]')!, jsonFile("m7.json", manifest));
    await act(async () => {
      panelButton(m7Panel, "Validate manifest")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const validation = worker.requests.find((request) => request.type === "VALIDATE_M7_CORPUS")!;
    const manifestJson = JSON.stringify(manifest);
    await act(async () => {
      worker.emit({
        requestId: validation.requestId,
        ok: true,
        type: "M7_CORPUS_VALIDATED",
        manifestJson,
        artifacts: [jsonArtifact("m7-isolated.json", manifestJson)],
      });
      await Promise.resolve();
    });
    expect(m7Panel.textContent).toContain("VALID manifest: isolated-m7");
    expect(m7Panel.textContent).toContain("m7-isolated.json");

    const m6Panel = container.querySelector<HTMLElement>('[aria-label="Local file session"]')!;
    const [sourceInput, appearanceInput] = Array.from(m6Panel.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    await selectFile(sourceInput, localFile("source.glb", 1));
    await selectFile(appearanceInput, localFile("appearance.2da", 2));
    await act(async () => {
      panelButton(m6Panel, "Build model package")?.dispatchEvent(new window.MouseEvent("click", { bubbles: true }));
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const build = worker.requests.find((request) => request.type === "BUILD_MODEL_PACKAGE")!;
    await act(async () => {
      worker.emit(builtResponse(build.requestId, "manifest"));
      await Promise.resolve();
    });
    expect(m6Panel.querySelector(".status strong")?.textContent).toBe("ERROR");
    expect(m7Panel.textContent).toContain("VALID manifest: isolated-m7");
    expect(m7Panel.textContent).toContain("m7-isolated.json");
  });
});
