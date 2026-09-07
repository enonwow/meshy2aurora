import type {
  AuthoredMaterialV1,
  ModelComponentInventoryV1,
  ModelMaterialAssignmentV1,
  ModelMaterialFaceAssignmentV2,
  ModelMaterialSeparationDocumentV2,
  SourceFaceKeyV2,
} from "./types";
import { componentKey, faceKey, parseComponentKey, parseFaceKey } from "./types";

export const MATERIAL_SEPARATION_V2_HISTORY_LIMIT = 50;
export type MaterialSelectionModeV2 = "COMPONENT" | "FACE";

export interface MaterialSeparationEditorStateV2 {
  readonly initial: ModelMaterialSeparationDocumentV2;
  readonly applied: ModelMaterialSeparationDocumentV2;
  readonly draft: ModelMaterialSeparationDocumentV2;
  readonly past: readonly ModelMaterialSeparationDocumentV2[];
  readonly future: readonly ModelMaterialSeparationDocumentV2[];
  readonly selectionMode: MaterialSelectionModeV2;
  readonly selectedComponentKeys: readonly string[];
  readonly selectedFaceKeys: readonly string[];
  readonly overlayEnabled: boolean;
  readonly isolateSelection: boolean;
}

export type MaterialSeparationActionV2 =
  | { readonly type: "SET_SELECTION_MODE"; readonly mode: MaterialSelectionModeV2 }
  | { readonly type: "SELECT_COMPONENT"; readonly key: string; readonly additive?: boolean }
  | { readonly type: "SELECT_FACE"; readonly key: string; readonly additive?: boolean }
  | { readonly type: "SELECT_FACES"; readonly keys: readonly string[]; readonly additive?: boolean }
  | { readonly type: "SELECT_NODE"; readonly nodeId: number; readonly inventory: ModelComponentInventoryV1 }
  | { readonly type: "SELECT_PRIMITIVE"; readonly nodeId: number; readonly primitiveId: number; readonly inventory: ModelComponentInventoryV1 }
  | { readonly type: "SELECT_BY_MATERIAL"; readonly authoredMaterialId: string }
  | { readonly type: "SELECT_UNASSIGNED_COMPONENTS"; readonly inventory: ModelComponentInventoryV1 }
  | { readonly type: "CLEAR_SELECTION" }
  | { readonly type: "NEW_MATERIAL"; readonly material: AuthoredMaterialV1 }
  | { readonly type: "RENAME_MATERIAL"; readonly authoredMaterialId: string; readonly displayName: string }
  | { readonly type: "DELETE_UNUSED" }
  | { readonly type: "ASSIGN_SELECTION"; readonly authoredMaterialId: string }
  | { readonly type: "UNASSIGN_SELECTION" }
  | { readonly type: "TOGGLE_OVERLAY" }
  | { readonly type: "TOGGLE_ISOLATE" }
  | { readonly type: "UNDO" }
  | { readonly type: "REDO" }
  | { readonly type: "APPLY" }
  | { readonly type: "CANCEL" }
  | { readonly type: "RESET" };

const clone = <T,>(value: T): T => structuredClone(value);
const equal = (left: unknown, right: unknown) => JSON.stringify(left) === JSON.stringify(right);

export function createMaterialSeparationEditorStateV2(
  document: ModelMaterialSeparationDocumentV2,
): MaterialSeparationEditorStateV2 {
  return {
    initial: clone(document),
    applied: clone(document),
    draft: clone(document),
    past: [],
    future: [],
    selectionMode: "FACE",
    selectedComponentKeys: [],
    selectedFaceKeys: [],
    overlayEnabled: true,
    isolateSelection: false,
  };
}

function sortComponentAssignments(assignments: readonly ModelMaterialAssignmentV1[]) {
  return [...assignments].sort((left, right) => (
    componentKey(left.component).localeCompare(componentKey(right.component))
      || left.authoredMaterialId.localeCompare(right.authoredMaterialId)
  ));
}

function faceAssignmentMap(assignments: readonly ModelMaterialFaceAssignmentV2[]) {
  const output = new Map<string, string>();
  for (const assignment of assignments) {
    for (const range of assignment.selection.triangleRanges) {
      for (let triangleIndex = range.startTriangle;
        triangleIndex < range.startTriangle + range.triangleCount;
        triangleIndex += 1) {
        output.set(faceKey({
          sceneId: assignment.selection.sceneId,
          nodeId: assignment.selection.nodeId,
          primitiveId: assignment.selection.primitiveId,
          triangleIndex,
        }), assignment.authoredMaterialId);
      }
    }
  }
  return output;
}

function compactTriangleIndices(indices: readonly number[]) {
  const sorted = [...new Set(indices)].sort((left, right) => left - right);
  const ranges: { startTriangle: number; triangleCount: number }[] = [];
  for (const triangleIndex of sorted) {
    const previous = ranges.at(-1);
    if (previous && previous.startTriangle + previous.triangleCount === triangleIndex) {
      previous.triangleCount += 1;
    } else {
      ranges.push({ startTriangle: triangleIndex, triangleCount: 1 });
    }
  }
  return ranges;
}

export function compactFaceAssignmentsV2(
  assignments: ReadonlyMap<string, string>,
): readonly ModelMaterialFaceAssignmentV2[] {
  const grouped = new Map<string, { identity: Omit<SourceFaceKeyV2, "triangleIndex">; material: string; indices: number[] }>();
  for (const [key, material] of assignments) {
    const face = parseFaceKey(key);
    const groupKey = `${face.sceneId}/${face.nodeId}/${face.primitiveId}/${material}`;
    const group = grouped.get(groupKey) ?? {
      identity: face,
      material,
      indices: [],
    };
    group.indices.push(face.triangleIndex);
    grouped.set(groupKey, group);
  }
  return [...grouped.values()]
    .sort((left, right) => (
      left.identity.sceneId - right.identity.sceneId
      || left.identity.nodeId - right.identity.nodeId
      || left.identity.primitiveId - right.identity.primitiveId
      || left.material.localeCompare(right.material)
    ))
    .map((group) => ({
      selection: {
        sceneId: group.identity.sceneId,
        nodeId: group.identity.nodeId,
        primitiveId: group.identity.primitiveId,
        triangleRanges: compactTriangleIndices(group.indices),
      },
      authoredMaterialId: group.material,
    }));
}

function commit(
  state: MaterialSeparationEditorStateV2,
  document: ModelMaterialSeparationDocumentV2,
): MaterialSeparationEditorStateV2 {
  if (equal(state.draft, document)) return state;
  return {
    ...state,
    draft: clone(document),
    past: [...state.past, clone(state.draft)].slice(-MATERIAL_SEPARATION_V2_HISTORY_LIMIT),
    future: [],
  };
}

export function materialSeparationReducerV2(
  state: MaterialSeparationEditorStateV2,
  action: MaterialSeparationActionV2,
): MaterialSeparationEditorStateV2 {
  switch (action.type) {
    case "SET_SELECTION_MODE":
      return { ...state, selectionMode: action.mode };
    case "SELECT_COMPONENT": {
      const selected = action.additive
        ? state.selectedComponentKeys.includes(action.key)
          ? state.selectedComponentKeys.filter((key) => key !== action.key)
          : [...state.selectedComponentKeys, action.key]
        : [action.key];
      return { ...state, selectionMode: "COMPONENT", selectedComponentKeys: selected };
    }
    case "SELECT_FACE": {
      const selected = action.additive
        ? state.selectedFaceKeys.includes(action.key)
          ? state.selectedFaceKeys.filter((key) => key !== action.key)
          : [...state.selectedFaceKeys, action.key]
        : [action.key];
      return { ...state, selectionMode: "FACE", selectedFaceKeys: selected };
    }
    case "SELECT_FACES": {
      const selected = action.additive
        ? [...new Set([...state.selectedFaceKeys, ...action.keys])]
        : [...new Set(action.keys)];
      return { ...state, selectionMode: "FACE", selectedFaceKeys: selected };
    }
    case "SELECT_NODE":
      return {
        ...state,
        selectionMode: "COMPONENT",
        selectedComponentKeys: action.inventory.components
          .filter((component) => component.key.nodeId === action.nodeId)
          .map((component) => componentKey(component.key)),
      };
    case "SELECT_PRIMITIVE":
      return {
        ...state,
        selectionMode: "COMPONENT",
        selectedComponentKeys: action.inventory.components
          .filter((component) => component.key.nodeId === action.nodeId
            && component.key.primitiveId === action.primitiveId)
          .map((component) => componentKey(component.key)),
      };
    case "SELECT_BY_MATERIAL": {
      const components = state.draft.componentAssignments
        .filter((assignment) => assignment.authoredMaterialId === action.authoredMaterialId)
        .map((assignment) => componentKey(assignment.component));
      const faces = [...faceAssignmentMap(state.draft.faceAssignments)]
        .filter(([, material]) => material === action.authoredMaterialId)
        .map(([key]) => key);
      return {
        ...state,
        selectionMode: faces.length ? "FACE" : "COMPONENT",
        selectedComponentKeys: components,
        selectedFaceKeys: faces,
      };
    }
    case "SELECT_UNASSIGNED_COMPONENTS": {
      const assigned = new Set(state.draft.componentAssignments.map((assignment) => componentKey(assignment.component)));
      return {
        ...state,
        selectionMode: "COMPONENT",
        selectedComponentKeys: action.inventory.components
          .map((component) => componentKey(component.key))
          .filter((key) => !assigned.has(key)),
      };
    }
    case "CLEAR_SELECTION":
      return { ...state, selectedComponentKeys: [], selectedFaceKeys: [] };
    case "NEW_MATERIAL":
      if (state.draft.materials.some((material) => material.authoredMaterialId === action.material.authoredMaterialId)) return state;
      return commit(state, { ...state.draft, materials: [...state.draft.materials, clone(action.material)] });
    case "RENAME_MATERIAL":
      return commit(state, {
        ...state.draft,
        materials: state.draft.materials.map((material) => (
          material.authoredMaterialId === action.authoredMaterialId
            ? { ...material, displayName: action.displayName }
            : material
        )),
      });
    case "DELETE_UNUSED": {
      const used = new Set([
        ...state.draft.componentAssignments.map((assignment) => assignment.authoredMaterialId),
        ...state.draft.faceAssignments.map((assignment) => assignment.authoredMaterialId),
      ]);
      return commit(state, {
        ...state.draft,
        materials: state.draft.materials.filter((material) => used.has(material.authoredMaterialId)),
      });
    }
    case "ASSIGN_SELECTION": {
      if (!state.draft.materials.some((material) => material.authoredMaterialId === action.authoredMaterialId)) return state;
      if (state.selectionMode === "FACE") {
        const byFace = faceAssignmentMap(state.draft.faceAssignments);
        state.selectedFaceKeys.forEach((key) => byFace.set(key, action.authoredMaterialId));
        return commit(state, { ...state.draft, faceAssignments: compactFaceAssignmentsV2(byFace) });
      }
      const selected = new Set(state.selectedComponentKeys);
      const retained = state.draft.componentAssignments.filter((assignment) => !selected.has(componentKey(assignment.component)));
      const added = state.selectedComponentKeys.map((key) => ({
        component: parseComponentKey(key),
        authoredMaterialId: action.authoredMaterialId,
      }));
      return commit(state, {
        ...state.draft,
        componentAssignments: sortComponentAssignments([...retained, ...added]),
      });
    }
    case "UNASSIGN_SELECTION": {
      if (state.selectionMode === "FACE") {
        const selected = new Set(state.selectedFaceKeys);
        const byFace = faceAssignmentMap(state.draft.faceAssignments);
        selected.forEach((key) => byFace.delete(key));
        return commit(state, { ...state.draft, faceAssignments: compactFaceAssignmentsV2(byFace) });
      }
      const selected = new Set(state.selectedComponentKeys);
      return commit(state, {
        ...state.draft,
        componentAssignments: state.draft.componentAssignments
          .filter((assignment) => !selected.has(componentKey(assignment.component))),
      });
    }
    case "TOGGLE_OVERLAY":
      return { ...state, overlayEnabled: !state.overlayEnabled };
    case "TOGGLE_ISOLATE":
      return { ...state, isolateSelection: !state.isolateSelection };
    case "UNDO": {
      const previous = state.past.at(-1);
      if (!previous) return state;
      return {
        ...state,
        draft: clone(previous),
        past: state.past.slice(0, -1),
        future: [clone(state.draft), ...state.future].slice(0, MATERIAL_SEPARATION_V2_HISTORY_LIMIT),
      };
    }
    case "REDO": {
      const next = state.future[0];
      if (!next) return state;
      return {
        ...state,
        draft: clone(next),
        past: [...state.past, clone(state.draft)].slice(-MATERIAL_SEPARATION_V2_HISTORY_LIMIT),
        future: state.future.slice(1),
      };
    }
    case "APPLY":
      return { ...state, applied: clone(state.draft), past: [], future: [] };
    case "CANCEL":
      return { ...state, draft: clone(state.applied), past: [], future: [] };
    case "RESET":
      return commit(state, clone(state.initial));
  }
}
