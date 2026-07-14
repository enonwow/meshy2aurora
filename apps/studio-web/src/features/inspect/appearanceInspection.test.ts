import { describe, expect, it } from "vitest";
import { projectAppearanceInspection } from "./appearanceInspection";

const SHA = "b".repeat(64);

function inspectionFixture() {
  return {
    schemaVersion: 1,
    format: "2DA",
    version: "V2.0",
    sourceSha256: SHA,
    byteLength: 38,
    newline: "LF",
    terminalNewline: true,
    defaultValue: { kind: "TEXT", value: "fallback text" },
    columns: ["LABEL", "ModelType", "RACE"],
    physicalRowCount: 2,
    nextAppendIndex: 2,
    rowLabelMismatchCount: 1,
    diagnostics: [{
      schemaVersion: 1,
      code: "M5-2DA-ROW-LABEL-MISMATCH",
      severity: "WARNING",
      path: "rows[1].label",
      line: 5,
      message: "physical row 1 uses label 9",
    }],
  };
}

describe("projectAppearanceInspection", () => {
  it("projects the successful TwoDaInspectionV1 schema without parsing table text", () => {
    const fixture = inspectionFixture();
    expect(projectAppearanceInspection(JSON.stringify(fixture))).toEqual(fixture);
  });

  it("preserves explicit nulls from the Rust schema", () => {
    const fixture = inspectionFixture();
    fixture.defaultValue = null as unknown as { kind: string; value: string };
    fixture.nextAppendIndex = null as unknown as number;
    fixture.diagnostics[0].line = null as unknown as number;

    const projected = projectAppearanceInspection(JSON.stringify(fixture));
    expect(projected.defaultValue).toBeNull();
    expect(projected.nextAppendIndex).toBeNull();
    expect(projected.diagnostics[0].line).toBeNull();
  });

  it("keeps lexical NULL distinct from TEXT", () => {
    const nullFixture = inspectionFixture();
    nullFixture.defaultValue = { kind: "NULL", value: "ignored" };
    const textFixture = inspectionFixture();
    textFixture.defaultValue = { kind: "TEXT", value: "****" };

    expect(projectAppearanceInspection(JSON.stringify(nullFixture)).defaultValue)
      .toEqual({ kind: "NULL" });
    expect(projectAppearanceInspection(JSON.stringify(textFixture)).defaultValue)
      .toEqual({ kind: "TEXT", value: "****" });
  });

  it("rejects absent fields instead of manufacturing preflight data", () => {
    const fixture = inspectionFixture();
    delete (fixture as Partial<typeof fixture>).columns;

    expect(() => projectAppearanceInspection(JSON.stringify(fixture)))
      .toThrow("Appearance inspection field inspectionJson.columns is missing or has the wrong type");
  });

  it("rejects invalid schema, identity and append index", () => {
    const fixture = inspectionFixture();
    fixture.schemaVersion = 2;
    expect(() => projectAppearanceInspection(JSON.stringify(fixture)))
      .toThrow("inspectionJson.schemaVersion");

    fixture.schemaVersion = 1;
    fixture.format = "CSV";
    expect(() => projectAppearanceInspection(JSON.stringify(fixture)))
      .toThrow("inspectionJson.format");

    fixture.format = "2DA";
    fixture.nextAppendIndex = 65_536;
    expect(() => projectAppearanceInspection(JSON.stringify(fixture)))
      .toThrow("inspectionJson.nextAppendIndex");
  });

  it("rejects malformed JSON", () => {
    expect(() => projectAppearanceInspection("{"))
      .toThrow("Appearance inspection field inspectionJson is missing or has the wrong type");
  });
});
