import { describe, expect, it } from "vitest";
import { projectCanonicalReadback } from "./projectReadback";

const valid = () => ({
  schemaVersion: 1,
  format: "nwn1-binary-mdl",
  nodeTree: {
    roots: [{
      offset: 12,
      number: 1,
      name: "root",
      controllers: [{ controllerName: "position", times: [0], values: [[0, 1, 2]] }],
      mesh: {
        vertices: [{ x: 0, y: 1, z: 2 }],
        normals: [{ x: 0, y: 0, z: 1 }],
        uv0: [{ x: 0, y: 1 }],
        rawIndices: [[0, 0, 0]],
        faces: [{ vertexIndices: [0, 0, 0] }],
      },
      skin: {
        nodeToBoneMap: [0],
        inlineMapping: [0],
        inverseBoneRotationsRaw: [[1, 0, 0, 0]],
        inverseBoneTranslations: [{ x: 0, y: 0, z: 0 }],
        vertexWeights: [[1, 0, 0, 0]],
        boneReferences: [[0, 0, 0, 0]],
      },
      children: [],
    }],
  },
  animations: [],
  diagnostics: [{ schemaVersion: 1, code: "NOTE", severity: "INFO", offset: 12, context: "owned" }],
});

describe("canonical readback projector", () => {
  it("projects every nested field consumed by preview and diagnostics", () => {
    const projected = projectCanonicalReadback(JSON.stringify(valid()));
    expect(projected).toMatchObject(valid());
    expect(projected.validation).toEqual({
      status: "PASS",
      structure: {
        schemaVersion: 1,
        format: "nwn1-binary-mdl",
        rootNodeCount: 1,
        hasRootNodes: true,
        structuralErrors: [],
      },
      diagnostics: { total: 1, warnings: 0, errors: 0, informational: 1, unrecognizedSeverity: 0 },
    });
  });

  it.each([
    ["PASS", "INFO"],
    ["WARNING", "warning"],
    ["ERROR", "FATAL"],
  ] as const)("derives %s only from projected diagnostic severity", (expected, severity) => {
    const value = valid();
    value.diagnostics[0].severity = severity;
    expect(projectCanonicalReadback(JSON.stringify(value)).validation?.status).toBe(expected);
  });

  it("fails closed for an unrecognized diagnostic severity", () => {
    const value = valid();
    value.diagnostics[0].severity = "CUSTOM";
    expect(projectCanonicalReadback(JSON.stringify(value)).validation).toMatchObject({
      status: "ERROR",
      diagnostics: { unrecognizedSeverity: 1 },
    });
  });

  it("reports an empty node tree as a structural ERROR", () => {
    const value = valid();
    value.nodeTree.roots = [];
    value.diagnostics = [];
    expect(projectCanonicalReadback(JSON.stringify(value)).validation).toMatchObject({
      status: "ERROR",
      structure: {
        rootNodeCount: 0,
        hasRootNodes: false,
        structuralErrors: ["READBACK_NODE_TREE_EMPTY"],
      },
    });
  });

  it("normalizes serialized Rust Option nulls for non-mesh nodes and unnamed controllers", () => {
    const value = JSON.parse(JSON.stringify(valid())) as {
      nodeTree: { roots: Array<Record<string, unknown>> };
    };
    const root = value.nodeTree.roots[0];
    root.mesh = null;
    root.skin = null;
    (root.controllers as Array<Record<string, unknown>>)[0].controllerName = null;
    expect(projectCanonicalReadback(JSON.stringify(value)).nodeTree.roots[0]).toEqual({
      offset: 12,
      number: 1,
      name: "root",
      controllers: [{ times: [0], values: [[0, 1, 2]] }],
      children: [],
    });
  });

  it.each([
    ["syntax", "{"],
    ["array root", "[]"],
    ["empty object", "{}"],
    ["wrong roots", JSON.stringify({ ...valid(), nodeTree: { roots: {} } })],
    ["wrong format", JSON.stringify({ ...valid(), format: "OTHER" })],
    ["wrong nested controller", (() => {
      const value = valid();
      value.nodeTree.roots[0].controllers[0].values = [[0, "bad" as unknown as number, 2]];
      return JSON.stringify(value);
    })()],
    ["wrong diagnostic", JSON.stringify({ ...valid(), diagnostics: [{ schemaVersion: 1, code: "X" }] })],
  ])("rejects %s without a partial typed readback", (_label, json) => {
    expect(() => projectCanonicalReadback(json)).toThrow(/Canonical readback/);
  });
});
