import * as THREE from "three";
import type { BinaryMdlInspectionReport, ReadbackNode } from "./types";
import {
  canonicalCreatureWeaponGripOptionsV1,
  defaultCreatureWeaponGripOptionsV1,
  type CreatureWeaponEulerOffsetV1,
  type CreatureWeaponGripOptionsV1,
} from "../source/weaponGrip";

export type WeaponGripPreviewHandV1 = "OFF" | "RIGHT" | "LEFT";

export interface WeaponGripPreviewOptionsV1 {
  readonly hand: WeaponGripPreviewHandV1;
  readonly showHookAxes: boolean;
  readonly anchorAuthoring?: WeaponAnchorAuthoringEvidenceV1;
  readonly appliedGrip?: CreatureWeaponGripOptionsV1;
  readonly draftGrip?: CreatureWeaponGripOptionsV1;
}

export interface WeaponAnchorAuthoringEvidenceV1 {
  readonly calibration: string;
  readonly anchors: readonly {
    readonly anchorName: string;
    readonly anchorNodeId: number;
    readonly parentBoneName: string;
    readonly localMatrix: readonly number[];
    readonly weightedVertexCount: number;
  }[];
}

export interface WeaponGripPreviewInspectionV1 {
  readonly schemaVersion: 1;
  readonly hand: Exclude<WeaponGripPreviewHandV1, "OFF">;
  readonly anchorName: "rhand" | "lhand";
  readonly anchorNodeId: number;
  readonly parentBoneName: string;
  readonly localPosition: [number, number, number];
  readonly localOrientation: [number, number, number, number];
  readonly calibration: "BINARY_MDL_READBACK_AUTO_V1";
  readonly renderFidelity: "NWN_SHORTSWORD_BASIS_PROXY";
}

interface NodeMatch {
  readonly node: ReadbackNode;
  readonly parent?: ReadbackNode;
}

const anchorForHand = (hand: Exclude<WeaponGripPreviewHandV1, "OFF">) => (
  hand === "RIGHT" ? "rhand" as const : "lhand" as const
);

function collectNodeMatches(
  roots: readonly ReadbackNode[],
  expectedName: string,
  parent?: ReadbackNode,
  matches: NodeMatch[] = [],
) {
  roots.forEach((node) => {
    if (node.name.toLocaleLowerCase() === expectedName) matches.push({ node, parent });
    collectNodeMatches(node.children, expectedName, node, matches);
  });
  return matches;
}

function controllerValue(node: ReadbackNode, name: string, width: number, fallback: number[]) {
  const values = node.controllers.find(({ controllerName }) => controllerName === name)?.values[0];
  const resolved = values?.length === width ? values : fallback;
  if (resolved.length !== width || resolved.some((value) => !Number.isFinite(value))) {
    throw new Error(`Weapon Grip Preview requires a finite ${width}-value ${name} controller on ${node.name}`);
  }
  return resolved;
}

export function availableWeaponGripHandsV1(report: BinaryMdlInspectionReport) {
  return (["RIGHT", "LEFT"] as const).filter((hand) => (
    collectNodeMatches(report.nodeTree.roots, anchorForHand(hand)).length > 0
  ));
}

export function inspectWeaponGripPreviewV1(
  report: BinaryMdlInspectionReport,
  hand: Exclude<WeaponGripPreviewHandV1, "OFF">,
): WeaponGripPreviewInspectionV1 {
  const anchorName = anchorForHand(hand);
  const matches = collectNodeMatches(report.nodeTree.roots, anchorName);
  if (matches.length !== 1) {
    throw new Error(`Weapon Grip Preview requires exactly one ${anchorName} hook in binary MDL readback; found ${matches.length}`);
  }
  const [{ node, parent }] = matches;
  if (!parent) throw new Error(`Weapon Grip Preview hook ${anchorName} must have an explicit parent bone`);
  const localPosition = controllerValue(node, "position", 3, [0, 0, 0]);
  const localOrientation = controllerValue(node, "orientation", 4, [0, 0, 0, 1]);
  return {
    schemaVersion: 1,
    hand,
    anchorName,
    anchorNodeId: node.number,
    parentBoneName: parent.name,
    localPosition: [localPosition[0]!, localPosition[1]!, localPosition[2]!],
    localOrientation: [
      localOrientation[0]!,
      localOrientation[1]!,
      localOrientation[2]!,
      localOrientation[3]!,
    ],
    calibration: "BINARY_MDL_READBACK_AUTO_V1",
    renderFidelity: "NWN_SHORTSWORD_BASIS_PROXY",
  };
}

function previewMaterial(color: number, metalness: number, roughness: number) {
  return new THREE.MeshStandardMaterial({
    color,
    emissive: new THREE.Color(color).multiplyScalar(0.08),
    metalness,
    roughness,
  });
}

function markPreviewMesh(mesh: THREE.Mesh) {
  mesh.userData.previewOverlay = "CREATURE_WEAPON_GRIP_PROXY_V1";
  mesh.castShadow = true;
  return mesh;
}

function buildHookAxesProxyV1() {
  const axes = new THREE.Group();
  axes.name = "m2a-weapon-hook-axes";
  axes.userData.previewOverlay = "CREATURE_WEAPON_GRIP_PROXY_V1";
  const length = 0.24;
  const thickness = 0.008;
  const definitions = [
    { geometry: new THREE.BoxGeometry(length, thickness, thickness), color: 0xef4444, position: [length / 2, 0, 0] },
    { geometry: new THREE.BoxGeometry(thickness, length, thickness), color: 0x22c55e, position: [0, length / 2, 0] },
    { geometry: new THREE.BoxGeometry(thickness, thickness, length), color: 0x3b82f6, position: [0, 0, length / 2] },
  ] as const;
  definitions.forEach(({ geometry, color, position }) => {
    const mesh = markPreviewMesh(new THREE.Mesh(geometry, new THREE.MeshBasicMaterial({ color })));
    mesh.position.set(position[0], position[1], position[2]);
    axes.add(mesh);
  });
  return axes;
}

export function buildCalibrationWeaponProxyV1(
  hand: Exclude<WeaponGripPreviewHandV1, "OFF">,
  showHookAxes: boolean,
) {
  const proxy = new THREE.Group();
  proxy.name = `m2a-calibration-weapon-${hand.toLocaleLowerCase()}`;
  proxy.userData.previewOverlay = "CREATURE_WEAPON_GRIP_PROXY_V1";
  proxy.userData.renderFidelity = "NWN_SHORTSWORD_BASIS_PROXY";
  proxy.userData.hand = hand;

  const gripMaterial = previewMaterial(0x3b2a1d, 0.05, 0.8);
  const metalMaterial = previewMaterial(0xc7d2da, 0.78, 0.24);
  const guardMaterial = previewMaterial(0xb78a35, 0.62, 0.3);

  const pommel = markPreviewMesh(new THREE.Mesh(new THREE.SphereGeometry(0.036, 12, 8), guardMaterial.clone()));
  pommel.name = "m2a-calibration-weapon-pommel";
  pommel.position.y = -0.18;
  const grip = markPreviewMesh(new THREE.Mesh(new THREE.CylinderGeometry(0.025, 0.03, 0.2, 12), gripMaterial));
  grip.name = "m2a-calibration-weapon-grip";
  grip.position.y = -0.075;
  // NWN modular WSwLs parts use local +Y as the longitudinal weapon axis.
  // In the audited retail bounds the guard/blade are broad in local Z and
  // thin in local X. Keeping those axes here is essential: swapping X/Z makes
  // the proxy hide exactly the roll error that is visible on the real item.
  const guard = markPreviewMesh(new THREE.Mesh(new THREE.BoxGeometry(0.035, 0.025, 0.28), guardMaterial));
  guard.name = "m2a-calibration-weapon-guard";
  guard.position.y = 0.035;
  const blade = markPreviewMesh(new THREE.Mesh(new THREE.BoxGeometry(0.014, 0.72, 0.064), metalMaterial));
  blade.name = "m2a-calibration-weapon-blade";
  blade.position.y = 0.41;
  const tip = markPreviewMesh(new THREE.Mesh(new THREE.ConeGeometry(0.045, 0.14, 4), metalMaterial.clone()));
  tip.name = "m2a-calibration-weapon-tip";
  tip.position.y = 0.84;
  tip.rotation.y = Math.PI / 4;

  proxy.add(pommel, grip, guard, blade, tip);
  if (showHookAxes) proxy.add(buildHookAxesProxyV1());
  return proxy;
}

function weaponEulerRotationMatrixV1(offset: CreatureWeaponEulerOffsetV1) {
  const rollY = new THREE.Matrix4().makeRotationY(THREE.MathUtils.degToRad(offset.rollDegrees));
  const pitchX = new THREE.Matrix4().makeRotationX(THREE.MathUtils.degToRad(offset.pitchDegrees));
  const yawZ = new THREE.Matrix4().makeRotationZ(THREE.MathUtils.degToRad(offset.yawDegrees));
  return yawZ.multiply(pitchX).multiply(rollY);
}

export function weaponGripPreviewDeltaMatrixV1(
  appliedGrip: CreatureWeaponGripOptionsV1,
  draftGrip: CreatureWeaponGripOptionsV1,
  hand: Exclude<WeaponGripPreviewHandV1, "OFF">,
) {
  const applied = canonicalCreatureWeaponGripOptionsV1(appliedGrip);
  const draft = canonicalCreatureWeaponGripOptionsV1(draftGrip);
  const key = hand === "RIGHT" ? "rightHand" : "leftHand";
  const appliedRotation = weaponEulerRotationMatrixV1(applied[key]);
  const draftRotation = weaponEulerRotationMatrixV1(draft[key]);
  return appliedRotation.clone().invert().multiply(draftRotation);
}

export function assertWeaponGripAuthoringParityV1(
  inspection: WeaponGripPreviewInspectionV1,
  authoring: WeaponAnchorAuthoringEvidenceV1,
) {
  const matches = authoring.anchors.filter(({ anchorName }) => (
    anchorName.toLocaleLowerCase() === inspection.anchorName
  ));
  if (matches.length !== 1) {
    throw new Error(`Weapon Grip Preview authoring report requires exactly one ${inspection.anchorName} binding; found ${matches.length}`);
  }
  const [binding] = matches;
  if (
    binding.parentBoneName.toLocaleLowerCase() !== inspection.parentBoneName.toLocaleLowerCase()
    || binding.weightedVertexCount !== 0
    || binding.localMatrix.length !== 16
  ) {
    throw new Error(`Weapon Grip Preview authoring report disagrees with binary MDL readback for ${inspection.anchorName}`);
  }
  const readbackMatrix = new THREE.Matrix4().compose(
    new THREE.Vector3(...inspection.localPosition),
    new THREE.Quaternion(...inspection.localOrientation),
    new THREE.Vector3(1, 1, 1),
  );
  const maximumDelta = readbackMatrix.elements.reduce((maximum, value, index) => (
    Math.max(maximum, Math.abs(value - (binding.localMatrix[index] ?? Number.NaN)))
  ), 0);
  if (!Number.isFinite(maximumDelta) || maximumDelta > 1e-5) {
    throw new Error(`Weapon Grip Preview authoring matrix disagrees with binary MDL readback for ${inspection.anchorName}`);
  }
}

export function attachWeaponGripPreviewV1(
  root: THREE.Object3D,
  report: BinaryMdlInspectionReport,
  options: WeaponGripPreviewOptionsV1,
) {
  if (options.hand === "OFF") return undefined;
  const inspection = inspectWeaponGripPreviewV1(report, options.hand);
  if (options.anchorAuthoring) assertWeaponGripAuthoringParityV1(inspection, options.anchorAuthoring);
  const matches: THREE.Object3D[] = [];
  root.traverse((object) => {
    if (object.name.toLocaleLowerCase() === inspection.anchorName) matches.push(object);
  });
  if (matches.length !== 1) {
    throw new Error(`Weapon Grip Preview requires exactly one ${inspection.anchorName} hook in reconstructed readback; found ${matches.length}`);
  }
  const proxy = buildCalibrationWeaponProxyV1(options.hand, options.showHookAxes);
  const appliedGrip = options.appliedGrip ?? defaultCreatureWeaponGripOptionsV1();
  const draftGrip = options.draftGrip ?? appliedGrip;
  proxy.quaternion.setFromRotationMatrix(
    weaponGripPreviewDeltaMatrixV1(appliedGrip, draftGrip, options.hand),
  );
  proxy.userData.appliedWeaponGrip = appliedGrip;
  proxy.userData.draftWeaponGrip = draftGrip;
  matches[0]!.add(proxy);
  return { inspection, proxy };
}
