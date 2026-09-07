import * as THREE from "three";
import {
  RigOverlayRuntimeV1,
  type ReadbackRigInventoryV1,
  type RigNodeDescriptorV1,
  type RigOverlayOptionV1,
} from "./rigOverlay";

export type SceneOverlayKind = "mesh" | "grid" | "axes" | RigOverlayOptionV1 | "bounds" | "wireframe" | "skinClusters";

const overlayOrder: readonly SceneOverlayKind[] = [
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
];

const rigOverlayKinds = new Set<SceneOverlayKind>(["bones", "joints", "helpers", "labels", "xray"]);

function materials(root: THREE.Object3D) {
  const result = new Set<THREE.Material>();
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    const meshMaterials = Array.isArray(object.material) ? object.material : [object.material];
    meshMaterials.forEach((material) => result.add(material));
  });
  return result;
}

export function sceneObjectBounds(root: THREE.Object3D) {
  root.updateWorldMatrix(true, true);
  const bounds = new THREE.Box3().setFromObject(root);
  if (!bounds.isEmpty()) return bounds;
  const points: THREE.Vector3[] = [];
  root.traverse((object) => {
    if (object instanceof THREE.Bone) points.push(object.getWorldPosition(new THREE.Vector3()));
  });
  return points.length > 0 ? new THREE.Box3().setFromPoints(points) : bounds;
}

export function availableSceneOverlays(root: THREE.Object3D): SceneOverlayKind[] {
  let hasMesh = false;
  let hasRig = false;
  root.traverse((object) => {
    hasMesh ||= object instanceof THREE.Mesh;
    hasRig ||= object instanceof THREE.Bone
      || object instanceof THREE.SkinnedMesh && object.skeleton.bones.length > 0;
  });
  const hasBounds = !sceneObjectBounds(root).isEmpty();
  return overlayOrder.filter((kind) => {
    if (rigOverlayKinds.has(kind)) return hasRig;
    if (kind === "mesh" || kind === "wireframe") return hasMesh;
    if (kind === "skinClusters") {
      let hasSkinClusters = false;
      root.traverse((object) => {
        hasSkinClusters ||= object instanceof THREE.SkinnedMesh
          && Boolean(object.geometry.getAttribute("skinIndex"))
          && Boolean(object.geometry.getAttribute("skinWeight"));
      });
      return hasSkinClusters;
    }
    if (kind === "bounds") return hasBounds;
    return true;
  });
}

export function defaultSceneOverlays(available: readonly SceneOverlayKind[]): SceneOverlayKind[] {
  const defaults = new Set<SceneOverlayKind>(["mesh", "grid", "bones", "joints", "xray"]);
  return available.filter((kind) => defaults.has(kind));
}

function disposeHelper(helper: THREE.Object3D) {
  const disposable = helper as THREE.Object3D & {
    geometry?: THREE.BufferGeometry;
    material?: THREE.Material | THREE.Material[];
  };
  disposable.geometry?.dispose();
  const helperMaterials = disposable.material
    ? Array.isArray(disposable.material) ? disposable.material : [disposable.material]
    : [];
  helperMaterials.forEach((material) => material.dispose());
}

export function groundedGridY(bounds: THREE.Box3) {
  if (bounds.isEmpty()) return 0;
  const extent = Math.max(bounds.getSize(new THREE.Vector3()).length(), 1);
  return bounds.min.y - Math.max(extent * 1e-4, 1e-5);
}

export class SceneOverlayRuntime {
  private readonly helpers = new Map<"grid" | "axes" | "bounds", THREE.Object3D>();
  private readonly wireframeDefaults = new Map<THREE.Material, boolean>();
  private readonly meshVisibilityDefaults = new Map<THREE.Mesh, boolean>();
  private readonly clusterColorDefaults = new Map<THREE.BufferGeometry, THREE.BufferAttribute | THREE.InterleavedBufferAttribute | undefined>();
  private readonly clusterMaterialDefaults = new Map<THREE.Material, boolean>();
  private rigRuntime?: RigOverlayRuntimeV1;

  constructor(
    private readonly scene: THREE.Scene,
    private readonly root: THREE.Object3D,
  ) {}

  set(kind: SceneOverlayKind, enabled: boolean) {
    if (kind === "mesh") {
      this.setMeshVisibility(enabled);
      return;
    }
    if (kind === "wireframe") {
      this.setWireframe(enabled);
      return;
    }
    if (kind === "skinClusters") {
      this.setSkinClusters(enabled);
      return;
    }
    if (rigOverlayKinds.has(kind)) {
      this.rigRuntime ??= new RigOverlayRuntimeV1(this.scene, this.root);
      this.rigRuntime.set(kind as RigOverlayOptionV1, enabled);
      return;
    }
    const helperKind = kind as "grid" | "axes" | "bounds";
    const existing = this.helpers.get(helperKind);
    if (!enabled) {
      if (existing) {
        this.scene.remove(existing);
        disposeHelper(existing);
        this.helpers.delete(helperKind);
      }
      return;
    }
    if (existing) return;

    const bounds = sceneObjectBounds(this.root);
    const extent = bounds.isEmpty() ? 1 : Math.max(bounds.getSize(new THREE.Vector3()).length(), 1);
    let helper: THREE.Object3D;
    if (kind === "grid") {
      helper = new THREE.GridHelper(extent * 2, 20, 0x38536b, 0x1e3040);
      helper.position.y = groundedGridY(bounds);
    }
    else if (kind === "axes") helper = new THREE.AxesHelper(extent * 0.45);
    else helper = new THREE.Box3Helper(bounds, 0x35d8e6);
    this.helpers.set(helperKind, helper);
    this.scene.add(helper);
  }

  update() {
    this.rigRuntime?.update();
  }

  rigInventory(): ReadbackRigInventoryV1 | undefined {
    return this.rigRuntime?.inventory;
  }

  rigDescriptorFromObject(object?: THREE.Object3D | null): RigNodeDescriptorV1 | undefined {
    return this.rigRuntime?.descriptorFromObject(object);
  }

  dispose() {
    [...this.helpers.values()].forEach((helper) => {
      this.scene.remove(helper);
      disposeHelper(helper);
    });
    this.helpers.clear();
    this.rigRuntime?.dispose();
    this.rigRuntime = undefined;
    this.restoreMeshVisibility();
    this.setSkinClusters(false);
    this.setWireframe(false);
  }

  private setMeshVisibility(enabled: boolean) {
    this.root.traverse((object) => {
      if (!(object instanceof THREE.Mesh)) return;
      if (!this.meshVisibilityDefaults.has(object)) this.meshVisibilityDefaults.set(object, object.visible);
      object.visible = enabled ? (this.meshVisibilityDefaults.get(object) ?? true) : false;
    });
  }

  private restoreMeshVisibility() {
    this.meshVisibilityDefaults.forEach((visible, mesh) => { mesh.visible = visible; });
    this.meshVisibilityDefaults.clear();
  }

  private setSkinClusters(enabled: boolean) {
    this.root.traverse((object) => {
      if (!(object instanceof THREE.SkinnedMesh)) return;
      const geometry = object.geometry;
      const skinIndex = geometry.getAttribute("skinIndex");
      const skinWeight = geometry.getAttribute("skinWeight");
      if (!skinIndex || !skinWeight || skinIndex.count !== skinWeight.count) return;
      const meshMaterials = Array.isArray(object.material) ? object.material : [object.material];
      if (!enabled) {
        if (this.clusterColorDefaults.has(geometry)) {
          const original = this.clusterColorDefaults.get(geometry);
          if (original) geometry.setAttribute("color", original);
          else geometry.deleteAttribute("color");
          this.clusterColorDefaults.delete(geometry);
        }
        meshMaterials.forEach((material) => {
          const colorMaterial = material as THREE.Material & { vertexColors?: boolean };
          if (this.clusterMaterialDefaults.has(material)) {
            colorMaterial.vertexColors = this.clusterMaterialDefaults.get(material) ?? false;
            material.needsUpdate = true;
            this.clusterMaterialDefaults.delete(material);
          }
        });
        return;
      }
      if (!this.clusterColorDefaults.has(geometry)) this.clusterColorDefaults.set(geometry, geometry.getAttribute("color"));
      const colors = new Float32Array(skinIndex.count * 3);
      const color = new THREE.Color();
      for (let vertex = 0; vertex < skinIndex.count; vertex += 1) {
        let dominantLane = 0;
        let dominantWeight = -1;
        for (let lane = 0; lane < Math.min(skinWeight.itemSize, 4); lane += 1) {
          const weight = skinWeight.getComponent(vertex, lane);
          if (weight > dominantWeight) {
            dominantWeight = weight;
            dominantLane = lane;
          }
        }
        const bone = skinIndex.getComponent(vertex, dominantLane);
        color.setHSL((bone * 0.61803398875) % 1, 0.72, 0.55);
        color.toArray(colors, vertex * 3);
      }
      geometry.setAttribute("color", new THREE.Float32BufferAttribute(colors, 3));
      meshMaterials.forEach((material) => {
        const colorMaterial = material as THREE.Material & { vertexColors?: boolean };
        if (!this.clusterMaterialDefaults.has(material)) this.clusterMaterialDefaults.set(material, colorMaterial.vertexColors ?? false);
        colorMaterial.vertexColors = true;
        material.needsUpdate = true;
      });
    });
  }

  private setWireframe(enabled: boolean) {
    materials(this.root).forEach((material) => {
      if (!("wireframe" in material) || typeof material.wireframe !== "boolean") return;
      if (!this.wireframeDefaults.has(material)) this.wireframeDefaults.set(material, material.wireframe);
      material.wireframe = enabled ? true : (this.wireframeDefaults.get(material) ?? false);
      material.needsUpdate = true;
    });
    if (!enabled) this.wireframeDefaults.clear();
  }
}
