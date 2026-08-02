import type {
  CreatureSourceForwardV1,
  SkinAccessoryStabilizationModeV1,
} from "../source/InputsPanel";
import { canonicalSkinAccessoryComponentBoneOverridesV2 } from "../source/skinAccessoryOverrides";

const SHA256_HEX = /^[0-9a-f]{64}$/;
const BASE32_ALPHABET = "abcdefghijklmnopqrstuvwxyz234567";

export interface CreatureArtifactIdentityInputV2 {
  readonly profile: "PRODUCT_300K" | "EXPERIMENTAL_P100K" | "EXPERIMENTAL_P300K";
  readonly sourceForward: CreatureSourceForwardV1;
  readonly sourceSha256: string;
  readonly appearanceSha256: string;
  readonly animationEventsSha256?: string;
  readonly textureArtifactCleanup: boolean;
  readonly skinAccessoryStabilizationMode: SkinAccessoryStabilizationModeV1;
  readonly skinAccessorySelectedBoneName: string;
  readonly skinAccessoryComponentBoneOverrides: string;
  readonly materialSeparationSha256?: string;
  readonly modelTextureAuthoringSha256?: string;
}

function requireSha256(value: string, field: string) {
  if (!SHA256_HEX.test(value)) {
    throw new Error(`${field} has no canonical SHA-256 identity`);
  }
}

function base32Prefix(bytes: Uint8Array, characterCount: number) {
  let bits = 0;
  let value = 0;
  let output = "";
  for (const byte of bytes) {
    value = (value << 8) | byte;
    bits += 8;
    while (bits >= 5) {
      output += BASE32_ALPHABET[(value >>> (bits - 5)) & 31];
      bits -= 5;
      if (output.length === characterCount) return output;
    }
  }
  if (bits > 0 && output.length < characterCount) {
    output += BASE32_ALPHABET[(value << (5 - bits)) & 31];
  }
  return output.slice(0, characterCount);
}

/**
 * Produces the 70-bit, SHA-256-derived token used by every generated Creature
 * resref. The seed binds both immutable inputs and every output-changing
 * option, preventing different HAK/MOD bytes from receiving the same normal
 * Studio identity.
 */
export async function sha256ArrayBufferHexV1(bytes: ArrayBuffer) {
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

export async function creatureArtifactIdentityTokenV2(
  input: CreatureArtifactIdentityInputV2,
) {
  requireSha256(input.sourceSha256, "Inspected source");
  requireSha256(input.appearanceSha256, "Inspected appearance.2da");
  if (input.animationEventsSha256 !== undefined) {
    requireSha256(input.animationEventsSha256, "Animation event sidecar");
  }
  if ((input.materialSeparationSha256 === undefined)
    !== (input.modelTextureAuthoringSha256 === undefined)) {
    throw new Error("Creature material identity requires both recipe hashes");
  }
  if (input.materialSeparationSha256 !== undefined) {
    requireSha256(input.materialSeparationSha256, "Material separation recipe");
    requireSha256(input.modelTextureAuthoringSha256!, "Model texture recipe");
  }
  const selectedBoneName = input.skinAccessoryStabilizationMode === "SELECT_BONE"
    ? input.skinAccessorySelectedBoneName.trim().toLowerCase()
    : "";
  const componentBoneOverrides = input.skinAccessoryStabilizationMode === "SELECT_BONE"
    ? canonicalSkinAccessoryComponentBoneOverridesV2(
        input.skinAccessoryComponentBoneOverrides,
      )
    : "";
  const materialIdentityPresent = input.materialSeparationSha256 !== undefined;
  const seed = JSON.stringify({
    schemaVersion: materialIdentityPresent ? 5 : 4,
    profile: input.profile,
    sourceForward: input.sourceForward,
    sourceSha256: input.sourceSha256,
    appearanceSha256: input.appearanceSha256,
    animationEventsSha256: input.animationEventsSha256 ?? null,
    textureArtifactCleanup: input.textureArtifactCleanup,
    skinAccessoryStabilizationMode: input.skinAccessoryStabilizationMode,
    selectedBoneName,
    componentBoneOverrides,
    ...(materialIdentityPresent ? {
      materialSeparationSha256: input.materialSeparationSha256,
      modelTextureAuthoringSha256: input.modelTextureAuthoringSha256,
    } : {}),
  });
  const digest = await globalThis.crypto.subtle.digest("SHA-256", new TextEncoder().encode(seed));
  return base32Prefix(new Uint8Array(digest), 14);
}
