import "./ReviewWorkflowActions.css";

interface ReviewWorkflowActionsProps {
  readonly canContinue: boolean;
  readonly blockedReason?: string;
  readonly onBack: () => void;
  readonly onContinue: () => void;
}

export function ReviewWorkflowActions({
  canContinue,
  blockedReason,
  onBack,
  onContinue,
}: ReviewWorkflowActionsProps) {
  return (
    <footer
      className="review-workflow-actions"
      aria-label="Review workflow actions"
    >
      <button type="button" onClick={onBack}>
        Back to Build
      </button>
      <div>
        {!canContinue && blockedReason ? (
          <span role="status">{blockedReason}</span>
        ) : null}
        <button
          type="button"
          className="review-workflow-actions__primary"
          disabled={!canContinue}
          onClick={onContinue}
        >
          Continue to Download
        </button>
      </div>
    </footer>
  );
}
