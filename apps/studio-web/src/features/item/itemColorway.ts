export type ItemWeaponColor = 1 | 2 | 3 | 4;

/** Must stay numerically aligned with core's concrete TGA colorway authoring. */
export function itemWeaponColorwayMultiplier(
  color: ItemWeaponColor | null,
): readonly [number, number, number] {
  switch (color) {
    case 2: return [3 / 4, 7 / 8, 9 / 8];
    case 3: return [9 / 8, 3 / 4, 1 / 2];
    case 4: return [1 / 2, 1 / 2, 1 / 2];
    default: return [1, 1, 1];
  }
}
