// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { WorkflowStepper } from "./WorkflowStepper";

const roots: Root[] = [];

afterEach(async () => {
  await act(async () => {
    roots.splice(0).forEach((root) => root.unmount());
  });
  document.body.replaceChildren();
});

describe("WorkflowStepper", () => {
  it("renders the five-step workflow with active, completed and blocked semantics", async () => {
    const onStepSelect = vi.fn();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => {
      root.render(
        <WorkflowStepper
          currentStep="INSPECT"
          visitedSteps={["SOURCE", "INSPECT"]}
          completedSteps={["SOURCE"]}
          onStepSelect={onStepSelect}
        />,
      );
    });

    const list = container.querySelector("ol");
    const buttons = Array.from(container.querySelectorAll<HTMLButtonElement>("ol button"));
    expect(list).not.toBeNull();
    expect(buttons).toHaveLength(5);
    expect(buttons.map((button) => button.textContent)).toEqual([
      "✓SourceSelect input files",
      "2InspectValidate & preview",
      "3BuildConvert & validate",
      "4Review OutputVerify results",
      "5DownloadGet your results",
    ]);
    expect(buttons[1].getAttribute("aria-current")).toBe("step");
    expect(buttons[0].querySelector(".workflow-stepper__marker")?.textContent).toBe("✓");
    expect(buttons[2].disabled).toBe(true);
    expect(buttons[2].tabIndex).toBe(-1);

    buttons[2].click();
    expect(onStepSelect).not.toHaveBeenCalled();

    buttons[0].click();
    expect(onStepSelect).toHaveBeenCalledWith("SOURCE");
  });
});
