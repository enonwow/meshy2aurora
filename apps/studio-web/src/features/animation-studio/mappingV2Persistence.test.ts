import { describe, expect, it } from "vitest";
import {
  loadCreatureAnimationAuthoringV2DraftV1,
  parseCreatureAnimationAuthoringV2,
  saveCreatureAnimationAuthoringV2DraftV1,
  serializeCreatureAnimationAuthoringV2,
} from "./mappingV2Persistence";
import { migrateCreatureAnimationAuthoringV1ToV2 } from "./schema";

const sourceRevision = "a".repeat(64);

function fixture() {
  const value = migrateCreatureAnimationAuthoringV1ToV2({
    schemaVersion: 1,
    profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
    modelType: "S",
    sourceRevision,
    authoringRevision: 3,
    assignments: [],
    fallbacks: [],
    customAnimations: [],
  });
  value.customAnimations.push({
    id: "custom-1",
    name: "custom_wave",
    playback: "ONE_SHOT",
    clipReference: {
      sourceKind: "AUTHORED_CLIP",
      sourceClipName: null,
      authoredClipId: "authored-1",
    },
    phases: [],
    provenance: {
      provider: "USER_CUSTOM",
      assetId: "authored-1",
      ownership: "USER_OWNED",
    },
  });
  value.assignments.push({
    targetSlot: "ca1slashl",
    sourceKind: "CUSTOM",
    sourceClipName: null,
    customAnimationId: "custom-1",
    provenance: {
      provider: "USER_CUSTOM",
      assetId: "custom-1",
      ownership: "USER_OWNED",
    },
  });
  return value;
}

describe("CreatureAnimationAuthoringV2 draft persistence", () => {
  it("round-trips exact authored IDs and Base 42 assignments", () => {
    const values = new Map<string, string>();
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
    };
    expect(saveCreatureAnimationAuthoringV2DraftV1(
      sourceRevision,
      fixture(),
      storage,
    )).toEqual({ kind: "SAVED" });
    expect(loadCreatureAnimationAuthoringV2DraftV1(sourceRevision, storage))
      .toEqual({ kind: "LOADED", value: fixture() });
  });

  it("has stable serialization and rejects unknown fields", () => {
    const serialized = serializeCreatureAnimationAuthoringV2(fixture());
    expect(serializeCreatureAnimationAuthoringV2(
      parseCreatureAnimationAuthoringV2(serialized),
    )).toBe(serialized);
    const value = JSON.parse(serialized);
    value.customAnimations[0].absolutePath = "C:/forbidden.glb";
    expect(() => parseCreatureAnimationAuthoringV2(JSON.stringify(value)))
      .toThrow(/unknown fields/);
  });

  it("fails closed on unknown wire enum values", () => {
    const assignmentValue = JSON.parse(
      serializeCreatureAnimationAuthoringV2(fixture()),
    );
    assignmentValue.assignments[0].sourceKind = "FUTURE_SOURCE";
    expect(() => parseCreatureAnimationAuthoringV2(
      JSON.stringify(assignmentValue),
    )).toThrow("$.assignments[0].sourceKind must be one of");

    const customValue = JSON.parse(
      serializeCreatureAnimationAuthoringV2(fixture()),
    );
    customValue.customAnimations[0].playback = "PING_PONG";
    expect(() => parseCreatureAnimationAuthoringV2(
      JSON.stringify(customValue),
    )).toThrow("$.customAnimations[0].playback must be one of");
  });

  it("fails closed when storage rejects a write", () => {
    const storage = {
      setItem() {
        throw new DOMException("quota", "QuotaExceededError");
      },
    };
    expect(saveCreatureAnimationAuthoringV2DraftV1(
      sourceRevision,
      fixture(),
      storage,
    )).toMatchObject({
      kind: "ERROR",
      diagnostic: { level: "BLOCKING" },
    });
  });

  it("does not persist V2 while a V1 mapping is only read and migrated", () => {
    let reads = 0;
    let writes = 0;
    const storage = {
      getItem() {
        reads += 1;
        return null;
      },
      setItem() {
        writes += 1;
      },
    };
    const loaded = loadCreatureAnimationAuthoringV2DraftV1(
      sourceRevision,
      storage,
    );
    const migrated = migrateCreatureAnimationAuthoringV1ToV2({
      schemaVersion: 1,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      modelType: "S",
      sourceRevision,
      authoringRevision: 3,
      assignments: [],
      fallbacks: [],
      customAnimations: [],
    });

    expect(loaded).toEqual({ kind: "EMPTY" });
    expect(migrated.schemaVersion).toBe(2);
    expect(reads).toBe(1);
    expect(writes).toBe(0);
  });
});
