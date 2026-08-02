// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  AnimationStudioDocumentV1,
  CreatureAnimationAuthoringV2,
} from "../animation-studio/types";
import type { BinaryMdlInspectionReport } from "../preview/types";
import type { CanonicalAnimationStudioEvidenceV1 } from "../results/projectCanonicalResult";
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

function evidenceFixture(): CanonicalAnimationStudioEvidenceV1 {
  return {
    animationStudioSchemaVersion: 1,
    animationStudioFingerprintSha256: "a".repeat(64),
    animationStudioRevision: 4,
    creatureAnimationAuthoringSchemaVersion: 2,
    creatureAnimationAuthoringFingerprintSha256: "b".repeat(64),
    authoredClipCount: 3,
    authoredClipIds: ["clip-one", "clip-loop", "clip-end"],
    authoredClipOutputNames: ["owned_wave", "owned_loop", "owned_end"],
    authoredEventCount: 0,
    customAssignmentCount: 2,
    customRuntimeExposures: [{
      schemaVersion: 1,
      customAnimationId: "custom-one",
      status: "BASE42_ROUTED",
      libraryOutputClipNames: ["Wave one-shot"],
      runtimeBaseSlots: ["cpause1"],
    }, {
      schemaVersion: 1,
      customAnimationId: "custom-phased",
      status: "BASE42_ROUTED",
      libraryOutputClipNames: ["Combat loop_s", "Combat loop", "Combat loop_e"],
      runtimeBaseSlots: ["cwalk"],
    }],
    sourceRevision: "c".repeat(64),
    readbackStatus: "MATCH",
    animationStudioReadback: {
      schemaVersion: 1,
      studioFingerprint: "a".repeat(64),
      sourceRevision: "c".repeat(64),
      status: "MATCH",
      clips: [],
      diagnostics: [],
    },
    sourceGlbUnchanged: true,
    authoredClips: [
      {
        id: "clip-one",
        outputName: "owned_wave",
        kind: "MOTION",
        status: "VALID",
        revision: 1,
        source: {
          kind: "SOURCE_CLIP_COPY",
          sourceRevision: "c".repeat(64),
          sourceClipName: "wave",
          sourceClipFingerprint: "d".repeat(64),
          proceduralTemplate: null,
        },
        keyframeCount: 2,
        eventCount: 0,
        usages: [{
          authoredClipId: "clip-one",
          outputClipName: "cpause1",
          usageKind: "BASE_SLOT",
          baseSlot: "cpause1",
          customAnimationId: "custom-one",
          phase: null,
        }],
      },
      {
        id: "clip-loop",
        outputName: "owned_loop",
        kind: "MOTION",
        status: "VALID",
        revision: 1,
        source: {
          kind: "SOURCE_CLIP_COPY",
          sourceRevision: "c".repeat(64),
          sourceClipName: "loop",
          sourceClipFingerprint: "e".repeat(64),
          proceduralTemplate: null,
        },
        keyframeCount: 2,
        eventCount: 0,
        usages: [],
      },
      {
        id: "clip-end",
        outputName: "owned_end",
        kind: "MOTION",
        status: "VALID",
        revision: 1,
        source: {
          kind: "SOURCE_CLIP_COPY",
          sourceRevision: "c".repeat(64),
          sourceClipName: "end",
          sourceClipFingerprint: "f".repeat(64),
          proceduralTemplate: null,
        },
        keyframeCount: 2,
        eventCount: 0,
        usages: [],
      },
    ],
  };
}

function readbackFixture(): BinaryMdlInspectionReport {
  return {
    schemaVersion: 1,
    format: "BINARY_MDL",
    nodeTree: { roots: [] },
    animations: [{
      offset: 1,
      name: "cpause1",
      length: 1,
      transition: 0.1,
      animationRoot: "root",
      events: [{ name: "impact", time: 0.5 }],
      nodeTree: {
        roots: [{
          offset: 2,
          number: 0,
          name: "root",
          controllers: [{
            controllerName: "position",
            times: [0, 1],
            values: [[0, 0, 0], [1, 0, 0]],
          }],
          children: [],
        }],
      },
    }],
    diagnostics: [],
  };
}

describe("AuthoredAnimationReview", () => {
  it("shows clip status, provenance, Base 42 usage, custom modes, fingerprint and readback", async () => {
    const openMismatch = vi.fn();
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
        evidence={evidenceFixture()}
        animationPlaybackAcceptance={{
          status: "OWNER_PROOF_REQUIRED",
          playbackProofStatus: "not_tested",
          proofCompleteness: "missing",
          reason: "Owner proof required.",
        }}
        readback={readbackFixture()}
        onOpenMismatch={openMismatch}
      />,
    );

    expect(container.textContent).toContain("Authored animations");
    expect(container.textContent).toContain("Source GLB unchanged");
    expect(container.textContent).toContain("aaaaaaaaaaaa...");
    expect(container.textContent).toContain("Binary readback MISMATCH");
    expect(container.textContent).toContain("Runtime playback proof");
    expect(container.textContent).toContain("not_tested");
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
    expect(container.textContent).toContain("Runtime: Base 42 routed via cpause1.");
    expect(container.textContent).toContain("M2A-ANIMATION-READBACK-CLIP-NOT-VALID");
    expect(container.textContent).toContain("Source → Edited → Binary MDL readback");
    expect(container.textContent).toContain("impact@0.500s");
    expect(container.textContent).toContain("Δ [1.000, 0.000, 0.000] · 1.000 m");

    const mismatchButton = Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent?.includes("Open clip or track"));
    await act(async () => mismatchButton?.click());
    expect(openMismatch).toHaveBeenCalledWith("authoredClips.clip-loop.status");
  });
});
