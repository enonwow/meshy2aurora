import type { WorkerArtifact } from "../../worker/types";
import type { CanonicalResultSnapshot } from "./projectCanonicalResult";

type JsonRecord = Record<string, unknown>;

function fail(path: string): never {
  throw new Error(`Reference-supermodel result identity mismatch at ${path}`);
}
function record(value: unknown, path: string): JsonRecord {
  if (!value || typeof value !== "object" || Array.isArray(value)) fail(path);
  return value as JsonRecord;
}
function string(value: unknown, path: string) {
  if (typeof value !== "string" || !value) fail(path);
  return value;
}
function integer(value: unknown, path: string) {
  if (!Number.isSafeInteger(value) || (value as number) < 0) fail(path);
  return value as number;
}
function boolean(value: unknown, path: string) {
  if (typeof value !== "boolean") fail(path);
  return value;
}
function sha256(value: unknown, path: string) {
  const text = string(value, path);
  if (!/^[0-9a-f]{64}$/.test(text)) fail(path);
  return text;
}
function parse(json: string, path: string) {
  try { return record(JSON.parse(json), path); } catch { return fail(path); }
}

export function projectReferenceSupermodelResultV1(
  reportJson: string,
  summaryJson: string,
  manifestJson: string,
  artifacts: readonly WorkerArtifact[],
): CanonicalResultSnapshot {
  const report = parse(reportJson, "reportJson");
  const summary = parse(summaryJson, "summaryJson");
  const manifest = parse(manifestJson, "manifestJson");
  const status = "REFERENCE_SUPERMODEL_CREATURE_PRODUCT_MATERIALIZED";
  if (report.status !== status || summary.status !== status || manifest.status !== status) {
    fail("status");
  }
  const exportAdmission = record(report.exportAdmission, "report.exportAdmission");
  const motionBuild = record(report.motionBuild, "report.motionBuild");
  const motionQuality = record(report.motionQuality, "report.motionQuality");
  const rigAnalysis = record(report.rigAnalysis, "report.rigAnalysis");
  if (
    exportAdmission.status !== "REFERENCE_SUPERMODEL_EXPORT_ADMITTED"
    || exportAdmission.fullCarrierCoverage !== true
    || exportAdmission.requiredJointCoverage !== true
    || exportAdmission.skinInfluenceCoverage !== true
    || exportAdmission.inheritedClipCoverage !== true
    || exportAdmission.visibleMotionCoverage !== true
    || exportAdmission.seamViolationCount !== 0
    || motionBuild.motionCompatible !== true
    || record(motionBuild.carrierCoverage, "report.motionBuild.carrierCoverage").fullCarrierCoverage !== true
    || record(motionBuild.carrierCoverage, "report.motionBuild.carrierCoverage").requiredJointCoverage !== true
    || motionBuild.skinInfluenceCoverage !== true
    || motionQuality.status !== "PASS"
    || motionQuality.inheritedClipCoverage !== true
    || motionQuality.jointClipCoverage !== true
    || motionQuality.visibleMotionCoverage !== true
    || motionQuality.seamPairViolationCount !== 0
    || rigAnalysis.fullCarrierCoverage !== true
    || rigAnalysis.requiredJointCoverage !== true
    || rigAnalysis.skinInfluenceCoverage !== true
    || report.retailPayloadCopied !== false
    || report.ownerRuntimeProof !== "NOT_RUN_HUMAN_OWNED"
  ) fail("exportAdmission");
  if (report.semanticDelta !== null && report.semanticDelta !== undefined) {
    const semanticDelta = record(report.semanticDelta, "report.semanticDelta");
    if (semanticDelta.exportDeltaProven !== true) fail("report.semanticDelta.exportDeltaProven");
  }
  const exactChain = report.exactChain;
  if (!Array.isArray(exactChain) || exactChain.length === 0) fail("report.exactChain");
  for (const [index, item] of exactChain.entries()) {
    const resource = record(item, `report.exactChain[${index}]`);
    string(resource.resref, `report.exactChain[${index}].resref`);
    sha256(resource.sha256, `report.exactChain[${index}].sha256`);
    integer(resource.byteLength, `report.exactChain[${index}].byteLength`);
  }
  const structuralAnalysis = record(report.structuralAnalysis, "report.structuralAnalysis");
  const inheritedAnimationNames = structuralAnalysis.inheritedAnimationNames;
  if (!Array.isArray(inheritedAnimationNames)) fail("report.structuralAnalysis.inheritedAnimationNames");
  inheritedAnimationNames.forEach((value, index) => string(value, `report.structuralAnalysis.inheritedAnimationNames[${index}]`));
  const identity = record(report.identity, "report.identity");
  const modelResref = string(identity.modelResref, "report.identity.modelResref");
  const textureResref = string(identity.textureResref, "report.identity.textureResref");
  const source = record(report.source, "report.source");
  const sourceInventory = record(source.inventory, "report.source.inventory");
  const sourceStatistics = record(source.statistics, "report.source.statistics");
  const model = record(report.model, "report.model");
  const projection = record(model.projection, "report.model.projection");
  const texture = record(report.texture, "report.texture");
  const appearance = record(report.appearance, "report.appearance");
  const hak = record(report.hak, "report.hak");
  const packageManifest = record(report.packageManifest, "report.packageManifest");
  const manifestPackage = record(manifest.packageManifest, "manifest.packageManifest");
  if (manifestPackage.packageSha256 !== packageManifest.packageSha256) fail("manifest.packageManifest");
  const resourcesJson = packageManifest.resources;
  if (!Array.isArray(resourcesJson) || resourcesJson.length !== 4) fail("report.packageManifest.resources");
  const resources = resourcesJson.map((value, index) => {
    const item = record(value, `report.packageManifest.resources[${index}]`);
    return {
      role: string(item.role, `report.packageManifest.resources[${index}].role`),
      resref: string(item.resref, `report.packageManifest.resources[${index}].resref`),
      type: integer(item.type, `report.packageManifest.resources[${index}].type`),
      byteLength: integer(item.byteLength, `report.packageManifest.resources[${index}].byteLength`),
      sha256: sha256(item.sha256, `report.packageManifest.resources[${index}].sha256`),
    };
  });
  if (new Set(artifacts.map((item) => item.artifactId)).size !== 8 || artifacts.length !== 8) {
    fail("artifacts");
  }
  const artifact = (id: string, kind: WorkerArtifact["kind"]) => {
    const item = artifacts.find((candidate) => candidate.artifactId === id) ?? fail(`artifacts.${id}`);
    if (item.kind !== kind || item.byteLength !== item.bytes.byteLength || item.provenance !== "M2A_WASM_WORKER") {
      fail(`artifacts.${id}`);
    }
    return item;
  };
  const modelArtifact = artifact("model-mdl", "MODEL");
  const textureArtifact = artifact("texture-tga", "TEXTURE");
  const materialArtifact = artifact("material-mtr", "MATERIAL");
  const appearanceArtifact = artifact("appearance-2da", "TWO_DA");
  const hakArtifact = artifact("package-hak", "HAK");
  const reportArtifact = artifact("report-json", "JSON_REPORT");
  artifact("manifest-json", "JSON_REPORT");
  artifact("summary-json", "JSON_REPORT");
  if (new TextDecoder().decode(reportArtifact.bytes) !== reportJson) fail("artifacts.report-json.bytes");
  if (modelArtifact.sha256 !== model.payloadSha256) fail("artifacts.model-mdl.sha256");
  if (textureArtifact.sha256 !== texture.outputSha256) fail("artifacts.texture-tga.sha256");
  if (appearanceArtifact.sha256 !== appearance.outputSha256) fail("artifacts.appearance-2da.sha256");
  if (hakArtifact.sha256 !== hak.archiveSha256 || hakArtifact.sha256 !== packageManifest.packageSha256) {
    fail("artifacts.package-hak.sha256");
  }
  const outputs = {
    model: { byteLength: modelArtifact.byteLength, sha256: modelArtifact.sha256 },
    texture: { byteLength: textureArtifact.byteLength, sha256: textureArtifact.sha256 },
    material: { byteLength: materialArtifact.byteLength, sha256: materialArtifact.sha256 },
    appearanceTwoDa: { byteLength: appearanceArtifact.byteLength, sha256: appearanceArtifact.sha256 },
    hak: { byteLength: hakArtifact.byteLength, sha256: hakArtifact.sha256 },
    report: { byteLength: reportArtifact.byteLength, sha256: reportArtifact.sha256 },
  };
  const deviations = Array.isArray(model.deviations) ? model.deviations.map((value, index) => {
    const item = record(value, `report.model.deviations[${index}]`);
    return {
      code: string(item.code, `report.model.deviations[${index}].code`),
      path: string(item.path, `report.model.deviations[${index}].path`),
      message: string(item.message, `report.model.deviations[${index}].message`),
    };
  }) : [];
  return {
    status,
    sourceMetrics: {
      nodes: integer(sourceInventory.nodeCount, "report.source.inventory.nodeCount"),
      meshes: integer(sourceInventory.meshCount, "report.source.inventory.meshCount"),
      vertices: integer(sourceStatistics.vertexCount, "report.source.statistics.vertexCount"),
      triangles: integer(sourceStatistics.triangleCount, "report.source.statistics.triangleCount"),
      animations: integer(sourceInventory.animationCount, "report.source.inventory.animationCount"),
    },
    convertedMetrics: {
      nodes: integer(projection.rigNodeCount, "report.model.projection.rigNodeCount")
        + integer(projection.meshNodeCount, "report.model.projection.meshNodeCount"),
      meshes: integer(projection.meshNodeCount, "report.model.projection.meshNodeCount"),
      vertices: integer(sourceStatistics.vertexCount, "report.source.statistics.vertexCount"),
      triangles: integer(projection.triangleCount, "report.model.projection.triangleCount"),
      animations: integer(inheritedAnimationNames.length, "report.structuralAnalysis.inheritedAnimationNames.length"),
    },
    geometry: {
      vertices: integer(sourceStatistics.vertexCount, "report.source.statistics.vertexCount"),
      triangles: integer(projection.triangleCount, "report.model.projection.triangleCount"),
      joints: integer(projection.rigNodeCount, "report.model.projection.rigNodeCount"),
      deformation: "SKINNED_REFERENCE_SUPERMODEL",
    },
    animation: {
      sourceName: string(motionBuild.supermodelResref, "report.motionBuild.supermodelResref"),
      outputName: "inherited",
      durationSeconds: 0,
      hasMotion: true,
    },
    texture: {
      width: integer(texture.width, "report.texture.width"),
      height: integer(texture.height, "report.texture.height"),
      pixelFormat: string(texture.pixelFormat, "report.texture.pixelFormat"),
      byteLength: textureArtifact.byteLength,
    },
    resrefs: { model: modelResref, texture: textureResref },
    appearance: {
      appendedRow: integer(appearance.appendedRowIndex, "report.appearance.appendedRowIndex"),
      sourcePrefixPreserved: boolean(appearance.sourcePrefixPreserved, "report.appearance.sourcePrefixPreserved"),
      policy: "CATALOG_DONOR_CLONE_EXACT_PREFIX_V2",
    },
    hak: {
      byteLength: hakArtifact.byteLength,
      sha256: hakArtifact.sha256,
      entryCount: integer(hak.entryCount, "report.hak.entryCount"),
    },
    outputs,
    resources,
    semanticEvidence: {
      semanticDiff: Array.isArray(model.semanticDiff) ? model.semanticDiff.map((value, index) => string(value, `report.model.semanticDiff[${index}]`)) : [],
      deviations,
    },
    conversionEvidence: {
      schemaVersion: 1,
      conversionEligible: true,
      policies: {
        basisStatus: "CANONICALIZED",
        assetForwardMapping: "PROFILE_BOUND",
        orientationParity: "REFERENCE_ADAPTER",
        engineFacingProof: "OFFLINE_ONLY",
        uvRuntimeProof: "OFFLINE_READBACK",
      },
      gates: [],
      diagnostics: [],
    },
    referenceSupermodelCoverage: {
      fullCarrierCoverage: true,
      requiredJointCoverage: true,
      skinInfluenceCoverage: true,
      inheritedClipCoverage: true,
      visibleMotionCoverage: true,
      carrierNodeCount: integer(rigAnalysis.carrierNodeCount, "report.rigAnalysis.carrierNodeCount"),
      allowedBoneCount: integer(rigAnalysis.allowedBoneCount, "report.rigAnalysis.allowedBoneCount"),
      activeWeightedBoneCount: integer(rigAnalysis.activeWeightedBoneCount, "report.rigAnalysis.activeWeightedBoneCount"),
      passiveUnweightedJointNames: Array.isArray(rigAnalysis.passiveUnweightedJointNames)
        ? rigAnalysis.passiveUnweightedJointNames.map((value, index) => string(value, `report.rigAnalysis.passiveUnweightedJointNames[${index}]`))
        : fail("report.rigAnalysis.passiveUnweightedJointNames"),
      requiredClipCount: integer(motionQuality.requiredClipCount, "report.motionQuality.requiredClipCount"),
      sampledClipCount: integer(motionQuality.sampledClipCount, "report.motionQuality.sampledClipCount"),
      jointClipRequiredCount: integer(motionQuality.jointClipRequiredCount, "report.motionQuality.jointClipRequiredCount"),
      jointClipPassCount: integer(motionQuality.jointClipPassCount, "report.motionQuality.jointClipPassCount"),
      seamViolationCount: 0,
    },
    packageAssemblyEvidence: { strictReconciled: true, resourceCount: 4, artifactCount: 8 },
    runtimeAcceptance: {
      status: "OPEN_M6",
      reason: "Human-owned Aurora/NWN proof was not run for these exact output hashes.",
    },
    artifacts: [...artifacts],
    reportJson,
    summaryJson,
    manifestJson,
  };
}
