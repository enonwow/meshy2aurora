import { describe, expect, it } from "vitest";
import {
  filterSupermodelCatalogEntriesV1,
  parseHakModelIndexV1,
  parseNwnBifIndexPlanV1,
  parseNwnKeyModelIndexV1,
  parseSupermodelCatalogV1,
} from "./types";

describe("supermodel catalog projections", () => {
  it("projects KEY and BIF plans without interpreting binary offsets in the UI", () => {
    expect(parseNwnKeyModelIndexV1(JSON.stringify({
      schemaVersion: 1,
      resourceCount: 10,
      modelResourceCount: 1,
      bifs: [{ index: 0, logicalName: "data/models_01.bif" }],
      models: [{ keyIndex: 2, resref: "c_wolf", bifIndex: 0, resourceIndex: 7 }],
    }))).toEqual({
      schemaVersion: 1,
      resourceCount: 10,
      modelResourceCount: 1,
      bifs: [{ index: 0, logicalName: "data/models_01.bif" }],
      models: [{ keyIndex: 2, resref: "c_wolf", bifIndex: 0, resourceIndex: 7 }],
    });
    expect(parseNwnBifIndexPlanV1(JSON.stringify({
      schemaVersion: 1,
      resourceCount: 8,
      tableOffset: 32,
      tableByteLength: 128,
    }))).toMatchObject({ tableOffset: 32, tableByteLength: 128 });
  });

  it("projects and filters every discovered supermodel independently of c_wolf", () => {
    const model = (resref: string, animations: number) => ({
      resref,
      sourceId: "base",
      sourceKind: "BASE_KEY_BIF",
      containerName: "data/models_01.bif",
      sourcePriority: 0,
      resourceIndex: 1,
      payloadOffset: 100,
      payloadSize: 244,
      header: {
        schemaVersion: 1,
        format: "BINARY",
        modelName: resref,
        supermodelName: "NULL",
        classification: 4,
        animationScale: 1,
        localAnimationCount: animations,
      },
    });
    const catalog = parseSupermodelCatalogV1(JSON.stringify({
      schemaVersion: 1,
      completeness: "COMPLETE",
      declaredModelCount: 3,
      scannedModelCount: 3,
      failedModelCount: 0,
      supermodelCount: 2,
      entries: [
        { resref: "c_horror", status: "RESOLVED", resource: model("c_horror", 42), definitions: [model("c_horror", 42)], children: ["c_child"], chain: ["c_horror"] },
        { resref: "c_wolf", status: "RESOLVED", resource: model("c_wolf", 35), definitions: [model("c_wolf", 35)], children: ["c_dog"], chain: ["c_wolf"] },
      ],
    }));
    expect(filterSupermodelCatalogEntriesV1(catalog.entries, "horror", "ALL").map((entry) => entry.resref)).toEqual(["c_horror"]);
    expect(filterSupermodelCatalogEntriesV1(catalog.entries, "", "WITH_ANIMATIONS")).toHaveLength(2);
  });

  it("rejects an error envelope instead of reporting a partial catalog as valid data", () => {
    expect(() => parseNwnKeyModelIndexV1(JSON.stringify({
      schemaVersion: 1,
      code: "M2A-SUPERMODEL-KEY-SIGNATURE",
      context: "expected KEY V1",
    }))).toThrow(/M2A-SUPERMODEL-KEY-SIGNATURE/);
  });

  it("projects HAK model locators while retaining per-resource failures", () => {
    const report = parseHakModelIndexV1(JSON.stringify({
      schemaVersion: 1,
      resourceCount: 2,
      modelResourceCount: 1,
      scannedModelCount: 0,
      failedModelCount: 1,
      models: [{
        resourceIndex: 1,
        resref: "broken_model",
        payloadOffset: 200,
        payloadSize: 32,
        header: null,
        error: { schemaVersion: 1, code: "BROKEN", offset: 0, context: "bad MDL" },
      }],
    }));
    expect(report.models[0].error?.code).toBe("BROKEN");
    expect(report.failedModelCount).toBe(1);
  });
});
