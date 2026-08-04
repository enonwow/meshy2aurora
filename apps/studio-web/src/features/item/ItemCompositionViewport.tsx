import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import type { ItemAttachmentProfileV1, ItemPartDraft } from "./types";
import {
  analyzeItemSeams,
  type ItemPreviewBounds,
  type ItemSeamResult,
} from "./itemPreview";
import { inspectItemPreviewSourceNode } from "./itemSourceNode";
import {
  authoredItemMatrix,
  itemPreviewGroundingMatrix,
  itemPropertiesCameraFrame,
} from "./itemTransform";
import { itemWeaponColorwayMultiplier, type ItemWeaponColor } from "./itemColorway";

export interface ItemCompositionViewportProps {
  readonly parts: readonly ItemPartDraft[];
  readonly mode: "COMPOSED" | "EXPLODED" | "ICON";
  readonly tolerance: number;
  readonly referenceProfile?: ItemAttachmentProfileV1;
  readonly showReference?: boolean;
  readonly onSeams: (seams: ItemSeamResult[]) => void;
  readonly onNodeNames: (field: string, names: string[]) => void;
}

const fileIdentities = new WeakMap<File, number>();
let nextFileIdentity = 1;

function fileIdentity(file?: File) {
  if (!file) return 0;
  const existing = fileIdentities.get(file);
  if (existing) return existing;
  const identity = nextFileIdentity;
  nextFileIdentity += 1;
  fileIdentities.set(file, identity);
  return identity;
}

function parseGlb(loader: GLTFLoader, bytes: ArrayBuffer) {
  return new Promise<THREE.Group>((resolve, reject) => {
    loader.parse(bytes, "", (gltf) => resolve(gltf.scene), reject);
  });
}

function disposeObject(object: THREE.Object3D) {
  object.traverse((child) => {
    if (!(child instanceof THREE.Mesh)) return;
    child.geometry?.dispose();
    const materials = Array.isArray(child.material) ? child.material : [child.material];
    for (const material of materials) {
      for (const value of Object.values(material)) {
        if (value instanceof THREE.Texture) value.dispose();
      }
      material.dispose();
    }
  });
}

function applyWeaponColorwayPreview(object: THREE.Object3D, color: number | null) {
  const multiplier = itemWeaponColorwayMultiplier(color as ItemWeaponColor | null);
  object.traverse((child) => {
    if (!(child instanceof THREE.Mesh)) return;
    const materials = Array.isArray(child.material) ? child.material : [child.material];
    for (const material of materials) {
      if (!("color" in material) || !(material.color instanceof THREE.Color)) continue;
      const original = material.userData.m2aItemOriginalColor as
        | readonly [number, number, number]
        | undefined;
      const source = original ?? [material.color.r, material.color.g, material.color.b] as const;
      if (!original) material.userData.m2aItemOriginalColor = source;
      material.color.setRGB(
        source[0] * multiplier[0],
        source[1] * multiplier[1],
        source[2] * multiplier[2],
      );
    }
  });
}

export function ItemCompositionViewport({
  parts,
  mode,
  tolerance,
  referenceProfile,
  showReference = false,
  onSeams,
  onNodeNames,
}: ItemCompositionViewportProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const partsRef = useRef(parts);
  const modeRef = useRef(mode);
  const toleranceRef = useRef(tolerance);
  const referenceProfileRef = useRef(referenceProfile);
  const showReferenceRef = useRef(showReference);
  const onSeamsRef = useRef(onSeams);
  const onNodeNamesRef = useRef(onNodeNames);
  const [error, setError] = useState<string>();
  partsRef.current = parts;
  modeRef.current = mode;
  toleranceRef.current = tolerance;
  referenceProfileRef.current = referenceProfile;
  showReferenceRef.current = showReference;
  onSeamsRef.current = onSeams;
  onNodeNamesRef.current = onNodeNames;
  const sourceSignature = parts
    .map((part) => [
      part.field,
      part.sourceKind,
      part.file?.name ?? "",
      part.file?.size ?? 0,
      part.file?.lastModified ?? 0,
      fileIdentity(part.file),
      part.sourceNode,
    ].join(":"))
    .join("|");

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return undefined;
    if (navigator.userAgent.toLowerCase().includes("jsdom")) {
      setError("WebGL preview is unavailable in the DOM test environment.");
      return undefined;
    }
    let cancelled = false;
    const scene = new THREE.Scene();
    scene.background = new THREE.Color("#5b6063");
    const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.001, 10_000);
    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    host.replaceChildren(renderer.domElement);
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    scene.add(new THREE.HemisphereLight(0xc8f8ff, 0x293238, 3));
    scene.add(new THREE.AmbientLight(0xffffff, 1.4));
    const key = new THREE.DirectionalLight(0xffffff, 3.5);
    key.position.set(4, 6, 5);
    scene.add(key, key.target);
    const content = new THREE.Group();
    scene.add(content);
    const referenceFrames = new THREE.Group();
    scene.add(referenceFrames);
    const loader = new GLTFLoader();
    const wrappers = new Map<string, THREE.Group>();
    let loaded = false;
    let appliedCompositionSignature = "";
    let viewHalfHeight = 1;
    const resize = () => {
      const width = Math.max(1, host.clientWidth);
      const height = Math.max(1, host.clientHeight);
      renderer.setSize(width, height, false);
      const aspect = width / height;
      camera.left = -viewHalfHeight * aspect;
      camera.right = viewHalfHeight * aspect;
      camera.top = viewHalfHeight;
      camera.bottom = -viewHalfHeight;
      camera.updateProjectionMatrix();
    };
    const observer = new ResizeObserver(resize);
    observer.observe(host);
    resize();
    const applyComposition = () => {
      const liveParts = partsRef.current.filter(
        (part) => part.sourceKind === "MESHY_GLB" && part.file,
      );
      const signature = JSON.stringify({
        mode: modeRef.current,
        tolerance: toleranceRef.current,
        showReference: showReferenceRef.current,
        referenceProfileSha256: referenceProfileRef.current?.profileSha256 ?? null,
        parts: liveParts.map((part) => ({
          field: part.field,
          translation: part.translation,
          rotationXyzw: part.rotationXyzw,
          uniformScale: part.uniformScale,
          pivot: part.pivot,
          targetSpaceScaleXyz: part.targetSpaceScaleXyz,
          weaponColor: part.weaponColor,
        })),
      });
      if (!loaded || signature === appliedCompositionSignature) return;
      appliedCompositionSignature = signature;

      const composed = liveParts.flatMap((part) => {
        const wrapper = wrappers.get(part.field);
        if (!wrapper) return [];
        applyWeaponColorwayPreview(wrapper, part.weaponColor);
        wrapper.matrix.copy(authoredItemMatrix(part));
        wrapper.updateWorldMatrix(true, true);
        return [{ part, wrapper, bounds: new THREE.Box3().setFromObject(wrapper) }];
      });
      const seamBounds: ItemPreviewBounds[] = composed.map(({ part, bounds }) => ({
        field: part.field,
        min: bounds.min.toArray() as [number, number, number],
        max: bounds.max.toArray() as [number, number, number],
      }));
      onSeamsRef.current(analyzeItemSeams(seamBounds, toleranceRef.current));

      for (const child of [...referenceFrames.children]) {
        referenceFrames.remove(child);
        if (child instanceof THREE.Box3Helper) {
          child.geometry.dispose();
          const materials = Array.isArray(child.material) ? child.material : [child.material];
          materials.forEach((material) => material.dispose());
        }
      }
      if (
        showReferenceRef.current
        && modeRef.current === "COMPOSED"
        && referenceProfileRef.current
      ) {
        for (const slot of referenceProfileRef.current.slots) {
          const bounds = new THREE.Box3(
            new THREE.Vector3(slot.boundsMin[0], slot.boundsMin[2], slot.boundsMin[1]),
            new THREE.Vector3(slot.boundsMax[0], slot.boundsMax[2], slot.boundsMax[1]),
          );
          const helper = new THREE.Box3Helper(bounds, 0x9b87f5);
          const materials = Array.isArray(helper.material) ? helper.material : [helper.material];
          materials.forEach((material) => {
            material.transparent = true;
            material.opacity = 0.7;
            material.depthTest = false;
          });
          helper.renderOrder = 10;
          referenceFrames.add(helper);
        }
      }

      if (modeRef.current === "EXPLODED" && composed.length > 1) {
        const combined = new THREE.Box3();
        composed.forEach(({ bounds }) => combined.union(bounds));
        const size = combined.getSize(new THREE.Vector3());
        const spacing = Math.max(size.x, size.y, size.z, 0.25) * 0.7;
        composed.forEach(({ wrapper }, index) => {
          const offset = (index - (composed.length - 1) / 2) * spacing;
          wrapper.matrix.premultiply(new THREE.Matrix4().makeTranslation(0, offset, 0));
          wrapper.updateWorldMatrix(true, true);
        });
      }

      const visibleBounds = new THREE.Box3().setFromObject(content);
      if (referenceFrames.children.length > 0) {
        visibleBounds.union(new THREE.Box3().setFromObject(referenceFrames));
      }
      if (!visibleBounds.isEmpty()) {
        const frame = itemPropertiesCameraFrame(
          { min: visibleBounds.min, max: visibleBounds.max },
          Math.max(1, host.clientWidth) / Math.max(1, host.clientHeight),
        );
        viewHalfHeight = frame.halfHeight;
        controls.target.set(...frame.target);
        camera.up.set(...frame.up);
        camera.position.set(...frame.position);
        camera.near = frame.near;
        camera.far = frame.far;
        key.position.set(
          frame.position[0],
          frame.position[1] + frame.halfHeight * 0.45,
          frame.position[2] + frame.halfHeight * 0.45,
        );
        key.target.position.set(...frame.target);
        key.target.updateMatrixWorld();
        resize();
        controls.update();
      }
    };
    let frame = 0;
    const render = () => {
      applyComposition();
      controls.update();
      renderer.render(scene, camera);
      frame = requestAnimationFrame(render);
    };
    render();

    void (async () => {
      try {
        setError(undefined);
        const meshParts = partsRef.current.filter(
          (part) => part.sourceKind === "MESHY_GLB" && part.file,
        );
        const loadedParts = await Promise.all(meshParts.map(async (part) => {
          const sourceBytes = await part.file!.arrayBuffer();
          const contract = inspectItemPreviewSourceNode(sourceBytes, part.sourceNode);
          const sourceScene = await parseGlb(loader, sourceBytes);
          onNodeNamesRef.current(part.field, [...contract.rootNames]);
          const matches = contract.selectedName
            ? sourceScene.children.filter((child) => child.name === contract.selectedName)
            : [sourceScene];
          if (matches.length !== 1) {
            throw new Error(
              `${part.field}: validated sourceNode ${JSON.stringify(contract.selectedName)} matched ${matches.length} rendered roots`,
            );
          }
          const renderedSource = matches[0].clone(true);
          renderedSource.updateWorldMatrix(true, true);
          const sourceBounds = new THREE.Box3().setFromObject(renderedSource);
          if (sourceBounds.isEmpty()) {
            throw new Error(`${part.field}: selected source contains no render geometry`);
          }
          const groundedSource = new THREE.Group();
          groundedSource.matrixAutoUpdate = false;
          groundedSource.matrix.copy(itemPreviewGroundingMatrix(sourceBounds));
          groundedSource.add(renderedSource);
          const wrapper = new THREE.Group();
          wrapper.matrixAutoUpdate = false;
          wrapper.add(groundedSource);
          content.add(wrapper);
          return { part, wrapper };
        }));
        if (cancelled) return;
        loadedParts.forEach(({ part, wrapper }) => wrappers.set(part.field, wrapper));
        appliedCompositionSignature = "";
        loaded = true;
        applyComposition();
      } catch (reason) {
        if (!cancelled) setError(reason instanceof Error ? reason.message : String(reason));
      }
    })();

    return () => {
      cancelled = true;
      cancelAnimationFrame(frame);
      observer.disconnect();
      controls.dispose();
      disposeObject(content);
      for (const child of [...referenceFrames.children]) {
        if (child instanceof THREE.Box3Helper) {
          child.geometry.dispose();
          const materials = Array.isArray(child.material) ? child.material : [child.material];
          materials.forEach((material) => material.dispose());
        }
      }
      renderer.dispose();
      renderer.domElement.remove();
    };
  }, [sourceSignature]);

  return (
    <div className="item-viewport" ref={hostRef}>
      {error ? <p className="item-viewport-error" role="alert">{error}</p> : null}
    </div>
  );
}
