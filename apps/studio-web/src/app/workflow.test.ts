import { describe, expect, it } from "vitest";
import {
  WORKFLOW_STEPS,
  compareWorkflowSteps,
  getWorkflowStepsForTarget,
  isWorkflowStep,
} from "./workflow";

describe("Studio workflow", () => {
  it("defines the six ordered creature steps", () => {
    expect(WORKFLOW_STEPS).toEqual([
      "SOURCE",
      "INSPECT",
      "ANIMATION_MAPPING",
      "BUILD",
      "REVIEW",
      "DOWNLOAD",
    ]);
    expect(getWorkflowStepsForTarget("CREATURE")).toEqual(WORKFLOW_STEPS);
    expect(getWorkflowStepsForTarget("PLACEABLE")).toEqual([
      "SOURCE", "INSPECT", "BUILD", "REVIEW", "DOWNLOAD",
    ]);
    expect(getWorkflowStepsForTarget("TILE")).not.toContain("ANIMATION_MAPPING");
  });

  it("compares steps using workflow order", () => {
    expect(compareWorkflowSteps("SOURCE", "SOURCE")).toBe(0);
    expect(compareWorkflowSteps("INSPECT", "ANIMATION_MAPPING")).toBeLessThan(0);
    expect(compareWorkflowSteps("ANIMATION_MAPPING", "BUILD")).toBeLessThan(0);
    expect(compareWorkflowSteps("INSPECT", "BUILD")).toBeLessThan(0);
    expect(compareWorkflowSteps("DOWNLOAD", "REVIEW")).toBeGreaterThan(0);
  });

  it("recognizes only workflow step values", () => {
    expect(isWorkflowStep("REVIEW")).toBe(true);
    expect(isWorkflowStep("RESULT")).toBe(false);
    expect(isWorkflowStep(undefined)).toBe(false);
  });
});
