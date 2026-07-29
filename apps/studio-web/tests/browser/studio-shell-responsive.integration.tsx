import { afterEach, describe, expect, it, vi } from "vitest";
import { page } from "vitest/browser";
import { createRoot, type Root } from "react-dom/client";
import { App } from "../../src/App";
import { ImportAnimationFromModelDialog } from "../../src/features/animation-editor/ImportAnimationFromModelDialog";
import { ProjectManagerDialog } from "../../src/features/project/ProjectManagerDialog";
import { createMeshy2AuroraProjectV1 } from "../../src/features/project";
import "../../src/styles.css";

const roots: Root[] = [];

async function frame() {
  await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
}

afterEach(() => {
  roots.splice(0).forEach((root) => root.unmount());
  document.body.replaceChildren();
});

describe("Studio shell responsive browser contract", () => {
  it("contains horizontal overflow and preserves Source focus at supported viewports", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    root.render(<App />);
    await frame();

    for (const [width, height] of [
      [1600, 1000],
      [1366, 768],
      [1024, 768],
    ] as const) {
      await page.viewport(width, height);
      await frame();
      expect(document.documentElement.scrollWidth).toBeLessThanOrEqual(
        document.documentElement.clientWidth + 1,
      );
      const project = document.querySelector<HTMLElement>('[aria-label="Project status"]');
      const source = document.querySelector<HTMLInputElement>(
        'input[aria-label="Meshy model file"]',
      );
      expect(project).not.toBeNull();
      expect(source).not.toBeNull();
      source!.focus();
      expect(document.activeElement).toBe(source);
    }

    // A 1024 Ã— 768 browser at 200% zoom exposes an effective 512 Ã— 384
    // CSS viewport and activates the same responsive media queries.
    await page.viewport(512, 384);
    await frame();
    expect(document.documentElement.scrollWidth).toBeLessThanOrEqual(
      document.documentElement.clientWidth + 1,
    );
    expect(document.querySelector(".studio-shell__workflow-rail")?.scrollWidth)
      .toBeGreaterThan(0);
  });

  it("keeps the animation import modal above the shell, inside the viewport and focus-trapped", async () => {
    await page.viewport(512, 384);
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    root.render(
      <ImportAnimationFromModelDialog
        currentRig={[]}
        onInspect={vi.fn()}
        onImport={vi.fn()}
        onClose={vi.fn()}
      />,
    );
    await frame();

    const dialog = document.querySelector<HTMLElement>('[role="dialog"]')!;
    const card = dialog.querySelector<HTMLElement>(".animation-import-dialog__card")!;
    const bounds = card.getBoundingClientRect();
    expect(getComputedStyle(dialog).position).toBe("fixed");
    expect(Number(getComputedStyle(dialog).zIndex)).toBeGreaterThanOrEqual(100);
    expect(bounds.left).toBeGreaterThanOrEqual(0);
    expect(bounds.top).toBeGreaterThanOrEqual(0);
    expect(bounds.right).toBeLessThanOrEqual(window.innerWidth + 1);
    expect(bounds.bottom).toBeLessThanOrEqual(window.innerHeight + 1);

    const focusable = Array.from(card.querySelectorAll<HTMLElement>(
      "button:not(:disabled), input:not(:disabled), select:not(:disabled)",
    ));
    const first = focusable[0]!;
    const last = focusable.at(-1)!;
    last.focus();
    dialog.dispatchEvent(new KeyboardEvent("keydown", {
      key: "Tab",
      bubbles: true,
    }));
    expect(document.activeElement).toBe(first);
  });

  it("keeps the project manager inside the effective 200% viewport and focus-trapped", async () => {
    await page.viewport(512, 384);
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    root.render(
      <ProjectManagerDialog
        project={createMeshy2AuroraProjectV1({
          projectId: "responsive-project-01",
          now: "2026-07-28T12:00:00.000Z",
        })}
        persistence="DIRTY"
        recoveryRecords={[]}
        onClose={vi.fn()}
        onRename={vi.fn()}
        onNew={vi.fn()}
        onExport={vi.fn()}
        onImport={vi.fn()}
        onDuplicate={vi.fn()}
        onDelete={vi.fn()}
        onRecover={vi.fn()}
        onDeleteRecovery={vi.fn()}
      />,
    );
    await frame();

    const dialog = document.querySelector<HTMLElement>(".project-dialog")!;
    const card = document.querySelector<HTMLElement>(".project-dialog__card")!;
    const bounds = card.getBoundingClientRect();
    expect(Number(getComputedStyle(dialog).zIndex)).toBeGreaterThanOrEqual(120);
    expect(bounds.left).toBeGreaterThanOrEqual(0);
    expect(bounds.top).toBeGreaterThanOrEqual(0);
    expect(bounds.right).toBeLessThanOrEqual(window.innerWidth + 1);
    expect(bounds.bottom).toBeLessThanOrEqual(window.innerHeight + 1);

    const focusable = Array.from(card.querySelectorAll<HTMLElement>(
      "button:not(:disabled), input:not(:disabled)",
    ));
    const first = focusable[0]!;
    const last = focusable.at(-1)!;
    last.focus();
    dialog.dispatchEvent(new KeyboardEvent("keydown", {
      key: "Tab",
      bubbles: true,
    }));
    expect(document.activeElement).toBe(first);
  });
});
