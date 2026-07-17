import { afterEach, describe, expect, it } from "vitest";
import { createElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import sourceUrl from "../.generated/owned-package/generated/source.glb?url";
import expectedHakUrl from "../.generated/owned-package/generated/m2a_codex_aproof.hak?url";
import appearanceUrl from "../fixtures/appearance.2da?url";
import { buildM7PayloadEnvelope } from "../../src/features/m7/envelope";
import { projectCanonicalResult } from "../../src/features/results/projectCanonicalResult";
import { projectCanonicalReadback } from "../../src/features/results/projectReadback";
import { StudioWorkerClient } from "../../src/worker/client";
import { App } from "../../src/App";

const clients: StudioWorkerClient[] = [];
const roots: Root[] = [];

async function fetchBytes(url: string): Promise<ArrayBuffer> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`fixture fetch failed: ${response.status} ${url}`);
  }
  return response.arrayBuffer();
}

async function fixtureFile(url: string, name: string, type: string): Promise<File> {
  return new File([await fetchBytes(url)], name, { type });
}

async function sha256(bytes: ArrayBuffer): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function expectExactJsonArtifact(
  artifacts: Array<{
    kind: string;
    fileName: string;
    mediaType: string;
    byteLength: number;
    sha256: string;
    bytes: ArrayBuffer;
    provenance: string;
  }>,
  fileName: string,
  json: string,
) {
  const artifact = artifacts.find((candidate) => candidate.fileName === fileName);
  expect(artifact).toBeDefined();
  expect(artifact).toMatchObject({
    kind: "JSON_REPORT",
    fileName,
    mediaType: "application/json",
    provenance: "M2A_WASM_WORKER",
  });
  const expectedBytes = new TextEncoder().encode(json);
  expect(new Uint8Array(artifact!.bytes)).toEqual(expectedBytes);
  expect(artifact!.byteLength).toBe(expectedBytes.byteLength);
  expect(artifact!.byteLength).toBe(artifact!.bytes.byteLength);
  expect(artifact!.sha256).toMatch(/^[0-9a-f]{64}$/);
  expect(artifact!.sha256).toBe(await sha256(artifact!.bytes));
}

function withoutRigAndAnimations(glb: ArrayBuffer): ArrayBuffer {
  const input = new Uint8Array(glb);
  const view = new DataView(glb);
  const jsonLength = view.getUint32(12, true);
  const jsonEnd = 20 + jsonLength;
  const root = JSON.parse(new TextDecoder().decode(input.slice(20, jsonEnd))) as {
    skins?: unknown;
    animations?: unknown;
    nodes: Array<Record<string, unknown>>;
  };
  delete root.skins;
  delete root.animations;
  root.nodes.forEach((node) => delete node.skin);
  const json = new TextEncoder().encode(JSON.stringify(root));
  const paddedLength = (json.byteLength + 3) & ~3;
  const result = new Uint8Array(20 + paddedLength + input.byteLength - jsonEnd);
  result.set(new TextEncoder().encode("glTF"), 0);
  const outputView = new DataView(result.buffer);
  outputView.setUint32(4, 2, true);
  outputView.setUint32(8, result.byteLength, true);
  outputView.setUint32(12, paddedLength, true);
  result.set(new TextEncoder().encode("JSON"), 16);
  result.fill(0x20, 20, 20 + paddedLength);
  result.set(json, 20);
  result.set(input.slice(jsonEnd), 20 + paddedLength);
  return result.buffer;
}

const provenance = (providerTaskId: string) => ({
  provider: "MESHY",
  providerTaskId,
  originalExportAttested: true,
  rightsConfirmed: true,
  notSyntheticFixtureAttested: true,
});

afterEach(async () => {
  while (clients.length) clients.pop()?.dispose();
  while (roots.length) {
    const root = roots.pop();
    root?.unmount();
  }
  document.body.replaceChildren();
});

describe("local file to canonical web-WASM Worker integration", () => {
  it("materializes the owned synthetic GLB as the native-identical HAK and reports", async () => {
    const source = await fixtureFile(sourceUrl, "source-owned.synthetic.glb", "model/gltf-binary");
    const appearance = await fixtureFile(appearanceUrl, "appearance.2da", "text/plain");
    const expectedHak = await fetchBytes(expectedHakUrl);
    const sourceGlb = await source.arrayBuffer();
    const appearanceTwoDa = await appearance.arrayBuffer();
    const client = new StudioWorkerClient();
    clients.push(client);

    const initialized = await client.request({ requestId: "integration-init", type: "INITIALIZE" });
    expect(initialized).toMatchObject({ ok: true, type: "INITIALIZED" });

    const response = await client.request(
      {
        requestId: "integration-build",
        type: "BUILD_MODEL_PACKAGE",
        sourceGlb,
        appearanceTwoDa,
      },
      [sourceGlb, appearanceTwoDa],
    );

    expect(sourceGlb.byteLength).toBe(0);
    expect(appearanceTwoDa.byteLength).toBe(0);
    expect(response.ok).toBe(true);
    expect(response.type).toBe("MODEL_PACKAGE_BUILT");
    if (!response.ok || response.type !== "MODEL_PACKAGE_BUILT") {
      throw new Error("real Worker did not return a model package");
    }

    const hak = response.artifacts.find((artifact) => artifact.kind === "HAK");
    const reportArtifact = response.artifacts.find(
      (artifact) => artifact.fileName === "inspection.json",
    );
    expect(hak).toBeDefined();
    expect(reportArtifact).toBeDefined();
    expect(hak?.provenance).toBe("M2A_WASM_WORKER");
    expect(new Uint8Array(hak!.bytes)).toEqual(new Uint8Array(expectedHak));
    expect(new TextDecoder().decode(hak!.bytes.slice(0, 8))).toBe("HAK V1.0");
    expect(hak?.byteLength).toBe(hak?.bytes.byteLength);
    expect(hak?.sha256).toBe(await sha256(hak!.bytes));

    const report = JSON.parse(response.reportJson) as {
      geometry?: { triangleCount?: number };
      texture?: { width?: number; height?: number };
    };
    const summary = JSON.parse(response.summaryJson) as {
      status?: string;
      inputGlb?: { byteLength?: number };
    };
    const manifest = JSON.parse(response.manifestJson) as {
      packageManifest?: { resources?: unknown[] };
    };
    const readback = JSON.parse(response.readbackJson) as {
      nodeTree?: { roots?: unknown[] };
    };
    expect(report.geometry?.triangleCount).toBeGreaterThan(1);
    expect(report.texture).toMatchObject({ width: 2, height: 2 });
    expect(summary.status).toBe("M6_MODEL_PACKAGE_MATERIALIZED");
    expect(summary.inputGlb?.byteLength).toBe(source.size);
    expect(manifest.packageManifest?.resources).toHaveLength(3);
    expect(readback.nodeTree?.roots?.length).toBeGreaterThan(0);
    expect(new TextDecoder().decode(reportArtifact!.bytes)).toBe(response.reportJson);
    const snapshot = projectCanonicalResult(
      response.reportJson,
      response.summaryJson,
      response.manifestJson,
      response.artifacts,
    );
    expect(snapshot).toMatchObject({
      status: "M6_MODEL_PACKAGE_MATERIALIZED",
      geometry: { vertices: 24, triangles: 12, joints: 2, deformation: "SKIN" },
      animation: {
        sourceName: "owned-linear-pause",
        outputName: "cpause1",
        durationSeconds: 1.25,
        hasMotion: true,
      },
      texture: { width: 2, height: 2, pixelFormat: "RGBA8", byteLength: 60 },
      resrefs: { model: "m2a_m6p01", texture: "m2a_m6t01" },
      appearance: { appendedRow: 1, sourcePrefixPreserved: true },
      hak: { entryCount: 3 },
    });
    expect(projectCanonicalReadback(response.readbackJson).nodeTree.roots.length).toBeGreaterThan(0);
  }, 30_000);

  it("returns deterministic deferred M7 JSON from local Files without claiming completion", async () => {
    const manifestFile = new File([JSON.stringify({
      schemaVersion: 1,
      corpusId: "browser-deferred",
      artDirectionApprovalId: null,
      samples: [
        { role: "RIGGED_HUMANOID_SOURCE_CLIPS", sampleId: "human", source: null, requiredSourceClipNames: ["walk"] },
        { role: "NON_HUMANOID_REFERENCE_SUPERMODEL", sampleId: "creature", source: null, referenceSupermodel: "c_dog" },
        { role: "STATIC_PLACEABLE_OR_ITEM", sampleId: "prop", source: null, resourceKind: "PLACEABLE" },
      ],
    })], "m7-deferred.json", { type: "application/json" });
    const appearance = new File([new Uint8Array([1])], "appearance.2da", { type: "text/plain" });
    const envelope = await buildM7PayloadEnvelope([], [{
      role: "RIGGED_HUMANOID_APPEARANCE_2DA",
      sampleId: "human",
      file: appearance,
    }]);
    const manifestJson = await manifestFile.text();
    const client = new StudioWorkerClient();
    clients.push(client);
    const response = await client.request({
      requestId: "m7-deferred",
      type: "BUILD_M7_CORPUS_BATCH",
      manifestJson,
      ...envelope,
    }, [envelope.payloadBlob]);

    expect(envelope.payloadBlob.byteLength).toBe(0);
    expect(response.ok).toBe(true);
    expect(response.type).toBe("M7_CORPUS_BATCH_BUILT");
    if (!response.ok || response.type !== "M7_CORPUS_BATCH_BUILT") throw new Error("missing M7 deferred response");
    const batch = JSON.parse(response.batchJson) as {
      report: { status: string; materializedPacketCount: number; m7DoneClaimAllowed: boolean };
      packets: Array<{ sampleId: string; status: string }>;
    };
    expect(batch.report).toMatchObject({ status: "INPUT_DEFERRED", materializedPacketCount: 0, m7DoneClaimAllowed: false });
    expect(batch.packets.map(({ sampleId, status }) => [sampleId, status])).toEqual([
      ["human", "INPUT_DEFERRED"], ["creature", "INPUT_DEFERRED"], ["prop", "INPUT_DEFERRED"],
    ]);
    expect(response.batchJson.toLowerCase()).not.toContain("base64");
    await expectExactJsonArtifact(response.artifacts, "m7-batch.json", response.batchJson);
  }, 30_000);

  it("runs the owned READY corpus through the real Worker and public M7 WASM exports", async () => {
    const humanoidBytes = await fetchBytes(sourceUrl);
    const staticBytes = withoutRigAndAnimations(humanoidBytes.slice(0));
    const appearanceBytes = await fetchBytes(appearanceUrl);
    const humanoid = new File([humanoidBytes], "humanoid.glb", { type: "model/gltf-binary" });
    const creature = new File([staticBytes], "creature.glb", { type: "model/gltf-binary" });
    const prop = new File([staticBytes], "prop.glb", { type: "model/gltf-binary" });
    const appearance = new File([appearanceBytes], "appearance.2da", { type: "text/plain" });
    const identity = async (bytes: ArrayBuffer) => ({ byteLength: bytes.byteLength, sha256: await sha256(bytes) });
    const manifestFile = new File([JSON.stringify({
      schemaVersion: 1,
      corpusId: "browser-ready-owned",
      artDirectionApprovalId: "owned-test-approval",
      samples: [
        { role: "RIGGED_HUMANOID_SOURCE_CLIPS", sampleId: "human", source: { relativePath: "models/h.glb", identity: await identity(humanoidBytes), provenance: provenance("task-h") }, requiredSourceClipNames: ["owned-linear-pause"] },
        { role: "NON_HUMANOID_REFERENCE_SUPERMODEL", sampleId: "creature", source: { relativePath: "models/c.glb", identity: await identity(staticBytes), provenance: provenance("task-c") }, referenceSupermodel: "c_dog" },
        { role: "STATIC_PLACEABLE_OR_ITEM", sampleId: "prop", source: { relativePath: "models/p.glb", identity: await identity(staticBytes), provenance: provenance("task-p") }, resourceKind: "PLACEABLE" },
      ],
    })], "m7-ready.json", { type: "application/json" });
    const manifestJson = await manifestFile.text();
    const selections = [
      { role: "SOURCE" as const, relativePath: "models/h.glb", file: humanoid },
      { role: "SOURCE" as const, relativePath: "models/c.glb", file: creature },
      { role: "SOURCE" as const, relativePath: "models/p.glb", file: prop },
    ];
    const appearances = [{ role: "RIGGED_HUMANOID_APPEARANCE_2DA" as const, sampleId: "human", file: appearance }];
    const client = new StudioWorkerClient();
    clients.push(client);

    const validation = await client.request({ requestId: "m7-validate", type: "VALIDATE_M7_CORPUS", manifestJson });
    expect(validation).toMatchObject({ ok: true, type: "M7_CORPUS_VALIDATED" });
    if (!validation.ok || validation.type !== "M7_CORPUS_VALIDATED") throw new Error("missing M7 validation response");
    await expectExactJsonArtifact(
      validation.artifacts,
      "m7-manifest-validation.json",
      validation.manifestJson,
    );
    const intakeEnvelope = await buildM7PayloadEnvelope(selections, appearances);
    const intake = await client.request({
      requestId: "m7-intake",
      type: "INSPECT_M7_CORPUS_INTAKE",
      manifestJson,
      ...intakeEnvelope,
    }, [intakeEnvelope.payloadBlob]);
    expect(intakeEnvelope.payloadBlob.byteLength).toBe(0);
    expect(intake.ok && intake.type === "M7_CORPUS_INTAKE_INSPECTED" && JSON.parse(intake.intakeJson).status).toBe("READY_FOR_M7_V5");
    if (!intake.ok || intake.type !== "M7_CORPUS_INTAKE_INSPECTED") throw new Error("missing M7 intake response");
    await expectExactJsonArtifact(intake.artifacts, "m7-intake.json", intake.intakeJson);

    const firstEnvelope = await buildM7PayloadEnvelope(selections, appearances);
    const first = await client.request({ requestId: "m7-build-1", type: "BUILD_M7_CORPUS_BATCH", manifestJson, ...firstEnvelope }, [firstEnvelope.payloadBlob]);
    const secondEnvelope = await buildM7PayloadEnvelope(selections, appearances);
    const second = await client.request({ requestId: "m7-build-2", type: "BUILD_M7_CORPUS_BATCH", manifestJson, ...secondEnvelope }, [secondEnvelope.payloadBlob]);
    expect(firstEnvelope.payloadBlob.byteLength).toBe(0);
    expect(secondEnvelope.payloadBlob.byteLength).toBe(0);
    expect(first.ok && first.type === "M7_CORPUS_BATCH_BUILT").toBe(true);
    expect(second.ok && second.type === "M7_CORPUS_BATCH_BUILT").toBe(true);
    if (!first.ok || first.type !== "M7_CORPUS_BATCH_BUILT" || !second.ok || second.type !== "M7_CORPUS_BATCH_BUILT") throw new Error("missing M7 READY response");
    expect(first.batchJson).toBe(second.batchJson);
    const batch = JSON.parse(first.batchJson) as { report: { materializedPacketCount: number; deferredPacketCount: number }; packets: Array<{ sampleId: string; status: string }> };
    expect(batch.report).toMatchObject({ materializedPacketCount: 1, deferredPacketCount: 2 });
    expect(batch.packets.map(({ sampleId, status }) => [sampleId, status])).toEqual([
      ["human", "CANONICAL_PACKAGE_MATERIALIZED"], ["creature", "INPUT_DEFERRED"], ["prop", "INPUT_DEFERRED"],
    ]);
    expect(first.batchJson.toLowerCase()).not.toContain("base64");
    await expectExactJsonArtifact(first.artifacts, "m7-batch.json", first.batchJson);
    await expectExactJsonArtifact(second.artifacts, "m7-batch.json", second.batchJson);
  }, 30_000);

  it("renders and resets the production App canonical result from local files", async () => {
    const source = await fixtureFile(sourceUrl, "source-owned.synthetic.glb", "model/gltf-binary");
    const appearance = await fixtureFile(appearanceUrl, "appearance.2da", "text/plain");
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    root.render(createElement(App));
    await expect.poll(() => container.querySelector(".inputs-panel")).toBeTruthy();
    const [sourceInput, appearanceInput] = Array.from(container.querySelectorAll<HTMLInputElement>(".inputs-panel input[type=file]"));
    const select = async (input: HTMLInputElement, file: File) => {
      Object.defineProperty(input, "files", { configurable: true, value: [file] });
      input.dispatchEvent(new Event("change", { bubbles: true }));
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    };
    await select(sourceInput, source);
    await select(appearanceInput, appearance);
    const findButton = (label: string) => Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find(({ textContent }) => textContent?.trim() === label);
    await expect.poll(() => findButton("Continue to Inspect")?.disabled).toBe(false);
    findButton("Continue to Inspect")?.click();
    await expect.poll(() => findButton("Continue to Build")?.disabled).toBe(false);
    findButton("Continue to Build")?.click();
    await expect.poll(() => findButton("Build Package")?.disabled).toBe(false);
    findButton("Build Package")!.click();
    await expect.poll(() => container.querySelector("#review-model-heading")?.textContent, { timeout: 20_000 }).toBe("Model Details");

    const workspace = container.querySelector<HTMLElement>(".review-model")!;
    expect(workspace.textContent).toContain("Conversion Readiness");
    expect(workspace.textContent).toContain("24");
    expect(workspace.textContent).toContain("12");
    expect(workspace.textContent).toContain("OPEN_M6");
    expect(workspace.textContent).toContain("Verified by binary readback");
    expect(workspace.textContent).toContain("Metrics available in both canonical snapshots");
    expect(container.querySelector('[aria-label="Canonical Worker artifact downloads"]')?.textContent)
      .toContain("GENERATED ARTIFACTS");

    await select(sourceInput, new File([await fetchBytes(sourceUrl)], "replacement.glb", { type: "model/gltf-binary" }));
    await expect.poll(() => container.querySelector("#review-model-heading")).toBeNull();
  }, 30_000);
});
