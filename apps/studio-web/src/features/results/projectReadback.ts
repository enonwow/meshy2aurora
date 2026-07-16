import type {
  BinaryMdlInspectionReport,
  BinaryReadbackValidationEvidence,
  ReadbackController,
  ReadbackDiagnostic,
  ReadbackMesh,
  ReadbackNode,
  ReadbackVec2,
  ReadbackVec3,
} from "../preview/types";

type JsonRecord = Record<string, unknown>;
const NWN1_BINARY_MDL_FORMAT = "nwn1-binary-mdl";
const fail = (path: string): never => { throw new Error(`Canonical readback field ${path} is missing or has the wrong type`); };
const record = (value: unknown, path: string): JsonRecord => value !== null && typeof value === "object" && !Array.isArray(value) ? value as JsonRecord : fail(path);
const array = (value: unknown, path: string): unknown[] => Array.isArray(value) ? value : fail(path);
const string = (value: unknown, path: string): string => typeof value === "string" ? value : fail(path);
const number = (value: unknown, path: string): number => typeof value === "number" && Number.isFinite(value) ? value : fail(path);
const integer = (value: unknown, path: string): number => Number.isSafeInteger(value) && (value as number) >= 0 ? value as number : fail(path);

const vec3 = (value: unknown, path: string): ReadbackVec3 => {
  const item = record(value, path);
  return { x: number(item.x, `${path}.x`), y: number(item.y, `${path}.y`), z: number(item.z, `${path}.z`) };
};
const vec2 = (value: unknown, path: string): ReadbackVec2 => {
  const item = record(value, path);
  return { x: number(item.x, `${path}.x`), y: number(item.y, `${path}.y`) };
};

function controller(value: unknown, path: string): ReadbackController {
  const item = record(value, path);
  if (item.controllerName !== undefined && item.controllerName !== null && typeof item.controllerName !== "string") fail(`${path}.controllerName`);
  return {
    ...(item.controllerName === undefined || item.controllerName === null ? {} : { controllerName: item.controllerName as string }),
    values: array(item.values, `${path}.values`).map((row, rowIndex) =>
      array(row, `${path}.values[${rowIndex}]`).map((entry, index) =>
        number(entry, `${path}.values[${rowIndex}][${index}]`))),
  };
}

function mesh(value: unknown, path: string): ReadbackMesh {
  const item = record(value, path);
  return {
    vertices: array(item.vertices, `${path}.vertices`).map((entry, index) => vec3(entry, `${path}.vertices[${index}]`)),
    normals: array(item.normals, `${path}.normals`).map((entry, index) => vec3(entry, `${path}.normals[${index}]`)),
    uv0: array(item.uv0, `${path}.uv0`).map((entry, index) => vec2(entry, `${path}.uv0[${index}]`)),
    rawIndices: array(item.rawIndices, `${path}.rawIndices`).map((row, rowIndex) =>
      array(row, `${path}.rawIndices[${rowIndex}]`).map((entry, index) =>
        integer(entry, `${path}.rawIndices[${rowIndex}][${index}]`))),
    faces: array(item.faces, `${path}.faces`).map((entry, index) => {
      const face = record(entry, `${path}.faces[${index}]`);
      const indices = array(face.vertexIndices, `${path}.faces[${index}].vertexIndices`);
      if (indices.length !== 3) fail(`${path}.faces[${index}].vertexIndices`);
      return {
        vertexIndices: [
          integer(indices[0], `${path}.faces[${index}].vertexIndices[0]`),
          integer(indices[1], `${path}.faces[${index}].vertexIndices[1]`),
          integer(indices[2], `${path}.faces[${index}].vertexIndices[2]`),
        ],
      };
    }),
  };
}

function node(value: unknown, path: string): ReadbackNode {
  const item = record(value, path);
  return {
    offset: integer(item.offset, `${path}.offset`),
    number: integer(item.number, `${path}.number`),
    name: string(item.name, `${path}.name`),
    controllers: array(item.controllers, `${path}.controllers`).map((entry, index) => controller(entry, `${path}.controllers[${index}]`)),
    ...(item.mesh === undefined || item.mesh === null ? {} : { mesh: mesh(item.mesh, `${path}.mesh`) }),
    children: array(item.children, `${path}.children`).map((entry, index) => node(entry, `${path}.children[${index}]`)),
  };
}

function diagnostic(value: unknown, path: string): ReadbackDiagnostic {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  return {
    schemaVersion: 1,
    code: string(item.code, `${path}.code`),
    severity: string(item.severity, `${path}.severity`),
    offset: integer(item.offset, `${path}.offset`),
    context: string(item.context, `${path}.context`),
  };
}

function validationEvidence(
  roots: ReadbackNode[],
  diagnostics: ReadbackDiagnostic[],
): BinaryReadbackValidationEvidence {
  const counts = diagnostics.reduce((current, item) => {
    const severity = item.severity.trim().toUpperCase();
    if (severity === "WARNING" || severity === "WARN") current.warnings += 1;
    else if (severity === "ERROR" || severity === "FATAL" || severity === "BLOCKING") current.errors += 1;
    else if (severity === "INFO" || severity === "INFORMATION" || severity === "NOTE") current.informational += 1;
    else current.unrecognizedSeverity += 1;
    return current;
  }, { warnings: 0, errors: 0, informational: 0, unrecognizedSeverity: 0 });
  const structuralErrors = roots.length === 0 ? ["READBACK_NODE_TREE_EMPTY"] : [];
  const status = structuralErrors.length > 0 || counts.errors > 0 || counts.unrecognizedSeverity > 0
    ? "ERROR"
    : counts.warnings > 0
      ? "WARNING"
      : "PASS";
  return {
    status,
    structure: {
      schemaVersion: 1,
      format: NWN1_BINARY_MDL_FORMAT,
      rootNodeCount: roots.length,
      hasRootNodes: roots.length > 0,
      structuralErrors,
    },
    diagnostics: {
      total: diagnostics.length,
      ...counts,
    },
  };
}

export function projectCanonicalReadback(readbackJson: string): BinaryMdlInspectionReport {
  let parsed: unknown;
  try { parsed = JSON.parse(readbackJson); } catch { return fail("readbackJson"); }
  const report = record(parsed, "readbackJson");
  if (integer(report.schemaVersion, "readbackJson.schemaVersion") !== 1) fail("readbackJson.schemaVersion");
  const tree = record(report.nodeTree, "readbackJson.nodeTree");
  const format = string(report.format, "readbackJson.format");
  if (format !== NWN1_BINARY_MDL_FORMAT) fail("readbackJson.format");
  const roots = array(tree.roots, "readbackJson.nodeTree.roots").map((entry, index) => node(entry, `readbackJson.nodeTree.roots[${index}]`));
  const diagnostics = array(report.diagnostics, "readbackJson.diagnostics").map((entry, index) => diagnostic(entry, `readbackJson.diagnostics[${index}]`));
  return {
    schemaVersion: 1,
    format,
    nodeTree: { roots },
    diagnostics,
    validation: validationEvidence(roots, diagnostics),
  };
}
