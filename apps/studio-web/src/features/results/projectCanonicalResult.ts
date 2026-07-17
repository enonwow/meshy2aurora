import type { WorkerArtifact } from "../../worker/types";

export interface CanonicalResultSnapshot {
  status: string;
  sourceMetrics: CanonicalModelMetrics;
  convertedMetrics: CanonicalModelMetrics;
  geometry: { vertices: number; triangles: number; joints: number; deformation: string };
  animation: { sourceName: string; outputName: string; durationSeconds: number; hasMotion: boolean };
  texture: { width: number; height: number; pixelFormat: string; byteLength: number };
  resrefs: { model: string; texture: string };
  appearance: { appendedRow: number; sourcePrefixPreserved: boolean; policy: string };
  hak: { byteLength: number; sha256: string; entryCount: number };
  outputs: Record<string, { byteLength: number; sha256: string }>;
  resources: Array<{ role: string; resref: string; type: number; byteLength: number; sha256: string }>;
  semanticEvidence: {
    semanticDiff: string[];
    deviations: Array<{ code: string; path: string; message: string }>;
  };
  conversionEvidence: CanonicalConversionEvidence;
  packageAssemblyEvidence: {
    strictReconciled: true;
    resourceCount: number;
    artifactCount: number;
  };
  artifacts: WorkerArtifact[];
  reportJson: string;
  summaryJson: string;
  manifestJson: string;
}

export interface CanonicalConversionGate {
  schemaVersion: 1;
  code: string;
  severity: string;
  path: string;
  expected: string;
  actual: string;
  message: string;
}

export interface CanonicalConversionDiagnostic {
  schemaVersion: 1;
  code: string;
  severity: string;
  path: string;
  message: string;
}

export interface CanonicalConversionEvidence {
  schemaVersion: 1;
  conversionEligible: boolean;
  policies: {
    engineFacingProof: string;
    uvRuntimeProof: string;
  };
  gates: CanonicalConversionGate[];
  diagnostics: CanonicalConversionDiagnostic[];
}

export interface CanonicalModelMetrics {
  nodes: number;
  meshes: number;
  vertices: number;
  triangles: number;
  animations: number;
}

type JsonRecord = Record<string, unknown>;

const fail = (path: string): never => { throw new Error(`Canonical result field ${path} is missing or has the wrong type`); };
const record = (value: unknown, path: string): JsonRecord => value !== null && typeof value === "object" && !Array.isArray(value) ? value as JsonRecord : fail(path);
const array = (value: unknown, path: string): unknown[] => Array.isArray(value) ? value : fail(path);
const string = (value: unknown, path: string): string => typeof value === "string" && value.length > 0 ? value : fail(path);
const boolean = (value: unknown, path: string): boolean => typeof value === "boolean" ? value : fail(path);
const number = (value: unknown, path: string): number => typeof value === "number" && Number.isFinite(value) ? value : fail(path);
const integer = (value: unknown, path: string): number => Number.isSafeInteger(value) && (value as number) >= 0 ? value as number : fail(path);
const stringArray = (value: unknown, path: string): string[] => array(value, path).map((entry, index) => string(entry, `${path}[${index}]`));
const sha256 = (value: unknown, path: string): string => {
  const result = string(value, path);
  return /^[0-9a-f]{64}$/.test(result) ? result : fail(path);
};

function identity(value: unknown, path: string) {
  const item = record(value, path);
  return { byteLength: integer(item.byteLength, `${path}.byteLength`), sha256: sha256(item.sha256, `${path}.sha256`) };
}

function parseJson(json: string, path: string) {
  try { return record(JSON.parse(json), path); } catch { return fail(path); }
}

function equal(actual: unknown, expected: unknown, path: string) {
  if (actual !== expected) throw new Error(`Canonical result identity mismatch at ${path}`);
}

function conversionGate(value: unknown, path: string): CanonicalConversionGate {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  return {
    schemaVersion: 1,
    code: string(item.code, `${path}.code`),
    severity: string(item.severity, `${path}.severity`),
    path: string(item.path, `${path}.path`),
    expected: string(item.expected, `${path}.expected`),
    actual: string(item.actual, `${path}.actual`),
    message: string(item.message, `${path}.message`),
  };
}

function conversionDiagnostic(value: unknown, path: string): CanonicalConversionDiagnostic {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  return {
    schemaVersion: 1,
    code: string(item.code, `${path}.code`),
    severity: string(item.severity, `${path}.severity`),
    path: string(item.path, `${path}.path`),
    message: string(item.message, `${path}.message`),
  };
}

export function projectCanonicalResult(
  reportJson: string,
  summaryJson: string,
  manifestJson: string,
  artifacts: readonly WorkerArtifact[],
): CanonicalResultSnapshot {
  const report = parseJson(reportJson, "reportJson");
  const summary = parseJson(summaryJson, "summaryJson");
  const manifest = parseJson(manifestJson, "manifestJson");
  for (const [value, path] of [[report, "report"], [summary, "summary"], [manifest, "manifest"]] as const) {
    if (integer(value.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  }
  const status = string(summary.status, "summary.status");
  if (status !== "M6_MODEL_PACKAGE_MATERIALIZED") fail("summary.status");
  equal(string(manifest.status, "manifest.status"), status, "manifest.status");

  const ingest = record(report.ingest, "report.ingest");
  if (integer(ingest.schemaVersion, "report.ingest.schemaVersion") !== 1) fail("report.ingest.schemaVersion");
  const sourceInventory = record(ingest.inventory, "report.ingest.inventory");
  const sourceStatistics = record(ingest.statistics, "report.ingest.statistics");
  const sourceMetrics: CanonicalModelMetrics = {
    nodes: integer(sourceInventory.nodeCount, "report.ingest.inventory.nodeCount"),
    meshes: integer(sourceInventory.meshCount, "report.ingest.inventory.meshCount"),
    vertices: integer(sourceStatistics.vertexCount, "report.ingest.statistics.vertexCount"),
    triangles: integer(sourceStatistics.triangleCount, "report.ingest.statistics.triangleCount"),
    animations: integer(sourceInventory.animationCount, "report.ingest.inventory.animationCount"),
  };

  const conversion = record(report.conversion, "report.conversion");
  if (integer(conversion.schemaVersion, "report.conversion.schemaVersion") !== 1) fail("report.conversion.schemaVersion");
  const conversionPolicies = record(conversion.policies, "report.conversion.policies");
  const conversionEvidence: CanonicalConversionEvidence = {
    schemaVersion: 1,
    conversionEligible: boolean(conversion.conversionEligible, "report.conversion.conversionEligible"),
    policies: {
      engineFacingProof: string(conversionPolicies.engineFacingProof, "report.conversion.policies.engineFacingProof"),
      uvRuntimeProof: string(conversionPolicies.uvRuntimeProof, "report.conversion.policies.uvRuntimeProof"),
    },
    gates: array(conversion.gates, "report.conversion.gates").map((value, index) =>
      conversionGate(value, `report.conversion.gates[${index}]`)),
    diagnostics: array(conversion.diagnostics, "report.conversion.diagnostics").map((value, index) =>
      conversionDiagnostic(value, `report.conversion.diagnostics[${index}]`)),
  };

  const geometryJson = record(report.geometry, "report.geometry");
  const geometry = {
    vertices: integer(geometryJson.vertexCount, "report.geometry.vertexCount"),
    triangles: integer(geometryJson.triangleCount, "report.geometry.triangleCount"),
    joints: integer(geometryJson.activeJointCount, "report.geometry.activeJointCount"),
    deformation: string(geometryJson.outputSegmentDeformation, "report.geometry.outputSegmentDeformation"),
  };
  const animationJson = record(summary.animation, "summary.animation");
  const animation = {
    sourceName: string(animationJson.sourceName, "summary.animation.sourceName"),
    outputName: string(animationJson.outputName, "summary.animation.outputName"),
    durationSeconds: number(animationJson.durationSeconds, "summary.animation.durationSeconds"),
    hasMotion: boolean(animationJson.hasMotion, "summary.animation.hasMotion"),
  };
  const textureJson = record(report.texture, "report.texture");
  const texture = {
    width: integer(textureJson.width, "report.texture.width"),
    height: integer(textureJson.height, "report.texture.height"),
    pixelFormat: string(textureJson.pixelFormat, "report.texture.pixelFormat"),
    byteLength: integer(textureJson.byteLength, "report.texture.byteLength"),
  };
  const appearanceJson = record(report.appearance, "report.appearance");
  const appendedRow = integer(appearanceJson.appendedRowIndex, "report.appearance.appendedRowIndex");
  equal(integer(summary.appendedPhysicalRow, "summary.appendedPhysicalRow"), appendedRow, "summary.appendedPhysicalRow");
  equal(integer(manifest.appendedPhysicalRow, "manifest.appendedPhysicalRow"), appendedRow, "manifest.appendedPhysicalRow");
  const policy = string(summary.appearancePayloadPolicy, "summary.appearancePayloadPolicy");
  equal(string(manifest.appearancePayloadPolicy, "manifest.appearancePayloadPolicy"), policy, "manifest.appearancePayloadPolicy");

  const outputJson = record(summary.outputs, "summary.outputs");
  const outputs = Object.fromEntries(["model", "texture", "appearanceTwoDa", "hak", "proofModule", "report"].map((name) => [name, identity(outputJson[name], `summary.outputs.${name}`)]));
  const hakJson = record(report.hak, "report.hak");
  const hak = {
    byteLength: integer(hakJson.byteLength, "report.hak.byteLength"),
    sha256: sha256(hakJson.archiveSha256, "report.hak.archiveSha256"),
    entryCount: integer(hakJson.entryCount, "report.hak.entryCount"),
  };
  equal(outputs.hak.byteLength, hak.byteLength, "summary.outputs.hak.byteLength");
  equal(outputs.hak.sha256, hak.sha256, "summary.outputs.hak.sha256");
  equal(texture.byteLength, outputs.texture.byteLength, "report.texture.byteLength");
  equal(sha256(textureJson.outputSha256, "report.texture.outputSha256"), outputs.texture.sha256, "report.texture.outputSha256");
  const modelJson = record(report.model, "report.model");
  const projection = record(modelJson.projection, "report.model.projection");
  const convertedMetrics: CanonicalModelMetrics = {
    nodes: integer(projection.rigNodeCount, "report.model.projection.rigNodeCount")
      + integer(projection.meshNodeCount, "report.model.projection.meshNodeCount"),
    meshes: integer(projection.meshNodeCount, "report.model.projection.meshNodeCount"),
    vertices: geometry.vertices,
    triangles: integer(projection.triangleCount, "report.model.projection.triangleCount"),
    animations: integer(projection.animationCount, "report.model.projection.animationCount"),
  };
  equal(convertedMetrics.triangles, geometry.triangles, "report.model.projection.triangleCount");
  const semanticDiff = stringArray(modelJson.semanticDiff, "report.model.semanticDiff");
  const deviations = array(modelJson.deviations, "report.model.deviations").map((value, index) => {
    const item = record(value, `report.model.deviations[${index}]`);
    return {
      code: string(item.code, `report.model.deviations[${index}].code`),
      path: string(item.path, `report.model.deviations[${index}].path`),
      message: string(item.message, `report.model.deviations[${index}].message`),
    };
  });
  equal(sha256(modelJson.payloadSha256, "report.model.payloadSha256"), outputs.model.sha256, "report.model.payloadSha256");
  equal(integer(record(modelJson.layout, "report.model.layout").fileLength, "report.model.layout.fileLength"), outputs.model.byteLength, "report.model.layout.fileLength");
  equal(integer(appearanceJson.outputByteLength, "report.appearance.outputByteLength"), outputs.appearanceTwoDa.byteLength, "report.appearance.outputByteLength");
  equal(sha256(appearanceJson.outputSha256, "report.appearance.outputSha256"), outputs.appearanceTwoDa.sha256, "report.appearance.outputSha256");
  const proofModuleJson = record(report.proofModule, "report.proofModule");
  equal(integer(proofModuleJson.byteLength, "report.proofModule.byteLength"), outputs.proofModule.byteLength, "report.proofModule.byteLength");
  equal(sha256(proofModuleJson.sha256, "report.proofModule.sha256"), outputs.proofModule.sha256, "report.proofModule.sha256");
  equal(integer(proofModuleJson.appearanceRow, "report.proofModule.appearanceRow"), appendedRow, "report.proofModule.appearanceRow");
  if (string(proofModuleJson.semanticReadbackStatus, "report.proofModule.semanticReadbackStatus") !== "PASS") fail("report.proofModule.semanticReadbackStatus");

  const packageManifest = record(manifest.packageManifest, "manifest.packageManifest");
  equal(sha256(packageManifest.packageSha256, "manifest.packageManifest.packageSha256"), hak.sha256, "manifest.packageManifest.packageSha256");
  const resources = array(packageManifest.resources, "manifest.packageManifest.resources").map((value, index) => {
    const item = record(value, `manifest.packageManifest.resources[${index}]`);
    return {
      role: string(item.role, `manifest.packageManifest.resources[${index}].role`),
      resref: string(item.resref, `manifest.packageManifest.resources[${index}].resref`),
      type: integer(item.type, `manifest.packageManifest.resources[${index}].type`),
      byteLength: integer(item.byteLength, `manifest.packageManifest.resources[${index}].byteLength`),
      sha256: sha256(item.sha256, `manifest.packageManifest.resources[${index}].sha256`),
    };
  });
  if (resources.length !== 3 || resources.length !== hak.entryCount) throw new Error("Canonical result identity mismatch at HAK resource count");
  const resourcesByRole = new Map(resources.map((resource) => [resource.role, resource]));
  if (resourcesByRole.size !== resources.length) throw new Error("Canonical result identity mismatch at duplicate resource role");
  const resource = (role: string) => resourcesByRole.get(role) ?? fail(`manifest.packageManifest.resources.${role}`);
  const modelResource = resource("MODEL");
  const textureResource = resource("TEXTURE");
  const appearanceResource = resource("APPEARANCE_TABLE");
  if (resourcesByRole.size !== 3) fail("manifest.packageManifest.resources.roles");
  const reconcile = (actual: { byteLength: number; sha256: string }, expected: { byteLength: number; sha256: string }, path: string) => {
    equal(actual.byteLength, expected.byteLength, `${path}.byteLength`);
    equal(actual.sha256, expected.sha256, `${path}.sha256`);
  };
  reconcile(modelResource, outputs.model, "manifest.packageManifest.resources.MODEL");
  reconcile(textureResource, outputs.texture, "manifest.packageManifest.resources.TEXTURE");
  reconcile(appearanceResource, outputs.appearanceTwoDa, "manifest.packageManifest.resources.APPEARANCE_TABLE");
  equal(modelResource.resref, string(summary.modelResref, "summary.modelResref"), "manifest.packageManifest.resources.MODEL.resref");
  equal(string(projection.modelResourceResref, "report.model.projection.modelResourceResref"), modelResource.resref, "report.model.projection.modelResourceResref");
  equal(textureResource.resref, string(summary.textureResref, "summary.textureResref"), "manifest.packageManifest.resources.TEXTURE.resref");
  equal(appearanceResource.resref, "appearance", "manifest.packageManifest.resources.APPEARANCE_TABLE.resref");

  if (artifacts.length !== 6 || new Set(artifacts.map(({ artifactId }) => artifactId)).size !== artifacts.length) {
    throw new Error("Canonical result identity mismatch at artifact inventory");
  }
  const requiredArtifacts = [
    ["package-hak", "HAK", outputs.hak],
    ["model-mdl", "MODEL", outputs.model],
    ["proof-module", "MODULE", outputs.proofModule],
    ["report-json", "JSON_REPORT", outputs.report],
  ] as const;
  for (const [artifactId, kind, expected] of requiredArtifacts) {
    const artifact = artifacts.find((item) => item.artifactId === artifactId)
      ?? fail(`artifacts.${artifactId}`);
    if (artifact.kind !== kind) fail(`artifacts.${artifactId}.kind`);
    equal(artifact.byteLength, expected.byteLength, `artifacts.${artifactId}.byteLength`);
    equal(artifact.sha256, expected.sha256, `artifacts.${artifactId}.sha256`);
  }
  for (const [artifactId, exactJson] of [
    ["report-json", reportJson],
    ["manifest-json", manifestJson],
    ["summary-json", summaryJson],
  ] as const) {
    const artifact = artifacts.find((item) => item.artifactId === artifactId)
      ?? fail(`artifacts.${artifactId}`);
    if (artifact.kind !== "JSON_REPORT") fail(`artifacts.${artifactId}.kind`);
    if (new TextDecoder().decode(artifact.bytes) !== exactJson) {
      throw new Error(`Canonical result identity mismatch at artifacts.${artifactId}.bytes`);
    }
  }
  for (const artifact of artifacts) {
    string(artifact.fileName, "artifacts.fileName");
    integer(artifact.byteLength, "artifacts.byteLength");
    sha256(artifact.sha256, "artifacts.sha256");
    if (artifact.provenance !== "M2A_WASM_WORKER") fail(`artifacts.${artifact.artifactId}.provenance`);
    if (artifact.bytes.byteLength !== artifact.byteLength) throw new Error(`Canonical result identity mismatch at artifact ${artifact.artifactId} bytes`);
  }

  return {
    status,
    sourceMetrics,
    convertedMetrics,
    geometry,
    animation,
    texture,
    resrefs: {
      model: string(summary.modelResref, "summary.modelResref"),
      texture: string(summary.textureResref, "summary.textureResref"),
    },
    appearance: {
      appendedRow,
      sourcePrefixPreserved: boolean(appearanceJson.sourcePrefixPreserved, "report.appearance.sourcePrefixPreserved"),
      policy,
    },
    hak,
    outputs,
    resources,
    semanticEvidence: { semanticDiff, deviations },
    conversionEvidence,
    packageAssemblyEvidence: {
      strictReconciled: true,
      resourceCount: resources.length,
      artifactCount: artifacts.length,
    },
    artifacts: [...artifacts],
    reportJson,
    summaryJson,
    manifestJson,
  };
}
