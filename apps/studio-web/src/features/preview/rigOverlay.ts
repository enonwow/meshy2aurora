import * as THREE from "three";
import type { ReadbackNode } from "./types";

export const RIG_NODE_USER_DATA_V1 = "auroraRigNodeV1";

export type RigNodeCategoryV1 =
  | "MODEL_ROOT"
  | "RIG_ROOT"
  | "SKIN_BONE"
  | "RIGID_PIVOT"
  | "ATTACHMENT"
  | "HELPER"
  | "UNKNOWN";

export interface RigNodeDescriptorV1 {
  readonly ordinal: number;
  readonly nodeNumber: number;
  readonly nodeOffset: number;
  readonly name: string;
  readonly parentOrdinal: number | null;
  readonly parentName: string | null;
  readonly depth: number;
  readonly category: RigNodeCategoryV1;
  readonly ownsMesh: boolean;
  readonly activeSkinBone: boolean;
}

export interface ReadbackRigInventoryV1 {
  readonly schemaVersion: 1;
  readonly nodes: readonly RigNodeDescriptorV1[];
  readonly counts: Readonly<Record<RigNodeCategoryV1, number>>;
}

export interface RigConnectionV1 {
  readonly parentOrdinal: number;
  readonly childOrdinal: number;
  readonly parentName: string;
  readonly childName: string;
  readonly helper: boolean;
}

const categories: readonly RigNodeCategoryV1[] = [
  "MODEL_ROOT",
  "RIG_ROOT",
  "SKIN_BONE",
  "RIGID_PIVOT",
  "ATTACHMENT",
  "HELPER",
  "UNKNOWN",
];

const attachmentNames = new Set([
  "impact",
  "headconjure",
  "handconjure",
  "rhand",
  "lhand",
  "rweapon",
  "lweapon",
  "rhook",
  "lhook",
]);

function normalizedName(name: string) {
  return name.trim().toLocaleLowerCase();
}

function isAttachmentName(name: string) {
  const normalized = normalizedName(name);
  return attachmentNames.has(normalized)
    || /(?:conjure|attachment|attach|hook|grip)$/.test(normalized);
}

function isRigRootName(name: string) {
  return /(?:^|[_-])root(?:dummy)?$/.test(normalizedName(name))
    || normalizedName(name).endsWith("rootdummy");
}

function flattenedReadbackNodes(roots: readonly ReadbackNode[]) {
  const result: Array<{
    node: ReadbackNode;
    ordinal: number;
    parentOrdinal: number | null;
    parentName: string | null;
    depth: number;
  }> = [];
  const visit = (node: ReadbackNode, parentOrdinal: number | null, parentName: string | null, depth: number) => {
    const ordinal = result.length;
    result.push({ node, ordinal, parentOrdinal, parentName, depth });
    node.children.forEach((child) => visit(child, ordinal, node.name, depth + 1));
  };
  roots.forEach((root) => visit(root, null, null, 0));
  return result;
}

export function classifyReadbackRigV1(roots: readonly ReadbackNode[]): ReadbackRigInventoryV1 {
  const flattened = flattenedReadbackNodes(roots);
  const activeSkinOrdinals = new Set<number>();
  flattened.forEach(({ node }) => {
    node.skin?.nodeToBoneMap.forEach((slot, ordinal) => {
      if (slot >= 0) activeSkinOrdinals.add(ordinal);
    });
  });

  const nodes = flattened.map(({ node, ordinal, parentOrdinal, parentName, depth }): RigNodeDescriptorV1 => {
    const name = node.name.trim() || `node-${node.number}`;
    const activeSkinBone = activeSkinOrdinals.has(ordinal);
    let category: RigNodeCategoryV1;
    if (!node.name.trim()) category = "UNKNOWN";
    else if (parentOrdinal === null) category = "MODEL_ROOT";
    else if (activeSkinBone) category = "SKIN_BONE";
    else if (isRigRootName(name)) category = "RIG_ROOT";
    else if (isAttachmentName(name)) category = "ATTACHMENT";
    else if (
      node.mesh?.vertices.length
      || node.children.length > 0
      || node.controllers.some(({ controllerName }) => (
        controllerName === "position" || controllerName === "orientation" || controllerName === "scale"
      ))
    ) category = "RIGID_PIVOT";
    else category = "HELPER";
    return {
      ordinal,
      nodeNumber: node.number,
      nodeOffset: node.offset,
      name,
      parentOrdinal,
      parentName,
      depth,
      category,
      ownsMesh: Boolean(node.mesh?.vertices.length),
      activeSkinBone,
    };
  });
  const counts = Object.fromEntries(categories.map((category) => [category, 0])) as Record<RigNodeCategoryV1, number>;
  nodes.forEach(({ category }) => { counts[category] += 1; });
  return { schemaVersion: 1, nodes, counts };
}

function mainRigCategory(category: RigNodeCategoryV1) {
  return category === "RIG_ROOT"
    || category === "SKIN_BONE"
    || category === "RIGID_PIVOT"
    || category === "UNKNOWN";
}

export function visibleRigConnectionsV1(
  rig: ReadbackRigInventoryV1,
  includeHelpers: boolean,
): RigConnectionV1[] {
  return rig.nodes.flatMap((child) => {
    if (child.parentOrdinal === null) return [];
    const parent = rig.nodes[child.parentOrdinal];
    if (!parent) return [];
    const helper = !mainRigCategory(parent.category) || !mainRigCategory(child.category);
    if (helper && !includeHelpers) return [];
    return [{
      parentOrdinal: parent.ordinal,
      childOrdinal: child.ordinal,
      parentName: parent.name,
      childName: child.name,
      helper,
    }];
  });
}

export type RigOverlayOptionV1 = "bones" | "joints" | "helpers" | "labels" | "xray";

interface RuntimeRigNode {
  readonly object: THREE.Bone;
  readonly descriptor: RigNodeDescriptorV1;
}

function descriptorFromBone(bone: THREE.Bone, ordinal: number): RigNodeDescriptorV1 {
  const stored = bone.userData[RIG_NODE_USER_DATA_V1] as RigNodeDescriptorV1 | undefined;
  if (stored) return stored;
  const parent = bone.parent instanceof THREE.Bone ? bone.parent : undefined;
  return {
    ordinal,
    nodeNumber: ordinal,
    nodeOffset: 0,
    name: bone.name.trim() || `bone-${ordinal}`,
    parentOrdinal: parent ? -1 : null,
    parentName: parent?.name ?? null,
    depth: 0,
    category: "SKIN_BONE",
    ownsMesh: false,
    activeSkinBone: true,
  };
}

function collectRuntimeNodes(root: THREE.Object3D): RuntimeRigNode[] {
  const result: RuntimeRigNode[] = [];
  root.traverse((object) => {
    if (object instanceof THREE.Bone) {
      result.push({ object, descriptor: descriptorFromBone(object, result.length) });
    }
  });
  return result;
}

function runtimeEdges(nodes: readonly RuntimeRigNode[]) {
  const byObject = new Map(nodes.map((node) => [node.object, node]));
  return nodes.flatMap((child) => {
    const parent = child.object.parent instanceof THREE.Bone
      ? byObject.get(child.object.parent)
      : undefined;
    if (!parent) return [];
    return [{ parent, child, helper: !mainRigCategory(parent.descriptor.category) || !mainRigCategory(child.descriptor.category) }];
  });
}

function categoryColor(category: RigNodeCategoryV1) {
  if (category === "SKIN_BONE") return 0x35d8e6;
  if (category === "RIGID_PIVOT") return 0x64df85;
  if (category === "RIG_ROOT") return 0xffd166;
  if (category === "ATTACHMENT") return 0xff8c42;
  if (category === "MODEL_ROOT") return 0xa78bfa;
  if (category === "HELPER") return 0xf472b6;
  return 0xd1d5db;
}

function labelSprite(node: RuntimeRigNode, extent: number) {
  const canvas = document.createElement("canvas");
  canvas.width = 256;
  canvas.height = 48;
  const context = canvas.getContext("2d");
  if (context) {
    context.clearRect(0, 0, canvas.width, canvas.height);
    context.font = "600 20px system-ui, sans-serif";
    context.fillStyle = "rgba(4, 12, 18, .82)";
    context.fillRect(0, 0, canvas.width, canvas.height);
    context.fillStyle = `#${categoryColor(node.descriptor.category).toString(16).padStart(6, "0")}`;
    context.fillText(node.descriptor.name, 8, 31, 240);
  }
  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: texture, transparent: true, depthTest: true }));
  sprite.name = `rig-label:${node.descriptor.name}`;
  sprite.scale.set(extent * 0.34, extent * 0.064, 1);
  sprite.userData.previewOverlay = "AURORA_RIG_LABEL_V1";
  sprite.userData[RIG_NODE_USER_DATA_V1] = node.descriptor;
  return sprite;
}

function disposeTree(root: THREE.Object3D) {
  const geometries = new Set<THREE.BufferGeometry>();
  const materials = new Set<THREE.Material>();
  root.traverse((object) => {
    const renderable = object as THREE.Object3D & { geometry?: THREE.BufferGeometry; material?: THREE.Material | THREE.Material[] };
    if (renderable.geometry) geometries.add(renderable.geometry);
    if (renderable.material) {
      (Array.isArray(renderable.material) ? renderable.material : [renderable.material]).forEach((material) => materials.add(material));
    }
  });
  geometries.forEach((geometry) => geometry.dispose());
  materials.forEach((material) => {
    Object.values(material).forEach((value) => {
      if (value instanceof THREE.Texture) value.dispose();
    });
    material.dispose();
  });
}

export class RigOverlayRuntimeV1 {
  readonly inventory: ReadbackRigInventoryV1;
  private readonly nodes: RuntimeRigNode[];
  private readonly edges: ReturnType<typeof runtimeEdges>;
  private readonly mainLines: THREE.LineSegments;
  private readonly helperLines: THREE.LineSegments;
  private readonly mainJoints = new THREE.Group();
  private readonly helperJoints = new THREE.Group();
  private readonly labels = new THREE.Group();
  private readonly labelEntries: Array<{ sprite: THREE.Sprite; node: RuntimeRigNode; helper: boolean }> = [];
  private readonly enabled = new Set<RigOverlayOptionV1>();
  private readonly groups: THREE.Object3D[];
  private readonly nodesByOrdinal: Map<number, RuntimeRigNode>;

  constructor(private readonly scene: THREE.Scene, private readonly root: THREE.Object3D) {
    this.nodes = collectRuntimeNodes(root);
    this.nodesByOrdinal = new Map(this.nodes.map((node) => [node.descriptor.ordinal, node]));
    this.edges = runtimeEdges(this.nodes);
    const counts = Object.fromEntries(categories.map((category) => [category, 0])) as Record<RigNodeCategoryV1, number>;
    this.nodes.forEach(({ descriptor }) => { counts[descriptor.category] += 1; });
    this.inventory = { schemaVersion: 1, nodes: this.nodes.map(({ descriptor }) => descriptor), counts };

    const bounds = new THREE.Box3().setFromObject(root);
    const extent = bounds.isEmpty() ? 1 : Math.max(bounds.getSize(new THREE.Vector3()).length(), 0.1);
    this.mainLines = new THREE.LineSegments(
      new THREE.BufferGeometry(),
      new THREE.LineBasicMaterial({ color: 0x25d9e8, transparent: true, opacity: 0.95 }),
    );
    this.helperLines = new THREE.LineSegments(
      new THREE.BufferGeometry(),
      new THREE.LineDashedMaterial({ color: 0xff8c42, transparent: true, opacity: 0.8, dashSize: extent * 0.025, gapSize: extent * 0.014 }),
    );
    this.mainLines.geometry.setAttribute(
      "position",
      new THREE.BufferAttribute(new Float32Array(this.edges.filter(({ helper }) => !helper).length * 6), 3),
    );
    this.helperLines.geometry.setAttribute(
      "position",
      new THREE.BufferAttribute(new Float32Array(this.edges.filter(({ helper }) => helper).length * 6), 3),
    );
    this.mainLines.name = "aurora-rig-bones-v1";
    this.helperLines.name = "aurora-rig-helper-links-v1";
    this.mainLines.userData.previewOverlay = "AURORA_RIG_BONES_V1";
    this.helperLines.userData.previewOverlay = "AURORA_RIG_HELPERS_V1";

    const sphere = new THREE.SphereGeometry(extent * 0.012, 10, 8);
    const jointMaterials = new Map<RigNodeCategoryV1, THREE.MeshBasicMaterial>();
    this.nodes.forEach((node) => {
      const helper = !mainRigCategory(node.descriptor.category);
      let material = jointMaterials.get(node.descriptor.category);
      if (!material) {
        material = new THREE.MeshBasicMaterial({ color: categoryColor(node.descriptor.category), transparent: true, opacity: 0.96 });
        jointMaterials.set(node.descriptor.category, material);
      }
      const joint = new THREE.Mesh(sphere, material);
      joint.name = `rig-joint:${node.descriptor.name}`;
      joint.userData.previewOverlay = helper ? "AURORA_RIG_HELPER_JOINT_V1" : "AURORA_RIG_JOINT_V1";
      joint.userData[RIG_NODE_USER_DATA_V1] = node.descriptor;
      joint.userData.modelPart = {
        kind: "READBACK_NODE",
        id: node.descriptor.nodeNumber,
        label: node.descriptor.name,
      };
      (helper ? this.helperJoints : this.mainJoints).add(joint);
      if (typeof document !== "undefined") {
        const sprite = labelSprite(node, extent);
        this.labels.add(sprite);
        this.labelEntries.push({ sprite, node, helper });
      }
    });
    this.mainJoints.name = "aurora-rig-joints-v1";
    this.helperJoints.name = "aurora-rig-helper-joints-v1";
    this.labels.name = "aurora-rig-labels-v1";
    this.groups = [this.mainLines, this.helperLines, this.mainJoints, this.helperJoints, this.labels];
    this.groups.forEach((group) => {
      group.visible = false;
      this.scene.add(group);
    });
    this.update();
  }

  set(option: RigOverlayOptionV1, enabled: boolean) {
    if (enabled) this.enabled.add(option);
    else this.enabled.delete(option);
    this.syncVisibility();
  }

  update() {
    this.root.updateWorldMatrix(true, true);
    this.updateLines(this.mainLines, this.edges.filter(({ helper }) => !helper));
    this.updateLines(this.helperLines, this.edges.filter(({ helper }) => helper));
    this.updateJointPositions(this.mainJoints);
    this.updateJointPositions(this.helperJoints);
    this.labelEntries.forEach(({ sprite, node }) => sprite.position.copy(node.object.getWorldPosition(new THREE.Vector3())));
  }

  descriptorFromObject(object?: THREE.Object3D | null) {
    let candidate = object;
    while (candidate) {
      const descriptor = candidate.userData[RIG_NODE_USER_DATA_V1] as RigNodeDescriptorV1 | undefined;
      if (descriptor) return descriptor;
      candidate = candidate.parent;
    }
    return undefined;
  }

  dispose() {
    this.groups.forEach((group) => {
      this.scene.remove(group);
      disposeTree(group);
    });
    this.enabled.clear();
  }

  private syncVisibility() {
    this.mainLines.visible = this.enabled.has("bones");
    this.mainJoints.visible = this.enabled.has("joints");
    this.helperLines.visible = this.enabled.has("helpers");
    this.helperJoints.visible = this.enabled.has("helpers");
    this.labels.visible = this.enabled.has("labels");
    this.labelEntries.forEach(({ sprite, helper }) => {
      sprite.visible = !helper || this.enabled.has("helpers");
    });
    const depthTest = !this.enabled.has("xray");
    this.groups.forEach((group) => group.traverse((object) => {
      const renderable = object as THREE.Object3D & { material?: THREE.Material | THREE.Material[] };
      const objectMaterials = renderable.material
        ? Array.isArray(renderable.material) ? renderable.material : [renderable.material]
        : [];
      objectMaterials.forEach((material) => {
        material.depthTest = depthTest;
        material.needsUpdate = true;
      });
    }));
  }

  private updateLines(
    lines: THREE.LineSegments,
    edges: readonly ReturnType<typeof runtimeEdges>[number][],
  ) {
    const attribute = lines.geometry.getAttribute("position") as THREE.BufferAttribute;
    const positions = attribute.array as Float32Array;
    const point = new THREE.Vector3();
    edges.forEach(({ parent, child }, index) => {
      parent.object.getWorldPosition(point).toArray(positions, index * 6);
      child.object.getWorldPosition(point).toArray(positions, index * 6 + 3);
    });
    attribute.needsUpdate = true;
    lines.geometry.computeBoundingSphere();
    if (lines.material instanceof THREE.LineDashedMaterial) lines.computeLineDistances();
  }

  private updateJointPositions(group: THREE.Group) {
    group.children.forEach((joint) => {
      const descriptor = joint.userData[RIG_NODE_USER_DATA_V1] as RigNodeDescriptorV1;
      const runtimeNode = this.nodesByOrdinal.get(descriptor.ordinal);
      if (runtimeNode) joint.position.copy(runtimeNode.object.getWorldPosition(new THREE.Vector3()));
    });
  }
}
