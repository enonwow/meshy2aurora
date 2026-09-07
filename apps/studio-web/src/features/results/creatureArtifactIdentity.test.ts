import { describe, expect, it } from "vitest";
import {
  creatureArtifactIdentityTokenV2,
  sha256ArrayBufferHexV1,
} from "./creatureArtifactIdentity";
import { defaultCreatureWeaponGripOptionsV1 } from "../source/weaponGrip";

const base = {
  profile: "PRODUCT_300K" as const,
  sourceForward: "POSITIVE_Z" as const,
  sourceSha256: "a".repeat(64),
  appearanceSha256: "b".repeat(64),
  textureArtifactCleanup: false,
  skinAccessoryStabilizationMode: "AUTO" as const,
  skinAccessorySelectedBoneName: "",
  skinAccessoryComponentBoneOverrides: "",
  weaponGrip: defaultCreatureWeaponGripOptionsV1(),
  heldWeaponMode: "NONE" as const,
};

describe("creatureArtifactIdentityTokenV2", () => {
  it("is deterministic and resref-safe at 70 bits", async () => {
    const first = await creatureArtifactIdentityTokenV2(base);
    const second = await creatureArtifactIdentityTokenV2(base);
    expect(first).toBe(second);
    expect(first).toMatch(/^[a-z2-7]{14}$/);
  });

  it("binds appearance bytes and every output-changing option", async () => {
    const baseline = await creatureArtifactIdentityTokenV2(base);
    const variants = [
      { ...base, appearanceSha256: "c".repeat(64) },
      { ...base, animationEventsSha256: "d".repeat(64) },
      { ...base, textureArtifactCleanup: true },
      { ...base, skinAccessoryStabilizationMode: "KEEP_SOURCE_WEIGHTS" as const },
      {
        ...base,
        skinAccessoryStabilizationMode: "SELECT_BONE" as const,
        skinAccessorySelectedBoneName: "Spine02",
      },
      { ...base, heldWeaponMode: "RIGHT_HAND" as const },
      {
        ...base,
        skinAccessoryStabilizationMode: "SELECT_BONE" as const,
        skinAccessoryComponentBoneOverrides: "0:1=Spine02",
      },
      { ...base, profile: "EXPERIMENTAL_P300K" as const },
      { ...base, sourceForward: "NEGATIVE_X" as const },
      {
        ...base,
        weaponGrip: {
          ...base.weaponGrip,
          mode: "AUTO_PLUS_OFFSETS" as const,
          rightHand: { ...base.weaponGrip.rightHand, rollDegrees: 1 },
        },
      },
    ];
    for (const variant of variants) {
      expect(await creatureArtifactIdentityTokenV2(variant)).not.toBe(baseline);
    }
  });

  it("binds both the Face V2 recipe and its exact texture authoring document", async () => {
    const materialBase = {
      ...base,
      materialSeparationSha256: "c".repeat(64),
      modelTextureAuthoringSha256: "d".repeat(64),
    };
    const baseline = await creatureArtifactIdentityTokenV2(materialBase);

    expect(await creatureArtifactIdentityTokenV2({
      ...materialBase,
      materialSeparationSha256: "e".repeat(64),
    })).not.toBe(baseline);
    expect(await creatureArtifactIdentityTokenV2({
      ...materialBase,
      modelTextureAuthoringSha256: "f".repeat(64),
    })).not.toBe(baseline);
    await expect(creatureArtifactIdentityTokenV2({
      ...base,
      materialSeparationSha256: "c".repeat(64),
    })).rejects.toThrow("Creature material identity requires both recipe hashes");
  });

  it("normalizes the selected bone name only when manual selection is active", async () => {
    expect(await creatureArtifactIdentityTokenV2({
      ...base,
      skinAccessoryStabilizationMode: "SELECT_BONE",
      skinAccessorySelectedBoneName: " Spine02 ",
    })).toBe(await creatureArtifactIdentityTokenV2({
      ...base,
      skinAccessoryStabilizationMode: "SELECT_BONE",
      skinAccessorySelectedBoneName: "spine02",
    }));
    expect(await creatureArtifactIdentityTokenV2({
      ...base,
      skinAccessorySelectedBoneName: "ignored",
      skinAccessoryComponentBoneOverrides: "0:1=ignored",
    })).toBe(await creatureArtifactIdentityTokenV2(base));
  });

  it("canonicalizes per-component override order, whitespace, and bone case", async () => {
    expect(await creatureArtifactIdentityTokenV2({
      ...base,
      skinAccessoryStabilizationMode: "SELECT_BONE",
      skinAccessoryComponentBoneOverrides: "2:4=SPINE\n0:1= Spine02 ",
    })).toBe(await creatureArtifactIdentityTokenV2({
      ...base,
      skinAccessoryStabilizationMode: "SELECT_BONE",
      skinAccessoryComponentBoneOverrides: "0:1=spine02\n2:4=spine",
    }));
  });

  it("rejects a non-canonical animation event sidecar hash", async () => {
    await expect(creatureArtifactIdentityTokenV2({
      ...base,
      animationEventsSha256: "not-a-sha256",
    })).rejects.toThrow("Animation event sidecar has no canonical SHA-256 identity");
  });

  it("hashes exact sidecar bytes deterministically", async () => {
    const bytes = new TextEncoder().encode('{"schemaVersion":1}').buffer;
    expect(await sha256ArrayBufferHexV1(bytes)).toMatch(/^[0-9a-f]{64}$/);
    expect(await sha256ArrayBufferHexV1(bytes)).toBe(await sha256ArrayBufferHexV1(bytes));
  });
});
