import * as THREE from "three";
import type {
  PlaceableAuthoringDocument,
  PlaceableAuthoringElement,
  PlaceableElementInspection,
  PlaceableElementTransform,
} from "./types";

export type PlaceableDiagnosticCode =
  | "ELEMENT_BELOW_GROUND"
  | "ELEMENT_FLOATING"
  | "ELEMENT_DISTANT"
  | "GAP_BETWEEN_ELEMENTS"
  | "SOURCE_DISCONNECTED_COMPONENTS";

export interface PlaceableAuthoringDiagnostic {
  readonly code: PlaceableDiagnosticCode;
  readonly severity: "INFO" | "WARNING";
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

function sourceBounds(
  element: PlaceableAuthoringElement,
  inspection: PlaceableElementInspection,
): THREE.Box3 | null {
  if (!element.source) return null;
  const node = inspection.nodes.find((candidate) => candidate.nodeId === element.source?.nodeId);
  if (!node) return null;
  const components = node.primitives.flatMap((primitive) => {
    if (
      element.source?.primitiveId !== null
      && primitive.primitiveId !== element.source?.primitiveId
    ) return [];
    return primitive.components.filter((component) => (
      element.source?.componentIndex === null
      || component.componentIndex === element.source?.componentIndex
    ));
  });
  if (!components.length) return null;
  const bounds = new THREE.Box3();
  for (const component of components) {
    bounds.expandByPoint(new THREE.Vector3(...component.boundsMin));
    bounds.expandByPoint(new THREE.Vector3(...component.boundsMax));
  }
  return bounds;
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
  return document.elements.filter(live).flatMap((element) => {
    const source = sourceBounds(element, inspection);
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
  const typicalElementSize = Math.max(
    ...measurements.map((measurement) => Math.hypot(...measurement.size)),
    1,
  );
  const distantThreshold = Math.max(typicalElementSize * 4, 5);

  for (const measurement of measurements) {
    if (measurement.boundsMin[1] < -0.01) {
      diagnostics.push({
        code: "ELEMENT_BELOW_GROUND",
        severity: "WARNING",
        elementIds: [measurement.id],
        message: `${measurement.id} intersects the ground plane by ${Math.abs(measurement.boundsMin[1]).toFixed(3)} m.`,
      });
    } else if (measurement.boundsMin[1] > 0.05) {
      diagnostics.push({
        code: "ELEMENT_FLOATING",
        severity: "WARNING",
        elementIds: [measurement.id],
        message: `${measurement.id} floats ${measurement.boundsMin[1].toFixed(3)} m above the ground plane.`,
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

  if (measurements.length > 1) {
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
