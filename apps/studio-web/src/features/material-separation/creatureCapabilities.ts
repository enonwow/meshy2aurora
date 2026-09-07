import type { CreatureConversionProfileV1 } from "../source/InputsPanel";

export interface CreatureMaterialCapabilitiesV1 {
  readonly materialSeparationSupported: boolean;
  readonly faceSelectionSupported: boolean;
}

const PRODUCT_CAPABILITIES: CreatureMaterialCapabilitiesV1 = Object.freeze({
  materialSeparationSupported: true,
  faceSelectionSupported: true,
});

const EXPERIMENT_CAPABILITIES: CreatureMaterialCapabilitiesV1 = Object.freeze({
  materialSeparationSupported: false,
  faceSelectionSupported: false,
});

export function creatureMaterialCapabilitiesV1(
  profile: CreatureConversionProfileV1,
): CreatureMaterialCapabilitiesV1 {
  return profile === "PRODUCT_300K" ? PRODUCT_CAPABILITIES : EXPERIMENT_CAPABILITIES;
}
