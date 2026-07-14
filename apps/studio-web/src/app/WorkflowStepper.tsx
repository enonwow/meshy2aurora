import type { WorkflowStep } from "./workflow";

export type WorkflowStepId = WorkflowStep;

export interface WorkflowStepperProps {
  currentStep: WorkflowStepId;
  visitedSteps: readonly WorkflowStepId[];
  completedSteps: readonly WorkflowStepId[];
  blockedSteps?: readonly WorkflowStepId[];
  onStepSelect: (step: WorkflowStepId) => void;
}

interface WorkflowStepDefinition {
  id: WorkflowStepId;
  label: string;
  description: string;
}

const WORKFLOW_STEPS: readonly WorkflowStepDefinition[] = [
  { id: "SOURCE", label: "Source", description: "Select input files" },
  { id: "INSPECT", label: "Inspect", description: "Validate & preview" },
  { id: "BUILD", label: "Build", description: "Convert & validate" },
  { id: "REVIEW", label: "Review Output", description: "Verify results" },
  { id: "DOWNLOAD", label: "Download", description: "Get your results" },
];

function LockIcon() {
  return (
    <svg aria-hidden="true" viewBox="0 0 16 16" fill="none">
      <rect x="3.5" y="7" width="9" height="7" rx="1.5" fill="currentColor" />
      <path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  );
}

export function WorkflowStepper({
  currentStep,
  visitedSteps,
  completedSteps,
  blockedSteps = [],
  onStepSelect,
}: WorkflowStepperProps) {
  const visited = new Set(visitedSteps);
  const completed = new Set(completedSteps);
  const explicitlyBlocked = new Set(blockedSteps);

  return (
    <nav className="workflow-stepper" aria-label="Conversion workflow">
      <ol className="workflow-stepper__list">
        {WORKFLOW_STEPS.map((step, index) => {
          const isCurrent = step.id === currentStep;
          const isCompleted = completed.has(step.id);
          const isVisited = visited.has(step.id) || isCurrent || isCompleted;
          const isBlocked = explicitlyBlocked.has(step.id) || !isVisited;
          const state = isCurrent ? "current" : isCompleted ? "completed" : isBlocked ? "blocked" : "visited";
          const statusLabel = isCurrent
            ? "current step"
            : isCompleted
              ? "completed"
              : isBlocked
                ? "unavailable"
                : "visited";

          return (
            <li key={step.id} className="workflow-stepper__item" data-state={state}>
              <button
                type="button"
                className="workflow-stepper__step"
                aria-current={isCurrent ? "step" : undefined}
                aria-label={`${step.label}: ${step.description}, ${statusLabel}`}
                disabled={isBlocked}
                tabIndex={isBlocked ? -1 : undefined}
                onClick={() => {
                  if (isVisited && !isBlocked) onStepSelect(step.id);
                }}
              >
                <span className="workflow-stepper__marker" aria-hidden="true">
                  {isCompleted ? "✓" : index + 1}
                </span>
                <span className="workflow-stepper__copy">
                  <span className="workflow-stepper__label">{step.label}</span>
                  <span className="workflow-stepper__description">{step.description}</span>
                </span>
                {isBlocked && <span className="workflow-stepper__lock"><LockIcon /></span>}
              </button>
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
