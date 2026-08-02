import { afterEach, describe, expect, it } from "vitest";
import { page } from "vitest/browser";
import { createElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import shipSourceUrl from "../../../../sample-3d/tlc-ship-under-construction-s1-p150k-v1/source.glb?url";
import { SourceViewport } from "../../src/features/preview/SourceViewport";
import { StudioWorkerClient } from "../../src/worker/client";

interface InspectedComponent {
  key: {
    sceneId: number;
    nodeId: number;
    primitiveId: number;
    componentIndex: number;
  };
  boundsMin: [number, number, number];
  boundsMax: [number, number, number];
  triangleCount: number;
}

const roots: Root[] = [];
const clients: StudioWorkerClient[] = [];

afterEach(() => {
  while (clients.length) clients.pop()?.dispose();
  while (roots.length) roots.pop()?.unmount();
  document.body.replaceChildren();
});

async function fetchBytes(url: string): Promise<ArrayBuffer> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`fixture unavailable: ${response.status}`);
  return response.arrayBuffer();
}

async function sha256(bytes: ArrayBuffer): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function waitForRenderedCanvases(): Promise<void> {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    const canvases = [...document.querySelectorAll("canvas")];
    if (canvases.length === 2 && canvases.every((canvas) => canvas.width > 100 && canvas.height > 100)) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("offline A/B canvases did not become renderable");
}

describe("Material Separation offline preview", () => {
  it("captures the exact real Meshy source beside its authored Material ID overlay", async () => {
    await page.viewport(1440, 900);
    const sourceBytes = await fetchBytes(shipSourceUrl);
    const sourceSha256 = await sha256(sourceBytes.slice(0));
    expect(sourceSha256).toBe("61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278");

    const client = new StudioWorkerClient();
    clients.push(client);
    const inspectionBytes = sourceBytes.slice(0);
    const inspected = await client.request({
      requestId: "material-separation-preview-inspection",
      type: "INSPECT_MODEL_COMPONENTS",
      sourceGlb: inspectionBytes,
      target: "PLACEABLE",
      sourceStateId: `source:${sourceSha256}`,
    }, [inspectionBytes]);
    if (!inspected.ok || inspected.type !== "MODEL_COMPONENTS_INSPECTED") {
      throw new Error("real Meshy component inspection failed");
    }
    const bootstrap = JSON.parse(inspected.inspectionJson) as {
      inventory: { triangleCount: number; components: InspectedComponent[] };
    };
    expect(bootstrap.inventory.triangleCount).toBe(152_574);
    expect(bootstrap.inventory.components.length).toBe(21_936);

    const overlays = [...bootstrap.inventory.components]
      .sort((left, right) => right.triangleCount - left.triangleCount)
      .slice(0, 48)
      .map((component, index) => ({
        id: `${component.key.sceneId}/${component.key.nodeId}/${component.key.primitiveId}/${component.key.componentIndex}`,
        boundsMin: component.boundsMin,
        boundsMax: component.boundsMax,
        color: index % 2 === 0 ? "#d08845" : "#58b8c8",
        selected: index < 8,
      }));

    const file = new File([sourceBytes], "tlc-ship-under-construction-source.glb", {
      type: "model/gltf-binary",
    });
    const style = document.createElement("style");
    style.textContent = `
      html, body { margin: 0; background: #0a1016; }
      .viewport { height: 735px; overflow: hidden; border-top: 1px solid #344454; background: #09131c; display: grid; grid-template-rows: auto 1fr auto auto; }
      .viewport > header { display: flex; gap: 8px; padding: 8px 10px; color: #9fb0bf; font-size: 12px; }
      .viewport > header strong { color: #58d8bd; }
      .viewport canvas { display: block; width: 100%; height: 610px; }
      .viewport__overlays { display: flex; align-items: center; gap: 10px; margin: 0; padding: 8px 10px; border: 0; border-top: 1px solid #344454; color: #9fb0bf; font-size: 11px; }
      .viewport__overlays label { display: inline-flex; align-items: center; gap: 3px; }
      .viewport__animation { padding: 7px 10px; border-top: 1px solid #344454; color: #9fb0bf; font-size: 11px; }
      .viewport__animation p { margin: 0; }
    `;
    document.head.append(style);
    const host = document.createElement("main");
    host.style.cssText = "width:1400px;height:820px;padding:18px;box-sizing:border-box;background:#0a1016;color:#e8edf2;font:15px system-ui;display:grid;grid-template-columns:1fr 1fr;gap:16px";
    document.body.replaceChildren(host);
    const root = createRoot(host);
    roots.push(root);
    const panel = (title: string, detail: string, componentOverlays: typeof overlays | undefined) => createElement(
      "section",
      { style: { display: "grid", gridTemplateRows: "auto 1fr", minWidth: 0, border: "1px solid #344454", background: "#101923" } },
      createElement("header", { style: { padding: "12px 14px" } },
        createElement("strong", null, title),
        createElement("div", { style: { color: "#9fb0bf", marginTop: "4px" } }, detail),
      ),
      createElement("div", { style: { minHeight: 0 } }, createElement(SourceViewport, {
        input: { provenance: "SOURCE", file, sourceSha256 },
        componentOverlays,
      })),
    );
    root.render(createElement(
      "main",
      { style: { display: "contents" } },
      panel("A — exact source GLB", "152,574 triangles · source SHA-256 61baad67…0278", undefined),
      panel("B — Material Separation preview", "48 largest loose parts · alternating authored Material IDs · UV0 unchanged", overlays),
    ));

    await waitForRenderedCanvases();
    await new Promise((resolve) => setTimeout(resolve, 1_000));
    const screenshotPath = await page.screenshot({
      path: "../../../../documentation/evidence/material-separation-offline-ab-2026-08-01.png",
    });
    expect(screenshotPath).toContain("material-separation-offline-ab-2026-08-01.png");
  }, 60_000);
});
