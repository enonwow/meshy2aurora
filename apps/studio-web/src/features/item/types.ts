import type { StudioWorkerRequest, StudioWorkerResponse, WorkerArtifact } from "../../worker/types";

export interface ItemPartSlot {
  readonly index: number;
  readonly field: string;
  readonly label: string;
  readonly token: string | null;
  readonly sourceKind: "MESHY_GLB" | "CAPART_SELECTION" | "CLOAK_MODEL_SELECTION";
  readonly referenceTable: string | null;
  readonly requiresExplicitResourceResrefs: boolean;
}

export type ItemCompositionProfile =
  | "SINGLE_PART"
  | "BOTTOM_MIDDLE_TOP"
  | "CAPART_ARMOR"
  | "CLOAK_MODEL";

export type ItemTextureProfile =
  | "DIRECT_COLOR"
  | "PALETTE_LAYERS"
  | "CAPART_PALETTE_LAYERS";

export type ItemIconProfile =
  | "STANDARD"
  | "LAYERED"
  | "IPRP_SPELL"
  | "CAPART_COMPOSITE"
  | "CLOAK_MODEL";

export interface ItemCapability {
  readonly schemaVersion: 1;
  readonly compositionProfile: ItemCompositionProfile;
  readonly textureProfile: ItemTextureProfile;
  readonly iconProfile: ItemIconProfile;
  readonly meshySourceCount: number;
  readonly requiredReferenceTables: readonly string[];
}

export interface ItemBaseItemRow {
  readonly schemaVersion: 1;
  readonly baseItem: number;
  readonly label: string;
  readonly itemClass: string;
  readonly modelType: 0 | 1 | 2 | 3;
  readonly genderSpecific: boolean;
  readonly defaultModel: string | null;
  readonly defaultIcon: string | null;
  readonly equipableSlots: number;
  readonly invSlotWidth: number;
  readonly invSlotHeight: number;
  readonly capability: ItemCapability;
  readonly partSlots: readonly ItemPartSlot[];
  readonly colorFields: readonly string[];
}

export interface ItemBaseItemsCatalog {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly physicalRowCount: number;
  readonly inactiveRowCount: number;
  readonly rows: readonly ItemBaseItemRow[];
}

export interface ItemPartDraft {
  readonly field: string;
  readonly label: string;
  readonly token: string | null;
  readonly sourceKind: "MESHY_GLB" | "CAPART_SELECTION" | "CLOAK_MODEL_SELECTION";
  readonly referenceTable: string | null;
  readonly requiresExplicitResourceResrefs: boolean;
  readonly file?: File;
  readonly sourceNode: string;
  readonly textureEncoding:
    | "DIRECT_COLOR"
    | "PLT_METAL1"
    | "PLT_METAL2"
    | "PLT_CLOTH1"
    | "PLT_CLOTH2"
    | "PLT_LEATHER1"
    | "PLT_LEATHER2";
  readonly variant: number;
  readonly explicitModelResref: string;
  readonly explicitIconResref: string;
  readonly translation: readonly [number, number, number];
  readonly rotationDegrees: readonly [number, number, number];
  readonly uniformScale: number;
  readonly pivot: readonly [number, number, number];
}

export interface ResolvedItemPartDraft extends ItemPartDraft {
  readonly modelResref: string;
  readonly iconResref: string;
  readonly textureResref: string;
}

export interface ItemBuildSnapshot {
  readonly report: {
    readonly status: string;
    readonly baseItem: number;
    readonly partCount: number;
    readonly meshyPartCount: number;
    readonly referenceSelectorCount: number;
    readonly iconLayerCount: number;
    readonly iconLayerMode:
      | "GEOMETRY_RASTER_TGA_V2"
      | "IPRP_SPELLS_ICON_RESOLUTION"
      | "RETAIL_CAPART_COMPOSITION_PLAN_V1"
      | "RETAIL_CLOAKMODEL_ICON";
    readonly iconRuntimeParity: "offline_semantic_readback_only";
    readonly triangleBudget: {
      readonly triangleCount: number;
      readonly triangleBudget: number;
      readonly warning: boolean;
    };
    readonly seamValidation: {
      readonly tolerance: number;
      readonly status: "PASSED";
      readonly results: readonly {
        readonly firstField: string;
        readonly secondField: string;
        readonly status: "TOUCHING";
        readonly gap: number;
        readonly overlap: false;
        readonly algorithm: "TRIANGLE_SURFACE_BVH_CONTAINMENT_V1";
        readonly firstSourceSha256: string;
        readonly secondSourceSha256: string;
        readonly firstTransformSha256: string;
        readonly secondTransformSha256: string;
        readonly firstTriangleCount: number;
        readonly secondTriangleCount: number;
        readonly measurementSha256: string;
      }[];
    };
    readonly modelVisibility: "not_tested";
    readonly proofCompleteness: "missing";
    readonly readyForOwnerProof: false;
    readonly proofBlocker: string;
  };
  readonly partReadbacks: readonly {
    readonly field: string;
    readonly variant: number;
    readonly modelResref: string;
    readonly readback: {
      readonly model?: { readonly name?: string };
      readonly nodeTree?: { readonly nodeCount?: number };
      readonly diagnostics?: readonly unknown[];
    };
  }[];
  readonly utiReport: {
    readonly baseItem: number;
    readonly modelType: number;
    readonly partCount: number;
    readonly colorFieldCount: number;
    readonly semanticReadbackStatus: string;
  };
  readonly artifacts: WorkerArtifact[];
}

export interface ItemWorkerClient {
  request(
    request: StudioWorkerRequest,
    transfer?: Transferable[],
  ): Promise<StudioWorkerResponse>;
  dispose?(): void;
}
