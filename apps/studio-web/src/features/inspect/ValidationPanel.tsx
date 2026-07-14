export type InspectValidationStatus = "PASS" | "WARNING" | "ERROR" | "INFO" | "UNAVAILABLE";

export interface InspectValidationEvidence {
  readonly code: string;
  readonly message?: string;
  readonly path?: string;
}

interface InspectValidationCheckBase {
  readonly id: string;
  readonly label: string;
}

export type InspectValidationCheck =
  | (InspectValidationCheckBase & {
      readonly status: Exclude<InspectValidationStatus, "UNAVAILABLE">;
      readonly evidence: InspectValidationEvidence;
    })
  | (InspectValidationCheckBase & {
      readonly status: "UNAVAILABLE";
      readonly evidence?: InspectValidationEvidence;
    });

export interface InspectValidationPanelProps {
  readonly checks: readonly InspectValidationCheck[];
  readonly onSelectCheck?: (check: InspectValidationCheck) => void;
}

const STATUS_LABEL: Readonly<Record<InspectValidationStatus, string>> = {
  PASS: "Pass",
  WARNING: "Warning",
  ERROR: "Error",
  INFO: "Info",
  UNAVAILABLE: "Unavailable",
};

export function ValidationPanel({ checks, onSelectCheck }: InspectValidationPanelProps) {
  return (
    <section className="inspect-panel inspect-validation" aria-labelledby="inspect-validation-heading">
      <header className="inspect-panel__header">
        <h2 id="inspect-validation-heading">Validation</h2>
        <span>{checks.length} {checks.length === 1 ? "check" : "checks"}</span>
      </header>
      {checks.length === 0 ? (
        <p className="inspect-validation__empty">Validation evidence is unavailable.</p>
      ) : (
        <ul className="inspect-validation__list">
          {checks.map((check) => (
            <li className="inspect-validation__check" key={check.id} data-status={check.status}>
              <div className="inspect-validation__check-heading">
                <strong>{check.label}</strong>
                <span className="inspect-validation__status">{STATUS_LABEL[check.status]}</span>
              </div>
              {check.evidence ? (
                <div className="inspect-validation__evidence">
                  <code>{check.evidence.code}</code>
                  {check.evidence.message ? <span>{check.evidence.message}</span> : null}
                  {check.evidence.path ? <span>Path: {check.evidence.path}</span> : null}
                </div>
              ) : (
                <span className="inspect-validation__no-evidence">No evidence supplied.</span>
              )}
              {onSelectCheck ? (
                <button type="button" className="inspect-validation__action" onClick={() => onSelectCheck(check)}>
                  Show details
                </button>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
