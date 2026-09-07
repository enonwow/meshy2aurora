import * as THREE from "three";
import type { CreatureSourceForwardV1 } from "../../worker/types";

export interface ReferenceSupermodelJointOverrideV1 {
  readonly carrierPartNumber: number;
  readonly bindLocalMatrix: readonly number[];
  readonly semanticRole: string | null;
  readonly jointAxis: readonly [number, number, number] | null;
  readonly locked: boolean;
}

export interface ReferenceSupermodelVertexWeightOverrideV1 {
  readonly segmentId: number;
  readonly vertexIndex: number;
  readonly influences: readonly {
    readonly boneNodeId: number;
    readonly value: number;
  }[];
}

export interface ReferenceSupermodelRigAuthoringDocumentV1 {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly sourceForward: CreatureSourceForwardV1;
  readonly selectedSupermodelResref: string;
  readonly exactChainSha256: string;
  readonly motionContractSha256: string;
  readonly baseRigSha256: string;
  readonly jointOverrides: readonly ReferenceSupermodelJointOverrideV1[];
  readonly weightOverrides: readonly ReferenceSupermodelVertexWeightOverrideV1[];
  readonly contentSha256: string;
}

export interface ReferenceSupermodelLandmarkOverrideV2 {
  readonly landmarkId: string;
  readonly carrierPartNumber: number;
  readonly targetWorldPosition: readonly [number, number, number];
  readonly locked: boolean;
}

export interface ReferenceSupermodelComponentBindingV2 {
  readonly segmentId: number;
  readonly componentIndex: number;
  readonly regionId: string;
  readonly allowedBoneNodeIds: readonly number[];
  readonly locked: boolean;
}

export interface ReferenceSupermodelRegionWeightConstraintV2 {
  readonly segmentId: number;
  readonly regionId: string;
  readonly vertexIndices: readonly number[];
  readonly allowedBoneNodeIds: readonly number[];
  readonly forbiddenBoneNodeIds: readonly number[];
  readonly maximumInfluenceCount: number;
  readonly locked: boolean;
}

export interface ReferenceSupermodelRigAuthoringDocumentV2 {
  readonly schemaVersion: 2;
  readonly sourceSha256: string;
  readonly sourceForward: CreatureSourceForwardV1;
  readonly selectedSupermodelResref: string;
  readonly exactChainSha256: string;
  readonly motionContractSha256: string;
  readonly structuralProfileSha256: string;
  readonly surfaceAnatomySha256: string;
  readonly fitterAlgorithm: string;
  readonly baseRigSha256: string;
  readonly landmarkOverrides: readonly ReferenceSupermodelLandmarkOverrideV2[];
  readonly jointOverrides: readonly ReferenceSupermodelJointOverrideV1[];
  readonly componentBindings: readonly ReferenceSupermodelComponentBindingV2[];
  readonly regionWeightConstraints: readonly ReferenceSupermodelRegionWeightConstraintV2[];
  readonly weightOverrides: readonly ReferenceSupermodelVertexWeightOverrideV1[];
  readonly contentSha256: string;
}

export interface ReferenceSupermodelTargetRigNodeV1 {
  readonly id: number;
  readonly name: string;
  readonly parentId: number | null;
  readonly bindLocalMatrix: readonly number[];
}

export interface ReferenceSupermodelTargetRigV1 {
  readonly schemaVersion: 1;
  readonly profileId: string;
  readonly contentSha256: string;
  readonly nodes: readonly ReferenceSupermodelTargetRigNodeV1[];
  readonly segments: readonly ReferenceSupermodelTargetRigSegmentV1[];
}

export interface ReferenceSupermodelTargetRigSegmentV1 {
  readonly id: number;
  readonly name: string;
  readonly allowedBoneNodeIds: readonly number[];
  readonly referenceWeights: readonly (readonly {
    readonly boneNodeId: number;
    readonly value: number;
  }[])[];
}

export interface JointTransformFieldsV1 {
  readonly translation: readonly [number, number, number];
  readonly rotationDegrees: readonly [number, number, number];
  readonly scale: readonly [number, number, number];
}

type JsonRecord = Record<string, unknown>;

function record(value: unknown, path: string): JsonRecord {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Invalid rig authoring at ${path}`);
  }
  return value as JsonRecord;
}

function integer(value: unknown, path: string) {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw new Error(`Invalid rig authoring at ${path}`);
  }
  return value as number;
}

function text(value: unknown, path: string) {
  if (typeof value !== "string" || !value.trim()) throw new Error(`Invalid rig authoring at ${path}`);
  return value;
}

function hash(value: unknown, path: string) {
  const candidate = text(value, path);
  if (!/^[0-9a-f]{64}$/.test(candidate)) throw new Error(`Invalid rig authoring at ${path}`);
  return candidate;
}

function finiteNumbers(value: unknown, length: number, path: string) {
  if (!Array.isArray(value) || value.length !== length || value.some((item) => typeof item !== "number" || !Number.isFinite(item))) {
    throw new Error(`Invalid rig authoring at ${path}`);
  }
  return value as number[];
}

function matrix(value: unknown, path: string) {
  return finiteNumbers(value, 16, path);
}

function sourceForward(value: unknown): CreatureSourceForwardV1 {
  if (value !== "POSITIVE_Z" && value !== "NEGATIVE_Z" && value !== "POSITIVE_X" && value !== "NEGATIVE_X") {
    throw new Error("Invalid rig authoring at $.sourceForward");
  }
  return value;
}

function nullableText(value: unknown, path: string) {
  if (value === null) return null;
  return text(value, path);
}

function nullableAxis(value: unknown, path: string): [number, number, number] | null {
  if (value === null) return null;
  const values = finiteNumbers(value, 3, path);
  return [values[0]!, values[1]!, values[2]!];
}

function boolean(value: unknown, path: string) {
  if (typeof value !== "boolean") throw new Error(`Invalid rig authoring at ${path}`);
  return value;
}

function integerArray(value: unknown, path: string) {
  if (!Array.isArray(value)) throw new Error(`Invalid rig authoring at ${path}`);
  const result = value.map((item, index) => integer(item, `${path}[${index}]`));
  if (new Set(result).size !== result.length) throw new Error(`Invalid rig authoring at ${path}: duplicate ids`);
  return result;
}

function jointOverride(value: unknown, path: string): ReferenceSupermodelJointOverrideV1 {
  const row = record(value, path);
  if (typeof row.locked !== "boolean") throw new Error(`Invalid rig authoring at ${path}.locked`);
  return {
    carrierPartNumber: integer(row.carrierPartNumber, `${path}.carrierPartNumber`),
    bindLocalMatrix: matrix(row.bindLocalMatrix, `${path}.bindLocalMatrix`),
    semanticRole: nullableText(row.semanticRole, `${path}.semanticRole`),
    jointAxis: nullableAxis(row.jointAxis, `${path}.jointAxis`),
    locked: row.locked,
  };
}

function weightOverride(value: unknown, path: string): ReferenceSupermodelVertexWeightOverrideV1 {
  const row = record(value, path);
  if (!Array.isArray(row.influences) || row.influences.length === 0 || row.influences.length > 4) {
    throw new Error(`Invalid rig authoring at ${path}.influences`);
  }
  return {
    segmentId: integer(row.segmentId, `${path}.segmentId`),
    vertexIndex: integer(row.vertexIndex, `${path}.vertexIndex`),
    influences: row.influences.map((value, index) => {
      const influence = record(value, `${path}.influences[${index}]`);
      if (typeof influence.value !== "number" || !Number.isFinite(influence.value)) {
        throw new Error(`Invalid rig authoring at ${path}.influences[${index}].value`);
      }
      return {
        boneNodeId: integer(influence.boneNodeId, `${path}.influences[${index}].boneNodeId`),
        value: influence.value,
      };
    }),
  };
}

export function parseReferenceSupermodelRigAuthoringV1(json: string): ReferenceSupermodelRigAuthoringDocumentV1 {
  let parsed: unknown;
  try { parsed = JSON.parse(json); } catch { throw new Error("Invalid rig authoring JSON"); }
  const value = record(parsed, "$" );
  if (value.schemaVersion !== 1) throw new Error("Invalid rig authoring at $.schemaVersion");
  if (!Array.isArray(value.jointOverrides)) throw new Error("Invalid rig authoring at $.jointOverrides");
  if (!Array.isArray(value.weightOverrides)) throw new Error("Invalid rig authoring at $.weightOverrides");
  const resref = text(value.selectedSupermodelResref, "$.selectedSupermodelResref");
  if (!/^[a-z0-9_]{1,16}$/.test(resref)) throw new Error("Invalid rig authoring at $.selectedSupermodelResref");
  return {
    schemaVersion: 1,
    sourceSha256: hash(value.sourceSha256, "$.sourceSha256"),
    sourceForward: sourceForward(value.sourceForward),
    selectedSupermodelResref: resref,
    exactChainSha256: hash(value.exactChainSha256, "$.exactChainSha256"),
    motionContractSha256: hash(value.motionContractSha256, "$.motionContractSha256"),
    baseRigSha256: hash(value.baseRigSha256, "$.baseRigSha256"),
    jointOverrides: value.jointOverrides.map((row, index) => jointOverride(row, `$.jointOverrides[${index}]`)),
    weightOverrides: value.weightOverrides.map((row, index) => weightOverride(row, `$.weightOverrides[${index}]`)),
    contentSha256: hash(value.contentSha256, "$.contentSha256"),
  };
}

export function parseReferenceSupermodelRigAuthoringV2(json: string): ReferenceSupermodelRigAuthoringDocumentV2 {
  let parsed: unknown;
  try { parsed = JSON.parse(json); } catch { throw new Error("Invalid rig authoring JSON"); }
  const value = record(parsed, "$");
  if (value.schemaVersion !== 2) throw new Error("Invalid rig authoring at $.schemaVersion");
  for (const field of ["landmarkOverrides", "jointOverrides", "componentBindings", "regionWeightConstraints", "weightOverrides"] as const) {
    if (!Array.isArray(value[field])) throw new Error(`Invalid rig authoring at $.${field}`);
  }
  const selectedSupermodelResref = text(value.selectedSupermodelResref, "$.selectedSupermodelResref");
  if (!/^[a-z0-9_]{1,16}$/.test(selectedSupermodelResref)) {
    throw new Error("Invalid rig authoring at $.selectedSupermodelResref");
  }
  return {
    schemaVersion: 2,
    sourceSha256: hash(value.sourceSha256, "$.sourceSha256"),
    sourceForward: sourceForward(value.sourceForward),
    selectedSupermodelResref,
    exactChainSha256: hash(value.exactChainSha256, "$.exactChainSha256"),
    motionContractSha256: hash(value.motionContractSha256, "$.motionContractSha256"),
    structuralProfileSha256: hash(value.structuralProfileSha256, "$.structuralProfileSha256"),
    surfaceAnatomySha256: hash(value.surfaceAnatomySha256, "$.surfaceAnatomySha256"),
    fitterAlgorithm: text(value.fitterAlgorithm, "$.fitterAlgorithm"),
    baseRigSha256: hash(value.baseRigSha256, "$.baseRigSha256"),
    landmarkOverrides: (value.landmarkOverrides as unknown[]).map((entry, index) => {
      const row = record(entry, `$.landmarkOverrides[${index}]`);
      const position = finiteNumbers(row.targetWorldPosition, 3, `$.landmarkOverrides[${index}].targetWorldPosition`);
      return {
        landmarkId: text(row.landmarkId, `$.landmarkOverrides[${index}].landmarkId`),
        carrierPartNumber: integer(row.carrierPartNumber, `$.landmarkOverrides[${index}].carrierPartNumber`),
        targetWorldPosition: [position[0]!, position[1]!, position[2]!],
        locked: boolean(row.locked, `$.landmarkOverrides[${index}].locked`),
      };
    }),
    jointOverrides: (value.jointOverrides as unknown[]).map((row, index) => jointOverride(row, `$.jointOverrides[${index}]`)),
    componentBindings: (value.componentBindings as unknown[]).map((entry, index) => {
      const row = record(entry, `$.componentBindings[${index}]`);
      return {
        segmentId: integer(row.segmentId, `$.componentBindings[${index}].segmentId`),
        componentIndex: integer(row.componentIndex, `$.componentBindings[${index}].componentIndex`),
        regionId: text(row.regionId, `$.componentBindings[${index}].regionId`),
        allowedBoneNodeIds: integerArray(row.allowedBoneNodeIds, `$.componentBindings[${index}].allowedBoneNodeIds`),
        locked: boolean(row.locked, `$.componentBindings[${index}].locked`),
      };
    }),
    regionWeightConstraints: (value.regionWeightConstraints as unknown[]).map((entry, index) => {
      const row = record(entry, `$.regionWeightConstraints[${index}]`);
      const maximumInfluenceCount = integer(row.maximumInfluenceCount, `$.regionWeightConstraints[${index}].maximumInfluenceCount`);
      if (maximumInfluenceCount < 1 || maximumInfluenceCount > 4) {
        throw new Error(`Invalid rig authoring at $.regionWeightConstraints[${index}].maximumInfluenceCount`);
      }
      return {
        segmentId: integer(row.segmentId, `$.regionWeightConstraints[${index}].segmentId`),
        regionId: text(row.regionId, `$.regionWeightConstraints[${index}].regionId`),
        vertexIndices: integerArray(row.vertexIndices, `$.regionWeightConstraints[${index}].vertexIndices`),
        allowedBoneNodeIds: integerArray(row.allowedBoneNodeIds, `$.regionWeightConstraints[${index}].allowedBoneNodeIds`),
        forbiddenBoneNodeIds: integerArray(row.forbiddenBoneNodeIds, `$.regionWeightConstraints[${index}].forbiddenBoneNodeIds`),
        maximumInfluenceCount,
        locked: boolean(row.locked, `$.regionWeightConstraints[${index}].locked`),
      };
    }),
    weightOverrides: (value.weightOverrides as unknown[]).map((row, index) => weightOverride(row, `$.weightOverrides[${index}]`)),
    contentSha256: hash(value.contentSha256, "$.contentSha256"),
  };
}

export function upsertLandmarkOverrideV2(
  document: ReferenceSupermodelRigAuthoringDocumentV2,
  override: ReferenceSupermodelLandmarkOverrideV2,
): ReferenceSupermodelRigAuthoringDocumentV2 {
  return {
    ...document,
    landmarkOverrides: document.landmarkOverrides
      .filter((row) => row.landmarkId !== override.landmarkId)
      .concat(override)
      .sort((left, right) => left.landmarkId.localeCompare(right.landmarkId)),
  };
}

export function upsertComponentBindingV2(
  document: ReferenceSupermodelRigAuthoringDocumentV2,
  binding: ReferenceSupermodelComponentBindingV2,
): ReferenceSupermodelRigAuthoringDocumentV2 {
  return {
    ...document,
    componentBindings: document.componentBindings
      .filter((row) => row.segmentId !== binding.segmentId || row.componentIndex !== binding.componentIndex)
      .concat(binding)
      .sort((left, right) => left.segmentId - right.segmentId || left.componentIndex - right.componentIndex),
  };
}

export function upsertRegionWeightConstraintV2(
  document: ReferenceSupermodelRigAuthoringDocumentV2,
  constraint: ReferenceSupermodelRegionWeightConstraintV2,
): ReferenceSupermodelRigAuthoringDocumentV2 {
  return {
    ...document,
    regionWeightConstraints: document.regionWeightConstraints
      .filter((row) => row.segmentId !== constraint.segmentId || row.regionId !== constraint.regionId)
      .concat(constraint)
      .sort((left, right) => left.segmentId - right.segmentId || left.regionId.localeCompare(right.regionId)),
  };
}

export function upsertJointOverrideV2(
  document: ReferenceSupermodelRigAuthoringDocumentV2,
  override: ReferenceSupermodelJointOverrideV1,
): ReferenceSupermodelRigAuthoringDocumentV2 {
  return {
    ...document,
    jointOverrides: document.jointOverrides
      .filter((row) => row.carrierPartNumber !== override.carrierPartNumber)
      .concat(override)
      .sort((left, right) => left.carrierPartNumber - right.carrierPartNumber),
  };
}

export function removeJointOverrideV2(
  document: ReferenceSupermodelRigAuthoringDocumentV2,
  carrierPartNumber: number,
): ReferenceSupermodelRigAuthoringDocumentV2 {
  return {
    ...document,
    jointOverrides: document.jointOverrides.filter((row) => row.carrierPartNumber !== carrierPartNumber),
  };
}

export function upsertWeightOverrideV2(
  document: ReferenceSupermodelRigAuthoringDocumentV2,
  override: ReferenceSupermodelVertexWeightOverrideV1,
): ReferenceSupermodelRigAuthoringDocumentV2 {
  return {
    ...document,
    weightOverrides: document.weightOverrides
      .filter((row) => row.segmentId !== override.segmentId || row.vertexIndex !== override.vertexIndex)
      .concat(override)
      .sort((left, right) => left.segmentId - right.segmentId || left.vertexIndex - right.vertexIndex),
  };
}

export function removeWeightOverrideV2(
  document: ReferenceSupermodelRigAuthoringDocumentV2,
  segmentId: number,
  vertexIndex: number,
): ReferenceSupermodelRigAuthoringDocumentV2 {
  return {
    ...document,
    weightOverrides: document.weightOverrides.filter((row) => (
      row.segmentId !== segmentId || row.vertexIndex !== vertexIndex
    )),
  };
}

export function parseReferenceSupermodelTargetRigV1(json: string): ReferenceSupermodelTargetRigV1 {
  let parsed: unknown;
  try { parsed = JSON.parse(json); } catch { throw new Error("Invalid target rig JSON"); }
  const value = record(parsed, "$targetRig");
  if (value.schemaVersion !== 1 || !Array.isArray(value.nodes) || !Array.isArray(value.segments)) {
    throw new Error("Invalid target rig at $targetRig.schemaVersion, nodes or segments");
  }
  return {
    schemaVersion: 1,
    profileId: text(value.profileId, "$targetRig.profileId"),
    contentSha256: hash(value.contentSha256, "$targetRig.contentSha256"),
    nodes: value.nodes.map((entry, index) => {
      const node = record(entry, `$targetRig.nodes[${index}]`);
      const parentId = node.parentId === null ? null : integer(node.parentId, `$targetRig.nodes[${index}].parentId`);
      return {
        id: integer(node.id, `$targetRig.nodes[${index}].id`),
        name: text(node.name, `$targetRig.nodes[${index}].name`),
        parentId,
        bindLocalMatrix: matrix(node.bindLocalMatrix, `$targetRig.nodes[${index}].bindLocalMatrix`),
      };
    }),
    segments: value.segments.map((entry, index) => {
      const segment = record(entry, `$targetRig.segments[${index}]`);
      if (!Array.isArray(segment.allowedBoneNodeIds) || !Array.isArray(segment.referenceWeights)) {
        throw new Error(`Invalid target rig at $targetRig.segments[${index}]`);
      }
      return {
        id: integer(segment.id, `$targetRig.segments[${index}].id`),
        name: text(segment.name, `$targetRig.segments[${index}].name`),
        allowedBoneNodeIds: segment.allowedBoneNodeIds.map((id, boneIndex) => (
          integer(id, `$targetRig.segments[${index}].allowedBoneNodeIds[${boneIndex}]`)
        )),
        referenceWeights: segment.referenceWeights.map((row, vertexIndex) => {
          if (!Array.isArray(row)) {
            throw new Error(`Invalid target rig at $targetRig.segments[${index}].referenceWeights[${vertexIndex}]`);
          }
          return row.map((entry, influenceIndex) => {
            const influence = record(entry, `$targetRig.segments[${index}].referenceWeights[${vertexIndex}][${influenceIndex}]`);
            if (typeof influence.value !== "number" || !Number.isFinite(influence.value)) {
              throw new Error(`Invalid target rig at $targetRig.segments[${index}].referenceWeights[${vertexIndex}][${influenceIndex}].value`);
            }
            return {
              boneNodeId: integer(influence.boneNodeId, `$targetRig.segments[${index}].referenceWeights[${vertexIndex}][${influenceIndex}].boneNodeId`),
              value: influence.value,
            };
          });
        }),
      };
    }),
  };
}

export function upsertJointOverrideV1(
  document: ReferenceSupermodelRigAuthoringDocumentV1,
  override: ReferenceSupermodelJointOverrideV1,
): ReferenceSupermodelRigAuthoringDocumentV1 {
  const rows = document.jointOverrides
    .filter((row) => row.carrierPartNumber !== override.carrierPartNumber)
    .concat(override)
    .sort((left, right) => left.carrierPartNumber - right.carrierPartNumber);
  return { ...document, jointOverrides: rows };
}

export function removeJointOverrideV1(
  document: ReferenceSupermodelRigAuthoringDocumentV1,
  carrierPartNumber: number,
): ReferenceSupermodelRigAuthoringDocumentV1 {
  return {
    ...document,
    jointOverrides: document.jointOverrides.filter((row) => row.carrierPartNumber !== carrierPartNumber),
  };
}

export function upsertWeightOverrideV1(
  document: ReferenceSupermodelRigAuthoringDocumentV1,
  override: ReferenceSupermodelVertexWeightOverrideV1,
): ReferenceSupermodelRigAuthoringDocumentV1 {
  const rows = document.weightOverrides
    .filter((row) => row.segmentId !== override.segmentId || row.vertexIndex !== override.vertexIndex)
    .concat(override)
    .sort((left, right) => left.segmentId - right.segmentId || left.vertexIndex - right.vertexIndex);
  return { ...document, weightOverrides: rows };
}

export function removeWeightOverrideV1(
  document: ReferenceSupermodelRigAuthoringDocumentV1,
  segmentId: number,
  vertexIndex: number,
): ReferenceSupermodelRigAuthoringDocumentV1 {
  return {
    ...document,
    weightOverrides: document.weightOverrides.filter((row) => (
      row.segmentId !== segmentId || row.vertexIndex !== vertexIndex
    )),
  };
}

export function composeJointMatrixV1(fields: JointTransformFieldsV1): number[] {
  const position = new THREE.Vector3(...fields.translation);
  const rotation = new THREE.Euler(
    THREE.MathUtils.degToRad(fields.rotationDegrees[0]),
    THREE.MathUtils.degToRad(fields.rotationDegrees[1]),
    THREE.MathUtils.degToRad(fields.rotationDegrees[2]),
    "XYZ",
  );
  const quaternion = new THREE.Quaternion().setFromEuler(rotation);
  const scale = new THREE.Vector3(...fields.scale);
  return new THREE.Matrix4().compose(position, quaternion, scale).toArray();
}

export function decomposeJointMatrixV1(matrixValues: readonly number[]): JointTransformFieldsV1 {
  if (matrixValues.length !== 16 || matrixValues.some((value) => !Number.isFinite(value))) {
    throw new Error("Invalid bindLocalMatrix");
  }
  const position = new THREE.Vector3();
  const quaternion = new THREE.Quaternion();
  const scale = new THREE.Vector3();
  new THREE.Matrix4().fromArray([...matrixValues]).decompose(position, quaternion, scale);
  const rotation = new THREE.Euler().setFromQuaternion(quaternion, "XYZ");
  const stable = (value: number) => Math.abs(value) < 1e-10 ? 0 : value;
  return {
    translation: [stable(position.x), stable(position.y), stable(position.z)],
    rotationDegrees: [
      stable(THREE.MathUtils.radToDeg(rotation.x)),
      stable(THREE.MathUtils.radToDeg(rotation.y)),
      stable(THREE.MathUtils.radToDeg(rotation.z)),
    ],
    scale: [stable(scale.x), stable(scale.y), stable(scale.z)],
  };
}
