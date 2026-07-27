export type PlaceableElementKind =
  | "SOURCE_NODE"
  | "SOURCE_COMPONENT"
  | "COPY"
  | "GROUP";

export interface PlaceableElementSource {
  readonly nodeId: number;
  readonly primitiveId: number | null;
  readonly componentIndex: number | null;
}

export interface PlaceableElementTransform {
  readonly translation: [number, number, number];
  readonly rotationXyzw: [number, number, number, number];
  readonly scale: [number, number, number];
  readonly pivot: [number, number, number];
}

export interface PlaceableElementFlags {
  readonly hidden: boolean;
  readonly locked: boolean;
  readonly renderable: boolean;
  readonly includeInCollision: boolean;
  readonly castShadow: boolean;
}

export interface PlaceableAuthoringElement {
  readonly id: string;
  readonly name: string;
  readonly kind: PlaceableElementKind;
  readonly source: PlaceableElementSource | null;
  readonly parentId: string | null;
  readonly transform: PlaceableElementTransform;
  readonly flags: PlaceableElementFlags;
  readonly deleted: boolean;
}

export interface PlaceableAuthoringDocument {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly elements: readonly PlaceableAuthoringElement[];
}

export interface PlaceableComponentInspection {
  readonly elementId: string;
  readonly componentIndex: number;
  readonly triangleCount: number;
  readonly vertexCount: number;
  readonly boundsMin: [number, number, number];
  readonly boundsMax: [number, number, number];
}

export interface PlaceablePrimitiveInspection {
  readonly primitiveId: number;
  readonly materialId: number | null;
  readonly triangleCount: number;
  readonly components: readonly PlaceableComponentInspection[];
}

export interface PlaceableNodeInspection {
  readonly elementId: string;
  readonly nodeId: number;
  readonly name: string;
  readonly meshId: number;
  readonly primitives: readonly PlaceablePrimitiveInspection[];
}

export interface PlaceableElementInspection {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly renderNodeCount: number;
  readonly primitiveCount: number;
  readonly connectedComponentCount: number;
  readonly nodes: readonly PlaceableNodeInspection[];
}

export interface PlaceableAuthoringBootstrap {
  readonly schemaVersion: 1;
  readonly inspection: PlaceableElementInspection;
  readonly document: PlaceableAuthoringDocument;
}

export type TransformTool = "TRANSLATE" | "ROTATE" | "SCALE";
export type TransformSpace = "LOCAL" | "WORLD";
export type PreviewMode = "ORIGINAL" | "EDITED";
export type SnapMode = "NONE" | "GRID" | "SURFACE" | "VERTEX" | "ELEMENT";

export interface PlaceableSnapSettings {
  readonly mode: SnapMode;
  readonly translationStep: number;
  readonly rotationDegrees: number;
  readonly scaleStep: number;
}

export interface PlaceableAuthoringEditorState {
  readonly original: PlaceableAuthoringDocument;
  readonly present: PlaceableAuthoringDocument;
  readonly past: readonly PlaceableAuthoringDocument[];
  readonly future: readonly PlaceableAuthoringDocument[];
  readonly selectedIds: readonly string[];
  readonly primaryId: string | null;
  readonly tool: TransformTool;
  readonly space: TransformSpace;
  readonly preview: PreviewMode;
  readonly snap: PlaceableSnapSettings;
  readonly transformClipboard: PlaceableElementTransform | null;
  readonly gestureOrigin: PlaceableAuthoringDocument | null;
}

export interface ElementTransformPatch {
  readonly id: string;
  readonly transform: PlaceableElementTransform;
}

export type PlaceableAuthoringAction =
  | { readonly type: "SELECT"; readonly id: string; readonly additive?: boolean }
  | { readonly type: "SELECT_ALL" }
  | { readonly type: "CLEAR_SELECTION" }
  | { readonly type: "SET_TOOL"; readonly tool: TransformTool }
  | { readonly type: "SET_SPACE"; readonly space: TransformSpace }
  | { readonly type: "SET_PREVIEW"; readonly preview: PreviewMode }
  | { readonly type: "SET_SNAP"; readonly snap: Partial<PlaceableSnapSettings> }
  | { readonly type: "BEGIN_GESTURE" }
  | { readonly type: "PREVIEW_TRANSFORMS"; readonly patches: readonly ElementTransformPatch[] }
  | { readonly type: "END_GESTURE" }
  | { readonly type: "SET_TRANSFORMS"; readonly patches: readonly ElementTransformPatch[] }
  | {
      readonly type: "PATCH_FLAGS";
      readonly ids: readonly string[];
      readonly flags: Partial<PlaceableElementFlags>;
    }
  | { readonly type: "SPLIT_NODE_COMPONENTS"; readonly id: string; readonly inspection: PlaceableElementInspection }
  | { readonly type: "DUPLICATE" }
  | { readonly type: "DELETE" }
  | { readonly type: "GROUP" }
  | { readonly type: "UNGROUP" }
  | { readonly type: "PARENT"; readonly parentId: string }
  | { readonly type: "UNPARENT" }
  | { readonly type: "MIRROR"; readonly axis: 0 | 1 | 2 }
  | { readonly type: "COPY_TRANSFORM" }
  | { readonly type: "PASTE_TRANSFORM" }
  | { readonly type: "SET_PIVOT"; readonly pivot: [number, number, number] }
  | { readonly type: "SET_ORIGIN"; readonly pivot: [number, number, number] }
  | { readonly type: "HIDE_SELECTED"; readonly hidden: boolean }
  | { readonly type: "ISOLATE_SELECTION" }
  | { readonly type: "SHOW_ALL" }
  | { readonly type: "LOCK_SELECTED"; readonly locked: boolean }
  | { readonly type: "ALIGN"; readonly axis: 0 | 1 | 2; readonly mode: "MIN" | "CENTER" | "MAX" }
  | { readonly type: "GROUND"; readonly boundsById: Readonly<Record<string, readonly [number, number]>> }
  | { readonly type: "UNDO" }
  | { readonly type: "REDO" }
  | { readonly type: "RESET" };

export function parsePlaceableAuthoringBootstrap(json: string): PlaceableAuthoringBootstrap {
  const value = JSON.parse(json) as PlaceableAuthoringBootstrap;
  if (
    value.schemaVersion !== 1
    || value.document?.schemaVersion !== 1
    || value.inspection?.schemaVersion !== 1
    || value.document.sourceSha256 !== value.inspection.sourceSha256
    || !Array.isArray(value.document.elements)
    || !Array.isArray(value.inspection.nodes)
  ) {
    throw new Error("PLACEABLE-AUTHORING-BOOTSTRAP-INVALID");
  }
  return value;
}
