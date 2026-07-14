// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import { createInitialStudioSession, studioSessionReducer } from "./studioSession";
import {
  canContinueToInspect,
  canNavigateToStep,
  getInputReadiness,
  getWorkflowStepStatus,
} from "./studioSelectors";

const file = (name: string) => new File([name], name);

describe("Studio session selectors", () => {
  it("reports empty, partial and ready input states without treating selection as validation", () => {
    const initial = createInitialStudioSession();
    const partial = studioSessionReducer(initial, { type: "SOURCE_SELECTED", file: file("source.glb") });
    const ready = studioSessionReducer(partial, { type: "APPEARANCE_SELECTED", file: file("appearance.2da") });

    expect(getInputReadiness(initial)).toBe("EMPTY");
    expect(getInputReadiness(partial)).toBe("PARTIAL");
    expect(getInputReadiness(ready)).toBe("READY");
    expect(ready.source?.parse.kind).toBe("NOT_STARTED");
    expect(canContinueToInspect(initial)).toBe(false);
    expect(canContinueToInspect(partial)).toBe(false);
    expect(canContinueToInspect(ready)).toBe(true);
  });

  it("exposes locked, available, active and complete step states", () => {
    const sourceReady = studioSessionReducer(
      studioSessionReducer(createInitialStudioSession(), { type: "SOURCE_SELECTED", file: file("source.glb") }),
      { type: "APPEARANCE_SELECTED", file: file("appearance.2da") },
    );
    const inspect = studioSessionReducer(sourceReady, { type: "CONTINUE_TO_INSPECT" });

    expect(getWorkflowStepStatus(inspect, "SOURCE")).toBe("COMPLETE");
    expect(getWorkflowStepStatus(inspect, "INSPECT")).toBe("ACTIVE");
    expect(getWorkflowStepStatus(inspect, "BUILD")).toBe("LOCKED");
    expect(canNavigateToStep(inspect, "SOURCE")).toBe(true);
    expect(canNavigateToStep(inspect, "INSPECT")).toBe(true);
    expect(canNavigateToStep(inspect, "BUILD")).toBe(false);

    const source = studioSessionReducer(inspect, { type: "NAVIGATE", step: "SOURCE" });
    expect(getWorkflowStepStatus(source, "SOURCE")).toBe("ACTIVE");
    expect(getWorkflowStepStatus(source, "INSPECT")).toBe("AVAILABLE");
  });
});
