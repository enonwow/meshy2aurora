import { Matrix4, Object3D, Quaternion, Vector3 } from "three";
import { describe, expect, it } from "vitest";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import {
  applyEditorPoseAtTimeV1,
  beginBoneTransformGestureV1,
  cancelBoneTransformGestureV1,
  commitBoneTransformGestureV1,
  projectAuthoredClipToThreeV1,
  projectDopeSheetRowsV1,
  projectThreeBoneTransformToAuthoredV1,
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

  it("rebases output-rig tracks onto the exact source GLB pose for viewport playback", () => {
    const sourceRoot = new Object3D();
    const hand = new Object3D();
    hand.name = "hand";
    hand.position.set(1, 2, 3);
    sourceRoot.add(hand);
    const outputBase = new Quaternion().setFromAxisAngle(
      new Vector3(0, 0, 1),
      Math.PI / 2,
    );
    const sourceTurn = outputBase.clone().multiply(
      new Quaternion().setFromAxisAngle(new Vector3(0, 1, 0), Math.PI / 2),
    );
    const basis = new Matrix4().set(
      1, 0, 0, 0,
      0, 0, 1, 0,
      0, 1, 0, 0,
      0, 0, 0, 1,
    );
    const toOutput = (source: Quaternion) => new Quaternion()
      .setFromRotationMatrix(
        basis.clone()
          .multiply(new Matrix4().makeRotationFromQuaternion(source))
          .multiply(basis),
      )
      .normalize();
    const outputBind = toOutput(outputBase);
    const outputTurn = toOutput(sourceTurn);
    const rebased = projectAuthoredClipToThreeV1({
      ...clip,
      tracks: [{
        ...clip.tracks[0]!,
        keyframes: [
          { id: "k0", timeSeconds: 0, value: [2, 6, 4] },
          { id: "k1", timeSeconds: 1, value: [4, 6, 4] },
        ],
      }, {
        id: "rotation",
        targetNodeId: 1,
        path: "ROTATION",
        interpolation: "LINEAR",
        keyframes: [
          {
            id: "r0",
            timeSeconds: 0,
            value: outputBind.toArray(),
          },
          {
            id: "r1",
            timeSeconds: 1,
            value: outputTurn.toArray(),
          },
        ],
      }],
    }, [{
      ...rig[0],
      translation: [2, 6, 4],
      rotation: [outputBind.x, outputBind.y, outputBind.z, outputBind.w],
    }], sourceRoot);

    expect(Array.from(rebased.tracks[0]!.values)).toEqual([1, 2, 3, 2, 2, 3]);
    expect(Array.from(rebased.tracks[1]!.values).slice(0, 4)).toEqual([
      0,
      0,
      expect.closeTo(Math.SQRT1_2, 6),
      expect.closeTo(Math.SQRT1_2, 6),
    ]);
    expect(Array.from(rebased.tracks[1]!.values).slice(4)).toEqual([
      expect.closeTo(-0.5, 6),
      expect.closeTo(0.5, 6),
      expect.closeTo(0.5, 6),
      expect.closeTo(0.5, 6),
    ]);
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

  it("projects a real Three transform gizmo result back to output-rig values", () => {
    const snapshot = {
      objectName: "hand",
      position: [2, 4, 3] as const,
      quaternion: [0, 0, Math.SQRT1_2, Math.SQRT1_2] as const,
      initialPosition: [1, 2, 3] as const,
      initialQuaternion: [0, 0, 0, 1] as const,
      sourceRestPosition: [1, 2, 3] as const,
    };
    const targetRig = {
      ...rig[0],
      translation: [2, 4, 6] as const,
    };
    expect(projectThreeBoneTransformToAuthoredV1(
      snapshot,
      targetRig,
      "TRANSLATION",
    )).toEqual([
      expect.closeTo(4),
      expect.closeTo(4),
      expect.closeTo(10),
    ]);
    expect(projectThreeBoneTransformToAuthoredV1(
      snapshot,
      targetRig,
      "ROTATION",
    )).toEqual([
      expect.closeTo(0),
      expect.closeTo(-Math.SQRT1_2),
      expect.closeTo(0),
      expect.closeTo(Math.SQRT1_2),
    ]);
  });
});
