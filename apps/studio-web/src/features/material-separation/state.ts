import type {
  AuthoredMaterialV1,
  ModelComponentInventoryV1,
  ModelMaterialAssignmentV1,
  ModelMaterialSeparationDocumentV1,
} from "./types";
import { componentKey, parseComponentKey } from "./types";

export const MATERIAL_SEPARATION_HISTORY_LIMIT = 50;

export interface MaterialSeparationEditorState {
  readonly initial: ModelMaterialSeparationDocumentV1;
  readonly applied: ModelMaterialSeparationDocumentV1;
  readonly draft: ModelMaterialSeparationDocumentV1;
  readonly past: readonly ModelMaterialSeparationDocumentV1[];
  readonly future: readonly ModelMaterialSeparationDocumentV1[];
  readonly selectedComponentKeys: readonly string[];
  readonly overlayEnabled: boolean;
  readonly isolateSelection: boolean;
}

export type MaterialSeparationAction =
  | { readonly type: "SELECT_COMPONENT"; readonly key: string; readonly additive?: boolean }
  | { readonly type: "SELECT_NODE"; readonly nodeId: number; readonly inventory: ModelComponentInventoryV1 }
  | { readonly type: "SELECT_PRIMITIVE"; readonly nodeId: number; readonly primitiveId: number; readonly inventory: ModelComponentInventoryV1 }
  | { readonly type: "SELECT_BY_MATERIAL"; readonly authoredMaterialId: string }
  | { readonly type: "SELECT_UNASSIGNED"; readonly inventory: ModelComponentInventoryV1 }
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

export function createMaterialSeparationEditorState(
  document: ModelMaterialSeparationDocumentV1,
): MaterialSeparationEditorState {
  return {
    initial: clone(document),
    applied: clone(document),
    draft: clone(document),
    past: [],
    future: [],
    selectedComponentKeys: [],
    overlayEnabled: true,
    isolateSelection: false,
  };
}

function sortAssignments(assignments: readonly ModelMaterialAssignmentV1[]) {
  return [...assignments].sort((left, right) => (
    componentKey(left.component).localeCompare(componentKey(right.component))
      || left.authoredMaterialId.localeCompare(right.authoredMaterialId)
  ));
}

function commit(
  state: MaterialSeparationEditorState,
  document: ModelMaterialSeparationDocumentV1,
): MaterialSeparationEditorState {
  if (equal(state.draft, document)) return state;
  return {
    ...state,
    draft: clone(document),
    past: [...state.past, clone(state.draft)].slice(-MATERIAL_SEPARATION_HISTORY_LIMIT),
    future: [],
  };
}

export function materialSeparationReducer(
  state: MaterialSeparationEditorState,
  action: MaterialSeparationAction,
): MaterialSeparationEditorState {
  switch (action.type) {
    case "SELECT_COMPONENT": {
      const selected = action.additive
        ? state.selectedComponentKeys.includes(action.key)
          ? state.selectedComponentKeys.filter((key) => key !== action.key)
          : [...state.selectedComponentKeys, action.key]
        : [action.key];
      return { ...state, selectedComponentKeys: selected };
    }
    case "SELECT_NODE":
      return {
        ...state,
        selectedComponentKeys: action.inventory.components
          .filter((component) => component.key.nodeId === action.nodeId)
          .map((component) => componentKey(component.key)),
      };
    case "SELECT_PRIMITIVE":
      return {
        ...state,
        selectedComponentKeys: action.inventory.components
          .filter((component) => component.key.nodeId === action.nodeId
            && component.key.primitiveId === action.primitiveId)
          .map((component) => componentKey(component.key)),
      };
    case "SELECT_BY_MATERIAL": {
      const selected = state.draft.assignments
        .filter((assignment) => assignment.authoredMaterialId === action.authoredMaterialId)
        .map((assignment) => componentKey(assignment.component));
      return { ...state, selectedComponentKeys: selected };
    }
    case "SELECT_UNASSIGNED": {
      const assigned = new Set(state.draft.assignments.map((assignment) => componentKey(assignment.component)));
      return {
        ...state,
        selectedComponentKeys: action.inventory.components
          .map((component) => componentKey(component.key))
          .filter((key) => !assigned.has(key)),
      };
    }
    case "CLEAR_SELECTION":
      return { ...state, selectedComponentKeys: [] };
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
      const used = new Set(state.draft.assignments.map((assignment) => assignment.authoredMaterialId));
      return commit(state, {
        ...state.draft,
        materials: state.draft.materials.filter((material) => used.has(material.authoredMaterialId)),
      });
    }
    case "ASSIGN_SELECTION": {
      if (!state.draft.materials.some((material) => material.authoredMaterialId === action.authoredMaterialId)) return state;
      const selected = new Set(state.selectedComponentKeys);
      const retained = state.draft.assignments.filter((assignment) => !selected.has(componentKey(assignment.component)));
      const added = state.selectedComponentKeys.map((key) => ({
        component: parseComponentKey(key),
        authoredMaterialId: action.authoredMaterialId,
      }));
      return commit(state, { ...state.draft, assignments: sortAssignments([...retained, ...added]) });
    }
    case "UNASSIGN_SELECTION": {
      const selected = new Set(state.selectedComponentKeys);
      return commit(state, {
        ...state.draft,
        assignments: state.draft.assignments.filter((assignment) => !selected.has(componentKey(assignment.component))),
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
        future: [clone(state.draft), ...state.future].slice(0, MATERIAL_SEPARATION_HISTORY_LIMIT),
      };
    }
    case "REDO": {
      const next = state.future[0];
      if (!next) return state;
      return {
        ...state,
        draft: clone(next),
        past: [...state.past, clone(state.draft)].slice(-MATERIAL_SEPARATION_HISTORY_LIMIT),
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
