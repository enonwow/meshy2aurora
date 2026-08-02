import { useCallback, useEffect, useMemo, useReducer, useRef, useState } from "react";
import * as THREE from "three";
import {
  diagnosePlaceableAuthoring,
  groundBoundsById,
  measurePlaceableElements,
} from "./diagnostics";
import { PlaceableAuthoringViewport } from "./PlaceableAuthoringViewport";
import { PlaceableTextureEditor } from "./PlaceableTextureEditor";
import {
  createPlaceableAuthoringEditorState,
  placeableAuthoringReducer,
} from "./state";
import type {
  PlaceableAuthoringBootstrap,
  PlaceableAuthoringDocument,
  PlaceableAuthoringElement,
  PlaceableElementTransform,
  ResolvedPlaceableCollision,
  SnapMode,
  TransformTool,
} from "./types";
import type {
  PlaceableTextureAuthoringBootstrap,
  PlaceableTextureEditorSnapshot,
  ResolvedPlaceableTextures,
} from "./textureTypes";
import "./placeable-authoring.css";

interface Props {
  readonly file: File;
  readonly sourceSha256: string;
  readonly bootstrap: PlaceableAuthoringBootstrap;
  readonly resolvedCollision?: ResolvedPlaceableCollision;
  readonly textureBootstrap?: PlaceableTextureAuthoringBootstrap;
  readonly resolvedTextures?: ResolvedPlaceableTextures;
  readonly onDocumentChange: (document: PlaceableAuthoringDocument) => void;
  readonly onTextureSnapshotChange?: (snapshot: PlaceableTextureEditorSnapshot) => void;
  readonly onError?: (message: string) => void;
}

const axisNames = ["X", "Y", "Z"] as const;
const OUTLINER_ROW_HEIGHT = 32;
const OUTLINER_VISIBLE_ROWS = 20;
const OUTLINER_OVERSCAN_ROWS = 6;
export const PLACEABLE_SPLIT_CONFIRM_COMPONENTS = 512;
const toolShortcuts: readonly { tool: TransformTool; key: string; label: string }[] = [
  { tool: "TRANSLATE", key: "W", label: "Move" },
  { tool: "ROTATE", key: "E", label: "Rotate" },
  { tool: "SCALE", key: "R", label: "Scale" },
];

function finite(value: string, fallback: number) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function live(element: PlaceableAuthoringElement) {
  return !element.deleted;
}

function sourceCenter(
  element: PlaceableAuthoringElement,
  bootstrap: PlaceableAuthoringBootstrap,
): [number, number, number] {
  if (!element.source) return [0, 0, 0];
  const node = bootstrap.inspection.nodes.find((candidate) => candidate.nodeId === element.source?.nodeId);
  const components = node?.primitives.flatMap((primitive) => {
    if (element.source?.primitiveId !== null && primitive.primitiveId !== element.source?.primitiveId) return [];
    return primitive.components.filter((component) => (
      element.source?.componentIndex === null
      || component.componentIndex === element.source?.componentIndex
    ));
  }) ?? [];
  if (!components.length) return [0, 0, 0];
  const min = components.reduce(
    (result, component) => result.map((value, axis) => Math.min(value, component.boundsMin[axis])) as [number, number, number],
    [Infinity, Infinity, Infinity] as [number, number, number],
  );
  const max = components.reduce(
    (result, component) => result.map((value, axis) => Math.max(value, component.boundsMax[axis])) as [number, number, number],
    [-Infinity, -Infinity, -Infinity] as [number, number, number],
  );
  return min.map((value, axis) => (value + max[axis]) / 2) as [number, number, number];
}

function transformEuler(transform: PlaceableElementTransform) {
  const euler = new THREE.Euler().setFromQuaternion(
    new THREE.Quaternion(...transform.rotationXyzw),
    "XYZ",
  );
  return [
    THREE.MathUtils.radToDeg(euler.x),
    THREE.MathUtils.radToDeg(euler.y),
    THREE.MathUtils.radToDeg(euler.z),
  ] as [number, number, number];
}

function withEuler(
  transform: PlaceableElementTransform,
  axis: number,
  degrees: number,
): PlaceableElementTransform {
  const values = transformEuler(transform);
  values[axis] = degrees;
  const quaternion = new THREE.Quaternion().setFromEuler(new THREE.Euler(
    THREE.MathUtils.degToRad(values[0]),
    THREE.MathUtils.degToRad(values[1]),
    THREE.MathUtils.degToRad(values[2]),
    "XYZ",
  ));
  return {
    ...transform,
    rotationXyzw: quaternion.toArray() as [number, number, number, number],
  };
}

function ToolbarButton({
  label,
  title,
  active = false,
  disabled = false,
  onClick,
}: {
  readonly label: string;
  readonly title: string;
  readonly active?: boolean;
  readonly disabled?: boolean;
  readonly onClick: () => void;
}) {
  return (
    <button
      type="button"
      className="placeable-tool"
      data-active={active}
      title={title}
      aria-label={title}
      disabled={disabled}
      onClick={onClick}
    >
      {label}
    </button>
  );
}

export function PlaceableAuthoringEditor({
  file,
  sourceSha256,
  bootstrap,
  resolvedCollision,
  textureBootstrap,
  resolvedTextures,
  onDocumentChange,
  onTextureSnapshotChange,
  onError,
}: Props) {
  const [state, dispatch] = useReducer(
    placeableAuthoringReducer,
    bootstrap.document,
    createPlaceableAuthoringEditorState,
  );
  const [alignMode, setAlignMode] = useState<"MIN" | "CENTER" | "MAX">("CENTER");
  const [showCollision, setShowCollision] = useState(true);
  const [collisionDrawing, setCollisionDrawing] = useState(false);
  const [cameraMode, setCameraMode] = useState<"PERSPECTIVE" | "TOP">("PERSPECTIVE");
  const [outlinerScrollTop, setOutlinerScrollTop] = useState(0);
  const [textureSnapshot, setTextureSnapshot] = useState<PlaceableTextureEditorSnapshot>();
  const sectionRef = useRef<HTMLElement>(null);
  const lastPublishedRef = useRef(JSON.stringify(bootstrap.document));
  const publishTextureSnapshot = useCallback((snapshot: PlaceableTextureEditorSnapshot) => {
    setTextureSnapshot(snapshot);
    onTextureSnapshotChange?.(snapshot);
  }, [onTextureSnapshotChange]);

  useEffect(() => {
    const serialized = JSON.stringify(state.present);
    if (serialized === lastPublishedRef.current) return;
    lastPublishedRef.current = serialized;
    onDocumentChange(state.present);
  }, [onDocumentChange, state.present]);

  const elements = state.present.elements.filter(live);
  const selected = elements.filter((element) => state.selectedIds.includes(element.id));
  const primary = elements.find((element) => element.id === state.primaryId);
  const outlinerStart = Math.max(
    0,
    Math.floor(outlinerScrollTop / OUTLINER_ROW_HEIGHT) - OUTLINER_OVERSCAN_ROWS,
  );
  const outlinerEnd = Math.min(
    elements.length,
    outlinerStart + OUTLINER_VISIBLE_ROWS + OUTLINER_OVERSCAN_ROWS * 2,
  );
  const outlinerElements = elements.slice(outlinerStart, outlinerEnd);
  const measurements = useMemo(
    () => measurePlaceableElements(state.present, bootstrap.inspection),
    [bootstrap.inspection, state.present],
  );
  const measurementById = new Map(measurements.map((measurement) => [measurement.id, measurement]));
  const diagnostics = useMemo(
    () => diagnosePlaceableAuthoring(state.present, bootstrap.inspection),
    [bootstrap.inspection, state.present],
  );
  const collisionVertices = state.present.collision.mode === "CUSTOM_POLYGON"
    ? state.present.collision.vertices
    : resolvedCollision?.sourceVertices ?? [];
  const activateCustomCollision = () => dispatch({
    type: "SET_COLLISION_MODE",
    mode: "CUSTOM_POLYGON",
    seedVertices: state.present.collision.vertices.length
      ? state.present.collision.vertices
      : resolvedCollision?.sourceVertices,
  });

  const updatePrimaryTransform = (transform: PlaceableElementTransform) => {
    if (!primary || primary.flags.locked) return;
    dispatch({ type: "SET_TRANSFORMS", patches: [{ id: primary.id, transform }] });
  };
  const updateVector = (
    key: "translation" | "scale" | "pivot",
    axis: number,
    raw: string,
  ) => {
    if (!primary) return;
    const vector = [...primary.transform[key]] as [number, number, number];
    vector[axis] = finite(raw, vector[axis]);
    updatePrimaryTransform({ ...primary.transform, [key]: vector });
  };

  const splitComponentCount = (element: PlaceableAuthoringElement) => {
    if (element.kind !== "SOURCE_NODE" || !element.source) return 0;
    const node = bootstrap.inspection.nodes.find((candidate) => candidate.nodeId === element.source?.nodeId);
    return node?.primitives.reduce((sum, primitive) => sum + primitive.components.length, 0) ?? 0;
  };
  const splitAvailable = (element: PlaceableAuthoringElement) => {
    const count = splitComponentCount(element);
    return count > 1;
  };
  const splitNode = (element: PlaceableAuthoringElement) => {
    const count = splitComponentCount(element);
    if (count <= 1) return;
    if (
      count > PLACEABLE_SPLIT_CONFIRM_COMPONENTS
      && !window.confirm(`Split ${count.toLocaleString("en-US")} connected components? The editor will keep the Outliner virtualized.`)
    ) return;
    dispatch({
      type: "SPLIT_NODE_COMPONENTS",
      id: element.id,
      inspection: bootstrap.inspection,
    });
  };

  const selectDiagnostic = (ids: readonly string[]) => {
    if (!ids.length) return;
    dispatch({ type: "CLEAR_SELECTION" });
    ids.forEach((id, index) => dispatch({ type: "SELECT", id, additive: index > 0 }));
  };

  const onKeyDown = (event: React.KeyboardEvent<HTMLElement>) => {
    const target = event.target as HTMLElement;
    const key = event.key.toLowerCase();
    if (key === "escape" && collisionDrawing) {
      event.preventDefault();
      setCollisionDrawing(false);
      return;
    }
    if (["INPUT", "SELECT", "TEXTAREA"].includes(target.tagName)) return;
    if (key === "enter" && collisionDrawing && state.present.collision.vertices.length >= 3) {
      event.preventDefault();
      setCollisionDrawing(false);
      return;
    }
    if ((event.ctrlKey || event.metaKey) && key === "z") {
      event.preventDefault();
      dispatch({ type: event.shiftKey ? "REDO" : "UNDO" });
    } else if ((event.ctrlKey || event.metaKey) && key === "y") {
      event.preventDefault();
      dispatch({ type: "REDO" });
    } else if ((event.ctrlKey || event.metaKey) && key === "d") {
      event.preventDefault();
      dispatch({ type: "DUPLICATE" });
    } else if (key === "delete" || key === "backspace") {
      event.preventDefault();
      dispatch({ type: "DELETE" });
    } else if (key === "w") dispatch({ type: "SET_TOOL", tool: "TRANSLATE" });
    else if (key === "e") dispatch({ type: "SET_TOOL", tool: "ROTATE" });
    else if (key === "r") dispatch({ type: "SET_TOOL", tool: "SCALE" });
    else if (key === "f") dispatch({ type: "ISOLATE_SELECTION" });
  };

  return (
    <section
      ref={sectionRef}
      className="placeable-authoring"
      aria-label="Placeable element editor"
      tabIndex={0}
      onKeyDown={onKeyDown}
    >
      <header className="placeable-authoring__toolbar">
        <div className="placeable-authoring__tool-group" aria-label="Transform tools">
          {toolShortcuts.map(({ tool, key, label }) => (
            <ToolbarButton
              key={tool}
              label={key}
              title={`${label} (${key})`}
              active={state.tool === tool}
              onClick={() => dispatch({ type: "SET_TOOL", tool })}
            />
          ))}
        </div>
        <div className="placeable-authoring__tool-group" aria-label="Transform space">
          {(["WORLD", "LOCAL"] as const).map((space) => (
            <ToolbarButton
              key={space}
              label={space === "WORLD" ? "World" : "Local"}
              title={`${space === "WORLD" ? "World" : "Local"} transform orientation`}
              active={state.space === space}
              onClick={() => dispatch({ type: "SET_SPACE", space })}
            />
          ))}
        </div>
        <label className="placeable-authoring__compact-field">
          Snap
          <select
            aria-label="Snap mode"
            value={state.snap.mode}
            onChange={(event) => dispatch({
              type: "SET_SNAP",
              snap: { mode: event.target.value as SnapMode },
            })}
          >
            <option value="NONE">Off</option>
            <option value="GRID">Grid</option>
            <option value="SURFACE">Surface</option>
            <option value="VERTEX">Vertex</option>
            <option value="ELEMENT">Element</option>
          </select>
        </label>
        <label className="placeable-authoring__compact-field">
          Step
          <input
            aria-label="Translation snap step"
            type="number"
            min="0.001"
            step="0.01"
            value={state.snap.translationStep}
            onChange={(event) => dispatch({
              type: "SET_SNAP",
              snap: { translationStep: Math.max(finite(event.target.value, 0.1), 0.001) },
            })}
          />
        </label>
        <label className="placeable-authoring__compact-field">
          Rot °
          <input
            aria-label="Rotation snap step"
            type="number"
            min="0.1"
            step="1"
            value={state.snap.rotationDegrees}
            onChange={(event) => dispatch({
              type: "SET_SNAP",
              snap: { rotationDegrees: Math.max(finite(event.target.value, 15), 0.1) },
            })}
          />
        </label>
        <label className="placeable-authoring__compact-field">
          Scale
          <input
            aria-label="Scale snap step"
            type="number"
            min="0.001"
            step="0.01"
            value={state.snap.scaleStep}
            onChange={(event) => dispatch({
              type: "SET_SNAP",
              snap: { scaleStep: Math.max(finite(event.target.value, 0.1), 0.001) },
            })}
          />
        </label>
        <div className="placeable-authoring__tool-group" aria-label="History">
          <ToolbarButton label="↶" title="Undo (Ctrl+Z)" disabled={!state.past.length} onClick={() => dispatch({ type: "UNDO" })} />
          <ToolbarButton label="↷" title="Redo (Ctrl+Y)" disabled={!state.future.length} onClick={() => dispatch({ type: "REDO" })} />
          <ToolbarButton label="Reset" title="Reset every authored edit" onClick={() => dispatch({ type: "RESET" })} />
        </div>
        <div className="placeable-authoring__segmented" aria-label="Preview mode">
          {(["ORIGINAL", "EDITED"] as const).map((preview) => (
            <button
              type="button"
              key={preview}
              data-active={state.preview === preview}
              onClick={() => dispatch({ type: "SET_PREVIEW", preview })}
            >
              {preview === "ORIGINAL" ? "Original" : "Edited"}
            </button>
          ))}
        </div>
        <div className="placeable-authoring__tool-group" aria-label="Collision viewport tools">
          <label className="placeable-authoring__collision-toggle">
            <input
              type="checkbox"
              checked={showCollision}
              onChange={(event) => setShowCollision(event.target.checked)}
            />
            Show PWK
          </label>
          <ToolbarButton
            label="Top"
            title="Orthographic top view for collision editing"
            active={cameraMode === "TOP"}
            onClick={() => setCameraMode((mode) => mode === "TOP" ? "PERSPECTIVE" : "TOP")}
          />
          <ToolbarButton
            label="Draw"
            title="Draw custom collision polygon on the ground plane"
            active={collisionDrawing}
            onClick={() => {
              if (state.present.collision.mode !== "CUSTOM_POLYGON") activateCustomCollision();
              setCameraMode("TOP");
              setShowCollision(true);
              setCollisionDrawing((drawing) => !drawing);
            }}
          />
        </div>
      </header>

      <div className="placeable-authoring__workspace">
        <aside className="placeable-authoring__outliner panel" aria-label="Placeable outliner">
          <header><strong>Outliner</strong><span>{elements.length} elements</span></header>
          <div className="placeable-authoring__outliner-actions">
            <ToolbarButton label="All" title="Select all elements" onClick={() => dispatch({ type: "SELECT_ALL" })} />
            <ToolbarButton label="Show" title="Show all elements" onClick={() => dispatch({ type: "SHOW_ALL" })} />
            <ToolbarButton label="Iso" title="Isolate selection (F)" disabled={!selected.length} onClick={() => dispatch({ type: "ISOLATE_SELECTION" })} />
          </div>
          <ul
            className="placeable-authoring__tree"
            onScroll={(event) => setOutlinerScrollTop(event.currentTarget.scrollTop)}
            style={{
              paddingTop: `${5 + outlinerStart * OUTLINER_ROW_HEIGHT}px`,
              paddingBottom: `${5 + (elements.length - outlinerEnd) * OUTLINER_ROW_HEIGHT}px`,
            }}
          >
            {outlinerElements.map((element) => {
              const depth = (() => {
                let value = 0;
                let parentId = element.parentId;
                while (parentId && value < 12) {
                  value += 1;
                  parentId = elements.find((candidate) => candidate.id === parentId)?.parentId ?? null;
                }
                return value;
              })();
              return (
                <li key={element.id} data-selected={state.selectedIds.includes(element.id)}>
                  <button
                    type="button"
                    className="placeable-authoring__tree-select"
                    style={{ paddingInlineStart: `${8 + depth * 14}px` }}
                    onClick={(event) => dispatch({
                      type: "SELECT",
                      id: element.id,
                      additive: event.ctrlKey || event.metaKey || event.shiftKey,
                    })}
                  >
                    <span>{element.kind === "GROUP" ? "◇" : element.flags.hidden ? "◌" : "◆"}</span>
                    <strong>{element.name}</strong>
                  </button>
                  <button
                    type="button"
                    title={element.flags.hidden ? "Show element" : "Hide element"}
                    onClick={() => dispatch({
                      type: "PATCH_FLAGS",
                      ids: [element.id],
                      flags: { hidden: !element.flags.hidden },
                    })}
                  >
                    {element.flags.hidden ? "○" : "●"}
                  </button>
                  <button
                    type="button"
                    title={element.flags.locked ? "Unlock element" : "Lock element"}
                    onClick={() => dispatch({
                      type: "PATCH_FLAGS",
                      ids: [element.id],
                      flags: { locked: !element.flags.locked },
                    })}
                  >
                    {element.flags.locked ? "🔒" : "·"}
                  </button>
                  {splitAvailable(element) ? (
                    <button
                      type="button"
                      title="Split by connected components"
                      onClick={() => splitNode(element)}
                    >
                      Split
                    </button>
                  ) : null}
                </li>
              );
            })}
          </ul>
        </aside>

        <div className="placeable-authoring__stage">
          <PlaceableAuthoringViewport
            file={file}
            dependency={`${file.name}:${sourceSha256}`}
            document={state.preview === "ORIGINAL" ? state.original : state.present}
            inspection={bootstrap.inspection}
            selectedIds={state.preview === "EDITED" ? state.selectedIds : []}
            tool={state.tool}
            space={state.space}
            snap={state.snap}
            editable={state.preview === "EDITED"}
            cameraMode={cameraMode}
            showCollision={showCollision}
            collisionDrawing={collisionDrawing}
            collisionVertices={collisionVertices}
            resolvedCollisionVertices={resolvedCollision?.sourceVertices ?? []}
            collisionTriangles={resolvedCollision?.triangles ?? []}
            collisionEditable={state.preview === "EDITED" && state.present.collision.mode === "CUSTOM_POLYGON"}
            onAddCollisionVertex={(vertex) => dispatch({ type: "ADD_COLLISION_VERTEX", vertex })}
            onMoveCollisionVertex={(index, vertex) => dispatch({ type: "MOVE_COLLISION_VERTEX", index, vertex })}
            onSelect={(id, additive) => dispatch({ type: "SELECT", id, additive })}
            onClearSelection={() => dispatch({ type: "CLEAR_SELECTION" })}
            onGestureStart={() => dispatch({ type: "BEGIN_GESTURE" })}
            onTransformPreview={(patches) => dispatch({ type: "PREVIEW_TRANSFORMS", patches })}
            onGestureEnd={() => dispatch({ type: "END_GESTURE" })}
            onError={onError}
            textureSnapshot={textureSnapshot}
            textureInspection={textureBootstrap?.inspection.materials ?? []}
          />
          <div className="placeable-authoring__quick-actions" aria-label="Element operations">
            <ToolbarButton label="Duplicate" title="Duplicate selected elements (Ctrl+D)" disabled={!selected.length} onClick={() => dispatch({ type: "DUPLICATE" })} />
            <ToolbarButton label="Delete" title="Delete selected elements" disabled={!selected.length} onClick={() => dispatch({ type: "DELETE" })} />
            <ToolbarButton label="Group" title="Group selected elements" disabled={selected.length < 2} onClick={() => dispatch({ type: "GROUP" })} />
            <ToolbarButton label="Ungroup" title="Ungroup selected group" disabled={!selected.some((element) => element.kind === "GROUP")} onClick={() => dispatch({ type: "UNGROUP" })} />
            <ToolbarButton label="Ground" title="Place selection on the ground plane" disabled={!selected.length} onClick={() => dispatch({ type: "GROUND", boundsById: groundBoundsById(measurements) })} />
            <ToolbarButton label="Hide" title="Hide selection" disabled={!selected.length} onClick={() => dispatch({ type: "HIDE_SELECTED", hidden: true })} />
            <ToolbarButton label="Lock" title="Lock selection" disabled={!selected.length} onClick={() => dispatch({ type: "LOCK_SELECTED", locked: true })} />
          </div>
        </div>

        <aside className="placeable-authoring__properties panel" aria-label="Element properties">
          <header><strong>Properties</strong><span>{selected.length ? `${selected.length} selected` : "Collision"}</span></header>
          <section className="placeable-authoring__collision-editor" aria-label="Placeable collision editor">
            <div className="placeable-authoring__collision-heading">
              <strong>Blocking footprint (PWK)</strong>
              <span>{resolvedCollision ? `${resolvedCollision.triangles.length} faces` : "resolving…"}</span>
            </div>
            <label>
              Mode
              <select
                aria-label="Collision mode"
                value={state.present.collision.mode}
                onChange={(event) => {
                  if (event.target.value === "CUSTOM_POLYGON") activateCustomCollision();
                  else {
                    setCollisionDrawing(false);
                    dispatch({ type: "SET_COLLISION_MODE", mode: "AUTO_RECTANGLE" });
                  }
                }}
              >
                <option value="AUTO_RECTANGLE">Auto rectangle</option>
                <option value="CUSTOM_POLYGON">Custom polygon</option>
              </select>
            </label>
            {state.present.collision.mode === "AUTO_RECTANGLE" ? (
              <label>
                Padding (m)
                <input
                  aria-label="Collision padding metres"
                  type="number"
                  min="0"
                  step="0.05"
                  value={state.present.collision.paddingMeters}
                  onChange={(event) => dispatch({
                    type: "SET_COLLISION_PADDING",
                    paddingMeters: finite(event.target.value, state.present.collision.paddingMeters),
                  })}
                />
              </label>
            ) : (
              <>
                <div className="placeable-authoring__collision-actions">
                  <ToolbarButton
                    label={collisionDrawing ? "Finish drawing" : "Draw points"}
                    title="Add polygon vertices by clicking the top viewport"
                    active={collisionDrawing}
                    onClick={() => {
                      setCameraMode("TOP");
                      setShowCollision(true);
                      setCollisionDrawing((drawing) => !drawing);
                    }}
                  />
                  <ToolbarButton
                    label="Clear"
                    title="Clear custom polygon vertices"
                    disabled={!state.present.collision.vertices.length}
                    onClick={() => dispatch({ type: "SET_COLLISION_VERTICES", vertices: [] })}
                  />
                  <ToolbarButton
                    label="Auto"
                    title="Return to the automatic rectangle"
                    onClick={() => {
                      setCollisionDrawing(false);
                      dispatch({ type: "SET_COLLISION_MODE", mode: "AUTO_RECTANGLE" });
                    }}
                  />
                </div>
                <ol className="placeable-authoring__collision-vertices">
                  {state.present.collision.vertices.map((vertex, index) => (
                    <li key={index}>
                      <span>{index + 1}</span>
                      <input
                        aria-label={`Collision vertex ${index + 1} X`}
                        type="number"
                        step="0.05"
                        value={Number(vertex[0].toFixed(4))}
                        onChange={(event) => dispatch({
                          type: "MOVE_COLLISION_VERTEX",
                          index,
                          vertex: [finite(event.target.value, vertex[0]), vertex[1]],
                        })}
                      />
                      <input
                        aria-label={`Collision vertex ${index + 1} Z`}
                        type="number"
                        step="0.05"
                        value={Number(vertex[1].toFixed(4))}
                        onChange={(event) => dispatch({
                          type: "MOVE_COLLISION_VERTEX",
                          index,
                          vertex: [vertex[0], finite(event.target.value, vertex[1])],
                        })}
                      />
                      <button
                        type="button"
                        aria-label={`Delete collision vertex ${index + 1}`}
                        onClick={() => dispatch({ type: "DELETE_COLLISION_VERTEX", index })}
                      >×</button>
                    </li>
                  ))}
                </ol>
              </>
            )}
            <dl className="placeable-authoring__collision-status">
              <div><dt>Space</dt><dd>GLTF X/Z → Aurora X/Y</dd></div>
              <div><dt>Surface</dt><dd>{resolvedCollision?.surfaceId ?? 7} (non-walk)</dd></div>
              <div><dt>Vertices</dt><dd>{collisionVertices.length} / 64</dd></div>
            </dl>
          </section>
          {primary ? (
            <div className="placeable-authoring__properties-body">
              <section>
                <h3>{primary.name}</h3>
                <p>{primary.id}</p>
              </section>
              {(["translation", "rotation", "scale", "pivot"] as const).map((kind) => {
                const values = kind === "rotation" ? transformEuler(primary.transform) : primary.transform[kind];
                return (
                  <fieldset key={kind}>
                    <legend>{kind === "translation" ? "Location" : kind === "rotation" ? "Rotation °" : kind[0].toUpperCase() + kind.slice(1)}</legend>
                    {axisNames.map((axis, index) => (
                      <label key={axis}>
                        <span>{axis}</span>
                        <input
                          aria-label={`${kind} ${axis}`}
                          type="number"
                          step={kind === "rotation" ? "1" : "0.01"}
                          value={Number(values[index].toFixed(4))}
                          disabled={primary.flags.locked}
                          onChange={(event) => {
                            if (kind === "rotation") {
                              updatePrimaryTransform(withEuler(
                                primary.transform,
                                index,
                                finite(event.target.value, values[index]),
                              ));
                            } else {
                              updateVector(kind, index, event.target.value);
                            }
                          }}
                        />
                      </label>
                    ))}
                  </fieldset>
                );
              })}
              <label className="placeable-authoring__uniform-scale">
                Uniform scale
                <input
                  aria-label="Uniform scale"
                  type="number"
                  step="0.05"
                  value={Number(primary.transform.scale[0].toFixed(4))}
                  disabled={primary.flags.locked}
                  onChange={(event) => {
                    const value = finite(event.target.value, primary.transform.scale[0]);
                    updatePrimaryTransform({ ...primary.transform, scale: [value, value, value] });
                  }}
                />
              </label>
              <div className="placeable-authoring__button-row">
                <ToolbarButton label="Set Origin" title="Move pivot to geometry center without moving geometry" onClick={() => dispatch({ type: "SET_ORIGIN", pivot: sourceCenter(primary, bootstrap) })} />
                <ToolbarButton label="Copy T" title="Copy transform" onClick={() => dispatch({ type: "COPY_TRANSFORM" })} />
                <ToolbarButton label="Paste T" title="Paste transform" disabled={!state.transformClipboard} onClick={() => dispatch({ type: "PASTE_TRANSFORM" })} />
              </div>
              <fieldset>
                <legend>Mirror</legend>
                {axisNames.map((axis, index) => (
                  <ToolbarButton key={axis} label={axis} title={`Mirror on ${axis}`} onClick={() => dispatch({ type: "MIRROR", axis: index as 0 | 1 | 2 })} />
                ))}
              </fieldset>
              <fieldset>
                <legend>Align selected</legend>
                <select
                  aria-label="Align mode"
                  value={alignMode}
                  onChange={(event) => setAlignMode(event.target.value as "MIN" | "CENTER" | "MAX")}
                >
                  <option value="MIN">Min</option>
                  <option value="CENTER">Center</option>
                  <option value="MAX">Max</option>
                </select>
                {axisNames.map((axis, index) => (
                  <ToolbarButton
                    key={axis}
                    label={axis}
                    title={`Align ${alignMode.toLowerCase()} on ${axis}`}
                    disabled={selected.length < 2}
                    onClick={() => dispatch({
                      type: "ALIGN",
                      axis: index as 0 | 1 | 2,
                      mode: alignMode,
                    })}
                  />
                ))}
              </fieldset>
              <label className="placeable-authoring__parent-field">
                Parent
                <select
                  value={primary.parentId ?? ""}
                  disabled={selected.length === 0}
                  onChange={(event) => {
                    if (event.target.value) dispatch({ type: "PARENT", parentId: event.target.value });
                    else dispatch({ type: "UNPARENT" });
                  }}
                >
                  <option value="">Scene root</option>
                  {elements.filter((element) => !state.selectedIds.includes(element.id)).map((element) => (
                    <option key={element.id} value={element.id}>{element.name}</option>
                  ))}
                </select>
              </label>
              <fieldset className="placeable-authoring__flags">
                <legend>Output participation</legend>
                {([
                  ["renderable", "Renderable"],
                  ["includeInCollision", "Include in collision"],
                  ["castShadow", "Cast shadow"],
                ] as const).map(([flag, label]) => (
                  <label key={flag}>
                    <input
                      type="checkbox"
                      checked={primary.flags[flag]}
                      onChange={(event) => dispatch({
                        type: "PATCH_FLAGS",
                        ids: state.selectedIds,
                        flags: { [flag]: event.target.checked },
                      })}
                    />
                    {label}
                  </label>
                ))}
              </fieldset>
              {measurementById.get(primary.id) ? (
                <dl className="placeable-authoring__measurements">
                  <div><dt>Size</dt><dd>{measurementById.get(primary.id)!.size.map((value) => value.toFixed(3)).join(" × ")} m</dd></div>
                  <div><dt>Center</dt><dd>{measurementById.get(primary.id)!.center.map((value) => value.toFixed(3)).join(", ")}</dd></div>
                  <div><dt>Origin distance</dt><dd>{measurementById.get(primary.id)!.distanceFromOrigin.toFixed(3)} m</dd></div>
                </dl>
              ) : null}
            </div>
          ) : (
            <p className="placeable-authoring__empty">Select an element in the viewport or Outliner.</p>
          )}
        </aside>
      </div>

      {textureBootstrap ? (
        <PlaceableTextureEditor
          bootstrap={textureBootstrap}
          resolved={resolvedTextures}
          onSnapshotChange={publishTextureSnapshot}
          onError={onError}
        />
      ) : null}

      <section className="placeable-authoring__diagnostics panel" aria-label="Placeable diagnostics">
        <header>
          <strong>Diagnostics</strong>
          <span>{diagnostics.length ? `${diagnostics.length} findings` : "No findings"}</span>
        </header>
        {diagnostics.length ? (
          <ul>
            {diagnostics.map((diagnostic, index) => (
              <li key={`${diagnostic.code}-${index}`} data-severity={diagnostic.severity}>
                <button type="button" onClick={() => selectDiagnostic(diagnostic.elementIds)}>
                  <strong>{diagnostic.code}</strong>
                  <span>{diagnostic.message}</span>
                </button>
              </li>
            ))}
          </ul>
        ) : <p>No floating, below-ground, distant or disconnected element issues detected.</p>}
      </section>
    </section>
  );
}
