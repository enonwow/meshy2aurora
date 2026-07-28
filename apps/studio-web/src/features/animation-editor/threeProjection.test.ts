import { Object3D } from "three";
import { describe, expect, it } from "vitest";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import {
  applyEditorPoseAtTimeV1,
  beginBoneTransformGestureV1,
  cancelBoneTransformGestureV1,
  commitBoneTransformGestureV1,
  projectAuthoredClipToThreeV1,
  projectDopeSheetRowsV1,
  updateBoneTransformGestureV1,
} from "./threeProjection";

const clip: AuthoredAnimationClipV1 = {
  id: "clip",
  name: "custom_test",
  kind: "MOTION",
  status: "VALID",
  source: {
    kind: "BLANK_POSE",
    sourceRevision: "a".repeat(64),
    sourceClipName: null,
    sourceClipFingerprint: null,
    proceduralTemplate: null,
  },
  lengthSeconds: 1,
  transitionSeconds: 0.1,
  animationRoot: "root",
  tracks: [{
    id: "track",
    targetNodeId: 1,
    path: "TRANSLATION",
    interpolation: "LINEAR",
    keyframes: [
      { id: "k0", timeSeconds: 0, value: [0, 0, 0] },
      { id: "k1", timeSeconds: 1, value: [2, 0, 0] },
    ],
  }],
  events: [],
  revision: 1,
};
const rig = [{
  id: 1,
  name: "hand",
  parentId: null,
  translation: [0, 0, 0],
  rotation: [0, 0, 0, 1],
}] as const;

describe("Three editor projection", () => {
  it("projects and applies one shared playhead time", () => {
    const projected = projectAuthoredClipToThreeV1(clip, rig);
    expect(projected.duration).toBe(1);
    expect(projected.tracks).toHaveLength(1);
    const root = new Object3D();
    const hand = new Object3D();
    hand.name = "hand";
    root.add(hand);
    applyEditorPoseAtTimeV1(root, clip, rig, 0.5);
    expect(hand.position.x).toBeCloseTo(1);
    expect(projectDopeSheetRowsV1(clip, rig)[0]).toMatchObject({
      targetNodeId: 1,
      label: "hand",
    });
  });

  it("commits one gesture and cancels without a document delta", () => {
    const begun = beginBoneTransformGestureV1({
      nodeId: 1,
      path: "TRANSLATION",
      value: [0, 0, 0],
    });
    const updated = updateBoneTransformGestureV1(begun, [1, 0, 0]);
    expect(commitBoneTransformGestureV1(updated)).toMatchObject({
      value: [1, 0, 0],
      changed: true,
    });
    expect(cancelBoneTransformGestureV1(updated)).toBeNull();
  });
});
