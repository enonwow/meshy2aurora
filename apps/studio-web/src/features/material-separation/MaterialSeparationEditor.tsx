import { useEffect, useMemo, useReducer, useState } from "react";
import { SourceViewport } from "../preview/SourceViewport";
import type { CreatureSourceForwardV1 } from "../source/InputsPanel";
import {
  createMaterialSeparationEditorState,
  materialSeparationReducer,
} from "./state";
import type {
  AuthoredMaterialV1,
  ModelComponentInspectionBootstrapV1,
  ModelMaterialResolutionV1,
  ModelMaterialSeparationDocumentV1,
  ModelTextureAuthoringDocumentV1,
} from "./types";
import { componentKey } from "./types";
import {
  overrideModelTextureBindingV1,
  sourceModelTextureBindingV1,
  type ModelTextureEditorSnapshotV1,
} from "./texturePayloads";
import "./material-separation.css";

interface Props {
  readonly file: File;
  readonly sourceSha256: string;
  readonly bootstrap: ModelComponentInspectionBootstrapV1;
  readonly resolution?: ModelMaterialResolutionV1;
  readonly sourceForward?: CreatureSourceForwardV1;
  readonly onPreviewDocumentChange?: (document: ModelMaterialSeparationDocumentV1) => void;
  readonly onApply: (document: ModelMaterialSeparationDocumentV1) => void;
  readonly onTextureSnapshotChange?: (snapshot: ModelTextureEditorSnapshotV1) => void;
  readonly onError?: (message: string) => void;
}

const colors = ["#7a4f2a", "#c8bea8", "#766f61", "#564638", "#8f6b45", "#5d7182"];

function nextMaterial(
  used: number,
  sourceFallback?: {
    readonly sourceMaterialId: number | null;
    readonly sourceImageSha256: string | null;
  },
): AuthoredMaterialV1 {
  const ordinal = used + 1;
  return {
    authoredMaterialId: `material:custom-${ordinal}`,
    displayName: `Material ${ordinal}`,
    previewColor: colors[used % colors.length],
    sourceFallbackMaterialId: sourceFallback?.sourceMaterialId ?? null,
    sourceFallbackImageSha256: sourceFallback?.sourceImageSha256 ?? null,
  };
}

export function MaterialSeparationEditor({
  file,
  sourceSha256,
  bootstrap,
  resolution,
  sourceForward,
  onPreviewDocumentChange,
  onApply,
  onTextureSnapshotChange,
  onError,
}: Props) {
  const [state, dispatch] = useReducer(
    materialSeparationReducer,
    bootstrap.document,
    createMaterialSeparationEditorState,
  );
  const [activeMaterialId, setActiveMaterialId] = useState<string>(
    bootstrap.document.materials[0]?.authoredMaterialId ?? "",
  );
  const [textureDocument, setTextureDocument] = useState<ModelTextureAuthoringDocumentV1>();
  const [textureFiles, setTextureFiles] = useState<ReadonlyMap<string, File>>(new Map());
  const separationSha256 = resolution?.textureAuthoring.separationSha256;
  useEffect(() => {
    if (!resolution) return;
    setTextureDocument(resolution.textureAuthoring);
    setTextureFiles(new Map());
  }, [separationSha256]);
  useEffect(() => {
    if (textureDocument) onTextureSnapshotChange?.({ document: textureDocument, files: textureFiles });
  }, [onTextureSnapshotChange, textureDocument, textureFiles]);
  const selected = new Set(state.selectedComponentKeys);
  const assignmentByComponent = new Map(
    state.draft.assignments.map((assignment) => [componentKey(assignment.component), assignment.authoredMaterialId]),
  );
  const materialById = new Map(
    state.draft.materials.map((material) => [material.authoredMaterialId, material]),
  );
  const visibleComponents = state.isolateSelection && selected.size
    ? bootstrap.inventory.components.filter((component) => selected.has(componentKey(component.key)))
    : bootstrap.inventory.components;
  const overlays = useMemo(() => state.overlayEnabled
    ? visibleComponents.map((component) => {
      const id = componentKey(component.key);
      const material = materialById.get(assignmentByComponent.get(id) ?? "");
      return {
        id,
        boundsMin: component.boundsMin,
        boundsMax: component.boundsMax,
        color: material?.previewColor ?? "#87909b",
        selected: selected.has(id),
      };
    })
    : [], [state.overlayEnabled, state.isolateSelection, state.draft, state.selectedComponentKeys]);
  const report = resolution?.report;
  const sourceFallbackForNewMaterial = report?.materialSlots.find((slot) => (
    slot.sourceMaterialId !== null && slot.sourceImageSha256 !== null
  ));
  const materialsRequiringOverride = state.draft.materials.filter((material) => {
    if (material.sourceFallbackMaterialId !== null && material.sourceFallbackImageSha256 !== null) {
      return false;
    }
    return textureDocument?.bindings.find(
      (binding) => binding.authoredMaterialId === material.authoredMaterialId,
    )?.mode !== "OVERRIDE";
  });
  const nodeIds = [...new Set(bootstrap.inventory.components.map((component) => component.key.nodeId))];
  const primitiveKeys = [...new Set(bootstrap.inventory.components.map(
    (component) => `${component.key.nodeId}/${component.key.primitiveId}`,
  ))];

  const preview = (next: ModelMaterialSeparationDocumentV1) => {
    onPreviewDocumentChange?.(next);
  };
  const dispatchWithPreview = (action: Parameters<typeof materialSeparationReducer>[1]) => {
    const next = materialSeparationReducer(state, action);
    dispatch(action);
    if (next.draft !== state.draft) preview(next.draft);
  };

  return (
    <section className="material-separation" aria-label="Material Separation editor">
      <header className="material-separation__toolbar">
        <strong>Material Separation</strong>
        <span>{bootstrap.capabilities.target}</span>
        <button type="button" onClick={() => dispatch({ type: "UNDO" })} disabled={!state.past.length}>Undo</button>
        <button type="button" onClick={() => dispatch({ type: "REDO" })} disabled={!state.future.length}>Redo</button>
        <button type="button" aria-pressed={state.overlayEnabled} onClick={() => dispatch({ type: "TOGGLE_OVERLAY" })}>Material ID Colors</button>
        <button type="button" aria-pressed={state.isolateSelection} onClick={() => dispatch({ type: "TOGGLE_ISOLATE" })}>Isolate</button>
      </header>
      <div className="material-separation__layout">
        <aside className="material-separation__panel">
          <div className="material-separation__metrics" aria-label="Material separation metrics">
            <span>Slots <strong>{report?.materialSlots.length ?? state.draft.materials.length}</strong> / {bootstrap.capabilities.maxMaterialSlots}</span>
            <span>Sections <strong>{report?.outputSectionCount ?? "—"}</strong></span>
            <span>Textures <strong>{report?.predictedTextureCount ?? "—"}</strong></span>
            <span>Triangles <strong>{bootstrap.inventory.triangleCount.toLocaleString("en-US")}</strong></span>
          </div>
          <div className="material-separation__materials" aria-label="Authored materials">
            {state.draft.materials.map((material) => {
              const assignmentCount = state.draft.assignments.filter((assignment) => assignment.authoredMaterialId === material.authoredMaterialId).length;
              const textureBinding = textureDocument?.bindings.find(
                (binding) => binding.authoredMaterialId === material.authoredMaterialId,
              );
              return (
                <div key={material.authoredMaterialId} className="material-separation__material" data-active={activeMaterialId === material.authoredMaterialId}>
                  <button type="button" aria-label={`Select material ${material.displayName}`} onClick={() => setActiveMaterialId(material.authoredMaterialId)}>
                    <span className="material-separation__swatch" style={{ background: material.previewColor }} />
                    {material.displayName} ({assignmentCount})
                  </button>
                  <button type="button" onClick={() => dispatch({ type: "SELECT_BY_MATERIAL", authoredMaterialId: material.authoredMaterialId })}>Select</button>
                  <label>
                    Name
                    <input
                      value={material.displayName}
                      onChange={(event) => dispatchWithPreview({
                        type: "RENAME_MATERIAL",
                        authoredMaterialId: material.authoredMaterialId,
                        displayName: event.currentTarget.value,
                      })}
                    />
                  </label>
                  <div className="material-separation__texture">
                    <span>Texture: {textureBinding?.mode ?? "resolving"}</span>
                    <button
                      type="button"
                      disabled={!textureBinding || textureBinding.mode === "SOURCE"}
                      onClick={() => {
                        if (!textureBinding) return;
                        setTextureDocument((document) => document && ({
                          ...document,
                          bindings: document.bindings.map((binding) => binding.materialSlot === textureBinding.materialSlot
                            ? sourceModelTextureBindingV1(binding)
                            : binding),
                        }));
                      }}
                    >Use Source</button>
                    <label>
                      Override PNG/JPEG
                      <input
                        type="file"
                        accept="image/png,image/jpeg,.png,.jpg,.jpeg"
                        disabled={!textureBinding}
                        onChange={(event) => {
                          const file = event.currentTarget.files?.[0];
                          if (!file || !textureBinding) return;
                          void overrideModelTextureBindingV1(textureBinding, file)
                            .then(({ assetId, binding: replacement }) => {
                              setTextureFiles((files) => new Map(files).set(assetId, file));
                              setTextureDocument((document) => document && ({
                                ...document,
                                bindings: document.bindings.map((binding) => binding.materialSlot === replacement.materialSlot
                                  ? replacement
                                  : binding),
                              }));
                            })
                            .catch((error: unknown) => onError?.(
                              error instanceof Error ? error.message : String(error),
                            ));
                        }}
                      />
                    </label>
                  </div>
                </div>
              );
            })}
          </div>
          <div className="material-separation__actions">
            <button type="button" onClick={() => {
              const material = nextMaterial(state.draft.materials.length, sourceFallbackForNewMaterial);
              setActiveMaterialId(material.authoredMaterialId);
              dispatchWithPreview({ type: "NEW_MATERIAL", material });
            }}>New Material</button>
            <button type="button" onClick={() => dispatchWithPreview({ type: "DELETE_UNUSED" })}>Delete Unused</button>
            <button type="button" disabled={!activeMaterialId || !selected.size} onClick={() => dispatchWithPreview({ type: "ASSIGN_SELECTION", authoredMaterialId: activeMaterialId })}>Assign Selection</button>
            <button type="button" disabled={!selected.size} onClick={() => dispatchWithPreview({ type: "UNASSIGN_SELECTION" })}>Unassign to Source</button>
            <button type="button" onClick={() => dispatch({ type: "SELECT_UNASSIGNED", inventory: bootstrap.inventory })}>Select Unassigned</button>
          </div>
          <div className="material-separation__groups" aria-label="Node and primitive selection">
            {nodeIds.map((nodeId) => (
              <button key={`node:${nodeId}`} type="button" onClick={() => dispatch({
                type: "SELECT_NODE",
                nodeId,
                inventory: bootstrap.inventory,
              })}>Select node {nodeId}</button>
            ))}
            {primitiveKeys.map((key) => {
              const [nodeId, primitiveId] = key.split("/").map(Number);
              return (
                <button key={`primitive:${key}`} type="button" onClick={() => dispatch({
                  type: "SELECT_PRIMITIVE",
                  nodeId,
                  primitiveId,
                  inventory: bootstrap.inventory,
                })}>Select primitive {nodeId}/{primitiveId}</button>
              );
            })}
          </div>
          <div className="material-separation__components" aria-label="Connected components">
            {bootstrap.inventory.components.map((component) => {
              const id = componentKey(component.key);
              const assigned = assignmentByComponent.get(id);
              return (
                <button
                  type="button"
                  key={id}
                  data-selected={selected.has(id)}
                  onClick={(event) => dispatch({ type: "SELECT_COMPONENT", key: id, additive: event.ctrlKey || event.metaKey })}
                >
                  <span>{id}</span>
                  <small>{component.triangleCount} tri · {materialById.get(assigned ?? "")?.displayName ?? "Source"}</small>
                </button>
              );
            })}
          </div>
          <p className="material-separation__notice">
            V1 assigns connected components only. UV0 is preserved and no material is inferred automatically.
          </p>
          {materialsRequiringOverride.length > 0 && (
            <p className="material-separation__notice" role="alert">
              MATERIAL-SEPARATION-TEXTURE-OVERRIDE-REQUIRED: {materialsRequiringOverride.length} material(s) have no exact source texture fallback.
            </p>
          )}
          {report?.warnings.map((warning) => <p key={warning} role="status">{warning}</p>)}
          <footer className="material-separation__footer">
            <button type="button" onClick={() => {
              onApply(state.draft);
              dispatch({ type: "APPLY" });
            }} disabled={materialsRequiringOverride.length > 0}>Apply</button>
            <button type="button" onClick={() => dispatchWithPreview({ type: "CANCEL" })}>Cancel</button>
            <button type="button" onClick={() => dispatchWithPreview({ type: "RESET" })}>Reset</button>
          </footer>
        </aside>
        <div className="material-separation__viewport">
          <SourceViewport
            input={{ provenance: "SOURCE", file, sourceSha256 }}
            sourceForward={sourceForward}
            componentOverlays={overlays}
            onError={onError}
          />
        </div>
      </div>
    </section>
  );
}
