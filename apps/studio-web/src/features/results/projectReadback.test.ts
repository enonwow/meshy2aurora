import { describe, expect, it } from "vitest";
import { projectCanonicalReadback } from "./projectReadback";

const valid = () => ({
  schemaVersion: 1,
  format: "BINARY_MDL",
  nodeTree: {
    roots: [{
      offset: 12,
      number: 1,
      name: "root",
      controllers: [{ controllerName: "position", values: [[0, 1, 2]] }],
      mesh: {
        vertices: [{ x: 0, y: 1, z: 2 }],
        normals: [{ x: 0, y: 0, z: 1 }],
        uv0: [{ x: 0, y: 1 }],
        rawIndices: [[0, 0, 0]],
        faces: [{ vertexIndices: [0, 0, 0] }],
      },
      children: [],
    }],
  },
  diagnostics: [{ schemaVersion: 1, code: "NOTE", severity: "INFO", offset: 12, context: "owned" }],
});

describe("canonical readback projector", () => {
  it("projects every nested field consumed by preview and diagnostics", () => {
    expect(projectCanonicalReadback(JSON.stringify(valid()))).toEqual(valid());
  });

  it("normalizes serialized Rust Option nulls for non-mesh nodes and unnamed controllers", () => {
    const value = JSON.parse(JSON.stringify(valid())) as {
      nodeTree: { roots: Array<Record<string, unknown>> };
    };
    const root = value.nodeTree.roots[0];
    root.mesh = null;
    (root.controllers as Array<Record<string, unknown>>)[0].controllerName = null;
    expect(projectCanonicalReadback(JSON.stringify(value)).nodeTree.roots[0]).toEqual({
      offset: 12,
      number: 1,
      name: "root",
      controllers: [{ values: [[0, 1, 2]] }],
      children: [],
    });
  });

  it.each([
    ["syntax", "{"],
    ["array root", "[]"],
    ["empty object", "{}"],
    ["wrong roots", JSON.stringify({ ...valid(), nodeTree: { roots: {} } })],
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
