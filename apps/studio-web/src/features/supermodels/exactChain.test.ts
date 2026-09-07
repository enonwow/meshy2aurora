// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import type { SupermodelCatalogSessionV1 } from "./filesystem";
import { loadExactReferenceSupermodelChainV2 } from "./exactChain";
import type { SupermodelModelV1 } from "./types";

function model(
  resref: string,
  supermodelName: string,
  format: "ASCII" | "BINARY",
  payloadOffset: number,
  payloadSize: number,
): SupermodelModelV1 {
  return {
    resref,
    sourceId: "fixture",
    sourceKind: "LOOSE_MDL",
    containerName: "chain.bin",
    sourcePriority: 1,
    resourceIndex: payloadOffset,
    payloadOffset,
    payloadSize,
    header: {
      schemaVersion: 1,
      format,
      modelName: resref,
      supermodelName,
      classification: 4,
      animationScale: 1,
      localAnimationCount: format === "BINARY" ? 3 : 0,
    },
  };
}

describe("exact reference-supermodel chain", () => {
  it("binds every selected-to-root payload, format and SHA in order", async () => {
    const selected = model("c_selected", "c_parent", "ASCII", 0, 3);
    const parent = model("c_parent", "NULL", "BINARY", 3, 4);
    const session: SupermodelCatalogSessionV1 = {
      sourceId: "fixture",
      sourceLabel: "synthetic chain",
      models: [selected, parent],
      filesByContainer: new Map([["chain.bin", new File([
        new Uint8Array([1, 2, 3, 4, 5, 6, 7]),
      ], "chain.bin")]]),
      catalog: {
        schemaVersion: 1,
        completeness: "COMPLETE",
        declaredModelCount: 2,
        scannedModelCount: 2,
        failedModelCount: 0,
        supermodelCount: 1,
        entries: [{
          resref: "c_selected",
          status: "RESOLVED",
          resource: selected,
          definitions: [selected],
          children: [],
          chain: ["c_selected", "c_parent"],
        }],
      },
    };
    const chain = await loadExactReferenceSupermodelChainV2(session, "c_selected");
    expect([...new Uint8Array(chain.blob)]).toEqual([1, 2, 3, 4, 5, 6, 7]);
    expect(chain.descriptors).toMatchObject([
      { resref: "c_selected", supermodelResref: "c_parent", format: "ASCII", byteOffset: 0, byteLength: 3 },
      { resref: "c_parent", supermodelResref: "NULL", format: "BINARY", byteOffset: 3, byteLength: 4 },
    ]);
    expect(chain.descriptors.every((resource) => /^[0-9a-f]{64}$/.test(resource.sha256))).toBe(true);

    await expect(loadExactReferenceSupermodelChainV2(session, "c_selected", [{
      resref: "c_selected",
      supermodelResref: "c_parent",
      format: "ASCII",
      sha256: "0".repeat(64),
      byteLength: 3,
      localAnimationCount: 0,
      nodeCount: 1,
      controllerCount: 0,
    }, {
      resref: "c_parent",
      supermodelResref: "NULL",
      format: "BINARY",
      sha256: chain.descriptors[1].sha256,
      byteLength: 4,
      localAnimationCount: 3,
      nodeCount: 2,
      controllerCount: 4,
    }])).rejects.toThrow("CHAIN-IDENTITY-MISMATCH");
  });
});
