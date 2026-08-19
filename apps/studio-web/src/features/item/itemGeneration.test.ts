import { describe, expect, it } from "vitest";

import {
  createItemGenerationPlan,
  createItemGenerationSession,
  deserializeItemGenerationSession,
  bindItemGenerationTask,
  recordItemGenerationArtifact,
  recordRecoveredItemGenerationArtifact,
  recordItemGenerationRun,
  serializeItemGenerationSession,
  setItemGenerationConcept,
} from "./itemGeneration";
import type { ItemBaseItemRow } from "./types";

function longsword(): ItemBaseItemRow {
  return {
    schemaVersion: 1,
    baseItem: 1,
    label: "Longsword",
    itemClass: "wswls",
    modelType: 2,
    minRange: 10,
    maxRange: 100,
    genderSpecific: false,
    defaultModel: null,
    defaultIcon: "iw_swls",
    equipableSlots: 0x10,
    invSlotWidth: 1,
    invSlotHeight: 3,
    weaponWield: null,
    weaponType: null,
    rangedWeapon: null,
    ammunitionType: null,
    capability: {
      schemaVersion: 1,
      compositionProfile: "BOTTOM_MIDDLE_TOP",
      textureProfile: "DIRECT_COLOR",
      iconProfile: "STANDARD",
      meshySourceCount: 3,
      requiredReferenceTables: [],
    },
    partSlots: [
      { index: 0, field: "ModelPart1", label: "Bottom", token: "b", sourceKind: "MESHY_GLB", referenceTable: null, requiresExplicitResourceResrefs: false },
      { index: 1, field: "ModelPart2", label: "Middle", token: "m", sourceKind: "MESHY_GLB", referenceTable: null, requiresExplicitResourceResrefs: false },
      { index: 2, field: "ModelPart3", label: "Top", token: "t", sourceKind: "MESHY_GLB", referenceTable: null, requiresExplicitResourceResrefs: false },
    ],
    colorFields: [],
  };
}

function concept(field: string) {
  return {
    fileName: `${field}.png`,
    mimeType: "image/png" as const,
    byteLength: 1234,
    sha256: field === "ModelPart1" ? "1".repeat(64)
      : field === "ModelPart2" ? "2".repeat(64) : "3".repeat(64),
  };
}

describe("Item generation session", () => {
  it("derives one ordered generation slot per Meshy BaseItem slot without usage categories", () => {
    const session = createItemGenerationSession(
      longsword(),
      "a".repeat(64),
      "item-session-1",
      "2026-08-02T12:00:00.000Z",
    );

    expect(session.baseItem).toBe(1);
    expect(session.modelType).toBe(2);
    expect(session.slots.map(({ field, role, status }) => [field, role, status])).toEqual([
      ["ModelPart1", "BOTTOM", "EMPTY"],
      ["ModelPart2", "MIDDLE", "EMPTY"],
      ["ModelPart3", "TOP", "EMPTY"],
    ]);
  });

  it("creates one aggregate plan only after every concept is hash-bound and within the owner cap", () => {
    let session = createItemGenerationSession(
      longsword(), "a".repeat(64), "item-session-2", "2026-08-02T12:00:00.000Z",
    );
    for (const field of ["ModelPart1", "ModelPart2", "ModelPart3"]) {
      session = setItemGenerationConcept(session, field, concept(field));
    }

    session = createItemGenerationPlan(session, [
      { field: "ModelPart1", previewId: "preview-bottom", maximumCredits: 30, targetPolycount: 5000 },
      { field: "ModelPart2", previewId: "preview-middle", maximumCredits: 30, targetPolycount: 8000 },
      { field: "ModelPart3", previewId: "preview-top", maximumCredits: 30, targetPolycount: 3000 },
    ], 90, 318);

    expect(session.status).toBe("REVIEWED");
    expect(session.maximumCredits).toBe(90);
    expect(session.slots.map((slot) => slot.status)).toEqual(["REVIEWED", "REVIEWED", "REVIEWED"]);
    expect(() => createItemGenerationPlan(session, session.slots.map((slot) => ({
      field: slot.field,
      previewId: `again-${slot.field}`,
      maximumCredits: 30,
      targetPolycount: slot.targetPolycount,
    })), 89, 318)).toThrow(/owner cap/i);
  });

  it("preserves successful slots and cannot replace their exact run or task identity", () => {
    let session = createItemGenerationSession(
      longsword(), "a".repeat(64), "item-session-3", "2026-08-02T12:00:00.000Z",
    );
    for (const field of ["ModelPart1", "ModelPart2", "ModelPart3"]) {
      session = setItemGenerationConcept(session, field, concept(field));
    }
    session = createItemGenerationPlan(session, [
      { field: "ModelPart1", previewId: "p1", maximumCredits: 30, targetPolycount: 5000 },
      { field: "ModelPart2", previewId: "p2", maximumCredits: 30, targetPolycount: 8000 },
      { field: "ModelPart3", previewId: "p3", maximumCredits: 30, targetPolycount: 3000 },
    ], 90, 318);
    session = recordItemGenerationRun(session, "ModelPart1", {
      runId: "run-bottom",
      taskId: "task-bottom",
      createdAt: "2026-08-02T12:01:00.000Z",
    });
    session = recordItemGenerationArtifact(session, "ModelPart1", {
      sha256: "b".repeat(64),
      byteLength: 4567,
      consumedCredits: 30,
      finishedAt: "2026-08-02T12:03:00.000Z",
    });

    expect(session.slots[0].status).toBe("ARTIFACT_VERIFIED");
    expect(session.slots.slice(1).map(({ status }) => status)).toEqual(["REVIEWED", "REVIEWED"]);
    expect(() => recordItemGenerationRun(session, "ModelPart1", {
      runId: "duplicate-run",
      taskId: "duplicate-task",
      createdAt: "2026-08-02T12:04:00.000Z",
    })).toThrow(/already bound/i);
  });

  it("binds the first observed task ID to a created run exactly once", () => {
    let session = createItemGenerationSession(
      longsword(), "a".repeat(64), "item-session-task-bind", "2026-08-02T12:00:00.000Z",
    );
    for (const field of ["ModelPart1", "ModelPart2", "ModelPart3"]) {
      session = setItemGenerationConcept(session, field, concept(field));
    }
    session = createItemGenerationPlan(session, [
      { field: "ModelPart1", previewId: "p1", maximumCredits: 30, targetPolycount: 5000 },
      { field: "ModelPart2", previewId: "p2", maximumCredits: 30, targetPolycount: 8000 },
      { field: "ModelPart3", previewId: "p3", maximumCredits: 30, targetPolycount: 3000 },
    ], 90, 318);
    session = recordItemGenerationRun(session, "ModelPart1", {
      runId: "run-bottom",
      taskId: null,
      createdAt: "2026-08-02T12:01:00.000Z",
    });
    session = bindItemGenerationTask(session, "ModelPart1", "task-bottom");

    expect(session.slots[0].run).toMatchObject({ runId: "run-bottom", taskId: "task-bottom" });
    expect(() => bindItemGenerationTask(session, "ModelPart1", "different-task")).toThrow(/already bound/i);
  });

  it("recovers one exact prior task without paid review and reviews only the remaining slots", () => {
    let session = createItemGenerationSession(
      longsword(), "a".repeat(64), "item-session-recovery", "2026-08-02T12:00:00.000Z",
    );
    for (const field of ["ModelPart1", "ModelPart2", "ModelPart3"]) {
      session = setItemGenerationConcept(session, field, concept(field));
    }
    session = recordRecoveredItemGenerationArtifact(session, "ModelPart1", {
      taskId: "prior-bottom-task",
      createdAt: "2026-08-01T12:00:00.000Z",
      artifact: {
        sha256: "b".repeat(64),
        byteLength: 4567,
        consumedCredits: 30,
        finishedAt: "2026-08-01T12:02:00.000Z",
      },
    });
    session = createItemGenerationPlan(session, [
      { field: "ModelPart2", previewId: "p2", maximumCredits: 30, targetPolycount: 8000 },
      { field: "ModelPart3", previewId: "p3", maximumCredits: 30, targetPolycount: 3000 },
    ], 60, 318);

    expect(session.maximumCredits).toBe(60);
    expect(session.slots.map(({ status }) => status)).toEqual([
      "ARTIFACT_VERIFIED", "REVIEWED", "REVIEWED",
    ]);
  });

  it("recovers an already bound task after polling or download was interrupted without replacing its identity", () => {
    let session = createItemGenerationSession(
      longsword(), "a".repeat(64), "item-session-interrupted", "2026-08-02T12:00:00.000Z",
    );
    for (const field of ["ModelPart1", "ModelPart2", "ModelPart3"]) {
      session = setItemGenerationConcept(session, field, concept(field));
    }
    session = createItemGenerationPlan(session, [
      { field: "ModelPart1", previewId: "p1", maximumCredits: 30, targetPolycount: 5000 },
      { field: "ModelPart2", previewId: "p2", maximumCredits: 30, targetPolycount: 8000 },
      { field: "ModelPart3", previewId: "p3", maximumCredits: 30, targetPolycount: 3000 },
    ], 90, 318);
    session = recordItemGenerationRun(session, "ModelPart1", {
      runId: "run-bottom",
      taskId: "task-bottom",
      createdAt: "2026-08-02T12:01:00.000Z",
    });

    expect(() => recordRecoveredItemGenerationArtifact(session, "ModelPart1", {
      taskId: "different-task",
      createdAt: "2026-08-02T12:01:00.000Z",
      artifact: {
        sha256: "b".repeat(64), byteLength: 4567, consumedCredits: 30,
        finishedAt: "2026-08-02T12:03:00.000Z",
      },
    })).toThrow(/differs from its bound exact task/i);

    session = recordRecoveredItemGenerationArtifact(session, "ModelPart1", {
      taskId: "task-bottom",
      createdAt: "2026-08-02T12:01:00.000Z",
      artifact: {
        sha256: "b".repeat(64), byteLength: 4567, consumedCredits: 30,
        finishedAt: "2026-08-02T12:03:00.000Z",
      },
    });
    expect(session.slots[0]).toMatchObject({
      status: "ARTIFACT_VERIFIED",
      run: { runId: "run-bottom", taskId: "task-bottom" },
      artifact: { sha256: "b".repeat(64) },
    });
  });

  it("roundtrips a non-secret recovery sidecar and rejects signed URLs", () => {
    let session = createItemGenerationSession(
      longsword(), "a".repeat(64), "item-session-4", "2026-08-02T12:00:00.000Z",
    );
    session = setItemGenerationConcept(session, "ModelPart1", concept("ModelPart1"));
    const serialized = serializeItemGenerationSession(session);

    expect(serialized).not.toMatch(/api[_-]?key|authorization|https?:\/\//i);
    expect(deserializeItemGenerationSession(serialized)).toEqual(session);
    expect(() => deserializeItemGenerationSession(serialized.replace(
      '"fileName":"ModelPart1.png"',
      '"fileName":"ModelPart1.png","signedUrl":"https://assets.meshy.ai/private"',
    ))).toThrow(/unknown field|signed|credential/i);
  });
});
