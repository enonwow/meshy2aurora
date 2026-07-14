import type { StudioSessionState } from "./studioSession";
import { compareWorkflowSteps, WORKFLOW_STEPS, type WorkflowStep } from "./workflow";

export type InputReadiness = "EMPTY" | "PARTIAL" | "READY";
export type WorkflowStepStatus = "LOCKED" | "AVAILABLE" | "ACTIVE" | "COMPLETE";

export function getInputReadiness(state: StudioSessionState): InputReadiness {
  if (state.source && state.appearance) return "READY";
  if (state.source || state.appearance) return "PARTIAL";
  return "EMPTY";
}

export function canContinueToInspect(state: StudioSessionState): boolean {
  return Boolean(state.source && state.appearance);
}

export function canNavigateToStep(state: StudioSessionState, step: WorkflowStep): boolean {
  return compareWorkflowSteps(step, state.lastAvailableStep) <= 0;
}

export function getWorkflowStepStatus(
  state: StudioSessionState,
  step: WorkflowStep,
): WorkflowStepStatus {
  if (step === state.currentStep) return "ACTIVE";
  if (!canNavigateToStep(state, step)) return "LOCKED";
  if (compareWorkflowSteps(step, state.currentStep) < 0) return "COMPLETE";
  return "AVAILABLE";
}

export function getUnlockedWorkflowSteps(state: StudioSessionState): readonly WorkflowStep[] {
  return WORKFLOW_STEPS.filter((step) => canNavigateToStep(state, step));
}
