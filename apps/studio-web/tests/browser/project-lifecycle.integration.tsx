import { afterEach, describe, expect, it } from "vitest";
import { createRoot, type Root } from "react-dom/client";
import sourceUrl from "../.generated/owned-full42-package/generated/source.glb?url";
import wrongSourceUrl from "../.generated/owned-package/generated/source.glb?url";
import appearanceUrl from "../fixtures/appearance.2da?url";
import { App } from "../../src/App";
import {
  createInMemoryProjectDatabaseV1,
  createMeshy2AuroraProjectV1,
  projectFileReferenceV1,
  reviseMeshy2AuroraProjectV1,
  serializeMeshy2AuroraProjectV1,
} from "../../src/features/project";

const roots: Root[] = [];

afterEach(() => {
  while (roots.length) roots.pop()?.unmount();
  document.body.replaceChildren();
});

async function fetchBytes(url: string): Promise<ArrayBuffer> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`fixture fetch failed: ${response.status}`);
  return response.arrayBuffer();
}

async function sha256(bytes: ArrayBuffer): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

describe("portable project lifecycle", () => {
  it("imports metadata, rejects a wrong source and rebinds exact files by SHA-256", async () => {
    const sourceBytes = await fetchBytes(sourceUrl);
    const appearanceBytes = await fetchBytes(appearanceUrl);
    const source = new File([sourceBytes], "exact-creature.glb", {
      type: "model/gltf-binary",
      lastModified: 100,
    });
    const appearance = new File([appearanceBytes], "appearance.2da", {
      type: "text/plain",
      lastModified: 200,
    });
    const created = createMeshy2AuroraProjectV1({
      projectId: "browser-portable-project",
      name: "Portable creature",
      now: "2026-07-28T12:00:00.000Z",
    });
    const portable = reviseMeshy2AuroraProjectV1(created, {
      files: {
        sourceGlb: projectFileReferenceV1(source, await sha256(sourceBytes)),
        baseTwoDa: projectFileReferenceV1(appearance, await sha256(appearanceBytes)),
        animationEvents: null,
      },
    }, "2026-07-28T12:01:00.000Z");
    const database = createInMemoryProjectDatabaseV1();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    root.render(<App projectDatabase={database} />);

    const button = (label: string) => Array.from(
      container.querySelectorAll<HTMLButtonElement>("button"),
    ).find(({ textContent }) => textContent?.trim() === label);
    await expect.poll(() => button("Manage")).toBeTruthy();
    button("Manage")!.click();
    await expect.poll(() => container.querySelector(
      'input[aria-label="Import project backup file"]',
    )).toBeTruthy();
    const backup = new File(
      [serializeMeshy2AuroraProjectV1(portable)],
      "portable.m2a-project.json",
      { type: "application/json" },
    );
    const importInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Import project backup file"]',
    )!;
    Object.defineProperty(importInput, "files", {
      configurable: true,
      value: [backup],
    });
    importInput.dispatchEvent(new Event("change", { bubbles: true }));

    await expect.poll(() => container.querySelector(
      '[aria-label="Project status"]',
    )?.textContent).toContain("Portable creature");
    await expect.poll(() => container.textContent)
      .toContain("Rebind the exact source file");

    const select = async (label: string, file: File) => {
      const input = container.querySelector<HTMLInputElement>(
        `input[aria-label="${label}"]`,
      )!;
      Object.defineProperty(input, "files", {
        configurable: true,
        value: [file],
      });
      input.dispatchEvent(new Event("change", { bubbles: true }));
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    };
    const wrongSource = new File(
      [await fetchBytes(wrongSourceUrl)],
      "wrong-creature.glb",
      { type: "model/gltf-binary" },
    );
    await select("Meshy model file", wrongSource);
    await expect.poll(() => container.textContent, { timeout: 10_000 })
      .toContain("Project expects source SHA-256");
    expect(container.textContent).toContain("Select the exact source file");

    await select("Meshy model file", source);
    await select("Base model table file", appearance);
    await expect.poll(
      () => button("Continue to Inspect")?.disabled,
      { timeout: 10_000 },
    ).toBe(false);
    await expect.poll(() => container.querySelector(
      '[aria-label="Project status"]',
    )?.textContent).toContain("Saved");

    button("Manage")!.click();
    await expect.poll(() => container.querySelector(
      ".project-dialog__recovery",
    )?.textContent).toContain("Portable creature");
    expect(container.querySelector(".project-dialog__recovery")?.textContent)
      .toContain("browser-portable-project");

    roots.pop();
    root.unmount();
    container.replaceChildren();
    const resumedRoot = createRoot(container);
    roots.push(resumedRoot);
    resumedRoot.render(<App projectDatabase={database} />);
    await expect.poll(() => button("Manage")).toBeTruthy();
    button("Manage")!.click();
    await expect.poll(() => button("Recover")).toBeTruthy();
    button("Recover")!.click();
    await expect.poll(() => container.querySelector(
      '[aria-label="Project status"]',
    )?.textContent).toContain("Portable creature");
    expect(container.textContent).toContain("Rebind the exact source file");
  }, 30_000);
});
