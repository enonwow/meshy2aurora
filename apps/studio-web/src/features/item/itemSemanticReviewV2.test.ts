import { describe, expect, it } from "vitest";
import {
  canonicalItemSemanticReviewV2,
  hashItemSemanticReviewV2,
  ITEM_REVIEW_VIEWS_V2,
  ITEM_SEMANTIC_CHECKS_V2,
  validateItemSemanticReviewV2,
  type ItemSemanticReviewV2,
} from "./itemSemanticReviewV2";

const sha = (value: string) => value.repeat(64);

function review(): ItemSemanticReviewV2 {
  return {
    schemaVersion: 2,
    candidateSha256: sha("a"),
    lineage: { moduleSha256: sha("a"), hakSha256: sha("b") },
    views: ITEM_REVIEW_VIEWS_V2.map((id) => ({
      id,
      technicalStatus: "PASSED",
      ownerReviewed: true,
      candidateSha256: sha("a"),
    })),
    semanticChecks: ITEM_SEMANTIC_CHECKS_V2.map((id) => ({
      id,
      status: "PASSED",
      candidateSha256: sha("a"),
    })),
    ownerStatus: "OWNER_ACCEPTED",
  };
}

describe("ItemSemanticReviewV2", () => {
  it("requires all five candidate-bound views and all six semantic checks", () => {
    expect(validateItemSemanticReviewV2(review())).toEqual({ ok: true, issues: [] });
    const incomplete = review();
    const changed: ItemSemanticReviewV2 = {
      ...incomplete,
      views: incomplete.views.map((view) => view.id === "GROUND"
        ? { ...view, ownerReviewed: false }
        : view),
      semanticChecks: incomplete.semanticChecks.map((check) => check.id === "TRIGGER_DOWN"
        ? { ...check, status: "NOT_EVALUATED" }
        : check),
    };
    expect(validateItemSemanticReviewV2(changed).issues).toEqual(expect.arrayContaining([
      "review view GROUND is incomplete or stale",
      "semantic check TRIGGER_DOWN is incomplete or stale",
    ]));
  });

  it("invalidates otherwise accepted evidence when the candidate hash changes", () => {
    const stale: ItemSemanticReviewV2 = {
      ...review(),
      candidateSha256: sha("c"),
    };
    const result = validateItemSemanticReviewV2(stale);
    expect(result.ok).toBe(false);
    expect(result.issues).toContain("review is not bound to the exact MOD/HAK candidate lineage");
  });

  it("serializes and hashes the exact review deterministically", async () => {
    const first = review();
    const canonical = canonicalItemSemanticReviewV2(first);

    expect(JSON.parse(canonical)).toEqual(first);
    expect(await hashItemSemanticReviewV2(first)).toMatch(/^[0-9a-f]{64}$/);
    expect(await hashItemSemanticReviewV2(first)).toBe(await hashItemSemanticReviewV2(first));
    expect(await hashItemSemanticReviewV2({
      ...first,
      ownerStatus: "OWNER_REJECTED",
    })).not.toBe(await hashItemSemanticReviewV2(first));
  });
});
