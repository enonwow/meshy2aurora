import assert from "node:assert/strict";
import { test } from "node:test";
import { inspectGlbArtifactV1 } from "./glb-artifact-inspection.mjs";
import { createTestGlb } from "./test-glb-fixture.mjs";

test("inspects skeleton, clip channels, duration, and root motion", () => {
  const result = inspectGlbArtifactV1(createTestGlb());
  assert.equal(result.status, "PARSED");
  assert.deepEqual(result.geometry, {
    meshCount: 1,
    primitiveCount: 1,
    triangleCount: 1,
  });
  assert.equal(result.skeleton.jointCount, 2);
  assert.equal(result.skeleton.skinnedMeshNodeCount, 1);
  assert.equal(result.skeleton.skinnedPrimitiveCount, 1);
  assert.equal(result.skeleton.weightedVertexCount, 3);
  assert.match(result.skeleton.signatureSha256, /^[0-9a-f]{64}$/);
  assert.deepEqual(result.clipInventory.clips[0], {
    name: "attack",
    durationSeconds: 1,
    channelCount: 1,
    jointChannelCount: 1,
    samplerCount: 1,
    channelPaths: { translation: 1 },
    rootMotion: {
      assessed: true,
      rootTranslationChannelCount: 1,
      maximumTranslationDelta: 0.25,
      classification: "ROOT_MOTION",
    },
  });
});

test("rejects empty/header-only GLB instead of reporting a parsed artifact", () => {
  assert.throws(
    () => inspectGlbArtifactV1(
      new Uint8Array([0x67, 0x6c, 0x54, 0x46]),
    ),
    /header or JSON chunk is truncated/,
  );
});

test("rejects corrupt geometry even when the GLB container is well formed", () => {
  assert.throws(
    () => inspectGlbArtifactV1(createTestGlb({ corruptIndex: true })),
    /references a vertex outside POSITION/,
  );
});

test("rejects corrupt skin joint bindings in an otherwise valid GLB", () => {
  assert.throws(
    () => inspectGlbArtifactV1(createTestGlb({ corruptJoint: true })),
    /references a joint outside/,
  );
});

test("keeps an unused skin distinct from an actual weighted mesh binding", () => {
  const result = inspectGlbArtifactV1(createTestGlb({ bindSkin: false }));
  assert.equal(result.skeleton.jointCount, 2);
  assert.equal(result.skeleton.skinnedMeshNodeCount, 0);
  assert.equal(result.skeleton.weightedVertexCount, 0);
});

test("keeps a non-joint animation channel distinct from rig animation", () => {
  const result = inspectGlbArtifactV1(createTestGlb({ animateJoint: false }));
  assert.equal(result.clipInventory.clips[0].channelCount, 1);
  assert.equal(result.clipInventory.clips[0].jointChannelCount, 0);
});

test("rejects geometry that is not reachable from the active scene", () => {
  assert.throws(
    () => inspectGlbArtifactV1(createTestGlb({ renderInScene: false })),
    /active GLB scene contains no render geometry/,
  );
});

test("keeps structural no-skin and no-animation facts explicit", () => {
  const result = inspectGlbArtifactV1(createTestGlb({
    withSkin: false,
    withAnimation: false,
  }));
  assert.equal(result.status, "PARSED");
  assert.equal(result.skeleton.jointCount, 0);
  assert.equal(result.skeleton.skinnedMeshNodeCount, 0);
  assert.equal(result.clipInventory.clipCount, 0);
});
