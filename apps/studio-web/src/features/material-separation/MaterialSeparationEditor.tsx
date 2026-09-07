import { useEffect, useMemo, useReducer, useState } from "react";
import { SourceViewport } from "../preview/SourceViewport";
import type { CreatureSourceForwardV1 } from "../source/InputsPanel";
import {
  createMaterialSeparationEditorStateV2,
  materialSeparationReducerV2,
} from "./stateV2";
import type {
  AuthoredMaterialV1,
  ModelFaceInspectionBootstrapV2,
  ModelMaterialResolutionV2,
  ModelMaterialSeparationDocumentV2,
  ModelTextureAuthoringDocumentV1,
} from "./types";
import { componentKey, faceKey } from "./types";
import {
  overrideModelTextureBindingV1,
  reuseModelTextureOverrideV1,
  sourceModelTextureBindingV1,
  type ModelTextureEditorSnapshotV1,
} from "./texturePayloads";
import "./material-separation.css";

interface Props {
  readonly file: File;
  readonly sourceSha256: string;
  readonly bootstrap: ModelFaceInspectionBootstrapV2;
  readonly resolution?: ModelMaterialResolutionV2;
  readonly sourceForward?: CreatureSourceForwardV1;
  readonly onPreviewDocumentChange?: (document: ModelMaterialSeparationDocumentV2) => void;
  readonly onApply: (document: ModelMaterialSeparationDocumentV2) => void;
  readonly onTextureSnapshotChange?: (snapshot: ModelTextureEditorSnapshotV1) => void;
  readonly uvProjectionMaterialIds?: readonly string[];
  readonly onUvProjectionMaterialIdsChange?: (ids: readonly string[]) => void;
  readonly uvProjectionRepeatsPerMetre?: Readonly<Record<string, number>>;
  readonly onUvProjectionRepeatsPerMetreChange?: (
    values: Readonly<Record<string, number>>,
  ) => void;
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
  uvProjectionMaterialIds = [],
  onUvProjectionMaterialIdsChange,
  uvProjectionRepeatsPerMetre = {},
  onUvProjectionRepeatsPerMetreChange,
  onError,
}: Props) {
  const [state, dispatch] = useReducer(
    materialSeparationReducerV2,
    bootstrap.document,
    createMaterialSeparationEditorStateV2,
  );
  const [activeMaterialId, setActiveMaterialId] = useState(
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

  const selectedComponents = new Set(state.selectedComponentKeys);
  const selectedFaces = new Set(state.selectedFaceKeys);
  const selectedCount = state.selectionMode === "FACE" ? selectedFaces.size : selectedComponents.size;
  const assignmentByComponent = new Map(
    state.draft.componentAssignments.map((assignment) => (
      [componentKey(assignment.component), assignment.authoredMaterialId]
    )),
  );
  const materialById = new Map(
    state.draft.materials.map((material) => [material.authoredMaterialId, material]),
  );
  const visibleComponents = state.isolateSelection && selectedComponents.size
    ? bootstrap.inventory.components.filter((component) => (
        selectedComponents.has(componentKey(component.key))
      ))
    : bootstrap.inventory.components;
  const overlays = useMemo(() => state.overlayEnabled && state.selectionMode === "COMPONENT"
    ? visibleComponents.map((component) => {
        const id = componentKey(component.key);
        const material = materialById.get(assignmentByComponent.get(id) ?? "");
        return {
          id,
          boundsMin: component.boundsMin,
          boundsMax: component.boundsMax,
          color: material?.previewColor ?? "#87909b",
          selected: selectedComponents.has(id),
        };
      })
    : [], [state.overlayEnabled, state.selectionMode, state.isolateSelection, state.draft, state.selectedComponentKeys]);
  const faceMaterialOverlay = useMemo(() => (
    state.overlayEnabled && state.selectionMode === "FACE"
      ? {
          identity: JSON.stringify({
            materials: state.draft.materials.map((material) => [
              material.authoredMaterialId,
              material.previewColor,
            ]),
            assignments: state.draft.faceAssignments,
          }),
          assignments: state.draft.faceAssignments,
          materials: state.draft.materials,
        }
      : undefined
  ), [state.draft.faceAssignments, state.draft.materials, state.overlayEnabled, state.selectionMode]);
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

  const preview = (next: ModelMaterialSeparationDocumentV2) => {
    onPreviewDocumentChange?.(next);
  };
  const dispatchWithPreview = (action: Parameters<typeof materialSeparationReducerV2>[1]) => {
    const next = materialSeparationReducerV2(state, action);
    dispatch(action);
    if (next.draft !== state.draft) preview(next.draft);
  };

  return (
    <section className="material-separation" aria-label="Material Separation editor">
      <header className="material-separation__toolbar">
        <strong>Material Separation · Face Mode V2</strong>
        <span>{bootstrap.capabilities.target}</span>
        <button type="button" onClick={() => dispatch({ type: "UNDO" })} disabled={!state.past.length}>Undo</button>
        <button type="button" onClick={() => dispatch({ type: "REDO" })} disabled={!state.future.length}>Redo</button>
        <button type="button" aria-pressed={state.overlayEnabled} onClick={() => dispatch({ type: "TOGGLE_OVERLAY" })}>Material ID Colors</button>
        <button type="button" aria-pressed={state.isolateSelection} onClick={() => dispatch({ type: "TOGGLE_ISOLATE" })}>Isolate</button>
        <button type="button" aria-pressed={state.selectionMode === "FACE"} onClick={() => dispatch({ type: "SET_SELECTION_MODE", mode: "FACE" })}>Face Mode</button>
        <button type="button" aria-pressed={state.selectionMode === "COMPONENT"} onClick={() => dispatch({ type: "SET_SELECTION_MODE", mode: "COMPONENT" })}>Component Mode</button>
      </header>
      <div className="material-separation__layout">
        <aside className="material-separation__panel">
          <div className="material-separation__metrics" aria-label="Material separation metrics">
            <span>Slots <strong>{report?.materialSlots.length ?? state.draft.materials.length}</strong> / {bootstrap.capabilities.maxMaterialSlots}</span>
            <span>Sections <strong>{report?.outputSectionCount ?? "—"}</strong></span>
            <span>Textures <strong>{report?.predictedTextureCount ?? "—"}</strong></span>
            <span>Triangles <strong>{bootstrap.inventory.triangleCount.toLocaleString("en-US")}</strong></span>
            <span>Assigned faces <strong>{report?.assignedFaceCount.toLocaleString("en-US") ?? "0"}</strong></span>
          </div>
          <div className="material-separation__materials" aria-label="Authored materials">
            {state.draft.materials.map((material) => {
              const componentCount = state.draft.componentAssignments
                .filter((assignment) => assignment.authoredMaterialId === material.authoredMaterialId)
                .length;
              const faceCount = state.draft.faceAssignments
                .filter((assignment) => assignment.authoredMaterialId === material.authoredMaterialId)
                .flatMap((assignment) => assignment.selection.triangleRanges)
                .reduce((sum, range) => sum + range.triangleCount, 0);
              const textureBinding = textureDocument?.bindings.find(
                (binding) => binding.authoredMaterialId === material.authoredMaterialId,
              );
              const reusableTextureBindings = textureDocument?.bindings.filter((binding) => (
                binding.mode === "OVERRIDE"
                && binding.overrideAssetId
                && binding.materialSlot !== textureBinding?.materialSlot
              )) ?? [];
              return (
                <div key={material.authoredMaterialId} className="material-separation__material" data-active={activeMaterialId === material.authoredMaterialId}>
                  <button type="button" aria-label={`Select material ${material.displayName}`} onClick={() => setActiveMaterialId(material.authoredMaterialId)}>
                    <span className="material-separation__swatch" style={{ background: material.previewColor }} />
                    {material.displayName} ({componentCount} components / {faceCount} faces)
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
                          bindings: document.bindings.map((binding) => (
                            binding.materialSlot === textureBinding.materialSlot
                              ? sourceModelTextureBindingV1(binding)
                              : binding
                          )),
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
                          const textureFile = event.currentTarget.files?.[0];
                          if (!textureFile || !textureBinding) return;
                          void overrideModelTextureBindingV1(textureBinding, textureFile)
                            .then(({ assetId, binding: replacement }) => {
                              setTextureFiles((files) => new Map(files).set(assetId, textureFile));
                              setTextureDocument((document) => document && ({
                                ...document,
                                bindings: document.bindings.map((binding) => (
                                  binding.materialSlot === replacement.materialSlot
                                    ? replacement
                                    : binding
                                )),
                              }));
                            })
                            .catch((error: unknown) => onError?.(
                              error instanceof Error ? error.message : String(error),
                            ));
                        }}
                      />
                    </label>
                    {textureBinding && reusableTextureBindings.length > 0 && (
                      <label>
                        Reuse uploaded texture
                        <select
                          aria-label={`Reuse texture ${material.displayName}`}
                          value=""
                          onChange={(event) => {
                            const source = textureDocument?.bindings.find(
                              (binding) => binding.overrideAssetId === event.currentTarget.value,
                            );
                            if (!source) return;
                            const replacement = reuseModelTextureOverrideV1(textureBinding, source);
                            setTextureDocument((document) => document && ({
                              ...document,
                              bindings: document.bindings.map((binding) => (
                                binding.materialSlot === replacement.materialSlot
                                  ? replacement
                                  : binding
                              )),
                            }));
                          }}
                        >
                          <option value="">Select…</option>
                          {reusableTextureBindings.map((binding) => (
                            <option key={binding.authoredMaterialId} value={binding.overrideAssetId ?? ""}>
                              {materialById.get(binding.authoredMaterialId)?.displayName
                                ?? binding.authoredMaterialId}
                            </option>
                          ))}
                        </select>
                      </label>
                    )}
                    {onUvProjectionMaterialIdsChange && (
                      <label title="World-scale box projection: one repeatable physical texture scale across fragmented faces of this material.">
                        <input
                          type="checkbox"
                          aria-label={`World-scale UV ${material.displayName}`}
                          checked={uvProjectionMaterialIds.includes(material.authoredMaterialId)}
                          onChange={(event) => {
                            const next = event.currentTarget.checked
                              ? [...new Set([...uvProjectionMaterialIds, material.authoredMaterialId])]
                              : uvProjectionMaterialIds.filter(
                                (id) => id !== material.authoredMaterialId,
                              );
                            onUvProjectionMaterialIdsChange(next);
                          }}
                        />
                        World-scale UV (experimental)
                      </label>
                    )}
                    {uvProjectionMaterialIds.includes(material.authoredMaterialId)
                      && onUvProjectionRepeatsPerMetreChange && (
                      <label>
                        Texture repeats / metre
                        <input
                          type="number"
                          min="0.01"
                          max="128"
                          step="0.05"
                          aria-label={`Texture repeats per metre ${material.displayName}`}
                          value={uvProjectionRepeatsPerMetre[material.authoredMaterialId] ?? 0.5}
                          onChange={(event) => {
                            const value = event.currentTarget.valueAsNumber;
                            if (!Number.isFinite(value) || value < 0.01 || value > 128) return;
                            onUvProjectionRepeatsPerMetreChange({
                              ...uvProjectionRepeatsPerMetre,
                              [material.authoredMaterialId]: value,
                            });
                          }}
                        />
                        <small>{((uvProjectionRepeatsPerMetre[material.authoredMaterialId] ?? 0.5) ** -1).toFixed(2)} m / repeat</small>
                      </label>
                    )}
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
            <button type="button" disabled={!activeMaterialId || !selectedCount} onClick={() => dispatchWithPreview({ type: "ASSIGN_SELECTION", authoredMaterialId: activeMaterialId })}>Assign Selection</button>
            <button type="button" disabled={!selectedCount} onClick={() => dispatchWithPreview({ type: "UNASSIGN_SELECTION" })}>Unassign to Source</button>
            <button type="button" onClick={() => dispatch({ type: "SELECT_UNASSIGNED_COMPONENTS", inventory: bootstrap.inventory })}>Select Unassigned Components</button>
            <button type="button" disabled={!selectedCount} onClick={() => dispatch({ type: "CLEAR_SELECTION" })}>Clear Selection</button>
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
            {bootstrap.inventory.components.slice(0, 500).map((component) => {
              const id = componentKey(component.key);
              const assigned = assignmentByComponent.get(id);
              return (
                <button
                  type="button"
                  key={id}
                  data-selected={selectedComponents.has(id)}
                  onClick={(event) => dispatch({
                    type: "SELECT_COMPONENT",
                    key: id,
                    additive: event.ctrlKey || event.metaKey,
                  })}
                >
                  <span>{id}</span>
                  <small>{component.triangleCount} tri · {materialById.get(assigned ?? "")?.displayName ?? "Source"}</small>
                </button>
              );
            })}
          </div>
          {bootstrap.inventory.components.length > 500 && (
            <p className="material-separation__notice">
              Showing 500 of {bootstrap.inventory.components.length.toLocaleString("en-US")} fragmented components. Use Face Mode for detailed painting.
            </p>
          )}
          <p className="material-separation__notice">
            Face Mode V2: click a face, Ctrl-click to add, Alt-click to grow across spatially welded faces (45°), or Shift-drag a rectangle. Source UV0 remains unchanged unless World-scale UV is enabled explicitly for a material; geometry and collision remain unchanged. Multiple groups may reuse one uploaded texture without duplicating its payload. No material is inferred automatically.
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
            faceMaterialOverlay={faceMaterialOverlay}
            faceSelection={{
              sceneId: bootstrap.inventory.sceneId,
              selectedFaceKeys: state.selectedFaceKeys,
              enabled: state.selectionMode === "FACE",
            }}
            onSelectFaces={(faces, additive) => {
              const keys = faces.map(faceKey);
              if (keys.length === 1) {
                dispatch({ type: "SELECT_FACE", key: keys[0], additive });
              } else {
                dispatch({ type: "SELECT_FACES", keys, additive });
              }
            }}
            onError={onError}
          />
        </div>
      </div>
    </section>
  );
}
