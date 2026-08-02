import { describe, expect, it } from "vitest";
import type { SourceInspectionSnapshot } from "../inspect/sourceInspection";
import type { MeshyArtifactProvenance } from "./bridge";
import { validateMeshyImportedSourceV1 } from "./validateMeshyImport";

const provenance: MeshyArtifactProvenance = {
  profileId: "H1-humanoid-animated/v1",
  bridgeProtocolVersion: 2,
  sha256: "a".repeat(64),
  byteLength: 1,
  taskIds: {},
  artifactKind: "MERGED_ANIMATION_GLTF",
  animationArtifacts: [
    { actionId: 0, clipName: "cpause1", sha256: "b".repeat(64), byteLength: 1 },
    { actionId: 198, clipName: "ca1slashl", sha256: "c".repeat(64), byteLength: 1 },
  ],
};

const inspection = {
  source: { sha256: "a".repeat(64) },
  clips: [{ name: "cpause1" }, { name: "ca1slashl" }],
} as SourceInspectionSnapshot;

describe("validateMeshyImportedSourceV1", () => {
  it("accepts the exact Bridge-bound source and ordered clip mapping", () => {
    expect(validateMeshyImportedSourceV1(provenance, inspection)).toBeUndefined();
  });

  it("blocks a stale one-clip import instead of silently building it", () => {
    expect(validateMeshyImportedSourceV1(provenance, {
      ...inspection,
      clips: [{ name: "cpause1" }] as SourceInspectionSnapshot["clips"],
    })).toContain("M2A-MESHY-ANIMATION-CONTRACT-MISMATCH");
  });

  it("blocks source bytes that do not match Bridge provenance", () => {
    expect(validateMeshyImportedSourceV1(provenance, {
      ...inspection,
      source: { ...inspection.source, sha256: "d".repeat(64) },
    } as SourceInspectionSnapshot)).toContain("M2A-MESHY-SOURCE-IDENTITY-MISMATCH");
  });
});
