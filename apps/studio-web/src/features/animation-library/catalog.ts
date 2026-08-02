import catalogJson from "../../../../../contracts/community-animation-catalog-v1.json";
import type {
  AnimationPresetCatalogEntryV1,
  AnimationPresetSourceV1,
  CommunityAnimationCatalogV1,
  LoadedAnimationPresetAssetsV1,
} from "./types";

const catalog = catalogJson as CommunityAnimationCatalogV1;
if (
  catalog.schemaVersion !== 1
  || catalog.source !== "EMBEDDED_RELEASE"
  || !/^[0-9a-f]{64}$/.test(catalog.catalogSha256)
  || !Array.isArray(catalog.entries)
) {
  throw new Error("Invalid generated community animation catalog V1");
}

const animationUrlLoaders = import.meta.glob<string>(
  "../../../../../animation-library/presets/*/animation.json",
  { query: "?url&no-inline", import: "default" },
);
const animationTestLoaders = import.meta.env.MODE === "test"
  ? import.meta.glob<string>(
      "../../../../../animation-library/presets/*/animation.json",
      { query: "?raw", import: "default" },
    )
  : {};
const previewLoaders = import.meta.glob<string>(
  "../../../../../animation-library/presets/*/preview.webp",
  { query: "?url", import: "default" },
);

export const COMMUNITY_ANIMATION_CATALOG_V1 = Object.freeze({
  ...catalog,
  entries: Object.freeze(catalog.entries.map((entry) => Object.freeze({
    ...entry,
    authors: Object.freeze(entry.authors.map((author) => Object.freeze({ ...author }))),
    tags: Object.freeze([...entry.tags]),
    requiredBones: Object.freeze([...entry.requiredBones]),
  }))),
}) as Readonly<CommunityAnimationCatalogV1>;

export const ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1 = 100;

export function animationPresetCatalogIdentityV1(
  preset: Pick<AnimationPresetCatalogEntryV1, "presetId" | "presetVersion">,
) {
  return `${preset.presetId}@${preset.presetVersion}`;
}

export function getAnimationPresetCatalogV1(): readonly AnimationPresetCatalogEntryV1[] {
  return COMMUNITY_ANIMATION_CATALOG_V1.entries;
}

export function searchAnimationPresetCatalogV1(
  query: string,
  source: AnimationPresetSourceV1 | "ALL",
  selectedTags: readonly string[],
): readonly AnimationPresetCatalogEntryV1[] {
  return filterAnimationPresetCatalogEntriesV1(
    COMMUNITY_ANIMATION_CATALOG_V1.entries,
    query,
    source,
    selectedTags,
  );
}

export function filterAnimationPresetCatalogEntriesV1(
  entries: readonly AnimationPresetCatalogEntryV1[],
  query: string,
  source: AnimationPresetSourceV1 | "ALL",
  selectedTags: readonly string[],
): readonly AnimationPresetCatalogEntryV1[] {
  const needle = query.trim().toLocaleLowerCase("en-US");
  return entries.filter((entry) => {
    if (source !== "ALL" && entry.source !== source) return false;
    if (!selectedTags.every((tag) => entry.tags.includes(tag))) return false;
    if (!needle) return true;
    return [
      entry.label,
      entry.presetId,
      entry.summary,
      ...entry.tags,
      ...entry.authors.map(({ name }) => name),
    ].some((value) => value.toLocaleLowerCase("en-US").includes(needle));
  });
}

export function projectAnimationPresetCatalogWindowV1(
  entries: readonly AnimationPresetCatalogEntryV1[],
  requestedCount: number,
): readonly AnimationPresetCatalogEntryV1[] {
  const count = Number.isSafeInteger(requestedCount) && requestedCount > 0
    ? requestedCount
    : ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1;
  return entries.slice(0, Math.min(count, entries.length));
}

export async function loadAnimationPresetAssetsV1(
  presetId: string,
  presetVersion: number,
): Promise<LoadedAnimationPresetAssetsV1> {
  const entry = COMMUNITY_ANIMATION_CATALOG_V1.entries.find((candidate) => (
    candidate.presetId === presetId && candidate.presetVersion === presetVersion
  ));
  if (!entry) throw new Error(`Unknown animation preset ${presetId}@${presetVersion}`);
  const animationLoader = findLoader(
    import.meta.env.MODE === "test" ? animationTestLoaders : animationUrlLoaders,
    presetId,
    presetVersion,
    "animation.json",
  );
  if (!animationLoader) {
    throw new Error(`Animation preset ${presetId}@${presetVersion} is missing its embedded payload`);
  }
  const loadedAnimation = await animationLoader();
  const animationJson = import.meta.env.MODE === "test"
    ? loadedAnimation
    : await fetchEmbeddedAnimationTextV1(loadedAnimation, presetId, presetVersion);
  const manifestJson = JSON.stringify(entry);
  const previewLoader = entry.previewPath
    ? findLoader(previewLoaders, presetId, presetVersion, "preview.webp")
    : undefined;
  const previewUrl = previewLoader ? await previewLoader() : null;
  if (previewUrl !== null && import.meta.env.MODE !== "test") {
    const response = await fetch(previewUrl);
    if (!response.ok) {
      throw new Error(
        `Animation preset ${presetId}@${presetVersion} preview is unavailable (${response.status})`,
      );
    }
    await verifyAnimationPresetPreviewBytesV1(entry, new Uint8Array(await response.arrayBuffer()));
  }
  return {
    entry,
    manifestJson,
    animationJson,
    catalogSha256: COMMUNITY_ANIMATION_CATALOG_V1.catalogSha256,
    previewUrl,
  };
}

export async function verifyAnimationPresetPreviewBytesV1(
  entry: AnimationPresetCatalogEntryV1,
  bytes: Uint8Array,
): Promise<void> {
  if (
    entry.previewPath === null
    || entry.previewByteLength === null
    || entry.previewSha256 === null
  ) {
    throw new Error(`Animation preset ${entry.presetId}@${entry.presetVersion} has no preview binding`);
  }
  if (bytes.byteLength !== entry.previewByteLength) {
    throw new Error(
      `Animation preset ${entry.presetId}@${entry.presetVersion} preview size mismatch`,
    );
  }
  const digest = await crypto.subtle.digest("SHA-256", Uint8Array.from(bytes).buffer);
  const sha256 = [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
  if (sha256 !== entry.previewSha256) {
    throw new Error(
      `Animation preset ${entry.presetId}@${entry.presetVersion} preview SHA-256 mismatch`,
    );
  }
}

async function fetchEmbeddedAnimationTextV1(
  url: string,
  presetId: string,
  presetVersion: number,
) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(
      `Animation preset ${presetId}@${presetVersion} payload is unavailable (${response.status})`,
    );
  }
  return response.text();
}

function findLoader<T>(
  loaders: Record<string, () => Promise<T>>,
  presetId: string,
  presetVersion: number,
  fileName: string,
): (() => Promise<T>) | undefined {
  const directory = presetVersion === 1 ? presetId : `${presetId}-v${presetVersion}`;
  const suffix = `/presets/${directory}/${fileName}`;
  return Object.entries(loaders).find(([key]) => key.replaceAll("\\", "/").endsWith(suffix))?.[1];
}
