// @vitest-environment jsdom

import { StrictMode, act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { InMemoryMeshyBridgeClient, type MeshyBridgeClient } from "./features/meshy/bridge";
import { getDirectCreatureBaseCatalogV1 } from "./features/animation-mapping/catalog";
import { FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 } from "./features/source/directCreatureAnimationProfile";
import type { StudioWorkerRequest, StudioWorkerResponse, WorkerArtifact } from "./worker/types";

vi.mock("./features/preview/SceneViewport", () => ({
  SceneViewport: ({ provenance }: { provenance: string }) => <div data-testid={`viewport-${provenance}`} />,
}));
vi.mock("./features/placeable-authoring/PlaceableAuthoringEditor", () => ({
  PlaceableAuthoringEditor: ({
    bootstrap,
    onDocumentChange,
  }: {
    bootstrap: {
      document: {
        schemaVersion: 1;
        sourceSha256: string;
        elements: unknown[];
      };
    };
    onDocumentChange: (document: unknown) => void;
  }) => (
    <button
      type="button"
      onClick={() => onDocumentChange({
        ...bootstrap.document,
        elements: [{
          id: "mock-authored-element",
          marker: "edited-in-placeable-editor",
        }],
      })}
    >
      Apply placeable edit
    </button>
  ),
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

function localFile(name: string, marker: number, text = ""): File {
  return {
    name,
    size: 1,
    type: name.endsWith(".glb")
      ? "model/gltf-binary"
      : name.endsWith(".json")
        ? "application/json"
        : "text/plain",
    lastModified: marker,
    arrayBuffer: async () => new Uint8Array([marker]).buffer,
    text: async () => text,
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

function placeableAuthoringJson() {
  const sourceSha256 = "a".repeat(64);
  return JSON.stringify({
    schemaVersion: 1,
    inspection: {
      schemaVersion: 1,
      sourceSha256,
      renderNodeCount: 0,
      primitiveCount: 0,
      connectedComponentCount: 0,
      nodes: [],
    },
    document: {
      schemaVersion: 1,
      sourceSha256,
      elements: [],
    },
  });
}

function fullNativeSourceInspectionJson() {
  const value = JSON.parse(sourceInspectionJson()) as {
    ir: { skins: unknown[]; animations: unknown[] };
    report: { inventory: { skinCount: number; animationCount: number } };
  };
  value.ir.skins = [{ jointNodeIds: [] }];
  value.ir.animations = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((name, id) => ({
    id,
    name,
    durationSeconds: 1,
    samplers: [],
    channels: [],
  }));
  value.report.inventory.skinCount = 1;
  value.report.inventory.animationCount = 42;
  return JSON.stringify(value);
}

function singleIdleSkinnedSourceInspectionJson() {
  const value = JSON.parse(sourceInspectionJson()) as {
    ir: { skins: unknown[]; animations: unknown[] };
    report: {
      inventory: {
        skinCount: number;
        jointReferenceCount: number;
        animationCount: number;
      };
    };
  };
  value.ir.skins = [{ jointNodeIds: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11] }];
  value.ir.animations = [{
    id: 0,
    name: "cpause1",
    durationSeconds: 1,
    samplers: [],
    channels: [],
  }];
  value.report.inventory.skinCount = 1;
  value.report.inventory.jointReferenceCount = 11;
  value.report.inventory.animationCount = 1;
  return JSON.stringify(value);
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

function placeableBuiltResponse(requestId: string): StudioWorkerResponse {
  const hash = (character: string) => character.repeat(64);
  const resources = [
    { container: "HAK", role: "PLACEABLES_2DA", resref: "placeables", resourceType: 2017, byteLength: 11, sha256: hash("d") },
    { container: "HAK", role: "MODEL", resref: "m2a_s1_plc_ped", resourceType: 2002, byteLength: 2, sha256: hash("b") },
    { container: "HAK", role: "TEXTURE", resref: "m2a_s1_plc_tex", resourceType: 3, byteLength: 3, sha256: hash("c") },
    { container: "MOD", role: "MODULE_INFO", resref: "module", resourceType: 2014, byteLength: 5, sha256: hash("3") },
    { container: "MOD", role: "FACTIONS", resref: "repute", resourceType: 2038, byteLength: 5, sha256: hash("4") },
    { container: "MOD", role: "AREA", resref: "m2a_s1_plc_ar", resourceType: 2012, byteLength: 5, sha256: hash("5") },
    { container: "MOD", role: "AREA_COMMENTS", resref: "m2a_s1_plc_ar", resourceType: 2046, byteLength: 5, sha256: hash("6") },
    { container: "MOD", role: "AREA_INSTANCES", resref: "m2a_s1_plc_ar", resourceType: 2023, byteLength: 5, sha256: hash("8") },
    { container: "MOD", role: "PLACEABLE_BLUEPRINT", resref: "m2a_s1_plc_utp", resourceType: 2044, byteLength: 5, sha256: hash("e") },
    { container: "MOD", role: "PLACEABLE_PALETTE", resref: "placeablepalcus", resourceType: 2030, byteLength: 5, sha256: hash("f") },
  ];
  const reportJson = JSON.stringify({
    schemaVersion: 1,
    status: "OFFLINE_ADMISSION_PASSED",
    profile: "STATIC_PLACEABLE",
    componentStatuses: {
      mdl: "passed", twoDa: "passed", utp: "passed", gitGic: "passed",
      palette: "passed", package: "passed", proof: "not_tested",
    },
    moduleFileName: "m2a_s1_plc_mod.mod",
    moduleDisplayName: "Meshy2Aurora S1 Placeable Proof",
    areaResref: "m2a_s1_plc_ar",
    areaName: "Meshy2Aurora S1 Ritual Pedestal",
    hakFileName: "m2a_s1_plc_hak.hak",
    modelResref: "m2a_s1_plc_ped",
    textureResref: "m2a_s1_plc_tex",
    blueprintResref: "m2a_s1_plc_utp",
    objectTag: "m2a_s1_ritual_pedestal",
    appearanceRow: { value: 16500 },
    placement: { x: 10, y: 14.5, z: 0, bearing: 0 },
    sourceModelSha256: hash("0"),
    mdlSha256: hash("b"),
    textureSha256: hash("c"),
    placeables2daSha256: hash("d"),
    utpSha256: hash("e"),
    itpSha256: hash("f"),
    gitSha256: hash("1"),
    gicSha256: hash("2"),
    hakSha256: hash("a"),
    moduleSha256: hash("7"),
    hakResourceCount: 3,
    moduleResourceCount: 7,
    modelVisibility: "not_tested",
    proofCompleteness: "missing",
    paletteCompleteness: "custom_itp_emitted",
    collisionCompleteness: "pwk_not_implemented",
    resources,
  });
  const withName = (value: WorkerArtifact, fileName: string) => ({ ...value, fileName });
  return {
    requestId,
    ok: true,
    type: "PLACEABLE_PACKAGE_BUILT",
    reportJson,
    readbackJson: JSON.stringify({
      schemaVersion: 1,
      format: "nwn1-binary-mdl",
      nodeTree: { roots: [{ offset: 12, number: 1, name: "m2a_s1_plc_ped", controllers: [], children: [] }] },
      animations: [],
      diagnostics: [],
    }),
    artifacts: [
      withName(artifact("placeable-package-hak", "HAK", [1, 2, 3], hash("a")), "m2a_s1_plc_hak.hak"),
      withName(artifact("placeable-model-mdl", "MODEL", [1, 2], hash("b")), "m2a_s1_plc_ped.mdl"),
      withName(artifact("placeable-proof-module", "MODULE", [4, 5, 6, 7], hash("7")), "m2a_s1_plc_mod.mod"),
      withName(
        artifact("placeable-report-json", "JSON_REPORT", [...new TextEncoder().encode(reportJson)], hash("9")),
        "placeable-materialization-report.json",
      ),
    ],
  };
}

function tileBuiltResponse(requestId: string): StudioWorkerResponse {
  const hash = (character: string) => character.repeat(64);
  const reportJson = JSON.stringify({
    schemaVersion: 1,
    status: "ready_for_owner_proof",
    profile: "TileStaticV1",
    moduleFileName: "m2a_tile_static_v1.mod",
    moduleDisplayName: "Meshy2Aurora Tile Static V1",
    areaResref: "m2atilearea",
    areaName: "M2A Tile Static 2x2",
    areaSize: [2, 2],
    areaTileCount: 4,
    entryPosition: [5, 5, 0],
    hakFileName: "m2a_tile_static_v1.hak",
    hakResref: "m2atilestv1",
    tilesetResref: "m2atilesetv1",
    tileId: 0,
    modelResref: "m2atilemdl1",
    wokResref: "m2atilemdl1",
    textureResref: "m2atiletex1",
    imageMapResref: "m2atilemap1",
    walkmeshClassToken: "msb01",
    surfaceId: 3,
    modelTriangleCount: 12,
    wokTriangleCount: 8,
    aabbEntryCount: 15,
    mdlSha256: hash("b"),
    wokSha256: hash("c"),
    setSha256: hash("d"),
    textureSha256: hash("e"),
    imageMapSha256: hash("f"),
    hakSha256: hash("a"),
    moduleSha256: hash("7"),
    hakResourceCount: 5,
    moduleResourceCount: 5,
    modelVisibility: "not_tested",
    proofCompleteness: "missing",
    navigationSpawnWalkable: "offline_verified",
    navigationSeamWalkable: "offline_verified",
    resources: [],
  });
  const modelReadbackJson = JSON.stringify({
    schemaVersion: 1,
    format: "nwn1-binary-mdl",
    nodeTree: { roots: [{ offset: 12, number: 1, name: "m2atilemdl1", controllers: [], children: [] }] },
    animations: [],
    diagnostics: [],
  });
  const wokReadbackJson = JSON.stringify({ schemaVersion: 1, surfaceId: 3 });
  const setReadbackJson = JSON.stringify({ sections: [] });
  const withName = (value: WorkerArtifact, fileName: string) => ({ ...value, fileName });
  const jsonArtifact = (id: string, fileName: string, json: string, marker: string) =>
    withName(artifact(id, "JSON_REPORT", [...new TextEncoder().encode(json)], hash(marker)), fileName);
  return {
    requestId,
    ok: true,
    type: "TILE_PACKAGE_BUILT",
    reportJson,
    modelReadbackJson,
    wokReadbackJson,
    setReadbackJson,
    artifacts: [
      withName(artifact("tile-package-hak", "HAK", [1], hash("a")), "m2a_tile_static_v1.hak"),
      withName(artifact("tile-proof-module", "MODULE", [2], hash("7")), "m2a_tile_static_v1.mod"),
      withName(artifact("tile-model-mdl", "MODEL", [3], hash("b")), "m2atilemdl1.mdl"),
      withName(artifact("tile-navigation-wok", "WOK", [4], hash("c")), "m2atilemdl1.wok"),
      withName(artifact("tile-tileset-set", "SET", [5], hash("d")), "m2atilesetv1.set"),
      withName(artifact("tile-texture-tga", "TEXTURE", [6], hash("e")), "m2atiletex1.tga"),
      withName(artifact("tile-image-map-tga", "TEXTURE", [7], hash("f")), "m2atilemap1.tga"),
      jsonArtifact("tile-report-json", "tile-materialization-report.json", reportJson, "1"),
      jsonArtifact("tile-model-readback-json", "tile-model-readback.json", modelReadbackJson, "2"),
      jsonArtifact("tile-wok-readback-json", "tile-wok-readback.json", wokReadbackJson, "3"),
      jsonArtifact("tile-set-readback-json", "tile-set-readback.json", setReadbackJson, "4"),
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

async function renderApp(
  strict = false,
  options: {
    meshyBridge?: MeshyBridgeClient;
    meshyLabEnabled?: boolean;
    tileTargetEnabled?: boolean;
  } = {},
) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  const app = <App {...options} />;
  await act(async () => root.render(strict ? <StrictMode>{app}</StrictMode> : app));
  return container;
}

async function driveToAnimationMapping(
  container: HTMLElement,
  sourceJson = sourceInspectionJson(),
  eventAuthoringJson?: string,
) {
  const [sourceInput, appearanceInput, animationEventsInput] = Array.from(
    container.querySelectorAll<HTMLInputElement>('input[type="file"]'),
  );
  await selectFile(sourceInput, localFile("source.glb", 1));
  await selectFile(appearanceInput, localFile("appearance.2da", 2));
  if (eventAuthoringJson !== undefined) {
    await selectFile(
      animationEventsInput,
      localFile("animation-events.json", 3, eventAuthoringJson),
    );
  }
  const worker = FakeWorker.instances.at(-1)!;
  const sourceRequest = worker.requests.filter((request) => request.type === "INSPECT_SOURCE").at(-1)!;
  const appearanceRequest = worker.requests.filter((request) => request.type === "INSPECT_APPEARANCE").at(-1)!;
  await act(async () => {
    worker.emit({ requestId: sourceRequest.requestId, ok: true, type: "SOURCE_INSPECTED", ingestJson: sourceJson });
    worker.emit({ requestId: appearanceRequest.requestId, ok: true, type: "APPEARANCE_INSPECTED", inspectionJson: appearanceInspectionJson() });
    await Promise.resolve();
  });
  await act(async () => button(container, "Continue to Inspect")?.click());
  await act(async () => button(container, "Continue to Build")?.click());
  await settle();
  return { sourceInput, worker };
}

async function driveToBuild(
  container: HTMLElement,
  sourceJson = fullNativeSourceInspectionJson(),
  eventAuthoringJson?: string,
) {
  const { sourceInput, worker } = await driveToAnimationMapping(
    container,
    sourceJson,
    eventAuthoringJson,
  );
  const generated = button(container, "Use generated Base 42");
  await act(async () => {
    if (generated && !generated.disabled) generated.click();
    else button(container, "Apply safe suggestions")?.click();
  });
  await settle();
  const validation = worker.requests
    .filter((request) => request.type === "VALIDATE_CREATURE_ANIMATION_MAPPING")
    .at(-1)!;
  await act(async () => {
    worker.emit({
      requestId: validation.requestId,
      ok: true,
      type: "CREATURE_ANIMATION_MAPPING_VALIDATED",
      validationJson: JSON.stringify({
        schemaVersion: 1,
        status: "READY",
        mappedBaseSlotCount: 42,
        reviewCount: 0,
        blockingCount: 0,
        customAnimationCount: 0,
        authoringFingerprintSha256: "f".repeat(64),
        diagnostics: [],
      }),
      resolutionJson: JSON.stringify({
        schemaVersion: 1,
        ok: true,
        validation: null,
        mapping: null,
      }),
      catalogJson: JSON.stringify(getDirectCreatureBaseCatalogV1()),
    });
    await Promise.resolve();
  });
  await act(async () => button(container, "Continue to Build")?.click());
  await act(async () => button(container, "Build Package")?.click());
  await settle();
  return { sourceInput, worker, build: worker.requests.find((request) => request.type === "BUILD_MODEL_PACKAGE")! };
}

async function driveToPlaceableBuild(container: HTMLElement) {
  const [sourceInput, appearanceInput] = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="file"]'));
  await selectFile(sourceInput, localFile("source.glb", 1));
  await selectFile(appearanceInput, localFile("placeables.2da", 2));
  const worker = FakeWorker.instances.at(-1)!;
  const sourceRequest = worker.requests.filter((request) => request.type === "INSPECT_SOURCE").at(-1)!;
  const appearanceRequest = worker.requests.filter((request) => request.type === "INSPECT_APPEARANCE").at(-1)!;
  await act(async () => {
    worker.emit({
      requestId: sourceRequest.requestId,
      ok: true,
      type: "SOURCE_INSPECTED",
      ingestJson: sourceInspectionJson(),
      placeableAuthoringJson: placeableAuthoringJson(),
    });
    worker.emit({ requestId: appearanceRequest.requestId, ok: true, type: "APPEARANCE_INSPECTED", inspectionJson: appearanceInspectionJson() });
    await Promise.resolve();
  });
  await act(async () => button(container, "Continue to Inspect")?.click());
  await act(async () => button(container, "Apply placeable edit")?.click());
  await act(async () => button(container, "Continue to Build")?.click());
  await act(async () => button(container, "Build Package")?.click());
  await settle();
  return {
    worker,
    build: worker.requests.find((request) => request.type === "BUILD_PLACEABLE_PACKAGE")!,
  };
}

async function driveToTileBuild(container: HTMLElement) {
  const tileTarget = container.querySelector<HTMLInputElement>(
    'input[name="conversion-target"][value="TILE"]',
  );
  await act(async () => tileTarget?.click());
  const [sourceInput] = Array.from(
    container.querySelectorAll<HTMLInputElement>('input[type="file"]'),
  );
  expect(sourceInput).toBeDefined();
  await selectFile(sourceInput, localFile("source.glb", 1));
  const worker = FakeWorker.instances.at(-1)!;
  const sourceRequest = worker.requests.filter((request) => request.type === "INSPECT_SOURCE").at(-1)!;
  await act(async () => {
    worker.emit({
      requestId: sourceRequest.requestId,
      ok: true,
      type: "SOURCE_INSPECTED",
      ingestJson: sourceInspectionJson(),
    });
    await Promise.resolve();
  });
  await act(async () => button(container, "Continue to Inspect")?.click());
  await act(async () => button(container, "Continue to Build")?.click());
  await act(async () => button(container, "Build Package")?.click());
  await settle();
  return {
    worker,
    build: worker.requests.find((request) => request.type === "BUILD_TILE_PACKAGE")!,
  };
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

  it("hides the unfinished Tile target by default", async () => {
    const container = await renderApp();
    expect(container.querySelector('input[name="conversion-target"][value="TILE"]')).toBeNull();
    expect(container.querySelector('option[value="TILE"]')).toBeNull();
    expect(container.textContent).toContain("choose Creature or Placeable");
  });

  it("keeps the legacy static M0 route behind the required Base 42 mapping gate", async () => {
    const container = await renderApp();
    const { worker } = await driveToAnimationMapping(container, sourceInspectionJson());
    expect(container.querySelector("h1")?.textContent).toBe("Creature Animation Mapping");
    expect(button(container, "Continue to Build")?.disabled).toBe(true);
    expect(worker.requests.find((request) => request.type === "BUILD_MODEL_PACKAGE"))
      .toBeUndefined();
  });

  it("offers Tile authoring and reviews the Worker package with WOK/AABB bindings", async () => {
    const container = await renderApp(false, { tileTargetEnabled: true });
    const { build, worker } = await driveToTileBuild(container);
    expect(build).toMatchObject({ type: "BUILD_TILE_PACKAGE" });
    if (build.type !== "BUILD_TILE_PACKAGE") throw new Error("tile request was not emitted");
    expect(JSON.parse(build.optionsJson)).toMatchObject({
      schemaVersion: 1,
      interior: false,
      terrainName: "Grass",
      surface: "GRASS",
    });
    await act(async () => {
      worker.emit(tileBuiltResponse(build.requestId));
      await Promise.resolve();
    });
    expect(container.querySelector("#tile-review-heading")?.textContent).toBe("TileStaticV1 Details");
    expect(container.textContent).toContain("Render triangles");
    expect(container.textContent).toContain("WOK triangles");
    expect(container.textContent).toContain("AABB entries");
    expect(container.textContent).toContain("m2atilemdl1.wok");
    expect(container.textContent).toContain("m2a_tile_static_v1.mod");
    expect(container.querySelector('[aria-label="Tile preview overlays"]')).not.toBeNull();
  });

  it("routes an exact 42-name skinned source through the authored H1 V4 lane", async () => {
    const container = await renderApp();
    const { build } = await driveToBuild(container, fullNativeSourceInspectionJson());
    expect(build.packageLane).toBe("H1_SKINNED_FULL_42_AUTHORED");
    if (build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "H1_SKINNED_FULL_42_AUTHORED") {
      throw new Error("authored model request unavailable");
    }
    expect(JSON.parse(build.animationAuthoringJson).assignments).toHaveLength(42);
  });

  it("routes a one-idle skinned Meshy humanoid through authored procedural assignments", async () => {
    const container = await renderApp();
    const { build } = await driveToBuild(container, singleIdleSkinnedSourceInspectionJson());
    expect(build.packageLane).toBe("H1_SKINNED_FULL_42_AUTHORED");
    if (build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "H1_SKINNED_FULL_42_AUTHORED") {
      throw new Error("authored model request unavailable");
    }
    expect(JSON.parse(build.animationAuthoringJson).assignments)
      .toEqual(expect.arrayContaining([
        expect.objectContaining({ sourceKind: "PROCEDURAL", targetSlot: "cpause1" }),
      ]));
  });

  it("routes caller-owned event JSON alongside the authored H1 V4 document", async () => {
    const container = await renderApp();
    const eventAuthoringJson = JSON.stringify({ schemaVersion: 1, clips: [] });
    const { build } = await driveToBuild(
      container,
      fullNativeSourceInspectionJson(),
      eventAuthoringJson,
    );
    expect(build.packageLane).toBe("H1_SKINNED_FULL_42_AUTHORED");
    if (build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "H1_SKINNED_FULL_42_AUTHORED") {
      throw new Error("eventful model request unavailable");
    }
    expect(build.eventAuthoringJson).toBe(eventAuthoringJson);
  });

  it("routes placeables.2da through the explicit placeable lane and shows offline-only evidence", async () => {
    const container = await renderApp();
    const { build, worker } = await driveToPlaceableBuild(container);
    expect(build.paletteId).toBe(7);
    expect(JSON.parse(build.identityJson).modelResref).toBe("m2a_s1_plc_ped");
    expect(JSON.parse(build.authoringJson ?? "{}").elements).toEqual([
      expect.objectContaining({ marker: "edited-in-placeable-editor" }),
    ]);
    await act(async () => {
      worker.emit(placeableBuiltResponse(build.requestId));
      await Promise.resolve();
    });

    expect(container.querySelector("#placeable-review-heading")?.textContent)
      .toBe("Meshy2Aurora S1 Placeable Proof");
    expect(container.textContent).toContain("Owner visual proof not performed");
    expect(container.textContent).toContain("16500");
    expect(container.textContent).not.toContain("Aurora compatible");
    expect(container.querySelector('[aria-label="Canonical Worker artifact downloads"]')?.textContent)
      .toContain("m2a_s1_plc_hak.hak");
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
    expect(container.querySelector('[aria-label="Conversion workflow"]')).toBeNull();
    expect(container.querySelector('[aria-label="Debug Drawer"]')).toBeNull();
    expect(container.querySelector(".studio-header")).toBeNull();
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
    await act(async () => button(container, "Generate model")?.click());
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
