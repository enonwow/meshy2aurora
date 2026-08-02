import snapshot from "../../../../../contracts/meshy-animation-catalog-v1.json";

export interface MeshyAnimationCatalogActionV1 {
  readonly id: number;
  readonly name: string;
  readonly category: string;
  readonly auroraCandidate: string;
}

export interface MeshyAnimationCatalogSnapshotV1 {
  readonly schemaVersion: 1;
  readonly snapshotId: string;
  readonly capturedAt: string;
  readonly sourceUrl: string;
  readonly sourceCatalogSha256: string;
  readonly curatedActionsSha256: string;
  readonly actions: readonly MeshyAnimationCatalogActionV1[];
}

function validateSnapshot(
  value: typeof snapshot,
): MeshyAnimationCatalogSnapshotV1 {
  if (
    value.schemaVersion !== 1
    || !value.snapshotId
    || !/^[a-f0-9]{64}$/.test(value.sourceCatalogSha256)
    || !/^[a-f0-9]{64}$/.test(value.curatedActionsSha256)
    || value.actions.length === 0
    || new Set(value.actions.map(({ id }) => id)).size !== value.actions.length
    || value.actions.some(({ id, name, category, auroraCandidate }) => (
      !Number.isSafeInteger(id)
      || id < 0
      || !name
      || !category
      || !auroraCandidate
    ))
  ) {
    throw new Error("Tracked Meshy animation catalog snapshot is invalid.");
  }
  return {
    ...value,
    schemaVersion: 1,
  };
}

export const MESHY_ANIMATION_CATALOG_V1 = validateSnapshot(snapshot);

export const MESHY_ANIMATION_ACTION_IDS_V1 = new Set(
  MESHY_ANIMATION_CATALOG_V1.actions.map(({ id }) => id),
);

export function meshAnimationActionV1(
  actionId: number,
): MeshyAnimationCatalogActionV1 | undefined {
  return MESHY_ANIMATION_CATALOG_V1.actions.find(({ id }) => id === actionId);
}
