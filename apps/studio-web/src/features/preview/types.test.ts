import { describe, expect, it } from "vitest";
import { mapReadbackDiagnostics } from "./types";
import type { BinaryMdlInspectionReport } from "./types";

describe("readback diagnostic mapping", () => {
  it("binds only exact parser-reported node offsets", () => {
    const report: BinaryMdlInspectionReport = {
      schemaVersion: 1,
      format: "binary-mdl-v1",
      nodeTree: {
        roots: [{
          offset: 128,
          number: 7,
          name: "body",
          controllers: [],
          children: [],
        }],
      },
      animations: [],
      diagnostics: [
        { schemaVersion: 1, code: "MATCH", severity: "warning", offset: 128, context: "node warning" },
        { schemaVersion: 1, code: "NO_MATCH", severity: "warning", offset: 129, context: "other warning" },
      ],
    };

    const diagnostics = mapReadbackDiagnostics(report);
    expect(diagnostics[0].target).toEqual({ kind: "READBACK_NODE", id: 7, label: "body" });
    expect(diagnostics[1].target).toBeUndefined();
    expect(diagnostics[1].path).toBe("byteOffset:129");
  });
});
