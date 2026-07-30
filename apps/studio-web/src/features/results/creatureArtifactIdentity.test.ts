import { describe, expect, it } from "vitest";
import {
  creatureArtifactIdentityTokenV2,
  sha256ArrayBufferHexV1,
} from "./creatureArtifactIdentity";

const base = {
  profile: "PRODUCT_300K" as const,
  sourceSha256: "a".repeat(64),
  appearanceSha256: "b".repeat(64),
  textureArtifactCleanup: false,
  skinAccessoryStabilizationMode: "AUTO" as const,
  skinAccessorySelectedBoneName: "",
  skinAccessoryComponentBoneOverrides: "",
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
      {
        ...base,
        skinAccessoryStabilizationMode: "SELECT_BONE" as const,
        skinAccessoryComponentBoneOverrides: "0:1=Spine02",
      },
      { ...base, profile: "EXPERIMENTAL_P300K" as const },
    ];
    for (const variant of variants) {
      expect(await creatureArtifactIdentityTokenV2(variant)).not.toBe(baseline);
    }
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
