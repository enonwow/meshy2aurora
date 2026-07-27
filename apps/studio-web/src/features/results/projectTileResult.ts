import type { WorkerArtifact } from "../../worker/types";

export interface TileResourceSnapshot {
  container: "HAK" | "MOD";
  role: string;
  resref: string;
  resourceType: number;
  byteLength: number;
  sha256: string;
}

export interface TileResultSnapshot {
  status: "ready_for_owner_proof";
  profile: "TileStaticV1";
  moduleFileName: string;
  moduleDisplayName: string;
  areaResref: string;
  areaName: string;
  areaSize: [number, number];
  areaTileCount: number;
  entryPosition: [number, number, number];
  hakFileName: string;
  hakResref: string;
  tilesetResref: string;
  tileId: number;
  modelResref: string;
  wokResref: string;
  textureResref: string;
  imageMapResref: string;
  walkmeshClassToken: string;
  surfaceId: number;
  modelTriangleCount: number;
  wokTriangleCount: number;
  aabbEntryCount: number;
  modelVisibility: "not_tested";
  proofCompleteness: "missing";
  navigationSpawnWalkable: "offline_verified";
  navigationSeamWalkable: "offline_verified";
  resources: TileResourceSnapshot[];
  artifacts: WorkerArtifact[];
  reportJson: string;
  wokReadbackJson: string;
  setReadbackJson: string;
}

type JsonRecord = Record<string, unknown>;

const fail = (path: string): never => {
  throw new Error(`Tile result field ${path} is missing, inconsistent or has the wrong type`);
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

function tuple(
  value: unknown,
  length: number,
  path: string,
): number[] {
  const values = array(value, path);
  if (values.length !== length) fail(path);
  return values.map((item, index) => finite(item, `${path}[${index}]`));
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
  expectedSha256?: string,
) {
  const value = artifactById(artifacts, artifactId);
  if (
    value.kind !== kind
    || value.fileName !== fileName
    || value.provenance !== "M2A_WASM_WORKER"
    || value.byteLength !== value.bytes.byteLength
    || (expectedSha256 !== undefined && value.sha256 !== expectedSha256)
  ) fail(`${artifactId}.identity`);
}

export function projectTileResult(
  reportJson: string,
  wokReadbackJson: string,
  setReadbackJson: string,
  artifactsInput: readonly WorkerArtifact[],
): TileResultSnapshot {
  let report: JsonRecord;
  try {
    report = record(JSON.parse(reportJson), "report");
    record(JSON.parse(wokReadbackJson), "wokReadback");
    record(JSON.parse(setReadbackJson), "setReadback");
  } catch {
    return fail("JSON");
  }
  if (integer(report.schemaVersion, "report.schemaVersion") !== 1) fail("report.schemaVersion");
  const moduleFileName = string(report.moduleFileName, "report.moduleFileName");
  const hakFileName = string(report.hakFileName, "report.hakFileName");
  const modelResref = string(report.modelResref, "report.modelResref");
  const wokResref = string(report.wokResref, "report.wokResref");
  const tilesetResref = string(report.tilesetResref, "report.tilesetResref");
  const textureResref = string(report.textureResref, "report.textureResref");
  const imageMapResref = string(report.imageMapResref, "report.imageMapResref");
  verifyArtifact(artifactsInput, "tile-package-hak", "HAK", hakFileName, sha256(report.hakSha256, "report.hakSha256"));
  verifyArtifact(artifactsInput, "tile-proof-module", "MODULE", moduleFileName, sha256(report.moduleSha256, "report.moduleSha256"));
  verifyArtifact(artifactsInput, "tile-model-mdl", "MODEL", `${modelResref}.mdl`, sha256(report.mdlSha256, "report.mdlSha256"));
  verifyArtifact(artifactsInput, "tile-navigation-wok", "WOK", `${wokResref}.wok`, sha256(report.wokSha256, "report.wokSha256"));
  verifyArtifact(artifactsInput, "tile-tileset-set", "SET", `${tilesetResref}.set`, sha256(report.setSha256, "report.setSha256"));
  verifyArtifact(artifactsInput, "tile-texture-tga", "TEXTURE", `${textureResref}.tga`, sha256(report.textureSha256, "report.textureSha256"));
  verifyArtifact(artifactsInput, "tile-image-map-tga", "TEXTURE", `${imageMapResref}.tga`, sha256(report.imageMapSha256, "report.imageMapSha256"));
  verifyArtifact(artifactsInput, "tile-report-json", "JSON_REPORT", "tile-materialization-report.json");
  verifyArtifact(artifactsInput, "tile-model-readback-json", "JSON_REPORT", "tile-model-readback.json");
  verifyArtifact(artifactsInput, "tile-wok-readback-json", "JSON_REPORT", "tile-wok-readback.json");
  verifyArtifact(artifactsInput, "tile-set-readback-json", "JSON_REPORT", "tile-set-readback.json");

  const resources = array(report.resources, "report.resources").map((value, index): TileResourceSnapshot => {
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
  const areaSize = tuple(report.areaSize, 2, "report.areaSize");
  const entryPosition = tuple(report.entryPosition, 3, "report.entryPosition");
  return {
    status: exact(report.status, "ready_for_owner_proof", "report.status"),
    profile: exact(report.profile, "TileStaticV1", "report.profile"),
    moduleFileName,
    moduleDisplayName: string(report.moduleDisplayName, "report.moduleDisplayName"),
    areaResref: string(report.areaResref, "report.areaResref"),
    areaName: string(report.areaName, "report.areaName"),
    areaSize: [areaSize[0], areaSize[1]],
    areaTileCount: integer(report.areaTileCount, "report.areaTileCount"),
    entryPosition: [entryPosition[0], entryPosition[1], entryPosition[2]],
    hakFileName,
    hakResref: string(report.hakResref, "report.hakResref"),
    tilesetResref,
    tileId: integer(report.tileId, "report.tileId"),
    modelResref,
    wokResref,
    textureResref,
    imageMapResref,
    walkmeshClassToken: string(report.walkmeshClassToken, "report.walkmeshClassToken"),
    surfaceId: integer(report.surfaceId, "report.surfaceId"),
    modelTriangleCount: integer(report.modelTriangleCount, "report.modelTriangleCount"),
    wokTriangleCount: integer(report.wokTriangleCount, "report.wokTriangleCount"),
    aabbEntryCount: integer(report.aabbEntryCount, "report.aabbEntryCount"),
    modelVisibility: exact(report.modelVisibility, "not_tested", "report.modelVisibility"),
    proofCompleteness: exact(report.proofCompleteness, "missing", "report.proofCompleteness"),
    navigationSpawnWalkable: exact(report.navigationSpawnWalkable, "offline_verified", "report.navigationSpawnWalkable"),
    navigationSeamWalkable: exact(report.navigationSeamWalkable, "offline_verified", "report.navigationSeamWalkable"),
    resources,
    artifacts: [...artifactsInput],
    reportJson,
    wokReadbackJson,
    setReadbackJson,
  };
}
