import type {
  AnimationMappingDiagnosticV1,
  CreatureAnimationMappingStatusV1,
} from "./types";

export interface CoreCreatureAnimationValidationV1 {
  schemaVersion: 1;
  status: CreatureAnimationMappingStatusV1;
  mappedBaseSlotCount: number;
  reviewCount: number;
  blockingCount: number;
  customAnimationCount: number;
  authoringFingerprintSha256: string;
  diagnostics: AnimationMappingDiagnosticV1[];
}

export function projectCoreCreatureAnimationValidationV1(
  validationJson: string,
): CoreCreatureAnimationValidationV1 {
  let value: unknown;
  try {
    value = JSON.parse(validationJson);
  } catch {
    throw new Error("Core animation validation response is not valid JSON");
  }
  if (!isRecord(value) || value.schemaVersion !== 1) {
    throw new Error("Core animation validation response has an unsupported schema");
  }
  if (!["READY", "NEEDS_REVIEW", "BLOCKED"].includes(String(value.status))) {
    throw new Error("Core animation validation response has an invalid status");
  }
  const diagnostics = array(value.diagnostics, "diagnostics").map((entry, index) => {
    if (!isRecord(entry)
      || entry.schemaVersion !== 1
      || typeof entry.code !== "string"
      || typeof entry.path !== "string"
      || !["INFO", "WARNING", "BLOCKING"].includes(String(entry.level))
      || typeof entry.message !== "string"
      || typeof entry.action !== "string") {
      throw new Error(`Core animation validation diagnostic ${index} is invalid`);
    }
    return entry as unknown as AnimationMappingDiagnosticV1;
  });
  return {
    schemaVersion: 1,
    status: value.status as CreatureAnimationMappingStatusV1,
    mappedBaseSlotCount: nonNegativeInteger(value.mappedBaseSlotCount, "mappedBaseSlotCount"),
    reviewCount: nonNegativeInteger(value.reviewCount, "reviewCount"),
    blockingCount: nonNegativeInteger(value.blockingCount, "blockingCount"),
    customAnimationCount: nonNegativeInteger(
      value.customAnimationCount,
      "customAnimationCount",
    ),
    authoringFingerprintSha256: typeof value.authoringFingerprintSha256 === "string"
      ? value.authoringFingerprintSha256
      : "",
    diagnostics,
  };
}

export function mergeUiAndCoreAnimationDiagnosticsV1(
  ui: readonly AnimationMappingDiagnosticV1[],
  core: readonly AnimationMappingDiagnosticV1[],
): AnimationMappingDiagnosticV1[] {
  const merged = new Map<string, AnimationMappingDiagnosticV1>();
  [...ui, ...core].forEach((diagnostic) => {
    merged.set(`${diagnostic.code}\0${diagnostic.path}`, diagnostic);
  });
  return [...merged.values()].sort((left, right) => (
    left.path.localeCompare(right.path) || left.code.localeCompare(right.code)
  ));
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function array(value: unknown, path: string): unknown[] {
  if (!Array.isArray(value)) throw new Error(`Core animation validation ${path} is invalid`);
  return value;
}

function nonNegativeInteger(value: unknown, path: string) {
  if (!Number.isSafeInteger(value) || Number(value) < 0) {
    throw new Error(`Core animation validation ${path} is invalid`);
  }
  return Number(value);
}
