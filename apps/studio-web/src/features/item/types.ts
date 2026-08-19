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
  readonly minRange: number | null;
  readonly maxRange: number | null;
  readonly genderSpecific: boolean;
  readonly defaultModel: string | null;
  readonly defaultIcon: string | null;
  readonly equipableSlots: number;
  readonly invSlotWidth: number;
  readonly invSlotHeight: number;
  readonly weaponWield: number | null;
  readonly weaponType: number | null;
  readonly rangedWeapon: number | null;
  readonly ammunitionType: number | null;
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
  /** ModelType 2 model selector. Null for non-weapon composition profiles. */
  readonly weaponModel: number | null;
  /** ModelType 2 native color selector (1..4). Null for other profiles. */
  readonly weaponColor: number | null;
  readonly explicitModelResref: string;
  readonly explicitIconResref: string;
  readonly translation: readonly [number, number, number];
  readonly rotationDegrees: readonly [number, number, number];
  readonly rotationXyzw: readonly [number, number, number, number];
  readonly uniformScale: number;
  readonly pivot: readonly [number, number, number];
  readonly targetSpaceScaleXyz: readonly [number, number, number];
}

export interface ResolvedItemPartDraft extends ItemPartDraft {
  readonly modelResref: string;
  readonly iconResref: string;
  readonly textureResref: string;
  readonly weaponColorways: readonly ItemWeaponColorwayResource[];
}

export interface ItemWeaponColorwayResource {
  readonly color: 1 | 2 | 3 | 4;
  readonly variant: number;
  readonly modelResref: string;
  readonly iconResref: string;
  readonly textureResref: string;
}

export interface ItemFitReportV3 {
  readonly schemaVersion: 3;
  readonly algorithm: "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX";
  readonly status: "PASSED" | "MANUAL_REQUIRED";
  readonly tolerance: number;
  readonly iterations: number;
  readonly solutionSha256: string;
  readonly orientationFrame: {
    readonly sourceAxialAxis: 0 | 1 | 2;
    readonly sourceWidthAxis: 0 | 1 | 2;
    readonly sourceDepthAxis: 0 | 1 | 2;
    readonly targetAxialAxis: 0 | 1 | 2;
    readonly targetWidthAxis: 0 | 1 | 2;
    readonly targetDepthAxis: 0 | 1 | 2;
    readonly widthToDepthRatio: number;
    readonly handednessDeterminant: number;
    readonly evidence: "GROUP_NORMALIZED_TRANSVERSE_EXTENTS_WITH_PROPER_HANDEDNESS_V1";
    readonly status: "PASSED" | "MANUAL_REQUIRED";
  };
  readonly parts: readonly {
    readonly field: string;
    readonly sourceSha256: string;
    readonly sourceNode: string | null;
    readonly triangleCount: number;
    readonly inputBoundsMin: readonly [number, number, number];
    readonly inputBoundsMax: readonly [number, number, number];
    readonly axialSourceAxis: 0 | 1 | 2;
    readonly axialTargetAxis: 0 | 1 | 2;
    readonly targetAxialLength: number;
    readonly transform: {
      readonly translation: readonly [number, number, number];
      readonly rotationXyzw: readonly [number, number, number, number];
      readonly uniformScale: number;
      readonly pivot: readonly [number, number, number];
    };
    readonly targetSpaceScaleXyz: readonly [number, number, number];
    readonly transformSha256: string;
    readonly outputBoundsMin: readonly [number, number, number];
    readonly outputBoundsMax: readonly [number, number, number];
    readonly bottomConnector: null | {
      readonly kind: "BOTTOM";
      readonly axialAxis: 0 | 1 | 2;
      readonly position: readonly [number, number, number];
    };
    readonly topConnector: null | {
      readonly kind: "TOP";
      readonly axialAxis: 0 | 1 | 2;
      readonly position: readonly [number, number, number];
    };
  }[];
  readonly adjacentSeams: readonly {
    readonly firstField: string;
    readonly secondField: string;
    readonly status: "TOUCHING" | "GAP" | "OVERLAP";
    readonly gap: number;
    readonly overlap: boolean;
    readonly measurementSha256: string;
  }[];
  readonly adjacentConnectors: readonly {
    readonly firstField: string;
    readonly firstConnector: "TOP";
    readonly secondField: string;
    readonly secondConnector: "BOTTOM";
    readonly axialAxis: 0 | 1 | 2;
    readonly axialOverlap: number;
    readonly requiredMinOverlap: number;
    readonly requiredMaxOverlap: number;
    readonly surfaceStatus: "TOUCHING" | "OVERLAP";
    readonly status: "OVERLAPPING" | "FAILED";
  }[];
  readonly nonAdjacentMeasurements: readonly {
    readonly firstField: string;
    readonly secondField: string;
    readonly status: "TOUCHING" | "GAP" | "OVERLAP";
    readonly gap: number;
    readonly overlap: boolean;
    readonly measurementSha256: string;
  }[];
}

export interface ItemReferenceSlotFrameV1 {
  readonly field: string;
  readonly label: string;
  readonly token: string;
  readonly modelResref: string;
  readonly modelSha256: string;
  readonly controllerNodeName: string;
  readonly controllerTranslation: readonly [number, number, number];
  readonly controllerRotationXyzw: readonly [number, number, number, number];
  readonly boundsMin: readonly [number, number, number];
  readonly boundsMax: readonly [number, number, number];
  readonly allowAxialExtensionAtMin: boolean;
  readonly allowAxialExtensionAtMax: boolean;
}

export interface ItemAttachmentProfileV1 {
  readonly schemaVersion: 1;
  readonly algorithm: "AURORA_ITEM_REFERENCE_PROFILE_V1";
  readonly status: "PASSED";
  readonly identity: {
    readonly schemaVersion: 1;
    readonly resourceContextSha256: string;
    readonly baseitemsSha256: string;
    readonly baseItem: number;
    readonly itemClass: string;
    readonly modelType: number;
    readonly referenceKind: string;
    readonly referenceId: string;
  };
  readonly attachmentRoute: "NONE" | "HAND" | "CAPART" | "CLOAK";
  readonly equipableSlots: number;
  readonly commonOrigin: readonly [number, number, number];
  readonly axialAxis: 0 | 1 | 2;
  readonly widthAxis: 0 | 1 | 2;
  readonly depthAxis: 0 | 1 | 2;
  readonly attachmentZoneMin: readonly [number, number, number];
  readonly attachmentZoneMax: readonly [number, number, number];
  readonly attachmentEvidence:
    | "ORIGIN_CONTAINING_REFERENCE_PARTS_V1"
    | "AUTHOR_MANUAL_ALIGNMENT_V1";
  readonly slots: readonly ItemReferenceSlotFrameV1[];
  readonly profileSha256: string;
}

export interface ItemFitReportV4 extends Omit<ItemFitReportV3,
  "schemaVersion" | "algorithm"
> {
  readonly schemaVersion: 4;
  readonly algorithm: "ITEM_REFERENCE_SLOT_FRAME_FIT_V1" | "ITEM_REFERENCE_MANUAL_FIT_V2";
  readonly referenceProfileSha256: string;
  readonly commonOrigin: readonly [number, number, number];
}

export type ItemFitReport = ItemFitReportV3 | ItemFitReportV4;

export interface ItemBuildSnapshot {
  readonly report: {
    readonly status: string;
    readonly baseItem: number;
    readonly partCount: number;
    readonly meshyPartCount: number;
    readonly referenceSelectorCount: number;
    readonly iconLayerCount: number;
    readonly iconLayerMode:
      | "AURORA_MODELTYPE2_ICON_LAYERS_V3"
      | "IPRP_SPELLS_ICON_RESOLUTION"
      | "RETAIL_CAPART_COMPOSITION_PLAN_V1"
      | "RETAIL_CLOAKMODEL_ICON";
    readonly iconRuntimeParity:
      | "offline_semantic_readback_only"
      | "offline_native_layer_composite_validated";
    readonly weaponColorwayCoverage: {
      readonly status: "COMPLETE" | "NOT_APPLICABLE";
      readonly expectedResourceCount: number;
      readonly emittedResourceCount: number;
      readonly colors: readonly number[];
      readonly geometryReuse: "ONE_MESHY_GLB_PER_PART" | null;
    };
    readonly itemPropertiesModelConformance: {
      readonly status: "PASSED" | "NOT_APPLICABLE";
      readonly algorithm: "ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V2" | null;
      readonly axialTargetAxis: 0 | 1 | 2 | null;
      readonly checkedMdlCount: number;
      readonly appendOrder: readonly string[];
      readonly colorways: readonly {
        readonly color: number;
        readonly compositeBoundsMin: readonly [number, number, number];
        readonly compositeBoundsMax: readonly [number, number, number];
        readonly totalTriangleCount: number;
        readonly parts: readonly {
          readonly field: string;
          readonly rootControllerOwner: "NONE";
          readonly transformControllerOwner: "TRIMESH_CHILD";
          readonly meshNodeNames: readonly string[];
        }[];
      }[];
    };
    readonly itemIconConformance: {
      readonly status: "PASSED" | "NOT_APPLICABLE";
      readonly algorithm: "AURORA_MODELTYPE2_ICON_LAYER_COMPOSITE_V3" | null;
      readonly layoutProfile: "LONG_VERTICAL_PART_ORDER_V2" | null;
      readonly colorways: readonly {
        readonly color: number;
        readonly opaquePixelCount: number;
        readonly boundsMin: readonly [number, number];
        readonly boundsMaxExclusive: readonly [number, number];
        readonly axialFillRatio: number;
        readonly occupiedFillRatio: number;
        readonly partOrderStatus: "PASSED";
        readonly compositeRgbaSha256: string;
      }[];
    };
    readonly triangleBudget: {
      readonly triangleCount: number;
      readonly triangleBudget: number;
      readonly warning: boolean;
    };
    readonly seamValidation: {
      readonly tolerance: number;
      readonly status: "PASSED" | "NOT_APPLICABLE";
      readonly results: readonly {
        readonly firstField: string;
        readonly secondField: string;
        readonly status: "TOUCHING" | "OVERLAP";
        readonly gap: number;
        readonly overlap: boolean;
        readonly requiredRelation: "ADJACENT_CONNECTED" | "NON_ADJACENT_NO_OVERLAP";
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
    readonly rangedAmmunition?: {
      readonly status: "OFFLINE_RANGED_AMMUNITION_PASSED";
      readonly binding: {
        readonly weaponBaseItem: number;
        readonly ammoBaseItem: number;
        readonly ammunitionType: number;
        readonly damageRangedProjectile: number;
        readonly ammunitiontypesRow: number;
        readonly projectileModelResref: string;
        readonly runtimeClip: string;
      };
      readonly damageRoute: {
        readonly status: string;
        readonly damageTypeRow: number;
        readonly expectedLabel: string;
        readonly semanticReadbackStatus: string;
      };
      readonly ammunitiontypes: {
        readonly status: string;
        readonly firstRow: number;
        readonly lastRow: number;
        readonly semanticReadbackStatus: string;
      };
      readonly ammunitionItem: {
        readonly baseItem: number;
        readonly modelResref: string;
        readonly blueprintResref: string;
      };
      readonly runtimeValidation: {
        readonly status: "OWNER_PROOF_REQUIRED";
        readonly clip: string;
        readonly modelVisibility: "not_tested";
        readonly proofCompleteness: "missing";
      };
    } | null;
    readonly hakSha256: string;
    readonly moduleSha256: string;
    readonly customWeaponBaseItem?: {
      readonly schemaVersion: 3;
      readonly status: "APPENDED_STANDALONE_EXACT";
      readonly outputBaseItem: number;
      readonly label: string;
      readonly itemClass: string;
      readonly invSlotWidth: number;
      readonly invSlotHeight: number;
      readonly definitionSource: "EXPLICIT_COLUMN_ASSIGNMENTS";
      readonly sourceSha256: string;
      readonly outputSha256: string;
    } | null;
    readonly proofModule: {
      readonly schemaVersion: number;
      readonly fixtureProfile: string;
      readonly groundItemCount: number;
      readonly creatureCount?: number;
      readonly equippedItemCount?: number;
      readonly weaponBlueprintResref?: string;
      readonly ammunitionBlueprintResref?: string;
      readonly targetBlueprintResref?: string;
      readonly targetAppearanceRow?: number;
      readonly targetPosition?: readonly [number, number, number];
      readonly targetWalkRate?: number;
      readonly targetScriptsEmpty?: boolean;
      readonly outputSha256: string;
      readonly semanticReadbackStatus: string;
      readonly modelVisibility: "not_tested";
      readonly proofCompleteness: "missing";
    };
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
    readonly weaponColorSelectorCount: number;
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
