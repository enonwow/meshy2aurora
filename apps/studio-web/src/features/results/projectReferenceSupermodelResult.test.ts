import { describe, expect, it } from "vitest";
import type { WorkerArtifact } from "../../worker/types";
import { projectReferenceSupermodelResultV1 } from "./projectReferenceSupermodelResult";

const hash = (character: string) => character.repeat(64);

function fixture() {
  const resources = [
    { role: "MODEL", resref: "cmreference", type: 2002, byteLength: 3, sha256: hash("a") },
    { role: "TEXTURE", resref: "ctreference", type: 3, byteLength: 3, sha256: hash("b") },
    { role: "TEXTURE", resref: "crreference", type: 2035, byteLength: 3, sha256: hash("c") },
    { role: "APPEARANCE_TABLE", resref: "appearance", type: 2017, byteLength: 3, sha256: hash("d") },
  ];
  const report = {
    schemaVersion: 2,
    status: "REFERENCE_SUPERMODEL_CREATURE_PRODUCT_MATERIALIZED",
    selectedSupermodelResref: "c_stag",
    selectedFormat: "ASCII",
    identity: {
      modelResref: "cmreference",
      textureResref: "ctreference",
      materialResref: "crreference",
      hakResref: "chreference",
      appearanceLabel: "M2A_REFERENCE",
      appearanceDonorResrefs: ["c_stag"],
      semanticControllerNames: [],
    },
    source: {
      inventory: { nodeCount: 1, meshCount: 1, animationCount: 0 },
      statistics: { vertexCount: 3, triangleCount: 1 },
    },
    model: {
      payloadSha256: hash("a"),
      projection: { rigNodeCount: 2, meshNodeCount: 1, triangleCount: 1 },
      semanticDiff: [],
      deviations: [],
    },
    texture: { width: 1, height: 1, pixelFormat: "RGB8", outputSha256: hash("b") },
    appearance: { appendedRowIndex: 848, sourcePrefixPreserved: true, outputSha256: hash("d") },
    hak: { entryCount: 4, archiveSha256: hash("e") },
    packageManifest: { packageSha256: hash("e"), resources },
    motionBuild: {
      motionCompatible: true,
      supermodelResref: "c_stag",
      skinInfluenceCoverage: true,
      carrierCoverage: { fullCarrierCoverage: true, requiredJointCoverage: true },
    },
    motionQuality: {
      status: "PASS",
      inheritedClipCoverage: true,
      jointClipCoverage: true,
      visibleMotionCoverage: true,
      seamPairViolationCount: 0,
      requiredClipCount: 3,
      sampledClipCount: 3,
      jointClipRequiredCount: 12,
      jointClipPassCount: 12,
    },
    semanticDelta: null,
    exactChain: [{ resref: "c_stag", supermodelResref: "NULL", format: "ASCII", sha256: hash("f"), byteLength: 123 }],
    structuralAnalysis: {
      status: "REFERENCE_SUPERMODEL_STRUCTURALLY_READY",
      inheritedAnimationNames: ["cpause1", "cwalk", "crun"],
    },
    rigAnalysis: {
      carrierNodeCount: 11,
      fullCarrierCoverage: true,
      requiredJointCoverage: true,
      skinInfluenceCoverage: true,
      allowedBoneCount: 10,
      activeWeightedBoneCount: 10,
      passiveUnweightedJointNames: ["headconjure"],
    },
    exportAdmission: {
      status: "REFERENCE_SUPERMODEL_EXPORT_ADMITTED",
      fullCarrierCoverage: true,
      requiredJointCoverage: true,
      skinInfluenceCoverage: true,
      inheritedClipCoverage: true,
      visibleMotionCoverage: true,
      seamViolationCount: 0,
    },
    retailPayloadCopied: false,
    ownerRuntimeProof: "NOT_RUN_HUMAN_OWNED",
  };
  const summary = { schemaVersion: 1, status: report.status };
  const manifest = { schemaVersion: 1, status: report.status, packageManifest: report.packageManifest };
  const reportJson = JSON.stringify(report);
  const bytes = (id: string) => id === "report-json"
    ? new TextEncoder().encode(reportJson).buffer
    : new Uint8Array([1, 2, 3]).buffer;
  const artifact = (
    artifactId: string,
    kind: WorkerArtifact["kind"],
    sha256: string,
    fileName: string,
  ): WorkerArtifact => ({
    artifactId,
    kind,
    fileName,
    mediaType: "application/octet-stream",
    provenance: "M2A_WASM_WORKER",
    bytes: bytes(artifactId),
    byteLength: bytes(artifactId).byteLength,
    sha256,
  });
  const artifacts = [
    artifact("package-hak", "HAK", hash("e"), "chreference.hak"),
    artifact("model-mdl", "MODEL", hash("a"), "cmreference.mdl"),
    artifact("texture-tga", "TEXTURE", hash("b"), "ctreference.tga"),
    artifact("material-mtr", "MATERIAL", hash("c"), "crreference.mtr"),
    artifact("appearance-2da", "TWO_DA", hash("d"), "appearance.2da"),
    artifact("report-json", "JSON_REPORT", hash("1"), "inspection.json"),
    artifact("manifest-json", "JSON_REPORT", hash("2"), "conversion-manifest.json"),
    artifact("summary-json", "JSON_REPORT", hash("3"), "summary.json"),
  ];
  return { report, reportJson, summaryJson: JSON.stringify(summary), manifestJson: JSON.stringify(manifest), artifacts };
}

describe("reference-supermodel canonical result", () => {
  it("projects the admitted MDL/MTR/2DA/HAK product into normal Review", () => {
    const value = fixture();
    const result = projectReferenceSupermodelResultV1(
      value.reportJson,
      value.summaryJson,
      value.manifestJson,
      value.artifacts,
    );
    expect(result.status).toBe("REFERENCE_SUPERMODEL_CREATURE_PRODUCT_MATERIALIZED");
    expect(result.packageAssemblyEvidence).toEqual({ strictReconciled: true, resourceCount: 4, artifactCount: 8 });
    expect(result.artifacts.some((artifact) => artifact.kind === "TWO_DA")).toBe(true);
    expect(result.runtimeAcceptance.status).toBe("OPEN_M6");
    expect(result.referenceSupermodelCoverage).toMatchObject({
      activeWeightedBoneCount: 10,
      jointClipPassCount: 12,
      seamViolationCount: 0,
    });
  });

  it("rejects a diagnostic motion verdict before Review", () => {
    const value = fixture();
    value.report.motionBuild.motionCompatible = false;
    expect(() => projectReferenceSupermodelResultV1(
      JSON.stringify(value.report),
      value.summaryJson,
      value.manifestJson,
      value.artifacts,
    )).toThrow(/exportAdmission/);
  });

  it("rejects any missing full-skeleton admission gate before Review", () => {
    const value = fixture();
    value.report.exportAdmission.skinInfluenceCoverage = false;
    expect(() => projectReferenceSupermodelResultV1(
      JSON.stringify(value.report),
      value.summaryJson,
      value.manifestJson,
      value.artifacts,
    )).toThrow(/exportAdmission/);
  });
});
