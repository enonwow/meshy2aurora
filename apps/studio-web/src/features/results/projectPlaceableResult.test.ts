import { describe, expect, it } from "vitest";
import type { WorkerArtifact } from "../../worker/types";
import { projectPlaceableResult } from "./projectPlaceableResult";

const hash = (character: string) => character.repeat(64);
const bytes = (values: number[]) => new Uint8Array(values).buffer;

function artifact(
  artifactId: string,
  kind: WorkerArtifact["kind"],
  fileName: string,
  payload: number[],
  sha256: string,
): WorkerArtifact {
  const buffer = bytes(payload);
  return {
    artifactId,
    kind,
    fileName,
    mediaType: kind === "JSON_REPORT" ? "application/json" : "application/octet-stream",
    byteLength: buffer.byteLength,
    sha256,
    bytes: buffer,
    provenance: "M2A_WASM_WORKER",
  };
}

function fixture() {
  const report = {
    schemaVersion: 1,
    status: "OFFLINE_ADMISSION_PASSED",
    profile: "STATIC_PLACEABLE",
    componentStatuses: {
      mdl: "passed",
      pwk: "passed",
      twoDa: "passed",
      utp: "passed",
      gitGic: "passed",
      palette: "passed",
      package: "passed",
      proof: "not_tested",
    },
    moduleFileName: "m2a_s1_plc_mod.mod",
    moduleDisplayName: "Meshy2Aurora S1 Placeable Proof",
    areaResref: "m2a_s1_plc_ar",
    areaName: "Meshy2Aurora S1 Ritual Pedestal",
    hakFileName: "m2a_s1_plc_hak.hak",
    modelResref: "m2a_s1_plc_ped",
    textureResref: "m2a_s1_plc_tex",
    blueprintResref: "m2a_s1_plc_utp",
    objectTag: "m2a_s1_ritual_pedestal",
    appearanceRow: { value: 16500 },
    placement: { x: 10, y: 14.5, z: 0, bearing: 0 },
    sourceModelSha256: hash("0"),
    mdlSha256: hash("b"),
    pwkSha256: hash("9"),
    textureSha256: hash("c"),
    placeables2daSha256: hash("d"),
    utpSha256: hash("e"),
    itpSha256: hash("f"),
    gitSha256: hash("1"),
    gicSha256: hash("2"),
    hakSha256: hash("a"),
    moduleSha256: hash("7"),
    hakResourceCount: 4,
    moduleResourceCount: 7,
    modelVisibility: "not_tested",
    proofCompleteness: "missing",
    paletteCompleteness: "custom_itp_emitted",
    collisionCompleteness: "ascii_pwk_emitted_offline_readback_passed",
    authoring: {
      sourceSha256: hash("0"),
      authoringSha256: hash("4"),
      sourceTriangleCount: 2,
      outputTriangleCount: 2,
      renderableElementCount: 1,
      collisionElementCount: 1,
      shadowElementCount: 1,
      boundsMin: [-1, 0, -1],
      boundsMax: [1, 1, 1],
    },
    collision: {
      schemaVersion: 1,
      mode: "CUSTOM_POLYGON",
      sourceCoordinateSpace: "GLTF_SOURCE_XZ_METERS",
      outputCoordinateSpace: "AURORA_XY_METERS",
      inputVertices: [[-1, -1], [1, -1], [1, 1], [-1, 1]],
      sourceVertices: [[-1, -1], [1, -1], [1, 1], [-1, 1]],
      vertices: [[-1, -1], [1, -1], [1, 1], [-1, 1]],
      triangles: [[0, 1, 2], [0, 2, 3]],
      boundsMin: [-1, -1],
      boundsMax: [1, 1],
      surfaceId: 7,
      authoringSha256: hash("4"),
      collisionSha256: hash("5"),
      pwkSha256: hash("9"),
    },
    resources: [
      { container: "HAK", role: "PLACEABLES_2DA", resref: "placeables", resourceType: 2017, byteLength: 11, sha256: hash("d") },
      { container: "HAK", role: "MODEL", resref: "m2a_s1_plc_ped", resourceType: 2002, byteLength: 2, sha256: hash("b") },
      { container: "HAK", role: "PLACEABLE_WALKMESH", resref: "m2a_s1_plc_ped", resourceType: 2053, byteLength: 4, sha256: hash("9") },
      { container: "HAK", role: "TEXTURE", resref: "m2a_s1_plc_tex", resourceType: 3, byteLength: 3, sha256: hash("c") },
      { container: "MOD", role: "MODULE_INFO", resref: "module", resourceType: 2014, byteLength: 5, sha256: hash("3") },
      { container: "MOD", role: "FACTIONS", resref: "repute", resourceType: 2038, byteLength: 5, sha256: hash("4") },
      { container: "MOD", role: "AREA", resref: "m2a_s1_plc_ar", resourceType: 2012, byteLength: 5, sha256: hash("5") },
      { container: "MOD", role: "AREA_COMMENTS", resref: "m2a_s1_plc_ar", resourceType: 2046, byteLength: 5, sha256: hash("6") },
      { container: "MOD", role: "AREA_INSTANCES", resref: "m2a_s1_plc_ar", resourceType: 2023, byteLength: 5, sha256: hash("8") },
      { container: "MOD", role: "PLACEABLE_BLUEPRINT", resref: "m2a_s1_plc_utp", resourceType: 2044, byteLength: 5, sha256: hash("e") },
      { container: "MOD", role: "PLACEABLE_PALETTE", resref: "placeablepalcus", resourceType: 2030, byteLength: 5, sha256: hash("f") },
    ],
  };
  const reportJson = JSON.stringify(report);
  const readbackJson = JSON.stringify({
    schemaVersion: 1,
    format: "nwn1-binary-mdl",
    nodeTree: { roots: [] },
    animations: [],
    diagnostics: [],
  });
  const artifacts = [
    artifact("placeable-package-hak", "HAK", report.hakFileName, [1, 2, 3], report.hakSha256),
    artifact("placeable-model-mdl", "MODEL", `${report.modelResref}.mdl`, [4, 5], report.mdlSha256),
    artifact("placeable-proof-module", "MODULE", report.moduleFileName, [6, 7, 8, 9], report.moduleSha256),
    artifact("placeable-report-json", "JSON_REPORT", "placeable-materialization-report.json", [...new TextEncoder().encode(reportJson)], hash("9")),
    artifact("placeable-model-readback-json", "JSON_REPORT", "placeable-model-readback.json", [...new TextEncoder().encode(readbackJson)], hash("8")),
  ];
  return { report, reportJson, readbackJson, artifacts };
}

describe("projectPlaceableResult", () => {
  it("projects the offline statuses, exact identities, placement and resource types", () => {
    const value = fixture();
    const result = projectPlaceableResult(value.reportJson, value.readbackJson, value.artifacts);
    expect(result.status).toBe("OFFLINE_ADMISSION_PASSED");
    expect(result.profile).toBe("STATIC_PLACEABLE");
    expect(result.appearanceRow).toBe(16500);
    expect(result.componentStatuses.proof).toBe("not_tested");
    expect(result.componentStatuses.pwk).toBe("passed");
    expect(result.modelVisibility).toBe("not_tested");
    expect(result.proofCompleteness).toBe("missing");
    expect(result.resources).toContainEqual(expect.objectContaining({
      resref: "m2a_s1_plc_utp",
      resourceType: 2044,
    }));
    expect(result.collision).toMatchObject({
      mode: "CUSTOM_POLYGON",
      surfaceId: 7,
      triangles: [[0, 1, 2], [0, 2, 3]],
    });
    expect(result.resources).toContainEqual(expect.objectContaining({
      role: "PLACEABLE_WALKMESH",
      resref: "m2a_s1_plc_ped",
      resourceType: 2053,
      sha256: hash("9"),
    }));
  });

  it("rejects a stale archive artifact instead of presenting a mixed lineage", () => {
    const value = fixture();
    value.artifacts[0] = { ...value.artifacts[0], sha256: hash("8") };
    expect(() => projectPlaceableResult(value.reportJson, value.readbackJson, value.artifacts))
      .toThrow("placeable-package-hak.sha256");
  });

  it("verifies each authored TGA and the combined V3 authoring recipe", () => {
    const value = fixture();
    const textureAuthoring = {
      schemaVersion: 1,
      sourceSha256: hash("0"),
      authoringSha256: hash("6"),
      alphaPolicy: "OPAQUE_ONLY",
      bindings: [{
        materialSlot: 0,
        sourceMaterialId: 0,
        sourceImageSha256: hash("1"),
        mode: "OVERRIDE",
        inputSha256: hash("2"),
        outputResref: "m2a_s1_plc_tex",
        outputSha256: hash("c"),
      }],
      resources: [{
        resref: "m2a_s1_plc_tex",
        resourceType: 3,
        byteLength: 3,
        sha256: hash("c"),
        materialSlots: [0],
      }],
    };
    const reportJson = JSON.stringify({ ...value.report, textureAuthoring });
    const artifacts = [
      ...value.artifacts,
      artifact(
        "placeable-texture-m2a_s1_plc_tex-tga",
        "TEXTURE",
        "m2a_s1_plc_tex.tga",
        [8, 8, 8],
        hash("c"),
      ),
      artifact(
        "placeable-authoring-v3-json",
        "JSON_REPORT",
        "placeable-authoring-v3.json",
        [1],
        hash("7"),
      ),
    ];
    const result = projectPlaceableResult(reportJson, value.readbackJson, artifacts);
    expect(result.textureAuthoring).toEqual(textureAuthoring);
  });

  it("projects neutral texture mip and double-sided quality diagnostics", () => {
    const value = fixture();
    const modelTextureAuthoring = {
      schemaVersion: 1,
      sourceSha256: hash("0"),
      separationSha256: hash("1"),
      authoringSha256: hash("2"),
      alphaPolicy: "OPAQUE_ONLY",
      uvPolicy: "MATERIAL_UV_PROJECTION_V1",
      warnings: [
        "MODEL-MATERIAL-DOUBLE-SIDED-TARGET-UNPROVEN:material:wood",
        "MODEL-TEXTURE-MIP-CONTRAST-LOW:material:wood",
      ],
      bindings: [{
        authoredMaterialId: "material:wood",
        materialSlot: 0,
        mode: "OVERRIDE",
        outputResref: "m2a_s1_plc_tex",
        sourceDoubleSided: true,
        targetDoubleSidedPolicy: "UNSUPPORTED_REPORT_ONLY",
        mipReadability: {
          sourceWidth: 2048,
          sourceHeight: 2048,
          baseLumaStddevMilli: 6550,
          mip16LumaStddevMilli: 1230,
          contrastRetentionBasisPoints: 1878,
          status: "LOW_CONTRAST",
        },
      }],
      resources: [],
    };

    const result = projectPlaceableResult(
      JSON.stringify({ ...value.report, modelTextureAuthoring }),
      value.readbackJson,
      value.artifacts,
    );

    expect(result.modelTextureAuthoring).toEqual({
      uvPolicy: "MATERIAL_UV_PROJECTION_V1",
      warnings: modelTextureAuthoring.warnings,
      bindings: modelTextureAuthoring.bindings,
    });
  });

  it("projects the complete V9 material pipeline only with semantic evidence", () => {
    const value = fixture();
    const report = {
      ...value.report,
      profile: "STATIC_PLACEABLE_V9_NWN_EE_MTR",
      hakResourceCount: value.report.hakResourceCount + 1,
      resources: [
        ...value.report.resources,
        { container: "HAK", role: "MATERIAL", resref: "m2a_s1_plc_m0", resourceType: 2072, byteLength: 3, sha256: hash("6") },
      ],
      materialCompilation: { schemaVersion: 1, status: "READY", targetProfile: "NWN_EE_MTR" },
      sourceQuality: { schemaVersion: 1, status: "PASS" },
      mdlMaterialExtension: { schemaVersion: 1, payloadSha256: value.report.mdlSha256, semanticDiff: [] },
      materialSemanticReadbackStatus: "PASS",
    };
    const artifacts = [
      ...value.artifacts,
      artifact("placeable-texture-m2a_s1_plc_tex-tga", "TEXTURE", "m2a_s1_plc_tex.tga", [8, 8, 8], hash("c")),
      artifact("placeable-material-resource-m2a_s1_plc_m0-2072", "MATERIAL", "m2a_s1_plc_m0.mtr", [9, 9, 9], hash("6")),
    ];
    const result = projectPlaceableResult(JSON.stringify(report), value.readbackJson, artifacts);
    expect(result).toMatchObject({
      profile: "STATIC_PLACEABLE_V9_NWN_EE_MTR",
      materialSemanticReadbackStatus: "PASS",
      sourceQuality: { status: "PASS" },
      mdlMaterialExtension: { semanticDiff: [] },
    });

    delete (report as { mdlMaterialExtension?: unknown }).mdlMaterialExtension;
    expect(() => projectPlaceableResult(JSON.stringify(report), value.readbackJson, artifacts))
      .toThrow("report.materialPipeline");
  });

  it("rejects a visual-proof claim at the offline Studio boundary", () => {
    const value = fixture();
    value.report.modelVisibility = "visible";
    expect(() => projectPlaceableResult(JSON.stringify(value.report), value.readbackJson, value.artifacts))
      .toThrow("report.modelVisibility");
  });

  it("rejects a readback payload that is not the exact artifact in the Placeable lineage", () => {
    const value = fixture();
    expect(() => projectPlaceableResult(value.reportJson, "{}", value.artifacts))
      .toThrow("placeable-model-readback-json.bytes");
  });
});
