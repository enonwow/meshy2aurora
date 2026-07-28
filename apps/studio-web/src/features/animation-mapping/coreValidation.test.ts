import { describe, expect, it } from "vitest";
import {
  mergeUiAndCoreAnimationDiagnosticsV1,
  projectCoreCreatureAnimationValidationV1,
} from "./coreValidation";

describe("core animation validation projection", () => {
  it("accepts the structured WASM validation contract", () => {
    expect(projectCoreCreatureAnimationValidationV1(JSON.stringify({
      schemaVersion: 1,
      status: "READY",
      mappedBaseSlotCount: 42,
      reviewCount: 0,
      blockingCount: 0,
      customAnimationCount: 1,
      authoringFingerprintSha256: "a".repeat(64),
      diagnostics: [],
    }))).toMatchObject({ status: "READY", mappedBaseSlotCount: 42 });
  });

  it("rejects malformed data and merges diagnostics deterministically", () => {
    expect(() => projectCoreCreatureAnimationValidationV1("{}")).toThrow(/schema/);
    const diagnostic = {
      schemaVersion: 1 as const,
      code: "M2A-TEST",
      path: "baseSlots.cwalk",
      level: "BLOCKING" as const,
      message: "problem",
      action: "fix",
    };
    expect(mergeUiAndCoreAnimationDiagnosticsV1([diagnostic], [diagnostic]))
      .toEqual([diagnostic]);
  });
});
