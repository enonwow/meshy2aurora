import type { StudioSessionState } from "./studioSession";
import {
  compareWorkflowSteps,
  getWorkflowStepsForTarget,
  type WorkflowStep,
} from "./workflow";
import { isAnimationMappingCurrentV1 } from "../features/animation-mapping/state";

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

export function canContinueToAnimationMapping(state: StudioSessionState): boolean {
  return state.target === "CREATURE"
    && Boolean(state.source && state.appearance)
    && state.sourceInspection?.revision === state.revision
    && state.appearanceInspection?.revision === state.revision
    && Boolean(state.source?.sha256);
}

export function canContinueFromAnimationMapping(state: StudioSessionState): boolean {
  return isAnimationMappingCurrentV1(state)
    && state.animationMappingValidation?.revision === state.revision
    && state.animationMappingValidation.value.authoringRevision
      === state.animationMapping?.value.authoringRevision
    && state.animationMappingValidation.value.status === "READY"
    && /^[0-9a-f]{64}$/.test(
      state.animationMappingValidation.value.authoringFingerprintSha256 ?? "",
    );
}

export function canNavigateToStep(state: StudioSessionState, step: WorkflowStep): boolean {
  return getWorkflowStepsForTarget(state.target).includes(step)
    && compareWorkflowSteps(step, state.lastAvailableStep) <= 0;
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
  return getWorkflowStepsForTarget(state.target)
    .filter((step) => canNavigateToStep(state, step));
}
