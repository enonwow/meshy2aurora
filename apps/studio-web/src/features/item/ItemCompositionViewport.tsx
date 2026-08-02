import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import type { ItemPartDraft } from "./types";
import {
  analyzeItemSeams,
  type ItemPreviewBounds,
  type ItemSeamResult,
} from "./itemPreview";
import { inspectItemPreviewSourceNode } from "./itemSourceNode";
import { authoredItemMatrix, itemPreviewGroundingMatrix } from "./itemTransform";

export interface ItemCompositionViewportProps {
  readonly parts: readonly ItemPartDraft[];
  readonly mode: "COMPOSED" | "EXPLODED" | "ICON";
  readonly tolerance: number;
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

export function ItemCompositionViewport({
  parts,
  mode,
  tolerance,
  onSeams,
  onNodeNames,
}: ItemCompositionViewportProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const partsRef = useRef(parts);
  const modeRef = useRef(mode);
  const toleranceRef = useRef(tolerance);
  const onSeamsRef = useRef(onSeams);
  const onNodeNamesRef = useRef(onNodeNames);
  const [error, setError] = useState<string>();
  partsRef.current = parts;
  modeRef.current = mode;
  toleranceRef.current = tolerance;
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
    scene.background = new THREE.Color("#0b1114");
    const camera = new THREE.PerspectiveCamera(42, 1, 0.001, 10_000);
    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    host.replaceChildren(renderer.domElement);
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    scene.add(new THREE.HemisphereLight(0xc8f8ff, 0x172027, 2.5));
    const key = new THREE.DirectionalLight(0xffffff, 3);
    key.position.set(4, 6, 5);
    scene.add(key);
    const grid = new THREE.GridHelper(10, 20, 0x315058, 0x20363c);
    scene.add(grid);
    const content = new THREE.Group();
    scene.add(content);
    const loader = new GLTFLoader();
    const wrappers = new Map<string, THREE.Group>();
    let loaded = false;
    let appliedCompositionSignature = "";
    const resize = () => {
      const width = Math.max(1, host.clientWidth);
      const height = Math.max(1, host.clientHeight);
      renderer.setSize(width, height, false);
      camera.aspect = width / height;
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
        parts: liveParts.map((part) => ({
          field: part.field,
          translation: part.translation,
          rotationDegrees: part.rotationDegrees,
          uniformScale: part.uniformScale,
          pivot: part.pivot,
        })),
      });
      if (!loaded || signature === appliedCompositionSignature) return;
      appliedCompositionSignature = signature;

      const composed = liveParts.flatMap((part) => {
        const wrapper = wrappers.get(part.field);
        if (!wrapper) return [];
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

      if (modeRef.current === "EXPLODED" && composed.length > 1) {
        const combined = new THREE.Box3();
        composed.forEach(({ bounds }) => combined.union(bounds));
        const size = combined.getSize(new THREE.Vector3());
        const spacing = Math.max(size.x, size.y, size.z, 0.25) * 0.7;
        composed.forEach(({ wrapper }, index) => {
          const offset = (index - (composed.length - 1) / 2) * spacing;
          wrapper.matrix.premultiply(new THREE.Matrix4().makeTranslation(offset, 0, 0));
          wrapper.updateWorldMatrix(true, true);
        });
      }
      grid.visible = modeRef.current !== "ICON";

      const visibleBounds = new THREE.Box3().setFromObject(content);
      if (!visibleBounds.isEmpty()) {
        const center = visibleBounds.getCenter(new THREE.Vector3());
        const size = visibleBounds.getSize(new THREE.Vector3());
        const radius = Math.max(size.x, size.y, size.z, 0.25);
        controls.target.copy(center);
        if (modeRef.current === "ICON") {
          camera.up.set(-1, -1, 2).normalize();
          camera.position.copy(center).add(
            new THREE.Vector3(1, 1, 1).normalize().multiplyScalar(radius * 3),
          );
        } else {
          camera.up.set(0, 1, 0);
          camera.position.copy(center).add(new THREE.Vector3(radius * 1.5, radius, radius * 1.8));
        }
        camera.near = Math.max(radius / 10_000, 0.001);
        camera.far = Math.max(radius * 100, 100);
        camera.updateProjectionMatrix();
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
