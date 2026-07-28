export type AnimationStudioAutosaveStateV1 =
  | { readonly kind: "IDLE" }
  | { readonly kind: "SAVING" }
  | { readonly kind: "SAVED" }
  | { readonly kind: "ERROR"; readonly message: string };

export function AnimationStudioAutosaveStatus({
  state,
}: {
  state: AnimationStudioAutosaveStateV1;
}) {
  const label = state.kind === "SAVED"
    ? "Autosaved"
    : state.kind === "SAVING"
      ? "Saving…"
      : state.kind === "ERROR"
        ? `Autosave failed: ${state.message}`
        : "Local project";
  return (
    <span
      className="animation-studio-autosave"
      data-state={state.kind}
      role={state.kind === "ERROR" ? "alert" : "status"}
      aria-live={state.kind === "ERROR" ? "assertive" : "polite"}
      aria-atomic="true"
      aria-busy={state.kind === "SAVING" || undefined}
    >
      {label}
    </span>
  );
}
