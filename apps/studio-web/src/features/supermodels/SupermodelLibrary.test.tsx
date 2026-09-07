// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import { SupermodelLibrary } from "./SupermodelLibrary";
import type { SupermodelCatalogSessionV1 } from "./filesystem";
import type { SupermodelModelV1 } from "./types";

vi.mock("./SupermodelPreviewViewport", () => ({
  SupermodelPreviewViewport: ({ detail }: { detail: string }) => (
    <div data-testid="supermodel-preview">{detail}</div>
  ),
}));

function model(resref: string, animations: number): SupermodelModelV1 {
  return {
    resref,
    sourceId: "base",
    sourceKind: "BASE_KEY_BIF",
    containerName: "data/models_01.bif",
    sourcePriority: 1_000,
    resourceIndex: 0,
    payloadOffset: 0,
    payloadSize: 244,
    header: {
      schemaVersion: 1,
      format: "BINARY",
      modelName: resref,
      supermodelName: "NULL",
      classification: 4,
      animationScale: 1,
      localAnimationCount: animations,
    },
  };
}

function session(): SupermodelCatalogSessionV1 {
  const horror = model("c_horror", 42);
  const wolf = model("c_wolf", 35);
  return {
    sourceId: "base",
    sourceLabel: "NWN",
    models: [horror, wolf],
    filesByContainer: new Map([["data/models_01.bif", new File([new Uint8Array(244)], "models_01.bif")]]),
    catalog: {
      schemaVersion: 1,
      completeness: "COMPLETE",
      declaredModelCount: 2,
      scannedModelCount: 2,
      failedModelCount: 0,
      supermodelCount: 2,
      entries: [
        { resref: "c_horror", status: "RESOLVED", resource: horror, definitions: [horror], children: ["c_child"], chain: ["c_horror"] },
        { resref: "c_wolf", status: "RESOLVED", resource: wolf, definitions: [wolf], children: ["c_dog"], chain: ["c_wolf"] },
      ],
    },
  };
}

function readbackJson(modelName: string, supermodelName: string, animations: readonly string[] = []) {
  return JSON.stringify({
    schemaVersion: 1,
    format: "nwn1-binary-mdl",
    byteLength: 244,
    model: { name: modelName, classification: 4, animationScale: 1, supermodelName },
    nodeTree: {
      roots: [{
        offset: 12,
        number: 0,
        name: modelName,
        controllers: [],
        children: [{ offset: 24, number: 1, name: "rootdummy", controllers: [], children: [] }],
      }],
    },
    animations: animations.map((name, index) => ({
      offset: 100 + index,
      name,
      length: 1,
      transition: 0.25,
      animationRoot: modelName,
      nodeTree: {
        roots: [{
          offset: 200 + index,
          number: 0,
          name: "rootdummy",
          controllers: [{
            controllerName: "position",
            times: [0, 1],
            values: [[0, 0, 0], [0, 1, 0]],
          }],
          children: [],
        }],
      },
    })),
    diagnostics: [],
  });
}

describe("SupermodelLibrary", () => {
  it("browses all catalog entries and records a candidate without applying it", async () => {
    const host = document.createElement("div");
    const root = createRoot(host);
    const onSelectCandidate = vi.fn();
    const worker = {
      request: vi.fn(() => new Promise<never>(() => {})),
      dispose: vi.fn(),
    };
    await act(async () => root.render(
      <SupermodelLibrary
        onBack={() => undefined}
        onSelectCandidate={onSelectCandidate}
        initialSession={session()}
        createWorker={() => worker}
      />,
    ));
    expect(host.textContent).toContain("c_horror");
    expect(host.textContent).toContain("c_wolf");
    const search = host.querySelector<HTMLInputElement>('input[aria-label="Szukaj supermodelu"]')!;
    await act(async () => {
      const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!;
      setter.call(search, "horror");
      search.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(host.textContent).toContain("c_horror");
    expect(host.textContent).not.toContain("c_wolf");
    await act(async () => host.querySelector<HTMLButtonElement>('[data-supermodel="c_horror"]')!.click());
    const choose = [...host.querySelectorAll<HTMLButtonElement>("button")]
      .find((button) => button.textContent?.includes("Wybierz jako kandydata"))!;
    await act(async () => choose.click());
    expect(onSelectCandidate).toHaveBeenCalledWith(expect.objectContaining({ resref: "c_horror" }));
    await act(async () => root.unmount());
    expect(worker.dispose).toHaveBeenCalled();
  });

  it("keeps an ASCII supermodel browsable and explains why its native viewport is unavailable", async () => {
    const host = document.createElement("div");
    const root = createRoot(host);
    const asciiSession = session();
    asciiSession.models[0] = {
      ...asciiSession.models[0],
      header: { ...asciiSession.models[0].header, format: "ASCII" },
    };
    asciiSession.catalog.entries[0] = {
      ...asciiSession.catalog.entries[0],
      resource: asciiSession.models[0],
      definitions: [asciiSession.models[0]],
      children: [],
    };
    const worker = { request: vi.fn(), dispose: vi.fn() };
    await act(async () => root.render(
      <SupermodelLibrary
        onBack={() => undefined}
        onSelectCandidate={() => undefined}
        initialSession={asciiSession}
        createWorker={() => worker}
      />,
    ));
    await act(async () => host.querySelector<HTMLButtonElement>('[data-supermodel="c_horror"]')!.click());
    await act(async () => undefined);
    expect(host.textContent).toContain("Ograniczenia tego podglądu");
    expect(host.textContent).toContain("ASCII MDL");
    expect(host.textContent).toContain("Analizuj i zastosuj c_horror");
    expect(host.textContent).not.toContain("PROFILE_MISSING");
    expect(worker.request).not.toHaveBeenCalled();
    await act(async () => root.unmount());
  });

  it("applies an arbitrary ASCII selection and exposes structural and motion analysis", async () => {
    const host = document.createElement("div");
    const root = createRoot(host);
    const onAppliedPreview = vi.fn();
    const asciiSession = session();
    asciiSession.models[0] = {
      ...asciiSession.models[0],
      header: { ...asciiSession.models[0].header, format: "ASCII" },
    };
    asciiSession.catalog.entries[0] = {
      ...asciiSession.catalog.entries[0],
      resource: asciiSession.models[0],
      definitions: [asciiSession.models[0]],
      children: [],
    };
    const referenceBytes = new Uint8Array(244).buffer;
    const referenceSha256 = [...new Uint8Array(await crypto.subtle.digest("SHA-256", referenceBytes))]
      .map((value) => value.toString(16).padStart(2, "0"))
      .join("");
    const exactChain = [{
      resref: "c_horror",
      supermodelResref: "NULL",
      format: "ASCII",
      sha256: referenceSha256,
      byteLength: 244,
      localAnimationCount: 42,
      nodeCount: 27,
      controllerCount: 1020,
    }];
    const worker = {
      request: vi.fn(async (request: { type: string }) => {
        if (request.type === "INSPECT_BINARY_MDL") {
          return {
            requestId: "inspect",
            ok: true as const,
            type: "BINARY_MDL_INSPECTED" as const,
            reportJson: readbackJson("c_horror", "NULL", ["crun"]),
          };
        }
        if (
          request.type === "BUILD_REFERENCE_SUPERMODEL_APPLIED_PREVIEW"
          || request.type === "BUILD_REFERENCE_SUPERMODEL_AUTHORED_PREVIEW"
        ) {
          return {
            requestId: "apply",
            ok: true as const,
            type: "REFERENCE_SUPERMODEL_APPLIED_PREVIEW_BUILT" as const,
            readbackJson: readbackJson("m2a_refpreview", "c_horror"),
            artifacts: [],
            authoringJson: JSON.stringify({
              schemaVersion: 2,
              sourceSha256: "c".repeat(64),
              sourceForward: "POSITIVE_Z",
              selectedSupermodelResref: "c_horror",
              exactChainSha256: "d".repeat(64),
              motionContractSha256: "e".repeat(64),
              structuralProfileSha256: "1".repeat(64),
              surfaceAnatomySha256: "2".repeat(64),
              fitterAlgorithm: "STRUCTURAL_ANATOMY_LOCAL_SKINNING_V40",
              baseRigSha256: "f".repeat(64),
              landmarkOverrides: [],
              jointOverrides: [],
              componentBindings: [],
              regionWeightConstraints: [],
              weightOverrides: [],
              contentSha256: "0".repeat(64),
            }),
            targetRigJson: JSON.stringify({
              schemaVersion: 1,
              profileId: "test-target",
              contentSha256: "f".repeat(64),
              nodes: [{
                id: 0,
                name: "root",
                parentId: null,
                bindLocalMatrix: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
              }],
              segments: [],
            }),
            applyReportJson: JSON.stringify({
              schemaVersion: 2,
              status: "APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY",
              supermodelResref: "c_horror",
              referenceFormat: "ASCII",
              referenceSha256,
              modelSha256: "b".repeat(64),
              localAnimationCount: 0,
              inheritedAnimationCount: 1,
              requiredClipCount: 1,
              motionCompatible: false,
              bindPoseCompatible: true,
              skinBindCompatible: true,
              fullCarrierCoverage: true,
              requiredJointCoverage: true,
              skinInfluenceCoverage: true,
              inheritedClipCoverage: true,
              visibleMotionCoverage: false,
              seamViolationCount: 9,
              motionQualityStatus: "BLOCKED",
              runtimeReadiness: "DIAGNOSTIC_PREVIEW_ONLY_MOTION_QUALITY_BLOCKED",
              experimentalAllowExcessiveSkinBranchRepair: true,
              exactChain,
              structuralAnalysis: {
                status: "REFERENCE_SUPERMODEL_STRUCTURALLY_READY",
                selectedSupermodelResref: "c_horror",
                selectedFormat: "ASCII",
                selectedSha256: referenceSha256,
                motionCarrierResref: "c_horror",
                carrierNodeCount: 27,
                carrierControllerCount: 1020,
                inheritedAnimationNames: ["crun"],
                structuralErrors: [],
                exactChain,
                retailPayloadCopied: false,
              },
              rigAnalysis: {
                algorithm: "HIERARCHY_CHAIN_SEGMENT_DISTANCE_TOPOLOGY_SMOOTH_V7",
                carrierNodeCount: 27,
                fullCarrierCoverage: true,
                requiredJointCoverage: true,
                allowedBoneCount: 26,
                weightedBoneCount: 26,
                activeWeightedBoneCount: 26,
                unweightedRequiredJointNames: [],
                passiveUnweightedJointNames: ["impact"],
                skinInfluenceCoverage: true,
                surfaceVertexCount: 24,
                surfaceTriangleCount: 12,
                duplicatePositionGroupCount: 8,
                noReferencePayloadCopied: true,
              },
              retailPayloadCopied: false,
              motionQuality: {
                status: "BLOCKED",
                edgeOutsideHardLimitCount: 4,
                edgeOutsideHardAllowedCount: 0,
                triangleAreaCollapseCount: 2,
                triangleAreaCollapseAllowedCount: 0,
                seamPairViolationCount: 9,
                seamPairAllowedCount: 1,
                visibleAnchorMotionViolationCount: 0,
                requiredClipCount: 1,
                sampledClipCount: 1,
                inheritedClipCoverage: true,
                requiredJointCount: 26,
                jointClipRequiredCount: 1,
                jointClipPassCount: 0,
                jointClipCoverage: false,
                visibleMotionCoverage: false,
                surfaceSeamGate: { status: "BLOCKED_VISIBLE_SEAM" },
              },
            }),
          };
        }
        throw new Error(`Unexpected request ${request.type}`);
      }),
      dispose: vi.fn(),
    };
    const sourceFile = new File([new Uint8Array([1, 2, 3])], "new-creature.glb", {
      type: "model/gltf-binary",
    });
    await act(async () => root.render(
      <SupermodelLibrary
        onBack={() => undefined}
        onSelectCandidate={() => undefined}
        onAppliedPreview={onAppliedPreview}
        sourceFile={sourceFile}
        sourceSha256={"c".repeat(64)}
        sourceForward="POSITIVE_Z"
        initialSession={asciiSession}
        createWorker={() => worker}
      />,
    ));
    await act(async () => host.querySelector<HTMLButtonElement>('[data-supermodel="c_horror"]')!.click());
    await act(async () => Promise.resolve());
    const apply = [...host.querySelectorAll<HTMLButtonElement>("button")]
      .find((button) => button.textContent?.includes("Analizuj i zastosuj c_horror"))!;
    expect(apply.disabled).toBe(false);
    const experimentalSkinning = [...host.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')]
      .find((input) => input.parentElement?.textContent?.includes("ponad limit 5%"))!;
    expect(experimentalSkinning.checked).toBe(false);
    await act(async () => experimentalSkinning.click());
    expect(experimentalSkinning.checked).toBe(true);
    await act(async () => apply.click());
    await act(async () => Promise.resolve());

    const request = worker.request.mock.calls
      .map(([value]) => value as { type: string; sourceGlb?: ArrayBuffer; referenceChainBlob?: ArrayBuffer })
      .find((value) => value.type === "BUILD_REFERENCE_SUPERMODEL_APPLIED_PREVIEW");
    expect(request).toMatchObject({
      type: "BUILD_REFERENCE_SUPERMODEL_APPLIED_PREVIEW",
      selectedSupermodelResref: "c_horror",
      sourceGlb: expect.any(ArrayBuffer),
      referenceChainBlob: expect.any(ArrayBuffer),
      referenceChainJson: expect.any(String),
      experimentalAllowExcessiveSkinBranchRepair: true,
    });
    expect(onAppliedPreview).toHaveBeenCalledWith(expect.objectContaining({
      sourceSha256: "c".repeat(64),
      supermodelResref: "c_horror",
      experimentalAllowExcessiveSkinBranchRepair: true,
      target: expect.objectContaining({
        model: expect.objectContaining({ supermodelName: "c_horror" }),
      }),
    }));
    expect(host.textContent).toContain("DIAGNOSTIC APPLIED PREVIEW");
    expect(host.textContent).toContain("nie jest gotowy do eksportu");
    expect(host.textContent).toContain("27 nodów");
    expect(host.textContent).toContain("new-creature.glb po nałożeniu c_horror");

    expect(host.textContent).toContain("retail 1:1 · tylko odczyt");
    expect(host.textContent).toContain("Dopasowanie dotyczy mesha i wag, nie szkieletu");
    expect([...host.querySelectorAll<HTMLButtonElement>("button")]
      .some((button) => button.textContent?.includes("Zastosuj joint"))).toBe(false);
    const authoredRequest = worker.request.mock.calls
      .map(([value]) => value as { type: string })
      .find((value) => value.type === "BUILD_REFERENCE_SUPERMODEL_AUTHORED_PREVIEW");
    expect(authoredRequest).toBeUndefined();
    await act(async () => root.unmount());
  });
});
