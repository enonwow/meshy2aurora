import type { WorkerArtifact } from "../../worker/types";

export interface PlaceableComponentStatuses {
  mdl: "passed";
  twoDa: "passed";
  utp: "passed";
  gitGic: "passed";
  palette: "passed";
  package: "passed";
  proof: "not_tested";
}

export interface PlaceableResourceSnapshot {
  container: "HAK" | "MOD";
  role: string;
  resref: string;
  resourceType: number;
  byteLength: number;
  sha256: string;
}

export interface PlaceableResultSnapshot {
  status: "OFFLINE_ADMISSION_PASSED";
  profile: "STATIC_PLACEABLE";
  componentStatuses: PlaceableComponentStatuses;
  moduleFileName: string;
  moduleDisplayName: string;
  areaResref: string;
  areaName: string;
  hakFileName: string;
  modelResref: string;
  textureResref: string;
  blueprintResref: string;
  objectTag: string;
  appearanceRow: number;
  placement: { x: number; y: number; z: number; bearing: number };
  modelVisibility: "not_tested";
  proofCompleteness: "missing";
  paletteCompleteness: string;
  collisionCompleteness: string;
  authoring?: {
    sourceSha256: string;
    authoringSha256: string;
    sourceTriangleCount: number;
    outputTriangleCount: number;
    renderableElementCount: number;
    collisionElementCount: number;
    shadowElementCount: number;
    boundsMin: [number, number, number];
    boundsMax: [number, number, number];
  };
  resources: PlaceableResourceSnapshot[];
  artifacts: WorkerArtifact[];
  reportJson: string;
}

type JsonRecord = Record<string, unknown>;

const fail = (path: string): never => {
  throw new Error(`Placeable result field ${path} is missing, inconsistent or has the wrong type`);
};
const record = (value: unknown, path: string): JsonRecord =>
  value !== null && typeof value === "object" && !Array.isArray(value) ? value as JsonRecord : fail(path);
const array = (value: unknown, path: string): unknown[] => Array.isArray(value) ? value : fail(path);
const string = (value: unknown, path: string): string =>
  typeof value === "string" && value.length > 0 ? value : fail(path);
const integer = (value: unknown, path: string): number =>
  Number.isSafeInteger(value) && (value as number) >= 0 ? value as number : fail(path);
const finite = (value: unknown, path: string): number =>
  typeof value === "number" && Number.isFinite(value) ? value : fail(path);
const sha256 = (value: unknown, path: string): string => {
  const result = string(value, path);
  return /^[0-9a-f]{64}$/.test(result) ? result : fail(path);
};
const exact = <T extends string>(value: unknown, expected: T, path: string): T =>
  string(value, path) === expected ? expected : fail(path);
const vec3 = (value: unknown, path: string): [number, number, number] => {
  const values = array(value, path);
  return values.length === 3
    ? [
        finite(values[0], `${path}[0]`),
        finite(values[1], `${path}[1]`),
        finite(values[2], `${path}[2]`),
      ]
    : fail(path);
};

function parseJson(json: string) {
  try {
    return record(JSON.parse(json), "report");
  } catch {
    return fail("reportJson");
  }
}

function artifactById(artifacts: readonly WorkerArtifact[], artifactId: string) {
  const matches = artifacts.filter((artifact) => artifact.artifactId === artifactId);
  return matches.length === 1 ? matches[0] : fail(`artifacts.${artifactId}`);
}

function verifyArtifact(
  artifacts: readonly WorkerArtifact[],
  artifactId: string,
  kind: WorkerArtifact["kind"],
  fileName: string,
  expectedSha256: string | undefined,
) {
  const artifact = artifactById(artifacts, artifactId);
  if (
    artifact.kind !== kind
    || artifact.fileName !== fileName
    || artifact.provenance !== "M2A_WASM_WORKER"
    || artifact.byteLength !== artifact.bytes.byteLength
  ) fail(`${artifactId}.identity`);
  if (expectedSha256 && artifact.sha256 !== expectedSha256) fail(`${artifactId}.sha256`);
}

export function projectPlaceableResult(
  reportJson: string,
  artifactsInput: readonly WorkerArtifact[],
): PlaceableResultSnapshot {
  const report = parseJson(reportJson);
  if (integer(report.schemaVersion, "report.schemaVersion") !== 1) fail("report.schemaVersion");
  const status = exact(report.status, "OFFLINE_ADMISSION_PASSED", "report.status");
  const profile = exact(report.profile, "STATIC_PLACEABLE", "report.profile");
  const statuses = record(report.componentStatuses, "report.componentStatuses");
  const componentStatuses: PlaceableComponentStatuses = {
    mdl: exact(statuses.mdl, "passed", "report.componentStatuses.mdl"),
    twoDa: exact(statuses.twoDa, "passed", "report.componentStatuses.twoDa"),
    utp: exact(statuses.utp, "passed", "report.componentStatuses.utp"),
    gitGic: exact(statuses.gitGic, "passed", "report.componentStatuses.gitGic"),
    palette: exact(statuses.palette, "passed", "report.componentStatuses.palette"),
    package: exact(statuses.package, "passed", "report.componentStatuses.package"),
    proof: exact(statuses.proof, "not_tested", "report.componentStatuses.proof"),
  };
  const appearance = record(report.appearanceRow, "report.appearanceRow");
  const placementJson = record(report.placement, "report.placement");
  const resources = array(report.resources, "report.resources").map((value, index): PlaceableResourceSnapshot => {
    const item = record(value, `report.resources[${index}]`);
    const containerValue = string(item.container, `report.resources[${index}].container`);
    const container: "HAK" | "MOD" = containerValue === "HAK"
      ? "HAK"
      : containerValue === "MOD"
        ? "MOD"
        : fail(`report.resources[${index}].container`);
    return {
      container,
      role: string(item.role, `report.resources[${index}].role`),
      resref: string(item.resref, `report.resources[${index}].resref`),
      resourceType: integer(item.resourceType, `report.resources[${index}].resourceType`),
      byteLength: integer(item.byteLength, `report.resources[${index}].byteLength`),
      sha256: sha256(item.sha256, `report.resources[${index}].sha256`),
    };
  });
  const hakResourceCount = integer(report.hakResourceCount, "report.hakResourceCount");
  const moduleResourceCount = integer(report.moduleResourceCount, "report.moduleResourceCount");
  if (
    resources.filter(({ container }) => container === "HAK").length !== hakResourceCount
    || resources.filter(({ container }) => container === "MOD").length !== moduleResourceCount
  ) fail("report.resources");

  const moduleFileName = string(report.moduleFileName, "report.moduleFileName");
  const hakFileName = string(report.hakFileName, "report.hakFileName");
  const modelResref = string(report.modelResref, "report.modelResref");
  const modelSha256 = sha256(report.mdlSha256, "report.mdlSha256");
  verifyArtifact(
    artifactsInput,
    "placeable-package-hak",
    "HAK",
    hakFileName,
    sha256(report.hakSha256, "report.hakSha256"),
  );
  verifyArtifact(
    artifactsInput,
    "placeable-model-mdl",
    "MODEL",
    `${modelResref}.mdl`,
    modelSha256,
  );
  verifyArtifact(
    artifactsInput,
    "placeable-proof-module",
    "MODULE",
    moduleFileName,
    sha256(report.moduleSha256, "report.moduleSha256"),
  );
  verifyArtifact(
    artifactsInput,
    "placeable-report-json",
    "JSON_REPORT",
    "placeable-materialization-report.json",
    undefined,
  );

  const modelResource = resources.filter(
    (resource) => resource.container === "HAK"
      && resource.resref === modelResref
      && resource.resourceType === 2002
      && resource.sha256 === modelSha256,
  );
  if (modelResource.length !== 1) fail("report.resources.MODEL");
  const authoringRecord = report.authoring === undefined
    ? undefined
    : record(report.authoring, "report.authoring");
  const authoring = authoringRecord ? {
    sourceSha256: sha256(authoringRecord.sourceSha256, "report.authoring.sourceSha256"),
    authoringSha256: sha256(authoringRecord.authoringSha256, "report.authoring.authoringSha256"),
    sourceTriangleCount: integer(authoringRecord.sourceTriangleCount, "report.authoring.sourceTriangleCount"),
    outputTriangleCount: integer(authoringRecord.outputTriangleCount, "report.authoring.outputTriangleCount"),
    renderableElementCount: integer(authoringRecord.renderableElementCount, "report.authoring.renderableElementCount"),
    collisionElementCount: integer(authoringRecord.collisionElementCount, "report.authoring.collisionElementCount"),
    shadowElementCount: integer(authoringRecord.shadowElementCount, "report.authoring.shadowElementCount"),
    boundsMin: vec3(authoringRecord.boundsMin, "report.authoring.boundsMin"),
    boundsMax: vec3(authoringRecord.boundsMax, "report.authoring.boundsMax"),
  } : undefined;

  return {
    status,
    profile,
    componentStatuses,
    moduleFileName,
    moduleDisplayName: string(report.moduleDisplayName, "report.moduleDisplayName"),
    areaResref: string(report.areaResref, "report.areaResref"),
    areaName: string(report.areaName, "report.areaName"),
    hakFileName,
    modelResref,
    textureResref: string(report.textureResref, "report.textureResref"),
    blueprintResref: string(report.blueprintResref, "report.blueprintResref"),
    objectTag: string(report.objectTag, "report.objectTag"),
    appearanceRow: integer(appearance.value, "report.appearanceRow.value"),
    placement: {
      x: finite(placementJson.x, "report.placement.x"),
      y: finite(placementJson.y, "report.placement.y"),
      z: finite(placementJson.z, "report.placement.z"),
      bearing: finite(placementJson.bearing, "report.placement.bearing"),
    },
    modelVisibility: exact(report.modelVisibility, "not_tested", "report.modelVisibility"),
    proofCompleteness: exact(report.proofCompleteness, "missing", "report.proofCompleteness"),
    paletteCompleteness: string(report.paletteCompleteness, "report.paletteCompleteness"),
    collisionCompleteness: string(report.collisionCompleteness, "report.collisionCompleteness"),
    authoring,
    resources,
    artifacts: [...artifactsInput],
    reportJson,
  };
}
