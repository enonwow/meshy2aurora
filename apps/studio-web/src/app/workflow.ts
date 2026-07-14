export const WORKFLOW_STEPS = [
  "SOURCE",
  "INSPECT",
  "BUILD",
  "REVIEW",
  "DOWNLOAD",
] as const;

export type WorkflowStep = (typeof WORKFLOW_STEPS)[number];

const WORKFLOW_STEP_INDEX: Readonly<Record<WorkflowStep, number>> = {
  SOURCE: 0,
  INSPECT: 1,
  BUILD: 2,
  REVIEW: 3,
  DOWNLOAD: 4,
};

export function compareWorkflowSteps(left: WorkflowStep, right: WorkflowStep): number {
  return WORKFLOW_STEP_INDEX[left] - WORKFLOW_STEP_INDEX[right];
}

export function isWorkflowStep(value: unknown): value is WorkflowStep {
  return typeof value === "string" && WORKFLOW_STEPS.some((step) => step === value);
}
