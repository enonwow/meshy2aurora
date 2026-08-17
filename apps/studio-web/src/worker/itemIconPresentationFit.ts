const SHA256 = /^[a-f0-9]{64}$/;

export interface ItemIconPresentationExpectedPartV1 {
  readonly field: string;
  readonly sourceSha256: string;
  readonly sourceNode: string | null;
}

export interface ItemIconPresentationFitSummaryV1 {
  readonly status: "PASSED";
  readonly sourceFitStatus: "PASSED" | "MANUAL_REQUIRED";
  readonly surfaceContinuityRequired: false;
  readonly algorithm: "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX";
  readonly solutionSha256: string;
  readonly targetAxialLengths: [number, number, number];
  readonly worldAttachmentUnaffected: true;
  readonly parts: Array<{
    field: string;
    sourceSha256: string;
    transformSha256: string;
  }>;
}

export interface ItemIconPresentationFitBindingV1 {
  readonly summary: ItemIconPresentationFitSummaryV1;
  readonly transforms: ReadonlyMap<string, unknown>;
}

type UnknownRecord = Record<string, unknown>;

function record(value: unknown): UnknownRecord | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? value as UnknownRecord
    : undefined;
}

function finiteTuple(value: unknown, length: number) {
  return Array.isArray(value)
    && value.length === length
    && value.every((entry) => typeof entry === "number" && Number.isFinite(entry));
}

function invalidPresentationFit(): never {
  throw new Error(
    "ITEM-ICON-PRESENTATION-FIT-INVALID: the source-bound Aurora YZX icon layout is incomplete or changed",
  );
}

/**
 * Binds a deterministic, source-identical YZX icon layout. Surface continuity
 * is intentionally not a presentation requirement: the emitted shared-canvas
 * icon composite has its own silhouette, order and clipping conformance gate.
 */
export function bindItemIconPresentationFitV1(
  value: unknown,
  expectedParts: readonly ItemIconPresentationExpectedPartV1[],
  targetAxialLengths: readonly [number, number, number],
): ItemIconPresentationFitBindingV1 {
  const report = record(value);
  const orientation = record(report?.orientationFrame);
  const parts = Array.isArray(report?.parts) ? report.parts.map(record) : [];
  const connectors = Array.isArray(report?.adjacentConnectors)
    ? report.adjacentConnectors.map(record)
    : [];
  const sourceFitStatus = report?.status;
  if (
    report?.schemaVersion !== 3
    || report.algorithm !== "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX"
    || (sourceFitStatus !== "PASSED" && sourceFitStatus !== "MANUAL_REQUIRED")
    || typeof report.solutionSha256 !== "string"
    || !SHA256.test(report.solutionSha256)
    || orientation?.status !== "PASSED"
    || orientation.targetAxialAxis !== 1
    || orientation.targetWidthAxis !== 2
    || orientation.targetDepthAxis !== 0
    || typeof orientation.handednessDeterminant !== "number"
    || Math.abs(orientation.handednessDeterminant - 1) > 1e-5
    || expectedParts.length !== 3
    || parts.length !== expectedParts.length
    || parts.some((part) => !part)
    || connectors.length !== expectedParts.length - 1
    || connectors.some((connector, index) => {
      if (!connector) return true;
      const overlap = connector.axialOverlap;
      const requiredMin = connector.requiredMinOverlap;
      const requiredMax = connector.requiredMaxOverlap;
      return connector.firstField !== expectedParts[index].field
        || connector.secondField !== expectedParts[index + 1].field
        || connector.axialAxis !== 1
        || typeof overlap !== "number"
        || typeof requiredMin !== "number"
        || typeof requiredMax !== "number"
        || !Number.isFinite(overlap)
        || !Number.isFinite(requiredMin)
        || !Number.isFinite(requiredMax)
        || overlap <= 0
        || overlap + 1e-6 < requiredMin
        || overlap > requiredMax + 1e-6;
    })
  ) {
    invalidPresentationFit();
  }

  const transforms = new Map<string, unknown>();
  const boundParts = expectedParts.map((expected, index) => {
    const part = parts[index]!;
    const transform = record(part.transform);
    if (
      part.field !== expected.field
      || part.sourceSha256 !== expected.sourceSha256
      || part.sourceNode !== expected.sourceNode
      || part.axialTargetAxis !== 1
      || typeof part.targetAxialLength !== "number"
      || Math.fround(part.targetAxialLength) !== Math.fround(targetAxialLengths[index])
      || typeof part.transformSha256 !== "string"
      || !SHA256.test(part.transformSha256)
      || !transform
      || !finiteTuple(transform.translation, 3)
      || !finiteTuple(transform.rotationXyzw, 4)
      || !finiteTuple(transform.pivot, 3)
      || typeof transform.uniformScale !== "number"
      || !Number.isFinite(transform.uniformScale)
      || transform.uniformScale <= 0
    ) {
      throw new Error(
        `ITEM-ICON-PRESENTATION-FIT-MISMATCH: ${expected.field} is not bound to its exact source and YZX transform`,
      );
    }
    transforms.set(expected.field, part.transform);
    return {
      field: expected.field,
      sourceSha256: expected.sourceSha256,
      transformSha256: part.transformSha256,
    };
  });

  return {
    summary: {
      status: "PASSED",
      sourceFitStatus,
      surfaceContinuityRequired: false,
      algorithm: "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX",
      solutionSha256: report.solutionSha256 as string,
      targetAxialLengths: [...targetAxialLengths],
      worldAttachmentUnaffected: true,
      parts: boundParts,
    },
    transforms,
  };
}
