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

  it("shows the full accessory stabilization audit for a creature product", async () => {
    const result = resultFixture();
    result.skinAccessoryStabilization = {
      schemaVersion: 2,
      mode: "AUTO",
      auditedClipCount: 42,
      weldTolerance: 0.00001,
      componentCount: 5,
      detachedComponentCount: 4,
      riskyComponentCount: 2,
      stabilizedComponentCount: 2,
      changedVertexCount: 128,
      components: [{
        segmentIndex: 0,
        componentIndex: 1,
        triangleCount: 64,
        vertexCount: 48,
        isPrimaryBody: false,
        centroid: [1, 2, 3],
        activeBoneCount: 4,
        dominantBoneName: "LeftArm",
        dominantBoneShare: 0.503,
        riskReasons: ["PAIR_DISTANCE_RATIO_ABOVE_LIMIT"],
        action: "STABILIZED",
        selectedBoneId: 7,
        selectedBoneName: "Spine02",
        changedVertexCount: 48,
        before: {
          sampledClipCount: 42,
          sampledPoseCount: 128,
          sampledVertexCount: 48,
          vertexSamplingMode: "EXACT_ALL_VERTICES",
          timeSamplingTruncatedClipCount: 0,
          maxPairDistanceRatio: 2.39,
          maxPairDistanceError: 0.1,
          minAxisAlignment: 0.26,
        },
        after: {
          sampledClipCount: 42,
          sampledPoseCount: 128,
          sampledVertexCount: 48,
          vertexSamplingMode: "EXACT_ALL_VERTICES",
          timeSamplingTruncatedClipCount: 0,
          maxPairDistanceRatio: 1,
          maxPairDistanceError: 0,
          minAxisAlignment: 0.26,
        },
      }],
      warnings: ["Component 1 was stabilized to Spine02."],
    };
    const container = await render(
      <ReviewModelDetails
        result={result}
        readback={readbackFixture}
        activeViewport="CONVERTED"
        onViewportChange={vi.fn()}
        onInspectBinary={vi.fn()}
        sourceViewport={<div />}
        convertedReadbackViewport={<div />}
      />,
    );

    expect(container.textContent).toContain("Accessory skinning audit");
    expect(container.textContent).toContain("42 animation clips audited");
    expect(container.textContent).toContain("2/4 detached components stabilized");
    expect(container.textContent).toContain("Segment 0 · component 1");
    expect(container.textContent).toContain("LeftArm → Spine02");
    expect(container.textContent).toContain("2.390 → 1.000");
    expect(container.textContent).toContain("48/48 vertices · EXACT_ALL_VERTICES");
    expect(container.textContent).toContain("Component 1 was stabilized to Spine02.");
  });

  it("shows the exact downloadable demo MOD and module-local UTC identity", async () => {
    const result = resultFixture();
    result.demo = {
      schemaVersion: 2,
      moduleResref: "m2c2demo",
      moduleDisplayName: "Meshy2Aurora procedural humanoid proof",
      areaResref: "m2c2area",
      areaDisplayName: "Meshy2Aurora procedural humanoid proof area",
      creatureResref: "m2c2utc",
      hakResref: "m2c2hak",
      appearanceRow: 42,
      resourceCount: 6,
      byteLength: 2048,
      sha256: "7".repeat(64),
      semanticReadbackStatus: "PASS",
    };
    const container = await render(
      <ReviewModelDetails
        result={result}
        readback={readbackFixture}
        activeViewport="CONVERTED"
        onViewportChange={vi.fn()}
        onInspectBinary={vi.fn()}
        sourceViewport={<div />}
        convertedReadbackViewport={<div />}
      />,
    );
    expect(container.textContent).toContain("Demo module / UTC");
    expect(container.textContent).toContain("m2c2demo.mod");
    expect(container.textContent).toContain("Module “Meshy2Aurora procedural humanoid proof”");
    expect(container.textContent).toContain(
      "Area “Meshy2Aurora procedural humanoid proof area” (m2c2area)",
    );
    expect(container.textContent).toContain("UTC m2c2utc");
    expect(container.textContent).toContain("appearance row 42");
  });

  it("shows which source material fields were mapped and which remain unsupported", async () => {
    const result = resultFixture();
    result.materialFidelity = {
      schemaVersion: 1,
      materialSlot: 0,
      sourceMaterialId: 0,
      baseColorFactor: [0.8, 1, 0.5, 1],
      baseColorFactorBaked: true,
      alphaMode: "OPAQUE",
      alphaChannelPreserved: true,
      metallicFactor: 0,
      roughnessFactor: 0.7,
      normalTexturePresent: true,
      emissiveFactor: [0.1, 0, 0],
      emissiveTexturePresent: false,
      doubleSided: true,
      auroraMaterialProfile: "CLASSIC_DIFFUSE_TGA_SAFE_V1",
      mappedFields: ["baseColorTexture->diffuseTga", "baseColorFactor->diffuseTgaPixels"],
      unsupportedFields: ["roughnessFactor", "normalTexture", "emissiveFactor", "doubleSided"],
    };
    const container = await render(
      <ReviewModelDetails
        result={result}
        readback={readbackFixture}
        activeViewport="CONVERTED"
        onViewportChange={vi.fn()}
        onInspectBinary={vi.fn()}
        sourceViewport={<div />}
        convertedReadbackViewport={<div />}
      />,
    );
    expect(container.textContent).toContain("Material fidelity");
    expect(container.textContent).toContain("Base-color factor baked");
    expect(container.textContent).toContain("baseColorTexture→diffuseTga");
    expect(container.textContent).toContain("Unsupported in safe classic profile");
    expect(container.textContent).toContain("roughnessFactor");
    expect(container.textContent).toContain("doubleSided");
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
