import { useCallback } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import type { AuthoredAnimationClipV1 } from "../animation-studio";
import type { AnimationRigNodeV1 } from "../animation-editor/AnimationBoneTree";
import { projectAuthoredClipToThreeV1 } from "../animation-editor/threeProjection";
import { SceneViewport } from "./SceneViewport";
import type { ModelPartRef, SourcePreviewInput } from "./types";

interface Props {
  input: SourcePreviewInput;
  onError?: (message: string) => void;
  initialAnimationName?: string | null;
  initialAnimationLoop?: boolean;
  authoredClip?: AuthoredAnimationClipV1 | null;
  authoredRig?: readonly AnimationRigNodeV1[];
  controlledAnimationTimeSeconds?: number;
}

export function SourceViewport({
  input,
  onError,
  initialAnimationName,
  initialAnimationLoop,
  authoredClip,
  authoredRig = [],
  controlledAnimationTimeSeconds,
}: Props) {
  const buildRoot = useCallback(async () => {
    const manager = new THREE.LoadingManager();
    manager.setURLModifier((url) => {
      if (url.startsWith("blob:") || url.startsWith("data:")) return url;
      throw new Error("Source preview forbids external GLB resource URLs");
    });
    const bytes = await input.file.arrayBuffer();
    return new Promise<{ root: THREE.Object3D; animations: readonly THREE.AnimationClip[] }>((resolve, reject) => {
      new GLTFLoader(manager).parse(bytes, "", (gltf) => {
        gltf.scene.traverse((object) => {
          object.userData.modelPart = {
            kind: "SOURCE_NODE",
            id: object.uuid,
            label: object.name || object.type,
          } satisfies ModelPartRef;
        });
        resolve({
          root: gltf.scene,
          animations: authoredClip
            ? [projectAuthoredClipToThreeV1(authoredClip, authoredRig)]
            : gltf.animations,
        });
      }, reject);
    });
  }, [authoredClip, authoredRig, input.file]);

  return (
    <SceneViewport
      provenance="SOURCE"
      detail="Original local GLB — viewport only, never proof of Aurora output"
      dependency={`${input.file.name}:${input.sourceSha256}`}
      buildRoot={buildRoot}
      tools={{ animationPlayback: true, overlays: true }}
      initialAnimationName={authoredClip?.name ?? initialAnimationName}
      initialAnimationLoop={initialAnimationLoop}
      controlledAnimationTimeSeconds={controlledAnimationTimeSeconds}
      hideAnimationControls={Boolean(authoredClip)}
      onError={onError}
    />
  );
}
