import { describe, expect, it } from "vitest";
import { isTileTargetEnabled } from "./feature";

describe("Tile target feature", () => {
  it("keeps Tile hidden until it is explicitly enabled", () => {
    expect(isTileTargetEnabled(undefined)).toBe(false);
    expect(isTileTargetEnabled("")).toBe(false);
    expect(isTileTargetEnabled("0")).toBe(false);
    expect(isTileTargetEnabled("true")).toBe(false);
    expect(isTileTargetEnabled("1")).toBe(true);
  });
});
