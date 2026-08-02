import { createElement } from "react";
import { flushSync } from "react-dom";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { cdp } from "vitest/browser";
import realH1Url from
  "@m2a-canonical-repository/sample-3d/h1-humanoid-1500/source.glb?url";
import voidCrystalKnightUrl from
  "@m2a-canonical-repository/sample-3d/void-crystal-knight-h1-v1/source.glb?url";
import appearanceUrl from "../fixtures/appearance.2da?url";
import {
  createBlankPoseClipV1,
  createHumanoidSwordSlashFromSourceClipV1,
  createProceduralTemplateClipV1,
} from "../../src/features/animation-editor/editing";
import {
  beginBoneTransformGestureV1,
  commitBoneTransformGestureV1,
  projectAuthoredClipToThreeV1,
  updateBoneTransformGestureV1,
} from "../../src/features/animation-editor/threeProjection";
import {
  ANIMATION_STUDIO_PRODUCT_LIMITS_V1,
} from "../../src/features/animation-studio/limits";
import type {
  AnimationStudioDocumentV1,
} from "../../src/features/animation-studio/types";
import {
  serializeAnimationStudioDocumentV1,
} from "../../src/features/animation-studio/schema";
import { SourceViewport } from "../../src/features/preview/SourceViewport";
import { StudioWorkerClient } from "../../src/worker/client";

const clients: StudioWorkerClient[] = [];
const roots: Root[] = [];

async function fetchBytes(url: string): Promise<ArrayBuffer> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`fixture fetch failed: ${response.status} ${url}`);
  }
  return response.arrayBuffer();
}

async function measureFrameIntervals(durationMs: number) {
  const intervals: number[] = [];
  const startedAt = performance.now();
  let previousFrameAt = startedAt;
  await new Promise<void>((resolve) => {
    const sample = (frameAt: number) => {
      intervals.push(frameAt - previousFrameAt);
      previousFrameAt = frameAt;
      if (frameAt - startedAt >= durationMs) resolve();
      else requestAnimationFrame(sample);
    };
    requestAnimationFrame(sample);
  });
  return intervals;
}

function percentile(values: readonly number[], fraction: number) {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.min(
    Math.max(Math.ceil(sorted.length * fraction) - 1, 0),
    sorted.length - 1,
  )] ?? Infinity;
}

afterEach(async () => {
  while (clients.length) clients.pop()?.dispose();
  while (roots.length) roots.pop()?.unmount();
  await Promise.resolve();
  document.body.replaceChildren();
});

describe("Animation Studio measured product limits on canonical real H1", () => {
  it("plays the exact Void Crystal Knight on the Three clock without React frame renders or GLB reloads", async () => {
    const sourceBytes = await fetchBytes(voidCrystalKnightUrl);
    const sourceFile = new File([sourceBytes], "source.glb", {
      type: "model/gltf-binary",
    });
    const arrayBufferSpy = vi.spyOn(sourceFile, "arrayBuffer");
    const client = new StudioWorkerClient();
    clients.push(client);
    await client.request({
      requestId: "smooth-vck-init",
      type: "INITIALIZE",
    });
    const inspected = await client.inspectEditableAnimationSource(
      sourceBytes.slice(0),
      undefined,
      "smooth-vck-inspect",
    );
    expect(inspected).toMatchObject({
      ok: true,
      type: "EDITABLE_ANIMATION_SOURCE_INSPECTED",
    });
    if (!inspected.ok || inspected.type !== "EDITABLE_ANIMATION_SOURCE_INSPECTED") {
      throw new Error("Void Crystal Knight output rig inspection failed");
    }
    const inspection = JSON.parse(inspected.inspectionJson) as {
      sourceRevision: string;
      rig: Array<{
        id: number;
        name: string;
        parentId: number | null;
        translation: [number, number, number];
        rotation: [number, number, number, number];
      }>;
    };
    const animationRoot = inspection.rig.find(({ parentId }) => parentId === null)
      ?.name ?? inspection.rig[0]?.name;
    if (!animationRoot) throw new Error("Void Crystal Knight animation root is missing");
    const pose = createBlankPoseClipV1({
      id: "smooth-vck-pose",
      name: "smooth_vck_pose",
      sourceRevision: inspection.sourceRevision,
      animationRoot,
      rig: inspection.rig,
    });
    const clip = {
      ...createHumanoidSwordSlashFromSourceClipV1(pose, inspection.rig, 0),
      id: "smooth-vck-cleave",
      name: "m2a_voidcleave",
    };
    const trailNode = inspection.rig.at(-1)!;
    const motionVisualization = {
      schemaVersion: 1 as const,
      sourceRevision: inspection.sourceRevision,
      clipId: clip.id,
      clipRevision: clip.revision,
      trails: [{
        nodeId: trailNode.id,
        points: Array.from({ length: 90 }, (_, index) => ({
          timeSeconds: clip.lengthSeconds * index / 89,
          translation: [
            trailNode.translation[0] + Math.sin(index / 12) * 0.12,
            trailNode.translation[1],
            trailNode.translation[2],
          ] as const,
        })),
      }],
      onionPoses: [],
      fingerprintSha256: "f".repeat(64),
    };
    const container = document.createElement("div");
    container.style.width = "800px";
    container.style.height = "600px";
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    const updates: Array<{
      at: number;
      timeSeconds: number;
      playing: boolean;
    }> = [];
    let parentRenderCount = 0;
    const renderViewport = async (
      authoredClip: typeof clip,
      playing: boolean,
      timeSeconds: number,
    ) => {
      flushSync(() => root.render((() => {
        parentRenderCount += 1;
        return createElement(SourceViewport, {
          input: {
            provenance: "SOURCE",
            file: sourceFile,
            sourceSha256: inspection.sourceRevision,
          },
          authoredClip,
          authoredRig: inspection.rig,
          controlledAnimationTimeSeconds: timeSeconds,
          controlledAnimationPlayback: {
            playing,
            onUpdate: (snapshot) => updates.push({
              at: Date.now(),
              timeSeconds: snapshot.timeSeconds,
              playing: snapshot.playing,
            }),
          },
          motionVisualization,
        });
      })()));
      await Promise.resolve();
    };

    await renderViewport(clip, true, 0);
    await vi.waitFor(() => {
      expect(updates.some(({ playing }) => playing)).toBe(true);
    }, { timeout: 8_000, interval: 50 });
    const desktopFrameIntervals = await measureFrameIntervals(600);
    await vi.waitFor(() => {
      expect(updates.some(({ playing }) => !playing)).toBe(true);
    }, { timeout: 8_000, interval: 50 });
    const activeUpdates = updates.filter(({ playing }) => playing);
    expect(activeUpdates.length).toBeGreaterThanOrEqual(5);
    expect(activeUpdates.length).toBeLessThanOrEqual(16);
    expect(activeUpdates.every((entry, index) => (
      index === 0
      || entry.timeSeconds >= (activeUpdates[index - 1]?.timeSeconds ?? 0)
    ))).toBe(true);
    expect(percentile(desktopFrameIntervals, 0.95)).toBeLessThanOrEqual(25);
    expect(desktopFrameIntervals.filter((duration) => duration > 50).length
      / desktopFrameIntervals.length).toBeLessThan(0.05);
    expect(parentRenderCount).toBe(1);
    expect(arrayBufferSpy).toHaveBeenCalledOnce();

    const devtools = cdp();
    await devtools.send("Emulation.setCPUThrottlingRate", { rate: 4 });
    try {
      await renderViewport(clip, false, 0);
      updates.length = 0;
      await renderViewport(clip, true, 0);
      await vi.waitFor(() => {
        expect(updates.some(({ playing }) => playing)).toBe(true);
      }, { timeout: 8_000, interval: 50 });
      const throttledFrameIntervals = await measureFrameIntervals(600);
      expect(percentile(throttledFrameIntervals, 0.95)).toBeLessThanOrEqual(50);
    } finally {
      await devtools.send("Emulation.setCPUThrottlingRate", { rate: 1 });
    }

    await renderViewport({ ...clip, revision: clip.revision + 1 }, false, 0.5);
    await vi.waitFor(() => {
      expect(arrayBufferSpy).toHaveBeenCalledOnce();
    });
    expect(parentRenderCount).toBe(4);
  }, 30_000);

  it("records cold start, parse, preview and build under a 4x CPU laptop profile", async () => {
    const devtools = cdp();
    await devtools.send("Emulation.setCPUThrottlingRate", { rate: 4 });
    try {
      const client = new StudioWorkerClient();
      clients.push(client);
      const coldStartStartedAt = performance.now();
      await client.request({
        requestId: "studio-performance-init",
        type: "INITIALIZE",
      });
      const coldWorkerWasmStartMs = performance.now() - coldStartStartedAt;

    const parseStartedAt = performance.now();
    const inspected = await client.inspectEditableAnimationSource(
      await fetchBytes(realH1Url),
      undefined,
      "studio-performance-inspect-real-h1",
    );
    const parseMs = performance.now() - parseStartedAt;
    expect(inspected).toMatchObject({
      ok: true,
      type: "EDITABLE_ANIMATION_SOURCE_INSPECTED",
    });
    if (!inspected.ok || inspected.type !== "EDITABLE_ANIMATION_SOURCE_INSPECTED") {
      throw new Error("real H1 output rig inspection failed");
    }
    const inspection = JSON.parse(inspected.inspectionJson) as {
      sourceRevision: string;
      rig: Array<{
        id: number;
        name: string;
        parentId: number | null;
        translation: [number, number, number];
        rotation: [number, number, number, number];
      }>;
    };
    const animationRoot = inspection.rig.find(({ parentId }) => parentId === null)
      ?.name ?? inspection.rig[0]?.name;
    if (!animationRoot || inspection.rig.length === 0) {
      throw new Error("real H1 output rig is empty");
    }
    const clip = {
      ...createProceduralTemplateClipV1({
        id: "real-h1-bind-pose",
        name: "h1_bind_pose",
        sourceRevision: inspection.sourceRevision,
        animationRoot,
        rig: inspection.rig,
        lengthSeconds: 1,
      }, "ROOT_TRANSLATION_PULSE"),
      status: "VALID" as const,
    };
    const document: AnimationStudioDocumentV1 = {
      schemaVersion: 1,
      sourceRevision: inspection.sourceRevision,
      authoringRevision: 1,
      status: "VALID",
      authoredClips: [clip],
    };
    const documentBytes = new TextEncoder().encode(
      serializeAnimationStudioDocumentV1(document),
    ).byteLength;
    const documentJson = serializeAnimationStudioDocumentV1(document);

    const previewStartedAt = performance.now();
    const preview = await client.previewAuthoredAnimationClip(
      documentJson,
      clip.id,
      "studio-performance-preview-real-h1",
    );
    const previewMs = performance.now() - previewStartedAt;
    expect(preview).toMatchObject({
      ok: true,
      type: "AUTHORED_ANIMATION_CLIP_PREVIEWED",
    });

    const sourceGlb = await fetchBytes(realH1Url);
    const appearanceTwoDa = await fetchBytes(appearanceUrl);
    const buildStartedAt = performance.now();
    const built = await client.request({
      requestId: "studio-performance-build-real-h1",
      type: "BUILD_MODEL_PACKAGE",
      sourceGlb,
      appearanceTwoDa,
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_42",
      textureArtifactCleanup: false,
      identityJson: JSON.stringify({
        modelResref: "m2a_perfmdl1",
        textureResref: "m2a_perftex1",
        hakResref: "m2a_perfhak1",
        appearanceLabel: "M2A_PERFORMANCE_CREATURE_V1",
      }),
    }, [sourceGlb, appearanceTwoDa]);
    const buildMs = performance.now() - buildStartedAt;
    expect(built).toMatchObject({
      ok: true,
      type: "MODEL_PACKAGE_BUILT",
    });

    const projectionIterations = 100;
    const projectionStartedAt = performance.now();
    for (let index = 0; index < projectionIterations; index += 1) {
      const projected = projectAuthoredClipToThreeV1(clip, inspection.rig);
      expect(projected.tracks).toHaveLength(inspection.rig.length * 2);
    }
    const projectionElapsedMs = performance.now() - projectionStartedAt;
    const projectionMeanMs = projectionElapsedMs / projectionIterations;

    const gestureIterations = 50_000;
    const rootNode = inspection.rig[0]!;
    let gesture = beginBoneTransformGestureV1({
      nodeId: rootNode.id,
      path: "TRANSLATION",
      value: rootNode.translation,
    });
    const gestureStartedAt = performance.now();
    for (let index = 0; index < gestureIterations; index += 1) {
      gesture = updateBoneTransformGestureV1(gesture, [
        rootNode.translation[0] + (index % 100) / 10_000,
        rootNode.translation[1],
        rootNode.translation[2],
      ]);
    }
    const committed = commitBoneTransformGestureV1(gesture);
    const gestureElapsedMs = performance.now() - gestureStartedAt;

    const metrics = {
      asset: "sample-3d/h1-humanoid-1500/source.glb",
      profile: "HEADLESS_CHROMIUM_CPU_THROTTLE_4X",
      coldWorkerWasmStartMs,
      parseMs,
      previewMs,
      buildMs,
      rigNodeCount: inspection.rig.length,
      authoredTrackCount: clip.tracks.length,
      authoredKeyframeCount: clip.tracks.reduce(
        (total, track) => total + track.keyframes.length,
        0,
      ),
      documentBytes,
      projectionIterations,
      projectionElapsedMs,
      projectionMeanMs,
      gestureIterations,
      gestureElapsedMs,
      limits: ANIMATION_STUDIO_PRODUCT_LIMITS_V1,
    };
    console.info("ANIMATION_STUDIO_PERFORMANCE_V1", JSON.stringify(metrics));

    // These are broad regression ceilings, not benchmark claims. The measured
    // values are recorded in the implementation report after this browser run.
    expect(documentBytes).toBeLessThan(1_000_000);
    expect(coldWorkerWasmStartMs).toBeLessThan(15_000);
    expect(parseMs).toBeLessThan(15_000);
    expect(previewMs).toBeLessThan(8_000);
    expect(buildMs).toBeLessThan(60_000);
    expect(projectionMeanMs).toBeLessThan(50);
    expect(gestureElapsedMs).toBeLessThan(2_000);
    expect(committed.changed).toBe(true);
    expect(ANIMATION_STUDIO_PRODUCT_LIMITS_V1).toEqual({
      maxAuthoredClips: 64,
      maxKeyframesPerClip: 10_000,
      maxTotalKeyframes: 100_000,
      maxTimelineDomMarkers: 2_000,
      maxDurationSeconds: 86_400,
      maxAbsoluteTrackValue: 1_000_000,
    });
    } finally {
      await devtools.send("Emulation.setCPUThrottlingRate", { rate: 1 });
    }
  }, 120_000);
});
