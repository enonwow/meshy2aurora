// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { BinaryMdlInspectionReport } from "../preview/types";
import type { CanonicalResultSnapshot } from "../results/projectCanonicalResult";
import { pairedReviewMetrics, ReviewModelDetails } from "./ReviewModelDetails";

const roots: Root[] = [];

function resultFixture(): CanonicalResultSnapshot {
  return {
    status: "M6_MODEL_PACKAGE_MATERIALIZED",
    sourceMetrics: { nodes: 8, meshes: 2, vertices: 24, triangles: 12, animations: 1 },
    convertedMetrics: { nodes: 4, meshes: 2, vertices: 24, triangles: 12, animations: 1 },
    geometry: { vertices: 24, triangles: 12, joints: 2, deformation: "SKIN" },
    animation: { sourceName: "walk", outputName: "cwalk", durationSeconds: 1.25, hasMotion: true },
    texture: { width: 2, height: 2, pixelFormat: "RGBA8", byteLength: 60 },
    resrefs: { model: "m2a_model", texture: "m2a_texture" },
    appearance: { appendedRow: 1, sourcePrefixPreserved: true, policy: "PRESERVED_AND_APPENDED" },
    hak: { byteLength: 3, sha256: "a".repeat(64), entryCount: 3 },
    outputs: {},
    resources: [],
    semanticEvidence: { semanticDiff: [], deviations: [] },
    conversionEvidence: {
      schemaVersion: 1,
      conversionEligible: true,
      policies: { engineFacingProof: "OPEN_M6", uvRuntimeProof: "OPEN_M6" },
      gates: [],
      diagnostics: [],
    },
    packageAssemblyEvidence: { strictReconciled: true, resourceCount: 3, artifactCount: 5 },
    runtimeAcceptance: {
      status: "OPEN_M6",
      reason: "No exact owner-verified Aurora/NWN lineage matches these output hashes.",
    },
    animationPlaybackAcceptance: {
      status: "OWNER_PROOF_REQUIRED",
      playbackProofStatus: "not_tested",
      proofCompleteness: "missing",
      reason: "No exact owner animation playback proof.",
    },
    artifacts: [],
    reportJson: "{}",
    summaryJson: "{}",
    manifestJson: "{}",
  };
}

const readbackFixture: BinaryMdlInspectionReport = {
  schemaVersion: 1,
  format: "BINARY_MDL",
  nodeTree: {
    roots: [{ offset: 0, number: 0, name: "root", controllers: [], children: [] }],
  },
  animations: [],
  diagnostics: [],
  validation: {
    status: "PASS",
    structure: {
      schemaVersion: 1,
      format: "BINARY_MDL",
      rootNodeCount: 1,
      hasRootNodes: true,
      structuralErrors: [],
    },
    diagnostics: { total: 0, warnings: 0, errors: 0, informational: 0, unrecognizedSeverity: 0 },
  },
};

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

describe("pairedReviewMetrics", () => {
  it("omits a metric unless it exists in both snapshots", () => {
    expect(pairedReviewMetrics(
      { nodes: 3, vertices: 12 },
      { nodes: 2, triangles: 4 },
    )).toEqual([{ key: "nodes", label: "Nodes", source: 3, converted: 2 }]);
  });
});

describe("ReviewModelDetails", () => {
  it("renders canonical metrics and evidence without inventing a quality score", async () => {
    const inspectBinary = vi.fn();
    const changeViewport = vi.fn();
    const container = await render(
      <ReviewModelDetails
        result={resultFixture()}
        readback={readbackFixture}
        activeViewport="CONVERTED"
        onViewportChange={changeViewport}
        onInspectBinary={inspectBinary}
        sourceViewport={<div>source viewport slot</div>}
        convertedReadbackViewport={<div>readback viewport slot</div>}
      />,
    );

    expect(container.textContent).toContain("readback viewport slot");
    expect(container.textContent).toContain("Verified by binary readback");
    expect(container.textContent).toContain("Canonical writer/readback semantic diff is empty.");
    expect(container.textContent).toContain("24");
    expect(container.textContent).not.toMatch(/quality|\/100/i);

    const badge = [...container.querySelectorAll("button")].find((button) => button.textContent?.includes("Inspect Binary"));
    const sourceTab = [...container.querySelectorAll("button")].find((button) => button.textContent?.includes("Source Model"));
    await act(async () => badge?.click());
    await act(async () => sourceTab?.click());
    expect(inspectBinary).toHaveBeenCalledOnce();
    expect(changeViewport).toHaveBeenCalledWith("SOURCE");
  });

  it("shows canonical semantic differences verbatim when the writer reports them", async () => {
    const result = resultFixture();
    result.semanticEvidence.semanticDiff = ["mesh[0].faces differs"];
    const container = await render(
      <ReviewModelDetails
        result={result}
        readback={readbackFixture}
        activeViewport="SOURCE"
        onViewportChange={vi.fn()}
        onInspectBinary={vi.fn()}
        sourceViewport={<div />}
        convertedReadbackViewport={<div />}
      />,
    );
    expect(container.textContent).toContain("DIFFERENCE REPORTED");
    expect(container.textContent).toContain("mesh[0].faces differs");
    expect(container.textContent).toContain("First mismatch");
    expect(container.querySelector<HTMLAnchorElement>(
      'a[href="#artifact-model-mdl"]',
    )).not.toBeNull();
  });

  it("shows full animation contracts and binary readback for Base 42 and Custom rows", async () => {
    const result = resultFixture();
    result.animationCompletenessEvidence = {
      profile: "FULL_NATIVE42_EXPLICIT_V1",
      requiredClipCount: 42,
      explicitClipCount: 42,
      proceduralClipCount: 0,
      fallbackAliasCount: 0,
      complete: true,
    };
    result.animationBehaviorEvidence = {
      schemaVersion: 1,
      requiredNamespaceClipCount: 42,
      observedClipCount: 42,
      fullNamespaceComplete: true,
      allRequiredContentPresent: true,
      activeMotionComplete: true,
      walkRunDistinct: true,
      essentialStatesDistinct: true,
      deathTransitionTerminalPose: true,
      behaviorCandidateEligible: true,
      clips: [],
      violations: [],
    };
    result.skinAnimationEvidence = {
      schemaVersion: 1,
      activeJointCount: 24,
      requiredClipCount: 5,
      clips: [{
        clipName: "cwalk",
        sampledTimeCount: 3,
        maxMovedVertexCount: 20,
        maxDisplacement: 0.5,
        nonRigidDeformationObserved: true,
      }],
      complete: true,
      violations: [],
    };
    result.animationMappingEvidence = {
      schemaVersion: 1,
      profile: "DIRECT_CREATURE_BASE42_V1",
      sourceRevision: "a".repeat(64),
      authoringRevision: 4,
      authoringFingerprintSha256: "b".repeat(64),
      baseAnimations: [{
        targetSlot: "cwalk",
        resolvedSourceSlot: "cwalk",
        sourceKind: "CUSTOM",
        sourceClipName: "donor_walk",
        customAnimationId: "custom_walk",
        provider: "MESHY",
        assetId: "asset-01",
        ownership: "OWNER_SELECTED",
        viaFallbackSlots: [],
      }],
      customAnimations: [{
        id: "custom_emote",
        playback: "ONE_SHOT",
        phases: [],
        outputClipNames: ["custom_emote"],
        sourceClipNames: ["donor_emote"],
        provider: "LOCAL_FILE",
        assetId: "donor-02",
        ownership: "OWNER_SELECTED",
      }],
      conformance: {
        status: "READY",
        expectedBaseSlotCount: 42,
        materializedBaseSlotCount: 42,
        expectedCustomClipNames: ["custom_emote"],
        materializedCustomClipNames: ["custom_emote"],
      },
    };
    const readback: BinaryMdlInspectionReport = {
      ...readbackFixture,
      animations: [
        {
          offset: 1,
          name: "cwalk",
          length: 1,
          transition: 0.1,
          animationRoot: "root",
          events: [],
          nodeTree: { roots: [] },
        },
        {
          offset: 2,
          name: "custom_emote",
          length: 1,
          transition: 0.1,
          animationRoot: "root",
          events: [],
          nodeTree: { roots: [] },
        },
      ],
    };
    const container = await render(
      <ReviewModelDetails
        result={result}
        readback={readback}
        activeViewport="CONVERTED"
        onViewportChange={vi.fn()}
        onInspectBinary={vi.fn()}
        sourceViewport={<div />}
        convertedReadbackViewport={<div />}
      />,
    );

    expect(container.textContent).toContain("42 required · 42 explicit");
    expect(container.textContent).toContain("CANDIDATE ELIGIBLE");
    expect(container.textContent).toContain("24 active joints");
    expect(container.textContent).toContain("donor_walk");
    expect(container.textContent).toContain("donor_emote → custom_emote");
    expect(container.querySelectorAll('td[data-status="pass"]')).toHaveLength(2);
  });

  it("distinguishes an event-complete creature package from a non-eventful skin package", async () => {
    const eventful = resultFixture();
    eventful.animationEventEvidence = {
      schemaVersion: 1,
      profile: "COMMON_NATIVE_GAMEPLAY_HOOKS_EXPLICIT_V1",
      requiredPairCount: 23,
      satisfiedPairCount: 23,
      totalEventCount: 24,
      unknownEventNames: ["owned_marker"],
      missingPairs: [],
      complete: true,
      authoringCanonical: { byteLength: 1_024, sha256: "8".repeat(64) },
    };
    const eventfulContainer = await render(
      <ReviewModelDetails
        result={eventful}
        readback={readbackFixture}
        activeViewport="CONVERTED"
        onViewportChange={vi.fn()}
        onInspectBinary={vi.fn()}
        sourceViewport={<div />}
        convertedReadbackViewport={<div />}
      />,
    );
    expect(eventfulContainer.textContent).toContain("Creature gameplay events");
    expect(eventfulContainer.textContent).toContain("23/23 required hooks");
    expect(eventfulContainer.textContent).toContain("canonical sidecar SHA-256 888888888888...");

    const nonEventfulContainer = await render(
      <ReviewModelDetails
        result={resultFixture()}
        readback={readbackFixture}
        activeViewport="CONVERTED"
        onViewportChange={vi.fn()}
        onInspectBinary={vi.fn()}
        sourceViewport={<div />}
        convertedReadbackViewport={<div />}
      />,
    );
    expect(nonEventfulContainer.textContent).toContain("NOT INCLUDED");
    expect(nonEventfulContainer.textContent).toContain(
      "did not use the caller-owned Full-42 event authoring lane",
    );
  });

  it.each([
    ["PASS", "Verified by binary readback"],
    ["WARNING", "Binary readback has warnings"],
    ["ERROR", "Binary readback has errors"],
  ] as const)("renders the projected %s readback status", async (status, label) => {
    const readback: BinaryMdlInspectionReport = {
      ...readbackFixture,
      validation: { ...readbackFixture.validation!, status },
    };
    const container = await render(
      <ReviewModelDetails
        result={resultFixture()}
        readback={readback}
        activeViewport="CONVERTED"
        onViewportChange={vi.fn()}
        onInspectBinary={vi.fn()}
        sourceViewport={<div />}
        convertedReadbackViewport={<div />}
      />,
    );
    expect(container.textContent).toContain(label);
    const binaryReadback = [...container.querySelectorAll(".review-model__evidence article")]
      .find((article) => article.querySelector("span")?.textContent === "Binary readback");
    expect(binaryReadback?.querySelector("strong")?.textContent).toBe(status);
  });

  it("shows unavailable rather than PASS when validation evidence is absent", async () => {
    const { validation: _validation, ...readback } = readbackFixture;
    const container = await render(
      <ReviewModelDetails
        result={resultFixture()}
        readback={readback}
        activeViewport="CONVERTED"
        onViewportChange={vi.fn()}
        onInspectBinary={vi.fn()}
        sourceViewport={<div />}
        convertedReadbackViewport={<div />}
      />,
    );
    expect(container.textContent).toContain("Binary readback evidence unavailable");
    expect(container.textContent).not.toContain("Verified by binary readback");
  });
});
