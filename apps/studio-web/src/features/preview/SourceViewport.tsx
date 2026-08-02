import { useCallback, useMemo } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import type { AuthoredAnimationClipV1 } from "../animation-studio";
import type { AnimationRigNodeV1 } from "../animation-editor/AnimationBoneTree";
import type { MotionVisualizationV1 } from "../animation-editor/animationWorkbench";
import { projectAuthoredClipToThreeV1 } from "../animation-editor/threeProjection";
import { projectThreeBoneTransformToAuthoredV1 } from "../animation-editor/threeProjection";
import { SceneViewport } from "./SceneViewport";
import type { AnimationPlaybackSnapshot } from "./animationPlayback";
import type { ModelPartRef, SourcePreviewInput } from "./types";
import type { SceneViewportTransformGizmoV1 } from "./SceneViewport";

export interface SourceViewportTransformGizmoV1 {
  readonly selectedBone: AnimationRigNodeV1 | null;
  readonly path: "ROTATION" | "TRANSLATION";
  readonly space: "local" | "world";
  readonly enabled?: boolean;
  readonly onBegin?: () => void;
  readonly onPreview?: (value: readonly number[]) => void;
  readonly onCommit?: (value: readonly number[]) => void;
}

export interface SourceViewportHeldWeaponV1 {
  readonly file: File;
  readonly targetBoneName: string;
  readonly translation: readonly [number, number, number];
  readonly rotationEulerDegrees: readonly [number, number, number];
  readonly scale: readonly [number, number, number];
}

interface Props {
  input: SourcePreviewInput;
  onError?: (message: string) => void;
  initialAnimationName?: string | null;
  initialAnimationLoop?: boolean;
  authoredClip?: AuthoredAnimationClipV1 | null;
  authoredRig?: readonly AnimationRigNodeV1[];
  controlledAnimationTimeSeconds?: number;
  controlledAnimationPlayback?: {
    playing: boolean;
    onUpdate: (snapshot: AnimationPlaybackSnapshot) => void;
  };
  hideAnimationControls?: boolean;
  transformGizmo?: SourceViewportTransformGizmoV1;
  heldWeapon?: SourceViewportHeldWeaponV1;
  motionVisualization?: MotionVisualizationV1 | null;
}

export function SourceViewport({
  input,
  onError,
  initialAnimationName,
  initialAnimationLoop,
  authoredClip,
  authoredRig = [],
  controlledAnimationTimeSeconds,
  controlledAnimationPlayback,
  hideAnimationControls,
  transformGizmo,
  heldWeapon,
  motionVisualization,
}: Props) {
  const buildRoot = useCallback(async () => {
    const manager = new THREE.LoadingManager();
    manager.setURLModifier((url) => {
      if (url.startsWith("blob:") || url.startsWith("data:")) return url;
      throw new Error("Source preview forbids external GLB resource URLs");
    });
    const bytes = await input.file.arrayBuffer();
    const parse = (payload: ArrayBuffer) => new Promise<Awaited<ReturnType<GLTFLoader["parseAsync"]>>>(
      (resolve, reject) => new GLTFLoader(manager).parse(payload, "", resolve, reject),
    );
    return new Promise<{ root: THREE.Object3D; animations: readonly THREE.AnimationClip[] }>((resolve, reject) => {
      new GLTFLoader(manager).parse(bytes, "", async (gltf) => {
        try {
        gltf.scene.traverse((object) => {
          object.userData.modelPart = {
            kind: "SOURCE_NODE",
            id: object.uuid,
            label: object.name || object.type,
          } satisfies ModelPartRef;
        });
        if (heldWeapon) {
          const target = gltf.scene.getObjectByName(heldWeapon.targetBoneName);
          if (!target) {
            throw new Error(`Held weapon target bone ${heldWeapon.targetBoneName} is missing.`);
          }
          const weapon = await parse(await heldWeapon.file.arrayBuffer());
          weapon.scene.name = `Held weapon: ${heldWeapon.file.name}`;
          weapon.scene.position.fromArray(heldWeapon.translation);
          weapon.scene.rotation.set(
            THREE.MathUtils.degToRad(heldWeapon.rotationEulerDegrees[0]),
            THREE.MathUtils.degToRad(heldWeapon.rotationEulerDegrees[1]),
            THREE.MathUtils.degToRad(heldWeapon.rotationEulerDegrees[2]),
          );
          weapon.scene.scale.fromArray(heldWeapon.scale);
          weapon.scene.traverse((object) => {
            object.userData.modelPart = {
              kind: "SOURCE_NODE",
              id: `held-weapon:${object.uuid}`,
              label: object.name || `Held weapon ${object.type}`,
            } satisfies ModelPartRef;
          });
          target.add(weapon.scene);
        }
        if (motionVisualization) {
          attachMotionVisualizationV1(gltf.scene, motionVisualization);
        }
        resolve({
          root: gltf.scene,
          animations: gltf.animations,
        });
        } catch (error) {
          reject(error);
        }
      }, reject);
    });
  }, [heldWeapon, input.file, motionVisualization]);
  const animationOverride = useMemo(() => authoredClip ? {
    dependency: authoredClip,
    project: (root: THREE.Object3D) => [
      projectAuthoredClipToThreeV1(authoredClip, authoredRig, root),
    ],
  } : undefined, [authoredClip, authoredRig]);
  const sceneTransformGizmo = useMemo<SceneViewportTransformGizmoV1 | undefined>(() => {
    const bone = transformGizmo?.selectedBone;
    if (!bone) return undefined;
    const project = (snapshot: Parameters<NonNullable<SceneViewportTransformGizmoV1["onCommit"]>>[0]) => (
      projectThreeBoneTransformToAuthoredV1(snapshot, bone, transformGizmo.path)
    );
    return {
      selectedObjectName: bone.name,
      mode: transformGizmo.path === "ROTATION" ? "rotate" : "translate",
      space: transformGizmo.space,
      enabled: transformGizmo.enabled,
      onBegin: transformGizmo.onBegin,
      onPreview: transformGizmo.onPreview
        ? (snapshot) => transformGizmo.onPreview?.(project(snapshot))
        : undefined,
      onCommit: transformGizmo.onCommit
        ? (snapshot) => transformGizmo.onCommit?.(project(snapshot))
        : undefined,
    };
  }, [transformGizmo]);

  return (
    <SceneViewport
      provenance="SOURCE"
      detail="Original local GLB — viewport only, never proof of Aurora output"
      dependency={`${input.file.name}:${input.sourceSha256}`}
      buildRoot={buildRoot}
      tools={{ animationPlayback: true, overlays: true }}
      initialAnimationName={authoredClip?.name ?? initialAnimationName}
      initialAnimationLoop={authoredClip ? false : initialAnimationLoop}
      animationOverride={animationOverride}
      controlledAnimationTimeSeconds={controlledAnimationTimeSeconds}
      controlledAnimationPlaying={controlledAnimationPlayback?.playing}
      onControlledAnimationPlaybackUpdate={
        controlledAnimationPlayback?.onUpdate
      }
      hideAnimationControls={hideAnimationControls ?? Boolean(authoredClip)}
      transformGizmo={sceneTransformGizmo}
      onError={onError}
    />
  );
}

function attachMotionVisualizationV1(
  root: THREE.Object3D,
  visualization: MotionVisualizationV1,
) {
  const helpers = new THREE.Group();
  helpers.name = "Animation motion visualization";
  helpers.userData.nonInteractiveOverlay = true;
  for (const [index, trail] of visualization.trails.entries()) {
    const geometry = new THREE.BufferGeometry().setFromPoints(
      trail.points.map(({ translation }) => new THREE.Vector3(...translation)),
    );
    const material = new THREE.LineBasicMaterial({
      color: new THREE.Color().setHSL((index * 0.19) % 1, 0.9, 0.58),
      transparent: true,
      opacity: 0.9,
      depthTest: false,
    });
    const line = new THREE.Line(geometry, material);
    line.name = `Motion trail node ${trail.nodeId}`;
    line.renderOrder = 999;
    line.raycast = () => undefined;
    helpers.add(line);
  }
  for (const onion of visualization.onionPoses) {
    const geometry = new THREE.BufferGeometry().setFromPoints(
      onion.pose.bones.map(({ translation }) => new THREE.Vector3(...translation)),
    );
    const material = new THREE.PointsMaterial({
      color: onion.offsetSamples < 0 ? 0x48a7ff : 0xff7a9f,
      size: 0.018,
      transparent: true,
      opacity: 0.34,
      depthWrite: false,
    });
    const ghost = new THREE.Points(geometry, material);
    ghost.name = onion.offsetSamples < 0 ? "Previous onion pose" : "Next onion pose";
    ghost.renderOrder = 998;
    ghost.raycast = () => undefined;
    helpers.add(ghost);
  }
  root.add(helpers);
}
