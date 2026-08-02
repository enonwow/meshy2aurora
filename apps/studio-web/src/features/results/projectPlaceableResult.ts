import type { WorkerArtifact } from "../../worker/types";
import {
  parseResolvedPlaceableTextures,
  type ResolvedPlaceableTextures,
} from "../placeable-authoring/textureTypes";

export interface PlaceableComponentStatuses {
  mdl: "passed";
  pwk: "passed";
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
  collision?: {
    mode: "AUTO_RECTANGLE" | "CUSTOM_POLYGON";
    inputVertices: [number, number][];
    sourceVertices: [number, number][];
    vertices: [number, number][];
    triangles: [number, number, number][];
    boundsMin: [number, number];
    boundsMax: [number, number];
    surfaceId: 7;
    authoringSha256: string;
    collisionSha256: string;
    pwkSha256: string;
  };
  textureAuthoring?: ResolvedPlaceableTextures;
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
const collisionMode = (value: unknown, path: string): "AUTO_RECTANGLE" | "CUSTOM_POLYGON" => {
  const mode = string(value, path);
  return mode === "AUTO_RECTANGLE" || mode === "CUSTOM_POLYGON" ? mode : fail(path);
};
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
const vec2 = (value: unknown, path: string): [number, number] => {
  const values = array(value, path);
  return values.length === 2
    ? [finite(values[0], `${path}[0]`), finite(values[1], `${path}[1]`)]
    : fail(path);
};
const vec2Array = (value: unknown, path: string): [number, number][] =>
  array(value, path).map((item, index) => vec2(item, `${path}[${index}]`));

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
    pwk: exact(statuses.pwk, "passed", "report.componentStatuses.pwk"),
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
  const pwkSha256 = sha256(report.pwkSha256, "report.pwkSha256");
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
  const walkmeshResource = resources.filter(
    (resource) => resource.container === "HAK"
      && resource.role === "PLACEABLE_WALKMESH"
      && resource.resref === modelResref
      && resource.resourceType === 2053
      && resource.sha256 === pwkSha256,
  );
  if (walkmeshResource.length !== 1) fail("report.resources.PLACEABLE_WALKMESH");
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
  const collisionRecord = report.collision === undefined
    ? undefined
    : record(report.collision, "report.collision");
  const collision = collisionRecord ? {
    mode: collisionMode(collisionRecord.mode, "report.collision.mode"),
    inputVertices: vec2Array(collisionRecord.inputVertices, "report.collision.inputVertices"),
    sourceVertices: vec2Array(collisionRecord.sourceVertices, "report.collision.sourceVertices"),
    vertices: vec2Array(collisionRecord.vertices, "report.collision.vertices"),
    triangles: array(collisionRecord.triangles, "report.collision.triangles").map((item, index) => {
      const triangle = array(item, `report.collision.triangles[${index}]`);
      return triangle.length === 3
        ? [
            integer(triangle[0], `report.collision.triangles[${index}][0]`),
            integer(triangle[1], `report.collision.triangles[${index}][1]`),
            integer(triangle[2], `report.collision.triangles[${index}][2]`),
          ] as [number, number, number]
        : fail(`report.collision.triangles[${index}]`);
    }),
    boundsMin: vec2(collisionRecord.boundsMin, "report.collision.boundsMin"),
    boundsMax: vec2(collisionRecord.boundsMax, "report.collision.boundsMax"),
    surfaceId: integer(collisionRecord.surfaceId, "report.collision.surfaceId") === 7
      ? 7 as const
      : fail("report.collision.surfaceId"),
    authoringSha256: sha256(collisionRecord.authoringSha256, "report.collision.authoringSha256"),
    collisionSha256: sha256(collisionRecord.collisionSha256, "report.collision.collisionSha256"),
    pwkSha256: sha256(collisionRecord.pwkSha256, "report.collision.pwkSha256"),
  } : undefined;
  if (collision && collision.pwkSha256 !== pwkSha256) fail("report.collision.pwkSha256");
  if (collision && authoring && collision.authoringSha256 !== authoring.authoringSha256) {
    fail("report.collision.authoringSha256");
  }
  const textureAuthoring = report.textureAuthoring === undefined
    ? undefined
    : parseResolvedPlaceableTextures(JSON.stringify(report.textureAuthoring));
  if (textureAuthoring) {
    verifyArtifact(
      artifactsInput,
      "placeable-authoring-v3-json",
      "JSON_REPORT",
      "placeable-authoring-v3.json",
      undefined,
    );
    for (const texture of textureAuthoring.resources) {
      verifyArtifact(
        artifactsInput,
        `placeable-texture-${texture.resref}-tga`,
        "TEXTURE",
        `${texture.resref}.tga`,
        texture.sha256,
      );
      const packaged = resources.filter((resource) => (
        resource.container === "HAK"
        && resource.resref === texture.resref
        && resource.resourceType === texture.resourceType
        && resource.sha256 === texture.sha256
      ));
      if (packaged.length !== 1) fail(`report.textureAuthoring.resources.${texture.resref}`);
    }
  }

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
    collisionCompleteness: exact(
      report.collisionCompleteness,
      "ascii_pwk_emitted_offline_readback_passed",
      "report.collisionCompleteness",
    ),
    authoring,
    collision,
    textureAuthoring,
    resources,
    artifacts: [...artifactsInput],
    reportJson,
  };
}
