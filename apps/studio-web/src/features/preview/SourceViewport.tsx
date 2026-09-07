import { useCallback } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { SceneViewport } from "./SceneViewport";
import type { ModelPartRef, SourcePreviewInput } from "./types";
import type { CreatureSourceForwardV1 } from "../source/InputsPanel";
import type {
  ModelMaterialFaceAssignmentV2,
  SourceFaceKeyV2,
} from "../material-separation/types";
import { faceKey } from "../material-separation/types";

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
  faceSelection?: {
    readonly sceneId: number;
    readonly selectedFaceKeys: readonly string[];
    readonly enabled: boolean;
  };
  faceMaterialOverlay?: {
    readonly identity: string;
    readonly assignments: readonly ModelMaterialFaceAssignmentV2[];
    readonly materials: readonly {
      readonly authoredMaterialId: string;
      readonly previewColor: string;
    }[];
    readonly opacity?: number;
  };
  onSelectFaces?: (faces: readonly SourceFaceKeyV2[], additive: boolean) => void;
  onError?: (message: string) => void;
}

const SOURCE_FORWARD_DIRECTIONS: Record<CreatureSourceForwardV1, THREE.Vector3> = {
  POSITIVE_Z: new THREE.Vector3(0, 0, 1),
  NEGATIVE_Z: new THREE.Vector3(0, 0, -1),
  POSITIVE_X: new THREE.Vector3(1, 0, 0),
  NEGATIVE_X: new THREE.Vector3(-1, 0, 0),
};

interface GlbFaceMapV2 {
  readonly primitiveIdByMeshPrimitive: ReadonlyMap<string, number>;
}

function readGlbFaceMapV2(bytes: ArrayBuffer): GlbFaceMapV2 {
  const view = new DataView(bytes);
  if (view.byteLength < 20 || view.getUint32(0, true) !== 0x46546c67
    || view.getUint32(16, true) !== 0x4e4f534a) {
    throw new Error("MATERIAL-SEPARATION-FACE-GLB-JSON-MISSING");
  }
  const jsonLength = view.getUint32(12, true);
  if (20 + jsonLength > view.byteLength) {
    throw new Error("MATERIAL-SEPARATION-FACE-GLB-JSON-TRUNCATED");
  }
  const json = JSON.parse(new TextDecoder().decode(
    new Uint8Array(bytes, 20, jsonLength),
  ).replace(/[\u0000 ]+$/u, "")) as { meshes?: { primitives?: unknown[] }[] };
  const primitiveIdByMeshPrimitive = new Map<string, number>();
  let primitiveId = 0;
  (json.meshes ?? []).forEach((mesh, meshIndex) => {
    (mesh.primitives ?? []).forEach((_, primitiveIndex) => {
      primitiveIdByMeshPrimitive.set(`${meshIndex}/${primitiveIndex}`, primitiveId);
      primitiveId += 1;
    });
  });
  return { primitiveIdByMeshPrimitive };
}

interface GltfAssociationV2 {
  readonly nodes?: number;
  readonly meshes?: number;
  readonly primitives?: number;
}

interface SourceFaceIdentityV2 {
  readonly sceneId: number;
  readonly nodeId: number;
  readonly primitiveId: number;
}

interface SpatialFaceTopologyV2 {
  readonly faceVertexKeys: readonly (readonly string[])[];
  readonly facesByVertexKey: ReadonlyMap<string, readonly number[]>;
  readonly normals: readonly THREE.Vector3[];
}

function spatialFaceTopologyV2(mesh: THREE.Mesh): SpatialFaceTopologyV2 | undefined {
  const cached = mesh.userData.spatialFaceTopologyV2 as SpatialFaceTopologyV2 | undefined;
  if (cached) return cached;
  const position = mesh.geometry.getAttribute("position");
  if (!(position instanceof THREE.BufferAttribute)) return undefined;
  const index = mesh.geometry.index;
  const triangleCount = index ? index.count / 3 : position.count / 3;
  const bounds = mesh.geometry.boundingBox ?? (() => {
    mesh.geometry.computeBoundingBox();
    return mesh.geometry.boundingBox;
  })();
  const diagonal = bounds?.getSize(new THREE.Vector3()).length() ?? 1;
  const epsilon = Math.max(diagonal * 1e-4, 1e-7);
  const quantized = (vertex: number) => [
    Math.round(position.getX(vertex) / epsilon),
    Math.round(position.getY(vertex) / epsilon),
    Math.round(position.getZ(vertex) / epsilon),
  ].join(",");
  const faceVertexKeys: string[][] = [];
  const facesByVertexKey = new Map<string, number[]>();
  const normals: THREE.Vector3[] = [];
  const a = new THREE.Vector3();
  const b = new THREE.Vector3();
  const c = new THREE.Vector3();
  for (let faceIndex = 0; faceIndex < triangleCount; faceIndex += 1) {
    const vertices = [0, 1, 2].map((corner) => (
      index ? index.getX(faceIndex * 3 + corner) : faceIndex * 3 + corner
    ));
    const keys = [...new Set(vertices.map(quantized))];
    faceVertexKeys.push(keys);
    keys.forEach((key) => {
      const faces = facesByVertexKey.get(key) ?? [];
      faces.push(faceIndex);
      facesByVertexKey.set(key, faces);
    });
    a.fromBufferAttribute(position, vertices[0]);
    b.fromBufferAttribute(position, vertices[1]);
    c.fromBufferAttribute(position, vertices[2]);
    normals.push(new THREE.Vector3().subVectors(b, a).cross(new THREE.Vector3().subVectors(c, a)).normalize());
  }
  const topology = { faceVertexKeys, facesByVertexKey, normals } satisfies SpatialFaceTopologyV2;
  mesh.userData.spatialFaceTopologyV2 = topology;
  return topology;
}

function growSpatialFaceRegionV2(mesh: THREE.Mesh, seedFaceIndex: number) {
  const topology = spatialFaceTopologyV2(mesh);
  if (!topology || !topology.faceVertexKeys[seedFaceIndex]) return [seedFaceIndex];
  const selected = new Set([seedFaceIndex]);
  const queue = [seedFaceIndex];
  const minimumNormalDot = Math.cos(45 * Math.PI / 180);
  while (queue.length && selected.size < 50_000) {
    const current = queue.shift()!;
    for (const key of topology.faceVertexKeys[current]) {
      for (const neighbor of topology.facesByVertexKey.get(key) ?? []) {
        if (selected.has(neighbor)) continue;
        if (Math.abs(topology.normals[current].dot(topology.normals[neighbor])) < minimumNormalDot) continue;
        selected.add(neighbor);
        queue.push(neighbor);
      }
    }
  }
  return [...selected];
}

function sourceFaceIdentity(object?: THREE.Object3D | null): SourceFaceIdentityV2 | undefined {
  let candidate = object;
  while (candidate) {
    const identity = candidate.userData.sourceFaceIdentityV2 as SourceFaceIdentityV2 | undefined;
    if (identity) return identity;
    candidate = candidate.parent;
  }
  return undefined;
}

export function SourceViewport({
  input,
  sourceForward,
  componentOverlays = [],
  faceSelection,
  faceMaterialOverlay,
  onSelectFaces,
  onError,
}: Props) {
  const buildRoot = useCallback(async () => {
    const manager = new THREE.LoadingManager();
    manager.setURLModifier((url) => {
      if (url.startsWith("blob:") || url.startsWith("data:")) return url;
      throw new Error("Source preview forbids external GLB resource URLs");
    });
    const bytes = await input.file.arrayBuffer();
    const faceMap = faceSelection?.enabled || faceMaterialOverlay
      ? readGlbFaceMapV2(bytes)
      : undefined;
    const overlaySceneId = faceSelection?.sceneId
      ?? faceMaterialOverlay?.assignments[0]?.selection.sceneId
      ?? 0;
    return new Promise<{ root: THREE.Object3D; animations: readonly THREE.AnimationClip[] }>((resolve, reject) => {
      new GLTFLoader(manager).parse(bytes, "", (gltf) => {
        const associations = (gltf.parser as unknown as {
          associations?: Map<THREE.Object3D, GltfAssociationV2>;
        }).associations;
        gltf.scene.traverse((object) => {
          object.userData.modelPart = {
            kind: "SOURCE_NODE",
            id: object.uuid,
            label: object.name || object.type,
          } satisfies ModelPartRef;
          if (faceMap && object instanceof THREE.Mesh) {
            const association = associations?.get(object);
            let nodeAssociation = association;
            let parent = object.parent;
            while (nodeAssociation?.nodes === undefined && parent) {
              nodeAssociation = associations?.get(parent);
              parent = parent.parent;
            }
            const meshIndex = association?.meshes;
            const primitiveIndex = association?.primitives ?? 0;
            const nodeIndex = association?.nodes ?? nodeAssociation?.nodes;
            const primitiveId = meshIndex === undefined
              ? undefined
              : faceMap.primitiveIdByMeshPrimitive.get(`${meshIndex}/${primitiveIndex}`);
            if (nodeIndex !== undefined && primitiveId !== undefined) {
              object.userData.sourceFaceIdentityV2 = {
                sceneId: overlaySceneId,
                nodeId: nodeIndex,
                primitiveId,
              } satisfies SourceFaceIdentityV2;
            }
          }
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
        if (faceMaterialOverlay) {
          const colorByMaterial = new Map(faceMaterialOverlay.materials.map((material) => (
            [material.authoredMaterialId, material.previewColor]
          )));
          const assignmentsByPrimitive = new Map<string, ModelMaterialFaceAssignmentV2[]>();
          for (const assignment of faceMaterialOverlay.assignments) {
            const key = [
              assignment.selection.sceneId,
              assignment.selection.nodeId,
              assignment.selection.primitiveId,
            ].join("/");
            const grouped = assignmentsByPrimitive.get(key) ?? [];
            grouped.push(assignment);
            assignmentsByPrimitive.set(key, grouped);
          }
          const meshes: THREE.Mesh[] = [];
          gltf.scene.traverse((object) => {
            if (object instanceof THREE.Mesh
              && !object.userData.previewOverlay
              && sourceFaceIdentity(object)) meshes.push(object);
          });
          for (const mesh of meshes) {
            const identity = sourceFaceIdentity(mesh)!;
            const assignments = assignmentsByPrimitive.get([
              identity.sceneId,
              identity.nodeId,
              identity.primitiveId,
            ].join("/")) ?? [];
            if (!assignments.length) continue;
            const position = mesh.geometry.getAttribute("position");
            if (!(position instanceof THREE.BufferAttribute)) continue;
            const index = mesh.geometry.index;
            const triangleCount = index ? index.count / 3 : position.count / 3;
            for (const assignment of assignments) {
              const points: number[] = [];
              for (const range of assignment.selection.triangleRanges) {
                const end = Math.min(range.startTriangle + range.triangleCount, triangleCount);
                for (let triangleIndex = Math.max(range.startTriangle, 0);
                  triangleIndex < end;
                  triangleIndex += 1) {
                  for (let corner = 0; corner < 3; corner += 1) {
                    const vertex = index
                      ? index.getX(triangleIndex * 3 + corner)
                      : triangleIndex * 3 + corner;
                    points.push(position.getX(vertex), position.getY(vertex), position.getZ(vertex));
                  }
                }
              }
              if (!points.length) continue;
              const geometry = new THREE.BufferGeometry();
              geometry.setAttribute("position", new THREE.Float32BufferAttribute(points, 3));
              const material = new THREE.MeshBasicMaterial({
                color: new THREE.Color(colorByMaterial.get(assignment.authoredMaterialId) ?? "#87909b"),
                transparent: true,
                opacity: faceMaterialOverlay.opacity ?? 0.9,
                side: THREE.DoubleSide,
                depthWrite: false,
                polygonOffset: true,
                polygonOffsetFactor: -1.5,
                polygonOffsetUnits: -1.5,
              });
              const overlay = new THREE.Mesh(geometry, material);
              overlay.name = `Material ID ${assignment.authoredMaterialId}`;
              overlay.userData.previewOverlay = "MATERIAL_FACE_ASSIGNMENT_COLORS_V2";
              overlay.renderOrder = 15;
              overlay.raycast = () => undefined;
              mesh.add(overlay);
            }
          }
        }
        if (faceSelection?.enabled && faceSelection.selectedFaceKeys.length) {
          const selected = new Set(faceSelection.selectedFaceKeys);
          const meshes: THREE.Mesh[] = [];
          gltf.scene.traverse((object) => {
            if (object instanceof THREE.Mesh
              && !object.userData.previewOverlay
              && sourceFaceIdentity(object)) meshes.push(object);
          });
          for (const mesh of meshes) {
            const identity = sourceFaceIdentity(mesh)!;
            const position = mesh.geometry.getAttribute("position");
            if (!(position instanceof THREE.BufferAttribute)) continue;
            const index = mesh.geometry.index;
            const triangleCount = index ? index.count / 3 : position.count / 3;
            const points: number[] = [];
            for (let triangleIndex = 0; triangleIndex < triangleCount; triangleIndex += 1) {
              if (!selected.has(faceKey({ ...identity, triangleIndex }))) continue;
              for (let corner = 0; corner < 3; corner += 1) {
                const vertex = index
                  ? index.getX(triangleIndex * 3 + corner)
                  : triangleIndex * 3 + corner;
                points.push(position.getX(vertex), position.getY(vertex), position.getZ(vertex));
              }
            }
            if (!points.length) continue;
            const geometry = new THREE.BufferGeometry();
            geometry.setAttribute("position", new THREE.Float32BufferAttribute(points, 3));
            const material = new THREE.MeshBasicMaterial({
              color: 0xffc247,
              transparent: true,
              opacity: 0.58,
              side: THREE.DoubleSide,
              polygonOffset: true,
              polygonOffsetFactor: -2,
              polygonOffsetUnits: -2,
            });
            const overlay = new THREE.Mesh(geometry, material);
            overlay.name = "Face Mode V2 selection";
            overlay.userData.previewOverlay = "MATERIAL_FACE_SELECTION_V2";
            overlay.renderOrder = 25;
            overlay.raycast = () => undefined;
            mesh.add(overlay);
          }
        }
        resolve({ root: gltf.scene, animations: gltf.animations });
      }, reject);
    });
  }, [componentOverlays, faceMaterialOverlay, faceSelection, input.file, sourceForward]);

  const pickedFace = (object: THREE.Object3D, faceIndex: number): SourceFaceKeyV2 | undefined => {
    const identity = sourceFaceIdentity(object);
    if (!identity || !Number.isSafeInteger(faceIndex) || faceIndex < 0) return undefined;
    return { ...identity, triangleIndex: faceIndex };
  };

  return (
    <SceneViewport
      provenance="SOURCE"
      detail="Original local GLB — viewport only, never proof of Aurora output"
      dependency={`${input.file.name}:${input.sourceSha256}:${sourceForward ?? "NO_FRONT"}:${JSON.stringify(componentOverlays)}:${faceMaterialOverlay?.identity ?? "NO_FACE_MATERIAL_OVERLAY"}`}
      buildRoot={buildRoot}
      tools={{ animationPlayback: true, overlays: true }}
      onSelectIntersection={faceSelection?.enabled && onSelectFaces
        ? (intersection, event) => {
            if (!intersection || intersection.faceIndex == null) return;
            const mesh = intersection.object instanceof THREE.Mesh ? intersection.object : undefined;
            if (!mesh) return;
            const indices = event.altKey
              ? growSpatialFaceRegionV2(mesh, intersection.faceIndex)
              : [intersection.faceIndex];
            const faces = indices
              .map((faceIndex) => pickedFace(mesh, faceIndex))
              .filter((face): face is SourceFaceKeyV2 => face !== undefined);
            if (faces.length) onSelectFaces(faces, event.ctrlKey || event.metaKey || event.shiftKey);
          }
        : undefined}
      onSelectTriangleRectangle={faceSelection?.enabled && onSelectFaces
        ? (triangles, event) => {
            const faces = triangles
              .map((triangle) => pickedFace(triangle.object, triangle.faceIndex))
              .filter((face): face is SourceFaceKeyV2 => face !== undefined);
            if (faces.length) onSelectFaces(faces, event.ctrlKey || event.metaKey);
          }
        : undefined}
      onError={onError}
    />
  );
}
