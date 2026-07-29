import { afterEach, describe, expect, it } from "vitest";
import { page } from "vitest/browser";
import { createElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import h2Url from "@m2a-canonical-repository/sample-3d/h2-clockwork-sentinel-1500/source.glb?url";
import appearanceUrl from "@m2a-canonical-repository/local-reference-assets/appearance.2da?url";
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

describe("real Meshy H2 procedural humanoid skin readback", () => {
  it("reconstructs a skinned Three asset only when inverse binds agree with the decoded MDL", async () => {
    const [source, appearance] = await Promise.all([
      input(h2Url, "h2-clockwork-sentinel-1500.glb", "model/gltf-binary"),
      input(appearanceUrl, "appearance.2da", "text/plain"),
    ]);
    const [sourceGlb, appearanceTwoDa] = await Promise.all([source.arrayBuffer(), appearance.arrayBuffer()]);
    const client = new StudioWorkerClient();
    clients.push(client);
    const response = await client.request({
      requestId: "real-h2-procedural-skinning-diagnostic",
      type: "BUILD_MODEL_PACKAGE",
      sourceGlb,
      appearanceTwoDa,
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_42",
      textureArtifactCleanup: false,
      identityJson: JSON.stringify({
        modelResref: "m2a_stcrmdl2",
        textureResref: "m2a_stcrtex2",
        hakResref: "m2a_stcrhak2",
        appearanceLabel: "M2A_STUDIO_CREATURE_V2",
      }),
    }, [sourceGlb, appearanceTwoDa]);
    expect(response).toMatchObject({ ok: true, type: "MODEL_PACKAGE_BUILT" });
    if (!response.ok || response.type !== "MODEL_PACKAGE_BUILT") {
      throw new Error("H2 procedural package unavailable");
    }
    const report = JSON.parse(response.reportJson) as {
      animationCompleteness?: {
        schemaVersion?: number;
        requiredClipCount?: number;
        inputSourceClipCount?: number;
        preservedSourceClipCount?: number;
        sourceDerivedClipCount?: number;
        proceduralClipCount?: number;
        discardedSourceClipCount?: number;
        fallbackAliasCount?: number;
      };
      skinAnimationConformance?: { complete?: boolean };
    };
    expect(report.animationCompleteness).toMatchObject({
      schemaVersion: 2,
      requiredClipCount: 42,
      inputSourceClipCount: 1,
      preservedSourceClipCount: 1,
      sourceDerivedClipCount: 0,
      proceduralClipCount: 41,
      discardedSourceClipCount: 0,
      fallbackAliasCount: 0,
    });
    expect(report.skinAnimationConformance?.complete).toBe(true);
    const asset = buildAuroraReadbackAsset(projectCanonicalReadback(response.readbackJson));
    expect(asset.animations).toHaveLength(42);
    expect(asset.animations.map((clip) => clip.name)).toEqual(
      expect.arrayContaining(["cpause1", "cwalk", "crun", "ca1slashl", "ckdbckdie"]),
    );
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
      await page.screenshot({ path: `../../../../proof-output/meshy-h2-procedural-offline-readback/animation-skinning-frames/frame-${frame}.png` });
    }
    await page.screenshot({ path: "../../../../proof-output/meshy-h2-procedural-offline-readback/converted-h2-skinned-000ms.png" });
    await page.screenshot({ path: "../../../../proof-output/meshy-h2-procedural-offline-readback/converted-h2-skinned-500ms.png" });
    await page.getByRole("button", { name: "Stop" }).click();
    for (let index = 0; index <= 15; index += 1) {
      if (index > 0) {
        for (let step = 0; step < 8; step += 1) {
          await page.getByRole("button", { name: "Next animation keyframe" }).click();
        }
      }
      const frame = String(index).padStart(3, "0");
      await page.screenshot({ path: `../../../../proof-output/meshy-h2-procedural-offline-readback/animation-skinning-full-frames/frame-${frame}.png` });
    }
    expect(errors).toEqual([]);
  }, 60_000);
});
