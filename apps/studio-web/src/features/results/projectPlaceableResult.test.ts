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
    projectIdentity: {
      schemaVersion: 1,
      projectId: "placeable-project",
      projectName: "Placeable Project",
      projectRevision: 7,
    },
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
    collisionCompleteness: "ascii_pwk_emitted_runtime_readback_passed",
    resources: [
      { container: "HAK", role: "PLACEABLES_2DA", resref: "placeables", resourceType: 2017, byteLength: 11, sha256: hash("d") },
      { container: "HAK", role: "MODEL", resref: "m2a_s1_plc_ped", resourceType: 2002, byteLength: 2, sha256: hash("b") },
      { container: "HAK", role: "PLACEABLE_WALKMESH", resref: "m2a_s1_plc_ped", resourceType: 2053, byteLength: 3, sha256: hash("9") },
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
  const artifacts = [
    artifact("placeable-package-hak", "HAK", report.hakFileName, [1, 2, 3], report.hakSha256),
    artifact("placeable-model-mdl", "MODEL", `${report.modelResref}.mdl`, [4, 5], report.mdlSha256),
    artifact("placeable-proof-module", "MODULE", report.moduleFileName, [6, 7, 8, 9], report.moduleSha256),
    artifact("placeable-report-json", "JSON_REPORT", "m2a_s1_plc_mod-placeable-materialization-report.json", [...new TextEncoder().encode(reportJson)], hash("9")),
  ];
  return { report, reportJson, artifacts };
}

describe("projectPlaceableResult", () => {
  it("projects the offline statuses, exact identities, placement and resource types", () => {
    const value = fixture();
    const result = projectPlaceableResult(value.reportJson, value.artifacts);
    expect(result.status).toBe("OFFLINE_ADMISSION_PASSED");
    expect(result.profile).toBe("STATIC_PLACEABLE");
    expect(result.projectIdentity).toEqual(value.report.projectIdentity);
    expect(result.appearanceRow).toBe(16500);
    expect(result.componentStatuses.proof).toBe("not_tested");
    expect(result.modelVisibility).toBe("not_tested");
    expect(result.proofCompleteness).toBe("missing");
    expect(result.resources).toContainEqual(expect.objectContaining({
      resref: "m2a_s1_plc_utp",
      resourceType: 2044,
    }));
  });

  it("rejects a stale archive artifact instead of presenting a mixed lineage", () => {
    const value = fixture();
    value.artifacts[0] = { ...value.artifacts[0], sha256: hash("8") };
    expect(() => projectPlaceableResult(value.reportJson, value.artifacts))
      .toThrow("placeable-package-hak.sha256");
  });

  it("rejects a visual-proof claim at the offline Studio boundary", () => {
    const value = fixture();
    value.report.modelVisibility = "visible";
    expect(() => projectPlaceableResult(JSON.stringify(value.report), value.artifacts))
      .toThrow("report.modelVisibility");
  });
});
