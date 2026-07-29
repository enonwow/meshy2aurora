import { describe, expect, it } from "vitest";
import {
  buildFailureStageLabel,
  CORE_BUILD_FAILURE_STAGES,
  projectBuildFailure,
} from "./projectBuildFailure";

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
    JSON.stringify({ schemaVersion: 1, stage: "FUTURE", code: "M4", path: "model", message: "unknown stage" }),
  ])("keeps non-contract input as one opaque raw message", (rawMessage) => {
    expect(projectBuildFailure(rawMessage)).toEqual({ message: rawMessage });
  });
});

describe("Core build failure stages", () => {
  it.each(CORE_BUILD_FAILURE_STAGES)(
    "projects and labels current Core stage %s",
    (stage) => {
      const failure = projectBuildFailure(JSON.stringify({
        schemaVersion: 1,
        stage,
        code: "M2A-TEST",
        path: "test.path",
        message: "typed failure",
      }));
      expect(failure.stage).toBe(stage);
      expect(buildFailureStageLabel(failure.stage)).not.toBe("Unclassified Worker failure");
    },
  );
});
