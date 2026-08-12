import type { ItemSemanticCheckIdV2 } from "./itemAuthoringRecipeV2";

export type ItemReviewViewIdV2 =
  | "ASSEMBLY"
  | "ITEM_PROPERTIES"
  | "GROUND"
  | "EQUIPPED"
  | "INVENTORY_ICON";

export interface ItemSemanticReviewV2 {
  readonly schemaVersion: 2;
  readonly candidateSha256: string;
  readonly lineage: {
    readonly moduleSha256: string;
    readonly hakSha256: string;
  };
  readonly views: readonly {
    readonly id: ItemReviewViewIdV2;
    readonly technicalStatus: "PASSED" | "FAILED";
    readonly ownerReviewed: boolean;
    readonly candidateSha256: string;
  }[];
  readonly semanticChecks: readonly {
    readonly id: ItemSemanticCheckIdV2;
    readonly status: "PASSED" | "FAILED" | "NOT_EVALUATED";
    readonly candidateSha256: string;
  }[];
  readonly ownerStatus: "NOT_REVIEWED" | "OWNER_ACCEPTED" | "OWNER_REJECTED";
}

const SHA256 = /^[0-9a-f]{64}$/;

function canonicalValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.entries(value as Record<string, unknown>)
      .sort(([first], [second]) => first.localeCompare(second))
      .map(([key, nested]) => [key, canonicalValue(nested)]));
  }
  return value;
}

export function canonicalItemSemanticReviewV2(review: ItemSemanticReviewV2): string {
  return JSON.stringify(canonicalValue(review));
}

export async function hashItemSemanticReviewV2(review: ItemSemanticReviewV2): Promise<string> {
  const bytes = new TextEncoder().encode(canonicalItemSemanticReviewV2(review));
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((value) => value.toString(16).padStart(2, "0"))
    .join("");
}

export const ITEM_REVIEW_VIEWS_V2: readonly ItemReviewViewIdV2[] = [
  "ASSEMBLY",
  "ITEM_PROPERTIES",
  "GROUND",
  "EQUIPPED",
  "INVENTORY_ICON",
];

export const ITEM_SEMANTIC_CHECKS_V2: readonly ItemSemanticCheckIdV2[] = [
  "BUTT_OUTER",
  "GRIP_AT_HAND",
  "TRIGGER_DOWN",
  "CORE_CENTERED",
  "MUZZLE_FORWARD",
  "BROAD_SIDE_VISIBLE",
];

export function validateItemSemanticReviewV2(review: ItemSemanticReviewV2) {
  const issues: string[] = [];
  if (
    review.schemaVersion !== 2
    || !SHA256.test(review.candidateSha256)
    || !SHA256.test(review.lineage.moduleSha256)
    || !SHA256.test(review.lineage.hakSha256)
    || review.candidateSha256 !== review.lineage.moduleSha256
  ) {
    issues.push("review is not bound to the exact MOD/HAK candidate lineage");
  }
  for (const id of ITEM_REVIEW_VIEWS_V2) {
    const matches = review.views.filter((view) => view.id === id);
    if (matches.length !== 1) {
      issues.push(`review view ${id} must occur exactly once`);
    } else if (
      matches[0].technicalStatus !== "PASSED"
      || !matches[0].ownerReviewed
      || matches[0].candidateSha256 !== review.candidateSha256
    ) {
      issues.push(`review view ${id} is incomplete or stale`);
    }
  }
  for (const id of ITEM_SEMANTIC_CHECKS_V2) {
    const matches = review.semanticChecks.filter((check) => check.id === id);
    if (matches.length !== 1) {
      issues.push(`semantic check ${id} must occur exactly once`);
    } else if (
      matches[0].status !== "PASSED"
      || matches[0].candidateSha256 !== review.candidateSha256
    ) {
      issues.push(`semantic check ${id} is incomplete or stale`);
    }
  }
  if (review.ownerStatus !== "OWNER_ACCEPTED") {
    issues.push("owner acceptance is missing");
  }
  return { ok: issues.length === 0, issues } as const;
}
