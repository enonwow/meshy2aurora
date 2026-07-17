// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import type { BinaryMdlInspectionReport, BinaryReadbackValidationStatus } from "../preview/types";
import type { CanonicalConversionGate, CanonicalResultSnapshot } from "../results/projectCanonicalResult";
import { ConversionReadiness } from "./ConversionReadiness";
import { projectConversionReadiness } from "./projectConversionReadiness";

const roots: Root[] = [];

function gate(code: string, severity: string, path: string): CanonicalConversionGate {
  return {
    schemaVersion: 1,
    code,
    severity,
    path,
    expected: "policy satisfied",
    actual: "policy violated",
    message: `${code} canonical evidence`,
  };
}

function resultFixture(gates: CanonicalConversionGate[] = []): CanonicalResultSnapshot {
  return {
    status: "M6_MODEL_PACKAGE_MATERIALIZED",
    sourceMetrics: { nodes: 2, meshes: 1, vertices: 3, triangles: 1, animations: 1 },
    convertedMetrics: { nodes: 2, meshes: 1, vertices: 3, triangles: 1, animations: 1 },
    geometry: { vertices: 3, triangles: 1, joints: 1, deformation: "SKIN" },
    animation: { sourceName: "walk", outputName: "cwalk", durationSeconds: 1, hasMotion: true },
    texture: { width: 2, height: 2, pixelFormat: "RGBA8", byteLength: 16 },
    resrefs: { model: "model", texture: "texture" },
    appearance: { appendedRow: 1, sourcePrefixPreserved: true, policy: "PRESERVED_AND_APPENDED" },
    hak: { byteLength: 1, sha256: "a".repeat(64), entryCount: 3 },
    outputs: {},
    resources: [],
    semanticEvidence: { semanticDiff: [], deviations: [] },
    conversionEvidence: {
      schemaVersion: 1,
      conversionEligible: !gates.some(({ severity }) => severity === "BLOCKING"),
      policies: { engineFacingProof: "OPEN_M6", uvRuntimeProof: "OPEN_M6" },
      gates,
      diagnostics: [],
    },
    packageAssemblyEvidence: { strictReconciled: true, resourceCount: 3, artifactCount: 5 },
    artifacts: [],
    reportJson: "{}",
    summaryJson: "{}",
    manifestJson: "{}",
  };
}

function readbackFixture(status: BinaryReadbackValidationStatus = "PASS"): BinaryMdlInspectionReport {
  const warnings = status === "WARNING" ? 1 : 0;
  const errors = status === "ERROR" ? 1 : 0;
  return {
    schemaVersion: 1,
    format: "BINARY_MDL",
    nodeTree: { roots: [{ offset: 10, number: 1, name: "root", controllers: [], children: [] }] },
    animations: [],
    diagnostics: status === "PASS" ? [] : [{
      schemaVersion: 1,
      code: status === "WARNING" ? "READBACK_WARN" : "READBACK_ERROR",
      severity: status,
      offset: 10,
      context: "canonical readback evidence",
    }],
    validation: {
      status,
      structure: { schemaVersion: 1, format: "BINARY_MDL", rootNodeCount: 1, hasRootNodes: true, structuralErrors: [] },
      diagnostics: { total: warnings + errors, warnings, errors, informational: 0, unrecognizedSeverity: 0 },
    },
  };
}

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
});

describe("projectConversionReadiness", () => {
  it("emits PASS, NOT_CHECKED and exact OPEN policy without inventing positive conversion rules", () => {
    const projection = projectConversionReadiness(resultFixture(), readbackFixture());
    expect(projection.items
      .filter(({ id }) => ["GEOMETRY", "MATERIALS_TEXTURES", "RIG", "ANIMATIONS"].includes(id))
      .every(({ status }) => status === "NOT_CHECKED"))
      .toBe(true);
    expect(projection.items.find(({ id }) => id === "BINARY_READBACK")).toMatchObject({ status: "PASS", checkCount: 1 });
    expect(projection.items.find(({ id }) => id === "PACKAGE_ASSEMBLY")).toMatchObject({ status: "PASS", checkCount: 8 });
    expect(projection.items.find(({ id }) => id === "RUNTIME_PROOF")).toMatchObject({ status: "OPEN", statusLabel: "OPEN_M6" });
  });

  it("maps real gate evidence to WARNING/FAIL and preserves every item in Validation", () => {
    const gates = [
      gate("M3A-MATERIAL-LIMIT", "WARNING", "sourceSelection.materials"),
      gate("M3A-WEIGHT-SUM-INVALID", "BLOCKING", "creature.segments[0].weights"),
    ];
    const projection = projectConversionReadiness(resultFixture(gates), readbackFixture("WARNING"));
    expect(projection.items.find(({ id }) => id === "MATERIALS_TEXTURES")?.status).toBe("WARNING");
    expect(projection.items.find(({ id }) => id === "RIG")?.status).toBe("FAIL");
    expect(projection.items.find(({ id }) => id === "BINARY_READBACK")?.status).toBe("WARNING");
    for (const item of projection.items.filter(({ status }) => status === "WARNING" || status === "FAIL")) {
      expect(projection.validation.some(({ category }) => category === item.id)).toBe(true);
    }
    expect(projection.validation.find(({ code }) => code === "M3A-MATERIAL-LIMIT")).toMatchObject({
      severity: "WARNING", expected: "policy satisfied", actual: "policy violated",
    });
  });

  it("maps readback ERROR to FAIL with its exact Validation entry", () => {
    const projection = projectConversionReadiness(resultFixture(), readbackFixture("ERROR"));
    expect(projection.items.find(({ id }) => id === "BINARY_READBACK")?.status).toBe("FAIL");
    expect(projection.validation).toContainEqual(expect.objectContaining({
      category: "BINARY_READBACK", code: "READBACK_ERROR", severity: "FAIL", path: "byteOffset:10",
    }));
  });
});

describe("ConversionReadiness", () => {
  it("renders canonical statuses, counts and Validation evidence", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    await act(async () => root.render(
      <ConversionReadiness
        result={resultFixture([gate("M3A-TRIANGLE-BUDGET-WARNING", "WARNING", "sourceSelection.meshInstances")])}
        readback={readbackFixture()}
      />,
    ));
    expect(container.textContent).toContain("ELIGIBLE WITH WARNINGS");
    expect(container.textContent).toContain("OPEN_M6");
    expect(container.textContent).toContain("3 reconciled resource(s); 5 reconciled artifact(s).");
    expect(container.textContent).toContain("M3A-TRIANGLE-BUDGET-WARNING");
    expect(container.textContent).toContain("policy violated");
  });
});
