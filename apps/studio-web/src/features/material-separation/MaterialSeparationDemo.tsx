import { useEffect, useMemo, useState } from "react";
import { SourceViewport } from "../preview/SourceViewport";
import { MaterialSeparationEditor } from "./MaterialSeparationEditor";
import type {
  ModelFaceInspectionBootstrapV2,
  ModelMaterialResolutionV2,
  ModelMaterialSeparationDocumentV2,
  ModelTextureAuthoringDocumentV1,
} from "./types";
import "./material-separation-demo.css";

interface DemoState {
  readonly file: File;
  readonly document: ModelMaterialSeparationDocumentV2;
  readonly textureAuthoring?: ModelTextureAuthoringDocumentV1;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseDocument(json: string): ModelMaterialSeparationDocumentV2 {
  const value = JSON.parse(json) as unknown;
  if (!isObject(value)
    || value.schemaVersion !== 2
    || typeof value.sourceSha256 !== "string"
    || !Array.isArray(value.materials)
    || !Array.isArray(value.componentAssignments)
    || !Array.isArray(value.faceAssignments)) {
    throw new Error("MATERIAL-SEPARATION-DEMO-RECIPE-INVALID");
  }
  return value as unknown as ModelMaterialSeparationDocumentV2;
}

function parseTextureAuthoring(json: string): ModelTextureAuthoringDocumentV1 {
  const value = JSON.parse(json) as unknown;
  if (!isObject(value)
    || value.schemaVersion !== 1
    || typeof value.sourceSha256 !== "string"
    || typeof value.separationSha256 !== "string"
    || !Array.isArray(value.bindings)) {
    throw new Error("MATERIAL-SEPARATION-DEMO-TEXTURE-AUTHORING-INVALID");
  }
  return value as unknown as ModelTextureAuthoringDocumentV1;
}

function editorData(state: DemoState): {
  readonly bootstrap: ModelFaceInspectionBootstrapV2;
  readonly resolution?: ModelMaterialResolutionV2;
} {
  const { document, textureAuthoring } = state;
  const capabilities = {
    schemaVersion: 2 as const,
    target: "PLACEABLE" as const,
    materialSeparationSupported: true,
    maxMaterialSlots: 256,
    maxOutputSections: 4_096,
    selectionGranularity: "CONNECTED_COMPONENTS_AND_FACES" as const,
    faceSelectionSupported: true as const,
    automaticMaterialInference: false as const,
    preservesSourceUv0: true as const,
  };
  const assignedFaceCount = document.faceAssignments.reduce((sum, assignment) => (
    sum + assignment.selection.triangleRanges.reduce(
      (rangeSum, range) => rangeSum + range.triangleCount,
      0,
    )
  ), 0);
  const sceneId = document.faceAssignments[0]?.selection.sceneId ?? 0;
  const nodes = new Set(document.faceAssignments.map((assignment) => assignment.selection.nodeId));
  const primitives = new Set(document.faceAssignments.map((assignment) => (
    `${assignment.selection.nodeId}/${assignment.selection.primitiveId}`
  )));
  const bootstrap: ModelFaceInspectionBootstrapV2 = {
    schemaVersion: 2,
    capabilities,
    inventory: {
      schemaVersion: 1,
      sourceSha256: document.sourceSha256,
      sceneId,
      renderNodeCount: nodes.size,
      primitiveInstanceCount: primitives.size,
      triangleCount: assignedFaceCount,
      components: [],
    },
    document,
  };
  if (!textureAuthoring) return { bootstrap };
  const faceCountByMaterial = new Map<string, number>();
  for (const assignment of document.faceAssignments) {
    const count = assignment.selection.triangleRanges.reduce(
      (sum, range) => sum + range.triangleCount,
      0,
    );
    faceCountByMaterial.set(
      assignment.authoredMaterialId,
      (faceCountByMaterial.get(assignment.authoredMaterialId) ?? 0) + count,
    );
  }
  const predictedTextures = new Set(textureAuthoring.bindings.map((binding) => (
    binding.overrideAssetId
      ?? `${binding.sourceMaterialId ?? "none"}:${binding.sourceImageSha256 ?? "none"}`
  ))).size;
  return {
    bootstrap,
    resolution: {
      schemaVersion: 2,
      capabilities,
      report: {
        schemaVersion: 2,
        sourceSha256: document.sourceSha256,
        separationSha256: textureAuthoring.separationSha256,
        sourceComponentCount: 0,
        assignedComponentCount: 0,
        unassignedComponentCount: 0,
        faceAssignmentCount: document.faceAssignments.length,
        triangleRangeCount: document.faceAssignments.reduce(
          (sum, assignment) => sum + assignment.selection.triangleRanges.length,
          0,
        ),
        assignedFaceCount,
        unassignedFaceCount: 0,
        sourceTriangleCount: assignedFaceCount,
        outputTriangleCount: assignedFaceCount,
        sourceVertexCount: 0,
        outputVertexCount: 0,
        duplicatedBoundaryVertexCount: 0,
        outputSectionCount: document.materials.length,
        predictedTextureCount: predictedTextures,
        warnings: [],
        materialSlots: document.materials.map((material, materialSlot) => ({
          materialSlot,
          authoredMaterialId: material.authoredMaterialId,
          displayName: material.displayName,
          previewColor: material.previewColor,
          sourceMaterialId: material.sourceFallbackMaterialId,
          sourceMaterialName: null,
          sourceImageSha256: material.sourceFallbackImageSha256,
          systemSourceMaterial: false,
          componentCount: 0,
          triangleCount: faceCountByMaterial.get(material.authoredMaterialId) ?? 0,
        })),
      },
      textureAuthoring,
    },
  };
}

async function sha256(bytes: ArrayBuffer) {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((value) => value.toString(16).padStart(2, "0"))
    .join("");
}

function fileName(url: string) {
  const path = new URL(url, window.location.href).pathname;
  return decodeURIComponent(path.split("/").at(-1) || "source.glb");
}

export function MaterialSeparationDemo() {
  const parameters = useMemo(() => new URLSearchParams(window.location.search), []);
  const sourceUrl = parameters.get("source") ?? "";
  const recipeUrl = parameters.get("recipe") ?? "";
  const textureAuthoringUrl = parameters.get("textureAuthoring") ?? "";
  const [screen, setScreen] = useState<"OVERVIEW" | "EDITOR">(
    parameters.get("mode") === "edit" ? "EDITOR" : "OVERVIEW",
  );
  const [state, setState] = useState<DemoState>();
  const [error, setError] = useState<string>();
  const [showOverlay, setShowOverlay] = useState(true);
  const [opacity, setOpacity] = useState(0.94);
  const [uvProjectionMaterialIds, setUvProjectionMaterialIds] = useState<readonly string[]>([]);
  const [uvProjectionRepeatsPerMetre, setUvProjectionRepeatsPerMetre] = useState<Readonly<Record<string, number>>>({});
  const [applyMessage, setApplyMessage] = useState<string>();

  useEffect(() => {
    let canceled = false;
    void Promise.all([
      fetch(sourceUrl).then(async (response) => {
        if (!response.ok) throw new Error(`Source GLB request failed (${response.status})`);
        return response.arrayBuffer();
      }),
      fetch(recipeUrl).then(async (response) => {
        if (!response.ok) throw new Error(`Recipe request failed (${response.status})`);
        return response.text();
      }),
      textureAuthoringUrl
        ? fetch(textureAuthoringUrl).then(async (response) => {
            if (!response.ok) throw new Error(`Texture authoring request failed (${response.status})`);
            return response.text();
          })
        : Promise.resolve(undefined),
    ]).then(async ([bytes, json, textureJson]) => {
      const document = parseDocument(json);
      const textureAuthoring = textureJson ? parseTextureAuthoring(textureJson) : undefined;
      const actualSha256 = await sha256(bytes);
      if (actualSha256 !== document.sourceSha256) {
        throw new Error("MATERIAL-SEPARATION-DEMO-SOURCE-SHA256-MISMATCH");
      }
      if (!canceled) {
        setState({
          file: new File([bytes], fileName(sourceUrl), { type: "model/gltf-binary" }),
          document,
          textureAuthoring,
        });
      }
    }).catch((reason: unknown) => {
      if (!canceled) setError(reason instanceof Error ? reason.message : String(reason));
    });
    return () => { canceled = true; };
  }, [recipeUrl, sourceUrl, textureAuthoringUrl]);

  const faceCountByMaterial = useMemo(() => {
    const counts = new Map<string, number>();
    for (const assignment of state?.document.faceAssignments ?? []) {
      const count = assignment.selection.triangleRanges.reduce(
        (sum, range) => sum + range.triangleCount,
        0,
      );
      counts.set(
        assignment.authoredMaterialId,
        (counts.get(assignment.authoredMaterialId) ?? 0) + count,
      );
    }
    return counts;
  }, [state?.document.faceAssignments]);

  const assignedFaceCount = [...faceCountByMaterial.values()]
    .reduce((sum, count) => sum + count, 0);
  const materialEditorData = useMemo(() => state ? editorData(state) : undefined, [state]);

  return (
    <main className="material-demo">
      <header className="material-demo__header">
        <div>
          <span className="material-demo__brand">meshy2aurora · Studio</span>
          <h1>Detailed Material Separation</h1>
          <p>The Last City · ship under construction · exact Face Mode V2 recipe</p>
        </div>
        <dl className="material-demo__summary">
          <div><dt>Materials</dt><dd>{state?.document.materials.length ?? "—"}</dd></div>
          <div><dt>Assigned triangles</dt><dd>{state ? assignedFaceCount.toLocaleString("en-US") : "—"}</dd></div>
          <div><dt>Source SHA-256</dt><dd>{state ? `${state.document.sourceSha256.slice(0, 12)}…` : "—"}</dd></div>
        </dl>
      </header>

      <nav className="material-demo__mode" aria-label="Material demo mode">
        <button
          type="button"
          aria-pressed={screen === "OVERVIEW"}
          onClick={() => setScreen("OVERVIEW")}
        >1. Inspect material split</button>
        <button
          type="button"
          aria-pressed={screen === "EDITOR"}
          onClick={() => setScreen("EDITOR")}
        >2. Edit materials</button>
        <span>
          {screen === "OVERVIEW"
            ? "Diagnostic view: shows which triangles belong to each material."
            : "Editing view: change names, texture bindings, UV projection and face assignments."}
        </span>
      </nav>

      {error ? <p className="material-demo__error" role="alert">{error}</p> : null}
      {!state && !error ? <p className="material-demo__loading">Loading exact GLB and material recipe…</p> : null}

      {state && screen === "OVERVIEW" ? (
        <div className="material-demo__workspace">
          <aside className="material-demo__legend" aria-label="Detailed material groups">
            <div className="material-demo__controls">
              <label>
                <input
                  type="checkbox"
                  checked={showOverlay}
                  onChange={(event) => setShowOverlay(event.currentTarget.checked)}
                />
                Material ID Colors
              </label>
              <label>
                Overlay opacity
                <input
                  type="range"
                  min="0.35"
                  max="1"
                  step="0.01"
                  value={opacity}
                  onChange={(event) => setOpacity(event.currentTarget.valueAsNumber)}
                />
              </label>
            </div>
            <h2>12 authored groups</h2>
            <p>Every color is drawn from the actual assigned triangle ranges, not a bounding box.</p>
            <ol>
              {state.document.materials.map((material) => (
                <li key={material.authoredMaterialId}>
                  <span
                    className="material-demo__swatch"
                    style={{ backgroundColor: material.previewColor }}
                  />
                  <span>
                    <strong>{material.displayName}</strong>
                    <small>{material.authoredMaterialId} · {(faceCountByMaterial.get(material.authoredMaterialId) ?? 0).toLocaleString("en-US")} triangles</small>
                  </span>
                </li>
              ))}
            </ol>
          </aside>
          <section className="material-demo__viewport" aria-label="Material-separated ship viewport">
            <SourceViewport
              input={{
                provenance: "SOURCE",
                file: state.file,
                sourceSha256: state.document.sourceSha256,
              }}
              faceMaterialOverlay={showOverlay ? {
                identity: `${recipeUrl}:${opacity}`,
                assignments: state.document.faceAssignments,
                materials: state.document.materials,
                opacity,
              } : undefined}
              onError={setError}
            />
          </section>
        </div>
      ) : null}

      {state && screen === "EDITOR" && materialEditorData ? (
        <section className="material-demo__editor" aria-label="Material editing workspace">
          <div className="material-demo__editor-help">
            <strong>Material editing workspace</strong>
            <span>Select a material on the left. Face Mode edits exact triangles; Component Mode edits whole connected components. Texture controls affect the selected authored material.</span>
            {state.textureAuthoring
              ? <span>Loaded: 12 material bindings reusing {new Set(state.textureAuthoring.bindings.map((binding) => binding.overrideAssetId)).size} texture assets.</span>
              : <span>Texture bindings were not supplied for this demo URL.</span>}
            {applyMessage ? <span role="status">{applyMessage}</span> : null}
          </div>
          <MaterialSeparationEditor
            key={`${state.document.sourceSha256}:${state.textureAuthoring?.separationSha256 ?? "no-textures"}`}
            file={state.file}
            sourceSha256={state.document.sourceSha256}
            bootstrap={materialEditorData.bootstrap}
            resolution={materialEditorData.resolution}
            onApply={() => setApplyMessage("Applied to this local editing session. Build/export remains a separate pipeline step.")}
            uvProjectionMaterialIds={uvProjectionMaterialIds}
            onUvProjectionMaterialIdsChange={setUvProjectionMaterialIds}
            uvProjectionRepeatsPerMetre={uvProjectionRepeatsPerMetre}
            onUvProjectionRepeatsPerMetreChange={setUvProjectionRepeatsPerMetre}
            onError={setError}
          />
        </section>
      ) : null}
    </main>
  );
}
