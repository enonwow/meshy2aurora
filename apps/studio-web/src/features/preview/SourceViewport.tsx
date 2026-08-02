import { useCallback } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { SceneViewport } from "./SceneViewport";
import type { ModelPartRef, SourcePreviewInput } from "./types";
import type { CreatureSourceForwardV1 } from "../source/InputsPanel";

interface Props {
  input: SourcePreviewInput;
  sourceForward?: CreatureSourceForwardV1;
  componentOverlays?: readonly {
    id: string;
    boundsMin: readonly [number, number, number];
    boundsMax: readonly [number, number, number];
    color: string;
    selected: boolean;
  }[];
  onError?: (message: string) => void;
}

const SOURCE_FORWARD_DIRECTIONS: Record<CreatureSourceForwardV1, THREE.Vector3> = {
  POSITIVE_Z: new THREE.Vector3(0, 0, 1),
  NEGATIVE_Z: new THREE.Vector3(0, 0, -1),
  POSITIVE_X: new THREE.Vector3(1, 0, 0),
  NEGATIVE_X: new THREE.Vector3(-1, 0, 0),
};

export function SourceViewport({ input, sourceForward, componentOverlays = [], onError }: Props) {
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
        if (sourceForward) {
          const bounds = new THREE.Box3().setFromObject(gltf.scene);
          const size = bounds.getSize(new THREE.Vector3());
          const center = bounds.getCenter(new THREE.Vector3());
          const length = Math.max(size.x, size.y, size.z, 1) * 0.45;
          const arrow = new THREE.ArrowHelper(
            SOURCE_FORWARD_DIRECTIONS[sourceForward],
            center,
            length,
            0x43f5c5,
            length * 0.2,
            length * 0.12,
          );
          arrow.name = `Source Front ${sourceForward}`;
          arrow.userData.previewOverlay = "CREATURE_SOURCE_FORWARD";
          gltf.scene.add(arrow);
        }
        for (const overlay of componentOverlays) {
          const bounds = new THREE.Box3(
            new THREE.Vector3(...overlay.boundsMin),
            new THREE.Vector3(...overlay.boundsMax),
          );
          const helper = new THREE.Box3Helper(bounds, new THREE.Color(overlay.color));
          helper.name = `Material component ${overlay.id}`;
          helper.userData.previewOverlay = "MATERIAL_ID_COMPONENT_BOUNDS_V1";
          helper.userData.selected = overlay.selected;
          helper.renderOrder = overlay.selected ? 20 : 10;
          gltf.scene.add(helper);
        }
        resolve({ root: gltf.scene, animations: gltf.animations });
      }, reject);
    });
  }, [componentOverlays, input.file, sourceForward]);

  return (
    <SceneViewport
      provenance="SOURCE"
      detail="Original local GLB — viewport only, never proof of Aurora output"
      dependency={`${input.file.name}:${input.sourceSha256}:${sourceForward ?? "NO_FRONT"}:${JSON.stringify(componentOverlays)}`}
      buildRoot={buildRoot}
      tools={{ animationPlayback: true, overlays: true }}
      onError={onError}
    />
  );
}
