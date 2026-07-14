import { useCallback } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { SceneViewport } from "./SceneViewport";
import type { ModelPartRef, SourcePreviewInput } from "./types";

export function SourceViewport({ input }: { input: SourcePreviewInput }) {
  const buildRoot = useCallback(async () => {
    const manager = new THREE.LoadingManager();
    manager.setURLModifier((url) => {
      if (url.startsWith("blob:") || url.startsWith("data:")) return url;
      throw new Error("Source preview forbids external GLB resource URLs");
    });
    const bytes = await input.file.arrayBuffer();
    return new Promise<THREE.Object3D>((resolve, reject) => {
      new GLTFLoader(manager).parse(bytes, "", (gltf) => {
        gltf.scene.traverse((object) => {
          object.userData.modelPart = {
            kind: "SOURCE_NODE",
            id: object.uuid,
            label: object.name || object.type,
          } satisfies ModelPartRef;
        });
        resolve(gltf.scene);
      }, reject);
    });
  }, [input.file]);

  return (
    <SceneViewport
      provenance="SOURCE"
      detail="Original local GLB — viewport only, never proof of Aurora output"
      dependency={`${input.file.name}:${input.sourceSha256}`}
      buildRoot={buildRoot}
    />
  );
}
