export interface SkinAccessoryComponentBoneOverrideV2 {
  readonly segmentIndex: number;
  readonly componentIndex: number;
  readonly boneName: string;
}

export function parseSkinAccessoryComponentBoneOverridesV2(
  source: string,
): SkinAccessoryComponentBoneOverrideV2[] {
  const overrides: SkinAccessoryComponentBoneOverrideV2[] = [];
  const keys = new Set<string>();
  for (const [lineIndex, rawLine] of source.split(/\r?\n/).entries()) {
    const line = rawLine.trim();
    if (!line) continue;
    const match = /^(\d+)\s*:\s*(\d+)\s*=\s*(.+)$/.exec(line);
    if (!match) {
      throw new Error(
        `Accessory override line ${lineIndex + 1} must use segment:component=BoneName.`,
      );
    }
    const segmentIndex = Number(match[1]);
    const componentIndex = Number(match[2]);
    const boneName = match[3].trim();
    if (!Number.isSafeInteger(segmentIndex) || !Number.isSafeInteger(componentIndex)) {
      throw new Error(`Accessory override line ${lineIndex + 1} has an invalid component index.`);
    }
    if (!boneName || boneName.length > 128 || [...boneName].some((character) => /\p{Cc}/u.test(character))) {
      throw new Error(`Accessory override line ${lineIndex + 1} has an invalid bone name.`);
    }
    const key = `${segmentIndex}:${componentIndex}`;
    if (keys.has(key)) {
      throw new Error(`Accessory override ${key} is declared more than once.`);
    }
    keys.add(key);
    overrides.push({ segmentIndex, componentIndex, boneName });
  }
  return overrides.sort((left, right) =>
    left.segmentIndex - right.segmentIndex
    || left.componentIndex - right.componentIndex
    || left.boneName.localeCompare(right.boneName));
}

export function canonicalSkinAccessoryComponentBoneOverridesV2(source: string): string {
  return parseSkinAccessoryComponentBoneOverridesV2(source)
    .map(({ segmentIndex, componentIndex, boneName }) =>
      `${segmentIndex}:${componentIndex}=${boneName.trim().toLocaleLowerCase("en-US")}`)
    .join("\n");
}
