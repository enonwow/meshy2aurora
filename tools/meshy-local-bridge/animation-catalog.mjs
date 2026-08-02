import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";

export const MESHY_ANIMATION_CATALOG_V1 = JSON.parse(readFileSync(
  fileURLToPath(new URL(
    "../../contracts/meshy-animation-catalog-v1.json",
    import.meta.url,
  )),
  "utf8",
));

if (
  MESHY_ANIMATION_CATALOG_V1.schemaVersion !== 1
  || typeof MESHY_ANIMATION_CATALOG_V1.snapshotId !== "string"
  || !/^[a-f0-9]{64}$/.test(
    MESHY_ANIMATION_CATALOG_V1.sourceCatalogSha256,
  )
  || !/^[a-f0-9]{64}$/.test(
    MESHY_ANIMATION_CATALOG_V1.curatedActionsSha256,
  )
  || !Array.isArray(MESHY_ANIMATION_CATALOG_V1.actions)
  || MESHY_ANIMATION_CATALOG_V1.actions.length === 0
  || new Set(MESHY_ANIMATION_CATALOG_V1.actions.map(({ id }) => id)).size
    !== MESHY_ANIMATION_CATALOG_V1.actions.length
) {
  throw new Error("Tracked Meshy animation catalog snapshot is invalid.");
}

const computedCuratedActionsSha256 = createHash("sha256")
  .update(JSON.stringify(MESHY_ANIMATION_CATALOG_V1.actions))
  .digest("hex");
if (
  computedCuratedActionsSha256
  !== MESHY_ANIMATION_CATALOG_V1.curatedActionsSha256
) {
  throw new Error(
    "Tracked Meshy animation catalog curated allowlist hash is stale.",
  );
}

export const MESHY_ANIMATION_ACTION_IDS_V1 = new Set(
  MESHY_ANIMATION_CATALOG_V1.actions.map(({ id }) => id),
);

export function animationCatalogRequestIdentityV1() {
  return {
    animationCatalogSnapshotId: MESHY_ANIMATION_CATALOG_V1.snapshotId,
    animationCatalogSha256:
      MESHY_ANIMATION_CATALOG_V1.curatedActionsSha256,
  };
}
