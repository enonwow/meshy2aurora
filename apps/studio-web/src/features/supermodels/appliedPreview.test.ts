import { describe, expect, it } from "vitest";
import { parseAppliedSupermodelReportV2 } from "./appliedPreview";

describe("applied supermodel report V2", () => {
  const exactChain = [{
    resref: "c_serpent",
    supermodelResref: "c_motion",
    format: "ASCII",
    sha256: "a".repeat(64),
    byteLength: 1_024,
    localAnimationCount: 0,
    nodeCount: 9,
    controllerCount: 18,
  }, {
    resref: "c_motion",
    supermodelResref: "NULL",
    format: "BINARY",
    sha256: "c".repeat(64),
    byteLength: 4_096,
    localAnimationCount: 12,
    nodeCount: 11,
    controllerCount: 80,
  }] as const;
  const quality = (status: "PASS" | "BLOCKED") => ({
    status,
    edgeOutsideHardLimitCount: status === "PASS" ? 0 : 4,
    edgeOutsideHardAllowedCount: 0,
    triangleAreaCollapseCount: status === "PASS" ? 0 : 2,
    triangleAreaCollapseAllowedCount: 0,
    seamPairViolationCount: status === "PASS" ? 0 : 9,
    seamPairAllowedCount: 1,
    visibleAnchorMotionViolationCount: 0,
    requiredClipCount: 12,
    sampledClipCount: 12,
    inheritedClipCoverage: true,
    requiredJointCount: 10,
    jointClipRequiredCount: 24,
    jointClipPassCount: status === "PASS" ? 24 : 23,
    jointClipCoverage: status === "PASS",
    visibleMotionCoverage: status === "PASS",
    surfaceSeamGate: { status: status === "PASS" ? "PASS" : "BLOCKED_VISIBLE_SEAM" },
  });
  const report = (status: "PASS" | "BLOCKED") => ({
    schemaVersion: 2,
    status: status === "PASS" ? "APPLIED_PREVIEW_READY" : "APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY",
    supermodelResref: "c_serpent",
    referenceFormat: "ASCII",
    referenceSha256: "a".repeat(64),
    modelSha256: "b".repeat(64),
    localAnimationCount: 0,
    inheritedAnimationCount: 12,
    requiredClipCount: 12,
    motionCompatible: status === "PASS",
    bindPoseCompatible: true,
    skinBindCompatible: true,
    fullCarrierCoverage: true,
    requiredJointCoverage: true,
    skinInfluenceCoverage: true,
    inheritedClipCoverage: true,
    visibleMotionCoverage: status === "PASS",
    seamViolationCount: status === "PASS" ? 0 : 9,
    motionQualityStatus: status,
    runtimeReadiness: status === "PASS" ? "RUNTIME_UNPROVEN" : "DIAGNOSTIC_PREVIEW_ONLY_MOTION_QUALITY_BLOCKED",
    exactChain,
    structuralAnalysis: {
      status: "REFERENCE_SUPERMODEL_STRUCTURALLY_READY",
      selectedSupermodelResref: "c_serpent",
      selectedFormat: "ASCII",
      selectedSha256: "a".repeat(64),
      motionCarrierResref: "c_motion",
      carrierNodeCount: 11,
      carrierControllerCount: 80,
      inheritedAnimationNames: ["cpause1", "cwalk", "crun"],
      structuralErrors: [],
      exactChain,
      retailPayloadCopied: false,
    },
    rigAnalysis: {
      algorithm: "HIERARCHY_CHAIN_SEGMENT_DISTANCE_TOPOLOGY_SMOOTH_V7",
      carrierNodeCount: 11,
      fullCarrierCoverage: true,
      requiredJointCoverage: true,
      allowedBoneCount: 10,
      weightedBoneCount: 10,
      activeWeightedBoneCount: 10,
      unweightedRequiredJointNames: [],
      passiveUnweightedJointNames: ["root_end"],
      skinInfluenceCoverage: true,
      surfaceVertexCount: 128,
      surfaceTriangleCount: 64,
      duplicatePositionGroupCount: 3,
      noReferencePayloadCopied: true,
    },
    retailPayloadCopied: false,
    motionQuality: quality(status),
  });

  it("accepts an ASCII-selected, binary-parent motion-compatible chain", () => {
    expect(parseAppliedSupermodelReportV2(JSON.stringify(report("PASS")))).toMatchObject({
      supermodelResref: "c_serpent",
      referenceFormat: "ASCII",
      inheritedAnimationCount: 12,
      motionCompatible: true,
      structuralAnalysis: { motionCarrierResref: "c_motion" },
    });
  });

  it("accepts a diagnostic preview while preserving the blocked quality verdict", () => {
    expect(parseAppliedSupermodelReportV2(JSON.stringify(report("BLOCKED")))).toMatchObject({
      status: "APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY",
      motionCompatible: false,
      motionQualityStatus: "BLOCKED",
    });
  });

  it("rejects a contradictory ready verdict", () => {
    expect(() => parseAppliedSupermodelReportV2(JSON.stringify({
      ...report("BLOCKED"),
      status: "APPLIED_PREVIEW_READY",
    }))).toThrow(/inconsistent/i);
  });

  it("accepts only a stage-consistent central admission for a new offline preview", () => {
    const current = {
      ...report("PASS"),
      status: "APPLIED_PREVIEW_OFFLINE_PASS",
      admissionV3: {
        schemaVersion: 3,
        mode: "DIAGNOSTIC",
        status: "PASS",
        exportAllowed: false,
        firstBlockingStage: null,
        blockingCodes: [],
        stages: ["structure", "surfaceAnatomy", "jointFit", "skinning", "bindPose", "motionQuality", "export"].map((stage) => ({
          stage,
          sourceStatus: "PASS",
          status: "PASS",
          blockingCodes: [],
        })),
      },
    };
    expect(parseAppliedSupermodelReportV2(JSON.stringify(current)).admissionV3).toMatchObject({
      status: "PASS",
      exportAllowed: false,
    });
    expect(() => parseAppliedSupermodelReportV2(JSON.stringify({
      ...current,
      admissionV3: {
        ...current.admissionV3,
        stages: current.admissionV3.stages.map((stage) => stage.stage === "jointFit" ? { ...stage, status: "BLOCKED" } : stage),
      },
    }))).toThrow(/inconsistent stage verdict/i);
  });
});
