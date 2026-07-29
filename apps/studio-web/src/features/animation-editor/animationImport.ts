import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";

export const ANIMATION_IMPORT_RIG_MISMATCH_V1 =
  "M2A-ANIMATION-IMPORT-RIG-MISMATCH";

export interface ExternalAnimationClipV1 {
  readonly name: string;
  readonly durationSeconds: number;
  readonly trackCount: number;
}

export interface ExternalAnimationSourceInspectionV1 {
  readonly sourceRevision: string;
  readonly rig: readonly AnimationRigNodeV1[];
  readonly clips: readonly ExternalAnimationClipV1[];
}

export interface AnimationRigCompatibilityV1 {
  readonly compatible: boolean;
  readonly code: typeof ANIMATION_IMPORT_RIG_MISMATCH_V1 | null;
  readonly message: string;
  readonly mismatches: readonly string[];
}

export function compareAnimationImportRigsV1(
  currentRig: readonly AnimationRigNodeV1[],
  donorRig: readonly AnimationRigNodeV1[],
  epsilon = 1e-5,
): AnimationRigCompatibilityV1 {
  const mismatches: string[] = [];
  if (currentRig.length !== donorRig.length) {
    mismatches.push(
      `Bone count differs: current ${currentRig.length}, donor ${donorRig.length}.`,
    );
  }
  const donorById = new Map(donorRig.map((node) => [node.id, node]));
  for (const current of currentRig) {
    const donor = donorById.get(current.id);
    if (!donor) {
      mismatches.push(`Donor rig has no bone ID ${current.id} (${current.name}).`);
      continue;
    }
    if (current.name !== donor.name) {
      mismatches.push(
        `Bone ID ${current.id} is ${current.name} in the current rig and ${donor.name} in the donor.`,
      );
    }
    if (current.parentId !== donor.parentId) {
      mismatches.push(
        `Parent differs for ${current.name}: current ${String(current.parentId)}, donor ${String(donor.parentId)}.`,
      );
    }
    if (!vectorsEqual(current.translation, donor.translation, epsilon)) {
      mismatches.push(`Rest translation differs for ${current.name}.`);
    }
    if (!rotationsEqual(current.rotation, donor.rotation, epsilon)) {
      mismatches.push(`Rest rotation differs for ${current.name}.`);
    }
  }
  for (const donor of donorRig) {
    if (!currentRig.some(({ id }) => id === donor.id)) {
      mismatches.push(`Donor-only bone ID ${donor.id} (${donor.name}).`);
    }
  }
  if (mismatches.length === 0) {
    return {
      compatible: true,
      code: null,
      message: "Exact output rig match. Animation tracks can be copied safely.",
      mismatches,
    };
  }
  return {
    compatible: false,
    code: ANIMATION_IMPORT_RIG_MISMATCH_V1,
    message: "The donor uses a different output rig. Automatic retargeting is not available yet.",
    mismatches,
  };
}

export function createImportedModelClipV1(
  projected: AuthoredAnimationClipV1,
  input: {
    readonly id: string;
    readonly name: string;
    readonly donorSourceRevision: string;
  },
): AuthoredAnimationClipV1 {
  if (
    projected.source.sourceClipName === null
    || projected.source.sourceClipFingerprint === null
  ) {
    throw new Error("The donor clip has no exact source name and fingerprint.");
  }
  return {
    ...projected,
    id: input.id,
    name: input.name,
    status: "DRAFT",
    source: {
      kind: "IMPORTED_MODEL_COPY",
      sourceRevision: input.donorSourceRevision,
      sourceClipName: projected.source.sourceClipName,
      sourceClipFingerprint: projected.source.sourceClipFingerprint,
      proceduralTemplate: null,
    },
    revision: 1,
  };
}

function vectorsEqual(
  left: readonly number[],
  right: readonly number[],
  epsilon: number,
) {
  return left.length === right.length && left.every(
    (value, index) => Math.abs(value - (right[index] ?? Number.NaN)) <= epsilon,
  );
}

function rotationsEqual(
  left: readonly number[],
  right: readonly number[],
  epsilon: number,
) {
  if (vectorsEqual(left, right, epsilon)) return true;
  return left.length === right.length && left.every(
    (value, index) => Math.abs(value + (right[index] ?? Number.NaN)) <= epsilon,
  );
}
