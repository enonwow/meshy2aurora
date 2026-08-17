import { describe, expect, it } from "vitest";
import { bindItemIconPresentationFitV1 } from "./itemIconPresentationFit";

const fields = ["ModelPart1", "ModelPart2", "ModelPart3"] as const;
const hashes = ["1".repeat(64), "2".repeat(64), "3".repeat(64)] as const;

function presentationReport(status: "PASSED" | "MANUAL_REQUIRED" = "MANUAL_REQUIRED") {
  return {
    schemaVersion: 3,
    algorithm: "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX",
    status,
    solutionSha256: "a".repeat(64),
    orientationFrame: {
      targetAxialAxis: 1,
      targetWidthAxis: 2,
      targetDepthAxis: 0,
      handednessDeterminant: 1,
      status: "PASSED",
    },
    parts: fields.map((field, index) => ({
      field,
      sourceSha256: hashes[index],
      sourceNode: null,
      axialTargetAxis: 1,
      targetAxialLength: [0.22, 0.08, 0.90][index],
      transform: {
        translation: [0, index * 0.2, 0],
        rotationXyzw: [0, 0, 0, 1],
        uniformScale: 1,
        pivot: [0, 0, 0],
      },
      transformSha256: String(index + 4).repeat(64),
    })),
    adjacentConnectors: [
      {
        firstField: "ModelPart1",
        secondField: "ModelPart2",
        axialAxis: 1,
        axialOverlap: 0.008,
        requiredMinOverlap: 0.006,
        requiredMaxOverlap: 0.010,
        surfaceStatus: "OVERLAP",
        status: "OVERLAPPING",
      },
      {
        firstField: "ModelPart2",
        secondField: "ModelPart3",
        axialAxis: 1,
        axialOverlap: 0.008,
        requiredMinOverlap: 0.006,
        requiredMaxOverlap: 0.010,
        surfaceStatus: "GAP",
        status: "FAILED",
      },
    ],
  };
}

describe("Item icon presentation fit binding", () => {
  it("accepts a source-bound YZX layout when only presentation surface continuity is missing", () => {
    const result = bindItemIconPresentationFitV1(
      presentationReport(),
      fields.map((field, index) => ({ field, sourceSha256: hashes[index], sourceNode: null })),
      [0.22, 0.08, 0.90],
    );

    expect(result.summary).toMatchObject({
      status: "PASSED",
      sourceFitStatus: "MANUAL_REQUIRED",
      surfaceContinuityRequired: false,
      targetAxialLengths: [0.22, 0.08, 0.90],
      worldAttachmentUnaffected: true,
    });
    expect(result.transforms.get("ModelPart2")).toEqual(
      presentationReport().parts[1].transform,
    );
  });

  it("fails closed when the presentation frame is not Aurora YZX", () => {
    const report = presentationReport();
    report.orientationFrame.targetAxialAxis = 2;
    expect(() => bindItemIconPresentationFitV1(
      report,
      fields.map((field, index) => ({ field, sourceSha256: hashes[index], sourceNode: null })),
      [0.22, 0.08, 0.90],
    )).toThrow("ITEM-ICON-PRESENTATION-FIT-INVALID");
  });

  it("fails closed when a presentation transform is bound to another source", () => {
    const expected = fields.map((field, index) => ({
      field,
      sourceSha256: index === 1 ? "f".repeat(64) : hashes[index],
      sourceNode: null,
    }));
    expect(() => bindItemIconPresentationFitV1(
      presentationReport(),
      expected,
      [0.22, 0.08, 0.90],
    )).toThrow("ITEM-ICON-PRESENTATION-FIT-MISMATCH: ModelPart2");
  });
});
