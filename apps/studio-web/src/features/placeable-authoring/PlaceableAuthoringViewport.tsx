import { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { TransformControls } from "three/examples/jsm/controls/TransformControls.js";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import type {
  ElementTransformPatch,
  PlaceableAuthoringDocument,
  PlaceableAuthoringElement,
  PlaceableElementInspection,
  PlaceableElementTransform,
  PlaceableSnapSettings,
  TransformSpace,
  TransformTool,
} from "./types";

interface Props {
  readonly file: File;
  readonly dependency: string;
  readonly document: PlaceableAuthoringDocument;
  readonly inspection: PlaceableElementInspection;
  readonly selectedIds: readonly string[];
  readonly tool: TransformTool;
  readonly space: TransformSpace;
  readonly snap: PlaceableSnapSettings;
  readonly editable: boolean;
  readonly onSelect: (id: string, additive: boolean) => void;
  readonly onClearSelection: () => void;
  readonly onGestureStart: () => void;
  readonly onTransformPreview: (patches: readonly ElementTransformPatch[]) => void;
  readonly onGestureEnd: () => void;
  readonly onError?: (message: string) => void;
}

interface PrimitiveCatalogEntry {
  readonly primitiveId: number;
  readonly mesh: THREE.Mesh;
}

interface SourceCatalog {
  readonly byNode: ReadonlyMap<number, readonly PrimitiveCatalogEntry[]>;
  readonly sourceRoot: THREE.Object3D;
}

interface ElementRuntime {
  element: PlaceableAuthoringElement;
  readonly control: THREE.Object3D;
  readonly payload: THREE.Object3D;
}

function applyPatchesToRuntime(
  runtime: ViewportRuntime,
  patches: readonly ElementTransformPatch[],
) {
  for (const patch of patches) {
    const entry = runtime.elements.get(patch.id);
    if (!entry) continue;
    entry.element = { ...entry.element, transform: patch.transform };
    setAuthoredTransform(entry.control, entry.payload, patch.transform);
    entry.control.updateMatrixWorld(true);
  }
}

interface ViewportRuntime {
  readonly scene: THREE.Scene;
  readonly camera: THREE.PerspectiveCamera;
  readonly renderer: THREE.WebGLRenderer;
  readonly orbit: OrbitControls;
  readonly gizmo: TransformControls;
  readonly authoredRoot: THREE.Group;
  readonly selectionProxy: THREE.Object3D;
  readonly raycaster: THREE.Raycaster;
  readonly pointer: THREE.Vector2;
  catalog?: SourceCatalog;
  elements: Map<string, ElementRuntime>;
  highlights: THREE.BoxHelper[];
  dragging: boolean;
  dragStart?: {
    proxyWorld: THREE.Matrix4;
    elementWorlds: Map<string, THREE.Matrix4>;
  };
}

const toolMode = {
  TRANSLATE: "translate",
  ROTATE: "rotate",
  SCALE: "scale",
} as const;

function disposeMaterial(material: THREE.Material) {
  material.dispose();
}

function disposeObject(root: THREE.Object3D, disposeTextures = false) {
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    object.geometry.dispose();
    const materials = Array.isArray(object.material) ? object.material : [object.material];
    for (const material of materials) {
      if (disposeTextures) {
        for (const value of Object.values(material)) {
          if (value instanceof THREE.Texture) value.dispose();
        }
      }
      disposeMaterial(material);
    }
  });
}

export function connectedTriangleComponents(geometry: THREE.BufferGeometry): readonly number[][] {
  const position = geometry.getAttribute("position");
  if (!position) return [];
  const sourceIndex = geometry.getIndex();
  const indices = sourceIndex
    ? Array.from(sourceIndex.array, Number)
    : Array.from({ length: position.count }, (_, index) => index);
  const triangleCount = Math.floor(indices.length / 3);
  const byVertex = new Map<number, number[]>();
  for (let triangle = 0; triangle < triangleCount; triangle += 1) {
    for (const vertex of indices.slice(triangle * 3, triangle * 3 + 3)) {
      const list = byVertex.get(vertex) ?? [];
      list.push(triangle);
      byVertex.set(vertex, list);
    }
  }
  const visited = new Set<number>();
  const output: number[][] = [];
  for (let seed = 0; seed < triangleCount; seed += 1) {
    if (visited.has(seed)) continue;
    const pending = [seed];
    const component: number[] = [];
    visited.add(seed);
    while (pending.length) {
      const triangle = pending.pop()!;
      component.push(triangle);
      for (const vertex of indices.slice(triangle * 3, triangle * 3 + 3)) {
        for (const neighbor of byVertex.get(vertex) ?? []) {
          if (!visited.has(neighbor)) {
            visited.add(neighbor);
            pending.push(neighbor);
          }
        }
      }
    }
    component.sort((left, right) => left - right);
    output.push(component);
  }
  return output;
}

export function componentGeometry(
  geometry: THREE.BufferGeometry,
  componentIndex: number,
) {
  const position = geometry.getAttribute("position");
  const sourceIndex = geometry.getIndex();
  const indices = sourceIndex
    ? Array.from(sourceIndex.array, Number)
    : Array.from({ length: position.count }, (_, index) => index);
  const component = connectedTriangleComponents(geometry)[componentIndex];
  if (!component) throw new Error(`PLACEABLE-VIEWPORT-COMPONENT-MISSING:${componentIndex}`);
  const output = geometry.clone();
  output.setIndex(component.flatMap((triangle) => indices.slice(triangle * 3, triangle * 3 + 3)));
  output.computeBoundingBox();
  output.computeBoundingSphere();
  return output;
}

function cloneMaterial(
  source: THREE.Material | readonly THREE.Material[],
  renderable: boolean,
) {
  const materials = (Array.isArray(source) ? source : [source]).map((material) => {
    const clone = material.clone();
    if (!renderable) {
      clone.transparent = true;
      clone.opacity = 0.28;
      if ("wireframe" in clone) (clone as THREE.MeshStandardMaterial).wireframe = true;
    }
    return clone;
  });
  return Array.isArray(source) ? materials : materials[0];
}

function bakeSourceMesh(
  source: THREE.Mesh,
  componentIndex: number | null,
  renderable: boolean,
) {
  const geometry = componentIndex === null
    ? source.geometry.clone()
    : componentGeometry(source.geometry, componentIndex);
  geometry.applyMatrix4(source.matrixWorld);
  const mesh = new THREE.Mesh(geometry, cloneMaterial(source.material, renderable));
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  return mesh;
}

function sourceCatalog(root: THREE.Object3D, inspection: PlaceableElementInspection): SourceCatalog {
  root.updateMatrixWorld(true);
  const meshes: THREE.Mesh[] = [];
  root.traverse((object) => {
    if (object instanceof THREE.Mesh) meshes.push(object);
  });
  const byNode = new Map<number, PrimitiveCatalogEntry[]>();
  let cursor = 0;
  for (const node of inspection.nodes) {
    const entries: PrimitiveCatalogEntry[] = [];
    for (const primitive of node.primitives) {
      const mesh = meshes[cursor++];
      if (!mesh) throw new Error(`PLACEABLE-VIEWPORT-PRIMITIVE-MISSING:${primitive.primitiveId}`);
      entries.push({ primitiveId: primitive.primitiveId, mesh });
    }
    byNode.set(node.nodeId, entries);
  }
  return { byNode, sourceRoot: root };
}

function setAuthoredTransform(
  control: THREE.Object3D,
  payload: THREE.Object3D,
  transform: PlaceableElementTransform,
) {
  control.position.set(
    transform.translation[0] + transform.pivot[0],
    transform.translation[1] + transform.pivot[1],
    transform.translation[2] + transform.pivot[2],
  );
  control.quaternion.set(...transform.rotationXyzw).normalize();
  control.scale.set(...transform.scale);
  payload.position.set(-transform.pivot[0], -transform.pivot[1], -transform.pivot[2]);
  control.updateMatrix();
}

function transformFromControl(
  control: THREE.Object3D,
  pivot: readonly [number, number, number],
): PlaceableElementTransform {
  return {
    translation: [
      control.position.x - pivot[0],
      control.position.y - pivot[1],
      control.position.z - pivot[2],
    ],
    rotationXyzw: control.quaternion.toArray() as [number, number, number, number],
    scale: control.scale.toArray() as [number, number, number],
    pivot: [...pivot],
  };
}

function createElementRuntime(
  element: PlaceableAuthoringElement,
  catalog: SourceCatalog,
): ElementRuntime {
  const control = new THREE.Group();
  control.name = `authoring:${element.id}`;
  control.userData.authoringElementId = element.id;
  const payload = new THREE.Group();
  payload.userData.authoringElementId = element.id;
  control.add(payload);
  setAuthoredTransform(control, payload, element.transform);
  control.visible = !element.flags.hidden;

  if (element.source) {
    const primitives = catalog.byNode.get(element.source.nodeId) ?? [];
    for (const primitive of primitives) {
      if (
        element.source.primitiveId !== null
        && primitive.primitiveId !== element.source.primitiveId
      ) continue;
      const mesh = bakeSourceMesh(
        primitive.mesh,
        element.source.componentIndex,
        element.flags.renderable,
      );
      mesh.userData.authoringElementId = element.id;
      mesh.castShadow = element.flags.castShadow;
      payload.add(mesh);
    }
  }
  return { element, control, payload };
}

function clearAuthored(runtime: ViewportRuntime) {
  runtime.gizmo.detach();
  runtime.highlights.forEach((highlight) => {
    runtime.scene.remove(highlight);
    highlight.geometry.dispose();
    (highlight.material as THREE.Material).dispose();
  });
  runtime.highlights = [];
  for (const child of [...runtime.authoredRoot.children]) {
    runtime.authoredRoot.remove(child);
    disposeObject(child);
  }
  runtime.elements.clear();
}

function rebuildAuthored(
  runtime: ViewportRuntime,
  document: PlaceableAuthoringDocument,
  selectedIds: readonly string[],
  editable: boolean,
) {
  const catalog = runtime.catalog;
  if (!catalog) return;
  clearAuthored(runtime);
  const live = document.elements.filter((element) => !element.deleted);
  for (const element of live) {
    runtime.elements.set(element.id, createElementRuntime(element, catalog));
  }
  for (const element of live) {
    const entry = runtime.elements.get(element.id)!;
    const parent = element.parentId ? runtime.elements.get(element.parentId) : undefined;
    (parent?.control ?? runtime.authoredRoot).add(entry.control);
  }
  runtime.authoredRoot.updateMatrixWorld(true);
  attachSelection(runtime, document, selectedIds, editable);
}

function attachSelection(
  runtime: ViewportRuntime,
  document: PlaceableAuthoringDocument,
  selectedIds: readonly string[],
  editable: boolean,
) {
  runtime.gizmo.detach();
  runtime.highlights.forEach((highlight) => {
    runtime.scene.remove(highlight);
    highlight.geometry.dispose();
    (highlight.material as THREE.Material).dispose();
  });
  runtime.highlights = [];
  const selected = selectedIds.flatMap((id) => {
    const entry = runtime.elements.get(id);
    return entry ? [entry] : [];
  });
  for (const entry of selected) {
    const highlight = new THREE.BoxHelper(entry.control, 0x4de1ee);
    runtime.scene.add(highlight);
    runtime.highlights.push(highlight);
  }
  if (!editable || !selected.length || selected.some((entry) => entry.element.flags.locked)) return;
  if (selected.length === 1) {
    runtime.gizmo.attach(selected[0].control);
    return;
  }
  const center = selected
    .map((entry) => entry.control.getWorldPosition(new THREE.Vector3()))
    .reduce((sum, value) => sum.add(value), new THREE.Vector3())
    .multiplyScalar(1 / selected.length);
  runtime.selectionProxy.position.copy(center);
  runtime.selectionProxy.quaternion.identity();
  runtime.selectionProxy.scale.setScalar(1);
  runtime.selectionProxy.updateMatrixWorld(true);
  runtime.gizmo.attach(runtime.selectionProxy);
}

function patchesFromGizmo(
  runtime: ViewportRuntime,
  document: PlaceableAuthoringDocument,
  selectedIds: readonly string[],
): ElementTransformPatch[] {
  const selected = selectedIds.flatMap((id) => {
    const entry = runtime.elements.get(id);
    return entry ? [entry] : [];
  });
  if (!selected.length) return [];
  if (selected.length === 1) {
    const entry = selected[0];
    return [{
      id: entry.element.id,
      transform: transformFromControl(entry.control, entry.element.transform.pivot),
    }];
  }
  const start = runtime.dragStart;
  if (!start) return [];
  runtime.selectionProxy.updateMatrixWorld(true);
  const delta = runtime.selectionProxy.matrixWorld.clone().multiply(
    start.proxyWorld.clone().invert(),
  );
  return selected.map((entry) => {
    const initial = start.elementWorlds.get(entry.element.id) ?? entry.control.matrixWorld;
    const world = delta.clone().multiply(initial);
    const parent = entry.control.parent;
    const local = parent
      ? parent.matrixWorld.clone().invert().multiply(world)
      : world;
    const position = new THREE.Vector3();
    const quaternion = new THREE.Quaternion();
    const scale = new THREE.Vector3();
    local.decompose(position, quaternion, scale);
    return {
      id: entry.element.id,
      transform: {
        translation: position.toArray() as [number, number, number],
        rotationXyzw: quaternion.toArray() as [number, number, number, number],
        scale: scale.toArray() as [number, number, number],
        pivot: [0, 0, 0],
      },
    };
  });
}

function elementId(object: THREE.Object3D | null | undefined): string | undefined {
  let cursor = object;
  while (cursor) {
    const id = cursor.userData.authoringElementId as string | undefined;
    if (id) return id;
    cursor = cursor.parent;
  }
  return undefined;
}

function applyNonGridSnap(
  runtime: ViewportRuntime,
  document: PlaceableAuthoringDocument,
  selectedIds: readonly string[],
  mode: PlaceableSnapSettings["mode"],
): ElementTransformPatch[] {
  if (!["SURFACE", "VERTEX", "ELEMENT"].includes(mode)) return [];
  runtime.raycaster.setFromCamera(runtime.pointer, runtime.camera);
  const selected = new Set(selectedIds);
  const hit = runtime.raycaster.intersectObject(runtime.authoredRoot, true)
    .find((intersection) => {
      const id = elementId(intersection.object);
      return id && !selected.has(id);
    });
  if (!hit) return [];
  let target = hit.point.clone();
  if (mode === "VERTEX" && hit.object instanceof THREE.Mesh && hit.face) {
    const position = hit.object.geometry.getAttribute("position");
    const candidates = [hit.face.a, hit.face.b, hit.face.c].map((index) => (
      new THREE.Vector3().fromBufferAttribute(position, index).applyMatrix4(hit.object.matrixWorld)
    ));
    target = candidates.sort((left, right) => left.distanceToSquared(hit.point) - right.distanceToSquared(hit.point))[0];
  } else if (mode === "ELEMENT") {
    const targetId = elementId(hit.object);
    const targetElement = targetId ? runtime.elements.get(targetId) : undefined;
    if (targetElement) target = new THREE.Box3().setFromObject(targetElement.control).getCenter(new THREE.Vector3());
  }
  const selectedEntries = selectedIds.flatMap((id) => {
    const entry = runtime.elements.get(id);
    return entry ? [entry] : [];
  });
  if (!selectedEntries.length) return [];
  const anchor = selectedEntries
    .map((entry) => entry.control.getWorldPosition(new THREE.Vector3()))
    .reduce((sum, value) => sum.add(value), new THREE.Vector3())
    .multiplyScalar(1 / selectedEntries.length);
  const delta = new THREE.Matrix4().makeTranslation(...target.sub(anchor).toArray());
  return selectedEntries.map((entry) => {
    const world = delta.clone().multiply(entry.control.matrixWorld);
    const parent = entry.control.parent;
    const local = parent ? parent.matrixWorld.clone().invert().multiply(world) : world;
    const position = new THREE.Vector3();
    const rotation = new THREE.Quaternion();
    const scale = new THREE.Vector3();
    local.decompose(position, rotation, scale);
    const pivot = entry.element.transform.pivot;
    return {
      id: entry.element.id,
      transform: {
        translation: [
          position.x - pivot[0],
          position.y - pivot[1],
          position.z - pivot[2],
        ] as [number, number, number],
        rotationXyzw: rotation.toArray() as [number, number, number, number],
        scale: scale.toArray() as [number, number, number],
        pivot: [...pivot] as [number, number, number],
      },
    };
  });
}

function fitCamera(runtime: ViewportRuntime) {
  const bounds = new THREE.Box3().setFromObject(runtime.authoredRoot);
  const center = bounds.isEmpty() ? new THREE.Vector3() : bounds.getCenter(new THREE.Vector3());
  const size = bounds.isEmpty() ? 1 : Math.max(bounds.getSize(new THREE.Vector3()).length(), 1);
  runtime.camera.position.copy(center).add(new THREE.Vector3(size, size * 0.7, size));
  runtime.camera.near = Math.max(size / 1000, 0.001);
  runtime.camera.far = Math.max(size * 100, 100);
  runtime.camera.updateProjectionMatrix();
  runtime.orbit.target.copy(center);
  runtime.orbit.update();
}

export function PlaceableAuthoringViewport({
  file,
  dependency,
  document,
  inspection,
  selectedIds,
  tool,
  space,
  snap,
  editable,
  onSelect,
  onClearSelection,
  onGestureStart,
  onTransformPreview,
  onGestureEnd,
  onError,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const runtimeRef = useRef<ViewportRuntime | undefined>(undefined);
  const latestRef = useRef({
    document,
    selectedIds,
    snap,
    onSelect,
    onClearSelection,
    onGestureStart,
    onTransformPreview,
    onGestureEnd,
  });
  latestRef.current = {
    document,
    selectedIds,
    snap,
    onSelect,
    onClearSelection,
    onGestureStart,
    onTransformPreview,
    onGestureEnd,
  };

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x091219);
    const camera = new THREE.PerspectiveCamera(45, 1, 0.001, 10_000);
    const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    renderer.shadowMap.enabled = true;
    const orbit = new OrbitControls(camera, canvas);
    orbit.enableDamping = true;
    const gizmo = new TransformControls(camera, canvas);
    const authoredRoot = new THREE.Group();
    const selectionProxy = new THREE.Object3D();
    scene.add(authoredRoot, selectionProxy, gizmo.getHelper());
    scene.add(new THREE.HemisphereLight(0xffffff, 0x213142, 2.2));
    const key = new THREE.DirectionalLight(0xffffff, 2.4);
    key.position.set(5, 8, 5);
    key.castShadow = true;
    scene.add(key);
    const grid = new THREE.GridHelper(20, 40, 0x3c6572, 0x213640);
    scene.add(grid);
    const axes = new THREE.AxesHelper(1);
    scene.add(axes);

    const runtime: ViewportRuntime = {
      scene,
      camera,
      renderer,
      orbit,
      gizmo,
      authoredRoot,
      selectionProxy,
      raycaster: new THREE.Raycaster(),
      pointer: new THREE.Vector2(),
      elements: new Map(),
      highlights: [],
      dragging: false,
    };
    runtimeRef.current = runtime;

    const resize = () => {
      const width = Math.max(canvas.clientWidth, 1);
      const height = Math.max(canvas.clientHeight, 1);
      renderer.setSize(width, height, false);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
    };
    const observer = new ResizeObserver(resize);
    observer.observe(canvas);
    resize();

    const updatePointer = (event: PointerEvent) => {
      const rect = canvas.getBoundingClientRect();
      runtime.pointer.set(
        ((event.clientX - rect.left) / Math.max(rect.width, 1)) * 2 - 1,
        -((event.clientY - rect.top) / Math.max(rect.height, 1)) * 2 + 1,
      );
    };
    const select = (event: PointerEvent) => {
      updatePointer(event);
      if (runtime.dragging || gizmo.axis) return;
      runtime.raycaster.setFromCamera(runtime.pointer, camera);
      const id = elementId(runtime.raycaster.intersectObject(authoredRoot, true)[0]?.object);
      if (id) latestRef.current.onSelect(id, event.ctrlKey || event.metaKey || event.shiftKey);
      else latestRef.current.onClearSelection();
    };
    canvas.addEventListener("pointermove", updatePointer);
    canvas.addEventListener("pointerdown", select);

    const draggingChanged = (event: { value: unknown }) => {
      orbit.enabled = !event.value;
      runtime.dragging = Boolean(event.value);
    };
    const mouseDown = () => {
      runtime.dragging = true;
      latestRef.current.onGestureStart();
      const selected = latestRef.current.selectedIds.flatMap((id) => {
        const entry = runtime.elements.get(id);
        return entry ? [entry] : [];
      });
      runtime.selectionProxy.updateMatrixWorld(true);
      runtime.dragStart = {
        proxyWorld: runtime.selectionProxy.matrixWorld.clone(),
        elementWorlds: new Map(selected.map((entry) => {
          entry.control.updateMatrixWorld(true);
          return [entry.element.id, entry.control.matrixWorld.clone()];
        })),
      };
    };
    const objectChange = () => {
      const patches = patchesFromGizmo(
        runtime,
        latestRef.current.document,
        latestRef.current.selectedIds,
      );
      if (patches.length) {
        if (latestRef.current.selectedIds.length > 1) applyPatchesToRuntime(runtime, patches);
        else {
          const entry = runtime.elements.get(patches[0].id);
          if (entry) entry.element = { ...entry.element, transform: patches[0].transform };
        }
        latestRef.current.onTransformPreview(patches);
      }
      runtime.highlights.forEach((highlight) => highlight.update());
    };
    const mouseUp = () => {
      const snapped = applyNonGridSnap(
        runtime,
        latestRef.current.document,
        latestRef.current.selectedIds,
        latestRef.current.snap.mode,
      );
      if (snapped.length) {
        applyPatchesToRuntime(runtime, snapped);
        latestRef.current.onTransformPreview(snapped);
      }
      runtime.dragging = false;
      runtime.dragStart = undefined;
      latestRef.current.onGestureEnd();
    };
    gizmo.addEventListener("dragging-changed", draggingChanged);
    gizmo.addEventListener("mouseDown", mouseDown);
    gizmo.addEventListener("objectChange", objectChange);
    gizmo.addEventListener("mouseUp", mouseUp);

    let frame = 0;
    const render = () => {
      orbit.update();
      renderer.render(scene, camera);
      frame = requestAnimationFrame(render);
    };
    render();

    return () => {
      cancelAnimationFrame(frame);
      canvas.removeEventListener("pointermove", updatePointer);
      canvas.removeEventListener("pointerdown", select);
      observer.disconnect();
      gizmo.removeEventListener("dragging-changed", draggingChanged);
      gizmo.removeEventListener("mouseDown", mouseDown);
      gizmo.removeEventListener("objectChange", objectChange);
      gizmo.removeEventListener("mouseUp", mouseUp);
      clearAuthored(runtime);
      if (runtime.catalog) disposeObject(runtime.catalog.sourceRoot, true);
      gizmo.dispose();
      orbit.dispose();
      grid.geometry.dispose();
      (grid.material as THREE.Material).dispose();
      axes.geometry.dispose();
      (axes.material as THREE.Material).dispose();
      renderer.dispose();
      runtimeRef.current = undefined;
    };
  }, []);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    let cancelled = false;
    const manager = new THREE.LoadingManager();
    manager.setURLModifier((url) => {
      if (url.startsWith("blob:") || url.startsWith("data:")) return url;
      throw new Error("Placeable authoring forbids external GLB resource URLs");
    });
    void file.arrayBuffer()
      .then((bytes) => new Promise<THREE.Object3D>((resolve, reject) => {
        new GLTFLoader(manager).parse(bytes, "", (gltf) => resolve(gltf.scene), reject);
      }))
      .then((root) => {
        if (cancelled) return disposeObject(root, true);
        if (runtime.catalog) disposeObject(runtime.catalog.sourceRoot, true);
        runtime.catalog = sourceCatalog(root, inspection);
        rebuildAuthored(runtime, latestRef.current.document, latestRef.current.selectedIds, editable);
        fitCamera(runtime);
      })
      .catch((error: unknown) => {
        if (!cancelled) onError?.(error instanceof Error ? error.message : String(error));
      });
    return () => {
      cancelled = true;
    };
  }, [dependency, editable, file, inspection, onError]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime || !runtime.catalog || runtime.dragging) return;
    rebuildAuthored(runtime, document, selectedIds, editable);
  }, [document, editable]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime || !runtime.catalog || runtime.dragging) return;
    attachSelection(runtime, document, selectedIds, editable);
  }, [document, editable, selectedIds]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!runtime) return;
    runtime.gizmo.setMode(toolMode[tool]);
    runtime.gizmo.setSpace(space.toLowerCase() as "local" | "world");
    runtime.gizmo.setTranslationSnap(snap.mode === "GRID" ? snap.translationStep : null);
    runtime.gizmo.setRotationSnap(
      snap.mode === "GRID" ? THREE.MathUtils.degToRad(snap.rotationDegrees) : null,
    );
    runtime.gizmo.setScaleSnap(snap.mode === "GRID" ? snap.scaleStep : null);
  }, [snap, space, tool]);

  return (
    <section className="placeable-authoring-viewport" aria-label="Editable placeable 3D viewport">
      <header>
        <strong>{editable ? "EDITED" : "ORIGINAL"}</strong>
        <span>Three.js 3D · orbit, select and W/E/R gizmo</span>
      </header>
      <canvas ref={canvasRef} tabIndex={0} />
      <div className="placeable-authoring-viewport__legend" aria-hidden="true">
        <span>X</span><span>Y</span><span>Z</span>
      </div>
    </section>
  );
}
