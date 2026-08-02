import { describe, expect, it, vi } from "vitest";
import {
  ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1,
  COMMUNITY_ANIMATION_CATALOG_V1,
  animationPresetCatalogIdentityV1,
  filterAnimationPresetCatalogEntriesV1,
  loadAnimationPresetAssetsV1,
  projectAnimationPresetCatalogWindowV1,
  searchAnimationPresetCatalogV1,
  verifyAnimationPresetPreviewBytesV1,
} from "./catalog";

describe("community animation catalog", () => {
  it("ships seven deterministic starter presets", () => {
    expect(COMMUNITY_ANIMATION_CATALOG_V1.entries).toHaveLength(7);
    expect(COMMUNITY_ANIMATION_CATALOG_V1.entries.map(({ presetId }) => presetId)).toEqual([
      "m2a_combat_guard",
      "m2a_dodge_left",
      "m2a_dodge_right",
      "m2a_left_jab",
      "m2a_right_cross",
      "m2a_right_hook",
      "m2a_uppercut",
    ]);
    expect(COMMUNITY_ANIMATION_CATALOG_V1.entries.every((entry) => (
      entry.previewPath === "preview.webp"
      && typeof entry.previewByteLength === "number"
      && entry.previewByteLength > 0
      && /^[0-9a-f]{64}$/.test(entry.previewSha256 ?? "")
    ))).toBe(true);
  });

  it("searches by tag, author, label and id with combinable tags", () => {
    expect(searchAnimationPresetCatalogV1("uppercut", "ALL", []).map(({ presetId }) => presetId))
      .toEqual(["m2a_uppercut"]);
    expect(searchAnimationPresetCatalogV1("contributors", "BUILT_IN", ["boxing", "right-hand"])
      .map(({ presetId }) => presetId))
      .toEqual(["m2a_right_cross", "m2a_right_hook", "m2a_uppercut"]);
    expect(searchAnimationPresetCatalogV1("m2a_dodge", "ALL", ["dodge"]))
      .toHaveLength(2);
    expect(animationPresetCatalogIdentityV1({
      presetId: "m2a_right_cross",
      presetVersion: 1,
    })).toBe("m2a_right_cross@1");
    expect(animationPresetCatalogIdentityV1({
      presetId: "m2a_right_cross",
      presetVersion: 2,
    })).toBe("m2a_right_cross@2");
  });

  it("loads motion lazily and binds the embedded manifest to the tracked catalog", async () => {
    const fetch = vi.fn(async () => {
      throw new Error("network is blocked");
    });
    vi.stubGlobal("fetch", fetch);
    const loaded = await loadAnimationPresetAssetsV1("m2a_right_cross", 1);
    expect(loaded.entry.label).toBe("Right cross");
    expect(JSON.parse(loaded.animationJson).tracks.length).toBeGreaterThan(1);
    expect(loaded.catalogSha256).toMatch(/^[0-9a-f]{64}$/);
    expect(loaded.previewUrl).toMatch(/preview\.webp/);
    expect(fetch).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
    await expect(loadAnimationPresetAssetsV1("missing", 1)).rejects.toThrow(
      "Unknown animation preset missing@1",
    );
  });

  it("searches the maximum 2048-entry contract while bounding one DOM result window", () => {
    const seed = COMMUNITY_ANIMATION_CATALOG_V1.entries[0]!;
    const entries = Array.from({ length: 2_048 }, (_, index) => ({
      ...seed,
      presetId: `m2a_synthetic_${index.toString().padStart(4, "0")}`,
      label: `Synthetic motion ${index}`,
      summary: `Deterministic catalog scale fixture ${index}.`,
      tags: index % 2 === 0 ? ["humanoid", "attack"] : ["humanoid", "defense"],
    }));

    const allHumanoid = filterAnimationPresetCatalogEntriesV1(
      entries,
      "",
      "BUILT_IN",
      ["humanoid"],
    );
    expect(allHumanoid).toHaveLength(2_048);
    expect(filterAnimationPresetCatalogEntriesV1(
      entries,
      "m2a_synthetic_2047",
      "ALL",
      [],
    )).toEqual([entries[2_047]]);
    expect(projectAnimationPresetCatalogWindowV1(
      allHumanoid,
      ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1,
    )).toEqual(entries.slice(0, ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1));
    expect(projectAnimationPresetCatalogWindowV1(allHumanoid, 200)).toHaveLength(200);
  });

  it("fails closed when runtime preview bytes do not match the catalog binding", async () => {
    const bytes = new TextEncoder().encode("preview-fixture");
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    const sha256 = [...new Uint8Array(digest)]
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join("");
    const entry = {
      ...COMMUNITY_ANIMATION_CATALOG_V1.entries[0]!,
      previewPath: "preview.webp" as const,
      previewByteLength: bytes.byteLength,
      previewSha256: sha256,
    };
    await expect(verifyAnimationPresetPreviewBytesV1(entry, bytes)).resolves.toBeUndefined();
    await expect(verifyAnimationPresetPreviewBytesV1(
      entry,
      new Uint8Array([...bytes, 0]),
    )).rejects.toThrow("preview size mismatch");
    await expect(verifyAnimationPresetPreviewBytesV1(
      { ...entry, previewSha256: "0".repeat(64) },
      bytes,
    )).rejects.toThrow("preview SHA-256 mismatch");
  });
});
