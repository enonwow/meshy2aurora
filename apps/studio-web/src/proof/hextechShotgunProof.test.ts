import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("persistent Hextech Shotgun offline renderer page", () => {
  it("boots the exact local source check without claiming owner proof", () => {
    const html = readFileSync(
      new URL("../../proof/hextech-shotgun.html", import.meta.url),
      "utf8",
    );

    expect(html).toContain("Hextech Shotgun · standalone source check");
    expect(html).toContain("Toolset/NWN not tested · owner proof required");
    expect(html).not.toContain("visual proof");
    expect(html).toContain("/src/proof/hextechShotgunProof.ts");
    expect(html).not.toContain("concept");
  });
});
