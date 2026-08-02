import { describe, expect, it } from "vitest";
import { parseAnimationLayerBakeReportV1 } from "./animationLayers";

describe("animation layer UI adapter", () => {
  it("accepts the deterministic Core bake report", () => {
    const report = parseAnimationLayerBakeReportV1(JSON.stringify({
      schemaVersion: 1,
      activeLayerIds: ["base", "correction"],
      keyCountBefore: 20,
      keyCountAfter: 12,
      clip: { id: "attack", name: "attack" },
      fingerprintSha256: "c".repeat(64),
    }));

    expect(report.activeLayerIds).toEqual(["base", "correction"]);
    expect(report.keyCountAfter).toBe(12);
  });
});
