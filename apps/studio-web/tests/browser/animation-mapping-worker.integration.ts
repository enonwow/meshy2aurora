import { afterEach, describe, expect, it } from "vitest";
import sourceUrl from "../.generated/owned-full42-package/generated/source.glb?url";
import appearanceUrl from "../fixtures/appearance.2da?url";
import { FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 } from "../../src/features/source/directCreatureAnimationProfile";
import { projectCanonicalResult } from "../../src/features/results/projectCanonicalResult";
import { projectCanonicalReadback } from "../../src/features/results/projectReadback";
import { StudioWorkerClient } from "../../src/worker/client";

const clients: StudioWorkerClient[] = [];

async function fetchBytes(url: string): Promise<ArrayBuffer> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`fixture fetch failed: ${response.status} ${url}`);
  }
  return response.arrayBuffer();
}

async function sha256(bytes: ArrayBuffer): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function completeAuthoring(sourceRevision: string) {
  return {
    schemaVersion: 1,
    profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
    modelType: "S",
    sourceRevision,
    authoringRevision: 7,
    assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((targetSlot) => ({
      targetSlot,
      sourceKind: "SOURCE_CLIP",
      sourceClipName: targetSlot,
      customAnimationId: null,
      provenance: {
        provider: "SOURCE_GLB",
        assetId: "generated-owned-full42-source",
        ownership: "USER_OWNED",
      },
    })),
    fallbacks: [],
    customAnimations: [{
      id: "custom-wave",
      name: "wave",
      playback: "ONE_SHOT",
      sourceClipName: "cpause1",
      phases: [],
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "generated-owned-full42-source",
        ownership: "USER_OWNED",
      },
    }],
  };
}

afterEach(() => {
  while (clients.length) clients.pop()?.dispose();
});

describe("authored Creature animation mapping through the real Worker and WASM", () => {
  it("validates Base 42, builds the frozen mapping and readbacks the exact outputs", async () => {
    const sourceForIdentity = await fetchBytes(sourceUrl);
    const sourceRevision = await sha256(sourceForIdentity);
    const authoring = completeAuthoring(sourceRevision);
    const authoringJson = JSON.stringify(authoring);
    const client = new StudioWorkerClient();
    clients.push(client);

    const initialized = await client.request({
      requestId: "animation-mapping-init",
      type: "INITIALIZE",
    });
    expect(initialized).toMatchObject({ ok: true, type: "INITIALIZED" });

    const validation = await client.validateCreatureAnimationMapping(
      authoringJson,
      "animation-mapping-validate",
    );
    expect(validation).toMatchObject({
      ok: true,
      type: "CREATURE_ANIMATION_MAPPING_VALIDATED",
    });
    if (
      !validation.ok
      || validation.type !== "CREATURE_ANIMATION_MAPPING_VALIDATED"
    ) {
      throw new Error("real Worker did not validate Creature animation authoring");
    }
    expect(JSON.parse(validation.catalogJson)).toHaveLength(42);
    expect(JSON.parse(validation.validationJson)).toMatchObject({
      schemaVersion: 1,
      status: "READY",
      mappedBaseSlotCount: 42,
      blockingCount: 0,
      reviewCount: 0,
      customAnimationCount: 1,
    });
    const resolutionEnvelope = JSON.parse(validation.resolutionJson) as {
      ok: boolean;
      mapping: {
        authoringRevision: number;
        authoringFingerprintSha256: string;
        baseAnimations: Array<{ targetSlot: string }>;
        customAnimations: Array<{ outputClipNames: string[] }>;
      };
    };
    expect(resolutionEnvelope.ok).toBe(true);
    const resolution = resolutionEnvelope.mapping;
    expect(resolution.authoringRevision).toBe(7);
    expect(resolution.authoringFingerprintSha256).toMatch(/^[0-9a-f]{64}$/);
    expect(resolution.baseAnimations.map(({ targetSlot }) => targetSlot))
      .toEqual(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1);
    expect(resolution.customAnimations).toEqual([
      expect.objectContaining({ outputClipNames: ["wave"] }),
    ]);

    const sourceGlb = await fetchBytes(sourceUrl);
    const appearanceTwoDa = await fetchBytes(appearanceUrl);
    const projectIdentity = {
      schemaVersion: 1,
      projectId: "browser-authored-project",
      projectName: "Browser authored project",
      projectRevision: 7,
    } as const;
    const built = await client.buildAuthoredCreatureModelPackage(
      sourceGlb,
      appearanceTwoDa,
      authoringJson,
      JSON.stringify(projectIdentity),
      undefined,
      "animation-mapping-build",
    );
    expect(sourceGlb.byteLength).toBe(0);
    expect(appearanceTwoDa.byteLength).toBe(0);
    expect(built).toMatchObject({ ok: true, type: "MODEL_PACKAGE_BUILT" });
    if (!built.ok || built.type !== "MODEL_PACKAGE_BUILT") {
      throw new Error("real Worker did not build the authored Creature package");
    }

    const snapshot = projectCanonicalResult(
      built.reportJson,
      built.summaryJson,
      built.manifestJson,
      built.artifacts,
    );
    expect(snapshot.projectIdentity).toEqual(projectIdentity);
    expect(built.artifacts.map(({ fileName }) => fileName)).toEqual(
      expect.arrayContaining([
        expect.stringMatching(/^m2[a-f0-9]{14}\.hak$/),
        expect.stringMatching(/^m2[a-f0-9]{14}\.mdl$/),
        expect.stringMatching(/^m2[a-f0-9]{14}\.mod$/),
        expect.stringMatching(/^m2[a-f0-9]{14}-inspection\.json$/),
        expect.stringMatching(/^m2[a-f0-9]{14}-conversion-manifest\.json$/),
        expect.stringMatching(/^m2[a-f0-9]{14}-summary\.json$/),
      ]),
    );
    expect(snapshot.animationMappingEvidence).toMatchObject({
      authoringRevision: 7,
      authoringFingerprintSha256: resolution.authoringFingerprintSha256,
      conformance: {
        status: "READY",
        expectedBaseSlotCount: 42,
        materializedBaseSlotCount: 42,
        expectedCustomClipNames: ["wave"],
        materializedCustomClipNames: ["wave"],
      },
    });
    expect(snapshot.animationMappingEvidence?.baseAnimations.map(
      ({ targetSlot }) => targetSlot,
    )).toEqual(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1);

    const readback = projectCanonicalReadback(built.readbackJson);
    expect(readback.animations.map(({ name }) => name)).toEqual([
      ...FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
      "wave",
    ]);
  }, 60_000);
});
