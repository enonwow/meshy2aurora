import { useCallback, useEffect, useState } from "react";
import * as THREE from "three";
import { animationClipsFromReadback } from "./readbackAnimationPlayback";
import {
  classifyReadbackRigV1,
  RIG_NODE_USER_DATA_V1,
  type RigNodeDescriptorV1,
} from "./rigOverlay";
import { SceneViewport } from "./SceneViewport";
import type {
  BinaryMdlInspectionReport,
  ModelPartRef,
  ReadbackController,
  ReadbackMesh,
  ReadbackNode,
  ReadbackSkin,
} from "./types";
import {
  attachWeaponGripPreviewV1,
  availableWeaponGripHandsV1,
  inspectWeaponGripPreviewV1,
  type WeaponAnchorAuthoringEvidenceV1,
  type WeaponGripPreviewHandV1,
  type WeaponGripPreviewOptionsV1,
} from "./weaponGripPreview";
import { WeaponGripControls } from "./WeaponGripControls";
import {
  defaultCreatureWeaponGripOptionsV1,
  sameCreatureWeaponGripOptionsV1,
  type CreatureWeaponGripOptionsV1,
} from "../source/weaponGrip";
import type { CanonicalHeldStockWeaponReadback } from "../results/projectCanonicalResult";

export { inspectWeaponGripPreviewV1 } from "./weaponGripPreview";

interface MeshPlan {
  owner: THREE.Bone;
  node: ReadbackNode;
  mesh: ReadbackMesh;
  skin?: ReadbackSkin;
  selected: boolean;
}

interface SkinSlotBones {
  bones: THREE.Bone[];
  treeOrdinals: number[];
}

function attribute(values: number[][], width: number) {
  return new THREE.BufferAttribute(new Float32Array(values.flat()), width);
}

function geometryFromReadback(mesh: ReadbackMesh) {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", attribute(mesh.vertices.map(({ x, y, z }) => [x, y, z]), 3));
  if (mesh.normals.length === mesh.vertices.length) {
    geometry.setAttribute("normal", attribute(mesh.normals.map(({ x, y, z }) => [x, y, z]), 3));
  } else {
    geometry.computeVertexNormals();
  }
  if (mesh.uv0.length === mesh.vertices.length) {
    geometry.setAttribute("uv", attribute(mesh.uv0.map(({ x, y }) => [x, y]), 2));
  }
  const rawIndices = mesh.rawIndices.flat();
  geometry.setIndex(rawIndices.length ? rawIndices : mesh.faces.flatMap((face) => face.vertexIndices));
  return geometry;
}

function firstController(controllers: ReadbackController[], name: string) {
  return controllers.find((controller) => controller.controllerName === name)?.values[0];
}

function material(selected: boolean) {
  return new THREE.MeshStandardMaterial({
    color: selected ? 0x62d6b2 : 0xd6a96c,
    emissive: selected ? 0x164f42 : 0x000000,
    roughness: 0.85,
    side: THREE.DoubleSide,
  });
}

function nodeFromReadback(
  node: ReadbackNode,
  selectedPart: ModelPartRef | undefined,
  nodeOrder: THREE.Bone[],
  meshPlans: MeshPlan[],
  rigNodes: readonly RigNodeDescriptorV1[],
): THREE.Bone {
  // A Bone is also an Object3D. Keeping the exact readback hierarchy as bones
  // lets Three apply decoded Aurora controller tracks to the same joints that
  // deform the readback mesh.
  const bone = new THREE.Bone();
  bone.name = node.name || `node-${node.number}`;
  const rigNode = rigNodes[nodeOrder.length];
  if (rigNode) bone.userData[RIG_NODE_USER_DATA_V1] = rigNode;
  bone.userData.modelPart = {
    kind: "READBACK_NODE",
    id: node.number,
    label: bone.name,
  } satisfies ModelPartRef;
  nodeOrder.push(bone);
  const position = firstController(node.controllers, "position");
  if (position?.length === 3) bone.position.fromArray(position);
  const orientation = firstController(node.controllers, "orientation");
  if (orientation?.length === 4) bone.quaternion.fromArray(orientation);
  const scale = firstController(node.controllers, "scale");
  if (scale?.length === 1) bone.scale.setScalar(scale[0]);

  if (node.mesh?.vertices.length) {
    meshPlans.push({
      owner: bone,
      node,
      mesh: node.mesh,
      skin: node.skin,
      selected: selectedPart?.kind === "READBACK_NODE" && String(selectedPart.id) === String(node.number),
    });
  }
  node.children.forEach((child) => bone.add(nodeFromReadback(child, selectedPart, nodeOrder, meshPlans, rigNodes)));
  return bone;
}

function slotBones(skin: ReadbackSkin, nodeOrder: readonly THREE.Bone[]): SkinSlotBones {
  if (skin.nodeToBoneMap.length !== nodeOrder.length) {
    throw new Error(`Canonical skin node-to-bone map has ${skin.nodeToBoneMap.length} entries, but the readback tree has ${nodeOrder.length} nodes`);
  }
  const activeSlots = skin.nodeToBoneMap.filter((slot) => slot >= 0);
  if (activeSlots.length === 0) throw new Error("Canonical skin declares no active bone slots");
  const uniqueSlots = new Set(activeSlots);
  const activeSlotCount = Math.max(...activeSlots) + 1;
  if (
    uniqueSlots.size !== activeSlots.length
    || activeSlots.length !== activeSlotCount
    || Array.from({ length: activeSlotCount }, (_, slot) => slot)
      .some((slot) => !uniqueSlots.has(slot))
  ) {
    throw new Error("Canonical skin node-to-bone map does not define unique contiguous active slots");
  }
  if (skin.inlineMapping.length < activeSlotCount) {
    throw new Error(`Canonical skin inline mapping has ${skin.inlineMapping.length} entries, but ${activeSlotCount} active slots are required`);
  }
  const result: THREE.Bone[] = [];
  const treeOrdinals: number[] = [];
  for (let slot = 0; slot < activeSlotCount; slot += 1) {
    const ordinal = skin.inlineMapping[slot] ?? -1;
    const bone = nodeOrder[ordinal];
    if (!bone || skin.nodeToBoneMap[ordinal] !== slot) {
      throw new Error(`Canonical skin inline mapping disagrees with node-to-bone map at slot ${slot}`);
    }
    result.push(bone);
    treeOrdinals.push(ordinal);
  }
  return { bones: result, treeOrdinals };
}

function applySkinAttributes(geometry: THREE.BufferGeometry, mesh: ReadbackMesh, skin: ReadbackSkin, bones: readonly THREE.Bone[]) {
  if (skin.vertexWeights.length !== mesh.vertices.length || skin.boneReferences.length !== mesh.vertices.length) {
    throw new Error("Canonical skin weights/references do not match the readback vertex count");
  }
  const skinIndices: number[] = [];
  const skinWeights: number[] = [];
  skin.vertexWeights.forEach((weights, vertexIndex) => {
    const references = skin.boneReferences[vertexIndex] ?? [];
    if (weights.length !== 4 || references.length !== 4) throw new Error(`Canonical skin lane count is invalid at vertex ${vertexIndex}`);
    let nonZeroWeight = 0;
    weights.forEach((weight, lane) => {
      const reference = references[lane] ?? 0;
      if (!Number.isFinite(weight) || weight < 0) throw new Error(`Canonical skin weight is invalid at vertex ${vertexIndex}, lane ${lane}`);
      if (weight > 0 && (!Number.isSafeInteger(reference) || reference < 0 || reference >= bones.length)) {
        throw new Error(`Canonical skin bone reference is invalid at vertex ${vertexIndex}, lane ${lane}`);
      }
      if (weight > 0) nonZeroWeight += weight;
      skinIndices.push(weight > 0 ? reference : 0);
      skinWeights.push(weight);
    });
    if (nonZeroWeight <= 0) throw new Error(`Canonical skin has no weighted bone at vertex ${vertexIndex}`);
  });
  geometry.setAttribute("skinIndex", new THREE.Uint16BufferAttribute(skinIndices, 4));
  geometry.setAttribute("skinWeight", new THREE.Float32BufferAttribute(skinWeights, 4));
}

function assertCanonicalInverseBindings(
  mesh: THREE.SkinnedMesh,
  skin: ReadbackSkin,
  bones: readonly THREE.Bone[],
  treeOrdinals: readonly number[],
) {
  if (skin.inverseBoneRotationsRaw.length !== skin.nodeToBoneMap.length || skin.inverseBoneTranslations.length !== skin.nodeToBoneMap.length) {
    throw new Error(`Canonical inverse-bind arrays (${skin.inverseBoneRotationsRaw.length}/${skin.inverseBoneTranslations.length}) do not match the tree map (${skin.nodeToBoneMap.length})`);
  }
  const actual = new THREE.Matrix4();
  const expected = new THREE.Matrix4();
  const expectedPosition = new THREE.Vector3();
  const expectedQuaternion = new THREE.Quaternion();
  const expectedScale = new THREE.Vector3();
  const actualPosition = new THREE.Vector3();
  const actualQuaternion = new THREE.Quaternion();
  const actualScale = new THREE.Vector3();
  bones.forEach((bone, slot) => {
    const ordinal = treeOrdinals[slot];
    const rotation = skin.inverseBoneRotationsRaw[ordinal] ?? [];
    const translation = skin.inverseBoneTranslations[ordinal];
    if (rotation.length !== 4 || !translation) throw new Error(`Canonical inverse-bind entry ${ordinal} is incomplete`);
    // Rust readback preserves Aurora's raw WXYZ order; Three uses XYZW.
    expected.compose(
      expectedPosition.set(translation.x, translation.y, translation.z),
      expectedQuaternion.set(rotation[1] ?? 0, rotation[2] ?? 0, rotation[3] ?? 0, rotation[0] ?? 1),
      expectedScale.set(1, 1, 1),
    );
    actual.copy(bone.matrixWorld).invert().multiply(mesh.matrixWorld);
    expected.decompose(expectedPosition, expectedQuaternion, expectedScale);
    actual.decompose(actualPosition, actualQuaternion, actualScale);
    const quaternionError = 1 - Math.abs(expectedQuaternion.dot(actualQuaternion));
    if (expectedPosition.distanceTo(actualPosition) > 1e-3 || quaternionError > 1e-5 || actualScale.distanceToSquared(expectedScale) > 1e-6) {
      throw new Error(`Canonical inverse bind disagrees with readback hierarchy at skin bone slot ${slot} (tree ordinal ${ordinal})`);
    }
  });
}

export type AuroraReadbackMaterialResolver = (
  mesh: ReadbackMesh,
  node: ReadbackNode,
  selected: boolean,
) => THREE.Material;

function attachMeshes(
  root: THREE.Group,
  meshPlans: readonly MeshPlan[],
  nodeOrder: readonly THREE.Bone[],
  resolveMaterial: AuroraReadbackMaterialResolver = (_mesh, _node, selected) => material(selected),
) {
  const skinned: Array<{ mesh: THREE.SkinnedMesh; skin: ReadbackSkin; bones: THREE.Bone[]; treeOrdinals: number[] }> = [];
  meshPlans.forEach((plan) => {
    const geometry = geometryFromReadback(plan.mesh);
    if (!plan.skin) {
      plan.owner.add(new THREE.Mesh(geometry, resolveMaterial(plan.mesh, plan.node, plan.selected)));
      return;
    }
    const slots = slotBones(plan.skin, nodeOrder);
    applySkinAttributes(geometry, plan.mesh, plan.skin, slots.bones);
    const mesh = new THREE.SkinnedMesh(geometry, resolveMaterial(plan.mesh, plan.node, plan.selected));
    plan.owner.add(mesh);
    skinned.push({ mesh, skin: plan.skin, bones: slots.bones, treeOrdinals: slots.treeOrdinals });
  });
  root.updateMatrixWorld(true);
  skinned.forEach(({ mesh, skin, bones, treeOrdinals }) => {
    assertCanonicalInverseBindings(mesh, skin, bones, treeOrdinals);
    const skeleton = new THREE.Skeleton(bones);
    skeleton.calculateInverses();
    mesh.bind(skeleton, mesh.matrixWorld);
  });
}

interface Props {
  report: BinaryMdlInspectionReport;
  weaponAnchorAuthoring?: WeaponAnchorAuthoringEvidenceV1;
  heldStockWeaponReadback?: CanonicalHeldStockWeaponReadback;
  selectedPart?: ModelPartRef;
  onSelectPart: (part?: ModelPartRef) => void;
  onError?: (message: string) => void;
  appliedWeaponGrip?: CreatureWeaponGripOptionsV1;
  onApplyWeaponGrip?: (weaponGrip: CreatureWeaponGripOptionsV1) => void;
}

const DEFAULT_WEAPON_GRIP_V1 = defaultCreatureWeaponGripOptionsV1();

export function buildAuroraReadbackAsset(
  report: BinaryMdlInspectionReport,
  selectedPart?: ModelPartRef,
  weaponGripPreview?: WeaponGripPreviewOptionsV1,
  materialResolver?: AuroraReadbackMaterialResolver,
) {
  const root = new THREE.Group();
  // Binary Aurora MDL is Z-up while Three renders its scene in Y-up. This is
  // preview-only; canonical binary readback values remain untouched.
  root.rotation.x = -Math.PI / 2;
  const nodeOrder: THREE.Bone[] = [];
  const meshPlans: MeshPlan[] = [];
  const rig = classifyReadbackRigV1(report.nodeTree.roots);
  report.nodeTree.roots.forEach((node) => root.add(nodeFromReadback(node, selectedPart, nodeOrder, meshPlans, rig.nodes)));
  attachMeshes(root, meshPlans, nodeOrder, materialResolver);
  if (weaponGripPreview) attachWeaponGripPreviewV1(root, report, weaponGripPreview);
  return { root, animations: animationClipsFromReadback(report) };
}

export function AuroraReadbackViewport({
  report,
  weaponAnchorAuthoring,
  heldStockWeaponReadback,
  selectedPart,
  onSelectPart,
  onError,
  appliedWeaponGrip = DEFAULT_WEAPON_GRIP_V1,
  onApplyWeaponGrip,
}: Props) {
  const availableHands = availableWeaponGripHandsV1(report);
  const demoHand: WeaponGripPreviewHandV1 = heldStockWeaponReadback?.fixtures[0].hand === "right_hand"
    ? "RIGHT"
    : heldStockWeaponReadback?.fixtures[0].hand === "left_hand"
      ? "LEFT"
      : "OFF";
  const [requestedHand, setRequestedHand] = useState<WeaponGripPreviewHandV1>(() => (
    demoHand
  ));
  useEffect(() => setRequestedHand(demoHand), [
    demoHand,
    heldStockWeaponReadback?.weapon.resref,
    heldStockWeaponReadback?.weapon.resourceScope,
    heldStockWeaponReadback?.weapon.resourceType,
  ]);
  const [showHookAxes, setShowHookAxes] = useState(true);
  const [draftWeaponGrip, setDraftWeaponGrip] = useState(appliedWeaponGrip);
  useEffect(() => setDraftWeaponGrip(appliedWeaponGrip), [appliedWeaponGrip]);
  const previewHand = requestedHand === "OFF" || availableHands.includes(requestedHand)
    ? requestedHand
    : availableHands[0] ?? "OFF";
  const previewInspection = previewHand === "OFF"
    ? undefined
    : inspectWeaponGripPreviewV1(report, previewHand);
  const buildRoot = useCallback(
    async () => buildAuroraReadbackAsset(report, selectedPart, {
      hand: previewHand,
      showHookAxes,
      anchorAuthoring: weaponAnchorAuthoring,
      appliedGrip: appliedWeaponGrip,
      draftGrip: draftWeaponGrip,
    }),
    [appliedWeaponGrip, draftWeaponGrip, previewHand, report, selectedPart, showHookAxes, weaponAnchorAuthoring],
  );
  const hasDraftChange = !sameCreatureWeaponGripOptionsV1(appliedWeaponGrip, draftWeaponGrip);

  return (
    <section className="weapon-grip-preview-shell">
      {availableHands.length > 0 && (
        <section className="weapon-grip-preview" aria-label="Weapon Grip Preview">
          <div className="weapon-grip-preview__heading">
            <div><strong>Weapon Grip Preview</strong><small>{weaponAnchorAuthoring?.calibration ?? "Binary-MDL hook with a basis-equivalent NWN short-sword proxy"}</small></div>
            <span className="weapon-grip-preview__fidelity">NWN SHORTSWORD BASIS PROXY</span>
          </div>
          <p className="weapon-grip-preview__warning">Not exact item geometry or Toolset proof.</p>
          {heldStockWeaponReadback ? (
            <div className="weapon-grip-preview__demo-evidence" aria-label="Demo held item readback">
              <strong>Demo equipment readback: PASS</strong>
              <dl className="weapon-grip-preview__readback">
                <div><dt>Item</dt><dd><code>{heldStockWeaponReadback.weapon.resref}</code></dd></div>
                <div><dt>Resource</dt><dd><code>{heldStockWeaponReadback.weapon.resourceScope} / {heldStockWeaponReadback.weapon.resourceType}</code></dd></div>
                <div><dt>Equipped hand</dt><dd><code>{heldStockWeaponReadback.fixtures[0].hand}</code></dd></div>
              </dl>
            </div>
          ) : (
            <p className="weapon-grip-preview__warning">Current demo has no selected held item; the proxy starts off.</p>
          )}
          <label>
            Preview hand
            <select
              aria-label="Preview hand"
              value={previewHand}
              onChange={(event) => setRequestedHand(event.target.value as WeaponGripPreviewHandV1)}
            >
              <option value="OFF">Off</option>
              {availableHands.includes("RIGHT") && <option value="RIGHT">Right hand</option>}
              {availableHands.includes("LEFT") && <option value="LEFT">Left hand</option>}
            </select>
          </label>
          <label className="weapon-grip-preview__checkbox">
            <input
              type="checkbox"
              checked={showHookAxes}
              onChange={(event) => setShowHookAxes(event.target.checked)}
              disabled={previewHand === "OFF"}
            />
            Show hook axes
          </label>
          {previewInspection ? (
            <dl className="weapon-grip-preview__readback">
              <div><dt>Hook</dt><dd><code>{previewInspection.anchorName}</code></dd></div>
              <div><dt>Parent</dt><dd><code>{previewInspection.parentBoneName}</code></dd></div>
              <div><dt>Auto position</dt><dd><code>{previewInspection.localPosition.map((value) => value.toFixed(5)).join(", ")}</code></dd></div>
              <div><dt>Auto rotation</dt><dd><code>{previewInspection.localOrientation.map((value) => value.toFixed(5)).join(", ")}</code></dd></div>
            </dl>
          ) : <p className="weapon-grip-preview__off">Weapon preview is off.</p>}
          <WeaponGripControls value={draftWeaponGrip} onChange={setDraftWeaponGrip} compact />
          <div className="weapon-grip-preview__apply">
            <span>{hasDraftChange ? "Draft offsets are preview-only until rebuild." : "Preview matches the current binary MDL."}</span>
            <button
              type="button"
              className="button button--primary"
              disabled={!hasDraftChange || !onApplyWeaponGrip}
              onClick={() => onApplyWeaponGrip?.(draftWeaponGrip)}
            >
              Apply and rebuild
            </button>
          </div>
        </section>
      )}
      <SceneViewport
        provenance="READBACK"
        detail="Geometry, skinning, controller tracks and weapon hook reconstructed from canonical Rust binary-MDL readback"
        dependency={`${report.format}:${report.schemaVersion}:${selectedPart?.kind}:${selectedPart?.id}:${previewHand}:${showHookAxes}:${JSON.stringify(draftWeaponGrip)}`}
        buildRoot={buildRoot}
        onSelectPart={onSelectPart}
        onError={onError}
        tools={{ animationPlayback: true, overlays: true }}
      />
    </section>
  );
}
