import { useCallback } from "react";
import * as THREE from "three";
import { SceneViewport } from "./SceneViewport";
import type {
  BinaryMdlInspectionReport,
  ModelPartRef,
  ReadbackController,
  ReadbackMesh,
  ReadbackNode,
} from "./types";

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

function nodeFromReadback(node: ReadbackNode, selectedPart?: ModelPartRef): THREE.Group {
  const group = new THREE.Group();
  group.name = node.name || `node-${node.number}`;
  group.userData.modelPart = {
    kind: "READBACK_NODE",
    id: node.number,
    label: group.name,
  } satisfies ModelPartRef;
  const position = firstController(node.controllers, "position");
  if (position?.length === 3) group.position.fromArray(position);
  const orientation = firstController(node.controllers, "orientation");
  if (orientation?.length === 4) group.quaternion.fromArray(orientation);
  const scale = firstController(node.controllers, "scale");
  if (scale?.length === 1) group.scale.setScalar(scale[0]);

  if (node.mesh?.vertices.length) {
    const selected = selectedPart?.kind === "READBACK_NODE" && String(selectedPart.id) === String(node.number);
    group.add(new THREE.Mesh(
      geometryFromReadback(node.mesh),
      new THREE.MeshStandardMaterial({
        color: selected ? 0x62d6b2 : 0xd6a96c,
        emissive: selected ? 0x164f42 : 0x000000,
        roughness: 0.85,
        side: THREE.DoubleSide,
      }),
    ));
  }
  node.children.forEach((child) => group.add(nodeFromReadback(child, selectedPart)));
  return group;
}

interface Props {
  report: BinaryMdlInspectionReport;
  selectedPart?: ModelPartRef;
  onSelectPart: (part?: ModelPartRef) => void;
  onError?: (message: string) => void;
}

export function AuroraReadbackViewport({ report, selectedPart, onSelectPart, onError }: Props) {
  const buildRoot = useCallback(async () => {
    const root = new THREE.Group();
    report.nodeTree.roots.forEach((node) => root.add(nodeFromReadback(node, selectedPart)));
    return root;
  }, [report, selectedPart]);

  return (
    <SceneViewport
      provenance="READBACK"
      detail="Geometry reconstructed only from canonical Rust binary-MDL readback"
      dependency={`${report.format}:${report.schemaVersion}:${selectedPart?.kind}:${selectedPart?.id}`}
      buildRoot={buildRoot}
      onSelectPart={onSelectPart}
      onError={onError}
    />
  );
}
