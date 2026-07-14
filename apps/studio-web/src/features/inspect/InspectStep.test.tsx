// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { InspectStep } from "./InspectStep";
import { SourceInspectionPanel } from "./SourceInspectionPanel";
import { ValidationPanel, type InspectValidationCheck } from "./ValidationPanel";

const roots: Root[] = [];

async function render(element: React.ReactNode) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  await act(async () => root.render(element));
  return container;
}

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
});

describe("SourceInspectionPanel", () => {
  it("renders only supplied source values and marks missing evidence unavailable", async () => {
    const container = await render(
      <SourceInspectionPanel metrics={{ meshCount: 8, vertexCount: 24_681, animationClipCount: 0 }} />,
    );

    expect(container.textContent).toContain("8");
    expect(container.textContent).toContain("24,681");
    expect(container.textContent).toContain("Animation clips0");
    expect(container.querySelectorAll('[data-available="false"]')).toHaveLength(4);
    expect(container.textContent?.match(/Unavailable/g)).toHaveLength(4);
  });
});

describe("ValidationPanel", () => {
  it("renders explicit evidence statuses without deriving a synthetic pass", async () => {
    const checks: InspectValidationCheck[] = [
      { id: "structure", label: "GLB structure", status: "PASS", evidence: { code: "GLB-STRUCTURE-OK" } },
      {
        id: "textures",
        label: "Materials and textures",
        status: "WARNING",
        evidence: { code: "GLB-TEXTURE-NPOT", message: "Texture dimensions require review.", path: "images[2]" },
      },
      {
        id: "topology",
        label: "Topology",
        status: "ERROR",
        evidence: { code: "GLB-TOPOLOGY-INVALID", message: "Unsupported primitive topology." },
      },
      { id: "appearance", label: "Base appearance schema", status: "UNAVAILABLE" },
    ];
    const onSelectCheck = vi.fn();
    const container = await render(<ValidationPanel checks={checks} onSelectCheck={onSelectCheck} />);

    expect(container.querySelector('[data-status="PASS"]')?.textContent).toContain("GLB-STRUCTURE-OK");
    expect(container.querySelector('[data-status="WARNING"]')?.textContent).toContain("Path: images[2]");
    expect(container.querySelector('[data-status="ERROR"]')?.textContent).toContain("GLB-TOPOLOGY-INVALID");
    expect(container.querySelector('[data-status="UNAVAILABLE"]')?.textContent).toContain("No evidence supplied.");

    const details = container.querySelectorAll<HTMLButtonElement>("button");
    await act(async () => details[1]?.click());
    expect(onSelectCheck).toHaveBeenCalledWith(checks[1]);
  });

  it("shows evidence unavailable instead of PASS when no checks are supplied", async () => {
    const container = await render(<ValidationPanel checks={[]} />);
    expect(container.textContent).toContain("Validation evidence is unavailable.");
    expect(container.textContent).not.toContain("Pass");
  });
});

describe("InspectStep", () => {
  it("composes caller-owned viewport and gates Continue exclusively through canContinue", async () => {
    const onBack = vi.fn();
    const onContinue = vi.fn();
    const container = await render(
      <InspectStep
        viewport={<div data-testid="real-viewport">Source renderer</div>}
        sourceMetrics={{ meshCount: 2 }}
        validationChecks={[]}
        animationPlayer={<div>Animation inventory</div>}
        debugOverlays={<button type="button">Overlays</button>}
        canContinue={false}
        onBack={onBack}
        onContinue={onContinue}
      />,
    );

    expect(container.querySelector('[data-testid="real-viewport"]')?.textContent).toBe("Source renderer");
    expect(container.textContent).toContain("Animation inventory");
    expect(container.textContent).not.toContain("Binary Readback");

    const continueButton = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent?.includes("Continue to Build"));
    const backButton = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent?.includes("Back to Source"));
    expect(continueButton?.disabled).toBe(true);

    await act(async () => {
      continueButton?.click();
      backButton?.click();
    });
    expect(onContinue).not.toHaveBeenCalled();
    expect(onBack).toHaveBeenCalledOnce();
  });
});
