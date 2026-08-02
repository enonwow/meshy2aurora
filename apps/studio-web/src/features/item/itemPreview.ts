export interface ItemPreviewBounds {
  readonly field: string;
  readonly min: readonly [number, number, number];
  readonly max: readonly [number, number, number];
}

export interface ItemSeamResult {
  readonly firstField: string;
  readonly secondField: string;
  readonly status: "TOUCHING" | "GAP" | "OVERLAP";
  readonly gap: number;
  readonly overlapVolume: number;
}

export function analyzeItemSeams(
  bounds: readonly ItemPreviewBounds[],
  tolerance: number,
): ItemSeamResult[] {
  if (!Number.isFinite(tolerance) || tolerance < 0) {
    throw new Error("seam tolerance must be a finite non-negative number");
  }
  return bounds.slice(0, -1).map((first, index) => {
    const second = bounds[index + 1];
    const separation = ([0, 1, 2] as const).map((axis) => Math.max(
      first.min[axis] - second.max[axis],
      second.min[axis] - first.max[axis],
      0,
    ));
    const overlap = ([0, 1, 2] as const).map((axis) => Math.max(
      Math.min(first.max[axis], second.max[axis])
        - Math.max(first.min[axis], second.min[axis]),
      0,
    ));
    const gap = Math.hypot(...separation);
    const overlapVolume = overlap[0] * overlap[1] * overlap[2];
    return {
      firstField: first.field,
      secondField: second.field,
      status: overlapVolume > 1e-9
        ? "OVERLAP"
        : gap > tolerance ? "GAP" : "TOUCHING",
      gap,
      overlapVolume,
    };
  });
}
