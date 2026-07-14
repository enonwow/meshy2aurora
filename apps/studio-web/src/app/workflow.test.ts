import { describe, expect, it } from "vitest";
import { WORKFLOW_STEPS, compareWorkflowSteps, isWorkflowStep } from "./workflow";

describe("Studio workflow", () => {
  it("defines the five ordered V1 steps", () => {
    expect(WORKFLOW_STEPS).toEqual([
      "SOURCE",
      "INSPECT",
      "BUILD",
      "REVIEW",
      "DOWNLOAD",
    ]);
  });

  it("compares steps using workflow order", () => {
    expect(compareWorkflowSteps("SOURCE", "SOURCE")).toBe(0);
    expect(compareWorkflowSteps("INSPECT", "BUILD")).toBeLessThan(0);
    expect(compareWorkflowSteps("DOWNLOAD", "REVIEW")).toBeGreaterThan(0);
  });

  it("recognizes only workflow step values", () => {
    expect(isWorkflowStep("REVIEW")).toBe(true);
    expect(isWorkflowStep("RESULT")).toBe(false);
    expect(isWorkflowStep(undefined)).toBe(false);
  });
});
