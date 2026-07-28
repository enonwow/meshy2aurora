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

  it("keeps Create & edit as a sub-mode of Animation Mapping instead of a seventh workflow step", () => {
    expect(getWorkflowStepsForTarget("CREATURE")).toHaveLength(6);
    expect(getWorkflowStepsForTarget("CREATURE")).toContain("ANIMATION_MAPPING");
    expect(getWorkflowStepsForTarget("CREATURE")).not.toContain("ANIMATION_STUDIO");
    expect(getWorkflowStepsForTarget("CREATURE")).not.toContain("CREATE_EDIT");
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
