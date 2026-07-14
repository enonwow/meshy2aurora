import { describe, expect, it } from "vitest";
import * as THREE from "three";
import { AnimationPlaybackRuntime, inventoryAnimationClips } from "./animationPlayback";

function positionClip(name = "walk") {
  return new THREE.AnimationClip(name, 1, [
    new THREE.NumberKeyframeTrack(".position[x]", [0, 1], [0, 2]),
  ]);
}

describe("animation clip inventory", () => {
  it("reports only clips present in the GLB payload", () => {
    const clips = [positionClip("idle"), positionClip("")];

    expect(inventoryAnimationClips(clips)).toEqual([
      { index: 0, name: "idle", durationSeconds: 1 },
      { index: 1, name: "Unnamed clip 2", durationSeconds: 1 },
    ]);
    expect(inventoryAnimationClips([])).toEqual([]);
  });
});

describe("AnimationPlaybackRuntime", () => {
  it("selects, plays, pauses, seeks and loops a real Three.js clip", () => {
    const root = new THREE.Group();
    const runtime = new AnimationPlaybackRuntime(root, [positionClip()]);

    expect(runtime.selectClip(0)).toMatchObject({ selectedClipIndex: 0, playing: false, durationSeconds: 1 });
    runtime.setPlaying(true);
    expect(runtime.update(0.5).timeSeconds).toBeCloseTo(0.5);
    expect(root.position.x).toBeCloseTo(1);

    runtime.setPlaying(false);
    runtime.update(0.25);
    expect(runtime.snapshot().timeSeconds).toBeCloseTo(0.5);

    expect(runtime.seek(0.75).timeSeconds).toBeCloseTo(0.75);
    runtime.setLoop(false);
    runtime.setPlaying(true);
    expect(runtime.update(1).playing).toBe(false);
    expect(runtime.snapshot().timeSeconds).toBeCloseTo(1);

    runtime.dispose();
    expect(runtime.snapshot()).toMatchObject({ selectedClipIndex: null, playing: false });
  });

  it("rejects an index not present in the loaded clip array", () => {
    const runtime = new AnimationPlaybackRuntime(new THREE.Group(), []);
    expect(() => runtime.selectClip(0)).toThrow(/unavailable/);
  });
});
