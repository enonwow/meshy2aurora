import { useCallback } from "react";
import * as THREE from "three";
import { animationClipsFromReadback } from "./readbackAnimationPlayback";
import { SceneViewport } from "./SceneViewport";
import type {
  BinaryMdlInspectionReport,
  ModelPartRef,
  ReadbackController,
  ReadbackMesh,
  ReadbackNode,
  ReadbackSkin,
} from "./types";

interface MeshPlan {
  owner: THREE.Bone;
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
): THREE.Bone {
  // A Bone is also an Object3D. Keeping the exact readback hierarchy as bones
  // lets Three apply decoded Aurora controller tracks to the same joints that
  // deform the readback mesh.
  const bone = new THREE.Bone();
  bone.name = node.name || `node-${node.number}`;
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
      mesh: node.mesh,
      skin: node.skin,
      selected: selectedPart?.kind === "READBACK_NODE" && String(selectedPart.id) === String(node.number),
    });
  }
  node.children.forEach((child) => bone.add(nodeFromReadback(child, selectedPart, nodeOrder, meshPlans)));
  return bone;
}

function slotBones(skin: ReadbackSkin, nodeOrder: readonly THREE.Bone[]): SkinSlotBones {
  if (skin.nodeToBoneMap.length !== nodeOrder.length) {
    throw new Error(`Canonical skin node-to-bone map has ${skin.nodeToBoneMap.length} entries, but the readback tree has ${nodeOrder.length} nodes`);
  }
  const result: THREE.Bone[] = [];
  const treeOrdinals: number[] = [];
  for (let slot = 0; slot < skin.inlineMapping.length; slot += 1) {
    const ordinal = skin.inlineMapping[slot] ?? -1;
    if (ordinal < 0) break;
    const bone = nodeOrder[ordinal];
    if (!bone || skin.nodeToBoneMap[ordinal] !== slot) {
      throw new Error(`Canonical skin inline mapping disagrees with node-to-bone map at slot ${slot}`);
    }
    result.push(bone);
    treeOrdinals.push(ordinal);
  }
  if (result.length === 0) throw new Error("Canonical skin declares no active bone slots");
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

function attachMeshes(root: THREE.Group, meshPlans: readonly MeshPlan[], nodeOrder: readonly THREE.Bone[]) {
  const skinned: Array<{ mesh: THREE.SkinnedMesh; skin: ReadbackSkin; bones: THREE.Bone[]; treeOrdinals: number[] }> = [];
  meshPlans.forEach((plan) => {
    const geometry = geometryFromReadback(plan.mesh);
    if (!plan.skin) {
      plan.owner.add(new THREE.Mesh(geometry, material(plan.selected)));
      return;
    }
    const slots = slotBones(plan.skin, nodeOrder);
    applySkinAttributes(geometry, plan.mesh, plan.skin, slots.bones);
    const mesh = new THREE.SkinnedMesh(geometry, material(plan.selected));
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
  selectedPart?: ModelPartRef;
  onSelectPart: (part?: ModelPartRef) => void;
  onError?: (message: string) => void;
}

export function buildAuroraReadbackAsset(
  report: BinaryMdlInspectionReport,
  selectedPart?: ModelPartRef,
) {
  const root = new THREE.Group();
  // Binary Aurora MDL is Z-up while Three renders its scene in Y-up. This is
  // preview-only; canonical binary readback values remain untouched.
  root.rotation.x = -Math.PI / 2;
  const nodeOrder: THREE.Bone[] = [];
  const meshPlans: MeshPlan[] = [];
  report.nodeTree.roots.forEach((node) => root.add(nodeFromReadback(node, selectedPart, nodeOrder, meshPlans)));
  attachMeshes(root, meshPlans, nodeOrder);
  return { root, animations: animationClipsFromReadback(report) };
}

export function AuroraReadbackViewport({ report, selectedPart, onSelectPart, onError }: Props) {
  const buildRoot = useCallback(async () => buildAuroraReadbackAsset(report, selectedPart), [report, selectedPart]);

  return (
    <SceneViewport
      provenance="READBACK"
      detail="Geometry, skinning and controller tracks reconstructed only from canonical Rust binary-MDL readback"
      dependency={`${report.format}:${report.schemaVersion}:${selectedPart?.kind}:${selectedPart?.id}`}
      buildRoot={buildRoot}
      onSelectPart={onSelectPart}
      onError={onError}
      tools={{ animationPlayback: true, overlays: true }}
    />
  );
}
