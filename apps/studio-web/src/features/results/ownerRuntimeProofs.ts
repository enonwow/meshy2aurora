export interface CanonicalOutputIdentity {
  byteLength: number;
  sha256: string;
}

export type CanonicalRuntimeAcceptance =
  | {
    status: "OPEN_M6";
    reason: string;
  }
  | {
    status: "OWNER_VERIFIED";
    evidenceId: string;
    evidencePath: string;
    verifiedAt: string;
    toolset: {
      modelVisibility: "visible";
      proofCompleteness: "verified";
    };
    nwn: {
      modelVisibility: "visible";
      proofCompleteness: "verified";
    };
    geometryArtifactResult: "fixed";
  };

interface OwnerRuntimeProofRegistryEntryV1 {
  schemaVersion: 1;
  evidenceId: string;
  evidencePath: string;
  verifiedAt: string;
  outputs: {
    model: string;
    texture: string;
    hak: string;
    proofModule: string;
  };
}

const OWNER_RUNTIME_PROOF_REGISTRY_V1: readonly OwnerRuntimeProofRegistryEntryV1[] = [
  {
    schemaVersion: 1,
    evidenceId: "tlc-stoneback-brute-p300k-geometry-ab-v5-2026-07-29",
    evidencePath:
      "documentation/evidence/tlc-stoneback-brute-p300k-geometry-ab-v5-ready-for-owner-proof-2026-07-29.md",
    verifiedAt: "2026-07-29",
    outputs: {
      model: "c8b173a59a576911ff938b7287f823694474be838a81cb55c370c8c3258eb6e9",
      texture: "ac6aceb3f2809c5ffe1170d2525df672fb077855cf8bc1d78f24241596f05fbc",
      hak: "90d79114f0bc725e91f54657dc39f7491862ab4c0f54e032703ac4fddd6c9853",
      proofModule: "da60bc35b45f7ce53753ed91d6c2c13ce09192b6d63143acf2fffca2c94cf1df",
    },
  },
];

const OPEN_M6: CanonicalRuntimeAcceptance = {
  status: "OPEN_M6",
  reason: "No exact owner-verified Aurora/NWN lineage matches these output hashes.",
};

export function resolveOwnerRuntimeProofV1(
  outputs: Readonly<Record<string, CanonicalOutputIdentity | undefined>>,
): CanonicalRuntimeAcceptance {
  const proof = OWNER_RUNTIME_PROOF_REGISTRY_V1.find((candidate) =>
    (["model", "texture", "hak", "proofModule"] as const).every(
      (name) => outputs[name]?.sha256 === candidate.outputs[name],
    ));
  if (!proof) return OPEN_M6;

  return {
    status: "OWNER_VERIFIED",
    evidenceId: proof.evidenceId,
    evidencePath: proof.evidencePath,
    verifiedAt: proof.verifiedAt,
    toolset: {
      modelVisibility: "visible",
      proofCompleteness: "verified",
    },
    nwn: {
      modelVisibility: "visible",
      proofCompleteness: "verified",
    },
    geometryArtifactResult: "fixed",
  };
}
