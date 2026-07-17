import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import config from "./vite.config";

describe("Vite development file boundary", () => {
  it("allows the canonical workspace so the aliased web-WASM binary can load", () => {
    const workspaceRoot = fileURLToPath(new URL("../..", import.meta.url));
    const allow = config.server?.fs?.allow;

    expect(Array.isArray(allow)).toBe(true);
    expect(allow).toContain(workspaceRoot);
  });
});
