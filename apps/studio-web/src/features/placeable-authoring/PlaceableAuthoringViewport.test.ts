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
    geometry.setAttribute("color", new THREE.BufferAttribute(new Uint8Array([
      255, 0, 0, 255,
      255, 0, 0, 255,
      255, 0, 0, 255,
      0, 0, 255, 255,
      0, 0, 255, 255,
      0, 0, 255, 255,
    ]), 4, true));
    geometry.setIndex([0, 1, 2, 3, 4, 5]);

    const firstInspection = connectedTriangleComponents(geometry);
    expect(firstInspection).toEqual([[0], [1]]);
    expect(connectedTriangleComponents(geometry)).toBe(firstInspection);
    const component = componentGeometry(geometry, 1);
    expect(Array.from(component.getIndex()?.array ?? [])).toEqual([0, 1, 2]);
    expect(component.getAttribute("position").count).toBe(3);
    expect(component.getAttribute("color").array).toBeInstanceOf(Uint8Array);
    expect(component.getAttribute("color").normalized).toBe(true);
    expect(Array.from(component.getAttribute("color").array)).toEqual([
      0, 0, 255, 255,
      0, 0, 255, 255,
      0, 0, 255, 255,
    ]);
    component.dispose();
    geometry.dispose();
  });

  it("keeps component extraction compact for thousands of disconnected triangles", () => {
    const geometry = new THREE.BufferGeometry();
    const positions: number[] = [];
    const indices: number[] = [];
    for (let component = 0; component < 4_000; component += 1) {
      const vertex = component * 3;
      positions.push(component, 0, 0, component + 0.25, 0, 0, component, 0.25, 0);
      indices.push(vertex, vertex + 1, vertex + 2);
    }
    geometry.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    geometry.setIndex(indices);

    expect(connectedTriangleComponents(geometry)).toHaveLength(4_000);
    const extracted = componentGeometry(geometry, 3_999);
    expect(extracted.getAttribute("position").count).toBe(3);
    expect(Array.from(extracted.getIndex()?.array ?? [])).toEqual([0, 1, 2]);
    extracted.dispose();
    geometry.dispose();
  });
});
