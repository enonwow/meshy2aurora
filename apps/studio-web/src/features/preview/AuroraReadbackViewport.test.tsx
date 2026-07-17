// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import * as THREE from "three";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AnimationPlaybackRuntime } from "./animationPlayback";
import type { BinaryMdlInspectionReport } from "./types";

interface CapturedSceneViewportProps {
  provenance: string;
  buildRoot: () => Promise<THREE.Object3D | { root: THREE.Object3D; animations?: readonly THREE.AnimationClip[] }>;
  tools?: { animationPlayback?: boolean; overlays?: boolean };
  animationUnavailableReason?: string;
}

const { capturedProps } = vi.hoisted(() => ({
  capturedProps: [] as CapturedSceneViewportProps[],
}));

vi.mock("./SceneViewport", () => ({
  SceneViewport: (props: CapturedSceneViewportProps) => {
    capturedProps.push(props);
    return (
      <section aria-label={`${props.provenance} model viewport`}>
        {props.animationUnavailableReason ? <p>{props.animationUnavailableReason}</p> : null}
      </section>
    );
  },
}));

import { AuroraReadbackViewport } from "./AuroraReadbackViewport";

const roots: Root[] = [];

const readback: BinaryMdlInspectionReport = {
  schemaVersion: 1,
  format: "BINARY_MDL",
  nodeTree: {
    roots: [{
      offset: 12,
      number: 1,
      name: "body",
      controllers: [],
      mesh: {
        vertices: [{ x: 0, y: 0, z: 0 }], normals: [], uv0: [], rawIndices: [[0, 0, 0]], faces: [{ vertexIndices: [0, 0, 0] }],
      },
      skin: {
        nodeToBoneMap: [-1, 0],
        inlineMapping: [1, -1],
        inverseBoneRotationsRaw: [[1, 0, 0, 0], [1, 0, 0, 0]],
        inverseBoneTranslations: [{ x: 0, y: 0, z: 0 }, { x: 0, y: 0, z: 0 }],
        vertexWeights: [[1, 0, 0, 0]],
        boneReferences: [[0, 0, 0, 0]],
      },
      children: [{ offset: 16, number: 2, name: "arm", controllers: [], children: [] }],
    }],
  },
  animations: [{
    offset: 24,
    name: "cpause1",
    length: 1,
    transition: 0,
    animationRoot: "body",
    nodeTree: {
      roots: [{
        offset: 24,
        number: 1,
        name: "body",
        controllers: [],
        children: [{
          offset: 28,
          number: 2,
          name: "arm",
          controllers: [{ controllerName: "position", times: [0, 1], values: [[0, 0, 0], [2, 0, 0]] }],
          children: [],
        }],
      }],
    },
  }],
  diagnostics: [],
};

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  capturedProps.length = 0;
  document.body.replaceChildren();
});

describe("AuroraReadbackViewport", () => {
  it("plays decoded converted-MDL controller data through the same viewport player", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => root.render(
      <AuroraReadbackViewport report={readback} onSelectPart={vi.fn()} />,
    ));

    expect(capturedProps).toHaveLength(1);
    const props = capturedProps[0];
    expect(props.tools).toEqual({ animationPlayback: true, overlays: true });
    expect(props.animationUnavailableReason).toBeUndefined();

    const asset = await props.buildRoot();
    expect(asset).not.toBeInstanceOf(THREE.Object3D);
    const animated = asset as { root: THREE.Object3D; animations: readonly THREE.AnimationClip[] };
    expect(animated.animations).toHaveLength(1);
    expect(animated.animations[0]?.name).toBe("cpause1");

    const runtime = new AnimationPlaybackRuntime(animated.root, animated.animations);
    runtime.selectClip(0);
    runtime.setPlaying(true);
    runtime.update(0.5);
    expect(animated.root.getObjectByName("arm")?.position.x).toBeCloseTo(1);
    animated.root.updateMatrixWorld(true);
    const skinned = animated.root.getObjectByProperty("isSkinnedMesh", true) as THREE.SkinnedMesh;
    expect(skinned).toBeInstanceOf(THREE.SkinnedMesh);
    expect(skinned.geometry.getAttribute("skinIndex")).toBeDefined();
    expect(skinned.geometry.getAttribute("skinWeight")).toBeDefined();
    expect(skinned.applyBoneTransform(0, new THREE.Vector3())).toMatchObject({ x: 1, y: 0, z: 0 });
    runtime.dispose();
  });
});
