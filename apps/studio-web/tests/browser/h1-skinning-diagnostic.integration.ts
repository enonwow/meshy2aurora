import { afterEach, describe, expect, it } from "vitest";
import { page } from "@vitest/browser/context";
import { createElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import h1Url from "../../../../test-assets/meshy/incoming/h1-humanoid-1500.glb?url";
import appearanceUrl from "../../../../local-reference-assets/appearance.2da?url";
import { AuroraReadbackViewport, buildAuroraReadbackAsset } from "../../src/features/preview/AuroraReadbackViewport";
import { projectCanonicalReadback } from "../../src/features/results/projectReadback";
import { StudioWorkerClient } from "../../src/worker/client";
import "../../src/styles.css";

const clients: StudioWorkerClient[] = [];
const roots: Root[] = [];

afterEach(() => {
  clients.splice(0).forEach((client) => client.dispose());
  roots.splice(0).forEach((root) => root.unmount());
  document.body.replaceChildren();
});

async function input(url: string, name: string, type: string) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`fixture unavailable: ${response.status}`);
  return new File([await response.arrayBuffer()], name, { type });
}

describe("real Meshy H1 canonical skin readback", () => {
  it("reconstructs a skinned Three asset only when inverse binds agree with the decoded MDL", async () => {
    const [source, appearance] = await Promise.all([
      input(h1Url, "h1-humanoid-1500.glb", "model/gltf-binary"),
      input(appearanceUrl, "appearance.2da", "text/plain"),
    ]);
    const [sourceGlb, appearanceTwoDa] = await Promise.all([source.arrayBuffer(), appearance.arrayBuffer()]);
    const client = new StudioWorkerClient();
    clients.push(client);
    const response = await client.request({
      requestId: "real-h1-skinning-diagnostic",
      type: "BUILD_MODEL_PACKAGE",
      sourceGlb,
      appearanceTwoDa,
    }, [sourceGlb, appearanceTwoDa]);
    expect(response).toMatchObject({ ok: true, type: "MODEL_PACKAGE_BUILT" });
    if (!response.ok || response.type !== "MODEL_PACKAGE_BUILT") throw new Error("H1 package unavailable");
    const asset = buildAuroraReadbackAsset(projectCanonicalReadback(response.readbackJson));
    expect(asset.animations.map((clip) => clip.name)).toContain("cpause1");
    expect(asset.root.getObjectByProperty("isSkinnedMesh", true)).toBeDefined();

    const container = document.createElement("main");
    container.style.width = "1440px";
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    const errors: string[] = [];
    root.render(createElement(AuroraReadbackViewport, {
      report: projectCanonicalReadback(response.readbackJson),
      onSelectPart: () => undefined,
      onError: (message: string) => errors.push(message),
    }));
    await page.viewport(1440, 900);
    await expect.element(page.getByRole("button", { name: "Play" })).toBeVisible();
    for (let index = 0; index <= 15; index += 1) {
      if (index > 0) await page.getByRole("button", { name: "Next animation keyframe" }).click();
      const frame = String(index).padStart(3, "0");
      await page.screenshot({ path: `../../../../proof-output/meshy-h1-e2e-2026-07-17/animation-skinning-frames/frame-${frame}.png` });
    }
    await page.screenshot({ path: "../../../../proof-output/meshy-h1-e2e-2026-07-17/13-converted-h1-skinned-000ms.png" });
    await page.screenshot({ path: "../../../../proof-output/meshy-h1-e2e-2026-07-17/14-converted-h1-skinned-500ms.png" });
    await page.getByRole("button", { name: "Stop" }).click();
    for (let index = 0; index <= 15; index += 1) {
      if (index > 0) {
        for (let step = 0; step < 8; step += 1) {
          await page.getByRole("button", { name: "Next animation keyframe" }).click();
        }
      }
      const frame = String(index).padStart(3, "0");
      await page.screenshot({ path: `../../../../proof-output/meshy-h1-e2e-2026-07-17/animation-skinning-full-frames/frame-${frame}.png` });
    }
    expect(errors).toEqual([]);
  }, 60_000);
});
