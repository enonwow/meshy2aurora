export const WORKFLOW_STEPS = [
  "SOURCE",
  "INSPECT",
  "ANIMATION_MAPPING",
  "BUILD",
  "REVIEW",
  "DOWNLOAD",
] as const;

export type WorkflowStep = (typeof WORKFLOW_STEPS)[number];

const WORKFLOW_STEP_INDEX: Readonly<Record<WorkflowStep, number>> = {
  SOURCE: 0,
  INSPECT: 1,
  ANIMATION_MAPPING: 2,
  BUILD: 3,
  REVIEW: 4,
  DOWNLOAD: 5,
};

export type WorkflowTarget = "CREATURE" | "PLACEABLE" | "TILE";

const NON_CREATURE_WORKFLOW_STEPS = WORKFLOW_STEPS.filter(
  (step) => step !== "ANIMATION_MAPPING",
);

export function getWorkflowStepsForTarget(
  target: WorkflowTarget,
): readonly WorkflowStep[] {
  return target === "CREATURE" ? WORKFLOW_STEPS : NON_CREATURE_WORKFLOW_STEPS;
}

export function compareWorkflowSteps(left: WorkflowStep, right: WorkflowStep): number {
  return WORKFLOW_STEP_INDEX[left] - WORKFLOW_STEP_INDEX[right];
}

export function isWorkflowStep(value: unknown): value is WorkflowStep {
  return typeof value === "string" && WORKFLOW_STEPS.some((step) => step === value);
}
