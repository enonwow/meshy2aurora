import registryJson from "../../../../../contracts/owner-animation-playback-proofs-v1.json";
import type { CanonicalOutputIdentity } from "./ownerRuntimeProofs";

export interface OwnerAnimationPlaybackProofRegistryEntryV1 {
  readonly schemaVersion: 1;
  readonly evidenceId: string;
  readonly evidencePath: string;
  readonly evidenceSha256: string;
  readonly verifiedAt: string;
  readonly animationStudioFingerprintSha256: string;
  readonly animationPreset?: {
    readonly presetId: string;
    readonly presetVersion: number;
    readonly motionSha256: string;
    readonly rigSignatureSha256: string;
  };
  readonly outputs: {
    readonly model: string;
    readonly hak: string;
    readonly proofModule: string;
  };
  readonly toolset: {
    readonly modelVisibility: "visible";
    readonly proofCompleteness: "verified";
  };
  readonly nwn: {
    readonly modelVisibility: "visible";
    readonly proofCompleteness: "verified";
    readonly animationPlayback: "verified";
  };
}

export type OwnerAnimationPlaybackProofResolutionV1 =
  | {
    readonly status: "OWNER_PROOF_REQUIRED";
    readonly playbackProofStatus: "not_tested";
    readonly proofCompleteness: "missing";
    readonly reason: string;
  }
  | {
    readonly status: "OWNER_VERIFIED";
    readonly playbackProofStatus: "verified";
    readonly proofCompleteness: "verified";
    readonly evidenceId: string;
    readonly evidencePath: string;
    readonly verifiedAt: string;
  };

function sha256(value: unknown, path: string): string {
  if (typeof value !== "string" || !/^[0-9a-f]{64}$/.test(value)) {
    throw new Error(`Invalid owner animation playback proof registry field ${path}.`);
  }
  return value;
}

function nonEmptyString(value: unknown, path: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`Invalid owner animation playback proof registry field ${path}.`);
  }
  return value;
}

function verifiedAt(value: unknown, path: string): string {
  const date = nonEmptyString(value, path);
  if (
    !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/.test(date)
    || !Number.isFinite(Date.parse(date))
  ) {
    throw new Error(`Invalid owner animation playback proof registry field ${path}.`);
  }
  return date;
}

function animationPresetProof(
  value: unknown,
  path: string,
): NonNullable<OwnerAnimationPlaybackProofRegistryEntryV1["animationPreset"]> {
  if (
    typeof value !== "object"
    || value === null
    || !("presetId" in value)
    || typeof value.presetId !== "string"
    || !/^[A-Za-z0-9_-]{1,64}$/.test(value.presetId)
    || !("presetVersion" in value)
    || !Number.isSafeInteger(value.presetVersion)
    || (value.presetVersion as number) < 1
    || !("motionSha256" in value)
    || !("rigSignatureSha256" in value)
  ) {
    throw new Error(`Invalid owner animation playback proof registry field ${path}.`);
  }
  return {
    presetId: value.presetId,
    presetVersion: value.presetVersion as number,
    motionSha256: sha256(value.motionSha256, `${path}.motionSha256`),
    rigSignatureSha256: sha256(
      value.rigSignatureSha256,
      `${path}.rigSignatureSha256`,
    ),
  };
}

function verifiedToolsetAxis(
  value: unknown,
  path: string,
): OwnerAnimationPlaybackProofRegistryEntryV1["toolset"] {
  if (
    typeof value !== "object"
    || value === null
    || !("modelVisibility" in value)
    || value.modelVisibility !== "visible"
    || !("proofCompleteness" in value)
    || value.proofCompleteness !== "verified"
  ) {
    throw new Error(`Invalid owner animation playback proof registry field ${path}.`);
  }
  return {
    modelVisibility: "visible",
    proofCompleteness: "verified",
  };
}

function verifiedNwnAxis(
  value: unknown,
  path: string,
): OwnerAnimationPlaybackProofRegistryEntryV1["nwn"] {
  if (
    typeof value !== "object"
    || value === null
    || !("modelVisibility" in value)
    || value.modelVisibility !== "visible"
    || !("proofCompleteness" in value)
    || value.proofCompleteness !== "verified"
    || !("animationPlayback" in value)
    || value.animationPlayback !== "verified"
  ) {
    throw new Error(`Invalid owner animation playback proof registry field ${path}.`);
  }
  return {
    modelVisibility: "visible",
    proofCompleteness: "verified",
    animationPlayback: "verified",
  };
}

function loadOwnerAnimationPlaybackProofRegistryV1(): readonly OwnerAnimationPlaybackProofRegistryEntryV1[] {
  const registry: unknown = registryJson;
  if (
    typeof registry !== "object"
    || registry === null
    || !("schemaVersion" in registry)
    || registry.schemaVersion !== 1
    || !("entries" in registry)
    || !Array.isArray(registry.entries)
  ) {
    throw new Error("Invalid owner animation playback proof registry.");
  }
  const evidenceIds = new Set<string>();
  return (registry.entries as unknown[]).map((raw, index) => {
    if (
      typeof raw !== "object"
      || raw === null
      || !("schemaVersion" in raw)
      || raw.schemaVersion !== 1
    ) {
      throw new Error(`Invalid owner animation playback proof registry entry ${index}.`);
    }
    const entry = raw as Record<string, unknown>;
    const evidenceId = nonEmptyString(entry.evidenceId, `entries[${index}].evidenceId`);
    if (evidenceIds.has(evidenceId)) {
      throw new Error(`Duplicate owner animation playback proof evidenceId ${evidenceId}.`);
    }
    evidenceIds.add(evidenceId);
    const evidencePath = nonEmptyString(
      entry.evidencePath,
      `entries[${index}].evidencePath`,
    );
    if (
      !/^documentation\/evidence\/[A-Za-z0-9._/-]+\.md$/.test(evidencePath)
      || evidencePath.includes("..")
    ) {
      throw new Error(
        `Invalid owner animation playback proof registry field entries[${index}].evidencePath.`,
      );
    }
    if (typeof entry.outputs !== "object" || entry.outputs === null) {
      throw new Error(`Invalid owner animation playback proof registry field entries[${index}].outputs.`);
    }
    const outputs = entry.outputs as Record<string, unknown>;
    const animationPreset = entry.animationPreset === undefined
      ? undefined
      : animationPresetProof(
        entry.animationPreset,
        `entries[${index}].animationPreset`,
      );
    return {
      schemaVersion: 1,
      evidenceId,
      evidencePath,
      evidenceSha256: sha256(
        entry.evidenceSha256,
        `entries[${index}].evidenceSha256`,
      ),
      verifiedAt: verifiedAt(
        entry.verifiedAt,
        `entries[${index}].verifiedAt`,
      ),
      animationStudioFingerprintSha256: sha256(
        entry.animationStudioFingerprintSha256,
        `entries[${index}].animationStudioFingerprintSha256`,
      ),
      ...(animationPreset === undefined ? {} : { animationPreset }),
      outputs: {
        model: sha256(outputs.model, `entries[${index}].outputs.model`),
        hak: sha256(outputs.hak, `entries[${index}].outputs.hak`),
        proofModule: sha256(
          outputs.proofModule,
          `entries[${index}].outputs.proofModule`,
        ),
      },
      toolset: verifiedToolsetAxis(
        entry.toolset,
        `entries[${index}].toolset`,
      ),
      nwn: verifiedNwnAxis(
        entry.nwn,
        `entries[${index}].nwn`,
      ),
    };
  });
}

const OWNER_ANIMATION_PLAYBACK_PROOF_REGISTRY_V1 =
  loadOwnerAnimationPlaybackProofRegistryV1();

const OWNER_PROOF_REQUIRED: OwnerAnimationPlaybackProofResolutionV1 = {
  status: "OWNER_PROOF_REQUIRED",
  playbackProofStatus: "not_tested",
  proofCompleteness: "missing",
  reason:
    "No exact owner-verified Aurora/NWN animation playback proof matches the Animation Studio fingerprint and output hashes.",
};

export function resolveOwnerAnimationPlaybackProofV1(
  outputs: Readonly<Record<string, CanonicalOutputIdentity | undefined>>,
  animationStudioFingerprintSha256: string | undefined,
  registry: readonly OwnerAnimationPlaybackProofRegistryEntryV1[] =
    OWNER_ANIMATION_PLAYBACK_PROOF_REGISTRY_V1,
): OwnerAnimationPlaybackProofResolutionV1 {
  if (!animationStudioFingerprintSha256) return OWNER_PROOF_REQUIRED;
  const proof = registry.find((candidate) => (
    /^[0-9a-f]{64}$/.test(candidate.evidenceSha256)
    && /^documentation\/evidence\/[A-Za-z0-9._/-]+\.md$/.test(
      candidate.evidencePath,
    )
    && !candidate.evidencePath.includes("..")
    && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/.test(
      candidate.verifiedAt,
    )
    && candidate.toolset?.modelVisibility === "visible"
    && candidate.toolset?.proofCompleteness === "verified"
    && candidate.nwn?.modelVisibility === "visible"
    && candidate.nwn?.proofCompleteness === "verified"
    && candidate.nwn?.animationPlayback === "verified"
    &&
    candidate.animationStudioFingerprintSha256 === animationStudioFingerprintSha256
    && outputs.model?.sha256 === candidate.outputs.model
    && outputs.hak?.sha256 === candidate.outputs.hak
    && outputs.proofModule?.sha256 === candidate.outputs.proofModule
  ));
  if (!proof) return OWNER_PROOF_REQUIRED;

  return {
    status: "OWNER_VERIFIED",
    playbackProofStatus: "verified",
    proofCompleteness: "verified",
    evidenceId: proof.evidenceId,
    evidencePath: proof.evidencePath,
    verifiedAt: proof.verifiedAt,
  };
}
