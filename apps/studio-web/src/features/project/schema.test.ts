// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import {
  animationStudioClipFixtureV1,
  animationStudioDocumentFixtureV1,
} from "../animation-studio/testFixtures";
import {
  createMeshy2AuroraProjectV1,
  duplicateMeshy2AuroraProjectV1,
  parseMeshy2AuroraProjectV1,
  projectBuildIdentityV1,
  projectExportFileNameV1,
  projectFileReferenceV1,
  reviseMeshy2AuroraProjectV1,
  sameProjectBuildIdentityV1,
  serializeMeshy2AuroraProjectV1,
  serializeProjectBuildIdentityV1,
} from "./schema";

const SHA = "a".repeat(64);

describe("Meshy2AuroraProjectV1", () => {
  it("round-trips a portable project without binary payloads or absolute paths", () => {
    const created = createMeshy2AuroraProjectV1({
      projectId: "project-aurora-01",
      name: "Aurora Creature",
      now: "2026-07-28T12:00:00.000Z",
    });
    const source = new File(["glb"], "creature.glb", {
      type: "model/gltf-binary",
      lastModified: 123,
    });
    const project = reviseMeshy2AuroraProjectV1(created, {
      files: {
        ...created.files,
        sourceGlb: projectFileReferenceV1(source, SHA),
      },
    }, "2026-07-28T12:01:00.000Z");

    const json = serializeMeshy2AuroraProjectV1(project);
    const parsed = parseMeshy2AuroraProjectV1(json);

    expect(parsed).toEqual({ kind: "VALID", value: project });
    expect(json).not.toContain("ArrayBuffer");
    expect(json).not.toContain("C:\\\\");
    expect(json).not.toContain("/home/");
    expect(projectExportFileNameV1(project)).toBe("aurora-creature.m2a-project.json");
    expect(projectBuildIdentityV1(project)).toEqual({
      schemaVersion: 1,
      projectId: "project-aurora-01",
      projectName: "Aurora Creature",
      projectRevision: 2,
    });
    expect(JSON.parse(serializeProjectBuildIdentityV1(project))).toEqual(
      projectBuildIdentityV1(project),
    );
  });

  it("compares project build identity semantically instead of relying on JSON property order", () => {
    const project = createMeshy2AuroraProjectV1({
      projectId: "project-identity-01",
      name: "Identity project",
      now: "2026-07-28T12:00:00.000Z",
    });
    const identity = projectBuildIdentityV1(project);
    expect(sameProjectBuildIdentityV1({
      projectRevision: identity.projectRevision,
      projectName: identity.projectName,
      projectId: identity.projectId,
      schemaVersion: 1,
    }, identity)).toBe(true);
    expect(sameProjectBuildIdentityV1(undefined, identity)).toBe(false);
    expect(sameProjectBuildIdentityV1(
      { ...identity, projectRevision: identity.projectRevision + 1 },
      identity,
    )).toBe(false);
  });

  it("keeps library preset provenance through the portable project backup", () => {
    const created = createMeshy2AuroraProjectV1({
      projectId: "project-library-01",
      name: "Library project",
      now: "2026-07-31T12:00:00.000Z",
    });
    const clip = animationStudioClipFixtureV1({
      source: {
        kind: "LIBRARY_PRESET_COPY",
        sourceRevision: SHA,
        sourceClipName: null,
        sourceClipFingerprint: "b".repeat(64),
        proceduralTemplate: null,
        libraryPreset: {
          presetId: "m2a_right_cross",
          presetVersion: 1,
          presetMotionSha256: "b".repeat(64),
          catalogSha256: "c".repeat(64),
          source: "BUILT_IN",
          authors: ["Meshy2Aurora contributors"],
          license: "LicenseRef-Meshy2Aurora-Project-Generated",
          rigSignatureSha256: "d".repeat(64),
          instantiationMode: "STRICT_RIG_V1",
        },
      },
    });
    const animationStudio = {
      ...animationStudioDocumentFixtureV1({
        sourceRevision: SHA,
        authoredClips: [clip],
      }),
      schemaVersion: 2 as const,
    };
    const project = reviseMeshy2AuroraProjectV1(created, {
      files: {
        ...created.files,
        sourceGlb: {
          name: "creature.glb",
          byteLength: 3,
          lastModified: 123,
          sha256: SHA,
        },
      },
      animationStudio,
    }, "2026-07-31T12:01:00.000Z");

    const json = serializeMeshy2AuroraProjectV1(project);
    expect(parseMeshy2AuroraProjectV1(json)).toEqual({ kind: "VALID", value: project });
    expect(json).toContain('"kind": "LIBRARY_PRESET_COPY"');
    expect(json).toContain('"presetMotionSha256"');
  });

  it("rejects unknown and newer fields without rewriting them", () => {
    const project = createMeshy2AuroraProjectV1({
      projectId: "project-schema-01",
      now: "2026-07-28T12:00:00.000Z",
    });
    const newer = { ...project, schemaVersion: 2, futureData: { keep: true } };
    const parsed = parseMeshy2AuroraProjectV1(newer);

    expect(parsed.kind).toBe("INVALID");
    if (parsed.kind === "INVALID") {
      expect(parsed.diagnostics[0]?.code).toBe("M2A-PROJECT-SCHEMA");
      expect(parsed.diagnostics[0]?.message).toContain("missing or unknown");
    }
    expect(newer.futureData).toEqual({ keep: true });
  });

  it("rejects absolute paths and source-bound authoring mismatches", () => {
    const project = createMeshy2AuroraProjectV1({
      projectId: "project-source-01",
      now: "2026-07-28T12:00:00.000Z",
    });
    const absolute = {
      ...project,
      files: {
        ...project.files,
        sourceGlb: {
          name: "C:\\private\\creature.glb",
          byteLength: 3,
          lastModified: 1,
          sha256: SHA,
        },
      },
    };

    const parsed = parseMeshy2AuroraProjectV1(absolute);
    expect(parsed.kind).toBe("INVALID");
    if (parsed.kind === "INVALID") {
      expect(parsed.diagnostics[0]?.message).toContain("basename");
    }
  });

  it("duplicates content under a fresh identity and revision", () => {
    const project = createMeshy2AuroraProjectV1({
      projectId: "project-original-01",
      name: "Original",
      now: "2026-07-28T12:00:00.000Z",
    });
    const duplicate = duplicateMeshy2AuroraProjectV1(project, {
      projectId: "project-copy-01",
      now: "2026-07-28T12:05:00.000Z",
    });

    expect(duplicate.identity.projectId).toBe("project-copy-01");
    expect(duplicate.identity.name).toBe("Original copy");
    expect(duplicate.revision).toBe(1);
    expect(duplicate.target).toBe(project.target);
  });

  it("migrates a legacy project to an empty animation workbench", () => {
    const project = createMeshy2AuroraProjectV1({
      projectId: "project-legacy-workbench",
      now: "2026-08-01T10:00:00.000Z",
    });
    const legacy = structuredClone(project) as unknown as Record<string, unknown>;
    delete legacy.animationWorkbench;

    const parsed = parseMeshy2AuroraProjectV1(legacy);
    expect(parsed.kind).toBe("VALID");
    if (parsed.kind === "VALID") {
      expect(parsed.value.animationWorkbench).toEqual({
        schemaVersion: 1,
        sourceRevision: null,
        heldWeapon: null,
        posePresets: [],
        phaseTimelines: [],
        sequences: [],
        semanticMaps: [],
        variantRecipes: [],
      });
    }
  });

  it("persists held-weapon identity and rejects a stale workbench source", () => {
    const created = createMeshy2AuroraProjectV1({
      projectId: "project-held-weapon",
      now: "2026-08-01T10:00:00.000Z",
    });
    const project = reviseMeshy2AuroraProjectV1(created, {
      files: {
        ...created.files,
        sourceGlb: {
          name: "creature.glb",
          byteLength: 10,
          lastModified: 1,
          sha256: SHA,
        },
      },
      animationWorkbench: {
        ...created.animationWorkbench,
        sourceRevision: SHA,
        heldWeapon: {
          source: {
            name: "sword.glb",
            byteLength: 120,
            lastModified: 2,
            sha256: "b".repeat(64),
          },
          attachment: {
            targetBoneName: "RightHand",
            translation: [0.1, 0.2, 0.3],
            rotationEulerDegrees: [0, 90, 0],
            scale: [1, 1, 1],
          },
        },
      },
    });
    expect(parseMeshy2AuroraProjectV1(serializeMeshy2AuroraProjectV1(project)))
      .toEqual({ kind: "VALID", value: project });

    const stale = {
      ...project,
      animationWorkbench: {
        ...project.animationWorkbench,
        sourceRevision: "c".repeat(64),
      },
    };
    const parsed = parseMeshy2AuroraProjectV1(stale);
    expect(parsed.kind).toBe("INVALID");
    if (parsed.kind === "INVALID") {
      expect(parsed.diagnostics[0]?.code).toBe("M2A-PROJECT-WORKBENCH-SOURCE-MISMATCH");
    }
  });
});
