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
    ingest: {
      schemaVersion: 1,
      inventory: { nodeCount: 8, meshCount: 2, jointReferenceCount: 4, animationCount: 1 },
      statistics: { vertexCount: 24, triangleCount: 12 },
    },
    conversion: {
      schemaVersion: 1,
      conversionEligible: true,
      policies: { engineFacingProof: "OPEN_M6", uvRuntimeProof: "OPEN_M6" },
      gates: [] as Array<{ schemaVersion: number; code: string; severity: string; path: string; expected: string; actual: string; message: string }>,
      diagnostics: [] as Array<{ schemaVersion: number; code: string; severity: string; path: string; message: string }>,
    },
    model: {
      payloadSha256: "b".repeat(64),
      layout: { fileLength: 2 },
      projection: {
        modelResourceResref: "m2a_model", animationCount: 1, rigNodeCount: 2, meshNodeCount: 2, triangleCount: 12,
      },
      semanticDiff: [],
      deviations: [],
    },
    texture: { width: 2, height: 2, pixelFormat: "RGBA8", byteLength: 60, outputSha256: "d".repeat(64) },
    appearance: { appendedRowIndex: 1, sourcePrefixPreserved: true, outputByteLength: 7, outputSha256: "e".repeat(64) },
    hak: { byteLength: 3, archiveSha256: "a".repeat(64), entryCount: 3 },
    proofModule: { byteLength: 4, sha256: "7".repeat(64), appearanceRow: 1, semanticReadbackStatus: "PASS" },
  };
  const reportJson = JSON.stringify(report);
  const summary = {
    schemaVersion: 1,
    status: "M6_MODEL_PACKAGE_MATERIALIZED",
    outputs: {
      model: id(2, "b"), texture: id(60, "d"), appearanceTwoDa: id(7, "e"),
      hak: id(3, "a"), proofModule: id(4, "7"), report: id(bytes(reportJson).byteLength, "c"),
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
    artifact("proof-module", "MODULE", new Uint8Array([4, 5, 6, 7]).buffer, "7".repeat(64)),
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
      sourceMetrics: { nodes: 8, meshes: 2, vertices: 24, triangles: 12, animations: 1 },
      convertedMetrics: { nodes: 4, meshes: 2, vertices: 24, triangles: 12, animations: 1 },
      geometry: { vertices: 24, triangles: 12, joints: 2, deformation: "SKIN" },
      animation: { sourceName: "walk", outputName: "cwalk", durationSeconds: 1.25, hasMotion: true },
      texture: { width: 2, height: 2, pixelFormat: "RGBA8", byteLength: 60 },
      resrefs: { model: "m2a_model", texture: "m2a_texture" },
      appearance: { appendedRow: 1, sourcePrefixPreserved: true, policy: "PRESERVED_AND_APPENDED" },
      hak: { byteLength: 3, sha256: "a".repeat(64), entryCount: 3 },
      semanticEvidence: { semanticDiff: [], deviations: [] },
      conversionEvidence: {
        schemaVersion: 1,
        conversionEligible: true,
        policies: { engineFacingProof: "OPEN_M6", uvRuntimeProof: "OPEN_M6" },
        gates: [],
        diagnostics: [],
      },
      packageAssemblyEvidence: { strictReconciled: true, resourceCount: 3, artifactCount: 6 },
    });
    expect(result.resources.map(({ role, resref }) => [role, resref])).toEqual([
      ["APPEARANCE_TABLE", "appearance"], ["MODEL", "m2a_model"], ["TEXTURE", "m2a_texture"],
    ]);
  });

  it("projects a schema V2 production creature without inventing a proof module", () => {
    const value = fixture();
    value.report.schemaVersion = 3;
    delete (value.report as Partial<typeof value.report>).proofModule;
    Object.assign(value.report, {
      skinAccessoryStabilization: {
        schemaVersion: 2,
        mode: "AUTO",
        auditedClipCount: 42,
        weldTolerance: 0.00001,
        componentCount: 5,
        detachedComponentCount: 4,
        riskyComponentCount: 2,
        stabilizedComponentCount: 2,
        changedVertexCount: 128,
        components: [{
          segmentIndex: 0,
          componentIndex: 1,
          triangleCount: 64,
          vertexCount: 48,
          isPrimaryBody: false,
          centroid: [1, 2, 3],
          activeBoneCount: 4,
          dominantBoneName: "LeftArm",
          dominantBoneShare: 0.503,
          riskReasons: ["PAIR_DISTANCE_RATIO_ABOVE_LIMIT"],
          action: "STABILIZED",
          selectedBoneId: 7,
          selectedBoneName: "Spine02",
          changedVertexCount: 48,
          before: {
            sampledClipCount: 42,
            sampledPoseCount: 128,
            sampledVertexCount: 48,
            vertexSamplingMode: "EXACT_ALL_VERTICES",
            timeSamplingTruncatedClipCount: 0,
            maxPairDistanceRatio: 2.39,
            maxPairDistanceError: 0.1,
            minAxisAlignment: 0.26,
          },
          after: {
            sampledClipCount: 42,
            sampledPoseCount: 128,
            sampledVertexCount: 48,
            vertexSamplingMode: "EXACT_ALL_VERTICES",
            timeSamplingTruncatedClipCount: 0,
            maxPairDistanceRatio: 1,
            maxPairDistanceError: 0,
            minAxisAlignment: 0.26,
          },
        }],
        warnings: ["Component 1 was stabilized to Spine02."],
      },
      materialFidelity: {
        schemaVersion: 1,
        materialSlot: 0,
        sourceMaterialId: 0,
        baseColorFactor: [0.8, 1, 0.5, 1],
        baseColorFactorBaked: true,
        alphaMode: "OPAQUE",
        alphaCutoff: null,
        alphaChannelPreserved: true,
        metallicFactor: 0,
        roughnessFactor: 0.7,
        normalTexturePresent: true,
        emissiveFactor: [0.1, 0, 0],
        emissiveTexturePresent: false,
        doubleSided: true,
        auroraMaterialProfile: "CLASSIC_DIFFUSE_TGA_SAFE_V1",
        mappedFields: ["baseColorTexture->diffuseTga", "baseColorFactor->diffuseTgaPixels"],
        unsupportedFields: ["roughnessFactor", "normalTexture", "emissiveFactor", "doubleSided"],
      },
    });
    value.summary.schemaVersion = 3;
    value.summary.status = "PROCEDURAL_CREATURE_PRODUCT_MATERIALIZED";
    delete (value.summary.outputs as Partial<typeof value.summary.outputs>).proofModule;
    delete (value.summary as Partial<typeof value.summary>).modelResref;
    delete (value.summary as Partial<typeof value.summary>).textureResref;
    Object.assign(value.summary, {
      identity: {
        modelResref: "m2a_model",
        textureResref: "m2a_texture",
        hakResref: "m2a_hak",
        appearanceLabel: "M2A_PRODUCT",
      },
    });
    value.manifest.schemaVersion = 3;
    value.manifest.status = "PROCEDURAL_CREATURE_PRODUCT_MATERIALIZED";
    value.reportJson = JSON.stringify(value.report);
    value.summary.outputs.report = id(bytes(value.reportJson).byteLength, "c");
    value.summaryJson = JSON.stringify(value.summary);
    value.manifestJson = JSON.stringify(value.manifest);
    value.artifacts.push({
      artifactId: "texture-tga",
      kind: "TEXTURE",
      fileName: "m2a_texture.tga",
      mediaType: "image/x-tga",
      byteLength: 60,
      sha256: "d".repeat(64),
      bytes: new Uint8Array(60).buffer,
      provenance: "M2A_WASM_WORKER",
    });
    value.artifacts = value.artifacts
      .filter(({ artifactId }) => artifactId !== "proof-module")
      .map((artifact) => {
        if (artifact.artifactId === "report-json") {
          return { ...artifact, bytes: bytes(value.reportJson), byteLength: bytes(value.reportJson).byteLength };
        }
        if (artifact.artifactId === "summary-json") {
          return { ...artifact, bytes: bytes(value.summaryJson), byteLength: bytes(value.summaryJson).byteLength };
        }
        if (artifact.artifactId === "manifest-json") {
          return { ...artifact, bytes: bytes(value.manifestJson), byteLength: bytes(value.manifestJson).byteLength };
        }
        return artifact;
      });

    const result = projectCanonicalResult(
      value.reportJson,
      value.summaryJson,
      value.manifestJson,
      value.artifacts,
    );
    expect(result.status).toBe("PROCEDURAL_CREATURE_PRODUCT_MATERIALIZED");
    expect(result.resrefs).toEqual({ model: "m2a_model", texture: "m2a_texture" });
    expect(result.outputs).not.toHaveProperty("proofModule");
    expect(result.artifacts.map(({ artifactId }) => artifactId)).not.toContain("proof-module");
    expect(result.packageAssemblyEvidence.artifactCount).toBe(6);
    expect(result.skinAccessoryStabilization).toMatchObject({
      mode: "AUTO",
      auditedClipCount: 42,
      componentCount: 5,
      detachedComponentCount: 4,
      riskyComponentCount: 2,
      stabilizedComponentCount: 2,
      changedVertexCount: 128,
      components: [{
        segmentIndex: 0,
        componentIndex: 1,
        action: "STABILIZED",
        selectedBoneName: "Spine02",
        changedVertexCount: 48,
        before: { maxPairDistanceRatio: 2.39 },
        after: { maxPairDistanceRatio: 1 },
      }],
    });
    expect(result.materialFidelity).toMatchObject({
      baseColorFactor: [0.8, 1, 0.5, 1],
      baseColorFactorBaked: true,
      auroraMaterialProfile: "CLASSIC_DIFFUSE_TGA_SAFE_V1",
      mappedFields: ["baseColorTexture->diffuseTga", "baseColorFactor->diffuseTgaPixels"],
      unsupportedFields: ["roughnessFactor", "normalTexture", "emissiveFactor", "doubleSided"],
    });

    const demoReport = {
      schemaVersion: 2,
      moduleResref: "m2c2demo",
      moduleDisplayName: "Meshy2Aurora procedural humanoid proof",
      areaResref: "m2c2area",
      areaDisplayName: "Meshy2Aurora procedural humanoid proof area",
      creatureResref: "m2c2utc",
      hakResref: "m2a_hak",
      appearanceRow: 1,
      resourceCount: 6,
      byteLength: 4,
      sha256: "7".repeat(64),
      semanticReadbackStatus: "PASS",
    };
    const demoReportJson = JSON.stringify(demoReport);
    const withDemo = [
      ...value.artifacts,
      {
        artifactId: "proof-module",
        kind: "MODULE" as const,
        fileName: "m2c2demo.mod",
        mediaType: "application/octet-stream",
        byteLength: 4,
        sha256: "7".repeat(64),
        bytes: new Uint8Array([4, 5, 6, 7]).buffer,
        provenance: "M2A_WASM_WORKER" as const,
      },
      {
        artifactId: "demo-report-json",
        kind: "JSON_REPORT" as const,
        fileName: "demo-report.json",
        mediaType: "application/json",
        byteLength: bytes(demoReportJson).byteLength,
        sha256: "6".repeat(64),
        bytes: bytes(demoReportJson),
        provenance: "M2A_WASM_WORKER" as const,
      },
    ];
    const resultWithDemo = projectCanonicalResult(
      value.reportJson,
      value.summaryJson,
      value.manifestJson,
      withDemo,
      demoReportJson,
    );
    expect(resultWithDemo.demo).toEqual(demoReport);
    expect(resultWithDemo.packageAssemblyEvidence.artifactCount).toBe(8);
  });

  it("accepts the separately identified static M0 runtime package", () => {
    const value = fixture();
    const contract = {
      schemaVersion: 2,
      lane: "M0_BINARY_VERTICAL_SLICE",
      module: { resref: "m2a_bm0p1", ...id(4, "7") },
      hak: { resref: "m2a_m0_proof", ...id(3, "a") },
      model: { resref: "m2a_model", ...id(2, "b") },
      texture: { resref: "m2a_texture", ...id(60, "d") },
      appearanceTwoDa: { resref: "appearance", ...id(7, "e") },
      appearance: { physicalRow: 1, label: "M2A_M0_MESHY_RIGID", modelType: "S", race: "m2a_model" },
      binaryScene: {
        moduleResref: "m2a_bm0p1",
        areaResref: "m2a_bm0a1",
        orderedHakResrefs: ["m2a_m0_proof"],
        entryPosition: { x: 10, y: 10, z: 0 },
        entryDirection: { x: 0, y: 1 },
        fixture: {
          templateResref: "nw_dwarfmerc001",
          appearanceRow: 1,
          position: { x: 10, y: 14.5, z: 0 },
          orientation: { x: 1, y: 0 },
        },
      },
      meshEligibility: {
        eligible: true,
        checkedMeshCount: 1,
        eligibleMeshCount: 1,
        meshTypes: [3],
        textureResrefs: ["m2a_texture"],
        rule: "render == 1 && meshType == 3 && vertexCount > 0 && faces.nonEmpty && faceIndicesMatchSingleRawIndexStream && positions.len == vertexCount",
      },
    };
    (value.report as Record<string, unknown>).m0RuntimeFixtureContract = contract;
    (value.summary as Record<string, unknown>).m0RuntimeFixtureContract = contract;
    (value.manifest as Record<string, unknown>).m0RuntimeFixtureContract = contract;
    value.summary.status = "M0_MESHY_STATIC_RIGID_PACKAGE_MATERIALIZED";
    value.manifest.status = "M0_MESHY_STATIC_RIGID_PACKAGE_MATERIALIZED";
    const reportJson = JSON.stringify(value.report);
    value.summary.outputs.report.byteLength = bytes(reportJson).byteLength;
    const summaryJson = JSON.stringify(value.summary);
    const manifestJson = JSON.stringify(value.manifest);
    for (const [artifactId, json] of [["report-json", reportJson], ["summary-json", summaryJson], ["manifest-json", manifestJson]] as const) {
      const artifact = value.artifacts.find((candidate) => candidate.artifactId === artifactId)!;
      artifact.bytes = bytes(json);
      artifact.byteLength = artifact.bytes.byteLength;
    }
    const result = projectCanonicalResult(
      reportJson,
      summaryJson,
      manifestJson,
      value.artifacts,
    );
    expect(result.status).toBe("M0_MESHY_STATIC_RIGID_PACKAGE_MATERIALIZED");

    (value.summary as Record<string, unknown>).m0RuntimeFixtureContract = {
      ...contract,
      appearance: { ...contract.appearance, race: "stale_resref" },
    };
    const staleSummaryJson = JSON.stringify(value.summary);
    const summaryArtifact = value.artifacts.find((candidate) => candidate.artifactId === "summary-json")!;
    summaryArtifact.bytes = bytes(staleSummaryJson);
    summaryArtifact.byteLength = summaryArtifact.bytes.byteLength;
    expect(() => projectCanonicalResult(
      reportJson,
      staleSummaryJson,
      manifestJson,
      value.artifacts,
    )).toThrow("Canonical result identity mismatch at m0RuntimeFixtureContract");
  });

  it("projects only complete binary-readback creature event evidence with canonical identity", () => {
    const value = fixture();
    Object.assign(value.report, {
      animationEventConformance: {
        schemaVersion: 1,
        profile: "COMMON_NATIVE_GAMEPLAY_HOOKS_EXPLICIT_V1",
        requiredPairCount: 23,
        satisfiedPairCount: 23,
        totalEventCount: 24,
        unknownEventNames: ["owned_marker"],
        missingPairs: [],
        complete: true,
      },
      animationEventAuthoringCanonical: id(1_024, "8"),
    });
    const reportJson = JSON.stringify(value.report);
    value.summary.outputs.report.byteLength = bytes(reportJson).byteLength;
    const summaryJson = JSON.stringify(value.summary);
    const reportArtifact = value.artifacts.find(({ artifactId }) => artifactId === "report-json")!;
    reportArtifact.bytes = bytes(reportJson);
    reportArtifact.byteLength = reportArtifact.bytes.byteLength;
    const summaryArtifact = value.artifacts.find(({ artifactId }) => artifactId === "summary-json")!;
    summaryArtifact.bytes = bytes(summaryJson);
    summaryArtifact.byteLength = summaryArtifact.bytes.byteLength;

    const result = projectCanonicalResult(
      reportJson,
      summaryJson,
      value.manifestJson,
      value.artifacts,
    );
    expect(result.animationEventEvidence).toEqual({
      schemaVersion: 1,
      profile: "COMMON_NATIVE_GAMEPLAY_HOOKS_EXPLICIT_V1",
      requiredPairCount: 23,
      satisfiedPairCount: 23,
      totalEventCount: 24,
      unknownEventNames: ["owned_marker"],
      missingPairs: [],
      complete: true,
      authoringCanonical: id(1_024, "8"),
    });

    (
      value.report as typeof value.report & {
        animationEventConformance: { complete: boolean };
      }
    ).animationEventConformance.complete = false;
    expect(() => projectCanonicalResult(
      JSON.stringify(value.report),
      summaryJson,
      value.manifestJson,
      value.artifacts,
    )).toThrow("Canonical result identity mismatch at report.animationEventConformance");
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
    ["schemaVersion", (value: ReturnType<typeof fixture>) => { value.report.conversion.schemaVersion = 2; }, "report.conversion.schemaVersion"],
    ["conversionEligible", (value: ReturnType<typeof fixture>) => { value.report.conversion.conversionEligible = "yes" as unknown as boolean; }, "report.conversion.conversionEligible"],
    ["engine policy", (value: ReturnType<typeof fixture>) => { value.report.conversion.policies.engineFacingProof = ""; }, "report.conversion.policies.engineFacingProof"],
    ["gate", (value: ReturnType<typeof fixture>) => { value.report.conversion.gates = [{ schemaVersion: 1, code: "G", severity: "WARNING", path: "geometry", expected: "", actual: "bad", message: "warning" }]; }, "report.conversion.gates[0].expected"],
    ["diagnostic", (value: ReturnType<typeof fixture>) => { value.report.conversion.diagnostics = [{ schemaVersion: 1, code: "D", severity: "", path: "materials", message: "diagnostic" }]; }, "report.conversion.diagnostics[0].severity"],
  ] as const)("strictly rejects malformed conversion %s", (_label, mutate, path) => {
    const value = fixture();
    mutate(value);
    expect(() => projectCanonicalResult(
      JSON.stringify(value.report),
      value.summaryJson,
      value.manifestJson,
      value.artifacts,
    )).toThrow(`Canonical result field ${path}`);
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
    ["duplicate artifact", (value: ReturnType<typeof fixture>) => {
      const manifestArtifact = value.artifacts.find(({ artifactId }) => artifactId === "manifest-json");
      if (manifestArtifact) manifestArtifact.artifactId = "report-json";
    }],
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
