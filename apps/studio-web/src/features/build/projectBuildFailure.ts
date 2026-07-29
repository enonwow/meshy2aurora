import type { BuildFailureSnapshot } from "../../app/studioSession";
type JsonRecord = Record<string, unknown>;

const EXPECTED_FIELDS = ["schemaVersion", "stage", "code", "path", "message"] as const;
export const CORE_BUILD_FAILURE_STAGES = [
  "ANIMATION",
  "APPEARANCE",
  "FIXTURE",
  "INGEST",
  "MODEL",
  "OUTPUT",
  "PACKAGE",
  "PROFILE",
  "PROJECT",
  "PROOF_MODULE",
  "READBACK",
  "REPORT",
  "RUNTIME_FIXTURE_CONTRACT",
  "TEXTURE",
] as const;

export type CoreBuildFailureStage = typeof CORE_BUILD_FAILURE_STAGES[number];

const CORE_BUILD_FAILURE_STAGE_LABELS: Readonly<Record<CoreBuildFailureStage, string>> = {
  ANIMATION: "Animation authoring",
  APPEARANCE: "2DA appearance",
  FIXTURE: "Fixture contract",
  INGEST: "Source ingest",
  MODEL: "Binary model",
  OUTPUT: "Output materialization",
  PACKAGE: "HAK package",
  PROFILE: "Canonical profile",
  PROJECT: "Project identity",
  PROOF_MODULE: "Proof module",
  READBACK: "Binary readback",
  REPORT: "Evidence report",
  RUNTIME_FIXTURE_CONTRACT: "Runtime fixture contract",
  TEXTURE: "Texture conversion",
};

const isRecord = (value: unknown): value is JsonRecord =>
  value !== null && typeof value === "object" && !Array.isArray(value);

const isNonEmptyString = (value: unknown): value is string =>
  typeof value === "string" && value.length > 0;

function hasExactFields(value: JsonRecord): boolean {
  const keys = Object.keys(value);
  return keys.length === EXPECTED_FIELDS.length
    && EXPECTED_FIELDS.every((field) => Object.hasOwn(value, field));
}

export function isCoreBuildFailureStage(value: unknown): value is CoreBuildFailureStage {
  return typeof value === "string"
    && (CORE_BUILD_FAILURE_STAGES as readonly string[]).includes(value);
}

export function buildFailureStageLabel(stage?: string) {
  return isCoreBuildFailureStage(stage)
    ? CORE_BUILD_FAILURE_STAGE_LABELS[stage]
    : "Unclassified Worker failure";
}

/**
 * Projects the exact JSON carried by a WASM `M6PipelineErrorV1`.
 * Any non-contract value remains an opaque raw message; partial JSON evidence
 * is never promoted into structured diagnostics.
 */
export function projectBuildFailure(rawMessage: string): BuildFailureSnapshot {
  try {
    const value: unknown = JSON.parse(rawMessage);
    if (
      !isRecord(value)
      || !hasExactFields(value)
      || value.schemaVersion !== 1
      || !isCoreBuildFailureStage(value.stage)
      || !isNonEmptyString(value.code)
      || !isNonEmptyString(value.path)
      || !isNonEmptyString(value.message)
    ) return { message: rawMessage };

    return {
      stage: value.stage,
      code: value.code,
      path: value.path,
      message: value.message,
    };
  } catch {
    return { message: rawMessage };
  }
}
