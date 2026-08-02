import { describe, expect, it } from "vitest";
import {
  resolveOwnerAnimationPlaybackProofV1,
  type OwnerAnimationPlaybackProofRegistryEntryV1,
} from "./ownerAnimationPlaybackProofs";

const outputs = {
  model: { byteLength: 1, sha256: "1".repeat(64) },
  hak: { byteLength: 2, sha256: "2".repeat(64) },
  proofModule: { byteLength: 3, sha256: "3".repeat(64) },
};
const fingerprint = "4".repeat(64);
const exactProof: OwnerAnimationPlaybackProofRegistryEntryV1 = {
  schemaVersion: 1,
  evidenceId: "owner-animation-playback-proof-v1",
  evidencePath: "documentation/evidence/owner-animation-playback-proof-v1.md",
  evidenceSha256: "5".repeat(64),
  verifiedAt: "2026-07-30T12:00:00Z",
  animationStudioFingerprintSha256: fingerprint,
  outputs: {
    model: outputs.model.sha256,
    hak: outputs.hak.sha256,
    proofModule: outputs.proofModule.sha256,
  },
  toolset: {
    modelVisibility: "visible",
    proofCompleteness: "verified",
  },
  nwn: {
    modelVisibility: "visible",
    proofCompleteness: "verified",
    animationPlayback: "verified",
  },
};

describe("owner animation playback proof registry", () => {
  it("accepts only an exact fingerprint and output lineage", () => {
    expect(resolveOwnerAnimationPlaybackProofV1(
      outputs,
      fingerprint,
      [exactProof],
    )).toEqual({
      status: "OWNER_VERIFIED",
      playbackProofStatus: "verified",
      proofCompleteness: "verified",
      evidenceId: exactProof.evidenceId,
      evidencePath: exactProof.evidencePath,
      verifiedAt: exactProof.verifiedAt,
    });

    expect(resolveOwnerAnimationPlaybackProofV1(
      outputs,
      "5".repeat(64),
      [exactProof],
    ).status).toBe("OWNER_PROOF_REQUIRED");
    expect(resolveOwnerAnimationPlaybackProofV1(
      {
        ...outputs,
        hak: { ...outputs.hak, sha256: "6".repeat(64) },
      },
      fingerprint,
      [exactProof],
    ).status).toBe("OWNER_PROOF_REQUIRED");
  });

  it("fails closed while the tracked registry has no owner proof", () => {
    expect(resolveOwnerAnimationPlaybackProofV1(
      outputs,
      fingerprint,
    )).toMatchObject({
      status: "OWNER_PROOF_REQUIRED",
      playbackProofStatus: "not_tested",
      proofCompleteness: "missing",
    });
  });

  it("rejects a registry entry without evidence identity or both verified live axes", () => {
    const missingEvidenceHash = {
      ...exactProof,
      evidenceSha256: "",
    } as OwnerAnimationPlaybackProofRegistryEntryV1;
    expect(resolveOwnerAnimationPlaybackProofV1(
      outputs,
      fingerprint,
      [missingEvidenceHash],
    ).status).toBe("OWNER_PROOF_REQUIRED");

    const missingNwnPlayback = {
      ...exactProof,
      nwn: {
        modelVisibility: "visible",
        proofCompleteness: "verified",
        animationPlayback: "not_tested",
      },
    } as unknown as OwnerAnimationPlaybackProofRegistryEntryV1;
    expect(resolveOwnerAnimationPlaybackProofV1(
      outputs,
      fingerprint,
      [missingNwnPlayback],
    ).status).toBe("OWNER_PROOF_REQUIRED");
  });
});
