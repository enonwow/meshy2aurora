import { describe, expect, it } from "vitest";
import { analyzeItemSeams } from "./itemPreview";

describe("Item seam analysis", () => {
  it("distinguishes touching, tolerated gaps, real gaps and overlap", () => {
    const result = analyzeItemSeams([
      { field: "bottom", min: [0, 0, 0], max: [1, 1, 1] },
      { field: "middle", min: [1, 0, 0], max: [2, 1, 1] },
      { field: "top", min: [2.005, 0, 0], max: [3, 1, 1] },
      { field: "cap", min: [3.25, 0, 0], max: [4, 1, 1] },
      { field: "overlap", min: [3.75, 0, 0], max: [4.5, 1, 1] },
    ], 0.01);

    expect(result.map((seam) => seam.status)).toEqual([
      "TOUCHING",
      "TOUCHING",
      "GAP",
      "OVERLAP",
    ]);
    expect(result[2].gap).toBeCloseTo(0.25);
    expect(result[3].overlapVolume).toBeCloseTo(0.25);
  });

  it("rejects an invalid tolerance", () => {
    expect(() => analyzeItemSeams([], -1)).toThrow("non-negative");
  });
});
