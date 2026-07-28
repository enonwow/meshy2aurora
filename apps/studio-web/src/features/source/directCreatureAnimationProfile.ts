export const FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 = [
  "ca1slashl",
  "ca1slashr",
  "ca1stab",
  "creach",
  "cconjure1",
  "ccastout",
  "cparryl",
  "cparryr",
  "cdodgelr",
  "cdodges",
  "creadyr",
  "creadyl",
  "cdamagel",
  "cdamager",
  "cdamages",
  "ckdbck",
  "ckdbckps",
  "ckdbckdie",
  "cguptokdb",
  "cgustandb",
  "cwalk",
  "crun",
  "ccwalkf",
  "ccwalkb",
  "ccwalkl",
  "ccwalkr",
  "cpause1",
  "chturnl",
  "chturnr",
  "ctaunt",
  "cclosel",
  "ccloseh",
  "cgetmid",
  "ckdbckdmg",
  "ccastoutlp",
  "cspasm",
  "cappear",
  "cdisappear",
  "cgetmidlp",
  "cdead",
  "cdisappearlp",
  "ccturnr",
] as const;

export type DirectCreatureBaseSlotV1 =
  (typeof FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1)[number];

const requiredNames = new Set<string>(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1);

export function hasFullNativeDirectCreatureProfileV1(
  clips: readonly { readonly name: string | null }[],
): boolean {
  if (clips.length !== FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.length) return false;
  const names = new Set(
    clips
      .map((clip) => clip.name?.toLowerCase() ?? "")
      .filter((name) => name.length > 0),
  );
  return names.size === requiredNames.size
    && [...requiredNames].every((name) => names.has(name));
}

export function canOfferGeneratedHumanoidProfileV1(source: {
  readonly skinCount: number;
  readonly boneCount: number;
  readonly clips: readonly unknown[];
}): boolean {
  return source.skinCount === 1
    && source.boneCount >= 11
    && source.clips.length > 0;
}
