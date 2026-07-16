import { describe, expect, it } from "vitest";
import { buildStageForPipelineStage, projectBuildFailure } from "./projectBuildFailure";

describe("projectBuildFailure", () => {
  it("strictly projects the exact M6PipelineErrorV1 JSON contract", () => {
    expect(projectBuildFailure(JSON.stringify({
      schemaVersion: 1,
      stage: "MODEL",
      code: "M4-WRITER-FAILED",
      path: "model.nodes[2]",
      message: "binary writer rejected node 2",
    }))).toEqual({
      stage: "MODEL",
      code: "M4-WRITER-FAILED",
      path: "model.nodes[2]",
      message: "binary writer rejected node 2",
    });
  });

  it.each([
    "plain worker failure",
    "{",
    JSON.stringify({ schemaVersion: 1, code: "M4-WRITER-FAILED", message: "missing stage and path" }),
    JSON.stringify({ schemaVersion: 2, stage: "MODEL", code: "M4", path: "model", message: "wrong schema" }),
    JSON.stringify({ schemaVersion: 1, stage: "MODEL", code: "M4", path: "model", message: "extra", unknown: true }),
  ])("keeps non-contract input as one opaque raw message", (rawMessage) => {
    expect(projectBuildFailure(rawMessage)).toEqual({ message: rawMessage });
  });
});

describe("buildStageForPipelineStage", () => {
  it.each([
    ["INGEST", "INGEST_SOURCE"],
    ["PROFILE", "NORMALIZE_CANONICAL_IR"],
    ["MODEL", "WRITE_BINARY_MDL"],
    ["READBACK", "CANONICAL_BINARY_READBACK"],
    ["APPEARANCE", "UPDATE_APPEARANCE_2DA"],
    ["HAK", "PACKAGE_HAK"],
  ] as const)("maps unambiguous %s evidence to %s", (pipelineStage, buildStage) => {
    expect(buildStageForPipelineStage(pipelineStage)).toBe(buildStage);
  });

  it.each([undefined, "ANIMATION", "TEXTURE", "PACKAGE", "FIXTURE", "model", ""])(
    "leaves unsupported stage %s unknown",
    (stage) => {
      expect(buildStageForPipelineStage(stage)).toBeUndefined();
    },
  );
});
