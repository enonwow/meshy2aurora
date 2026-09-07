import { describe, expect, it } from "vitest";
import * as THREE from "three";
import {
  availableSceneOverlays,
  defaultSceneOverlays,
  sceneObjectBounds,
  SceneOverlayRuntime,
} from "./sceneOverlays";

describe("scene overlay inventory", () => {
  it("enables x-ray with the default rig so internal bones are visible without another click", () => {
    expect(defaultSceneOverlays([
      "mesh",
      "grid",
      "axes",
      "bones",
      "joints",
      "helpers",
      "labels",
      "xray",
      "bounds",
      "wireframe",
      "skinClusters",
    ])).toEqual(["mesh", "grid", "bones", "joints", "xray"]);
  });

  it("only offers overlays supported by the loaded scene", () => {
    const empty = new THREE.Group();
    expect(availableSceneOverlays(empty)).toEqual(["grid", "axes"]);

    const meshRoot = new THREE.Group();
    meshRoot.add(new THREE.Mesh(new THREE.BoxGeometry(), new THREE.MeshBasicMaterial()));
    expect(availableSceneOverlays(meshRoot)).toEqual(["mesh", "grid", "axes", "bounds", "wireframe"]);

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
    expect(availableSceneOverlays(skinnedRoot)).toEqual(expect.arrayContaining(["mesh", "bones", "joints", "labels", "xray", "skinClusters"]));

    const animationOnlyRoot = new THREE.Group();
    const animationRoot = new THREE.Bone();
    const animationChild = new THREE.Bone();
    animationChild.position.set(0, 3, 0);
    animationRoot.add(animationChild);
    animationOnlyRoot.add(animationRoot);
    animationOnlyRoot.updateMatrixWorld(true);
    expect(availableSceneOverlays(animationOnlyRoot)).toEqual(expect.arrayContaining(["bones", "joints"]));
    expect(sceneObjectBounds(animationOnlyRoot).getSize(new THREE.Vector3()).y).toBe(3);
  });
});

describe("SceneOverlayRuntime", () => {
  it("places the grid below the model bounds without moving the model or pivot axes", () => {
    const scene = new THREE.Scene();
    const root = new THREE.Mesh(new THREE.BoxGeometry(2, 4, 2), new THREE.MeshBasicMaterial());
    root.position.set(3, -7, 5);
    root.updateMatrixWorld(true);
    const originalPosition = root.position.clone();
    const bounds = new THREE.Box3().setFromObject(root);
    const runtime = new SceneOverlayRuntime(scene, root);

    runtime.set("grid", true);
    runtime.set("axes", true);

    const grid = scene.children.find((child) => child instanceof THREE.GridHelper);
    const axes = scene.children.find((child) => child instanceof THREE.AxesHelper);
    expect(grid?.position.y).toBeLessThan(bounds.min.y);
    expect(axes?.position.toArray()).toEqual([0, 0, 0]);
    expect(root.position.toArray()).toEqual(originalPosition.toArray());
  });

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

  it("uses semantic rig lines and joint markers instead of Three SkeletonHelper", () => {
    const scene = new THREE.Scene();
    const root = new THREE.Group();
    const rigRoot = new THREE.Bone();
    rigRoot.name = "rootdummy";
    const pivot = new THREE.Bone();
    pivot.name = "torso";
    pivot.position.y = 2;
    rigRoot.add(pivot);
    root.add(rigRoot);
    root.updateMatrixWorld(true);
    const runtime = new SceneOverlayRuntime(scene, root);

    runtime.set("bones", true);
    runtime.set("joints", true);
    runtime.update();

    expect(scene.getObjectByName("aurora-rig-bones-v1")).toBeInstanceOf(THREE.LineSegments);
    expect(scene.getObjectByName("aurora-rig-joints-v1")?.children).toHaveLength(2);
    expect(scene.children.some((child) => child instanceof THREE.SkeletonHelper)).toBe(false);
  });

  it("toggles the mesh independently and visualizes dominant skin clusters without losing original colors", () => {
    const scene = new THREE.Scene();
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.Float32BufferAttribute([0, 0, 0, 1, 0, 0], 3));
    geometry.setAttribute("skinIndex", new THREE.Uint16BufferAttribute([0, 1, 0, 0, 1, 0, 0, 0], 4));
    geometry.setAttribute("skinWeight", new THREE.Float32BufferAttribute([1, 0, 0, 0, 1, 0, 0, 0], 4));
    const originalColors = new THREE.Float32BufferAttribute([1, 1, 1, 1, 1, 1], 3);
    geometry.setAttribute("color", originalColors);
    const material = new THREE.MeshBasicMaterial({ vertexColors: false });
    const mesh = new THREE.SkinnedMesh(geometry, material);
    const runtime = new SceneOverlayRuntime(scene, mesh);

    runtime.set("mesh", false);
    expect(mesh.visible).toBe(false);
    runtime.set("mesh", true);
    expect(mesh.visible).toBe(true);

    runtime.set("skinClusters", true);
    expect(material.vertexColors).toBe(true);
    expect(geometry.getAttribute("color")).not.toBe(originalColors);
    runtime.set("skinClusters", false);
    expect(material.vertexColors).toBe(false);
    expect(geometry.getAttribute("color")).toBe(originalColors);
  });
});
