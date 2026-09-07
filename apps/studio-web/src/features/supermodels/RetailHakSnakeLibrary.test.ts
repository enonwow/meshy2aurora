import { describe, expect, it } from "vitest";
import {
  diffuseTextureResrefsV1,
  retailHakResourceUrlV1,
  SNAKE_MODEL_LINEAGES_V1,
} from "./RetailHakSnakeLibrary";

describe("retail HAK snake model library", () => {
  it("lists the exact existing child models and their declared snake-family supermodels", () => {
    expect(SNAKE_MODEL_LINEAGES_V1).toHaveLength(18);
    expect(SNAKE_MODEL_LINEAGES_V1).toContainEqual({ resref: "fwp_cobr_sap", supermodel: "c_cobra01", family: "Cobra" });
    expect(SNAKE_MODEL_LINEAGES_V1).toContainEqual({ resref: "c_viper_desert_h", supermodel: "c_viper_forest_h", family: "Viper" });
    expect(SNAKE_MODEL_LINEAGES_V1).toContainEqual({ resref: "c_worm2", supermodel: "c_worm", family: "Worm" });
    expect(SNAKE_MODEL_LINEAGES_V1).toContainEqual({ resref: "zcp_nagab", supermodel: "zcp_nagaa", family: "Naga" });
  });

  it("keeps HAK reads on the dedicated same-origin local reference endpoint", () => {
    expect(retailHakResourceUrlV1(
      "/__m2a_nwn_user_reference",
      "hak/cep3_core1.hak",
      "http://127.0.0.1:5174/",
    )).toBe("http://127.0.0.1:5174/__m2a_nwn_user_reference/hak/cep3_core1.hak");
    expect(() => retailHakResourceUrlV1(
      "/__m2a_nwn_user_reference",
      "../private.txt",
      "http://127.0.0.1:5174/",
    )).toThrow(/safe HAK resource path/i);
  });

  it("collects exact diffuse texture resrefs from the canonical MDL readback", () => {
    expect(diffuseTextureResrefsV1({
      schemaVersion: 1,
      format: "binary-mdl",
      nodeTree: {
        roots: [{
          offset: 0,
          number: 0,
          name: "root",
          controllers: [],
          mesh: { vertices: [], normals: [], uv0: [], rawIndices: [], faces: [], textures: ["c_Viper_Desert"] },
          children: [{
            offset: 1,
            number: 1,
            name: "child",
            controllers: [],
            mesh: { vertices: [], normals: [], uv0: [], rawIndices: [], faces: [], textures: ["C_VIPER_DESERT"] },
            children: [],
          }],
        }],
      },
      animations: [],
      diagnostics: [],
    })).toEqual(["c_viper_desert"]);
  });
});
