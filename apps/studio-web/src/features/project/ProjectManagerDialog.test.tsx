// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createMeshy2AuroraProjectV1 } from "./schema";
import { ProjectManagerDialog } from "./ProjectManagerDialog";

const roots: Root[] = [];

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
});

describe("ProjectManagerDialog", () => {
  it("exposes the complete project lifecycle and confirmation-gates current deletion", async () => {
    const handlers = {
      onClose: vi.fn(),
      onRename: vi.fn(),
      onNew: vi.fn(),
      onExport: vi.fn(),
      onImport: vi.fn(),
      onDuplicate: vi.fn(),
      onDelete: vi.fn(),
      onRecover: vi.fn(),
      onDeleteRecovery: vi.fn(),
    };
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    await act(async () => root.render(
      <ProjectManagerDialog
        {...handlers}
        project={createMeshy2AuroraProjectV1({
          projectId: "project-ui-01",
          name: "UI project",
          now: "2026-07-28T12:00:00.000Z",
        })}
        persistence="SAVED"
        recoveryRecords={[{
          projectId: "project-recovery-01",
          name: "Recovery project",
          target: "PLACEABLE",
          revision: 4,
          updatedAt: "2026-07-28T12:00:00.000Z",
          sourceSha256: null,
          recoverable: true,
          diagnostics: [],
        }]}
      />,
    ));

    const button = (label: string) => Array.from(
      container.querySelectorAll<HTMLButtonElement>("button"),
    ).find(({ textContent }) => textContent?.trim() === label)!;
    await act(async () => {
      button("New project").click();
      button("Export project").click();
      button("Duplicate project").click();
      button("Recover").click();
    });
    expect(handlers.onNew).toHaveBeenCalledOnce();
    expect(handlers.onExport).toHaveBeenCalledOnce();
    expect(handlers.onDuplicate).toHaveBeenCalledOnce();
    expect(handlers.onRecover).toHaveBeenCalledWith("project-recovery-01");

    await act(async () => button("Delete project").click());
    expect(handlers.onDelete).not.toHaveBeenCalled();
    await act(async () => button("Confirm delete").click());
    expect(handlers.onDelete).toHaveBeenCalledOnce();
  });

  it("routes a selected JSON backup to import", async () => {
    const onImport = vi.fn();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    await act(async () => root.render(
      <ProjectManagerDialog
        project={createMeshy2AuroraProjectV1({
          projectId: "project-import-ui",
          now: "2026-07-28T12:00:00.000Z",
        })}
        persistence="NOT_SAVED"
        recoveryRecords={[]}
        onClose={vi.fn()}
        onRename={vi.fn()}
        onNew={vi.fn()}
        onExport={vi.fn()}
        onImport={onImport}
        onDuplicate={vi.fn()}
        onDelete={vi.fn()}
        onRecover={vi.fn()}
        onDeleteRecovery={vi.fn()}
      />,
    ));
    const file = new File(["{}"], "project.m2a-project.json", {
      type: "application/json",
    });
    const input = container.querySelector<HTMLInputElement>(
      'input[aria-label="Import project backup file"]',
    )!;
    Object.defineProperty(input, "files", { configurable: true, value: [file] });
    await act(async () => input.dispatchEvent(new Event("change", { bubbles: true })));

    expect(onImport).toHaveBeenCalledWith(file);
  });
});
