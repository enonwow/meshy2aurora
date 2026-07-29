// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
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
});
