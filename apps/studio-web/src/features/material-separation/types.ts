import type { ModelMaterialTargetV1 } from "../../worker/types";

export interface SourceComponentKeyV1 {
  readonly sceneId: number;
  readonly nodeId: number;
  readonly primitiveId: number;
  readonly componentIndex: number;
}

export interface ModelComponentInspectionV1 {
  readonly key: SourceComponentKeyV1;
  readonly sourceMaterialId: number | null;
  readonly sourceMaterialName: string | null;
  readonly triangleCount: number;
  readonly vertexCount: number;
  readonly boundsMin: readonly [number, number, number];
  readonly boundsMax: readonly [number, number, number];
}

export interface ModelComponentInventoryV1 {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly sceneId: number;
  readonly renderNodeCount: number;
  readonly primitiveInstanceCount: number;
  readonly triangleCount: number;
  readonly components: readonly ModelComponentInspectionV1[];
}

export interface ModelMaterialCapabilitiesV1 {
  readonly schemaVersion: 1;
  readonly target: ModelMaterialTargetV1;
  readonly materialSeparationSupported: boolean;
  readonly maxMaterialSlots: number;
  readonly maxOutputSections: number;
  readonly selectionGranularity: "CONNECTED_COMPONENTS";
  readonly faceSelectionSupported: false;
  readonly automaticMaterialInference: false;
  readonly preservesSourceUv0: true;
}

export interface ModelMaterialCapabilitiesV2 {
  readonly schemaVersion: 2;
  readonly target: ModelMaterialTargetV1;
  readonly materialSeparationSupported: boolean;
  readonly maxMaterialSlots: number;
  readonly maxOutputSections: number;
  readonly selectionGranularity: "CONNECTED_COMPONENTS_AND_FACES";
  readonly faceSelectionSupported: true;
  readonly automaticMaterialInference: false;
  readonly preservesSourceUv0: true;
}

export interface AuthoredMaterialV1 {
  readonly authoredMaterialId: string;
  readonly displayName: string;
  readonly previewColor: string;
  readonly sourceFallbackMaterialId: number | null;
  readonly sourceFallbackImageSha256: string | null;
}

export interface ModelMaterialAssignmentV1 {
  readonly component: SourceComponentKeyV1;
  readonly authoredMaterialId: string;
}

export interface ModelMaterialSeparationDocumentV1 {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly materials: readonly AuthoredMaterialV1[];
  readonly assignments: readonly ModelMaterialAssignmentV1[];
}

export interface SourceTriangleRangeV2 {
  readonly startTriangle: number;
  readonly triangleCount: number;
}

export interface SourceFaceSelectionV2 {
  readonly sceneId: number;
  readonly nodeId: number;
  readonly primitiveId: number;
  readonly triangleRanges: readonly SourceTriangleRangeV2[];
}

export interface ModelMaterialFaceAssignmentV2 {
  readonly selection: SourceFaceSelectionV2;
  readonly authoredMaterialId: string;
}

export interface ModelMaterialSeparationDocumentV2 {
  readonly schemaVersion: 2;
  readonly sourceSha256: string;
  readonly materials: readonly AuthoredMaterialV1[];
  readonly componentAssignments: readonly ModelMaterialAssignmentV1[];
  readonly faceAssignments: readonly ModelMaterialFaceAssignmentV2[];
}

export interface ModelMaterialSlotReportV1 {
  readonly materialSlot: number;
  readonly authoredMaterialId: string;
  readonly displayName: string;
  readonly previewColor: string;
  readonly sourceMaterialId: number | null;
  readonly sourceMaterialName: string | null;
  readonly sourceImageSha256: string | null;
  readonly systemSourceMaterial: boolean;
  readonly componentCount: number;
  readonly triangleCount: number;
}

export interface ModelMaterialSeparationReportV1 {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly separationSha256: string;
  readonly sourceComponentCount: number;
  readonly assignedComponentCount: number;
  readonly unassignedComponentCount: number;
  readonly sourceTriangleCount: number;
  readonly outputTriangleCount: number;
  readonly sourceVertexCount: number;
  readonly outputVertexCount: number;
  readonly duplicatedBoundaryVertexCount: number;
  readonly outputSectionCount: number;
  readonly predictedTextureCount: number;
  readonly warnings: readonly string[];
  readonly materialSlots: readonly ModelMaterialSlotReportV1[];
}

export interface ModelMaterialSeparationReportV2 {
  readonly schemaVersion: 2;
  readonly sourceSha256: string;
  readonly separationSha256: string;
  readonly sourceComponentCount: number;
  readonly assignedComponentCount: number;
  readonly unassignedComponentCount: number;
  readonly faceAssignmentCount: number;
  readonly triangleRangeCount: number;
  readonly assignedFaceCount: number;
  readonly unassignedFaceCount: number;
  readonly sourceTriangleCount: number;
  readonly outputTriangleCount: number;
  readonly sourceVertexCount: number;
  readonly outputVertexCount: number;
  readonly duplicatedBoundaryVertexCount: number;
  readonly outputSectionCount: number;
  readonly predictedTextureCount: number;
  readonly warnings: readonly string[];
  readonly materialSlots: readonly ModelMaterialSlotReportV1[];
}

export interface ModelComponentInspectionBootstrapV1 {
  readonly schemaVersion: 1;
  readonly capabilities: ModelMaterialCapabilitiesV1;
  readonly inventory: ModelComponentInventoryV1;
  readonly document: ModelMaterialSeparationDocumentV1;
}

export interface ModelMaterialResolutionV1 {
  readonly schemaVersion: 1;
  readonly capabilities: ModelMaterialCapabilitiesV1;
  readonly report: ModelMaterialSeparationReportV1;
  readonly textureAuthoring: ModelTextureAuthoringDocumentV1;
}

export interface ModelFaceInspectionBootstrapV2 {
  readonly schemaVersion: 2;
  readonly capabilities: ModelMaterialCapabilitiesV2;
  readonly inventory: ModelComponentInventoryV1;
  readonly document: ModelMaterialSeparationDocumentV2;
}

export interface ModelMaterialResolutionV2 {
  readonly schemaVersion: 2;
  readonly capabilities: ModelMaterialCapabilitiesV2;
  readonly report: ModelMaterialSeparationReportV2;
  readonly textureAuthoring: ModelTextureAuthoringDocumentV1;
}

export interface ModelTextureBindingAuthoringV1 {
  readonly authoredMaterialId: string;
  readonly materialSlot: number;
  readonly sourceMaterialId: number | null;
  readonly sourceImageSha256: string | null;
  readonly mode: "SOURCE" | "OVERRIDE";
  readonly overrideAssetId: string | null;
  readonly overrideSha256: string | null;
  readonly overrideMimeType: string | null;
  readonly overrideByteLength: number | null;
  readonly alphaPolicy: "OPAQUE_ONLY";
}

export interface ModelTextureAuthoringDocumentV1 {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly separationSha256: string;
  readonly bindings: readonly ModelTextureBindingAuthoringV1[];
}

export type ModelMaterialUvProjectionModeV1 =
  | "SOURCE"
  | "COMPONENT_LONG_AXIS"
  | "MATERIAL_LONG_AXIS"
  | "MATERIAL_BOX"
  | "MATERIAL_BOX_WORLD";

export interface ModelMaterialUvProjectionRuleV1 {
  readonly authoredMaterialId: string;
  readonly mode: ModelMaterialUvProjectionModeV1;
  readonly uRepeats: number;
  readonly vMin: number;
  readonly vMax: number;
  readonly deterministicUPhase: boolean;
}

export interface ModelMaterialUvProjectionDocumentV1 {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly separationSha256: string;
  readonly rules: readonly ModelMaterialUvProjectionRuleV1[];
}

export function materialBoxUvProjectionDocumentV1(
  sourceSha256: string,
  separationSha256: string,
  authoredMaterialIds: readonly string[],
): ModelMaterialUvProjectionDocumentV1 | undefined {
  const ids = [...new Set(authoredMaterialIds)].sort();
  if (!ids.length) return undefined;
  return {
    schemaVersion: 1,
    sourceSha256,
    separationSha256,
    rules: ids.map((authoredMaterialId) => ({
      authoredMaterialId,
      mode: "MATERIAL_BOX",
      uRepeats: 1,
      vMin: 0,
      vMax: 1,
      deterministicUPhase: false,
    })),
  };
}

export function materialBoxWorldUvProjectionDocumentV1(
  sourceSha256: string,
  separationSha256: string,
  authoredMaterialIds: readonly string[],
  repeatsPerMetreByMaterial: Readonly<Record<string, number>> = {},
): ModelMaterialUvProjectionDocumentV1 | undefined {
  const ids = [...new Set(authoredMaterialIds)].sort();
  if (!ids.length) return undefined;
  return {
    schemaVersion: 1,
    sourceSha256,
    separationSha256,
    rules: ids.map((authoredMaterialId) => ({
      authoredMaterialId,
      mode: "MATERIAL_BOX_WORLD",
      uRepeats: repeatsPerMetreByMaterial[authoredMaterialId] ?? 0.5,
      vMin: 0,
      vMax: 1,
      deterministicUPhase: false,
    })),
  };
}

const object = (value: unknown): value is Record<string, unknown> => (
  typeof value === "object" && value !== null && !Array.isArray(value)
);

export function componentKey(value: SourceComponentKeyV1): string {
  return `${value.sceneId}/${value.nodeId}/${value.primitiveId}/${value.componentIndex}`;
}

export interface SourceFaceKeyV2 {
  readonly sceneId: number;
  readonly nodeId: number;
  readonly primitiveId: number;
  readonly triangleIndex: number;
}

export function faceKey(value: SourceFaceKeyV2): string {
  return `${value.sceneId}/${value.nodeId}/${value.primitiveId}/${value.triangleIndex}`;
}

export function parseFaceKey(value: string): SourceFaceKeyV2 {
  const parts = value.split("/").map(Number);
  if (parts.length !== 4 || parts.some((part) => !Number.isSafeInteger(part) || part < 0)) {
    throw new Error("MATERIAL-SEPARATION-FACE-KEY-INVALID");
  }
  return {
    sceneId: parts[0],
    nodeId: parts[1],
    primitiveId: parts[2],
    triangleIndex: parts[3],
  };
}

export interface ModelMaterialResponseStateV1 {
  readonly responseSourceStateId: string;
  readonly responseRecipeStateId: string;
  readonly expectedSourceStateId: string;
  readonly expectedRecipeStateId: string;
  readonly currentSourceSha256: string | null | undefined;
  readonly expectedSourceSha256: string;
  readonly currentTarget: ModelMaterialTargetV1;
  readonly expectedTarget: ModelMaterialTargetV1;
}

export function isCurrentModelMaterialResponseV1(state: ModelMaterialResponseStateV1): boolean {
  return state.responseSourceStateId === state.expectedSourceStateId
    && state.responseRecipeStateId === state.expectedRecipeStateId
    && state.currentSourceSha256 === state.expectedSourceSha256
    && state.currentTarget === state.expectedTarget;
}

export function parseComponentKey(value: string): SourceComponentKeyV1 {
  const parts = value.split("/").map(Number);
  if (parts.length !== 4 || parts.some((part) => !Number.isSafeInteger(part) || part < 0)) {
    throw new Error("MATERIAL-SEPARATION-COMPONENT-KEY-INVALID");
  }
  return {
    sceneId: parts[0],
    nodeId: parts[1],
    primitiveId: parts[2],
    componentIndex: parts[3],
  };
}

export function parseModelComponentInspectionV1(json: string): ModelComponentInspectionBootstrapV1 {
  const value = JSON.parse(json) as unknown;
  if (!object(value) || value.schemaVersion !== 1 || !object(value.capabilities)
    || !object(value.inventory) || !object(value.document)) {
    throw new Error("MODEL-COMPONENT-INSPECTION-INVALID");
  }
  const inventory = value.inventory as unknown as ModelComponentInventoryV1;
  const document = value.document as unknown as ModelMaterialSeparationDocumentV1;
  const capabilities = value.capabilities as unknown as ModelMaterialCapabilitiesV1;
  if (!Array.isArray(inventory.components) || !Array.isArray(document.materials)
    || !Array.isArray(document.assignments) || inventory.sourceSha256 !== document.sourceSha256
    || capabilities.selectionGranularity !== "CONNECTED_COMPONENTS"
    || capabilities.maxMaterialSlots < 1) {
    throw new Error("MODEL-COMPONENT-INSPECTION-INVALID");
  }
  return value as unknown as ModelComponentInspectionBootstrapV1;
}

export function parseModelMaterialResolutionV1(json: string): ModelMaterialResolutionV1 {
  const value = JSON.parse(json) as unknown;
  if (!object(value) || value.schemaVersion !== 1 || !object(value.capabilities)
    || !object(value.report) || !object(value.textureAuthoring)
    || !Array.isArray(value.textureAuthoring.bindings) || !Array.isArray(value.report.materialSlots)
    || !Array.isArray(value.report.warnings)
    || !Number.isSafeInteger(value.capabilities.maxMaterialSlots)
    || (value.capabilities.maxMaterialSlots as number) < 1
    || !Number.isSafeInteger(value.report.predictedTextureCount)
    || (value.report.predictedTextureCount as number) < 0
    || (value.report.predictedTextureCount as number) > (value.capabilities.maxMaterialSlots as number)) {
    throw new Error("MODEL-MATERIAL-RESOLUTION-INVALID");
  }
  return value as unknown as ModelMaterialResolutionV1;
}

export function parseModelFaceInspectionV2(json: string): ModelFaceInspectionBootstrapV2 {
  const value = JSON.parse(json) as unknown;
  if (!object(value) || value.schemaVersion !== 2 || !object(value.capabilities)
    || !object(value.inventory) || !object(value.document)) {
    throw new Error("MODEL-FACE-INSPECTION-INVALID");
  }
  const inventory = value.inventory as unknown as ModelComponentInventoryV1;
  const document = value.document as unknown as ModelMaterialSeparationDocumentV2;
  const capabilities = value.capabilities as unknown as ModelMaterialCapabilitiesV2;
  if (!Array.isArray(inventory.components) || !Array.isArray(document.materials)
    || !Array.isArray(document.componentAssignments) || !Array.isArray(document.faceAssignments)
    || inventory.sourceSha256 !== document.sourceSha256
    || capabilities.selectionGranularity !== "CONNECTED_COMPONENTS_AND_FACES"
    || capabilities.faceSelectionSupported !== true
    || capabilities.maxMaterialSlots < 1) {
    throw new Error("MODEL-FACE-INSPECTION-INVALID");
  }
  return value as unknown as ModelFaceInspectionBootstrapV2;
}

export function parseModelMaterialResolutionV2(json: string): ModelMaterialResolutionV2 {
  const value = JSON.parse(json) as unknown;
  if (!object(value) || value.schemaVersion !== 2 || !object(value.capabilities)
    || !object(value.report) || !object(value.textureAuthoring)
    || !Array.isArray(value.textureAuthoring.bindings) || !Array.isArray(value.report.materialSlots)
    || !Array.isArray(value.report.warnings)
    || !Number.isSafeInteger(value.report.assignedFaceCount)
    || !Number.isSafeInteger(value.capabilities.maxMaterialSlots)
    || (value.capabilities.maxMaterialSlots as number) < 1
    || !Number.isSafeInteger(value.report.predictedTextureCount)
    || (value.report.predictedTextureCount as number) < 0
    || (value.report.predictedTextureCount as number) > (value.capabilities.maxMaterialSlots as number)) {
    throw new Error("MODEL-MATERIAL-RESOLUTION-V2-INVALID");
  }
  return value as unknown as ModelMaterialResolutionV2;
}
