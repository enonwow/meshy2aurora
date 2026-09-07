import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import {
  AnimationPlaybackRuntime,
  inventoryAnimationClips,
  type AnimationPlaybackSnapshot,
} from "./animationPlayback";
import {
  availableSceneOverlays,
  defaultSceneOverlays,
  groundedGridY,
  sceneObjectBounds,
  SceneOverlayRuntime,
  type SceneOverlayKind,
} from "./sceneOverlays";
import type { ReadbackRigInventoryV1, RigNodeDescriptorV1 } from "./rigOverlay";
import type { ModelPartRef } from "./types";

export interface SceneViewportAsset {
  root: THREE.Object3D;
  animations?: readonly THREE.AnimationClip[];
}

export interface SceneTriangleSelectionV2 {
  readonly object: THREE.Mesh;
  readonly faceIndex: number;
}

export interface SceneAnimationBindingClipV1 {
  readonly clipName: string;
  readonly animationSource: string;
  readonly controlledNodeCount: number;
  readonly matchedNodeCount: number;
  readonly unmatchedNodeNames: readonly string[];
}

export interface SceneAnimationBindingV1 {
  readonly schemaVersion: 1;
  readonly rigSource: string;
  readonly clips: readonly SceneAnimationBindingClipV1[];
}

interface SceneViewportProps {
  provenance: "SOURCE" | "AURORA IR" | "READBACK";
  detail: string;
  buildRoot: () => Promise<THREE.Object3D | SceneViewportAsset>;
  dependency: unknown;
  onSelectPart?: (part?: ModelPartRef) => void;
  onSelectIntersection?: (
    intersection: THREE.Intersection<THREE.Object3D> | undefined,
    event: PointerEvent,
  ) => void;
  onSelectTriangleRectangle?: (
    triangles: readonly SceneTriangleSelectionV2[],
    event: PointerEvent,
  ) => void;
  onSelectRigNode?: (node?: RigNodeDescriptorV1) => void;
  onError?: (message: string) => void;
  tools?: {
    animationPlayback?: boolean;
    overlays?: boolean;
  };
  animationUnavailableReason?: string;
  animationBinding?: SceneAnimationBindingV1;
}

interface AnimationUiState extends AnimationPlaybackSnapshot {
  inventory: ReturnType<typeof inventoryAnimationClips>;
  loading: boolean;
}

const emptyAnimationUi: AnimationUiState = {
  inventory: [],
  loading: false,
  selectedClipIndex: null,
  playing: false,
  loop: true,
  timeSeconds: 0,
  durationSeconds: 0,
  playbackRate: 1,
  poseMode: "ANIMATED",
};

const overlayLabels: Record<SceneOverlayKind, string> = {
  mesh: "Mesh",
  grid: "Grid",
  axes: "Axes",
  bones: "Kości",
  joints: "Jointy",
  helpers: "Helpery / attachmenty",
  labels: "Etykiety",
  xray: "Kości przez model (X-Ray)",
  bounds: "Bounds",
  wireframe: "Wireframe",
  skinClusters: "Klastry skinningu",
};

function disposeMaterial(material: THREE.Material) {
  Object.values(material).forEach((value) => {
    if (value instanceof THREE.Texture) value.dispose();
  });
  material.dispose();
}

export function disposeObjectResources(root: THREE.Object3D) {
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    object.geometry.dispose();
    const materials = Array.isArray(object.material) ? object.material : [object.material];
    materials.forEach(disposeMaterial);
    if (object instanceof THREE.SkinnedMesh) object.skeleton.dispose();
  });
}

function modelPart(object?: THREE.Object3D | null): ModelPartRef | undefined {
  let candidate = object;
  while (candidate) {
    const part = candidate.userData.modelPart as ModelPartRef | undefined;
    if (part) return part;
    candidate = candidate.parent;
  }
  return undefined;
}

function fitCamera(camera: THREE.PerspectiveCamera, controls: OrbitControls, root: THREE.Object3D) {
  const bounds = sceneObjectBounds(root);
  const center = bounds.isEmpty() ? new THREE.Vector3() : bounds.getCenter(new THREE.Vector3());
  const size = bounds.isEmpty() ? new THREE.Vector3(1, 1, 1) : bounds.getSize(new THREE.Vector3());
  const distance = Math.max(size.length(), 1);
  camera.position.copy(center).add(new THREE.Vector3(distance, distance * 0.7, distance));
  camera.near = Math.max(distance / 1000, 0.001);
  camera.far = Math.max(distance * 100, 100);
  camera.updateProjectionMatrix();
  controls.target.copy(center);
  controls.update();
}

export function SceneViewport({
  provenance,
  detail,
  buildRoot,
  dependency,
  onSelectPart,
  onSelectIntersection,
  onSelectTriangleRectangle,
  onSelectRigNode,
  onError,
  tools,
  animationUnavailableReason,
  animationBinding,
}: SceneViewportProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRuntimeRef = useRef<AnimationPlaybackRuntime | undefined>(undefined);
  const overlayRuntimeRef = useRef<SceneOverlayRuntime | undefined>(undefined);
  const [animationUi, setAnimationUi] = useState<AnimationUiState>(emptyAnimationUi);
  const [availableOverlays, setAvailableOverlays] = useState<SceneOverlayKind[]>([]);
  const [enabledOverlays, setEnabledOverlays] = useState<SceneOverlayKind[]>([]);
  const [rigInventory, setRigInventory] = useState<ReadbackRigInventoryV1 | undefined>();
  const [selectedRigNode, setSelectedRigNode] = useState<RigNodeDescriptorV1 | undefined>();
  const animationPlaybackEnabled = tools?.animationPlayback === true;
  const overlaysEnabled = tools?.overlays === true;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    let stopped = false;
    let root: THREE.Object3D | undefined;
    let animationRuntime: AnimationPlaybackRuntime | undefined;
    let overlayRuntime: SceneOverlayRuntime | undefined;
    let staticGrid: THREE.GridHelper | undefined;
    let frame = 0;
    let lastAnimationUiUpdate = 0;
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0a121a);
    const camera = new THREE.PerspectiveCamera(45, 1, 0.01, 10_000);
    const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    const controls = new OrbitControls(camera, canvas);
    controls.enableDamping = true;
    scene.add(new THREE.HemisphereLight(0xffffff, 0x203044, 2));
    const key = new THREE.DirectionalLight(0xffffff, 2);
    key.position.set(4, 6, 3);
    scene.add(key);
    if (!overlaysEnabled) {
      staticGrid = new THREE.GridHelper(10, 20, 0x38536b, 0x1e3040);
      scene.add(staticGrid);
    }

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

    const raycaster = new THREE.Raycaster();
    const pointer = new THREE.Vector2();
    let rectangleStart: { x: number; y: number } | undefined;
    const select = (event: PointerEvent) => {
      const rect = canvas.getBoundingClientRect();
      pointer.set(
        ((event.clientX - rect.left) / rect.width) * 2 - 1,
        -((event.clientY - rect.top) / rect.height) * 2 + 1,
      );
      raycaster.setFromCamera(pointer, camera);
      const intersection = raycaster.intersectObjects(scene.children, true)[0];
      const rigNode = overlayRuntime?.rigDescriptorFromObject(intersection?.object);
      setSelectedRigNode(rigNode);
      onSelectRigNode?.(rigNode);
      onSelectPart?.(modelPart(intersection?.object));
      onSelectIntersection?.(intersection, event);
    };
    canvas.addEventListener("pointerdown", select);
    const beginRectangle = (event: PointerEvent) => {
      if (!onSelectTriangleRectangle || !event.shiftKey) return;
      rectangleStart = { x: event.clientX, y: event.clientY };
      controls.enabled = false;
    };
    const finishRectangle = (event: PointerEvent) => {
      const start = rectangleStart;
      rectangleStart = undefined;
      controls.enabled = true;
      if (!start || !root || !onSelectTriangleRectangle) return;
      const rect = canvas.getBoundingClientRect();
      const left = Math.min(start.x, event.clientX) - rect.left;
      const right = Math.max(start.x, event.clientX) - rect.left;
      const top = Math.min(start.y, event.clientY) - rect.top;
      const bottom = Math.max(start.y, event.clientY) - rect.top;
      if (right - left < 3 || bottom - top < 3) return;
      root.updateWorldMatrix(true, true);
      const centroid = new THREE.Vector3();
      const projected = new THREE.Vector3();
      const a = new THREE.Vector3();
      const b = new THREE.Vector3();
      const c = new THREE.Vector3();
      const hits: SceneTriangleSelectionV2[] = [];
      root.traverse((object) => {
        if (!(object instanceof THREE.Mesh) || object.userData.previewOverlay) return;
        const position = object.geometry.getAttribute("position");
        if (!(position instanceof THREE.BufferAttribute)) return;
        const indices = object.geometry.index;
        const triangleCount = indices ? indices.count / 3 : position.count / 3;
        for (let faceIndex = 0; faceIndex < triangleCount; faceIndex += 1) {
          const ia = indices ? indices.getX(faceIndex * 3) : faceIndex * 3;
          const ib = indices ? indices.getX(faceIndex * 3 + 1) : faceIndex * 3 + 1;
          const ic = indices ? indices.getX(faceIndex * 3 + 2) : faceIndex * 3 + 2;
          a.fromBufferAttribute(position, ia);
          b.fromBufferAttribute(position, ib);
          c.fromBufferAttribute(position, ic);
          centroid.copy(a).add(b).add(c).multiplyScalar(1 / 3).applyMatrix4(object.matrixWorld);
          projected.copy(centroid).project(camera);
          if (projected.z < -1 || projected.z > 1) continue;
          const x = (projected.x + 1) * 0.5 * rect.width;
          const y = (1 - projected.y) * 0.5 * rect.height;
          if (x >= left && x <= right && y >= top && y <= bottom) {
            hits.push({ object, faceIndex });
          }
        }
      });
      onSelectTriangleRectangle(hits, event);
    };
    canvas.addEventListener("pointerdown", beginRectangle);
    canvas.addEventListener("pointerup", finishRectangle);

    if (animationPlaybackEnabled) setAnimationUi({ ...emptyAnimationUi, loading: true });
    if (overlaysEnabled) {
      setAvailableOverlays([]);
      setEnabledOverlays([]);
      setRigInventory(undefined);
      setSelectedRigNode(undefined);
      onSelectRigNode?.(undefined);
    }

    void buildRoot()
      .then((value) => {
        const asset = value instanceof THREE.Object3D ? { root: value, animations: [] } : value;
        if (stopped) return disposeObjectResources(asset.root);
        root = asset.root;
        scene.add(asset.root);
        if (staticGrid) {
          staticGrid.position.y = groundedGridY(sceneObjectBounds(asset.root));
        }
        fitCamera(camera, controls, asset.root);

        if (animationPlaybackEnabled) {
          const clips = asset.animations ?? [];
          animationRuntime = new AnimationPlaybackRuntime(asset.root, clips);
          animationRuntimeRef.current = animationRuntime;
          const inventory = inventoryAnimationClips(clips);
          const snapshot = inventory.length ? animationRuntime.selectClip(0) : animationRuntime.snapshot();
          setAnimationUi({ inventory, loading: false, ...snapshot });
        }

        if (overlaysEnabled) {
          overlayRuntime = new SceneOverlayRuntime(scene, asset.root);
          overlayRuntimeRef.current = overlayRuntime;
          const available = availableSceneOverlays(asset.root);
          setAvailableOverlays(available);
          const defaults = defaultSceneOverlays(available);
          defaults.forEach((kind) => overlayRuntime?.set(kind, true));
          setEnabledOverlays(defaults);
          setRigInventory(overlayRuntime.rigInventory());
        }
      })
      .catch((error: unknown) => {
        if (!stopped) {
          if (animationPlaybackEnabled) setAnimationUi({ ...emptyAnimationUi, loading: false });
          onError?.(error instanceof Error ? error.message : String(error));
        }
      });

    let previousFrameTime = performance.now();
    const render = (frameTime = performance.now()) => {
      const deltaSeconds = Math.max(frameTime - previousFrameTime, 0) / 1_000;
      previousFrameTime = frameTime;
      controls.update();
      if (animationRuntime) {
        const snapshot = animationRuntime.update(deltaSeconds);
        if (frameTime - lastAnimationUiUpdate >= 80) {
          lastAnimationUiUpdate = frameTime;
          setAnimationUi((current) => ({ ...current, ...snapshot }));
        }
      }
      overlayRuntime?.update();
      renderer.render(scene, camera);
      frame = requestAnimationFrame(render);
    };
    render();

    return () => {
      stopped = true;
      cancelAnimationFrame(frame);
      canvas.removeEventListener("pointerdown", select);
      canvas.removeEventListener("pointerdown", beginRectangle);
      canvas.removeEventListener("pointerup", finishRectangle);
      observer.disconnect();
      controls.dispose();
      animationRuntime?.dispose();
      overlayRuntime?.dispose();
      if (animationRuntimeRef.current === animationRuntime) animationRuntimeRef.current = undefined;
      if (overlayRuntimeRef.current === overlayRuntime) overlayRuntimeRef.current = undefined;
      if (root) disposeObjectResources(root);
      if (staticGrid) {
        staticGrid.geometry.dispose();
        const material = Array.isArray(staticGrid.material) ? staticGrid.material : [staticGrid.material];
        material.forEach((entry) => entry.dispose());
      }
      renderer.dispose();
    };
  }, [animationPlaybackEnabled, buildRoot, dependency, onSelectIntersection, onSelectPart, onSelectRigNode, onSelectTriangleRectangle, onError, overlaysEnabled]);

  const updateAnimation = (snapshot: AnimationPlaybackSnapshot) => {
    setAnimationUi((current) => ({ ...current, ...snapshot }));
  };

  const selectClip = (index: number) => {
    const runtime = animationRuntimeRef.current;
    if (runtime) updateAnimation(runtime.selectClip(index));
  };

  const toggleOverlay = (kind: SceneOverlayKind) => {
    setEnabledOverlays((current) => {
      const enabled = !current.includes(kind);
      overlayRuntimeRef.current?.set(kind, enabled);
      return enabled ? [...current, kind] : current.filter((candidate) => candidate !== kind);
    });
  };

  const selectedBinding = animationBinding?.clips[animationUi.selectedClipIndex ?? 0];

  return (
    <section className="viewport" aria-label={`${provenance} model viewport`}>
      <header><strong>{provenance}</strong><span>{detail}</span></header>
      <canvas ref={canvasRef} />
      {overlaysEnabled && (
        <fieldset className="viewport__overlays" aria-label="Debug overlays">
          <legend>Debug overlays</legend>
          {availableOverlays.length === 0
            ? <span>Overlay inventory is available after the model loads.</span>
            : availableOverlays.map((kind) => (
              <label key={kind}>
                <input
                  type="checkbox"
                  checked={enabledOverlays.includes(kind)}
                  onChange={() => toggleOverlay(kind)}
                />
                {overlayLabels[kind]}
              </label>
            ))}
        </fieldset>
      )}
      {rigInventory && (
        <section className="viewport__rig" aria-label="Rig inspector">
          <div className="viewport__rig-summary">
            <strong>Rig / szkielet</strong>
            <span>{rigInventory.nodes.length} węzłów transformacji</span>
            <span>{rigInventory.counts.RIG_ROOT + rigInventory.counts.RIGID_PIVOT + rigInventory.counts.SKIN_BONE + rigInventory.counts.UNKNOWN} jointów riga</span>
            {rigInventory.counts.RIGID_PIVOT > 0 && <span>{rigInventory.counts.RIGID_PIVOT} sztywnych pivotów</span>}
            {rigInventory.counts.SKIN_BONE > 0 && <span>{rigInventory.counts.SKIN_BONE} skin bones</span>}
            <span>{rigInventory.counts.ATTACHMENT + rigInventory.counts.HELPER} helperów / attachmentów</span>
          </div>
          <div className="viewport__rig-legend" aria-label="Rig legend">
            <span data-category="RIG_ROOT">Rig root</span>
            <span data-category="RIGID_PIVOT">Rigid pivot</span>
            <span data-category="SKIN_BONE">Skin bone</span>
            <span data-category="ATTACHMENT">Attachment</span>
          </div>
          <div className="viewport__joint-detail" aria-live="polite">
            {selectedRigNode ? (
              <>
                <strong>{selectedRigNode.name}</strong>
                <span>{selectedRigNode.category}</span>
                <span>parent: {selectedRigNode.parentName ?? "—"}</span>
                <span>node #{selectedRigNode.nodeNumber} · depth {selectedRigNode.depth}</span>
              </>
            ) : <span>Kliknij joint, aby zobaczyć nazwę, typ i rodzica.</span>}
          </div>
        </section>
      )}
      {animationBinding && (
        <section className="viewport__binding" aria-label="Animation binding report">
          <span><strong>Rig:</strong> {animationBinding.rigSource}</span>
          {selectedBinding ? (
            <>
              <span><strong>Animacja:</strong> {selectedBinding.animationSource} / {selectedBinding.clipName}</span>
              <span data-status={selectedBinding.unmatchedNodeNames.length === 0 ? "pass" : "warning"}>
                <strong>Binding:</strong> {selectedBinding.matchedNodeCount}/{selectedBinding.controlledNodeCount}
              </span>
              {selectedBinding.unmatchedNodeNames.length > 0 && (
                <span title={selectedBinding.unmatchedNodeNames.join(", ")}>
                  Brak: {selectedBinding.unmatchedNodeNames.join(", ")}
                </span>
              )}
            </>
          ) : <span>Brak kontrolerów animacji do związania.</span>}
        </section>
      )}
      {(animationPlaybackEnabled || animationUnavailableReason) && (
        <section className="viewport__animation" aria-label="Animation player">
          {animationUnavailableReason ? (
            <p>{animationUnavailableReason}</p>
          ) : animationUi.loading ? (
            <p>Reading animation clips from the GLB…</p>
          ) : animationUi.inventory.length === 0 ? (
            <p>No animation clips are present in this GLB.</p>
          ) : (
            <>
              <label>
                Clip
                <select
                  aria-label="Animation clip"
                  value={animationUi.selectedClipIndex ?? 0}
                  onChange={(event) => selectClip(Number(event.target.value))}
                >
                  {animationUi.inventory.map((clip) => (
                    <option key={clip.index} value={clip.index}>
                      {clip.name} ({clip.durationSeconds.toFixed(2)} s)
                    </option>
                  ))}
                </select>
              </label>
              <button
                type="button"
                className="button button--secondary"
                onClick={() => {
                  const runtime = animationRuntimeRef.current;
                  if (runtime) updateAnimation(runtime.setPlaying(!animationUi.playing));
                }}
              >
                {animationUi.playing ? "Pause" : "Play"}
              </button>
              <button
                type="button"
                className="button button--secondary"
                onClick={() => {
                  const runtime = animationRuntimeRef.current;
                  if (runtime) updateAnimation(runtime.stop());
                }}
              >
                Stop
              </button>
              <div className="viewport__pose" role="group" aria-label="Rig pose">
                <button
                  type="button"
                  aria-pressed={animationUi.poseMode === "REST"}
                  onClick={() => {
                    const runtime = animationRuntimeRef.current;
                    if (runtime) updateAnimation(runtime.setPoseMode("REST"));
                  }}
                >Bind / rest</button>
                <button
                  type="button"
                  aria-pressed={animationUi.poseMode === "ANIMATED"}
                  onClick={() => {
                    const runtime = animationRuntimeRef.current;
                    if (runtime) updateAnimation(runtime.setPoseMode("ANIMATED"));
                  }}
                >Animated</button>
              </div>
              <button
                type="button"
                className="button button--secondary"
                aria-label="Previous animation keyframe"
                onClick={() => {
                  const runtime = animationRuntimeRef.current;
                  if (runtime) updateAnimation(runtime.stepKeyframe(-1));
                }}
              >
                Previous keyframe
              </button>
              <button
                type="button"
                className="button button--secondary"
                aria-label="Next animation keyframe"
                onClick={() => {
                  const runtime = animationRuntimeRef.current;
                  if (runtime) updateAnimation(runtime.stepKeyframe(1));
                }}
              >
                Next keyframe
              </button>
              <label className="viewport__loop">
                <input
                  type="checkbox"
                  checked={animationUi.loop}
                  onChange={(event) => {
                    const runtime = animationRuntimeRef.current;
                    if (runtime) updateAnimation(runtime.setLoop(event.target.checked));
                  }}
                />
                Loop
              </label>
              <label>
                Speed
                <select
                  aria-label="Animation speed"
                  value={animationUi.playbackRate}
                  onChange={(event) => {
                    const runtime = animationRuntimeRef.current;
                    if (runtime) updateAnimation(runtime.setPlaybackRate(Number(event.target.value)));
                  }}
                >
                  {[0.25, 0.5, 1, 1.5, 2].map((rate) => <option key={rate} value={rate}>{rate}×</option>)}
                </select>
              </label>
              <label className="viewport__timeline">
                Timeline
                <input
                  aria-label="Animation timeline"
                  type="range"
                  min={0}
                  max={Math.max(animationUi.durationSeconds, 0)}
                  step={0.01}
                  value={Math.min(animationUi.timeSeconds, animationUi.durationSeconds)}
                  onChange={(event) => {
                    const runtime = animationRuntimeRef.current;
                    if (runtime) updateAnimation(runtime.seek(Number(event.target.value)));
                  }}
                />
              </label>
              <output>{animationUi.timeSeconds.toFixed(2)} / {animationUi.durationSeconds.toFixed(2)} s</output>
            </>
          )}
        </section>
      )}
    </section>
  );
}
