import { describe, expect, it } from "vitest";
import { isMeshyLabEnabled } from "./feature";

describe("isMeshyLabEnabled", () => {
  it("keeps the optional Meshy Lab disabled until it is explicitly enabled", () => {
    expect(isMeshyLabEnabled(undefined)).toBe(false);
    expect(isMeshyLabEnabled("0")).toBe(false);
    expect(isMeshyLabEnabled("1")).toBe(true);
  });
});
