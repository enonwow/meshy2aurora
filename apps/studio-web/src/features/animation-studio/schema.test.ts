import { describe, expect, it } from "vitest";
import type { CreatureAnimationAuthoringV1 } from "../animation-mapping/types";
import rustTypeScriptParityFixture from
  "./fixtures/animation-studio-document-v1.json";
import {
  canonicalFloat32ForWireV1,
  fingerprintAnimationStudioDocumentV1,
  migrateCreatureAnimationAuthoringV1ToV2,
  parseAnimationStudioDocumentV1,
  serializeAnimationStudioDocumentV1,
  validateAnimationStudioSchemaV1,
  validateCreatureAnimationAuthoringV2,
} from "./schema";
import {
  animationStudioClipFixtureV1,
  animationStudioDocumentFixtureV1,
} from "./testFixtures";
import type { AnimationStudioDocumentV1 } from "./types";

describe("Animation Studio V1 schema", () => {
  it("enforces the product clip limit independently from writer boundaries", () => {
    const document = animationStudioDocumentFixtureV1();
    document.authoredClips = Array.from({ length: 65 }, (_, index) => ({
      ...structuredClone(document.authoredClips[0]!),
      id: `authored-${index}`,
      name: `clip_${index}`,
    }));
    expect(validateAnimationStudioSchemaV1(document)).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "M2A-ANIMATION-EDIT-ROW-LIMIT",
          path: "$.authoredClips",
        }),
      ]),
    );
  });

  it("blocks float magnitudes whose JSON notation cannot share one canonical f32 fingerprint", () => {
    const outOfRange = animationStudioDocumentFixtureV1();
    outOfRange.authoredClips[0]!.lengthSeconds = 1e20;
    outOfRange.authoredClips[0]!.tracks[0]!.keyframes[0]!.value[0] = 1e20;
    expect(validateAnimationStudioSchemaV1(outOfRange)).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          path: "$.authoredClips[0].lengthSeconds",
        }),
        expect.objectContaining({
          path: "$.authoredClips[0].tracks[0].keyframes[0].value",
        }),
      ]),
    );

    const exact = animationStudioDocumentFixtureV1();
    exact.authoredClips[0]!.lengthSeconds = 86_400;
    exact.authoredClips[0]!.transitionSeconds = 0;
    exact.authoredClips[0]!.tracks[0]!.keyframes[0]!.value =
      [1_000_000, 0, 0, 1];
    expect(validateAnimationStudioSchemaV1(exact).some(({ message }) => (
      message.includes("exceeds the Animation Studio product limit")
      || message.includes("Transform components must be within")
    ))).toBe(false);
  });

  it("strictly round-trips the canonical wire document", async () => {
    const document = animationStudioDocumentFixtureV1();
    const json = serializeAnimationStudioDocumentV1(document);

    expect(parseAnimationStudioDocumentV1(json)).toEqual({
      kind: "VALID",
      value: document,
    });
    expect(serializeAnimationStudioDocumentV1(document)).toBe(json);
    expect(json).toBe(JSON.stringify(rustTypeScriptParityFixture));
    await expect(fingerprintAnimationStudioDocumentV1(document)).resolves.toBe(
      "4a3b66cd909292c33e7eae7f4e881b359525cd0234b95098084a6d462311d36a",
    );
  });

  it("canonicalizes editor f64 values to the exact Rust f32 wire representation", async () => {
    expect(canonicalFloat32ForWireV1(Math.SQRT1_2)).toBe(0.70710677);
    expect(canonicalFloat32ForWireV1(1 / 30)).toBe(0.033333335);
    expect(canonicalFloat32ForWireV1(-0)).toBe(0);

    const clip = animationStudioClipFixtureV1({
      tracks: [{
        id: "track-torso-rotation",
        targetNodeId: 7,
        path: "ROTATION",
        interpolation: "LINEAR",
        keyframes: [{
          id: "key-f64-editor",
          timeSeconds: 1 / 30,
          value: [0, 0, Math.SQRT1_2, Math.SQRT1_2],
        }],
      }],
      events: [{
        id: "event-f64-editor",
        timeSeconds: 1 / 30,
        name: "impact",
      }],
    });
    const document = animationStudioDocumentFixtureV1({
      authoredClips: [clip],
    });
    const json = serializeAnimationStudioDocumentV1(document);
    expect(json).toContain(
      '"timeSeconds":0.033333335,"value":[0,0,0.70710677,0.70710677]',
    );
    const parsed = parseAnimationStudioDocumentV1(json);
    expect(parsed).toMatchObject({
      kind: "VALID",
      value: {
        authoredClips: [{
          tracks: [{
            keyframes: [{
              timeSeconds: 0.033333335,
              value: [0, 0, 0.70710677, 0.70710677],
            }],
          }],
        }],
      },
    });
    if (parsed.kind !== "VALID") {
      throw new Error("canonical f32 wire document did not parse");
    }
    expect(serializeAnimationStudioDocumentV1(parsed.value)).toBe(json);
    await expect(fingerprintAnimationStudioDocumentV1(document)).resolves
      .toMatch(/^[a-f0-9]{64}$/);
  });

  it("fails closed on unknown fields at every object level", () => {
    const document = animationStudioDocumentFixtureV1() as unknown as Record<string, unknown>;
    document.futureRootField = true;
    const result = parseAnimationStudioDocumentV1(JSON.stringify(document));
    expect(result).toMatchObject({
      kind: "INVALID",
      diagnostics: [expect.objectContaining({
        code: "M2A-ANIMATION-EDIT-SCHEMA",
        path: "$.futureRootField",
      })],
    });

    const nested = animationStudioDocumentFixtureV1();
    const json = JSON.stringify({
      ...nested,
      authoredClips: [{
        ...nested.authoredClips[0],
        source: {
          ...nested.authoredClips[0].source,
          futureSourceField: true,
        },
      }],
    });
    expect(parseAnimationStudioDocumentV1(json)).toMatchObject({
      kind: "INVALID",
      diagnostics: [expect.objectContaining({
        path: "$.authoredClips[0].source.futureSourceField",
      })],
    });
  });

  it("has an explicit version gate", () => {
    expect(parseAnimationStudioDocumentV1(JSON.stringify({
      ...animationStudioDocumentFixtureV1(),
      schemaVersion: 2,
    }))).toMatchObject({
      kind: "INVALID",
      diagnostics: [expect.objectContaining({
        path: "$.schemaVersion",
      })],
    });
  });

  it("fails closed on STEP because the MVP wire contract is LINEAR-only", () => {
    const document = animationStudioDocumentFixtureV1() as unknown as {
      authoredClips: Array<{
        tracks: Array<{ interpolation: string }>;
      }>;
    };
    document.authoredClips[0]!.tracks[0]!.interpolation = "STEP";
    expect(parseAnimationStudioDocumentV1(JSON.stringify(document))).toMatchObject({
      kind: "INVALID",
      diagnostics: [expect.objectContaining({
        code: "M2A-ANIMATION-EDIT-SCHEMA",
        path: "$.authoredClips[0].tracks[0].interpolation",
      })],
    });
  });

  it("blocks duplicate IDs and case-insensitive output names", () => {
    const first = animationStudioClipFixtureV1();
    const duplicate = animationStudioClipFixtureV1({
      name: first.name.toUpperCase(),
    });
    const diagnostics = validateAnimationStudioSchemaV1(
      animationStudioDocumentFixtureV1({ authoredClips: [first, duplicate] }),
    );
    expect(diagnostics.map(({ code }) => code)).toEqual(expect.arrayContaining([
      "M2A-ANIMATION-EDIT-CLIP-DUPLICATE",
      "M2A-ANIMATION-EDIT-CLIP-NAME",
    ]));
  });

  it("marks a clip from another exact source revision as stale", () => {
    const clip = animationStudioClipFixtureV1({
      source: {
        kind: "BLANK_POSE",
        sourceRevision: "b".repeat(64),
        sourceClipName: null,
        sourceClipFingerprint: null,
        proceduralTemplate: null,
      },
    });
    expect(validateAnimationStudioSchemaV1(
      animationStudioDocumentFixtureV1({ authoredClips: [clip] }),
    )).toEqual([expect.objectContaining({
      code: "M2A-ANIMATION-EDIT-SOURCE-STALE",
      path: "$.authoredClips[0].source.sourceRevision",
    })]);
  });

  it("accepts a self-contained imported clip with exact donor provenance", () => {
    const clip = animationStudioClipFixtureV1({
      source: {
        kind: "IMPORTED_MODEL_COPY",
        sourceRevision: "b".repeat(64),
        sourceClipName: "donor_walk",
        sourceClipFingerprint: "c".repeat(64),
        proceduralTemplate: null,
      },
    });
    const document = animationStudioDocumentFixtureV1({ authoredClips: [clip] });

    expect(validateAnimationStudioSchemaV1(document)).toEqual([]);
    expect(parseAnimationStudioDocumentV1(
      serializeAnimationStudioDocumentV1(document),
    )).toEqual({
      kind: "VALID",
      value: document,
    });
  });

  it("fingerprints canonical object keys independently of insertion order", async () => {
    const document = animationStudioDocumentFixtureV1();
    const reordered = {
      authoredClips: document.authoredClips,
      status: document.status,
      authoringRevision: document.authoringRevision,
      sourceRevision: document.sourceRevision,
      schemaVersion: document.schemaVersion,
    } as AnimationStudioDocumentV1;

    expect(serializeAnimationStudioDocumentV1(reordered))
      .toBe(serializeAnimationStudioDocumentV1(document));
    await expect(fingerprintAnimationStudioDocumentV1(reordered))
      .resolves.toBe(await fingerprintAnimationStudioDocumentV1(document));
  });
});

describe("Creature animation authoring V1 to V2", () => {
  const v1Fixture = (): CreatureAnimationAuthoringV1 => ({
    schemaVersion: 1,
    profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
    modelType: "S",
    sourceRevision: "a".repeat(64),
    authoringRevision: 17,
    assignments: [{
      targetSlot: "cwalk",
      sourceKind: "CUSTOM",
      sourceClipName: null,
      customAnimationId: "custom-wave",
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "asset-1",
        ownership: "USER_OWNED",
      },
    }],
    fallbacks: [{
      id: "fallback-1",
      targetSlot: "cdamager",
      sourceSlot: "cdamagel",
      reason: "mirror",
      review: "ACCEPTED",
    }],
    customAnimations: [{
      id: "custom-wave",
      name: "custom_wave",
      playback: "LOOPING_PHASED",
      sourceClipName: null,
      phases: [
        { phase: "START", sourceClipName: "WaveStart" },
        { phase: "LOOP", sourceClipName: "WaveLoop" },
        { phase: "END", sourceClipName: "WaveEnd" },
      ],
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "asset-1",
        ownership: "USER_OWNED",
      },
    }],
  });

  it("migrates deterministically and preserves the exact Base-42 mapping", () => {
    const v1 = v1Fixture();
    const migrated = migrateCreatureAnimationAuthoringV1ToV2(v1);
    expect(migrated).toEqual(migrateCreatureAnimationAuthoringV1ToV2(v1));
    expect(migrated).toMatchObject({
      schemaVersion: 2,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      authoringRevision: 17,
      assignments: v1.assignments,
      fallbacks: v1.fallbacks,
      customAnimations: [{
        id: "custom-wave",
        clipReference: null,
      }],
    });
    expect(migrated.customAnimations[0].phases[0]).toEqual({
      phase: "START",
      clipReference: {
        sourceKind: "SOURCE_CLIP",
        sourceClipName: "WaveStart",
        authoredClipId: null,
      },
    });
    expect(migrated.assignments).not.toBe(v1.assignments);
  });

  it("validates stable authored IDs rather than output names", () => {
    const studio = animationStudioDocumentFixtureV1();
    const migrated = migrateCreatureAnimationAuthoringV1ToV2(v1Fixture());
    migrated.customAnimations = [{
      id: "custom-authored",
      name: "custom_output",
      playback: "ONE_SHOT",
      clipReference: {
        sourceKind: "AUTHORED_CLIP",
        sourceClipName: null,
        authoredClipId: studio.authoredClips[0].id,
      },
      phases: [],
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "asset-1",
        ownership: "USER_OWNED",
      },
    }];
    migrated.assignments = [];
    expect(validateCreatureAnimationAuthoringV2(migrated, studio)).toEqual([]);

    migrated.customAnimations[0].clipReference = {
      sourceKind: "AUTHORED_CLIP",
      sourceClipName: null,
      authoredClipId: "missing",
    };
    expect(validateCreatureAnimationAuthoringV2(migrated, studio))
      .toEqual([expect.objectContaining({
        code: "M2A-ANIMATION-EDIT-SCHEMA",
        path: "$.authoring.customAnimations[0].clipReference",
      })]);
  });
});
