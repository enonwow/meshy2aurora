import { describe, expect, it } from "vitest";
import * as THREE from "three";
import { availableSceneOverlays, SceneOverlayRuntime } from "./sceneOverlays";

describe("scene overlay inventory", () => {
  it("only offers overlays supported by the loaded scene", () => {
    const empty = new THREE.Group();
    expect(availableSceneOverlays(empty)).toEqual(["grid", "axes"]);

    const meshRoot = new THREE.Group();
    meshRoot.add(new THREE.Mesh(new THREE.BoxGeometry(), new THREE.MeshBasicMaterial()));
    expect(availableSceneOverlays(meshRoot)).toEqual(["grid", "axes", "bounds", "wireframe"]);

    const skinnedRoot = new THREE.Group();
    const bone = new THREE.Bone();
    const geometry = new THREE.BoxGeometry();
    const vertexCount = geometry.getAttribute("position").count;
    const weights = new Float32Array(vertexCount * 4);
    for (let index = 0; index < vertexCount; index += 1) weights[index * 4] = 1;
    geometry.setAttribute("skinIndex", new THREE.Uint16BufferAttribute(new Uint16Array(vertexCount * 4), 4));
    geometry.setAttribute("skinWeight", new THREE.Float32BufferAttribute(weights, 4));
    const skinned = new THREE.SkinnedMesh(geometry, new THREE.MeshBasicMaterial());
    skinned.add(bone);
    skinned.bind(new THREE.Skeleton([bone]));
    skinnedRoot.add(skinned);
    expect(availableSceneOverlays(skinnedRoot)).toContain("skeleton");
  });
});

describe("SceneOverlayRuntime", () => {
  it("adds and removes real helpers and restores material wireframe state on dispose", () => {
    const scene = new THREE.Scene();
    const material = new THREE.MeshBasicMaterial({ wireframe: false });
    const root = new THREE.Mesh(new THREE.BoxGeometry(), material);
    scene.add(root);
    const runtime = new SceneOverlayRuntime(scene, root);

    runtime.set("grid", true);
    runtime.set("bounds", true);
    runtime.set("wireframe", true);
    expect(scene.children.some((child) => child instanceof THREE.GridHelper)).toBe(true);
    expect(scene.children.some((child) => child instanceof THREE.Box3Helper)).toBe(true);
    expect(material.wireframe).toBe(true);

    runtime.set("grid", false);
    expect(scene.children.some((child) => child instanceof THREE.GridHelper)).toBe(false);

    runtime.dispose();
    expect(scene.children.some((child) => child instanceof THREE.Box3Helper)).toBe(false);
    expect(material.wireframe).toBe(false);
  });
});
