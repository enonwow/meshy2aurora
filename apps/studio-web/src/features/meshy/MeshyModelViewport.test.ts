// @vitest-environment jsdom

import * as THREE from "three";
import { describe, expect, it } from "vitest";
import {
  applyMeshyViewportMaterial,
  applyMeshyViewportWireframe,
  collectMeshyMaterialChannels,
  collectMeshyViewportStatistics,
  materialMapComponent,
  retextureInputTaskIdForArtifact,
} from "./MeshyModelViewport";

describe("collectMeshyViewportStatistics", () => {
  it("reports the default BufferGeometry draw mode as triangles", () => {
    const root = new THREE.Group();
    root.add(new THREE.Mesh(new THREE.BufferGeometry().setAttribute("position", new THREE.BufferAttribute(new Float32Array([
      0, 0, 0,
      1, 0, 0,
      0, 1, 0,
    ]), 3)), new THREE.MeshBasicMaterial()));

    expect(collectMeshyViewportStatistics(root)).toEqual({
      topology: "Triangles",
      faces: 1,
      vertices: 3,
    });
  });
});

describe("collectMeshyMaterialChannels", () => {
  it("only exposes texture maps that the locally loaded GLB actually contains", () => {
    const root = new THREE.Group();
    const material = new THREE.MeshStandardMaterial({
      map: new THREE.Texture(),
      roughnessMap: new THREE.Texture(),
      normalMap: new THREE.Texture(),
    });
    root.add(new THREE.Mesh(new THREE.BoxGeometry(), material));

    expect(collectMeshyMaterialChannels(root)).toEqual(["baseColor", "roughness", "normal"]);
  });
});

describe("materialMapComponent", () => {
  it("reads the correct glTF packed texture component for scalar PBR maps", () => {
    expect(materialMapComponent("baseColor")).toBe("rgb");
    expect(materialMapComponent("normal")).toBe("rgb");
    expect(materialMapComponent("roughness")).toBe("g");
    expect(materialMapComponent("metallic")).toBe("b");
  });
});

describe("Meshy viewport material controls", () => {
  it("opens Material Matching only for a verified Meshy generation task", () => {
    expect(retextureInputTaskIdForArtifact({ taskIds: { REFINE: "refined-model" } } as never)).toBe("refined-model");
    expect(retextureInputTaskIdForArtifact({ taskIds: { PREVIEW: "image-to-3d-model" } } as never)).toBe("image-to-3d-model");
    expect(retextureInputTaskIdForArtifact({ taskIds: { RIG: "rig-only" } } as never)).toBeUndefined();
  });

  it("changes only the locally rendered recovered model for every display mode", () => {
    const texture = new THREE.Texture();
    const original = new THREE.MeshStandardMaterial({ map: texture });
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(), original);
    const root = new THREE.Group();
    root.add(mesh);
    const originals = new Map<THREE.Mesh, THREE.Material | THREE.Material[]>([[mesh, original]]);

    applyMeshyViewportMaterial(root, originals, "solid", undefined);
    expect(mesh.material).toBeInstanceOf(THREE.MeshStandardMaterial);
    expect(mesh.material).not.toBe(original);

    applyMeshyViewportMaterial(root, originals, "unlit", undefined);
    expect(mesh.material).toBeInstanceOf(THREE.MeshBasicMaterial);
    expect((mesh.material as unknown as THREE.MeshBasicMaterial).map).toBe(texture);

    applyMeshyViewportMaterial(root, originals, "lit", "baseColor");
    expect(mesh.material).toBeInstanceOf(THREE.MeshBasicMaterial);
    expect((mesh.material as unknown as THREE.MeshBasicMaterial).map).toBe(texture);

    applyMeshyViewportMaterial(root, originals, "lit", undefined);
    expect(mesh.material).toBe(original);
  });

  it("applies Material Matching as a local lit-material adjustment, never a task request", () => {
    const original = new THREE.MeshStandardMaterial({ color: 0x406080, metalness: .2, roughness: .8 });
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(), original);
    const root = new THREE.Group();
    root.add(mesh);
    const originals = new Map<THREE.Mesh, THREE.Material | THREE.Material[]>([[mesh, original]]);

    applyMeshyViewportMaterial(root, originals, "lit", undefined, { target: "metallic", intensity: 1, contrast: 1.4 });

    expect(mesh.material).toBeInstanceOf(THREE.MeshStandardMaterial);
    expect(mesh.material).not.toBe(original);
    const adjusted = mesh.material as THREE.MeshStandardMaterial;
    expect(adjusted.metalness).toBeGreaterThan(original.metalness);
    expect(adjusted.color.r).not.toBe(original.color.r);

    applyMeshyViewportMaterial(root, originals, "lit", undefined);
    expect(mesh.material).toBe(original);
  });

  it("applies wireframe to the material currently visible in the local viewport", () => {
    const material = new THREE.MeshStandardMaterial();
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(), material);
    const root = new THREE.Group();
    root.add(mesh);

    applyMeshyViewportWireframe(root, true);
    expect(material.wireframe).toBe(true);
    applyMeshyViewportWireframe(root, false);
    expect(material.wireframe).toBe(false);
  });
});
