import * as THREE from "three";
import type {
  PlaceableAuthoringAction,
  PlaceableAuthoringDocument,
  PlaceableAuthoringEditorState,
  PlaceableAuthoringElement,
  PlaceableElementTransform,
} from "./types";

const clone = <T,>(value: T): T => structuredClone(value);
const live = (element: PlaceableAuthoringElement) => !element.deleted;

export function createPlaceableAuthoringEditorState(
  document: PlaceableAuthoringDocument,
): PlaceableAuthoringEditorState {
  return {
    original: clone(document),
    present: clone(document),
    past: [],
    future: [],
    selectedIds: [],
    primaryId: null,
    tool: "TRANSLATE",
    space: "WORLD",
    preview: "EDITED",
    snap: {
      mode: "GRID",
      translationStep: 0.1,
      rotationDegrees: 15,
      scaleStep: 0.1,
    },
    transformClipboard: null,
    gestureOrigin: null,
  };
}

function documentEquals(left: PlaceableAuthoringDocument, right: PlaceableAuthoringDocument) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function commit(
  state: PlaceableAuthoringEditorState,
  document: PlaceableAuthoringDocument,
  selection = state.selectedIds,
): PlaceableAuthoringEditorState {
  if (documentEquals(state.present, document)) return state;
  return {
    ...state,
    present: document,
    past: [...state.past, clone(state.present)],
    future: [],
    selectedIds: selection,
    primaryId: selection.at(-1) ?? null,
    preview: "EDITED",
    gestureOrigin: null,
  };
}

function updateElements(
  document: PlaceableAuthoringDocument,
  update: (element: PlaceableAuthoringElement) => PlaceableAuthoringElement,
): PlaceableAuthoringDocument {
  return { ...document, elements: document.elements.map(update) };
}

function replaceTransforms(
  document: PlaceableAuthoringDocument,
  patches: readonly { id: string; transform: PlaceableElementTransform }[],
): PlaceableAuthoringDocument {
  const byId = new Map(patches.map((patch) => [patch.id, patch.transform]));
  return updateElements(document, (element) => {
    const transform = byId.get(element.id);
    return transform ? { ...element, transform: clone(transform) } : element;
  });
}

function matrixFor(transform: PlaceableElementTransform) {
  const translation = new THREE.Matrix4().makeTranslation(...transform.translation);
  const pivot = new THREE.Matrix4().makeTranslation(...transform.pivot);
  const inversePivot = new THREE.Matrix4().makeTranslation(
    -transform.pivot[0],
    -transform.pivot[1],
    -transform.pivot[2],
  );
  const rotationScale = new THREE.Matrix4().compose(
    new THREE.Vector3(),
    new THREE.Quaternion(...transform.rotationXyzw).normalize(),
    new THREE.Vector3(...transform.scale),
  );
  return translation.multiply(pivot).multiply(rotationScale).multiply(inversePivot);
}

function transformFor(matrix: THREE.Matrix4): PlaceableElementTransform {
  const translation = new THREE.Vector3();
  const rotation = new THREE.Quaternion();
  const scale = new THREE.Vector3();
  matrix.decompose(translation, rotation, scale);
  return {
    translation: translation.toArray() as [number, number, number],
    rotationXyzw: rotation.toArray() as [number, number, number, number],
    scale: scale.toArray() as [number, number, number],
    pivot: [0, 0, 0],
  };
}

function worldMatrix(
  element: PlaceableAuthoringElement,
  document: PlaceableAuthoringDocument,
  visiting = new Set<string>(),
): THREE.Matrix4 {
  if (visiting.has(element.id)) return new THREE.Matrix4();
  visiting.add(element.id);
  const local = matrixFor(element.transform);
  const parent = element.parentId
    ? document.elements.find((candidate) => candidate.id === element.parentId && live(candidate))
    : undefined;
  const result = parent
    ? worldMatrix(parent, document, visiting).multiply(local)
    : local;
  visiting.delete(element.id);
  return result;
}

function reparent(
  document: PlaceableAuthoringDocument,
  ids: readonly string[],
  parentId: string | null,
): PlaceableAuthoringDocument {
  const selected = new Set(ids);
  const parent = parentId
    ? document.elements.find((element) => element.id === parentId && live(element))
    : undefined;
  if (parentId && (!parent || selected.has(parentId))) return document;
  for (const id of ids) {
    let cursor = parent;
    while (cursor) {
      if (cursor.id === id) return document;
      cursor = cursor.parentId
        ? document.elements.find((element) => element.id === cursor?.parentId && live(element))
        : undefined;
    }
  }
  const parentInverse = parent
    ? worldMatrix(parent, document).invert()
    : new THREE.Matrix4();
  const replacements = new Map<string, PlaceableAuthoringElement>();
  for (const element of document.elements) {
    if (!selected.has(element.id) || !live(element)) continue;
    const local = parentInverse.clone().multiply(worldMatrix(element, document));
    replacements.set(element.id, {
      ...element,
      parentId,
      transform: transformFor(local),
    });
  }
  return updateElements(document, (element) => replacements.get(element.id) ?? element);
}

function uniqueId(document: PlaceableAuthoringDocument, stem: string) {
  const used = new Set(document.elements.map((element) => element.id));
  for (let index = 1; ; index += 1) {
    const candidate = `${stem}-${String(index).padStart(3, "0")}`;
    if (!used.has(candidate)) return candidate;
  }
}

function withTranslation(
  transform: PlaceableElementTransform,
  axis: 0 | 1 | 2,
  value: number,
): PlaceableElementTransform {
  const translation = [...transform.translation] as [number, number, number];
  translation[axis] = value;
  return { ...transform, translation };
}

function selectedLive(state: PlaceableAuthoringEditorState) {
  const selected = new Set(state.selectedIds);
  return state.present.elements.filter((element) => selected.has(element.id) && live(element));
}

export function placeableAuthoringReducer(
  state: PlaceableAuthoringEditorState,
  action: PlaceableAuthoringAction,
): PlaceableAuthoringEditorState {
  switch (action.type) {
    case "SELECT": {
      const element = state.present.elements.find((candidate) => candidate.id === action.id && live(candidate));
      if (!element) return state;
      const selectedIds = action.additive
        ? state.selectedIds.includes(action.id)
          ? state.selectedIds.filter((id) => id !== action.id)
          : [...state.selectedIds, action.id]
        : [action.id];
      return { ...state, selectedIds, primaryId: selectedIds.at(-1) ?? null };
    }
    case "SELECT_ALL": {
      const selectedIds = state.present.elements.filter(live).map((element) => element.id);
      return { ...state, selectedIds, primaryId: selectedIds.at(-1) ?? null };
    }
    case "CLEAR_SELECTION":
      return { ...state, selectedIds: [], primaryId: null };
    case "SET_TOOL":
      return { ...state, tool: action.tool };
    case "SET_SPACE":
      return { ...state, space: action.space };
    case "SET_PREVIEW":
      return { ...state, preview: action.preview };
    case "SET_SNAP":
      return { ...state, snap: { ...state.snap, ...action.snap } };
    case "BEGIN_GESTURE":
      return state.gestureOrigin ? state : { ...state, gestureOrigin: clone(state.present) };
    case "PREVIEW_TRANSFORMS":
      return {
        ...state,
        present: replaceTransforms(state.present, action.patches),
        preview: "EDITED",
      };
    case "END_GESTURE":
      if (!state.gestureOrigin) return state;
      if (documentEquals(state.gestureOrigin, state.present)) {
        return { ...state, gestureOrigin: null };
      }
      return {
        ...state,
        past: [...state.past, state.gestureOrigin],
        future: [],
        gestureOrigin: null,
      };
    case "SET_TRANSFORMS":
      return commit(state, replaceTransforms(state.present, action.patches));
    case "PATCH_FLAGS": {
      const ids = new Set(action.ids);
      return commit(state, updateElements(state.present, (element) => ids.has(element.id)
        ? { ...element, flags: { ...element.flags, ...action.flags } }
        : element));
    }
    case "SPLIT_NODE_COMPONENTS": {
      const element = state.present.elements.find((candidate) => candidate.id === action.id);
      if (!element || element.kind !== "SOURCE_NODE" || !element.source) return state;
      const node = action.inspection.nodes.find((candidate) => candidate.nodeId === element.source?.nodeId);
      if (!node) return state;
      const replacements = node.primitives.flatMap((primitive) => primitive.components.map((component) => ({
        ...element,
        id: component.elementId,
        name: `${node.name} · P${primitive.primitiveId} C${component.componentIndex}`,
        kind: "SOURCE_COMPONENT" as const,
        source: {
          nodeId: node.nodeId,
          primitiveId: primitive.primitiveId,
          componentIndex: component.componentIndex,
        },
      })));
      const document = {
        ...state.present,
        elements: state.present.elements.flatMap((candidate) => candidate.id === element.id
          ? replacements
          : [candidate]),
      };
      return commit(state, document, replacements.map((candidate) => candidate.id));
    }
    case "DUPLICATE": {
      const copies: PlaceableAuthoringElement[] = [];
      let document = state.present;
      for (const element of selectedLive(state)) {
        if (!element.source) continue;
        const id = uniqueId({ ...document, elements: [...document.elements, ...copies] }, `${element.id}-copy`);
        copies.push({
          ...clone(element),
          id,
          name: `${element.name} copy`,
          kind: "COPY",
          transform: withTranslation(
            element.transform,
            0,
            element.transform.translation[0] + Math.max(state.snap.translationStep, 0.01),
          ),
        });
      }
      if (!copies.length) return state;
      document = { ...document, elements: [...document.elements, ...copies] };
      return commit(state, document, copies.map((copy) => copy.id));
    }
    case "DELETE": {
      const ids = new Set(state.selectedIds);
      let document = state.present;
      for (const element of state.present.elements.filter((candidate) => ids.has(candidate.parentId ?? ""))) {
        document = reparent(document, [element.id], element.parentId
          ? state.present.elements.find((parent) => parent.id === element.parentId)?.parentId ?? null
          : null);
      }
      document = updateElements(document, (element) => ids.has(element.id)
        ? { ...element, deleted: true }
        : element);
      return commit(state, document, []);
    }
    case "GROUP": {
      const elements = selectedLive(state);
      if (elements.length < 2) return state;
      const parentId = elements.every((element) => element.parentId === elements[0].parentId)
        ? elements[0].parentId
        : null;
      const centers = elements.map((element) => new THREE.Vector3().setFromMatrixPosition(worldMatrix(element, state.present)));
      const center = centers.reduce((sum, value) => sum.add(value), new THREE.Vector3()).multiplyScalar(1 / centers.length);
      const groupId = uniqueId(state.present, "group");
      const parent = parentId
        ? state.present.elements.find((element) => element.id === parentId)
        : undefined;
      const local = parent
        ? worldMatrix(parent, state.present).invert().multiply(new THREE.Matrix4().makeTranslation(...center.toArray()))
        : new THREE.Matrix4().makeTranslation(...center.toArray());
      const group: PlaceableAuthoringElement = {
        id: groupId,
        name: "Group",
        kind: "GROUP",
        source: null,
        parentId,
        transform: transformFor(local),
        flags: {
          hidden: false,
          locked: false,
          renderable: true,
          includeInCollision: true,
          castShadow: true,
        },
        deleted: false,
      };
      let document: PlaceableAuthoringDocument = {
        ...state.present,
        elements: [...state.present.elements, group],
      };
      document = reparent(document, elements.map((element) => element.id), groupId);
      return commit(state, document, [groupId]);
    }
    case "UNGROUP": {
      let document = state.present;
      const groups = selectedLive(state).filter((element) => element.kind === "GROUP");
      if (!groups.length) return state;
      const childIds: string[] = [];
      for (const group of groups) {
        const children = document.elements.filter((element) => live(element) && element.parentId === group.id);
        childIds.push(...children.map((element) => element.id));
        document = reparent(document, children.map((element) => element.id), group.parentId);
        document = updateElements(document, (element) => element.id === group.id
          ? { ...element, deleted: true }
          : element);
      }
      return commit(state, document, childIds);
    }
    case "PARENT":
      return commit(state, reparent(state.present, state.selectedIds, action.parentId));
    case "UNPARENT":
      return commit(state, reparent(state.present, state.selectedIds, null));
    case "MIRROR": {
      const ids = new Set(state.selectedIds);
      return commit(state, updateElements(state.present, (element) => {
        if (!ids.has(element.id) || element.flags.locked) return element;
        const scale = [...element.transform.scale] as [number, number, number];
        scale[action.axis] *= -1;
        return { ...element, transform: { ...element.transform, scale } };
      }));
    }
    case "COPY_TRANSFORM": {
      const element = state.present.elements.find((candidate) => candidate.id === state.primaryId);
      return element ? { ...state, transformClipboard: clone(element.transform) } : state;
    }
    case "PASTE_TRANSFORM": {
      if (!state.transformClipboard) return state;
      const ids = new Set(state.selectedIds);
      return commit(state, updateElements(state.present, (element) => ids.has(element.id) && !element.flags.locked
        ? { ...element, transform: clone(state.transformClipboard!) }
        : element));
    }
    case "SET_PIVOT": {
      const ids = new Set(state.selectedIds);
      return commit(state, updateElements(state.present, (element) => ids.has(element.id) && !element.flags.locked
        ? { ...element, transform: { ...element.transform, pivot: [...action.pivot] } }
        : element));
    }
    case "SET_ORIGIN": {
      const ids = new Set(state.selectedIds);
      return commit(state, updateElements(state.present, (element) => {
        if (!ids.has(element.id) || element.flags.locked) return element;
        const oldPivot = new THREE.Vector3(...element.transform.pivot);
        const newPivot = new THREE.Vector3(...action.pivot);
        const rotationScale = new THREE.Matrix4().compose(
          new THREE.Vector3(),
          new THREE.Quaternion(...element.transform.rotationXyzw),
          new THREE.Vector3(...element.transform.scale),
        );
        const compensation = oldPivot.clone()
          .sub(oldPivot.clone().applyMatrix4(rotationScale))
          .sub(newPivot)
          .add(newPivot.clone().applyMatrix4(rotationScale));
        const translation = new THREE.Vector3(...element.transform.translation).add(compensation);
        return {
          ...element,
          transform: {
            ...element.transform,
            translation: translation.toArray() as [number, number, number],
            pivot: [...action.pivot],
          },
        };
      }));
    }
    case "HIDE_SELECTED":
      return placeableAuthoringReducer(state, {
        type: "PATCH_FLAGS",
        ids: state.selectedIds,
        flags: { hidden: action.hidden },
      });
    case "ISOLATE_SELECTION": {
      const ids = new Set(state.selectedIds);
      return commit(state, updateElements(state.present, (element) => ({
        ...element,
        flags: { ...element.flags, hidden: !ids.has(element.id) },
      })));
    }
    case "SHOW_ALL":
      return commit(state, updateElements(state.present, (element) => ({
        ...element,
        flags: { ...element.flags, hidden: false },
      })));
    case "LOCK_SELECTED":
      return placeableAuthoringReducer(state, {
        type: "PATCH_FLAGS",
        ids: state.selectedIds,
        flags: { locked: action.locked },
      });
    case "ALIGN": {
      const elements = selectedLive(state).filter((element) => !element.flags.locked);
      if (elements.length < 2) return state;
      const values = elements.map((element) => element.transform.translation[action.axis]);
      const target = action.mode === "MIN"
        ? Math.min(...values)
        : action.mode === "MAX"
          ? Math.max(...values)
          : values.reduce((sum, value) => sum + value, 0) / values.length;
      return commit(state, replaceTransforms(state.present, elements.map((element) => ({
        id: element.id,
        transform: withTranslation(element.transform, action.axis, target),
      }))));
    }
    case "GROUND": {
      const patches = selectedLive(state)
        .filter((element) => !element.flags.locked)
        .flatMap((element) => {
          const bounds = action.boundsById[element.id];
          if (!bounds) return [];
          return [{
            id: element.id,
            transform: withTranslation(
              element.transform,
              1,
              element.transform.translation[1] - bounds[0],
            ),
          }];
        });
      return commit(state, replaceTransforms(state.present, patches));
    }
    case "UNDO": {
      const previous = state.past.at(-1);
      if (!previous) return state;
      return {
        ...state,
        present: previous,
        past: state.past.slice(0, -1),
        future: [clone(state.present), ...state.future],
        gestureOrigin: null,
      };
    }
    case "REDO": {
      const next = state.future[0];
      if (!next) return state;
      return {
        ...state,
        present: next,
        past: [...state.past, clone(state.present)],
        future: state.future.slice(1),
        gestureOrigin: null,
      };
    }
    case "RESET":
      return commit(state, clone(state.original), []);
  }
}
