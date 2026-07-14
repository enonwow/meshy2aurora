// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import type { WorkerArtifact } from "../../worker/types";
import { projectCanonicalResult } from "./projectCanonicalResult";

const id = (byteLength: number, marker: string) => ({ byteLength, sha256: marker.repeat(64) });
const bytes = (text: string) => new TextEncoder().encode(text).buffer;

function fixture() {
  const report = {
    schemaVersion: 1,
    geometry: { vertexCount: 24, triangleCount: 12, activeJointCount: 2, outputSegmentDeformation: "SKIN" },
    model: { payloadSha256: "b".repeat(64), layout: { fileLength: 2 } },
    texture: { width: 2, height: 2, pixelFormat: "RGBA8", byteLength: 60, outputSha256: "d".repeat(64) },
    appearance: { appendedRowIndex: 1, sourcePrefixPreserved: true, outputByteLength: 7, outputSha256: "e".repeat(64) },
    hak: { byteLength: 3, archiveSha256: "a".repeat(64), entryCount: 3 },
  };
  const reportJson = JSON.stringify(report);
  const summary = {
    schemaVersion: 1,
    status: "M6_MODEL_PACKAGE_MATERIALIZED",
    outputs: {
      model: id(2, "b"), texture: id(60, "d"), appearanceTwoDa: id(7, "e"),
      hak: id(3, "a"), report: id(bytes(reportJson).byteLength, "c"),
    },
    appendedPhysicalRow: 1,
    modelResref: "m2a_model",
    textureResref: "m2a_texture",
    animation: { sourceName: "walk", outputName: "cwalk", durationSeconds: 1.25, hasMotion: true },
    appearancePayloadPolicy: "PRESERVED_AND_APPENDED",
  };
  const manifest = {
    schemaVersion: 1,
    status: "M6_MODEL_PACKAGE_MATERIALIZED",
    appendedPhysicalRow: 1,
    appearancePayloadPolicy: "PRESERVED_AND_APPENDED",
    packageManifest: {
      packageSha256: "a".repeat(64),
      resources: [
        { role: "APPEARANCE_TABLE", resref: "appearance", type: 2017, ...id(7, "e") },
        { role: "MODEL", resref: "m2a_model", type: 2002, ...id(2, "b") },
        { role: "TEXTURE", resref: "m2a_texture", type: 3, ...id(60, "d") },
      ],
    },
  };
  const summaryJson = JSON.stringify(summary);
  const manifestJson = JSON.stringify(manifest);
  const artifact = (
    artifactId: string,
    kind: WorkerArtifact["kind"],
    content: ArrayBuffer,
    sha256: string,
  ): WorkerArtifact => ({
    artifactId,
    kind,
    fileName: `${artifactId}.bin`,
    mediaType: kind === "JSON_REPORT" ? "application/json" : "application/octet-stream",
    byteLength: content.byteLength,
    sha256,
    bytes: content,
    provenance: "M2A_WASM_WORKER",
  });
  const artifacts = [
    artifact("package-hak", "HAK", new Uint8Array([1, 2, 3]).buffer, "a".repeat(64)),
    artifact("model-mdl", "MODEL", new Uint8Array([1, 2]).buffer, "b".repeat(64)),
    artifact("report-json", "JSON_REPORT", bytes(reportJson), "c".repeat(64)),
    artifact("manifest-json", "JSON_REPORT", bytes(manifestJson), "f".repeat(64)),
    artifact("summary-json", "JSON_REPORT", bytes(summaryJson), "9".repeat(64)),
  ];
  return { report, summary, manifest, reportJson, summaryJson, manifestJson, artifacts };
}

describe("canonical result projector", () => {
  it("projects exact required KPI and byte identities", () => {
    const value = fixture();
    const result = projectCanonicalResult(value.reportJson, value.summaryJson, value.manifestJson, value.artifacts);
    expect(result).toMatchObject({
      status: "M6_MODEL_PACKAGE_MATERIALIZED",
      geometry: { vertices: 24, triangles: 12, joints: 2, deformation: "SKIN" },
      animation: { sourceName: "walk", outputName: "cwalk", durationSeconds: 1.25, hasMotion: true },
      texture: { width: 2, height: 2, pixelFormat: "RGBA8", byteLength: 60 },
      resrefs: { model: "m2a_model", texture: "m2a_texture" },
      appearance: { appendedRow: 1, sourcePrefixPreserved: true, policy: "PRESERVED_AND_APPENDED" },
      hak: { byteLength: 3, sha256: "a".repeat(64), entryCount: 3 },
    });
    expect(result.resources.map(({ role, resref }) => [role, resref])).toEqual([
      ["APPEARANCE_TABLE", "appearance"], ["MODEL", "m2a_model"], ["TEXTURE", "m2a_texture"],
    ]);
  });

  it.each(["report", "summary", "manifest", "artifact"] as const)("rejects malformed %s input without fallback values", (part) => {
    const value = fixture();
    if (part === "report") delete (value.report as { geometry?: unknown }).geometry;
    if (part === "summary") value.summary.animation.durationSeconds = "1.25" as unknown as number;
    if (part === "manifest") delete (value.manifest as { packageManifest?: unknown }).packageManifest;
    if (part === "artifact") value.artifacts = value.artifacts.filter(({ artifactId }) => artifactId !== "summary-json");
    const projection = () => projectCanonicalResult(
      part === "report" ? JSON.stringify(value.report) : value.reportJson,
      part === "summary" ? JSON.stringify(value.summary) : value.summaryJson,
      part === "manifest" ? JSON.stringify(value.manifest) : value.manifestJson,
      value.artifacts,
    );
    if (part === "summary") {
      expect(projection).toThrow("Canonical result field summary.animation.durationSeconds");
    } else {
      expect(projection).toThrow(/Canonical result/);
    }
  });

  it.each(["report", "summary", "manifest"] as const)("rejects invalid %s JSON syntax", (part) => {
    const value = fixture();
    expect(() => projectCanonicalResult(
      part === "report" ? "{" : value.reportJson,
      part === "summary" ? "{" : value.summaryJson,
      part === "manifest" ? "{" : value.manifestJson,
      value.artifacts,
    )).toThrow(`Canonical result field ${part}Json`);
  });

  it("reports the exact wrong-type field after earlier required fields pass", () => {
    const value = fixture();
    value.report.geometry.vertexCount = "24" as unknown as number;
    const reportJson = JSON.stringify(value.report);
    value.summary.outputs.report.byteLength = bytes(reportJson).byteLength;
    value.artifacts.find(({ artifactId }) => artifactId === "report-json")!.bytes = bytes(reportJson);
    value.artifacts.find(({ artifactId }) => artifactId === "report-json")!.byteLength = bytes(reportJson).byteLength;
    expect(() => projectCanonicalResult(
      reportJson,
      JSON.stringify(value.summary),
      value.manifestJson,
      value.artifacts,
    )).toThrow("Canonical result field report.geometry.vertexCount");
  });

  it.each([
    ["schema", (value: ReturnType<typeof fixture>) => { value.summary.schemaVersion = 2; }],
    ["status", (value: ReturnType<typeof fixture>) => { value.summary.status = "DONE"; }],
    ["model identity", (value: ReturnType<typeof fixture>) => { value.manifest.packageManifest.resources[1].sha256 = "0".repeat(64); }],
    ["texture identity", (value: ReturnType<typeof fixture>) => { value.report.texture.outputSha256 = "0".repeat(64); }],
    ["appearance identity", (value: ReturnType<typeof fixture>) => { value.report.appearance.outputByteLength = 8; }],
    ["HAK identity", (value: ReturnType<typeof fixture>) => { value.report.hak.archiveSha256 = "0".repeat(64); }],
    ["model resref", (value: ReturnType<typeof fixture>) => { value.manifest.packageManifest.resources[1].resref = "other"; }],
    ["texture resref", (value: ReturnType<typeof fixture>) => { value.manifest.packageManifest.resources[2].resref = "other"; }],
    ["appearance resref", (value: ReturnType<typeof fixture>) => { value.manifest.packageManifest.resources[0].resref = "other"; }],
    ["duplicate resource", (value: ReturnType<typeof fixture>) => { value.manifest.packageManifest.resources[2].role = "MODEL"; }],
    ["artifact provenance", (value: ReturnType<typeof fixture>) => { value.artifacts[0].provenance = "OTHER" as "M2A_WASM_WORKER"; }],
    ["duplicate artifact", (value: ReturnType<typeof fixture>) => { value.artifacts[4].artifactId = "report-json"; }],
  ] as const)("rejects %s mismatch", (_label, mutate) => {
    const value = fixture();
    mutate(value);
    const reportJson = JSON.stringify(value.report);
    const summaryJson = JSON.stringify(value.summary);
    const manifestJson = JSON.stringify(value.manifest);
    for (const [artifactId, json] of [["report-json", reportJson], ["summary-json", summaryJson], ["manifest-json", manifestJson]] as const) {
      const artifact = value.artifacts.find((candidate) => candidate.artifactId === artifactId);
      if (artifact) {
        artifact.bytes = bytes(json);
        artifact.byteLength = artifact.bytes.byteLength;
      }
    }
    expect(() => projectCanonicalResult(reportJson, summaryJson, manifestJson, value.artifacts)).toThrow(/Canonical result/);
  });
});
