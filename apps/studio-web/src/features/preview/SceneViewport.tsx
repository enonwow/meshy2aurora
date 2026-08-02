import { useCallback, useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { TransformControls } from "three/examples/jsm/controls/TransformControls.js";
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

export interface SceneViewportAnimationOverride {
  dependency: unknown;
  project: (root: THREE.Object3D) => readonly THREE.AnimationClip[];
}

export interface SceneViewportTransformSnapshotV1 {
  readonly objectName: string;
  readonly position: readonly [number, number, number];
  readonly quaternion: readonly [number, number, number, number];
  readonly initialPosition: readonly [number, number, number];
  readonly initialQuaternion: readonly [number, number, number, number];
  readonly sourceRestPosition: readonly [number, number, number];
}

export interface SceneViewportTransformGizmoV1 {
  readonly selectedObjectName: string | null;
  readonly mode: "translate" | "rotate";
  readonly space: "local" | "world";
  readonly enabled?: boolean;
  readonly onBegin?: () => void;
  readonly onPreview?: (snapshot: SceneViewportTransformSnapshotV1) => void;
  readonly onCommit?: (snapshot: SceneViewportTransformSnapshotV1) => void;
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
  initialAnimationName?: string | null;
  initialAnimationLoop?: boolean;
  animationOverride?: SceneViewportAnimationOverride;
  controlledAnimationTimeSeconds?: number;
  controlledAnimationPlaying?: boolean;
  onControlledAnimationPlaybackUpdate?: (
    snapshot: AnimationPlaybackSnapshot,
  ) => void;
  hideAnimationControls?: boolean;
  transformGizmo?: SceneViewportTransformGizmoV1;
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
  initialAnimationName,
  initialAnimationLoop = true,
  animationOverride,
  controlledAnimationTimeSeconds,
  controlledAnimationPlaying,
  onControlledAnimationPlaybackUpdate,
  hideAnimationControls = false,
  transformGizmo,
}: SceneViewportProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rootRef = useRef<THREE.Object3D | undefined>(undefined);
  const sourceAnimationsRef = useRef<readonly THREE.AnimationClip[]>([]);
  const animationRuntimeRef = useRef<AnimationPlaybackRuntime | undefined>(undefined);
  const overlayRuntimeRef = useRef<SceneOverlayRuntime | undefined>(undefined);
  const transformControlsRef = useRef<TransformControls | undefined>(undefined);
  const [animationUi, setAnimationUi] = useState<AnimationUiState>(emptyAnimationUi);
  const [availableOverlays, setAvailableOverlays] = useState<SceneOverlayKind[]>([]);
  const [enabledOverlays, setEnabledOverlays] = useState<SceneOverlayKind[]>([]);
  const animationPlaybackEnabled = tools?.animationPlayback === true;
  const overlaysEnabled = tools?.overlays === true;
  const animationOverrideRef = useRef(animationOverride);
  const playbackConfigurationRef = useRef({
    controlledAnimationPlaying,
    controlledAnimationTimeSeconds,
    hideAnimationControls,
    initialAnimationLoop,
    initialAnimationName,
  });
  const controlledPlaybackUpdateRef = useRef(
    onControlledAnimationPlaybackUpdate,
  );
  const transformGizmoRef = useRef(transformGizmo);
  animationOverrideRef.current = animationOverride;
  playbackConfigurationRef.current = {
    controlledAnimationPlaying,
    controlledAnimationTimeSeconds,
    hideAnimationControls,
    initialAnimationLoop,
    initialAnimationName,
  };
  controlledPlaybackUpdateRef.current = onControlledAnimationPlaybackUpdate;
  transformGizmoRef.current = transformGizmo;

  const syncTransformGizmo = useCallback(() => {
    const controls = transformControlsRef.current;
    const root = rootRef.current;
    const configuration = transformGizmoRef.current;
    if (!controls || !root || !configuration?.selectedObjectName) {
      controls?.detach();
      return;
    }
    const object = root.getObjectByName(configuration.selectedObjectName);
    if (!object || configuration.enabled === false) {
      controls.detach();
      return;
    }
    controls.setMode(configuration.mode);
    controls.setSpace(configuration.space);
    controls.attach(object);
  }, []);

  const installAnimationClips = useCallback((
    root: THREE.Object3D,
    sourceAnimations: readonly THREE.AnimationClip[],
  ) => {
    const clips = animationOverrideRef.current?.project(root)
      ?? sourceAnimations;
    let runtime = animationRuntimeRef.current;
    if (runtime) {
      runtime.replaceClips(clips);
    } else {
      runtime = new AnimationPlaybackRuntime(root, clips);
      animationRuntimeRef.current = runtime;
    }
    const inventory = inventoryAnimationClips(clips);
    const configuration = playbackConfigurationRef.current;
    const requestedIndex = configuration.initialAnimationName
      ? inventory.findIndex(({ name }) => (
          name.localeCompare(
            configuration.initialAnimationName ?? "",
            undefined,
            { sensitivity: "base" },
          ) === 0
        ))
      : -1;
    let snapshot = inventory.length
      ? runtime.setLoop(configuration.initialAnimationLoop)
      : runtime.snapshot();
    if (inventory.length) {
      snapshot = runtime.selectClip(requestedIndex >= 0 ? requestedIndex : 0);
    }
    if (
      inventory.length
      && configuration.controlledAnimationTimeSeconds !== undefined
    ) {
      snapshot = runtime.seek(configuration.controlledAnimationTimeSeconds);
    }
    if (configuration.controlledAnimationPlaying !== undefined) {
      snapshot = runtime.setPlaying(configuration.controlledAnimationPlaying);
    }
    setAnimationUi({ inventory, loading: false, ...snapshot });
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    let stopped = false;
    let root: THREE.Object3D | undefined;
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
    const transformControls = new TransformControls(camera, canvas);
    const transformHelper = transformControls.getHelper();
    transformControlsRef.current = transformControls;
    scene.add(transformHelper);
    let transformStart: {
      position: [number, number, number];
      quaternion: [number, number, number, number];
    } | null = null;
    const transformSnapshot = (): SceneViewportTransformSnapshotV1 | null => {
      const object = transformControls.object;
      if (!object || !transformStart) return null;
      const rest = object.userData.m2aRestPosition as
        | readonly [number, number, number]
        | undefined;
      return {
        objectName: object.name,
        position: [object.position.x, object.position.y, object.position.z],
        quaternion: [
          object.quaternion.x,
          object.quaternion.y,
          object.quaternion.z,
          object.quaternion.w,
        ],
        initialPosition: transformStart.position,
        initialQuaternion: transformStart.quaternion,
        sourceRestPosition: rest
          ? [rest[0], rest[1], rest[2]]
          : transformStart.position,
      };
    };
    const beginTransform = () => {
      const object = transformControls.object;
      if (!object) return;
      controls.enabled = false;
      transformStart = {
        position: [object.position.x, object.position.y, object.position.z],
        quaternion: [
          object.quaternion.x,
          object.quaternion.y,
          object.quaternion.z,
          object.quaternion.w,
        ],
      };
      transformGizmoRef.current?.onBegin?.();
    };
    const previewTransform = () => {
      const snapshot = transformSnapshot();
      if (snapshot) transformGizmoRef.current?.onPreview?.(snapshot);
    };
    const commitTransform = () => {
      const snapshot = transformSnapshot();
      controls.enabled = true;
      transformStart = null;
      if (snapshot) transformGizmoRef.current?.onCommit?.(snapshot);
    };
    transformControls.addEventListener("mouseDown", beginTransform);
    transformControls.addEventListener("objectChange", previewTransform);
    transformControls.addEventListener("mouseUp", commitTransform);
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
      if (root) fitCamera(camera, controls, root);
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
        rootRef.current = root;
        root.traverse((object) => {
          object.userData.m2aRestPosition ??= [
            object.position.x,
            object.position.y,
            object.position.z,
          ];
        });
        sourceAnimationsRef.current = asset.animations ?? [];
        scene.add(asset.root);
        syncTransformGizmo();

        if (animationPlaybackEnabled) {
          installAnimationClips(asset.root, sourceAnimationsRef.current);
        }

        // Animation selection/seek mutates the skeleton pose. Frame the actual
        // pose shown to the author instead of the pre-mixer bind pose.
        fitCamera(camera, controls, asset.root);

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
      const animationRuntime = animationRuntimeRef.current;
      if (animationRuntime) {
        const snapshot = animationRuntime.update(deltaSeconds);
        if (frameTime - lastAnimationUiUpdate >= 80) {
          lastAnimationUiUpdate = frameTime;
          const configuration = playbackConfigurationRef.current;
          if (configuration.controlledAnimationPlaying === true) {
            controlledPlaybackUpdateRef.current?.(snapshot);
          }
          if (!configuration.hideAnimationControls) {
            setAnimationUi((current) => ({ ...current, ...snapshot }));
          }
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
      transformControls.removeEventListener("mouseDown", beginTransform);
      transformControls.removeEventListener("objectChange", previewTransform);
      transformControls.removeEventListener("mouseUp", commitTransform);
      transformControls.detach();
      transformControls.dispose();
      scene.remove(transformHelper);
      if (transformControlsRef.current === transformControls) {
        transformControlsRef.current = undefined;
      }
      if (rootRef.current === root) {
        animationRuntimeRef.current?.dispose();
        animationRuntimeRef.current = undefined;
        rootRef.current = undefined;
        sourceAnimationsRef.current = [];
      }
      overlayRuntime?.dispose();
      if (overlayRuntimeRef.current === overlayRuntime) overlayRuntimeRef.current = undefined;
      if (root) disposeObjectResources(root);
      if (staticGrid) {
        staticGrid.geometry.dispose();
        const material = Array.isArray(staticGrid.material) ? staticGrid.material : [staticGrid.material];
        material.forEach((entry) => entry.dispose());
      }
      renderer.dispose();
    };
  }, [
    animationPlaybackEnabled,
    buildRoot,
    dependency,
    installAnimationClips,
    onSelectPart,
    onError,
    overlaysEnabled,
    syncTransformGizmo,
  ]);

  useEffect(() => {
    syncTransformGizmo();
  }, [
    dependency,
    syncTransformGizmo,
    transformGizmo?.enabled,
    transformGizmo?.mode,
    transformGizmo?.selectedObjectName,
    transformGizmo?.space,
  ]);

  useEffect(() => {
    if (!animationPlaybackEnabled) return;
    const root = rootRef.current;
    if (!root) return;
    installAnimationClips(root, sourceAnimationsRef.current);
  }, [
    animationOverride?.dependency,
    animationPlaybackEnabled,
    installAnimationClips,
  ]);

  useEffect(() => {
    if (controlledAnimationTimeSeconds === undefined) return;
    if (controlledAnimationPlaying) return;
    const runtime = animationRuntimeRef.current;
    if (!runtime) return;
    const snapshot = runtime.seek(controlledAnimationTimeSeconds);
    if (!hideAnimationControls) {
      setAnimationUi((current) => ({ ...current, ...snapshot }));
    }
  }, [
    animationUi.inventory.length,
    controlledAnimationPlaying,
    controlledAnimationTimeSeconds,
    hideAnimationControls,
  ]);

  useEffect(() => {
    if (controlledAnimationPlaying === undefined) return;
    const runtime = animationRuntimeRef.current;
    if (!runtime) return;
    const snapshot = runtime.setPlaying(controlledAnimationPlaying);
    if (!hideAnimationControls) {
      setAnimationUi((current) => ({ ...current, ...snapshot }));
    }
    if (!controlledAnimationPlaying) {
      controlledPlaybackUpdateRef.current?.(snapshot);
    }
  }, [
    animationUi.inventory.length,
    controlledAnimationPlaying,
    hideAnimationControls,
  ]);

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
      {((animationPlaybackEnabled && !hideAnimationControls) || animationUnavailableReason) && (
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
