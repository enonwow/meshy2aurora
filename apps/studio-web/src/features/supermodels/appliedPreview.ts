import type { BinaryMdlInspectionReport } from "../preview/types";
import type { WorkerArtifact } from "../../worker/types";
import type {
  ReferenceSupermodelRigAuthoringDocumentV2,
  ReferenceSupermodelTargetRigV1,
} from "./rigAuthoring";

export type ReferenceSupermodelFormatV2 = "BINARY" | "ASCII";

export interface AppliedSupermodelExactChainResourceV2 {
  readonly resref: string;
  readonly supermodelResref: string;
  readonly format: ReferenceSupermodelFormatV2;
  readonly sha256: string;
  readonly byteLength: number;
  readonly localAnimationCount: number;
  readonly nodeCount: number;
  readonly controllerCount: number;
}

export interface AppliedSupermodelStructuralAnalysisV2 {
  readonly status: "REFERENCE_SUPERMODEL_STRUCTURALLY_READY";
  readonly selectedSupermodelResref: string;
  readonly selectedFormat: ReferenceSupermodelFormatV2;
  readonly selectedSha256: string;
  readonly motionCarrierResref: string;
  readonly carrierNodeCount: number;
  readonly carrierControllerCount: number;
  readonly inheritedAnimationNames: readonly string[];
  readonly structuralErrors: readonly string[];
  readonly exactChain: readonly AppliedSupermodelExactChainResourceV2[];
  readonly retailPayloadCopied: false;
}

export interface AppliedSupermodelRigAnalysisV2 {
  readonly algorithm: string;
  readonly carrierNodeCount: number;
  readonly fullCarrierCoverage: boolean;
  readonly requiredJointCoverage: boolean;
  readonly allowedBoneCount: number;
  readonly weightedBoneCount: number;
  readonly activeWeightedBoneCount: number;
  readonly unweightedRequiredJointNames: readonly string[];
  readonly passiveUnweightedJointNames: readonly string[];
  readonly skinInfluenceCoverage: boolean;
  readonly surfaceVertexCount: number;
  readonly surfaceTriangleCount: number;
  readonly duplicatePositionGroupCount: number;
  readonly noReferencePayloadCopied: true;
  readonly surfaceAnatomy?: AppliedSurfaceAnatomyV2;
  readonly jointFit?: AppliedJointFitV2;
  readonly skinning?: AppliedSkinningV2;
  readonly bindPose?: AppliedBindPoseV2;
}

export interface AppliedSurfaceAnatomyV2 {
  readonly status: string;
  readonly authoritativeComponentCount: number;
  readonly auxiliaryComponentCount: number;
  readonly virtualWeldGroupCount: number;
  readonly analysisSurfaceVertexCount: number;
  readonly authoritativeSurfaceVertexCount: number;
  readonly ambiguities: readonly string[];
}

export interface AppliedJointFitConstraintV2 {
  readonly code: string;
  readonly status: string;
  readonly partNumbers: readonly number[];
  readonly measuredFraction: number;
  readonly allowedFraction: number;
  readonly message: string;
}

export interface AppliedFittedJointV2 {
  readonly partNumber: number;
  readonly jointName: string;
  readonly referenceWorldPosition: readonly [number, number, number];
  readonly initialTargetWorldPosition: readonly [number, number, number];
  readonly targetWorldPosition: readonly [number, number, number];
  readonly residualFraction: number;
  readonly semanticRegion: string;
  readonly constraintVerdict: string;
  readonly provenance: string;
  readonly confidence: number;
  readonly constraints: readonly string[];
}

export interface AppliedJointFitV2 {
  readonly status: string;
  readonly minimumConfidence: number;
  readonly constraintRequiredCount: number;
  readonly constraintPassCount: number;
  readonly constraintViolations: readonly AppliedJointFitConstraintV2[];
  readonly constraints: readonly AppliedJointFitConstraintV2[];
  readonly joints: readonly AppliedFittedJointV2[];
}

export interface AppliedSkinningV2 {
  readonly status: string;
  readonly crossSideLeakageVertexCount: number;
  readonly crossBranchLeakageVertexCount: number;
  readonly crossBranchTriangleCount: number;
  readonly auxiliaryComponentCount: number;
  readonly auxiliaryProjectionCount: number;
  readonly auxiliaryProjectionCoverage: boolean;
  readonly branchBoundaryRepairVertexCount: number;
  readonly branchBoundaryRepairMaximumObservedVertexCount: number;
  readonly branchBoundaryRepairLimitVertexCount: number;
  readonly branchBoundaryRepairLimitExceeded: boolean;
  readonly branchBoundaryRepairLimitBypassEnabled: boolean;
  readonly warnings: readonly string[];
  readonly validationViolations: readonly string[];
}

export interface AppliedBindPoseV2 {
  readonly status: string;
  readonly requiredCheckCount: number;
  readonly passCount: number;
  readonly violations: readonly AppliedJointFitConstraintV2[];
}

export interface AppliedAdmissionStageV3 {
  readonly stage: string;
  readonly sourceStatus: string;
  readonly status: "PASS" | "BLOCKED";
  readonly blockingCodes: readonly string[];
}

export interface AppliedAdmissionV3 {
  readonly schemaVersion: 3;
  readonly mode: "DIAGNOSTIC" | "PRODUCT";
  readonly status: "PASS" | "DIAGNOSTIC_BLOCKED" | "BLOCKED";
  readonly exportAllowed: boolean;
  readonly firstBlockingStage: string | null;
  readonly blockingCodes: readonly string[];
  readonly stages: readonly AppliedAdmissionStageV3[];
}

export interface AppliedMotionQualitySummaryV2 {
  readonly status: "PASS" | "BLOCKED";
  readonly edgeOutsideHardLimitCount: number;
  readonly edgeOutsideHardAllowedCount: number;
  readonly triangleAreaCollapseCount: number;
  readonly triangleAreaCollapseAllowedCount: number;
  readonly seamPairViolationCount: number;
  readonly seamPairAllowedCount: number;
  readonly visibleAnchorMotionViolationCount: number;
  readonly requiredClipCount: number;
  readonly sampledClipCount: number;
  readonly inheritedClipCoverage: boolean;
  readonly requiredJointCount: number;
  readonly jointClipRequiredCount: number;
  readonly jointClipPassCount: number;
  readonly jointClipCoverage: boolean;
  readonly visibleMotionCoverage: boolean;
  readonly surfaceSeamGateStatus: "PASS" | "PASS_NO_SEAM_PAIRS" | "BLOCKED_VISIBLE_SEAM";
}

export interface AppliedSupermodelReportV2 {
  readonly schemaVersion: 2;
  readonly status: "APPLIED_PREVIEW_READY" | "APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY" | "APPLIED_PREVIEW_OFFLINE_PASS" | "DIAGNOSTIC_BLOCKED";
  readonly supermodelResref: string;
  readonly referenceFormat: ReferenceSupermodelFormatV2;
  readonly referenceSha256: string;
  readonly modelSha256: string;
  readonly localAnimationCount: number;
  readonly inheritedAnimationCount: number;
  readonly requiredClipCount: number;
  readonly motionCompatible: boolean;
  readonly bindPoseCompatible: boolean;
  readonly skinBindCompatible: boolean;
  readonly fullCarrierCoverage: boolean;
  readonly requiredJointCoverage: boolean;
  readonly skinInfluenceCoverage: boolean;
  readonly inheritedClipCoverage: boolean;
  readonly visibleMotionCoverage: boolean;
  readonly seamViolationCount: number;
  readonly motionQualityStatus: "PASS" | "BLOCKED";
  readonly runtimeReadiness: "RUNTIME_UNPROVEN" | "DIAGNOSTIC_PREVIEW_ONLY_MOTION_QUALITY_BLOCKED";
  readonly exactChain: readonly AppliedSupermodelExactChainResourceV2[];
  readonly structuralAnalysis: AppliedSupermodelStructuralAnalysisV2;
  readonly rigAnalysis: AppliedSupermodelRigAnalysisV2;
  readonly experimentalAllowExcessiveSkinBranchRepair: boolean;
  readonly admissionV3?: AppliedAdmissionV3;
  readonly retailPayloadCopied: false;
  readonly motionQuality: AppliedMotionQualitySummaryV2;
}

export interface AppliedSupermodelPreviewV2 {
  readonly sourceName: string;
  readonly sourceSha256: string;
  readonly supermodelResref: string;
  readonly experimentalAllowExcessiveSkinBranchRepair: boolean;
  readonly target: BinaryMdlInspectionReport;
  readonly report: AppliedSupermodelReportV2;
  readonly rigAuthoring: ReferenceSupermodelRigAuthoringDocumentV2;
  readonly targetRig: ReferenceSupermodelTargetRigV1;
  readonly diagnosticArtifacts: readonly WorkerArtifact[];
}

type JsonRecord = Record<string, unknown>;

function record(value: unknown, path: string): JsonRecord {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Invalid applied supermodel report at ${path}`);
  }
  return value as JsonRecord;
}
function exactLiteral<T extends string | number | boolean>(value: unknown, expected: T, path: string): T {
  if (value !== expected) throw new Error(`Invalid applied supermodel report at ${path}`);
  return expected;
}
function sha256(value: unknown, path: string) {
  if (typeof value !== "string" || !/^[0-9a-f]{64}$/.test(value)) {
    throw new Error(`Invalid applied supermodel report at ${path}`);
  }
  return value;
}
function integer(value: unknown, path: string) {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw new Error(`Invalid applied supermodel report at ${path}`);
  }
  return value as number;
}
function finite(value: unknown, path: string) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`Invalid applied supermodel report at ${path}`);
  }
  return value;
}
function boolean(value: unknown, path: string) {
  if (typeof value !== "boolean") throw new Error(`Invalid applied supermodel report at ${path}`);
  return value;
}
function nonEmptyString(value: unknown, path: string) {
  if (typeof value !== "string" || !value.trim()) throw new Error(`Invalid applied supermodel report at ${path}`);
  return value;
}
function format(value: unknown, path: string): ReferenceSupermodelFormatV2 {
  if (value !== "BINARY" && value !== "ASCII") {
    throw new Error(`Invalid applied supermodel report at ${path}`);
  }
  return value;
}
function strings(value: unknown, path: string) {
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string")) {
    throw new Error(`Invalid applied supermodel report at ${path}`);
  }
  return value as string[];
}
function vector3(value: unknown, path: string): [number, number, number] {
  if (!Array.isArray(value) || value.length !== 3 || value.some((item) => typeof item !== "number" || !Number.isFinite(item))) {
    throw new Error(`Invalid applied supermodel report at ${path}`);
  }
  return value as [number, number, number];
}
function constraint(value: unknown, path: string, defaultStatus?: "PASS" | "BLOCKED"): AppliedJointFitConstraintV2 {
  const item = record(value, path);
  const status = item.status ?? defaultStatus;
  if (status !== "PASS" && status !== "BLOCKED") {
    throw new Error(`Invalid applied supermodel report at ${path}.status`);
  }
  if (!Array.isArray(item.partNumbers) || item.partNumbers.some((part) => !Number.isSafeInteger(part) || (part as number) < 0)) {
    throw new Error(`Invalid applied supermodel report at ${path}.partNumbers`);
  }
  return {
    code: nonEmptyString(item.code, `${path}.code`),
    status,
    partNumbers: item.partNumbers as number[],
    measuredFraction: finite(item.measuredFraction, `${path}.measuredFraction`),
    allowedFraction: finite(item.allowedFraction, `${path}.allowedFraction`),
    message: nonEmptyString(item.message, `${path}.message`),
  };
}
function constraints(value: unknown, path: string) {
  if (!Array.isArray(value)) throw new Error(`Invalid applied supermodel report at ${path}`);
  return value.map((item, index) => constraint(item, `${path}[${index}]`));
}
function admissionV3(value: unknown): AppliedAdmissionV3 | undefined {
  if (value === undefined) return undefined;
  const admission = record(value, "report.admissionV3");
  if (admission.mode !== "DIAGNOSTIC" && admission.mode !== "PRODUCT") throw new Error("Invalid applied supermodel report at report.admissionV3.mode");
  if (admission.status !== "PASS" && admission.status !== "DIAGNOSTIC_BLOCKED" && admission.status !== "BLOCKED") throw new Error("Invalid applied supermodel report at report.admissionV3.status");
  if (admission.firstBlockingStage !== null && typeof admission.firstBlockingStage !== "string") throw new Error("Invalid applied supermodel report at report.admissionV3.firstBlockingStage");
  if (!Array.isArray(admission.stages)) throw new Error("Invalid applied supermodel report at report.admissionV3.stages");
  const stages = admission.stages.map((stage, index): AppliedAdmissionStageV3 => {
    const item = record(stage, `report.admissionV3.stages[${index}]`);
    if (item.status !== "PASS" && item.status !== "BLOCKED") throw new Error(`Invalid applied supermodel report at report.admissionV3.stages[${index}].status`);
    return {
      stage: nonEmptyString(item.stage, `report.admissionV3.stages[${index}].stage`),
      sourceStatus: nonEmptyString(item.sourceStatus, `report.admissionV3.stages[${index}].sourceStatus`),
      status: item.status,
      blockingCodes: strings(item.blockingCodes, `report.admissionV3.stages[${index}].blockingCodes`),
    };
  });
  const parsed: AppliedAdmissionV3 = {
    schemaVersion: exactLiteral(admission.schemaVersion, 3, "report.admissionV3.schemaVersion"),
    mode: admission.mode,
    status: admission.status,
    exportAllowed: boolean(admission.exportAllowed, "report.admissionV3.exportAllowed"),
    firstBlockingStage: admission.firstBlockingStage as string | null,
    blockingCodes: strings(admission.blockingCodes, "report.admissionV3.blockingCodes"),
    stages,
  };
  if ((parsed.status === "PASS") !== parsed.stages.every((stage) => stage.status === "PASS")) {
    throw new Error("Applied supermodel admission has an inconsistent stage verdict");
  }
  return parsed;
}
function optionalInteger(value: unknown, path: string, fallback = 0) {
  return value === undefined ? fallback : integer(value, path);
}
function optionalFinite(value: unknown, path: string, fallback = 0) {
  return value === undefined ? fallback : finite(value, path);
}
function optionalStrings(value: unknown, path: string) {
  return value === undefined ? [] : strings(value, path);
}
function surfaceAnatomy(value: unknown): AppliedSurfaceAnatomyV2 | undefined {
  if (value === undefined) return undefined;
  const anatomy = record(value, "report.rigAnalysis.surfaceAnatomy");
  return {
    status: nonEmptyString(anatomy.status, "report.rigAnalysis.surfaceAnatomy.status"),
    authoritativeComponentCount: optionalInteger(anatomy.authoritativeComponentCount, "report.rigAnalysis.surfaceAnatomy.authoritativeComponentCount"),
    auxiliaryComponentCount: optionalInteger(anatomy.auxiliaryComponentCount, "report.rigAnalysis.surfaceAnatomy.auxiliaryComponentCount"),
    virtualWeldGroupCount: optionalInteger(anatomy.virtualWeldGroupCount, "report.rigAnalysis.surfaceAnatomy.virtualWeldGroupCount"),
    analysisSurfaceVertexCount: optionalInteger(anatomy.analysisSurfaceVertexCount, "report.rigAnalysis.surfaceAnatomy.analysisSurfaceVertexCount"),
    authoritativeSurfaceVertexCount: optionalInteger(anatomy.authoritativeSurfaceVertexCount, "report.rigAnalysis.surfaceAnatomy.authoritativeSurfaceVertexCount"),
    ambiguities: optionalStrings(anatomy.ambiguities, "report.rigAnalysis.surfaceAnatomy.ambiguities"),
  };
}
function jointFit(value: unknown): AppliedJointFitV2 | undefined {
  if (value === undefined) return undefined;
  const fit = record(value, "report.rigAnalysis.jointFit");
  const parsedConstraints = fit.constraints === undefined ? [] : constraints(fit.constraints, "report.rigAnalysis.jointFit.constraints");
  const parsedViolations = fit.constraintViolations === undefined ? parsedConstraints.filter((item) => item.status !== "PASS") : constraints(fit.constraintViolations, "report.rigAnalysis.jointFit.constraintViolations");
  if (!Array.isArray(fit.joints)) throw new Error("Invalid applied supermodel report at report.rigAnalysis.jointFit.joints");
  const joints = fit.joints.map((value, index): AppliedFittedJointV2 => {
    const path = `report.rigAnalysis.jointFit.joints[${index}]`;
    const item = record(value, path);
    const target = vector3(item.targetWorldPosition, `${path}.targetWorldPosition`);
    return {
      partNumber: integer(item.partNumber, `${path}.partNumber`),
      jointName: nonEmptyString(item.jointName, `${path}.jointName`),
      referenceWorldPosition: item.referenceWorldPosition === undefined ? target : vector3(item.referenceWorldPosition, `${path}.referenceWorldPosition`),
      initialTargetWorldPosition: item.initialTargetWorldPosition === undefined ? target : vector3(item.initialTargetWorldPosition, `${path}.initialTargetWorldPosition`),
      targetWorldPosition: target,
      residualFraction: optionalFinite(item.residualFraction, `${path}.residualFraction`),
      semanticRegion: item.semanticRegion === undefined ? "LEGACY_UNREPORTED" : nonEmptyString(item.semanticRegion, `${path}.semanticRegion`),
      constraintVerdict: item.constraintVerdict === undefined ? "UNREPORTED" : nonEmptyString(item.constraintVerdict, `${path}.constraintVerdict`),
      provenance: nonEmptyString(item.provenance, `${path}.provenance`),
      confidence: finite(item.confidence, `${path}.confidence`),
      constraints: optionalStrings(item.constraints, `${path}.constraints`),
    };
  });
  return {
    status: nonEmptyString(fit.status, "report.rigAnalysis.jointFit.status"),
    minimumConfidence: finite(fit.minimumConfidence, "report.rigAnalysis.jointFit.minimumConfidence"),
    constraintRequiredCount: optionalInteger(fit.constraintRequiredCount, "report.rigAnalysis.jointFit.constraintRequiredCount", parsedConstraints.length),
    constraintPassCount: optionalInteger(fit.constraintPassCount, "report.rigAnalysis.jointFit.constraintPassCount", parsedConstraints.filter((item) => item.status === "PASS").length),
    constraintViolations: parsedViolations,
    constraints: parsedConstraints,
    joints,
  };
}
function skinning(value: unknown): AppliedSkinningV2 | undefined {
  if (value === undefined) return undefined;
  const skin = record(value, "report.rigAnalysis.skinning");
  return {
    status: nonEmptyString(skin.status, "report.rigAnalysis.skinning.status"),
    crossSideLeakageVertexCount: integer(skin.crossSideLeakageVertexCount, "report.rigAnalysis.skinning.crossSideLeakageVertexCount"),
    crossBranchLeakageVertexCount: integer(skin.crossBranchLeakageVertexCount, "report.rigAnalysis.skinning.crossBranchLeakageVertexCount"),
    crossBranchTriangleCount: integer(skin.crossBranchTriangleCount, "report.rigAnalysis.skinning.crossBranchTriangleCount"),
    auxiliaryComponentCount: optionalInteger(skin.auxiliaryComponentCount, "report.rigAnalysis.skinning.auxiliaryComponentCount"),
    auxiliaryProjectionCount: optionalInteger(skin.auxiliaryProjectionCount, "report.rigAnalysis.skinning.auxiliaryProjectionCount"),
    auxiliaryProjectionCoverage: skin.auxiliaryProjectionCoverage === undefined ? false : boolean(skin.auxiliaryProjectionCoverage, "report.rigAnalysis.skinning.auxiliaryProjectionCoverage"),
    branchBoundaryRepairVertexCount: optionalInteger(skin.branchBoundaryRepairVertexCount, "report.rigAnalysis.skinning.branchBoundaryRepairVertexCount"),
    branchBoundaryRepairMaximumObservedVertexCount: optionalInteger(skin.branchBoundaryRepairMaximumObservedVertexCount, "report.rigAnalysis.skinning.branchBoundaryRepairMaximumObservedVertexCount"),
    branchBoundaryRepairLimitVertexCount: optionalInteger(skin.branchBoundaryRepairLimitVertexCount, "report.rigAnalysis.skinning.branchBoundaryRepairLimitVertexCount"),
    branchBoundaryRepairLimitExceeded: skin.branchBoundaryRepairLimitExceeded === undefined ? false : boolean(skin.branchBoundaryRepairLimitExceeded, "report.rigAnalysis.skinning.branchBoundaryRepairLimitExceeded"),
    branchBoundaryRepairLimitBypassEnabled: skin.branchBoundaryRepairLimitBypassEnabled === undefined ? false : boolean(skin.branchBoundaryRepairLimitBypassEnabled, "report.rigAnalysis.skinning.branchBoundaryRepairLimitBypassEnabled"),
    warnings: optionalStrings(skin.warnings, "report.rigAnalysis.skinning.warnings"),
    validationViolations: optionalStrings(skin.validationViolations, "report.rigAnalysis.skinning.validationViolations"),
  };
}
function bindPose(value: unknown): AppliedBindPoseV2 | undefined {
  if (value === undefined) return undefined;
  const bind = record(value, "report.rigAnalysis.bindPose");
  return {
    status: nonEmptyString(bind.status, "report.rigAnalysis.bindPose.status"),
    requiredCheckCount: integer(bind.requiredCheckCount, "report.rigAnalysis.bindPose.requiredCheckCount"),
    passCount: integer(bind.passCount, "report.rigAnalysis.bindPose.passCount"),
    violations: Array.isArray(bind.violations)
      ? bind.violations.map((item, index) => constraint(item, `report.rigAnalysis.bindPose.violations[${index}]`, "BLOCKED"))
      : (() => { throw new Error("Invalid applied supermodel report at report.rigAnalysis.bindPose.violations"); })(),
  };
}
function exactChain(value: unknown, path: string): AppliedSupermodelExactChainResourceV2[] {
  if (!Array.isArray(value) || value.length === 0) {
    throw new Error(`Invalid applied supermodel report at ${path}`);
  }
  return value.map((item, index) => {
    const resource = record(item, `${path}[${index}]`);
    return {
      resref: nonEmptyString(resource.resref, `${path}[${index}].resref`),
      supermodelResref: nonEmptyString(resource.supermodelResref, `${path}[${index}].supermodelResref`),
      format: format(resource.format, `${path}[${index}].format`),
      sha256: sha256(resource.sha256, `${path}[${index}].sha256`),
      byteLength: integer(resource.byteLength, `${path}[${index}].byteLength`),
      localAnimationCount: integer(resource.localAnimationCount, `${path}[${index}].localAnimationCount`),
      nodeCount: integer(resource.nodeCount, `${path}[${index}].nodeCount`),
      controllerCount: integer(resource.controllerCount, `${path}[${index}].controllerCount`),
    };
  });
}
function motionQualitySummary(value: unknown): AppliedMotionQualitySummaryV2 {
  const quality = record(value, "report.motionQuality");
  if (quality.status !== "PASS" && quality.status !== "BLOCKED") {
    throw new Error("Invalid applied supermodel report at report.motionQuality.status");
  }
  const seamGate = record(quality.surfaceSeamGate, "report.motionQuality.surfaceSeamGate");
  if (seamGate.status !== "PASS" && seamGate.status !== "PASS_NO_SEAM_PAIRS" && seamGate.status !== "BLOCKED_VISIBLE_SEAM") {
    throw new Error("Invalid applied supermodel report at report.motionQuality.surfaceSeamGate.status");
  }
  return {
    status: quality.status,
    edgeOutsideHardLimitCount: integer(quality.edgeOutsideHardLimitCount, "report.motionQuality.edgeOutsideHardLimitCount"),
    edgeOutsideHardAllowedCount: integer(quality.edgeOutsideHardAllowedCount, "report.motionQuality.edgeOutsideHardAllowedCount"),
    triangleAreaCollapseCount: integer(quality.triangleAreaCollapseCount, "report.motionQuality.triangleAreaCollapseCount"),
    triangleAreaCollapseAllowedCount: integer(quality.triangleAreaCollapseAllowedCount, "report.motionQuality.triangleAreaCollapseAllowedCount"),
    seamPairViolationCount: integer(quality.seamPairViolationCount, "report.motionQuality.seamPairViolationCount"),
    seamPairAllowedCount: integer(quality.seamPairAllowedCount, "report.motionQuality.seamPairAllowedCount"),
    visibleAnchorMotionViolationCount: integer(quality.visibleAnchorMotionViolationCount, "report.motionQuality.visibleAnchorMotionViolationCount"),
    requiredClipCount: integer(quality.requiredClipCount, "report.motionQuality.requiredClipCount"),
    sampledClipCount: integer(quality.sampledClipCount, "report.motionQuality.sampledClipCount"),
    inheritedClipCoverage: boolean(quality.inheritedClipCoverage, "report.motionQuality.inheritedClipCoverage"),
    requiredJointCount: integer(quality.requiredJointCount, "report.motionQuality.requiredJointCount"),
    jointClipRequiredCount: integer(quality.jointClipRequiredCount, "report.motionQuality.jointClipRequiredCount"),
    jointClipPassCount: integer(quality.jointClipPassCount, "report.motionQuality.jointClipPassCount"),
    jointClipCoverage: boolean(quality.jointClipCoverage, "report.motionQuality.jointClipCoverage"),
    visibleMotionCoverage: boolean(quality.visibleMotionCoverage, "report.motionQuality.visibleMotionCoverage"),
    surfaceSeamGateStatus: seamGate.status,
  };
}

export function parseAppliedSupermodelReportV2(json: string): AppliedSupermodelReportV2 {
  let parsed: unknown;
  try { parsed = JSON.parse(json); } catch { throw new Error("Invalid applied supermodel report JSON"); }
  const value = record(parsed, "report");
  if (value.status !== "APPLIED_PREVIEW_READY"
    && value.status !== "APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY"
    && value.status !== "APPLIED_PREVIEW_OFFLINE_PASS"
    && value.status !== "DIAGNOSTIC_BLOCKED") {
    throw new Error("Invalid applied supermodel report at report.status");
  }
  const motionCompatible = boolean(value.motionCompatible, "report.motionCompatible");
  const motionQualityStatus = value.motionQualityStatus;
  if (motionQualityStatus !== "PASS" && motionQualityStatus !== "BLOCKED") {
    throw new Error("Invalid applied supermodel report at report.motionQualityStatus");
  }
  const runtimeReadiness = value.runtimeReadiness;
  if (runtimeReadiness !== "RUNTIME_UNPROVEN" && runtimeReadiness !== "DIAGNOSTIC_PREVIEW_ONLY_MOTION_QUALITY_BLOCKED") {
    throw new Error("Invalid applied supermodel report at report.runtimeReadiness");
  }
  const motionQuality = motionQualitySummary(value.motionQuality);
  const fullCarrierCoverage = boolean(value.fullCarrierCoverage, "report.fullCarrierCoverage");
  const requiredJointCoverage = boolean(value.requiredJointCoverage, "report.requiredJointCoverage");
  const skinInfluenceCoverage = boolean(value.skinInfluenceCoverage, "report.skinInfluenceCoverage");
  const inheritedClipCoverage = boolean(value.inheritedClipCoverage, "report.inheritedClipCoverage");
  const visibleMotionCoverage = boolean(value.visibleMotionCoverage, "report.visibleMotionCoverage");
  const seamViolationCount = integer(value.seamViolationCount, "report.seamViolationCount");
  const admission = admissionV3(value.admissionV3);
  const legacyReady = value.status === "APPLIED_PREVIEW_READY" && motionCompatible
    && fullCarrierCoverage && requiredJointCoverage && skinInfluenceCoverage
    && inheritedClipCoverage && visibleMotionCoverage && seamViolationCount === 0
    && motionQualityStatus === "PASS" && motionQuality.status === "PASS" && runtimeReadiness === "RUNTIME_UNPROVEN";
  const offlineReady = value.status === "APPLIED_PREVIEW_OFFLINE_PASS" && admission?.status === "PASS"
    && motionCompatible && motionQualityStatus === "PASS" && motionQuality.status === "PASS";
  const legacyDiagnostic = value.status === "APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY" && !motionCompatible
    && motionQualityStatus === "BLOCKED" && motionQuality.status === "BLOCKED"
    && runtimeReadiness === "DIAGNOSTIC_PREVIEW_ONLY_MOTION_QUALITY_BLOCKED";
  const diagnostic = value.status === "DIAGNOSTIC_BLOCKED" && admission?.status === "DIAGNOSTIC_BLOCKED";
  if (!legacyReady && !offlineReady && !legacyDiagnostic && !diagnostic) {
    throw new Error("Applied supermodel report has an inconsistent central admission verdict");
  }

  const selectedChain = exactChain(value.exactChain, "report.exactChain");
  const structural = record(value.structuralAnalysis, "report.structuralAnalysis");
  const structuralChain = exactChain(structural.exactChain, "report.structuralAnalysis.exactChain");
  const rig = record(value.rigAnalysis, "report.rigAnalysis");
  const supermodelResref = nonEmptyString(value.supermodelResref, "report.supermodelResref");
  const referenceSha256 = sha256(value.referenceSha256, "report.referenceSha256");
  if (selectedChain[0].resref.toLocaleLowerCase() !== supermodelResref.toLocaleLowerCase()
    || selectedChain[0].sha256 !== referenceSha256
    || JSON.stringify(selectedChain) !== JSON.stringify(structuralChain)) {
    throw new Error("Applied supermodel report exact-chain identity is inconsistent");
  }
  return {
    schemaVersion: exactLiteral(value.schemaVersion, 2, "report.schemaVersion"),
    status: value.status,
    supermodelResref,
    referenceFormat: format(value.referenceFormat, "report.referenceFormat"),
    referenceSha256,
    modelSha256: sha256(value.modelSha256, "report.modelSha256"),
    localAnimationCount: integer(value.localAnimationCount, "report.localAnimationCount"),
    inheritedAnimationCount: integer(value.inheritedAnimationCount, "report.inheritedAnimationCount"),
    requiredClipCount: integer(value.requiredClipCount, "report.requiredClipCount"),
    motionCompatible,
    bindPoseCompatible: boolean(value.bindPoseCompatible, "report.bindPoseCompatible"),
    skinBindCompatible: boolean(value.skinBindCompatible, "report.skinBindCompatible"),
    fullCarrierCoverage,
    requiredJointCoverage,
    skinInfluenceCoverage,
    inheritedClipCoverage,
    visibleMotionCoverage,
    seamViolationCount,
    motionQualityStatus,
    runtimeReadiness,
    exactChain: selectedChain,
    structuralAnalysis: {
      status: exactLiteral(structural.status, "REFERENCE_SUPERMODEL_STRUCTURALLY_READY", "report.structuralAnalysis.status"),
      selectedSupermodelResref: nonEmptyString(structural.selectedSupermodelResref, "report.structuralAnalysis.selectedSupermodelResref"),
      selectedFormat: format(structural.selectedFormat, "report.structuralAnalysis.selectedFormat"),
      selectedSha256: sha256(structural.selectedSha256, "report.structuralAnalysis.selectedSha256"),
      motionCarrierResref: nonEmptyString(structural.motionCarrierResref, "report.structuralAnalysis.motionCarrierResref"),
      carrierNodeCount: integer(structural.carrierNodeCount, "report.structuralAnalysis.carrierNodeCount"),
      carrierControllerCount: integer(structural.carrierControllerCount, "report.structuralAnalysis.carrierControllerCount"),
      inheritedAnimationNames: strings(structural.inheritedAnimationNames, "report.structuralAnalysis.inheritedAnimationNames"),
      structuralErrors: strings(structural.structuralErrors, "report.structuralAnalysis.structuralErrors"),
      exactChain: structuralChain,
      retailPayloadCopied: exactLiteral(structural.retailPayloadCopied, false, "report.structuralAnalysis.retailPayloadCopied"),
    },
      rigAnalysis: {
      algorithm: nonEmptyString(rig.algorithm, "report.rigAnalysis.algorithm"),
      carrierNodeCount: integer(rig.carrierNodeCount, "report.rigAnalysis.carrierNodeCount"),
      fullCarrierCoverage: boolean(rig.fullCarrierCoverage, "report.rigAnalysis.fullCarrierCoverage"),
      requiredJointCoverage: boolean(rig.requiredJointCoverage, "report.rigAnalysis.requiredJointCoverage"),
      allowedBoneCount: integer(rig.allowedBoneCount, "report.rigAnalysis.allowedBoneCount"),
      weightedBoneCount: integer(rig.weightedBoneCount, "report.rigAnalysis.weightedBoneCount"),
      activeWeightedBoneCount: integer(rig.activeWeightedBoneCount, "report.rigAnalysis.activeWeightedBoneCount"),
      unweightedRequiredJointNames: strings(rig.unweightedRequiredJointNames, "report.rigAnalysis.unweightedRequiredJointNames"),
      passiveUnweightedJointNames: strings(rig.passiveUnweightedJointNames, "report.rigAnalysis.passiveUnweightedJointNames"),
      skinInfluenceCoverage: boolean(rig.skinInfluenceCoverage, "report.rigAnalysis.skinInfluenceCoverage"),
      surfaceVertexCount: integer(rig.surfaceVertexCount, "report.rigAnalysis.surfaceVertexCount"),
      surfaceTriangleCount: integer(rig.surfaceTriangleCount, "report.rigAnalysis.surfaceTriangleCount"),
      duplicatePositionGroupCount: integer(rig.duplicatePositionGroupCount, "report.rigAnalysis.duplicatePositionGroupCount"),
      noReferencePayloadCopied: exactLiteral(rig.noReferencePayloadCopied, true, "report.rigAnalysis.noReferencePayloadCopied"),
      surfaceAnatomy: surfaceAnatomy(rig.surfaceAnatomy),
      jointFit: jointFit(rig.jointFit),
        skinning: skinning(rig.skinning),
      bindPose: bindPose(rig.bindPose),
      },
      experimentalAllowExcessiveSkinBranchRepair: value.experimentalAllowExcessiveSkinBranchRepair === undefined
        ? false
        : boolean(value.experimentalAllowExcessiveSkinBranchRepair, "report.experimentalAllowExcessiveSkinBranchRepair"),
    admissionV3: admission,
    retailPayloadCopied: exactLiteral(value.retailPayloadCopied, false, "report.retailPayloadCopied"),
    motionQuality,
  };
}

export function isAppliedPreviewCurrentV2(
  preview: AppliedSupermodelPreviewV2 | undefined,
  sourceSha256: string | undefined,
) {
  return Boolean(preview && sourceSha256 && preview.sourceSha256 === sourceSha256);
}
