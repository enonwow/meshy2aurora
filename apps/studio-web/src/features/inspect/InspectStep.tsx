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

export interface InspectStepProps {
  readonly viewport: ReactNode;
  readonly sourceMetrics: SourceInspectionMetrics;
  readonly validationChecks: readonly InspectValidationCheck[];
  readonly animationPlayer?: ReactNode;
  readonly debugOverlays?: ReactNode;
  readonly canContinue: boolean;
  readonly onBack: () => void;
  readonly onContinue: () => void;
  readonly onSelectValidationCheck?: (check: InspectValidationCheck) => void;
}

export function InspectStep({
  viewport,
  sourceMetrics,
  validationChecks,
  animationPlayer,
  debugOverlays,
  canContinue,
  onBack,
  onContinue,
  onSelectValidationCheck,
}: InspectStepProps) {
  return (
    <section className="inspect-step" aria-labelledby="inspect-step-heading">
      <header className="inspect-step__heading">
        <div>
          <p className="inspect-step__eyebrow">Step 2</p>
          <h1 id="inspect-step-heading">Inspect source</h1>
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
          Continue to Build
        </button>
      </footer>
    </section>
  );
}
