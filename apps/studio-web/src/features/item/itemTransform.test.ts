import { describe, expect, it } from "vitest";
import * as THREE from "three";
import {
  authoredItemMatrix,
  itemPreviewGroundingMatrix,
  itemPropertiesCameraFrame,
} from "./itemTransform";

describe("Item authored transform parity", () => {
  it("keeps the authored Aurora pivot fixed while applying rotation and scale", () => {
    const translation: [number, number, number] = [2.5, -1.25, 4.75];
    const pivot: [number, number, number] = [1.5, -2, 0.75];
    const matrix = authoredItemMatrix({
      translation,
      rotationXyzw: new THREE.Quaternion().setFromEuler(
        new THREE.Euler(37 * Math.PI / 180, -23 * Math.PI / 180, 71 * Math.PI / 180, "XYZ"),
      ).toArray(),
      uniformScale: 2.25,
      pivot,
    });
    const basis = new THREE.Matrix4().set(
      1, 0, 0, 0,
      0, 0, 1, 0,
      0, 1, 0, 0,
      0, 0, 0, 1,
    );
    const displayedPivot = new THREE.Vector3(...pivot).applyMatrix4(basis);
    const expected = new THREE.Vector3(
      pivot[0] + translation[0],
      pivot[1] + translation[1],
      pivot[2] + translation[2],
    ).applyMatrix4(basis);

    expect(displayedPivot.applyMatrix4(matrix).toArray()).toEqual(
      expect.arrayContaining(expected.toArray().map((value) => expect.closeTo(value, 6))),
    );
  });

  it("grounds the source-space bottom center exactly like the Aurora M0 profile", () => {
    const grounding = itemPreviewGroundingMatrix({
      min: new THREE.Vector3(-4, -2, -8),
      max: new THREE.Vector3(6, 10, 12),
    });
    expect(new THREE.Vector3(1, -2, 2).applyMatrix4(grounding).toArray()).toEqual([0, 0, 0]);
    expect(new THREE.Vector3(6, 10, 12).applyMatrix4(grounding).toArray()).toEqual([5, 12, 10]);
  });

  it("shows the post-rotation target-space scale used by the Item composer", () => {
    const matrix = authoredItemMatrix({
      translation: [0, 0, 0],
      rotationXyzw: [0, 0, 0, 1],
      uniformScale: 1,
      pivot: [0, 0, 0],
      targetSpaceScaleXyz: [2, 3, 4],
    });

    expect(new THREE.Vector3(1, 2, 3).applyMatrix4(matrix).toArray()).toEqual([2, 8, 9]);
  });

  it("frames Aurora Item axial length as screen-up without rotating the model", () => {
    const frame = itemPropertiesCameraFrame({
      min: new THREE.Vector3(-0.05, -0.2, -0.3),
      max: new THREE.Vector3(0.05, 0.2, 1.3),
    }, 1.5);

    expect(frame.target).toEqual([0, 0, 0.5]);
    expect(frame.position[0]).toBeGreaterThan(frame.target[0]);
    expect(frame.position[1]).toBe(frame.target[1]);
    expect(frame.position[2]).toBe(frame.target[2]);
    expect(frame.up).toEqual([0, 0, 1]);
    expect(frame.halfHeight).toBeGreaterThan(0.8);
    expect(frame.near).toBeGreaterThan(0);
    expect(frame.far).toBeGreaterThan(frame.near);
  });

  it("fits Aurora Item width horizontally for narrow viewports", () => {
    const widePart = {
      min: new THREE.Vector3(-0.05, -2, -0.25),
      max: new THREE.Vector3(0.05, 2, 0.25),
    };

    expect(itemPropertiesCameraFrame(widePart, 0.5).halfHeight).toBeCloseTo(4.72, 6);
    expect(itemPropertiesCameraFrame(widePart, 2).halfHeight).toBeCloseTo(1.18, 6);
  });
});
