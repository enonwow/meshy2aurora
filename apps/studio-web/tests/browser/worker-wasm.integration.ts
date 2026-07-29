import { afterEach, describe, expect, it } from "vitest";
import { userEvent } from "vitest/browser";
import { createElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import sourceUrl from "../.generated/owned-package/generated/source.glb?url";
import fullNative42SourceUrl from "../.generated/owned-full42-package/generated/source.glb?url";
import proceduralHumanoidSourceUrl from "@m2a-canonical-repository/sample-3d/h2-clockwork-sentinel-1500/source.glb?url";
import ownerPlaceableSourceUrl from "@m2a-canonical-repository/sample-3d/s1-placeable-ritual-pedestal-1500/source.glb?url";
import appearanceUrl from "../fixtures/appearance.2da?url";
import placeablesUrl from "../fixtures/placeables.2da?url";
import { buildM7PayloadEnvelope } from "../../src/features/m7/envelope";
import { projectCanonicalResult } from "../../src/features/results/projectCanonicalResult";
import { projectPlaceableResult } from "../../src/features/results/projectPlaceableResult";
import { projectTileResult } from "../../src/features/results/projectTileResult";
import { projectCanonicalReadback } from "../../src/features/results/projectReadback";
import { StudioWorkerClient } from "../../src/worker/client";
import { App } from "../../src/App";
import {
  createInMemoryProjectDatabaseV1,
  createMeshy2AuroraProjectV1,
  projectFileReferenceV1,
  reviseMeshy2AuroraProjectV1,
  serializeMeshy2AuroraProjectV1,
} from "../../src/features/project";

const clients: StudioWorkerClient[] = [];
const roots: Root[] = [];
const OWNER_H2_SHA256 =
  "f8cf0af21c8143a62b64c490a81dd2855ad3c3f9865922e3854f84b714dec3a3";
const OWNER_PLACEABLE_SHA256 =
  "dad22a5c3490242458cb7a81e50c53e265abf75886f57f6bd8a770938c2f7372";
const COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1 = [
  ["ca1slashl", "hit"],
  ["ca1slashr", "hit"],
  ["ca1stab", "hit"],
  ["ca1stab", "snd_footstep"],
  ["ccastout", "cast"],
  ["ccloseh", "hit"],
  ["cclosel", "hit"],
  ["ccturnr", "snd_footstep"],
  ["ccwalkb", "snd_footstep"],
  ["ccwalkf", "snd_footstep"],
  ["ccwalkl", "snd_footstep"],
  ["ccwalkr", "snd_footstep"],
  ["cdamagel", "snd_footstep"],
  ["cdamager", "snd_footstep"],
  ["cdamages", "snd_footstep"],
  ["cdodgelr", "snd_footstep"],
  ["cdodges", "snd_footstep"],
  ["ckdbck", "snd_hitground"],
  ["creach", "hit"],
  ["creach", "snd_footstep"],
  ["crun", "snd_footstep"],
  ["ctaunt", "snd_footstep"],
  ["cwalk", "snd_footstep"],
] as const;

function commonNativeEventAuthoringJson(): string {
  const clips = new Map<string, Array<{ timeSeconds: number; name: string }>>();
  for (const [clipName, name] of COMMON_NATIVE_DIRECT_CREATURE_EVENT_PAIRS_V1) {
    const events = clips.get(clipName) ?? [];
    events.push({ timeSeconds: 0.5, name });
    clips.set(clipName, events);
  }
  return JSON.stringify({
    schemaVersion: 1,
    clips: [...clips].map(([clipName, events]) => ({ clipName, events })),
  });
}

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

function asStaticPlaceable(glb: ArrayBuffer): ArrayBuffer {
  const input = new Uint8Array(glb);
  const view = new DataView(glb);
  const jsonLength = view.getUint32(12, true);
  const jsonEnd = 20 + jsonLength;
  const root = JSON.parse(new TextDecoder().decode(input.slice(20, jsonEnd))) as {
    skins: unknown[];
    animations: unknown[];
    scenes: Array<{ nodes: number[] }>;
    nodes: Array<Record<string, unknown>>;
    meshes: Array<{ primitives: Array<{ attributes: Record<string, unknown> }> }>;
  };
  root.skins = [];
  root.animations = [];
  root.scenes[0].nodes = [0];
  root.nodes = [{ name: "placeable-source-root", mesh: 0 }];
  delete root.meshes[0].primitives[0].attributes.JOINTS_0;
  delete root.meshes[0].primitives[0].attributes.WEIGHTS_0;
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
  it("materializes the owned single-idle H2 through the procedural 42-state Worker lane", async () => {
    const source = await fixtureFile(
      proceduralHumanoidSourceUrl,
      "h2-clockwork-sentinel-1500.glb",
      "model/gltf-binary",
    );
    const appearance = await fixtureFile(appearanceUrl, "appearance.2da", "text/plain");
    const sourceGlb = await source.arrayBuffer();
    const appearanceTwoDa = await appearance.arrayBuffer();
    expect(sourceGlb.byteLength).toBe(8_234_708);
    expect(await sha256(sourceGlb)).toBe(OWNER_H2_SHA256);
    const client = new StudioWorkerClient();
    clients.push(client);

    const initialized = await client.request({ requestId: "integration-init", type: "INITIALIZE" });
    expect(initialized).toMatchObject({ ok: true, type: "INITIALIZED" });

    const response = await client.request(
      {
        requestId: "integration-procedural-humanoid-build",
        type: "BUILD_MODEL_PACKAGE",
        sourceGlb,
        appearanceTwoDa,
        packageLane: "SKINNED_PROCEDURAL_HUMANOID_42",
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
    const moduleArtifact = response.artifacts.find((artifact) => artifact.kind === "MODULE");
    const reportArtifact = response.artifacts.find((artifact) => artifact.artifactId === "report-json");
    expect(hak).toBeDefined();
    expect(moduleArtifact).toBeDefined();
    expect(reportArtifact).toBeDefined();
    expect(reportArtifact?.fileName).toBe(
      `${moduleArtifact!.fileName.slice(0, -4)}-inspection.json`,
    );
    expect(hak?.provenance).toBe("M2A_WASM_WORKER");
    expect(new TextDecoder().decode(hak!.bytes.slice(0, 8))).toBe("HAK V1.0");
    expect(hak?.byteLength).toBe(hak?.bytes.byteLength);
    expect(hak?.sha256).toBe(await sha256(hak!.bytes));

    const report = JSON.parse(response.reportJson) as {
      geometry?: {
        triangleCount?: number;
        activeJointCount?: number;
        outputSegmentDeformation?: string;
      };
      texture?: { width?: number; height?: number };
      animationCompleteness?: {
        profile?: string;
        requiredClipCount?: number;
        explicitClipCount?: number;
        proceduralClipCount?: number;
        fallbackAliasCount?: number;
        complete?: boolean;
      };
      animationBehavior?: { behaviorCandidateEligible?: boolean };
      animationEventConformance?: { requiredPairCount?: number; complete?: boolean };
      skinAnimationConformance?: { requiredClipCount?: number; complete?: boolean };
    };
    const summary = JSON.parse(response.summaryJson) as {
      status?: string;
      inputGlb?: { byteLength?: number };
    };
    const manifest = JSON.parse(response.manifestJson) as {
      packageManifest?: { resources?: unknown[] };
    };
    const readback = JSON.parse(response.readbackJson) as {
      nodeTree?: { roots?: Array<{ name?: string; controllers?: unknown[] }> };
      animations?: Array<{
        name?: string;
        animationType?: number;
        nodeTree?: { roots?: unknown[] };
      }>;
    };
    expect(report.geometry?.triangleCount).toBeGreaterThan(1);
    expect(report.geometry?.activeJointCount).toBeGreaterThan(1);
    expect(report.geometry?.outputSegmentDeformation).toBe("SKIN");
    expect(report.animationCompleteness).toMatchObject({
      profile: "FULL_NATIVE42_PROCEDURAL_HUMANOID_V1",
      requiredClipCount: 42,
      explicitClipCount: 1,
      proceduralClipCount: 41,
      fallbackAliasCount: 0,
      complete: true,
    });
    expect(report.animationBehavior?.behaviorCandidateEligible).toBe(true);
    expect(report.animationEventConformance).toMatchObject({
      requiredPairCount: 23,
      complete: true,
    });
    expect(report.skinAnimationConformance).toMatchObject({
      requiredClipCount: 5,
      complete: true,
    });
    expect(summary.status).toBe("M6_MODEL_PACKAGE_MATERIALIZED");
    expect(summary.inputGlb?.byteLength).toBe(source.size);
    expect(manifest.packageManifest?.resources).toHaveLength(3);
    expect(readback.nodeTree?.roots).toHaveLength(1);
    expect(readback.nodeTree?.roots?.[0]).toMatchObject({
      name: "m2a_m6p01",
      controllers: [],
    });
    expect(readback.animations).toHaveLength(42);
    expect(readback.animations?.every((animation) => animation.animationType === 5)).toBe(true);
    expect(readback.animations?.every(
      (animation) => animation.nodeTree?.roots?.length === 1,
    )).toBe(true);
    expect(new TextDecoder().decode(reportArtifact!.bytes)).toBe(response.reportJson);
    const snapshot = projectCanonicalResult(
      response.reportJson,
      response.summaryJson,
      response.manifestJson,
      response.artifacts,
    );
    expect(snapshot.status).toBe("M6_MODEL_PACKAGE_MATERIALIZED");
    expect(snapshot.geometry).toMatchObject({ joints: report.geometry?.activeJointCount, deformation: "SKIN" });
    expect(snapshot.resrefs).toMatchObject({ model: "m2a_m6p01", texture: "m2a_m6t01" });
    expect(snapshot.hak.entryCount).toBe(3);
    expect(projectCanonicalReadback(response.readbackJson).nodeTree.roots.length).toBeGreaterThan(0);
    const repeatedSourceGlb = await fetchBytes(proceduralHumanoidSourceUrl);
    const repeatedAppearanceTwoDa = await fetchBytes(appearanceUrl);
    const repeated = await client.request(
      {
        requestId: "integration-procedural-humanoid-build-repeat",
        type: "BUILD_MODEL_PACKAGE",
        sourceGlb: repeatedSourceGlb,
        appearanceTwoDa: repeatedAppearanceTwoDa,
        packageLane: "SKINNED_PROCEDURAL_HUMANOID_42",
      },
      [repeatedSourceGlb, repeatedAppearanceTwoDa],
    );
    expect(repeated).toMatchObject({ ok: true, type: "MODEL_PACKAGE_BUILT" });
    if (!repeated.ok || repeated.type !== "MODEL_PACKAGE_BUILT") {
      throw new Error("repeated real H2 Worker package did not complete");
    }
    const artifactIdentities = (
      artifacts: typeof response.artifacts,
    ) => artifacts.map(({ fileName, byteLength, sha256: artifactSha256 }) => ({
      fileName,
      byteLength,
      sha256: artifactSha256,
    })).sort((left, right) => left.fileName.localeCompare(right.fileName));
    expect(artifactIdentities(repeated.artifacts))
      .toEqual(artifactIdentities(response.artifacts));
    console.info("E7_OWNER_H2_CREATURE_E2E_V1", JSON.stringify({
      schemaVersion: 1,
      sourceAssetId: "h2-clockwork-sentinel-1500",
      sourceSha256: OWNER_H2_SHA256,
      status: snapshot.status,
      modelResref: snapshot.resrefs.model,
      textureResref: snapshot.resrefs.texture,
      animationProfile: report.animationCompleteness?.profile,
      animationCount: readback.animations?.length,
      repeatBuildMatched: true,
      artifacts: response.artifacts.map(
        ({ artifactId, kind, fileName, byteLength, sha256: artifactSha256 }) => ({
          artifactId,
          kind,
          fileName,
          byteLength,
          sha256: artifactSha256,
        }),
      ),
      modelVisibility: "not_tested",
      proofCompleteness: "missing",
    }));
  }, 60_000);

  it("rejects the quarantined corrupt-draw H1 lane without producing artifacts", async () => {
    const sourceGlb = await fetchBytes(sourceUrl);
    const appearanceTwoDa = await fetchBytes(appearanceUrl);
    const client = new StudioWorkerClient();
    clients.push(client);

    await expect(client.request(
      {
        requestId: "legacy-corrupt-draw-lane-rejected",
        type: "BUILD_MODEL_PACKAGE",
        sourceGlb,
        appearanceTwoDa,
        packageLane: "H1_SKINNED" as never,
      },
      [sourceGlb, appearanceTwoDa],
    )).rejects.toThrow("Unsupported model package lane: H1_SKINNED");
  });

  it("carries caller-owned gameplay events through the real Full-42 WASM Worker lane", async () => {
    const sourceGlb = await fetchBytes(fullNative42SourceUrl);
    const appearanceTwoDa = await fetchBytes(appearanceUrl);
    const client = new StudioWorkerClient();
    clients.push(client);
    const response = await client.request(
      {
        requestId: "integration-full42-events",
        type: "BUILD_MODEL_PACKAGE",
        sourceGlb,
        appearanceTwoDa,
        packageLane: "H1_SKINNED_FULL_42_EVENTS",
        eventAuthoringJson: commonNativeEventAuthoringJson(),
      },
      [sourceGlb, appearanceTwoDa],
    );

    expect(response).toMatchObject({ ok: true, type: "MODEL_PACKAGE_BUILT" });
    if (!response.ok || response.type !== "MODEL_PACKAGE_BUILT") {
      throw new Error("real Worker did not return an eventful Full-42 package");
    }
    const report = JSON.parse(response.reportJson) as {
      animationEventConformance?: {
        requiredPairCount?: number;
        satisfiedPairCount?: number;
        totalEventCount?: number;
        complete?: boolean;
      };
      animationEventAuthoringCanonical?: {
        byteLength?: number;
        sha256?: string;
      };
    };
    expect(report.animationEventConformance).toMatchObject({
      requiredPairCount: 23,
      satisfiedPairCount: 23,
      totalEventCount: 23,
      complete: true,
    });
    expect(report.animationEventAuthoringCanonical?.byteLength).toBeGreaterThan(0);
    expect(report.animationEventAuthoringCanonical?.sha256).toMatch(/^[0-9a-f]{64}$/);
    const snapshot = projectCanonicalResult(
      response.reportJson,
      response.summaryJson,
      response.manifestJson,
      response.artifacts,
    );
    expect(snapshot.animationEventEvidence).toMatchObject({
      requiredPairCount: 23,
      satisfiedPairCount: 23,
      totalEventCount: 23,
      complete: true,
    });
    const readback = JSON.parse(response.readbackJson) as {
      animations?: Array<{ name?: string; events?: Array<{ time?: number; name?: string }> }>;
    };
    const cast = readback.animations
      ?.find(({ name }) => name === "ccastout")
      ?.events?.find(({ name }) => name === "cast");
    expect(cast).toEqual({ time: 0.5, name: "cast" });

    const malformedSource = await fetchBytes(fullNative42SourceUrl);
    const malformedAppearance = await fetchBytes(appearanceUrl);
    await expect(client.request(
      {
        requestId: "integration-full42-events-malformed",
        type: "BUILD_MODEL_PACKAGE",
        sourceGlb: malformedSource,
        appearanceTwoDa: malformedAppearance,
        packageLane: "H1_SKINNED_FULL_42_EVENTS",
        eventAuthoringJson: "{",
      },
      [malformedSource, malformedAppearance],
    )).rejects.toThrow("M6-ANIMATION-EVENT-AUTHORING-JSON");
  }, 30_000);

  it("materializes a static placeable through the real WASM Worker without a creature fork", async () => {
    const client = new StudioWorkerClient();
    clients.push(client);
    const inspectionSource = asStaticPlaceable(await fetchBytes(sourceUrl));
    const inspectionResponse = await client.request({
      requestId: "placeable-authoring-inspection",
      type: "INSPECT_SOURCE",
      sourceGlb: inspectionSource,
      target: "PLACEABLE",
    }, [inspectionSource]);
    expect(inspectionResponse).toMatchObject({ ok: true, type: "SOURCE_INSPECTED" });
    if (!inspectionResponse.ok || inspectionResponse.type !== "SOURCE_INSPECTED") {
      throw new Error("real Worker did not return placeable authoring inspection");
    }
    const authoring = JSON.parse(inspectionResponse.placeableAuthoringJson!) as {
      document: {
        elements: Array<{
          transform: {
            translation: [number, number, number];
            scale: [number, number, number];
          };
        }>;
      };
    };
    authoring.document.elements[0].transform.translation = [0.25, 0, 0];
    authoring.document.elements[0].transform.scale = [1.2, 1.2, 1.2];

    const sourceGlb = asStaticPlaceable(await fetchBytes(sourceUrl));
    const placeablesTwoDa = await fetchBytes(placeablesUrl);
    const projectIdentity = {
      schemaVersion: 1,
      projectId: "browser-synthetic-placeable",
      projectName: "Synthetic Ritual Pedestal",
      projectRevision: 3,
    };
    const response = await client.request({
      requestId: "placeable-build",
      type: "BUILD_PLACEABLE_PACKAGE",
      sourceGlb,
      placeablesTwoDa,
      projectIdentityJson: JSON.stringify(projectIdentity),
      placementJson: JSON.stringify({ x: 10, y: 14.5, z: 0, bearing: 0 }),
      paletteId: 7,
      authoringJson: JSON.stringify(authoring.document),
    }, [sourceGlb, placeablesTwoDa]);

    expect(sourceGlb.byteLength).toBe(0);
    expect(placeablesTwoDa.byteLength).toBe(0);
    expect(response).toMatchObject({ ok: true, type: "PLACEABLE_PACKAGE_BUILT" });
    if (!response.ok || response.type !== "PLACEABLE_PACKAGE_BUILT") {
      throw new Error("real Worker did not return a placeable package");
    }
    const result = projectPlaceableResult(response.reportJson, response.artifacts);
    expect(result).toMatchObject({
      status: "OFFLINE_ADMISSION_PASSED",
      profile: "STATIC_PLACEABLE",
      appearanceRow: 1,
      projectIdentity,
      modelVisibility: "not_tested",
      proofCompleteness: "missing",
      authoring: expect.objectContaining({
        renderableElementCount: 1,
        collisionElementCount: 1,
        shadowElementCount: 1,
      }),
    });
    expect(result.resources).toEqual(expect.arrayContaining([
      expect.objectContaining({ role: "MODEL", resourceType: 2002 }),
      expect.objectContaining({ role: "PLACEABLES_2DA", resourceType: 2017 }),
      expect.objectContaining({ role: "PLACEABLE_BLUEPRINT", resourceType: 2044 }),
      expect.objectContaining({ role: "PLACEABLE_PALETTE", resourceType: 2030 }),
    ]));
    expect(result.modelResref).toHaveLength(16);
    expect(result.blueprintResref).toBe(result.modelResref);
    expect(result.moduleFileName).toBe(`${result.modelResref}.mod`);
    expect(result.hakFileName).toBe(`${result.modelResref}.hak`);
    const hak = response.artifacts.find(({ kind }) => kind === "HAK");
    const module = response.artifacts.find(({ kind }) => kind === "MODULE");
    expect(new TextDecoder().decode(hak!.bytes.slice(0, 8))).toBe("HAK V1.0");
    expect(new TextDecoder().decode(module!.bytes.slice(0, 8))).toBe("MOD V1.0");
    expect(hak!.sha256).toBe(await sha256(hak!.bytes));
    expect(module!.sha256).toBe(await sha256(module!.bytes));
    await expectExactJsonArtifact(
      response.artifacts,
      `${result.modelResref}-placeable-materialization-report.json`,
      response.reportJson,
    );
  }, 30_000);

  it("builds the canonical owner Placeable sample with revision-owned evidence", async () => {
    const client = new StudioWorkerClient();
    clients.push(client);
    const inspectionSource = await fetchBytes(ownerPlaceableSourceUrl);
    expect(await sha256(inspectionSource))
      .toBe(OWNER_PLACEABLE_SHA256);
    const inspectionResponse = await client.request({
      requestId: "owner-placeable-inspection",
      type: "INSPECT_SOURCE",
      sourceGlb: inspectionSource,
      target: "PLACEABLE",
    }, [inspectionSource]);
    expect(inspectionResponse).toMatchObject({ ok: true, type: "SOURCE_INSPECTED" });
    if (!inspectionResponse.ok || inspectionResponse.type !== "SOURCE_INSPECTED") {
      throw new Error("owner Placeable inspection did not complete");
    }
    const bootstrap = JSON.parse(inspectionResponse.placeableAuthoringJson!) as {
      document: unknown;
    };
    const sourceGlb = await fetchBytes(ownerPlaceableSourceUrl);
    const placeablesTwoDa = await fetchBytes(placeablesUrl);
    const projectIdentity = {
      schemaVersion: 1,
      projectId: "owner-s1-placeable-ritual-pedestal",
      projectName: "Owner Ritual Pedestal",
      projectRevision: 1,
    };
    const response = await client.request({
      requestId: "owner-placeable-build",
      type: "BUILD_PLACEABLE_PACKAGE",
      sourceGlb,
      placeablesTwoDa,
      projectIdentityJson: JSON.stringify(projectIdentity),
      placementJson: JSON.stringify({ x: 10, y: 14.5, z: 0, bearing: 0 }),
      paletteId: 7,
      authoringJson: JSON.stringify(bootstrap.document),
    }, [sourceGlb, placeablesTwoDa]);

    expect(response).toMatchObject({ ok: true, type: "PLACEABLE_PACKAGE_BUILT" });
    if (!response.ok || response.type !== "PLACEABLE_PACKAGE_BUILT") {
      throw new Error("owner Placeable package did not complete");
    }
    const result = projectPlaceableResult(response.reportJson, response.artifacts);
    expect(result.projectIdentity).toEqual(projectIdentity);
    expect(result.modelVisibility).toBe("not_tested");
    expect(result.proofCompleteness).toBe("missing");
    expect(result.authoring).toEqual(expect.objectContaining({
      sourceSha256: OWNER_PLACEABLE_SHA256,
      renderableElementCount: expect.any(Number),
      collisionElementCount: expect.any(Number),
    }));
    expect(result.resources).toEqual(expect.arrayContaining([
      expect.objectContaining({ role: "MODEL", resourceType: 2002 }),
      expect.objectContaining({ role: "PLACEABLE_WALKMESH", resourceType: 2053 }),
      expect.objectContaining({ role: "PLACEABLE_BLUEPRINT", resourceType: 2044 }),
    ]));
    for (const artifact of response.artifacts) {
      expect(artifact.sha256).toBe(await sha256(artifact.bytes));
    }
    await expectExactJsonArtifact(
      response.artifacts,
      `${result.modelResref}-placeable-materialization-report.json`,
      response.reportJson,
    );

    const repeatedSourceGlb = await fetchBytes(ownerPlaceableSourceUrl);
    const repeatedPlaceablesTwoDa = await fetchBytes(placeablesUrl);
    const repeated = await client.request({
      requestId: "owner-placeable-build-repeat",
      type: "BUILD_PLACEABLE_PACKAGE",
      sourceGlb: repeatedSourceGlb,
      placeablesTwoDa: repeatedPlaceablesTwoDa,
      projectIdentityJson: JSON.stringify(projectIdentity),
      placementJson: JSON.stringify({ x: 10, y: 14.5, z: 0, bearing: 0 }),
      paletteId: 7,
      authoringJson: JSON.stringify(bootstrap.document),
    }, [repeatedSourceGlb, repeatedPlaceablesTwoDa]);
    expect(repeated).toMatchObject({ ok: true, type: "PLACEABLE_PACKAGE_BUILT" });
    if (!repeated.ok || repeated.type !== "PLACEABLE_PACKAGE_BUILT") {
      throw new Error("repeated owner Placeable package did not complete");
    }
    const artifactIdentities = (
      artifacts: typeof response.artifacts,
    ) => artifacts.map(({ fileName, byteLength, sha256: artifactSha256 }) => ({
      fileName,
      byteLength,
      sha256: artifactSha256,
    })).sort((left, right) => left.fileName.localeCompare(right.fileName));
    expect(artifactIdentities(repeated.artifacts))
      .toEqual(artifactIdentities(response.artifacts));

    console.info("E7_OWNER_PLACEABLE_E2E_V1", JSON.stringify({
      schemaVersion: 1,
      sourceAssetId: "s1-placeable-ritual-pedestal-1500",
      sourceSha256: OWNER_PLACEABLE_SHA256,
      projectIdentity,
      authoringSha256: result.authoring?.authoringSha256,
      status: result.status,
      moduleFileName: result.moduleFileName,
      moduleDisplayName: result.moduleDisplayName,
      areaName: result.areaName,
      hakFileName: result.hakFileName,
      modelResref: result.modelResref,
      textureResref: result.textureResref,
      blueprintResref: result.blueprintResref,
      objectTag: result.objectTag,
      appearanceRow: result.appearanceRow,
      placement: result.placement,
      artifacts: artifactIdentities(response.artifacts),
      repeatBuildMatched: true,
      modelVisibility: result.modelVisibility,
      proofCompleteness: result.proofCompleteness,
    }));
  }, 60_000);

  it("materializes TileStaticV1 with MDL, semantic AABB, WOK, SET, HAK and 2x2 MOD in the real Worker", async () => {
    const sourceGlb = asStaticPlaceable(await fetchBytes(sourceUrl));
    const client = new StudioWorkerClient();
    clients.push(client);
    const response = await client.request({
      requestId: "tile-build",
      type: "BUILD_TILE_PACKAGE",
      sourceGlb,
      optionsJson: JSON.stringify({
        schemaVersion: 1,
        identity: {
          moduleResref: "m2atilestv1",
          moduleFileName: "m2a_tile_static_v1.mod",
          moduleDisplayName: "Meshy2Aurora Tile Static V1",
          areaResref: "m2atilearea",
          areaName: "M2A Tile Static 2x2",
          hakResref: "m2atilestv1",
          hakFileName: "m2a_tile_static_v1.hak",
          tilesetResref: "m2atilesetv1",
          modelResref: "m2atilemdl1",
          textureResref: "m2atiletex1",
          imageMapResref: "m2atilemap1",
        },
        interior: false,
        terrainName: "Grass",
        surface: "GRASS",
      }),
    }, [sourceGlb]);

    expect(sourceGlb.byteLength).toBe(0);
    expect(response).toMatchObject({ ok: true, type: "TILE_PACKAGE_BUILT" });
    if (!response.ok || response.type !== "TILE_PACKAGE_BUILT") {
      throw new Error("real Worker did not return a tile package");
    }
    const result = projectTileResult(
      response.reportJson,
      response.wokReadbackJson,
      response.setReadbackJson,
      response.artifacts,
    );
    expect(result).toMatchObject({
      status: "ready_for_owner_proof",
      profile: "TileStaticV1",
      moduleFileName: "m2a_tile_static_v1.mod",
      moduleDisplayName: "Meshy2Aurora Tile Static V1",
      areaName: "M2A Tile Static 2x2",
      areaSize: [2, 2],
      areaTileCount: 4,
      entryPosition: [5, 5, 0],
      tileId: 0,
      modelResref: "m2atilemdl1",
      wokResref: "m2atilemdl1",
      walkmeshClassToken: "msb01",
      surfaceId: 3,
      modelVisibility: "not_tested",
      proofCompleteness: "missing",
      navigationSpawnWalkable: "offline_verified",
      navigationSeamWalkable: "offline_verified",
    });
    expect(result.modelTriangleCount).toBeGreaterThan(0);
    expect(result.wokTriangleCount).toBe(8);
    expect(result.aabbEntryCount).toBe(15);
    expect(response.artifacts).toEqual(expect.arrayContaining([
      expect.objectContaining({ kind: "HAK", fileName: "m2a_tile_static_v1.hak" }),
      expect.objectContaining({ kind: "MODULE", fileName: "m2a_tile_static_v1.mod" }),
      expect.objectContaining({ kind: "MODEL", fileName: "m2atilemdl1.mdl" }),
      expect.objectContaining({ kind: "WOK", fileName: "m2atilemdl1.wok" }),
      expect.objectContaining({ kind: "SET", fileName: "m2atilesetv1.set" }),
      expect.objectContaining({ kind: "TEXTURE", fileName: "m2atiletex1.tga" }),
    ]));
    const modelReadback = JSON.parse(response.modelReadbackJson) as {
      model?: { classification?: number };
      nodeTree?: {
        roots?: Array<{
          contentFlags?: number;
          aabb?: { entries?: unknown[] };
          children?: Array<{ contentFlags?: number; aabb?: { entries?: unknown[] } }>;
        }>;
      };
    };
    const readbackNodes = (modelReadback.nodeTree?.roots ?? []).flatMap(
      (node) => [node, ...(node.children ?? [])],
    );
    expect(modelReadback.model?.classification).toBe(2);
    expect(readbackNodes.some(({ contentFlags, aabb }) =>
      contentFlags === 0x221 && aabb?.entries?.length === 15
    )).toBe(true);
    expect(response.wokReadbackJson).toContain('"surfaceId":3');
    expect(response.setReadbackJson).toContain('"Model","m2atilemdl1"');
    expect(response.setReadbackJson).toContain('"WalkMesh","msb01"');
    await expectExactJsonArtifact(
      response.artifacts,
      "tile-materialization-report.json",
      response.reportJson,
    );
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

  it("keyboard-smokes Source through Download and resets stale output on replacement", async () => {
    const source = await fixtureFile(
      proceduralHumanoidSourceUrl,
      "h2-clockwork-sentinel-1500.glb",
      "model/gltf-binary",
    );
    const appearance = await fixtureFile(appearanceUrl, "appearance.2da", "text/plain");
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    root.render(createElement(App));
    await expect.poll(() => container.querySelector(
      'input[aria-label="Meshy model file"]',
    )).toBeTruthy();
    const currentWorkflowStep = () => container.querySelector(
      '[aria-current="step"] .workflow-stepper__label',
    )?.textContent;
    expect(currentWorkflowStep()).toBe("Source");
    const sourceInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Meshy model file"]',
    )!;
    const appearanceInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Base model table file"]',
    )!;
    const select = async (input: HTMLInputElement, file: File) => {
      Object.defineProperty(input, "files", { configurable: true, value: [file] });
      input.dispatchEvent(new Event("change", { bubbles: true }));
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    };
    await select(sourceInput, source);
    await select(appearanceInput, appearance);
    const findButton = (label: string) => Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find(({ textContent }) => textContent?.trim() === label);
    const activateWithKeyboard = async (label: string) => {
      const target = findButton(label);
      if (!target) throw new Error(`Missing keyboard target: ${label}`);
      target.focus();
      expect(document.activeElement).toBe(target);
      await userEvent.keyboard("{Enter}");
    };
    await expect.poll(() => findButton("Continue to Inspect")?.disabled).toBe(false);
    await activateWithKeyboard("Continue to Inspect");
    await expect.poll(currentWorkflowStep).toBe("Inspect");
    await expect.poll(() => findButton("Continue to Animation Mapping")?.disabled).toBe(false);
    await activateWithKeyboard("Continue to Animation Mapping");
    await expect.poll(currentWorkflowStep).toBe("Animation Mapping");
    await expect
      .poll(() => container.querySelector("#animation-mapping-title")?.textContent)
      .toBe("Creature Animation Mapping");
    await expect.poll(() => findButton("Use generated Base 42")?.disabled).toBe(false);
    await activateWithKeyboard("Use generated Base 42");
    await expect.poll(() => findButton("Continue to Build")?.disabled).toBe(false);
    await activateWithKeyboard("Continue to Build");
    await expect.poll(currentWorkflowStep).toBe("Build");
    await expect.poll(() => findButton("Build Package")?.disabled).toBe(false);
    expect(container.querySelector('[aria-label="Build project identity"]')?.textContent)
      .toContain("Project revision");
    await activateWithKeyboard("Build Package");
    await expect.poll(() => container.querySelector("#review-model-heading")?.textContent, { timeout: 20_000 }).toBe("Model Details");
    expect(currentWorkflowStep()).toBe("Review");

    const workspace = container.querySelector<HTMLElement>(".review-model")!;
    expect(workspace.textContent).toContain("Conversion Readiness");
    expect(workspace.textContent).toContain("Animation clips");
    expect(workspace.textContent).toContain("42");
    expect(workspace.textContent).toContain("23/23");
    expect(workspace.textContent).toContain("OPEN_M6");
    expect(workspace.textContent).toContain("Verified by binary readback");
    expect(workspace.textContent).toContain("Metrics available in both canonical snapshots");
    expect(container.querySelector('[aria-label="Canonical Worker artifact downloads"]'))
      .toBeNull();
    await activateWithKeyboard("Continue to Download");
    await expect.poll(() => container.querySelector("#download-step-heading")?.textContent)
      .toBe("Download");
    expect(currentWorkflowStep()).toBe("Download");
    expect(container.querySelector('[aria-label="Canonical Worker artifact downloads"]')?.textContent)
      .toContain("GENERATED ARTIFACTS");

    const replacementInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Meshy GLB model file"]',
    )!;
    await select(
      replacementInput,
      new File(
        [await fetchBytes(proceduralHumanoidSourceUrl)],
        "replacement.glb",
        { type: "model/gltf-binary" },
      ),
    );
    await expect.poll(() => container.querySelector("#review-model-heading")).toBeNull();
  }, 30_000);

  it("routes the canonical owner Placeable through Source, Inspect, Build, Review and Download", async () => {
    const source = await fixtureFile(
      ownerPlaceableSourceUrl,
      "s1-placeable-ritual-pedestal-1500.glb",
      "model/gltf-binary",
    );
    const placeables = await fixtureFile(placeablesUrl, "placeables.2da", "text/plain");
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    root.render(createElement(App));

    await expect.poll(() => container.querySelector(
      'input[name="conversion-target"][value="PLACEABLE"]',
    )).toBeTruthy();
    container.querySelector<HTMLInputElement>(
      'input[name="conversion-target"][value="PLACEABLE"]',
    )!.click();

    const select = async (label: string, file: File) => {
      const input = container.querySelector<HTMLInputElement>(
        `input[aria-label="${label}"]`,
      )!;
      Object.defineProperty(input, "files", { configurable: true, value: [file] });
      input.dispatchEvent(new Event("change", { bubbles: true }));
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    };
    await select("Meshy model file", source);
    await select("Base model table file", placeables);

    const findButton = (label: string) => Array.from(
      container.querySelectorAll<HTMLButtonElement>("button"),
    ).find(({ textContent }) => textContent?.trim() === label);
    await expect.poll(() => findButton("Continue to Inspect")?.disabled).toBe(false);
    findButton("Continue to Inspect")!.click();
    await expect.poll(() => findButton("Continue to Build")?.disabled).toBe(false);
    expect(findButton("Continue to Animation Mapping")).toBeUndefined();
    expect(container.querySelector("#animation-mapping-title")).toBeNull();

    findButton("Continue to Build")!.click();
    await expect.poll(() => container.querySelector("#build-step-heading")?.textContent)
      .toBe("Build");
    const workflowText = container.querySelector(
      ".studio-shell__workflow-rail",
    )?.textContent ?? "";
    expect(workflowText).not.toContain("Animation Mapping");
    expect(container.querySelector(".studio-header__context")?.textContent)
      .toBe("Placeable · Build");
    findButton("Build Package")!.click();
    await expect.poll(
      () => container.querySelector("#placeable-review-heading")?.textContent,
      { timeout: 30_000 },
    ).toContain("Placeable");
    expect(container.querySelector('[aria-label="Built project identity"]')?.textContent)
      .toContain("Revision");
    expect(container.textContent).toContain("Owner visual proof not performed");
    findButton("Continue to Download")!.click();
    await expect.poll(() => container.querySelector("#download-step-heading")?.textContent)
      .toBe("Download");
    expect(container.querySelector('[aria-label="Canonical Worker artifact downloads"]')?.textContent)
      .toContain("Download manifest");
    expect(container.textContent).toContain("owner proof pending");
    expect(container.textContent).toContain("revision");
  }, 60_000);
});
