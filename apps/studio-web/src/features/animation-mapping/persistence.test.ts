// @vitest-environment jsdom

import { beforeEach, describe, expect, it } from "vitest";
import { createCreatureAnimationAuthoringV1 } from "./state";
import {
  loadCreatureAnimationDraftV1,
  migrateCreatureAnimationDraftV1,
  parseCreatureAnimationAuthoringV1,
  saveCreatureAnimationDraftV1,
  serializeCreatureAnimationAuthoringV1,
} from "./persistence";

describe("creature animation authoring persistence v1", () => {
  beforeEach(() => localStorage.clear());

  it("round-trips a stable versioned JSON document", () => {
    const authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    const serialized = serializeCreatureAnimationAuthoringV1(authoring);
    expect(serialized).toBe(serializeCreatureAnimationAuthoringV1(authoring));
    expect(parseCreatureAnimationAuthoringV1(serialized)).toEqual({
      kind: "VALID",
      value: authoring,
    });
    expect(migrateCreatureAnimationDraftV1(JSON.parse(serialized))).toEqual({
      kind: "VALID",
      value: authoring,
    });
  });

  it("saves and loads a project-scoped draft with an explicit status", () => {
    const authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    authoring.fallbacks.push({
      id: "fallback",
      targetSlot: "cdamager",
      sourceSlot: "cdamagel",
      reason: "Mirror",
      review: "ACCEPTED",
    });
    authoring.customAnimations.push({
      id: "wave",
      name: "custom1",
      playback: "ONE_SHOT",
      sourceClipName: "Wave",
      phases: [],
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "source",
        ownership: "USER_OWNED",
      },
    });
    expect(saveCreatureAnimationDraftV1("project-1", authoring, localStorage))
      .toMatchObject({ kind: "SAVED" });
    expect(loadCreatureAnimationDraftV1("project-1", localStorage)).toEqual({
      kind: "LOADED",
      value: authoring,
    });
    expect(loadCreatureAnimationDraftV1("missing", localStorage)).toEqual({
      kind: "EMPTY",
    });
  });

  it("reports corrupt and newer drafts without throwing", () => {
    expect(parseCreatureAnimationAuthoringV1("{not json")).toMatchObject({
      kind: "INVALID",
      diagnostics: [expect.objectContaining({
        code: "M2A-ANIMATION-DRAFT-JSON",
      })],
    });
    expect(migrateCreatureAnimationDraftV1({
      schemaVersion: 2,
      profile: "FUTURE",
    })).toMatchObject({
      kind: "INVALID",
      diagnostics: [expect.objectContaining({
        code: "M2A-ANIMATION-DRAFT-NEWER-SCHEMA",
      })],
    });
  });

  it("returns an autosave error when storage is unavailable", () => {
    const unavailable: Storage = {
      length: 0,
      clear: () => undefined,
      getItem: () => null,
      key: () => null,
      removeItem: () => undefined,
      setItem: () => {
        throw new DOMException("quota", "QuotaExceededError");
      },
    };
    expect(saveCreatureAnimationDraftV1(
      "project-1",
      createCreatureAnimationAuthoringV1("sha256:source", "S"),
      unavailable,
    )).toMatchObject({
      kind: "ERROR",
      message: expect.stringMatching(/quota/i),
    });
  });
});
