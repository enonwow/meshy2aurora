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
  SceneOverlayRuntime,
  type SceneOverlayKind,
} from "./sceneOverlays";
import type { ModelPartRef } from "./types";

export interface SceneViewportAsset {
  root: THREE.Object3D;
  animations?: readonly THREE.AnimationClip[];
}

interface SceneViewportProps {
  provenance: "SOURCE" | "AURORA IR" | "READBACK";
  detail: string;
  buildRoot: () => Promise<THREE.Object3D | SceneViewportAsset>;
  dependency: unknown;
  onSelectPart?: (part?: ModelPartRef) => void;
  onError?: (message: string) => void;
  tools?: {
    animationPlayback?: boolean;
    overlays?: boolean;
  };
  animationUnavailableReason?: string;
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
};

const overlayLabels: Record<SceneOverlayKind, string> = {
  grid: "Grid",
  axes: "Axes",
  skeleton: "Skeleton",
  bounds: "Bounds",
  wireframe: "Wireframe",
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
  const bounds = new THREE.Box3().setFromObject(root);
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
  onError,
  tools,
  animationUnavailableReason,
}: SceneViewportProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRuntimeRef = useRef<AnimationPlaybackRuntime | undefined>(undefined);
  const overlayRuntimeRef = useRef<SceneOverlayRuntime | undefined>(undefined);
  const [animationUi, setAnimationUi] = useState<AnimationUiState>(emptyAnimationUi);
  const [availableOverlays, setAvailableOverlays] = useState<SceneOverlayKind[]>([]);
  const [enabledOverlays, setEnabledOverlays] = useState<SceneOverlayKind[]>([]);
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
    const select = (event: PointerEvent) => {
      const rect = canvas.getBoundingClientRect();
      pointer.set(
        ((event.clientX - rect.left) / rect.width) * 2 - 1,
        -((event.clientY - rect.top) / rect.height) * 2 + 1,
      );
      raycaster.setFromCamera(pointer, camera);
      onSelectPart?.(modelPart(raycaster.intersectObjects(scene.children, true)[0]?.object));
    };
    canvas.addEventListener("pointerdown", select);

    if (animationPlaybackEnabled) setAnimationUi({ ...emptyAnimationUi, loading: true });
    if (overlaysEnabled) {
      setAvailableOverlays([]);
      setEnabledOverlays([]);
    }

    void buildRoot()
      .then((value) => {
        const asset = value instanceof THREE.Object3D ? { root: value, animations: [] } : value;
        if (stopped) return disposeObjectResources(asset.root);
        root = asset.root;
        scene.add(asset.root);
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
          const defaults = available.includes("grid") ? ["grid" as const] : [];
          defaults.forEach((kind) => overlayRuntime?.set(kind, true));
          setEnabledOverlays(defaults);
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
      renderer.render(scene, camera);
      frame = requestAnimationFrame(render);
    };
    render();

    return () => {
      stopped = true;
      cancelAnimationFrame(frame);
      canvas.removeEventListener("pointerdown", select);
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
  }, [animationPlaybackEnabled, buildRoot, dependency, onSelectPart, onError, overlaysEnabled]);

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
