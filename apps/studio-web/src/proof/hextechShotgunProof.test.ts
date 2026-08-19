import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("persistent Hextech Shotgun proof page", () => {
  it("boots the exact local proof without importing owner concept art", () => {
    const html = readFileSync(
      new URL("../../proof/hextech-shotgun.html", import.meta.url),
      "utf8",
    );

    expect(html).toContain("Hextech Shotgun · standalone proof");
    expect(html).toContain("/src/proof/hextechShotgunProof.ts");
    expect(html).not.toContain("concept");
  });
});
