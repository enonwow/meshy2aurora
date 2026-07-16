import type { BuildFailureSnapshot } from "../../app/studioSession";
import type { BuildStageId } from "./BuildStep";

type JsonRecord = Record<string, unknown>;

const EXPECTED_FIELDS = ["schemaVersion", "stage", "code", "path", "message"] as const;

const isRecord = (value: unknown): value is JsonRecord =>
  value !== null && typeof value === "object" && !Array.isArray(value);

const isNonEmptyString = (value: unknown): value is string =>
  typeof value === "string" && value.length > 0;

function hasExactFields(value: JsonRecord): boolean {
  const keys = Object.keys(value);
  return keys.length === EXPECTED_FIELDS.length
    && EXPECTED_FIELDS.every((field) => Object.hasOwn(value, field));
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
      || !isNonEmptyString(value.stage)
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

const PIPELINE_STAGE_TO_BUILD_STAGE: Readonly<Record<string, BuildStageId>> = {
  INGEST: "INGEST_SOURCE",
  PROFILE: "NORMALIZE_CANONICAL_IR",
  MODEL: "WRITE_BINARY_MDL",
  READBACK: "CANONICAL_BINARY_READBACK",
  APPEARANCE: "UPDATE_APPEARANCE_2DA",
  HAK: "PACKAGE_HAK",
};

/** Maps only stages with an unambiguous one-to-one V1 ledger equivalent. */
export function buildStageForPipelineStage(stage?: string): BuildStageId | undefined {
  return stage === undefined ? undefined : PIPELINE_STAGE_TO_BUILD_STAGE[stage];
}
