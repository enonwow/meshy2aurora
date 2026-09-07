import { describe, expect, it } from "vitest";
import {
  parseSavedSupermodelDiagnosticDetailsV1,
  savedSupermodelDiagnosticUrlsV1,
} from "./SavedSupermodelDiagnostic";

describe("savedSupermodelDiagnosticUrlsV1", () => {
  it("resolves one canonical diagnostic directory to the four immutable evidence files", () => {
    expect(savedSupermodelDiagnosticUrlsV1(
      "/@fs/C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-v10-skeleton-repair-motion-three-joint-seam-v9",
      "http://127.0.0.1:5173/",
    )).toEqual({
      report: "http://127.0.0.1:5173/@fs/C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-v10-skeleton-repair-motion-three-joint-seam-v9/base-preview-report.json",
      readback: "http://127.0.0.1:5173/@fs/C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-v10-skeleton-repair-motion-three-joint-seam-v9/base-preview-readback.json",
      authoring: "http://127.0.0.1:5173/@fs/C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-v10-skeleton-repair-motion-three-joint-seam-v9/base-authoring.json",
      targetRig: "http://127.0.0.1:5173/@fs/C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-v10-skeleton-repair-motion-three-joint-seam-v9/base-target-rig.json",
    });
  });

  it("rejects traversal, external origins and non-diagnostic project paths", () => {
    expect(() => savedSupermodelDiagnosticUrlsV1(
      "/@fs/C:/Projects/meshy2aurora/artifacts/diagnostics/../secret",
      "http://127.0.0.1:5173/",
    )).toThrow(/canonical diagnostic directory/i);
    expect(() => savedSupermodelDiagnosticUrlsV1(
      "https://example.com/report",
      "http://127.0.0.1:5173/",
    )).toThrow(/same Studio origin/i);
    expect(() => savedSupermodelDiagnosticUrlsV1(
      "/@fs/C:/Projects/meshy2aurora/sample-3d/borzoi-c-wolf-bind-v6-p300k-v1",
      "http://127.0.0.1:5173/",
    )).toThrow(/canonical diagnostic directory/i);
  });
});

describe("parseSavedSupermodelDiagnosticDetailsV1", () => {
  it("reports the failing per-component samples instead of mislabeling passing aggregate budgets", () => {
    const details = parseSavedSupermodelDiagnosticDetailsV1(JSON.stringify({
      motionQuality: {
        perClipDeformationCoverage: false,
        perComponentDeformationCoverage: false,
        clips: [{ components: [
          { pass: true, maxEdgeRatio: 2, minEdgeRatio: 0.5, maxTriangleAreaRatio: 3, minTriangleAreaRatio: 0.4 },
          { pass: false, maxEdgeRatio: 34.25, minEdgeRatio: 0.03, maxTriangleAreaRatio: 31.75, minTriangleAreaRatio: 0.004 },
        ] }],
      },
    }));
    expect(details).toEqual({
      perClipDeformationCoverage: false,
      perComponentDeformationCoverage: false,
      failedComponentSampleCount: 1,
      worstMaxEdgeRatio: 34.25,
      worstMinEdgeRatio: 0.03,
      worstMaxTriangleAreaRatio: 31.75,
      worstMinTriangleAreaRatio: 0.004,
    });
  });
});
