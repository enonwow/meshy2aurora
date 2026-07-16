// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { BUILD_STAGE_IDS, BuildStep, type BuildStepProps } from "./BuildStep";

const roots: Root[] = [];

async function render(element: React.ReactNode) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  await act(async () => root.render(element));
  return container;
}

function callbacks(): Pick<BuildStepProps, "onBack" | "onBuild" | "onRetry" | "onCancel"> {
  return {
    onBack: vi.fn(),
    onBuild: vi.fn(),
    onRetry: vi.fn(),
    onCancel: vi.fn(),
  };
}

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
});

describe("BuildStep", () => {
  it("renders the six-stage idle ledger without a viewport or invented progress", async () => {
    const handlers = callbacks();
    const container = await render(
      <BuildStep
        {...handlers}
        state={{
          kind: "IDLE",
          inputs: {
            source: {
              name: "creature.glb",
              byteLength: 2_486_912,
              sha256: "a".repeat(64),
              inspectionStatus: "INSPECTED",
            },
            appearance: {
              name: "appearance.2da",
              byteLength: 1_024,
              sha256: "b".repeat(64),
              inspectionStatus: "INSPECTED",
            },
          },
        }}
        canGoBack
        canBuild={false}
        canRetry={false}
        canCancel={false}
      />,
    );

    expect(container.querySelectorAll(".build-ledger__stage")).toHaveLength(BUILD_STAGE_IDS.length);
    expect(container.querySelectorAll('[data-status="PENDING"]')).toHaveLength(6);
    expect(container.textContent).toContain("Canonical binary readback");
    expect(container.textContent).toContain("creature.glb");
    expect(container.textContent).toContain("2,486,912 bytes");
    expect(container.textContent).toContain("appearance.2da");
    expect(container.textContent).toContain("a".repeat(64));
    expect(container.querySelectorAll('[data-status="INSPECTED"]')).toHaveLength(2);
    expect(container.textContent).not.toMatch(/\d+%|estimated|remaining|viewport/i);
    const build = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent?.includes("Build Package"));
    expect(build?.disabled).toBe(true);
  });

  it("shows only explicit running evidence and routes Cancel", async () => {
    const handlers = callbacks();
    const container = await render(
      <BuildStep
        {...handlers}
        state={{
          kind: "RUNNING",
          completedStages: ["INGEST_SOURCE", "NORMALIZE_CANONICAL_IR"],
          activeStage: "WRITE_BINARY_MDL",
          message: "Worker accepted the binary writer request.",
        }}
        canGoBack={false}
        canBuild={false}
        canRetry={false}
        canCancel
      />,
    );

    expect(container.querySelectorAll('[data-status="COMPLETE"]')).toHaveLength(2);
    expect(container.querySelector('[data-status="RUNNING"]')?.textContent).toContain("Write binary MDL");
    expect(container.textContent).toContain("Worker accepted the binary writer request.");
    expect(container.textContent).toContain("Running — indeterminate");
    expect(container.textContent).not.toMatch(/\d+%|estimated|remaining/i);

    const cancel = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent?.includes("Cancel Build"));
    await act(async () => cancel?.click());
    expect(handlers.onCancel).toHaveBeenCalledOnce();
  });

  it("marks the real failed stage, keeps other unfinished stages Not run and routes Retry", async () => {
    const handlers = callbacks();
    const openFailureDiagnostics = vi.fn();
    const container = await render(
      <BuildStep
        {...handlers}
        state={{
          kind: "FAILED",
          completedStages: ["INGEST_SOURCE"],
          failedStage: "NORMALIZE_CANONICAL_IR",
          failure: {
            stage: "PROFILE",
            code: "M3A-PROFILE-INELIGIBLE",
            path: "conversion.report",
            message: "Canonical normalization was rejected.",
          },
        }}
        canGoBack
        canBuild={false}
        canRetry
        canCancel={false}
        failureDiagnostics={{
          canOpen: true,
          onOpen: openFailureDiagnostics,
          reportPackage: {
            status: "UNAVAILABLE",
            reason: "FE-D7 report packaging is not implemented.",
          },
        }}
      />,
    );

    expect(container.querySelector('[data-status="FAILED"]')?.textContent).toContain("M3A-PROFILE-INELIGIBLE");
    expect(container.querySelectorAll('[data-status="NOT_RUN"]')).toHaveLength(4);
    expect(container.textContent).toContain("Canonical normalization was rejected.");
    expect(container.textContent).toContain("Failure diagnostics");
    expect(container.textContent).toContain("PROFILE");
    expect(container.textContent).toContain("conversion.report");
    expect(container.textContent).toContain("Debug report package: Unavailable");
    expect(container.textContent).toContain("FE-D7 report packaging is not implemented.");
    expect(container.textContent).not.toContain("Generate Debug Report");

    const retry = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent?.includes("Retry Build"));
    const back = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent?.includes("Back to Inspect"));
    const diagnostics = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent?.includes("Open failure diagnostics"));
    await act(async () => {
      retry?.click();
      back?.click();
      diagnostics?.click();
    });
    expect(handlers.onRetry).toHaveBeenCalledOnce();
    expect(handlers.onBack).toHaveBeenCalledOnce();
    expect(openFailureDiagnostics).toHaveBeenCalledOnce();
  });

  it("shows unavailable instead of inventing an idle input summary", async () => {
    const container = await render(
      <BuildStep
        {...callbacks()}
        state={{ kind: "IDLE" }}
        canGoBack
        canBuild={false}
        canRetry={false}
        canCancel={false}
      />,
    );
    expect(container.textContent).toContain("Input inspection summary is unavailable.");
    expect(container.textContent).not.toContain("INSPECTED");
  });
});
