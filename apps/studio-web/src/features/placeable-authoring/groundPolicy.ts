export const PLACEABLE_AUTHORING_GROUND_Y_V1 = 0;
export const PLACEABLE_BELOW_GROUND_TOLERANCE_V1 = 0.01;
export const PLACEABLE_FLOATING_TOLERANCE_V1 = 0.05;

export type PlaceableGroundRelationV1 =
  | { readonly state: "BELOW"; readonly distanceMeters: number }
  | { readonly state: "GROUNDED"; readonly distanceMeters: number }
  | { readonly state: "FLOATING"; readonly distanceMeters: number };

/**
 * Placeable authoring, custom PWK coordinates, grid rendering and pointer
 * projection all use the immutable Aurora authoring ground plane Y=0.
 */
export function placeableGroundRelationV1(minimumY: number): PlaceableGroundRelationV1 {
  if (!Number.isFinite(minimumY)) {
    throw new Error("Placeable ground measurement must be finite");
  }
  if (minimumY < PLACEABLE_AUTHORING_GROUND_Y_V1 - PLACEABLE_BELOW_GROUND_TOLERANCE_V1) {
    return {
      state: "BELOW",
      distanceMeters: PLACEABLE_AUTHORING_GROUND_Y_V1 - minimumY,
    };
  }
  if (minimumY > PLACEABLE_AUTHORING_GROUND_Y_V1 + PLACEABLE_FLOATING_TOLERANCE_V1) {
    return {
      state: "FLOATING",
      distanceMeters: minimumY - PLACEABLE_AUTHORING_GROUND_Y_V1,
    };
  }
  return {
    state: "GROUNDED",
    distanceMeters: Math.abs(minimumY - PLACEABLE_AUTHORING_GROUND_Y_V1),
  };
}
