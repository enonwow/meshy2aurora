type JsonRecord = Record<string, unknown>;

export type TwoDaNewline = "LF" | "CR_LF";

export type TwoDaCellValue =
  | { kind: "NULL" }
  | { kind: "TEXT"; value: string };

export interface AppearanceDiagnostic {
  schemaVersion: 1;
  code: string;
  severity: string;
  path: string;
  line: number | null;
  message: string;
}

export interface AppearanceInspectionSnapshot {
  schemaVersion: 1;
  format: "2DA";
  version: "V2.0";
  sourceSha256: string;
  byteLength: number;
  newline: TwoDaNewline;
  terminalNewline: boolean;
  defaultValue: TwoDaCellValue | null;
  columns: string[];
  physicalRowCount: number;
  nextAppendIndex: number | null;
  rowLabelMismatchCount: number;
  diagnostics: AppearanceDiagnostic[];
}

const fail = (path: string): never => {
  throw new Error(`Appearance inspection field ${path} is missing or has the wrong type`);
};

const record = (value: unknown, path: string): JsonRecord =>
  value !== null && typeof value === "object" && !Array.isArray(value)
    ? value as JsonRecord
    : fail(path);

const array = (value: unknown, path: string): unknown[] =>
  Array.isArray(value) ? value : fail(path);

const string = (value: unknown, path: string): string =>
  typeof value === "string" ? value : fail(path);

const nonEmptyString = (value: unknown, path: string): string => {
  const parsed = string(value, path);
  return parsed.length > 0 ? parsed : fail(path);
};

const boolean = (value: unknown, path: string): boolean =>
  typeof value === "boolean" ? value : fail(path);

const nonNegativeInteger = (value: unknown, path: string): number =>
  Number.isSafeInteger(value) && (value as number) >= 0 ? value as number : fail(path);

const nullableInteger = (value: unknown, path: string): number | null =>
  value === null ? null : nonNegativeInteger(value, path);

const schemaVersion = (value: unknown, path: string): 1 =>
  nonNegativeInteger(value, path) === 1 ? 1 : fail(path);

const exact = <T extends string>(value: unknown, expected: T, path: string): T =>
  value === expected ? expected : fail(path);

const sha256 = (value: unknown, path: string): string => {
  const parsed = nonEmptyString(value, path);
  return /^[0-9a-f]{64}$/.test(parsed) ? parsed : fail(path);
};

const parseNewline = (value: unknown, path: string): TwoDaNewline => {
  const parsed = string(value, path);
  return parsed === "LF" || parsed === "CR_LF" ? parsed : fail(path);
};

function parseCellValue(value: unknown, path: string): TwoDaCellValue | null {
  if (value === null) return null;
  const cell = record(value, path);
  if (cell.kind === "NULL") return { kind: "NULL" };
  if (cell.kind === "TEXT") {
    return { kind: "TEXT", value: string(cell.value, `${path}.value`) };
  }
  return fail(`${path}.kind`);
}

function parseDiagnostic(value: unknown, index: number): AppearanceDiagnostic {
  const path = `inspectionJson.diagnostics[${index}]`;
  const diagnostic = record(value, path);
  return {
    schemaVersion: schemaVersion(diagnostic.schemaVersion, `${path}.schemaVersion`),
    code: nonEmptyString(diagnostic.code, `${path}.code`),
    severity: nonEmptyString(diagnostic.severity, `${path}.severity`),
    path: nonEmptyString(diagnostic.path, `${path}.path`),
    line: nullableInteger(diagnostic.line, `${path}.line`),
    message: nonEmptyString(diagnostic.message, `${path}.message`),
  };
}

/** Projects the exact successful JSON returned by `inspectTwoDaV2Json`. */
export function projectAppearanceInspection(inspectionJson: string): AppearanceInspectionSnapshot {
  let parsed: unknown;
  try {
    parsed = JSON.parse(inspectionJson);
  } catch {
    return fail("inspectionJson");
  }
  const inspection = record(parsed, "inspectionJson");
  const newline = parseNewline(inspection.newline, "inspectionJson.newline");

  const columns = array(inspection.columns, "inspectionJson.columns")
    .map((value, index) => nonEmptyString(value, `inspectionJson.columns[${index}]`));
  const nextAppendIndex = nullableInteger(
    inspection.nextAppendIndex,
    "inspectionJson.nextAppendIndex",
  );
  if (nextAppendIndex !== null && nextAppendIndex > 65_535) {
    fail("inspectionJson.nextAppendIndex");
  }

  return {
    schemaVersion: schemaVersion(inspection.schemaVersion, "inspectionJson.schemaVersion"),
    format: exact(inspection.format, "2DA", "inspectionJson.format"),
    version: exact(inspection.version, "V2.0", "inspectionJson.version"),
    sourceSha256: sha256(inspection.sourceSha256, "inspectionJson.sourceSha256"),
    byteLength: nonNegativeInteger(inspection.byteLength, "inspectionJson.byteLength"),
    newline,
    terminalNewline: boolean(inspection.terminalNewline, "inspectionJson.terminalNewline"),
    defaultValue: parseCellValue(inspection.defaultValue, "inspectionJson.defaultValue"),
    columns,
    physicalRowCount: nonNegativeInteger(
      inspection.physicalRowCount,
      "inspectionJson.physicalRowCount",
    ),
    nextAppendIndex,
    rowLabelMismatchCount: nonNegativeInteger(
      inspection.rowLabelMismatchCount,
      "inspectionJson.rowLabelMismatchCount",
    ),
    diagnostics: array(inspection.diagnostics, "inspectionJson.diagnostics")
      .map(parseDiagnostic),
  };
}
