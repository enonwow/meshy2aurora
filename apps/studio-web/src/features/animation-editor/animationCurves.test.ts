import { describe, expect, it } from "vitest";
import type { AuthoredAnimationTrackV1 } from "../animation-studio/types";
import {
  createAutoCurveFromTrackV1,
  parseAnimationCurveResampleReportV1,
} from "./animationCurves";

describe("animation curve UI adapter", () => {
  it("derives deterministic translation tangents without implementing resampling", () => {
    const track: AuthoredAnimationTrackV1 = {
      id: "hand-translation",
      targetNodeId: 7,
      path: "TRANSLATION",
      interpolation: "LINEAR",
      keyframes: [
        { id: "a", timeSeconds: 0, value: [0, 0, 0] },
        { id: "b", timeSeconds: 1, value: [2, 0, 0] },
        { id: "c", timeSeconds: 2, value: [4, 2, 0] },
      ],
    };
    const curve = createAutoCurveFromTrackV1(track);

    expect(curve.keys[1]?.inTangent).toEqual([2, 1, 0]);
    expect(curve.keys[1]?.outTangent).toEqual([2, 1, 0]);
  });

  it("parses a source-independent Core resample report", () => {
    const parsed = parseAnimationCurveResampleReportV1(JSON.stringify({
      schemaVersion: 1,
      editorKeyCount: 3,
      linearKeyCount: 9,
      maxPositionError: 0.004,
      maxAngularErrorRadians: 0,
      track: {
        id: "hand-translation",
        targetNodeId: 7,
        path: "TRANSLATION",
        interpolation: "LINEAR",
        keyframes: [],
      },
      fingerprintSha256: "a".repeat(64),
    }));

    expect(parsed.linearKeyCount).toBe(9);
    expect(parsed.track.interpolation).toBe("LINEAR");
  });
});
