// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import type {
  AnimationStudioDocumentV1,
  CreatureAnimationAuthoringV2,
} from "../animation-studio/types";
import { AuthoredAnimationReview } from "./AuthoredAnimationReview";
import type { AnimationStudioReadbackReconciliationV1 } from "./reconcileAnimationStudioReadback";

const roots: Root[] = [];

async function render(element: React.ReactNode) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  await act(async () => root.render(element));
  return container;
}

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
});

const clip = (
  id: string,
  name: string,
  status: "DRAFT" | "VALID" | "INVALID",
): AnimationStudioDocumentV1["authoredClips"][number] => ({
  id,
  name,
  kind: "MOTION",
  status,
  source: {
    kind: "SOURCE_CLIP_COPY",
    sourceRevision: "sha256:source",
    sourceClipName: "wave",
    sourceClipFingerprint: "sha256:wave",
    proceduralTemplate: null,
  },
  lengthSeconds: 1,
  transitionSeconds: 0.1,
  animationRoot: "root",
  tracks: [],
  events: [],
  revision: 1,
});

function studioFixture(): AnimationStudioDocumentV1 {
  return {
    schemaVersion: 1,
    sourceRevision: "sha256:source",
    authoringRevision: 4,
    status: "INVALID",
    authoredClips: [
      clip("clip-one", "owned_wave", "VALID"),
      clip("clip-loop", "owned_loop", "DRAFT"),
      clip("clip-end", "owned_end", "INVALID"),
    ],
  };
}

function authoringFixture(): CreatureAnimationAuthoringV2 {
  return {
    schemaVersion: 2,
    profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
    modelType: "S",
    sourceRevision: "sha256:source",
    authoringRevision: 6,
    assignments: [{
      targetSlot: "cpause1",
      sourceKind: "CUSTOM",
      sourceClipName: null,
      customAnimationId: "custom-one",
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "custom-one",
        ownership: "USER_OWNED",
      },
    }, {
      targetSlot: "cwalk",
      sourceKind: "CUSTOM",
      sourceClipName: null,
      customAnimationId: "custom-phased",
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "custom-phased",
        ownership: "USER_OWNED",
      },
    }],
    fallbacks: [],
    customAnimations: [{
      id: "custom-one",
      name: "Wave one-shot",
      playback: "ONE_SHOT",
      clipReference: {
        sourceKind: "AUTHORED_CLIP",
        sourceClipName: null,
        authoredClipId: "clip-one",
      },
      phases: [],
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "clip-one",
        ownership: "USER_OWNED",
      },
    }, {
      id: "custom-phased",
      name: "Combat loop",
      playback: "LOOPING_PHASED",
      clipReference: null,
      phases: [{
        phase: "LOOP",
        clipReference: {
          sourceKind: "AUTHORED_CLIP",
          sourceClipName: null,
          authoredClipId: "clip-loop",
        },
      }, {
        phase: "END",
        clipReference: {
          sourceKind: "AUTHORED_CLIP",
          sourceClipName: null,
          authoredClipId: "clip-end",
        },
      }],
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "custom-phased",
        ownership: "USER_OWNED",
      },
    }],
  };
}

describe("AuthoredAnimationReview", () => {
  it("shows clip status, provenance, Base 42 usage, custom modes, fingerprint and readback", async () => {
    const reconciliation: AnimationStudioReadbackReconciliationV1 = {
      schemaVersion: 1,
      status: "MISMATCH",
      checkedClipCount: 3,
      matchedClipIds: ["clip-one"],
      diagnostics: [{
        schemaVersion: 1,
        code: "M2A-ANIMATION-READBACK-CLIP-NOT-VALID",
        path: "authoredClips.clip-loop.status",
        level: "BLOCKING",
        message: "Draft authored animation cannot be reconciled.",
        action: "Validate the authored clip before building.",
      }],
    };
    const container = await render(
      <AuthoredAnimationReview
        studio={studioFixture()}
        authoring={authoringFixture()}
        studioFingerprintSha256={"a".repeat(64)}
        reconciliation={reconciliation}
      />,
    );

    expect(container.textContent).toContain("Authored animations");
    expect(container.textContent).toContain("Source GLB unchanged");
    expect(container.textContent).toContain("aaaaaaaaaaaa...");
    expect(container.textContent).toContain("Readback MISMATCH");
    expect(container.textContent).toContain("owned_wave");
    expect(container.textContent).toContain("VALID");
    expect(container.textContent).toContain("SOURCE_CLIP_COPY");
    expect(container.textContent).toContain("wave");
    expect(container.textContent).toContain("Base 42: cpause1");
    expect(container.textContent).toContain("Wave one-shot");
    expect(container.textContent).toContain("ONE SHOT");
    expect(container.textContent).toContain("Combat loop");
    expect(container.textContent).toContain("LOOPING PHASED");
    expect(container.textContent).toContain("LOOP: owned_loop");
    expect(container.textContent).toContain("END: owned_end");
    expect(container.textContent).toContain("M2A-ANIMATION-READBACK-CLIP-NOT-VALID");
  });
});
