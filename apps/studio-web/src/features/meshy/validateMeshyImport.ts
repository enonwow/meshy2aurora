import type { SourceInspectionSnapshot } from "../inspect/sourceInspection";
import type { MeshyArtifactProvenance } from "./bridge";

export function validateMeshyImportedSourceV1(
  provenance: MeshyArtifactProvenance | undefined,
  inspection: SourceInspectionSnapshot,
): string | undefined {
  if (!provenance) return undefined;
  if (inspection.source.sha256 !== provenance.sha256) {
    return "M2A-MESHY-SOURCE-IDENTITY-MISMATCH: inspected GLB does not match the imported Bridge provenance.";
  }
  const expected = provenance.animationArtifacts?.map(({ clipName }) => clipName);
  if (!expected?.length) return undefined;
  const actual = inspection.clips.map(({ name }) => name);
  if (
    actual.length !== expected.length
    || actual.some((name, index) => name !== expected[index])
  ) {
    return `M2A-MESHY-ANIMATION-CONTRACT-MISMATCH: expected ${expected.length} named clips [${expected.join(", ")}], inspected ${actual.length} [${actual.map((name) => name ?? "<unnamed>").join(", ")}].`;
  }
  return undefined;
}
