import * as THREE from "three";

export type SceneOverlayKind = "grid" | "axes" | "skeleton" | "bounds" | "wireframe";

const overlayOrder: readonly SceneOverlayKind[] = ["grid", "axes", "skeleton", "bounds", "wireframe"];

function materials(root: THREE.Object3D) {
  const result = new Set<THREE.Material>();
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    const meshMaterials = Array.isArray(object.material) ? object.material : [object.material];
    meshMaterials.forEach((material) => result.add(material));
  });
  return result;
}

export function availableSceneOverlays(root: THREE.Object3D): SceneOverlayKind[] {
  let hasMesh = false;
  let hasSkeleton = false;
  root.traverse((object) => {
    hasMesh ||= object instanceof THREE.Mesh;
    hasSkeleton ||= object instanceof THREE.SkinnedMesh && object.skeleton.bones.length > 0;
  });
  const hasBounds = !new THREE.Box3().setFromObject(root).isEmpty();
  return overlayOrder.filter((kind) => {
    if (kind === "skeleton") return hasSkeleton;
    if (kind === "wireframe") return hasMesh;
    if (kind === "bounds") return hasBounds;
    return true;
  });
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

export class SceneOverlayRuntime {
  private readonly helpers = new Map<Exclude<SceneOverlayKind, "wireframe">, THREE.Object3D>();
  private readonly wireframeDefaults = new Map<THREE.Material, boolean>();

  constructor(
    private readonly scene: THREE.Scene,
    private readonly root: THREE.Object3D,
  ) {}

  set(kind: SceneOverlayKind, enabled: boolean) {
    if (kind === "wireframe") {
      this.setWireframe(enabled);
      return;
    }
    const existing = this.helpers.get(kind);
    if (!enabled) {
      if (existing) {
        this.scene.remove(existing);
        disposeHelper(existing);
        this.helpers.delete(kind);
      }
      return;
    }
    if (existing) return;

    const bounds = new THREE.Box3().setFromObject(this.root);
    const extent = bounds.isEmpty() ? 1 : Math.max(bounds.getSize(new THREE.Vector3()).length(), 1);
    let helper: THREE.Object3D;
    if (kind === "grid") helper = new THREE.GridHelper(extent * 2, 20, 0x38536b, 0x1e3040);
    else if (kind === "axes") helper = new THREE.AxesHelper(extent * 0.45);
    else if (kind === "skeleton") helper = new THREE.SkeletonHelper(this.root);
    else helper = new THREE.Box3Helper(bounds, 0x35d8e6);
    this.helpers.set(kind, helper);
    this.scene.add(helper);
  }

  dispose() {
    [...this.helpers.values()].forEach((helper) => {
      this.scene.remove(helper);
      disposeHelper(helper);
    });
    this.helpers.clear();
    this.setWireframe(false);
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
