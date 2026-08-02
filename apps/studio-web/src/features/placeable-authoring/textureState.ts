import type {
  PlaceableTextureAuthoringDocument,
  PlaceableTextureBindingAuthoring,
  PlaceableTexturePreviewMode,
} from "./textureTypes";
import { sourceTextureBindingV1 } from "./textureTypes";

export interface PlaceableTextureEditorState {
  readonly original: PlaceableTextureAuthoringDocument;
  readonly present: PlaceableTextureAuthoringDocument;
  readonly past: readonly PlaceableTextureAuthoringDocument[];
  readonly future: readonly PlaceableTextureAuthoringDocument[];
  readonly preview: PlaceableTexturePreviewMode;
  readonly selectedMaterialSlot: number | null;
}

export type PlaceableTextureEditorAction =
  | { readonly type: "SELECT_MATERIAL"; readonly materialSlot: number }
  | { readonly type: "SET_PREVIEW"; readonly preview: PlaceableTexturePreviewMode }
  | { readonly type: "REPLACE"; readonly binding: PlaceableTextureBindingAuthoring }
  | { readonly type: "USE_SOURCE"; readonly materialSlot: number }
  | { readonly type: "RESET_MATERIAL"; readonly materialSlot: number }
  | { readonly type: "UNDO" }
  | { readonly type: "REDO" }
  | { readonly type: "RESET_ALL" };

export function createPlaceableTextureEditorState(
  document: PlaceableTextureAuthoringDocument,
): PlaceableTextureEditorState {
  return {
    original: document,
    present: document,
    past: [],
    future: [],
    preview: "EDITED",
    selectedMaterialSlot: document.bindings[0]?.materialSlot ?? null,
  };
}

function commit(
  state: PlaceableTextureEditorState,
  document: PlaceableTextureAuthoringDocument,
): PlaceableTextureEditorState {
  if (JSON.stringify(document) === JSON.stringify(state.present)) return state;
  return {
    ...state,
    present: document,
    past: [...state.past, state.present],
    future: [],
  };
}

function replaceBinding(
  document: PlaceableTextureAuthoringDocument,
  binding: PlaceableTextureBindingAuthoring,
): PlaceableTextureAuthoringDocument {
  return {
    ...document,
    bindings: document.bindings.map((candidate) => (
      candidate.materialSlot === binding.materialSlot ? binding : candidate
    )),
  };
}

export function placeableTextureEditorReducer(
  state: PlaceableTextureEditorState,
  action: PlaceableTextureEditorAction,
): PlaceableTextureEditorState {
  switch (action.type) {
    case "SELECT_MATERIAL":
      return { ...state, selectedMaterialSlot: action.materialSlot };
    case "SET_PREVIEW":
      return { ...state, preview: action.preview };
    case "REPLACE":
      return commit(state, replaceBinding(state.present, action.binding));
    case "USE_SOURCE": {
      const binding = state.present.bindings.find(
        (candidate) => candidate.materialSlot === action.materialSlot,
      );
      return binding
        ? commit(state, replaceBinding(state.present, sourceTextureBindingV1(binding)))
        : state;
    }
    case "RESET_MATERIAL": {
      const binding = state.original.bindings.find(
        (candidate) => candidate.materialSlot === action.materialSlot,
      );
      return binding ? commit(state, replaceBinding(state.present, binding)) : state;
    }
    case "UNDO": {
      const previous = state.past.at(-1);
      if (!previous) return state;
      return {
        ...state,
        present: previous,
        past: state.past.slice(0, -1),
        future: [state.present, ...state.future],
      };
    }
    case "REDO": {
      const next = state.future[0];
      if (!next) return state;
      return {
        ...state,
        present: next,
        past: [...state.past, state.present],
        future: state.future.slice(1),
      };
    }
    case "RESET_ALL":
      return commit(state, state.original);
    default: {
      const exhaustive: never = action;
      return exhaustive;
    }
  }
}
