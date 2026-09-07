import { describe, expect, it } from "vitest";
import { retailNwnResourceUrlV1 } from "./RetailSupermodelDiagnostic";

describe("retailNwnResourceUrlV1", () => {
  it("resolves exact KEY/BIF resource paths below the selected local NWN root", () => {
    expect(retailNwnResourceUrlV1(
      "/@fs/C:/Program Files (x86)/Steam/steamapps/common/Neverwinter Nights",
      "data\\models_01.bif",
      "http://127.0.0.1:5173/",
    )).toBe("http://127.0.0.1:5173/@fs/C:/Program%20Files%20(x86)/Steam/steamapps/common/Neverwinter%20Nights/data/models_01.bif");
    expect(retailNwnResourceUrlV1(
      "/__m2a_nwn_reference",
      "data/nwn_base.key",
      "http://127.0.0.1:5173/",
    )).toBe("http://127.0.0.1:5173/__m2a_nwn_reference/data/nwn_base.key");
  });

  it("rejects external origins and traversal outside the selected retail root", () => {
    expect(() => retailNwnResourceUrlV1(
      "https://example.com/Neverwinter Nights",
      "data/nwn_base.key",
      "http://127.0.0.1:5173/",
    )).toThrow(/same Studio origin/i);
    expect(() => retailNwnResourceUrlV1(
      "/@fs/C:/Program Files (x86)/Steam/steamapps/common/Neverwinter Nights",
      "../secret",
      "http://127.0.0.1:5173/",
    )).toThrow(/safe retail resource path/i);
  });
});
