import type { ReactNode } from "react";
import {
  SourceInspectionPanel,
  type SourceInspectionMetrics,
} from "./SourceInspectionPanel";
import {
  ValidationPanel,
  type InspectValidationCheck,
} from "./ValidationPanel";
import "./inspect.css";

export interface InspectProfileRequirement {
  readonly id: string;
  readonly label: string;
  readonly expected: string;
  readonly actual: string;
  readonly status: "PASS" | "ERROR" | "UNAVAILABLE";
  readonly repair: string;
}

export interface InspectStepProps {
  readonly viewport: ReactNode;
  readonly sourceMetrics: SourceInspectionMetrics;
  readonly validationChecks: readonly InspectValidationCheck[];
  readonly profileRequirements?: readonly InspectProfileRequirement[];
  readonly animationPlayer?: ReactNode;
  readonly debugOverlays?: ReactNode;
  readonly canContinue: boolean;
  readonly continueLabel?: "Continue to Animation Mapping" | "Continue to Build";
  readonly wideViewport?: boolean;
  readonly onBack: () => void;
  readonly onContinue: () => void;
  readonly onSelectValidationCheck?: (check: InspectValidationCheck) => void;
}

export function InspectStep({
  viewport,
  sourceMetrics,
  validationChecks,
  profileRequirements = [],
  animationPlayer,
  debugOverlays,
  canContinue,
  continueLabel = "Continue to Build",
  wideViewport = false,
  onBack,
  onContinue,
  onSelectValidationCheck,
}: InspectStepProps) {
  return (
    <section
      className="inspect-step"
      data-wide-viewport={wideViewport}
      aria-labelledby="inspect-step-heading"
    >
      <header className="inspect-step__heading">
        <div>
          <p className="inspect-step__eyebrow">Inspect</p>
          <h1 id="inspect-step-heading">Inspect</h1>
        </div>
        <p>Review the source model and validation evidence before building.</p>
      </header>

      <div className="inspect-step__workspace">
        <div className="inspect-step__preview">
          <div className="inspect-step__viewport">
            {viewport}
            {debugOverlays ? <div className="inspect-step__overlays">{debugOverlays}</div> : null}
          </div>
          {animationPlayer ? <div className="inspect-step__animation">{animationPlayer}</div> : null}
        </div>

        <aside className="inspect-step__evidence" aria-label="Source inspection evidence">
          <SourceInspectionPanel metrics={sourceMetrics} />
          {profileRequirements.length > 0 ? (
            <section
              className="inspect-panel inspect-profile-requirements"
              aria-labelledby="inspect-profile-requirements-heading"
            >
              <header className="inspect-panel__header">
                <h2 id="inspect-profile-requirements-heading">Creature H1 profile</h2>
                <span>Pre-build requirements</span>
              </header>
              <ul>
                {profileRequirements.map((requirement) => (
                  <li key={requirement.id} data-status={requirement.status}>
                    <header>
                      <strong>{requirement.label}</strong>
                      <span>{requirement.status}</span>
                    </header>
                    <p>
                      Expected: {requirement.expected}
                      <br />
                      Actual: {requirement.actual}
                    </p>
                    {requirement.status === "ERROR" ? (
                      <small>Repair: {requirement.repair}</small>
                    ) : null}
                  </li>
                ))}
              </ul>
            </section>
          ) : null}
          <ValidationPanel checks={validationChecks} onSelectCheck={onSelectValidationCheck} />
        </aside>
      </div>

      <footer className="inspect-step__actions">
        <p aria-live="polite">
          {canContinue
            ? "Inspection evidence is ready."
            : "Required inspection evidence is not ready."}
        </p>
        <button type="button" className="inspect-step__button inspect-step__button--secondary" onClick={onBack}>
          Back to Source
        </button>
        <button
          type="button"
          className="inspect-step__button inspect-step__button--primary"
          onClick={onContinue}
          disabled={!canContinue}
        >
          {continueLabel}
        </button>
      </footer>
    </section>
  );
}
