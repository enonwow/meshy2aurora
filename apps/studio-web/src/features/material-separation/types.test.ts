import { describe, expect, it } from "vitest";
import { materialBoxWorldUvProjectionDocumentV1 } from "./types";

describe("materialBoxWorldUvProjectionDocumentV1", () => {
  it("binds world-scale box projection to sorted material IDs", () => {
    expect(materialBoxWorldUvProjectionDocumentV1("source", "separation", ["wood", "deck", "wood"]))
      .toEqual({
        schemaVersion: 1,
        sourceSha256: "source",
        separationSha256: "separation",
        rules: [
          {
            authoredMaterialId: "deck",
            mode: "MATERIAL_BOX_WORLD",
            uRepeats: 0.5,
            vMin: 0,
            vMax: 1,
            deterministicUPhase: false,
          },
          {
            authoredMaterialId: "wood",
            mode: "MATERIAL_BOX_WORLD",
            uRepeats: 0.5,
            vMin: 0,
            vMax: 1,
            deterministicUPhase: false,
          },
        ],
      });
  });

  it("keeps projection disabled when no material was selected", () => {
    expect(materialBoxWorldUvProjectionDocumentV1("source", "separation", [])).toBeUndefined();
  });

  it("uses an independently authored physical scale per material", () => {
    const document = materialBoxWorldUvProjectionDocumentV1(
      "source",
      "separation",
      ["hull", "mast"],
      { hull: 0.25, mast: 1.5 },
    );
    expect(document?.rules.map((rule) => [rule.authoredMaterialId, rule.uRepeats])).toEqual([
      ["hull", 0.25],
      ["mast", 1.5],
    ]);
  });
});
