import { describe, expect, it, vi } from "vitest";
import * as THREE from "three";
import { disposeObjectResources } from "./SceneViewport";

describe("viewport resource disposal", () => {
  it("disposes geometry, material textures, materials and skeleton resources", () => {
    const root = new THREE.Group();
    const texture = new THREE.Texture();
    const geometry = new THREE.BufferGeometry();
    const material = new THREE.MeshStandardMaterial({ map: texture });
    const skeleton = new THREE.Skeleton([]);
    const mesh = new THREE.SkinnedMesh(geometry, material);
    mesh.bind(skeleton);
    root.add(mesh);

    const textureDispose = vi.spyOn(texture, "dispose");
    const geometryDispose = vi.spyOn(geometry, "dispose");
    const materialDispose = vi.spyOn(material, "dispose");
    const skeletonDispose = vi.spyOn(skeleton, "dispose");

    disposeObjectResources(root);

    expect(textureDispose).toHaveBeenCalledOnce();
    expect(geometryDispose).toHaveBeenCalledOnce();
    expect(materialDispose).toHaveBeenCalledOnce();
    expect(skeletonDispose).toHaveBeenCalledOnce();
  });
});
