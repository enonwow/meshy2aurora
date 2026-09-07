import { describe, expect, it } from "vitest";
import { creatureMaterialCapabilitiesV1 } from "./creatureCapabilities";

describe("Creature Material Separation capability matrix", () => {
  it("supports Component and Face V2 only for the product profile", () => {
    expect(creatureMaterialCapabilitiesV1("PRODUCT_300K")).toEqual({
      materialSeparationSupported: true,
      faceSelectionSupported: true,
    });
    expect(creatureMaterialCapabilitiesV1("EXPERIMENTAL_P100K")).toEqual({
      materialSeparationSupported: false,
      faceSelectionSupported: false,
    });
    expect(creatureMaterialCapabilitiesV1("EXPERIMENTAL_P300K")).toEqual({
      materialSeparationSupported: false,
      faceSelectionSupported: false,
    });
  });
});
