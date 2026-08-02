// @vitest-environment jsdom

import { StrictMode, act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { InMemoryMeshyBridgeClient, type MeshyBridgeClient } from "./features/meshy/bridge";
import { FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 } from "./features/source/directCreatureAnimationProfile";
import type { StudioWorkerRequest, StudioWorkerResponse, WorkerArtifact } from "./worker/types";

vi.mock("./features/preview/SceneViewport", () => ({
  SceneViewport: ({ provenance }: { provenance: string }) => <div data-testid={`viewport-${provenance}`} />,
}));
vi.mock("./features/placeable-authoring/PlaceableAuthoringEditor", () => ({
  PlaceableAuthoringEditor: ({
    bootstrap,
    textureBootstrap,
    onDocumentChange,
    onTextureSnapshotChange,
  }: {
    bootstrap: {
      document: {
        schemaVersion: 2;
        sourceSha256: string;
        elements: unknown[];
        collision: unknown;
      };
    };
    textureBootstrap: { document: unknown };
    onDocumentChange: (document: unknown) => void;
    onTextureSnapshotChange: (snapshot: unknown) => void;
  }) => (
    <button
      type="button"
      onClick={() => {
        onDocumentChange({
          ...bootstrap.document,
          elements: [{
            id: "mock-authored-element",
            marker: "edited-in-placeable-editor",
            source: { nodeId: 0, primitiveId: 0, componentIndex: 0 },
            flags: { includeInCollision: true },
            deleted: false,
          }],
        });
        onTextureSnapshotChange({
          document: textureBootstrap.document,
          files: new Map(),
          preview: "EDITED",
          selectedMaterialSlot: null,
        });
      }}
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
  const payload = text === ""
    ? new Uint8Array([marker])
    : new TextEncoder().encode(text);
  return {
    name,
    size: payload.byteLength,
    type: name.endsWith(".glb")
      ? "model/gltf-binary"
      : name.endsWith(".json")
        ? "application/json"
        : "text/plain",
    lastModified: marker,
    arrayBuffer: async () => payload.slice().buffer,
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
    schemaVersion: 2,
    inspection: {
      schemaVersion: 1,
      sourceSha256,
      renderNodeCount: 0,
      primitiveCount: 0,
      connectedComponentCount: 0,
      nodes: [],
    },
    document: {
      schemaVersion: 2,
      sourceSha256,
      elements: [],
      collision: {
        schemaVersion: 1,
        mode: "AUTO_RECTANGLE",
        coordinateSpace: "GLTF_SOURCE_XZ_METERS",
        paddingMeters: 0,
        vertices: [],
      },
    },
  });
}

function placeableTextureAuthoringJson() {
  const sourceSha256 = "a".repeat(64);
  return JSON.stringify({
    schemaVersion: 1,
    inspection: {
      schemaVersion: 1,
      sourceSha256,
      materials: [],
    },
    document: {
      schemaVersion: 1,
      sourceSha256,
      bindings: [],
    },
  });
}

function placeableCollisionJson() {
  return JSON.stringify({
    schemaVersion: 1,
    mode: "AUTO_RECTANGLE",
    sourceCoordinateSpace: "GLTF_SOURCE_XZ_METERS",
    outputCoordinateSpace: "AURORA_XY_METERS",
    inputVertices: [[-1, -1], [1, -1], [1, 1], [-1, 1]],
    sourceVertices: [[-1, -1], [1, -1], [-1, 1], [1, 1]],
    vertices: [[-1, -1], [1, -1], [-1, 1], [1, 1]],
    triangles: [[0, 1, 2], [1, 3, 2]],
    boundsMin: [-1, -1],
    boundsMax: [1, 1],
    surfaceId: 7,
    authoringSha256: "1".repeat(64),
    collisionSha256: "2".repeat(64),
    pwkSha256: "3".repeat(64),
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
  value.ir.skins = [{ jointNodeIds: [1, 2] }];
  value.ir.animations = [{
    id: 0,
    name: "cpause1",
    durationSeconds: 1,
    samplers: [],
    channels: [],
  }];
  value.report.inventory.skinCount = 1;
  value.report.inventory.jointReferenceCount = 2;
  value.report.inventory.animationCount = 1;
  return JSON.stringify(value);
}

function appearanceInspectionJson(sourceSha256 = "b".repeat(64)) {
  return JSON.stringify({
    schemaVersion: 1, format: "2DA", version: "V2.0", sourceSha256, byteLength: 1,
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
    conversion: { schemaVersion: 1, conversionEligible: true, policies: { basisStatus: "CREATURE_BASIS_V2_RESOLVED", assetForwardMapping: "GLTF_POSITIVE_Z_TO_AURORA_NEGATIVE_Y", orientationParity: "POSITIVE_PROPER_ROTATION_COMPOSITE_DETERMINANT", engineFacingProof: "OPEN_M6", uvRuntimeProof: "OPEN_M6" }, gates: [], diagnostics: [] },
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
    { container: "HAK", role: "PLACEABLE_WALKMESH", resref: "m2a_s1_plc_ped", resourceType: 2053, byteLength: 4, sha256: hash("9") },
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
      mdl: "passed", pwk: "passed", twoDa: "passed", utp: "passed", gitGic: "passed",
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
    pwkSha256: hash("9"),
    textureSha256: hash("c"),
    placeables2daSha256: hash("d"),
    utpSha256: hash("e"),
    itpSha256: hash("f"),
    gitSha256: hash("1"),
    gicSha256: hash("2"),
    hakSha256: hash("a"),
    moduleSha256: hash("7"),
    hakResourceCount: 4,
    moduleResourceCount: 7,
    modelVisibility: "not_tested",
    proofCompleteness: "missing",
    paletteCompleteness: "custom_itp_emitted",
    collisionCompleteness: "ascii_pwk_emitted_offline_readback_passed",
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

async function waitForWorkerRequest<T extends StudioWorkerRequest["type"]>(
  worker: FakeWorker,
  type: T,
): Promise<Extract<StudioWorkerRequest, { type: T }>> {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    const request = worker.requests.find((candidate) => candidate.type === type);
    if (request) return request as Extract<StudioWorkerRequest, { type: T }>;
    await act(async () => {
      await new Promise((resolve) => window.setTimeout(resolve, 10));
    });
  }
  throw new Error(`Timed out waiting for Worker request ${type}`);
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

function setSelectValue(element: HTMLSelectElement, value: string) {
  Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value")?.set?.call(element, value);
  element.dispatchEvent(new Event("change", { bubbles: true }));
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

async function driveToBuild(
  container: HTMLElement,
  sourceJson = sourceInspectionJson(),
  eventAuthoringJson?: string,
  appearanceJson = appearanceInspectionJson(),
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
    worker.emit({ requestId: appearanceRequest.requestId, ok: true, type: "APPEARANCE_INSPECTED", inspectionJson: appearanceJson });
    await Promise.resolve();
  });
  await act(async () => button(container, "Continue to Inspect")?.click());
  await act(async () => button(container, "Continue to Build")?.click());
  await act(async () => button(container, "Build Package")?.click());
  const build = await waitForWorkerRequest(worker, "BUILD_MODEL_PACKAGE");
  return { sourceInput, worker, build };
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
      placeableCollisionJson: placeableCollisionJson(),
      placeableTexturesJson: placeableTextureAuthoringJson(),
    });
    worker.emit({ requestId: appearanceRequest.requestId, ok: true, type: "APPEARANCE_INSPECTED", inspectionJson: appearanceInspectionJson() });
    await Promise.resolve();
  });
  await act(async () => button(container, "Continue to Inspect")?.click());
  await act(async () => button(container, "Apply placeable edit")?.click());
  await act(async () => {
    await new Promise((resolve) => window.setTimeout(resolve, 140));
  });
  const collisionRequest = worker.requests
    .filter((request) => request.type === "RESOLVE_PLACEABLE_COLLISION")
    .at(-1);
  if (!collisionRequest) throw new Error("placeable collision request missing");
  const textureRequest = worker.requests
    .filter((request) => request.type === "RESOLVE_PLACEABLE_TEXTURES")
    .at(-1);
  if (!textureRequest) throw new Error("placeable texture request missing");
  await act(async () => {
    worker.emit({
      requestId: collisionRequest.requestId,
      ok: true,
      type: "PLACEABLE_COLLISION_RESOLVED",
      collisionJson: placeableCollisionJson(),
    });
    worker.emit({
      requestId: textureRequest.requestId,
      ok: true,
      type: "PLACEABLE_TEXTURES_RESOLVED",
      texturesJson: JSON.stringify({
        schemaVersion: 1,
        sourceSha256: "a".repeat(64),
        authoringSha256: "4".repeat(64),
        alphaPolicy: "OPAQUE_ONLY",
        bindings: [],
        resources: [],
      }),
    });
    await Promise.resolve();
  });
  await act(async () => button(container, "Continue to Build")?.click());
  await act(async () => button(container, "Build Package")?.click());
  const build = await waitForWorkerRequest(worker, "BUILD_PLACEABLE_PACKAGE");
  return {
    worker,
    build,
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
  const build = await waitForWorkerRequest(worker, "BUILD_TILE_PACKAGE");
  return {
    worker,
    build,
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

  it("moves inspected local inputs through Build into a readback-verified review", async () => {
    const container = await renderApp();
    const { build, worker } = await driveToBuild(container);
    expect(build.packageLane).toBe("M0_STATIC_RIGID");
    await act(async () => { worker.emit(builtResponse(build.requestId)); await Promise.resolve(); });

    expect(container.querySelector("#review-model-heading")?.textContent).toBe("Model Details");
    expect(container.textContent).toContain("Verified by binary readback");
    expect(container.textContent).toContain("Conversion Readiness");
    expect(container.querySelector('[aria-label="Canonical Worker artifact downloads"]')?.textContent)
      .toContain("package-hak.bin");
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

  it("routes an exact 42-name source through the collision-free full H1 V4 lane", async () => {
    const container = await renderApp();
    const { build } = await driveToBuild(container, fullNativeSourceInspectionJson());
    expect(build.packageLane).toBe("H1_SKINNED_FULL_42");
    if (
      build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "H1_SKINNED_FULL_42"
    ) {
      throw new Error("full-native H1 request unavailable");
    }
    expect(JSON.parse(build.identityJson)).toMatchObject({
      modelResref: expect.stringMatching(/^cm[a-z2-7]{14}$/),
      textureResref: expect.stringMatching(/^ct[a-z2-7]{14}$/),
      hakResref: expect.stringMatching(/^ch[a-z2-7]{14}$/),
    });
    expect(JSON.parse(build.demoModuleIdentityJson)).toMatchObject({
      moduleResref: expect.stringMatching(/^cd[a-z2-7]{14}$/),
      areaResref: expect.stringMatching(/^ca[a-z2-7]{14}$/),
      hakResref: expect.stringMatching(/^ch[a-z2-7]{14}$/),
    });
    expect(build.skinAccessoryStabilization).toEqual({ mode: "AUTO" });
  });

  it("routes a one-idle skinned Meshy humanoid through the procedural 42-state lane", async () => {
    const container = await renderApp();
    const stabilization = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Detached accessory skinning"]',
    );
    expect(stabilization?.value).toBe("AUTO");
    const { build } = await driveToBuild(container, singleIdleSkinnedSourceInspectionJson());
    expect(build.packageLane).toBe("SKINNED_PROCEDURAL_HUMANOID_42");
    if (
      build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "SKINNED_PROCEDURAL_HUMANOID_42"
    ) {
      throw new Error("procedural product request unavailable");
    }
    const identity = JSON.parse(build.identityJson);
    expect(identity).toEqual({
      modelResref: expect.stringMatching(/^cm[a-z2-7]{14}$/),
      textureResref: expect.stringMatching(/^ct[a-z2-7]{14}$/),
      hakResref: expect.stringMatching(/^ch[a-z2-7]{14}$/),
      appearanceLabel: expect.stringMatching(/^M2A_CREATURE_V3_[A-Z2-7]{14}$/),
    });
    const suffix = identity.modelResref.slice(2);
    expect(build.skinAccessoryStabilization).toEqual({ mode: "AUTO" });
    expect(JSON.parse(build.demoModuleIdentityJson)).toEqual({
      moduleResref: `cd${suffix}`,
      areaResref: `ca${suffix}`,
      hakResref: `ch${suffix}`,
    });
    expect(build.demoCreatureResref).toBe(`cc${suffix}`);
  });

  it("passes the selected source-forward axis into the canonical Creature build", async () => {
    const container = await renderApp();
    const sourceForward = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Model front in source GLB"]',
    );
    expect(sourceForward?.value).toBe("POSITIVE_Z");
    await act(async () => setSelectValue(sourceForward!, "NEGATIVE_X"));

    const { build } = await driveToBuild(container, singleIdleSkinnedSourceInspectionJson());
    if (
      build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "SKINNED_PROCEDURAL_HUMANOID_42"
    ) {
      throw new Error("procedural product request unavailable");
    }
    expect(build.sourceForward).toBe("NEGATIVE_X");
  });

  it("routes an explicit accessory bone through the procedural build request", async () => {
    const container = await renderApp();
    const stabilization = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Detached accessory skinning"]',
    );
    expect(stabilization).not.toBeNull();
    await act(async () => setSelectValue(stabilization!, "SELECT_BONE"));
    const bone = container.querySelector<HTMLInputElement>(
      'input[aria-label="Accessory bone name"]',
    );
    expect(bone).not.toBeNull();
    await act(async () => setValue(bone!, "Spine02"));

    const { build } = await driveToBuild(container, singleIdleSkinnedSourceInspectionJson());
    if (
      build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "SKINNED_PROCEDURAL_HUMANOID_42"
    ) {
      throw new Error("procedural product request unavailable");
    }
    expect(build.skinAccessoryStabilization).toEqual({
      mode: "SELECT_BONE",
      selectedBoneName: "Spine02",
      componentBoneOverrides: [],
    });
    expect(JSON.parse(build.identityJson)).toEqual({
      modelResref: expect.stringMatching(/^cm[a-z2-7]{14}$/),
      textureResref: expect.stringMatching(/^ct[a-z2-7]{14}$/),
      hakResref: expect.stringMatching(/^ch[a-z2-7]{14}$/),
      appearanceLabel: expect.stringMatching(/^M2A_CREATURE_V3_[A-Z2-7]{14}$/),
    });
  });

  it("routes deterministic per-component accessory bone overrides without a global fallback", async () => {
    const container = await renderApp();
    const stabilization = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Detached accessory skinning"]',
    );
    expect(stabilization).not.toBeNull();
    await act(async () => setSelectValue(stabilization!, "SELECT_BONE"));
    const overrides = container.querySelector<HTMLTextAreaElement>(
      'textarea[aria-label="Accessory component bone overrides"]',
    );
    expect(overrides).not.toBeNull();
    await act(async () => setValue(overrides!, "0:4=Spine\n0:2=Spine02"));

    const { build } = await driveToBuild(container, singleIdleSkinnedSourceInspectionJson());
    if (
      build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "SKINNED_PROCEDURAL_HUMANOID_42"
    ) {
      throw new Error("procedural product request unavailable");
    }
    expect(build.skinAccessoryStabilization).toEqual({
      mode: "SELECT_BONE",
      componentBoneOverrides: [
        { segmentIndex: 0, componentIndex: 2, boneName: "Spine02" },
        { segmentIndex: 0, componentIndex: 4, boneName: "Spine" },
      ],
    });
  });

  it("changes every Creature artifact identity when the base appearance.2da changes", async () => {
    const firstContainer = await renderApp();
    const first = await driveToBuild(
      firstContainer,
      singleIdleSkinnedSourceInspectionJson(),
      undefined,
      appearanceInspectionJson("b".repeat(64)),
    );
    const secondContainer = await renderApp();
    const second = await driveToBuild(
      secondContainer,
      singleIdleSkinnedSourceInspectionJson(),
      undefined,
      appearanceInspectionJson("c".repeat(64)),
    );
    if (
      first.build.type !== "BUILD_MODEL_PACKAGE"
      || first.build.packageLane !== "SKINNED_PROCEDURAL_HUMANOID_42"
      || second.build.type !== "BUILD_MODEL_PACKAGE"
      || second.build.packageLane !== "SKINNED_PROCEDURAL_HUMANOID_42"
    ) {
      throw new Error("procedural product request unavailable");
    }
    const firstIdentity = JSON.parse(first.build.identityJson);
    const secondIdentity = JSON.parse(second.build.identityJson);
    expect(secondIdentity.modelResref).not.toBe(firstIdentity.modelResref);
    expect(secondIdentity.textureResref).not.toBe(firstIdentity.textureResref);
    expect(secondIdentity.hakResref).not.toBe(firstIdentity.hakResref);
    expect(second.build.demoModuleIdentityJson).not.toBe(first.build.demoModuleIdentityJson);
  });

  it("routes the explicit 100K Creature experiment through its full-package app lane", async () => {
    const container = await renderApp();
    const profile = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Creature conversion profile"]',
    );
    expect(profile).not.toBeNull();
    await act(async () => setSelectValue(profile!, "EXPERIMENTAL_P100K"));

    const { build, worker } = await driveToBuild(
      container,
      singleIdleSkinnedSourceInspectionJson(),
    );
    const sourceInspection = worker.requests
      .filter((request) => request.type === "INSPECT_SOURCE")
      .at(-1);
    expect(sourceInspection).toMatchObject({
      type: "INSPECT_SOURCE",
      creatureProfile: "EXPERIMENTAL_P100K",
    });
    expect(build.packageLane).toBe("SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT");
    if (
      build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT"
    ) {
      throw new Error("P100K application lane unavailable");
    }
    const identity = JSON.parse(build.identityJson);
    const token = identity.modelResref.slice(2);
    expect(identity).toEqual({
      modelResref: expect.stringMatching(/^pm[a-z2-7]{14}$/),
      textureResref: `pt${token}`,
      module: {
        moduleResref: `pd${token}`,
        areaResref: `pa${token}`,
        hakResref: `ph${token}`,
      },
      creatureResref: `pc${token}`,
    });
  });

  it("routes the explicit 300K Creature experiment through its own full-package app lane", async () => {
    const container = await renderApp();
    const profile = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Creature conversion profile"]',
    );
    expect(profile).not.toBeNull();
    await act(async () => setSelectValue(profile!, "EXPERIMENTAL_P300K"));
    const textureCleanup = container.querySelector<HTMLInputElement>(
      'input[aria-label="Repair texture artifacts"]',
    );
    expect(textureCleanup).not.toBeNull();
    await act(async () => textureCleanup?.click());

    const { build, worker } = await driveToBuild(
      container,
      singleIdleSkinnedSourceInspectionJson(),
    );
    const sourceInspection = worker.requests
      .filter((request) => request.type === "INSPECT_SOURCE")
      .at(-1);
    expect(sourceInspection).toMatchObject({
      type: "INSPECT_SOURCE",
      creatureProfile: "EXPERIMENTAL_P300K",
    });
    expect(build.packageLane).toBe("SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT");
    if (
      build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT"
    ) {
      throw new Error("P300K application lane unavailable");
    }
    expect(build.textureArtifactCleanup).toBe(true);
    const identity = JSON.parse(build.identityJson);
    const token = identity.modelResref.slice(2);
    expect(identity).toEqual({
      modelResref: expect.stringMatching(/^pm[a-z2-7]{14}$/),
      textureResref: `pt${token}`,
      module: {
        moduleResref: `pd${token}`,
        areaResref: `pa${token}`,
        hakResref: `ph${token}`,
      },
      creatureResref: `pc${token}`,
    });
  });

  it("routes caller-owned event JSON through the explicit full H1 V3 lane", async () => {
    const container = await renderApp();
    const eventAuthoringJson = JSON.stringify({ schemaVersion: 1, clips: [] });
    const { build } = await driveToBuild(
      container,
      fullNativeSourceInspectionJson(),
      eventAuthoringJson,
    );
    expect(build.packageLane).toBe("H1_SKINNED_FULL_42_EVENTS");
    if (build.type !== "BUILD_MODEL_PACKAGE"
      || build.packageLane !== "H1_SKINNED_FULL_42_EVENTS") {
      throw new Error("eventful model request unavailable");
    }
    expect(build.eventAuthoringJson).toBe(eventAuthoringJson);
  });

  it("binds exact animation event sidecar bytes into the generated artifact identity", async () => {
    const first = await driveToBuild(
      await renderApp(),
      fullNativeSourceInspectionJson(),
      JSON.stringify({ schemaVersion: 1, clips: [{ clip: "ca1slashl", event: "hit", time: 0.25 }] }),
    );
    const second = await driveToBuild(
      await renderApp(),
      fullNativeSourceInspectionJson(),
      JSON.stringify({ schemaVersion: 1, clips: [{ clip: "ca1slashl", event: "hit", time: 0.75 }] }),
    );
    if (
      first.build.type !== "BUILD_MODEL_PACKAGE"
      || first.build.packageLane !== "H1_SKINNED_FULL_42_EVENTS"
      || second.build.type !== "BUILD_MODEL_PACKAGE"
      || second.build.packageLane !== "H1_SKINNED_FULL_42_EVENTS"
    ) {
      throw new Error("eventful model request unavailable");
    }
    expect(JSON.parse(first.build.identityJson).modelResref)
      .not.toBe(JSON.parse(second.build.identityJson).modelResref);
  });

  it("routes placeables.2da through the explicit placeable lane and shows offline-only evidence", async () => {
    const container = await renderApp();
    const { build, worker } = await driveToPlaceableBuild(container);
    expect(build.paletteId).toBe(7);
    expect(build.experimentalAggressiveGeometryCleanup).toBe(false);
    expect(JSON.parse(build.identityJson).modelResref).toMatch(/^pm[0-9a-f]{8}$/);
    expect(JSON.parse(build.authoringJson ?? "{}").elements).toEqual([
      expect.objectContaining({ marker: "edited-in-placeable-editor" }),
    ]);
    expect(JSON.parse(build.textureAuthoringJson ?? "{}")).toMatchObject({
      schemaVersion: 1,
      bindings: [],
    });
    expect(build.texturePayloadBlob?.byteLength).toBe(0);
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
