import type {
  AnimationCatalogRowV1,
  CreatureAnimationMappingStatusV1,
} from "./types";

export type AnimationMappingSaveStateV1 =
  | { kind: "IDLE" }
  | { kind: "SAVING" }
  | { kind: "SAVED" }
  | { kind: "ERROR"; message: string };

export interface AnimationMappingStatusBarProps {
  rows: readonly AnimationCatalogRowV1[];
  status: CreatureAnimationMappingStatusV1;
  saveState: AnimationMappingSaveStateV1;
  canonicalValidationPending?: boolean;
}

export function AnimationMappingStatusBar({
  rows,
  status,
  saveState,
  canonicalValidationPending = false,
}: AnimationMappingStatusBarProps) {
  const baseRows = rows.filter(({ kind }) => kind === "BASE");
  const mapped = baseRows.filter(({ status }) => status === "MAPPED").length;
  const review = baseRows.filter(({ status }) => status === "NEEDS_REVIEW").length;
  const blockers = baseRows.filter(({ status }) => status === "BLOCKED").length;
  const custom = rows.filter(({ kind }) => kind === "CUSTOM").length;
  const saveLabel = saveState.kind === "ERROR"
    ? `Save error: ${saveState.message}`
    : saveState.kind === "SAVING"
      ? "Saving"
      : saveState.kind === "SAVED"
        ? "Saved"
        : "Not saved yet";

  return (
    <div className="animation-mapping-status" aria-live="polite">
      <strong data-status={canonicalValidationPending ? "VALIDATING" : status}>
        {canonicalValidationPending ? "VALIDATING WITH CORE" : status.replace("_", " ")}
      </strong>
      <span>{mapped}/42 mapped</span>
      <span>{review} review</span>
      <span>{blockers} blockers</span>
      <span>{custom} custom</span>
      <span data-save-state={saveState.kind}>{saveLabel}</span>
    </div>
  );
}
