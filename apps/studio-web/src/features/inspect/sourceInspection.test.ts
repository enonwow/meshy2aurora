import { describe, expect, it } from "vitest";
import { projectSourceInspection } from "./sourceInspection";

const SHA = "a".repeat(64);

function readyFixture() {
  return {
    schemaVersion: 1,
    ir: {
      schemaVersion: 1,
      source: {
        format: "GLB_2_0",
        byteLength: 2048,
        sha256: SHA,
        assetVersion: "2.0",
        generator: "meshy2aurora-test",
      },
      scenes: [{}],
      nodes: [{}, {}],
      meshes: [{}],
      primitives: [{}],
      materials: [{}],
      textures: [{}],
      samplers: [{}],
      images: [{}],
      skins: [{ jointNodeIds: [0, 1] }],
      animations: [{
        id: 0,
        name: "idle",
        durationSeconds: 1.25,
        samplers: [
          { id: 0, interpolation: "LINEAR", inputTimesSeconds: [0, 0.5, 1.25] },
          { id: 1, interpolation: "STEP", inputTimesSeconds: [0, 1.25] },
        ],
        channels: [
          { samplerId: 0, targetNodeId: 0, targetPath: "TRANSLATION" },
          { samplerId: 1, targetNodeId: 1, targetPath: "ROTATION" },
        ],
      }],
    },
    report: {
      schemaVersion: 1,
      format: "GLB_2_0",
      input: { byteLength: 2048, sha256: SHA },
      inventory: {
        sceneCount: 1,
        nodeCount: 2,
        meshCount: 1,
        primitiveCount: 1,
        materialCount: 1,
        textureCount: 1,
        samplerCount: 1,
        imageCount: 1,
        skinCount: 1,
        jointReferenceCount: 2,
        animationCount: 1,
        keyframeCount: 5,
      },
      statistics: {
        vertexCount: 24,
        indexCount: 36,
        triangleCount: 12,
        boundsMin: [-1, 0, -2],
        boundsMax: [1, 3, 2],
        primitivesMissingNormals: 0,
        primitivesMissingUv0: 0,
        nonTrianglePrimitives: 0,
      },
      gates: [{
        code: "M2A-GLB-TRIANGLE-WARNING",
        severity: "WARNING",
        path: "statistics.triangleCount",
        expected: "<= 5000",
        actual: "6000",
        message: "triangle count exceeds the warning threshold",
      }],
      diagnostics: [{
        schemaVersion: 1,
        code: "M2A-GLB-UNKNOWN-CHUNK",
        severity: "INFO",
        byteOffset: 128,
        jsonPath: null,
        message: "unknown optional chunk ignored",
      }],
      conversionEligible: true,
    },
  };
}

describe("projectSourceInspection", () => {
  it("projects the real ingest result contract and clip inventory", () => {
    const result = projectSourceInspection(JSON.stringify(readyFixture()));

    expect(result).toEqual({
      kind: "READY",
      snapshot: {
        schemaVersion: 1,
        source: {
          format: "GLB_2_0",
          byteLength: 2048,
          sha256: SHA,
          assetVersion: "2.0",
          generator: "meshy2aurora-test",
        },
        inventory: readyFixture().report.inventory,
        statistics: readyFixture().report.statistics,
        boneCount: 2,
        clips: [{
          id: 0,
          name: "idle",
          durationSeconds: 1.25,
          samplerCount: 2,
          channelCount: 2,
          keyframeCount: 5,
          targetNodeIds: [0, 1],
          targetPaths: ["TRANSLATION", "ROTATION"],
        }],
        gates: readyFixture().report.gates,
        diagnostics: readyFixture().report.diagnostics,
        conversionEligible: true,
      },
    });
  });

  it("keeps absent optional source data explicit", () => {
    const fixture = readyFixture();
    fixture.ir.source.generator = null as unknown as string;
    fixture.ir.animations = [];
    fixture.report.inventory.animationCount = 0;
    fixture.report.inventory.keyframeCount = 0;
    fixture.report.statistics.boundsMin = null as unknown as number[];
    fixture.report.statistics.boundsMax = null as unknown as number[];

    const result = projectSourceInspection(JSON.stringify(fixture));

    expect(result.kind).toBe("READY");
    if (result.kind !== "READY") throw new Error("expected READY");
    expect(result.snapshot.source.generator).toBeNull();
    expect(result.snapshot.clips).toEqual([]);
    expect(result.snapshot.statistics.boundsMin).toBeNull();
    expect(result.snapshot.statistics.boundsMax).toBeNull();
  });

  it("projects the fatal error shape emitted by the same WASM boundary", () => {
    expect(projectSourceInspection(JSON.stringify({
      schemaVersion: 1,
      code: "M2A-GLB-INPUT-EMPTY",
      message: "GLB input is empty",
    }))).toEqual({
      kind: "FAILED",
      failure: {
        schemaVersion: 1,
        code: "M2A-GLB-INPUT-EMPTY",
        message: "GLB input is empty",
        byteOffset: null,
        jsonPath: null,
      },
    });
  });

  it("rejects missing fields instead of manufacturing metrics", () => {
    const fixture = readyFixture();
    delete (fixture.report.statistics as Partial<typeof fixture.report.statistics>).vertexCount;

    expect(() => projectSourceInspection(JSON.stringify(fixture)))
      .toThrow("Source inspection field ingestJson.report.statistics.vertexCount is missing or has the wrong type");
  });

  it("rejects inventory that disagrees with the IR", () => {
    const fixture = readyFixture();
    fixture.report.inventory.meshCount = 2;

    expect(() => projectSourceInspection(JSON.stringify(fixture)))
      .toThrow("Source inspection identity mismatch at ingestJson.ir.meshes");
  });

  it("rejects invalid JSON and ambiguous result/error payloads", () => {
    expect(() => projectSourceInspection("{"))
      .toThrow("Source inspection field ingestJson is missing or has the wrong type");

    const fixture = readyFixture() as ReturnType<typeof readyFixture> & { code: string };
    fixture.code = "M2A-GLB-INPUT-EMPTY";
    expect(() => projectSourceInspection(JSON.stringify(fixture)))
      .toThrow("Source inspection field ingestJson is missing or has the wrong type");
  });
});
