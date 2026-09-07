import * as THREE from "three";
import type {
  PlaceableAuthoringDocument,
  PlaceableAuthoringElement,
  PlaceableElementInspection,
  PlaceableElementTransform,
} from "./types";
import { placeableGroundRelationV1 } from "./groundPolicy";

export type PlaceableDiagnosticCode =
  | "ELEMENT_BELOW_GROUND"
  | "ELEMENT_FLOATING"
  | "ELEMENT_DISTANT"
  | "GAP_BETWEEN_ELEMENTS"
  | "GAP_ANALYSIS_SKIPPED"
  | "COLLISION_EMPTY"
  | "SOURCE_DISCONNECTED_COMPONENTS";

export interface PlaceableAuthoringDiagnostic {
  readonly code: PlaceableDiagnosticCode;
  readonly severity: "INFO" | "WARNING" | "ERROR";
  readonly elementIds: readonly string[];
  readonly message: string;
}

export interface PlaceableElementMeasurement {
  readonly id: string;
  readonly boundsMin: [number, number, number];
  readonly boundsMax: [number, number, number];
  readonly size: [number, number, number];
  readonly center: [number, number, number];
  readonly distanceFromOrigin: number;
}

export const MAX_PAIRWISE_GAP_DIAGNOSTIC_ELEMENTS = 512;

const live = (element: PlaceableAuthoringElement) => !element.deleted && element.kind !== "GROUP";

function localMatrix(transform: PlaceableElementTransform) {
  const translation = new THREE.Matrix4().makeTranslation(...transform.translation);
  const pivot = new THREE.Matrix4().makeTranslation(...transform.pivot);
  const inversePivot = new THREE.Matrix4().makeTranslation(
    -transform.pivot[0],
    -transform.pivot[1],
    -transform.pivot[2],
  );
  const rotationScale = new THREE.Matrix4().compose(
    new THREE.Vector3(),
    new THREE.Quaternion(...transform.rotationXyzw).normalize(),
    new THREE.Vector3(...transform.scale),
  );
  return translation.multiply(pivot).multiply(rotationScale).multiply(inversePivot);
}

function worldMatrix(
  element: PlaceableAuthoringElement,
  document: PlaceableAuthoringDocument,
  visiting = new Set<string>(),
): THREE.Matrix4 {
  if (!visiting.add(element.id)) return new THREE.Matrix4();
  const parent = element.parentId
    ? document.elements.find((candidate) => candidate.id === element.parentId && !candidate.deleted)
    : undefined;
  const result = parent
    ? worldMatrix(parent, document, visiting).multiply(localMatrix(element.transform))
    : localMatrix(element.transform);
  visiting.delete(element.id);
  return result;
}

function sourceKey(nodeId: number, primitiveId: number | null, componentIndex: number | null) {
  return `${nodeId}:${primitiveId ?? "*"}:${componentIndex ?? "*"}`;
}

function sourceBoundsIndex(inspection: PlaceableElementInspection) {
  const index = new Map<string, THREE.Box3>();
  for (const node of inspection.nodes) {
    const nodeBounds = new THREE.Box3();
    for (const primitive of node.primitives) {
      const primitiveBounds = new THREE.Box3();
      for (const component of primitive.components) {
        const bounds = new THREE.Box3(
          new THREE.Vector3(...component.boundsMin),
          new THREE.Vector3(...component.boundsMax),
        );
        index.set(sourceKey(node.nodeId, primitive.primitiveId, component.componentIndex), bounds);
        primitiveBounds.union(bounds);
        nodeBounds.union(bounds);
      }
      index.set(sourceKey(node.nodeId, primitive.primitiveId, null), primitiveBounds);
    }
    index.set(sourceKey(node.nodeId, null, null), nodeBounds);
  }
  return index;
}

function sourceBounds(
  element: PlaceableAuthoringElement,
  index: ReadonlyMap<string, THREE.Box3>,
): THREE.Box3 | null {
  if (!element.source) return null;
  return index.get(sourceKey(
    element.source.nodeId,
    element.source.primitiveId,
    element.source.componentIndex,
  )) ?? null;
}

function transformBounds(bounds: THREE.Box3, matrix: THREE.Matrix4) {
  const output = new THREE.Box3();
  for (const x of [bounds.min.x, bounds.max.x]) {
    for (const y of [bounds.min.y, bounds.max.y]) {
      for (const z of [bounds.min.z, bounds.max.z]) {
        output.expandByPoint(new THREE.Vector3(x, y, z).applyMatrix4(matrix));
      }
    }
  }
  return output;
}

export function measurePlaceableElements(
  document: PlaceableAuthoringDocument,
  inspection: PlaceableElementInspection,
): readonly PlaceableElementMeasurement[] {
  const boundsIndex = sourceBoundsIndex(inspection);
  return document.elements.filter(live).flatMap((element) => {
    const source = sourceBounds(element, boundsIndex);
    if (!source) return [];
    const bounds = transformBounds(source, worldMatrix(element, document));
    const size = bounds.getSize(new THREE.Vector3());
    const center = bounds.getCenter(new THREE.Vector3());
    return [{
      id: element.id,
      boundsMin: bounds.min.toArray() as [number, number, number],
      boundsMax: bounds.max.toArray() as [number, number, number],
      size: size.toArray() as [number, number, number],
      center: center.toArray() as [number, number, number],
      distanceFromOrigin: center.length(),
    }];
  });
}

function aabbDistance(left: PlaceableElementMeasurement, right: PlaceableElementMeasurement) {
  return Math.hypot(...([0, 1, 2] as const).map((axis) => Math.max(
    left.boundsMin[axis] - right.boundsMax[axis],
    right.boundsMin[axis] - left.boundsMax[axis],
    0,
  )));
}

export function diagnosePlaceableAuthoring(
  document: PlaceableAuthoringDocument,
  inspection: PlaceableElementInspection,
): readonly PlaceableAuthoringDiagnostic[] {
  const measurements = measurePlaceableElements(document, inspection);
  const diagnostics: PlaceableAuthoringDiagnostic[] = [];
  if (
    document.collision.mode === "AUTO_RECTANGLE"
    && !document.elements.some((element) => (
      live(element) && element.source !== null && element.flags.includeInCollision
    ))
  ) {
    diagnostics.push({
      code: "COLLISION_EMPTY",
      severity: "ERROR",
      elementIds: [],
      message: "At least one non-deleted source element must be included in collision before build.",
    });
  }
  if (document.collision.mode === "CUSTOM_POLYGON" && document.collision.vertices.length < 3) {
    diagnostics.push({
      code: "COLLISION_EMPTY",
      severity: "ERROR",
      elementIds: [],
      message: "Custom collision requires at least three polygon vertices before build.",
    });
  }
  const typicalElementSize = Math.max(
    ...measurements.map((measurement) => Math.hypot(...measurement.size)),
    1,
  );
  const distantThreshold = Math.max(typicalElementSize * 4, 5);

  for (const measurement of measurements) {
    const ground = placeableGroundRelationV1(measurement.boundsMin[1]);
    if (ground.state === "BELOW") {
      diagnostics.push({
        code: "ELEMENT_BELOW_GROUND",
        severity: "WARNING",
        elementIds: [measurement.id],
        message: `${measurement.id} intersects the ground plane by ${ground.distanceMeters.toFixed(3)} m.`,
      });
    } else if (ground.state === "FLOATING") {
      diagnostics.push({
        code: "ELEMENT_FLOATING",
        severity: "WARNING",
        elementIds: [measurement.id],
        message: `${measurement.id} floats ${ground.distanceMeters.toFixed(3)} m above the ground plane.`,
      });
    }
    if (measurement.distanceFromOrigin > distantThreshold) {
      diagnostics.push({
        code: "ELEMENT_DISTANT",
        severity: "WARNING",
        elementIds: [measurement.id],
        message: `${measurement.id} is ${measurement.distanceFromOrigin.toFixed(2)} m from the model origin.`,
      });
    }
  }

  if (measurements.length > MAX_PAIRWISE_GAP_DIAGNOSTIC_ELEMENTS) {
    diagnostics.push({
      code: "GAP_ANALYSIS_SKIPPED",
      severity: "INFO",
      elementIds: [],
      message: `Pairwise gap analysis is disabled above ${MAX_PAIRWISE_GAP_DIAGNOSTIC_ELEMENTS} elements to keep authoring responsive.`,
    });
  } else if (measurements.length > 1) {
    for (const measurement of measurements) {
      const nearest = measurements
        .filter((candidate) => candidate.id !== measurement.id)
        .map((candidate) => ({ candidate, distance: aabbDistance(measurement, candidate) }))
        .sort((left, right) => left.distance - right.distance)[0];
      if (nearest && nearest.distance > 0.1 && nearest.distance <= distantThreshold) {
        const pair = [measurement.id, nearest.candidate.id].sort();
        if (!diagnostics.some((diagnostic) => (
          diagnostic.code === "GAP_BETWEEN_ELEMENTS"
          && diagnostic.elementIds.join("|") === pair.join("|")
        ))) {
          diagnostics.push({
            code: "GAP_BETWEEN_ELEMENTS",
            severity: "INFO",
            elementIds: pair,
            message: `A ${nearest.distance.toFixed(3)} m gap separates ${pair.join(" and ")}.`,
          });
        }
      }
    }
  }

  if (inspection.connectedComponentCount > inspection.renderNodeCount) {
    diagnostics.push({
      code: "SOURCE_DISCONNECTED_COMPONENTS",
      severity: "INFO",
      elementIds: [],
      message: `Source contains ${inspection.connectedComponentCount} connected components across ${inspection.renderNodeCount} render nodes.`,
    });
  }
  return diagnostics;
}

export function groundBoundsById(
  measurements: readonly PlaceableElementMeasurement[],
): Readonly<Record<string, readonly [number, number]>> {
  return Object.fromEntries(measurements.map((measurement) => [
    measurement.id,
    [measurement.boundsMin[1], measurement.boundsMax[1]] as const,
  ]));
}
