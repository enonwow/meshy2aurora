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

import {
  AuroraReadbackViewport,
  buildAuroraReadbackAsset,
  inspectWeaponGripPreviewV1,
} from "./AuroraReadbackViewport";
import { weaponGripPreviewDeltaMatrixV1 } from "./weaponGripPreview";
import { RIG_NODE_USER_DATA_V1, type RigNodeDescriptorV1 } from "./rigOverlay";
import { defaultCreatureWeaponGripOptionsV1 } from "../source/weaponGrip";
import type { CanonicalHeldStockWeaponReadback } from "../results/projectCanonicalResult";

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
        nodeToBoneMap: [-1, 0, -1, -1, -1],
        inlineMapping: [1, -1, -1, -1, -1],
        inverseBoneRotationsRaw: Array.from({ length: 5 }, () => [1, 0, 0, 0]),
        inverseBoneTranslations: Array.from({ length: 5 }, () => ({ x: 0, y: 0, z: 0 })),
        vertexWeights: [[1, 0, 0, 0]],
        boneReferences: [[0, 0, 0, 0]],
      },
      children: [
        {
          offset: 16,
          number: 2,
          name: "arm",
          controllers: [],
          children: [{
            offset: 18,
            number: 3,
            name: "rhand",
            controllers: [
              { controllerName: "position", times: [0], values: [[0.02, 0.03, 0.08]] },
              { controllerName: "orientation", times: [0], values: [[0, 0, 0.7071068, 0.7071068]] },
            ],
            children: [],
          }],
        },
        {
          offset: 20,
          number: 4,
          name: "leftArm",
          controllers: [],
          children: [{
            offset: 22,
            number: 5,
            name: "lhand",
            controllers: [
              { controllerName: "position", times: [0], values: [[-0.02, 0.03, 0.08]] },
              { controllerName: "orientation", times: [0], values: [[0, 0, -0.7071068, 0.7071068]] },
            ],
            children: [],
          }],
        },
      ],
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
  const heldStockWeaponReadback: CanonicalHeldStockWeaponReadback = {
    schemaVersion: 2 as const,
    weapon: {
      resref: "nw_wswss001",
      resourceType: 2025 as const,
      resourceScope: "NWN_BASE_GAME" as const,
    },
    fixtures: [{
      hand: "right_hand" as const,
      equippedItemResref: "nw_wswss001",
    }],
  };

  it("accepts native zero-filled unused extended64 inline slots", () => {
    const zeroTerminated = structuredClone(readback);
    const skin = zeroTerminated.nodeTree.roots[0]?.skin;
    if (!skin) throw new Error("test skin unavailable");
    skin.inlineMapping = [1, ...Array.from({ length: 63 }, () => 0)];

    const asset = buildAuroraReadbackAsset(zeroTerminated);
    const skinned = asset.root.getObjectByProperty("isSkinnedMesh", true) as THREE.SkinnedMesh;
    expect(skinned).toBeInstanceOf(THREE.SkinnedMesh);
    expect(skinned.skeleton.bones.map((bone) => bone.name)).toEqual(["arm"]);
  });

  it("tags the exact readback hierarchy with semantic joint categories", () => {
    const asset = buildAuroraReadbackAsset(readback);
    const descriptor = (name: string) => asset.root.getObjectByName(name)
      ?.userData[RIG_NODE_USER_DATA_V1] as RigNodeDescriptorV1 | undefined;

    expect(descriptor("body")).toMatchObject({ category: "MODEL_ROOT", nodeNumber: 1, ownsMesh: true });
    expect(descriptor("arm")).toMatchObject({ category: "SKIN_BONE", nodeNumber: 2, activeSkinBone: true });
    expect(descriptor("rhand")).toMatchObject({ category: "ATTACHMENT", parentName: "arm" });
  });

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

  it("attaches the calibration weapon to the exact readback hook and inherits hand motion", () => {
    const asset = buildAuroraReadbackAsset(readback, undefined, {
      hand: "RIGHT",
      showHookAxes: true,
    });
    const hook = asset.root.getObjectByName("rhand");
    const weapon = asset.root.getObjectByName("m2a-calibration-weapon-right");
    expect(hook).toBeInstanceOf(THREE.Bone);
    expect(weapon?.parent).toBe(hook);
    expect(weapon?.userData).toMatchObject({
      previewOverlay: "CREATURE_WEAPON_GRIP_PROXY_V1",
      renderFidelity: "NWN_SHORTSWORD_BASIS_PROXY",
    });
    expect(weapon?.getObjectByName("m2a-weapon-hook-axes")).toBeDefined();

    asset.root.updateMatrixWorld(true);
    const before = weapon?.getWorldPosition(new THREE.Vector3());
    hook?.position.add(new THREE.Vector3(0.5, 0.25, -0.1));
    asset.root.updateMatrixWorld(true);
    const after = weapon?.getWorldPosition(new THREE.Vector3());
    expect(after?.distanceTo(before ?? new THREE.Vector3())).toBeCloseTo(Math.sqrt(0.3225), 5);
  });

  it("uses the audited NWN item basis: blade and guard are broad in local Z, not local X", () => {
    const asset = buildAuroraReadbackAsset(readback, undefined, {
      hand: "RIGHT",
      showHookAxes: false,
    });
    const dimensions = (name: string) => {
      const mesh = asset.root.getObjectByName(name) as THREE.Mesh;
      expect(mesh).toBeInstanceOf(THREE.Mesh);
      mesh.geometry.computeBoundingBox();
      return mesh.geometry.boundingBox!.getSize(new THREE.Vector3());
    };
    const blade = dimensions("m2a-calibration-weapon-blade");
    const guard = dimensions("m2a-calibration-weapon-guard");
    expect(blade.z).toBeGreaterThan(blade.x * 4);
    expect(guard.z).toBeGreaterThan(guard.x * 4);
    expect(blade.y).toBeGreaterThan(blade.z * 10);
  });

  it("previews the exact local Rz(yaw) * Rx(pitch) * Ry(roll) delta without changing hook translation", () => {
    const automatic = defaultCreatureWeaponGripOptionsV1();
    const manual = {
      ...automatic,
      mode: "AUTO_PLUS_OFFSETS" as const,
      rightHand: { rollDegrees: 90, pitchDegrees: 0, yawDegrees: 0 },
    };
    const delta = weaponGripPreviewDeltaMatrixV1(automatic, manual, "RIGHT");
    expect(new THREE.Vector3(1, 0, 0).applyMatrix4(delta).toArray()).toEqual([
      expect.closeTo(0, 6),
      expect.closeTo(0, 6),
      expect.closeTo(-1, 6),
    ]);

    const asset = buildAuroraReadbackAsset(readback, undefined, {
      hand: "RIGHT",
      showHookAxes: false,
      appliedGrip: automatic,
      draftGrip: manual,
    });
    const hook = asset.root.getObjectByName("rhand");
    const weapon = asset.root.getObjectByName("m2a-calibration-weapon-right");
    expect(weapon?.position.toArray()).toEqual([0, 0, 0]);
    expect(weapon?.parent).toBe(hook);
    expect(new THREE.Vector3(1, 0, 0).applyQuaternion(weapon!.quaternion).toArray()).toEqual([
      expect.closeTo(0, 6),
      expect.closeTo(0, 6),
      expect.closeTo(-1, 6),
    ]);
  });

  it("keeps off/right/left explicit and reports the exact binary-readback transform", () => {
    const off = buildAuroraReadbackAsset(readback, undefined, { hand: "OFF", showHookAxes: false });
    expect(off.root.getObjectByName("m2a-calibration-weapon-right")).toBeUndefined();
    expect(off.root.getObjectByName("m2a-calibration-weapon-left")).toBeUndefined();

    const left = buildAuroraReadbackAsset(readback, undefined, { hand: "LEFT", showHookAxes: false });
    expect(left.root.getObjectByName("m2a-calibration-weapon-left")?.parent?.name).toBe("lhand");
    expect(left.root.getObjectByName("m2a-weapon-hook-axes")).toBeUndefined();

    expect(inspectWeaponGripPreviewV1(readback, "RIGHT")).toEqual({
      schemaVersion: 1,
      hand: "RIGHT",
      anchorName: "rhand",
      anchorNodeId: 3,
      parentBoneName: "arm",
      localPosition: [0.02, 0.03, 0.08],
      localOrientation: [0, 0, 0.7071068, 0.7071068],
      calibration: "BINARY_MDL_READBACK_AUTO_V1",
      renderFidelity: "NWN_SHORTSWORD_BASIS_PROXY",
    });
  });

  it("fails closed when the requested readback hook is missing or duplicated", () => {
    const missing = structuredClone(readback);
    missing.nodeTree.roots[0]!.children[0]!.children[0]!.name = "missing_rhand";
    expect(() => buildAuroraReadbackAsset(missing, undefined, { hand: "RIGHT", showHookAxes: false }))
      .toThrow(/exactly one rhand/i);

    const duplicate = structuredClone(readback);
    duplicate.nodeTree.roots[0]!.children.push({
      offset: 30,
      number: 6,
      name: "rhand",
      controllers: [],
      children: [],
    });
    expect(() => inspectWeaponGripPreviewV1(duplicate, "RIGHT")).toThrow(/exactly one rhand/i);
  });

  it("requires the Core authoring evidence to agree with the exact binary-MDL hook", () => {
    const inspection = inspectWeaponGripPreviewV1(readback, "RIGHT");
    const matrix = new THREE.Matrix4().compose(
      new THREE.Vector3(...inspection.localPosition),
      new THREE.Quaternion(...inspection.localOrientation),
      new THREE.Vector3(1, 1, 1),
    ).elements;
    const evidence = {
      calibration: "AURORA_CREATURE_HOOK_CALIBRATION_V1",
      anchors: [{
        anchorName: "rhand",
        // Core authoring ids belong to the IR node-id space. The binary writer
        // assigns a separate depth-first part number, so equality with
        // NodeReport.number is neither expected nor required for parity.
        anchorNodeId: 103,
        parentBoneName: "arm",
        localMatrix: [...matrix],
        weightedVertexCount: 0,
      }],
    };

    expect(() => buildAuroraReadbackAsset(readback, undefined, {
      hand: "RIGHT",
      showHookAxes: false,
      anchorAuthoring: evidence,
    })).not.toThrow();

    const mismatched = structuredClone(evidence);
    mismatched.anchors[0]!.localMatrix[12] += 0.5;
    expect(() => buildAuroraReadbackAsset(readback, undefined, {
      hand: "RIGHT",
      showHookAxes: false,
      anchorAuthoring: mismatched,
    })).toThrow(/matrix disagrees with binary MDL readback/i);
  });

  it("exposes visible grip controls, previews the draft, and applies it through rebuild", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    const onApplyWeaponGrip = vi.fn();
    await act(async () => root.render(
      <AuroraReadbackViewport
        report={readback}
        heldStockWeaponReadback={heldStockWeaponReadback}
        onSelectPart={vi.fn()}
        onApplyWeaponGrip={onApplyWeaponGrip}
      />,
    ));

    expect(container.textContent).toContain("Weapon Grip Preview");
    expect(container.textContent).toContain("NWN SHORTSWORD BASIS PROXY");
    expect(container.textContent).toContain("Not exact item geometry or Toolset proof");
    expect(container.textContent).toContain("Demo equipment readback: PASS");
    expect(container.textContent).toContain("nw_wswss001");
    expect(container.textContent).toContain("NWN_BASE_GAME");
    expect(container.textContent).toContain("rhand");
    const select = container.querySelector<HTMLSelectElement>('select[aria-label="Preview hand"]');
    expect(select?.value).toBe("RIGHT");

    await act(async () => {
      if (!select) throw new Error("preview hand select unavailable");
      select.value = "LEFT";
      select.dispatchEvent(new window.Event("change", { bubbles: true }));
    });
    expect(container.textContent).toContain("lhand");
    const latest = capturedProps.at(-1);
    const asset = await latest?.buildRoot();
    const built = asset instanceof THREE.Object3D ? asset : asset?.root;
    expect(built?.getObjectByName("m2a-calibration-weapon-left")?.parent?.name).toBe("lhand");

    const mode = container.querySelector<HTMLSelectElement>('select[aria-label="Weapon rotation mode"]');
    await act(async () => {
      if (!mode) throw new Error("weapon mode select unavailable");
      mode.value = "AUTO_PLUS_OFFSETS";
      mode.dispatchEvent(new Event("change", { bubbles: true }));
    });
    const apply = [...container.querySelectorAll("button")].find((button) => button.textContent === "Apply and rebuild");
    expect(apply?.hasAttribute("disabled")).toBe(false);
    await act(async () => apply?.click());
    expect(onApplyWeaponGrip).toHaveBeenCalledWith(expect.objectContaining({
      schemaVersion: 1,
      mode: "AUTO_PLUS_OFFSETS",
    }));
  });

  it("does not invent a held item when the current demo has no equipment readback", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => root.render(
      <AuroraReadbackViewport report={readback} onSelectPart={vi.fn()} />,
    ));

    expect(container.textContent).toContain("Current demo has no selected held item");
    const select = container.querySelector<HTMLSelectElement>('select[aria-label="Preview hand"]');
    expect(select?.value).toBe("OFF");
    const asset = await capturedProps.at(-1)?.buildRoot();
    const built = asset instanceof THREE.Object3D ? asset : asset?.root;
    expect(built?.getObjectByName("m2a-calibration-weapon-right")).toBeUndefined();
    expect(built?.getObjectByName("m2a-calibration-weapon-left")).toBeUndefined();
  });
});
