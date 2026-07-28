// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import { FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 } from "../source/directCreatureAnimationProfile";
import { projectAnimationCatalogRowsV1 } from "./catalog";
import {
  canBuildCreatureAnimationPackageV1,
  countAnimationMappingStatusesV1,
  focusFirstBlockingAnimationIssueV1,
  formatAnimationDiagnosticV1,
  getCreatureAnimationMappingStatusV1,
  groupAnimationDiagnosticsV1,
  validateCreatureAnimationAuthoringV1,
} from "./readiness";
import { createCreatureAnimationAuthoringV1 } from "./state";

const inspection = {
  sourceClips: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((name, index) => ({
    clipId: `clip:${index}`,
    name,
    durationSeconds: 1,
    trackCount: 1,
    targetNodeIds: [1],
    targetPaths: ["Hips.translation"],
  })),
};

function fullyMapped() {
  const authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
  authoring.assignments = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((targetSlot) => ({
    targetSlot,
    sourceKind: "SOURCE_CLIP",
    sourceClipName: targetSlot,
    customAnimationId: null,
    provenance: {
      provider: "SOURCE_GLB",
      assetId: "source",
      ownership: "USER_OWNED",
    },
  }));
  return authoring;
}

describe("creature animation mapping readiness v1", () => {
  it("prioritizes BLOCKED over review and permits build only for READY", () => {
    const empty = createCreatureAnimationAuthoringV1("sha256:source", "S");
    const diagnostics = validateCreatureAnimationAuthoringV1(empty, inspection);
    expect(diagnostics).toHaveLength(42);
    expect(getCreatureAnimationMappingStatusV1(diagnostics)).toBe("BLOCKED");
    expect(canBuildCreatureAnimationPackageV1("BLOCKED")).toBe(false);
    expect(canBuildCreatureAnimationPackageV1("NEEDS_REVIEW")).toBe(false);
    expect(canBuildCreatureAnimationPackageV1("READY")).toBe(true);

    const ready = fullyMapped();
    expect(getCreatureAnimationMappingStatusV1(
      validateCreatureAnimationAuthoringV1(ready, inspection),
    )).toBe("READY");
    expect(countAnimationMappingStatusesV1(
      projectAnimationCatalogRowsV1(ready, inspection),
    )).toEqual({ mapped: 42, review: 0, blockers: 0, custom: 0 });
  });

  it("keeps an unreviewed fallback at NEEDS_REVIEW and an accepted one at READY", () => {
    const authoring = fullyMapped();
    authoring.assignments = authoring.assignments.filter(
      ({ targetSlot }) => targetSlot !== "cdamager",
    );
    authoring.fallbacks = [{
      id: "fallback",
      targetSlot: "cdamager",
      sourceSlot: "cdamagel",
      reason: "Mirror",
      review: "PENDING",
    }];
    expect(getCreatureAnimationMappingStatusV1(
      validateCreatureAnimationAuthoringV1(authoring, inspection),
    )).toBe("NEEDS_REVIEW");
    authoring.fallbacks[0].review = "ACCEPTED";
    expect(getCreatureAnimationMappingStatusV1(
      validateCreatureAnimationAuthoringV1(authoring, inspection),
    )).toBe("READY");
  });

  it("groups, formats and focuses actionable blocking diagnostics", () => {
    const diagnostics = validateCreatureAnimationAuthoringV1(
      createCreatureAnimationAuthoringV1("sha256:source", "S"),
      inspection,
    );
    const groups = groupAnimationDiagnosticsV1(diagnostics);
    expect(groups.BASE_COVERAGE).toHaveLength(42);
    expect(formatAnimationDiagnosticV1(diagnostics[0])).toMatch(/assign/i);

    const button = document.createElement("button");
    button.dataset.animationDiagnosticCode = diagnostics[0].code;
    document.body.append(button);
    expect(focusFirstBlockingAnimationIssueV1(diagnostics)).toBe(true);
    expect(document.activeElement).toBe(button);
  });

  it("fails closed for a portable supermodel identity until a provider is packaged", () => {
    const authoring = fullyMapped();
    authoring.assignments[0] = {
      targetSlot: "ca1slashl",
      sourceKind: "INHERITED_SUPERMODEL",
      sourceClipName: null,
      customAnimationId: null,
      provenance: {
        provider: "COMPATIBLE_SUPERMODEL",
        assetId: "builtin:c_horror",
        ownership: "ENGINE_INHERITED",
      },
    };
    expect(validateCreatureAnimationAuthoringV1(authoring, inspection))
      .toContainEqual(expect.objectContaining({
        code: "M2A-ANIMATION-SUPERMODEL-PROVIDER-UNAVAILABLE",
        level: "BLOCKING",
      }));
  });
});
