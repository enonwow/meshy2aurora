import { describe, expect, it } from "vitest";
import * as THREE from "three";
import {
  componentGeometry,
  connectedTriangleComponents,
} from "./PlaceableAuthoringViewport";

describe("placeable viewport topology", () => {
  it("uses the same shared-index connected-component rule as core", () => {
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.Float32BufferAttribute([
      0, 0, 0,
      1, 0, 0,
      0, 1, 0,
      4, 0, 0,
      5, 0, 0,
      4, 1, 0,
    ], 3));
    geometry.setIndex([0, 1, 2, 3, 4, 5]);

    expect(connectedTriangleComponents(geometry)).toEqual([[0], [1]]);
    expect(Array.from(componentGeometry(geometry, 1).getIndex()?.array ?? [])).toEqual([3, 4, 5]);
    geometry.dispose();
  });
});
