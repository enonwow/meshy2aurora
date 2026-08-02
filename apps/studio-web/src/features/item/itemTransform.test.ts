import { describe, expect, it } from "vitest";
import * as THREE from "three";
import { authoredItemMatrix, itemPreviewGroundingMatrix } from "./itemTransform";

describe("Item authored transform parity", () => {
  it("keeps the authored Aurora pivot fixed while applying rotation and scale", () => {
    const translation: [number, number, number] = [2.5, -1.25, 4.75];
    const pivot: [number, number, number] = [1.5, -2, 0.75];
    const matrix = authoredItemMatrix({
      translation,
      rotationDegrees: [37, -23, 71],
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
});
