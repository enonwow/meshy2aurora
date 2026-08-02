import type { TileAuthoringOptions } from "../source/InputsPanel";
import type { StudioWorkerRequest } from "../../worker/types";

const PLACEABLE_PLACEMENT_V1 = {
  x: 10,
  y: 14.5,
  z: 0,
  bearing: 0,
} as const;

const EXPERIMENTAL_TILE_IDENTITY_V1 = {
  moduleResref: "m2atilestv1",
  moduleFileName: "m2a_tile_static_v1.mod",
  moduleDisplayName: "Meshy2Aurora Tile Static V1",
  areaResref: "m2atilearea",
  areaName: "M2A Tile Static 2x2",
  hakResref: "m2atilestv1",
  hakFileName: "m2a_tile_static_v1.hak",
  tilesetResref: "m2atilesetv1",
  modelResref: "m2atilemdl1",
  textureResref: "m2atiletex1",
  imageMapResref: "m2atilemap1",
} as const;

type TargetBuildRequestInputV1 =
  | {
      readonly target: "TILE";
      readonly requestId: string;
      readonly sourceGlb: ArrayBuffer;
      readonly tileOptions: TileAuthoringOptions;
    }
  | {
      readonly target: "PLACEABLE";
      readonly requestId: string;
      readonly sourceGlb: ArrayBuffer;
      readonly baseTwoDa: ArrayBuffer;
      readonly projectIdentityJson: string;
      readonly authoringJson?: string;
    }
  | {
      readonly target: "CREATURE";
      readonly requestId: string;
      readonly sourceGlb: ArrayBuffer;
      readonly baseTwoDa: ArrayBuffer;
      readonly projectIdentityJson: string;
      readonly packageLane:
        | "H1_SKINNED_FULL_42_AUTHORED"
        | "H1_SKINNED_FULL_42_EDITED";
      readonly animationAuthoringJson: string;
      readonly animationStudioDocumentJson?: string;
      readonly eventAuthoringJson?: string;
      readonly heldWeapon?: {
        readonly glb: ArrayBuffer;
        readonly filename: string;
        readonly rigJson: string;
        readonly primaryHand: "RIGHT" | "LEFT";
        readonly targetNodeId: number;
        readonly localTransformJson: string;
        readonly attachmentRevision: number;
      };
    };

export interface TargetBuildRequestV1 {
  readonly request: StudioWorkerRequest;
  readonly transfer: ArrayBuffer[];
}

export function createTargetBuildRequestV1(
  input: TargetBuildRequestInputV1,
): TargetBuildRequestV1 {
  if (input.target === "TILE") {
    return {
      request: {
        requestId: input.requestId,
        type: "BUILD_TILE_PACKAGE",
        sourceGlb: input.sourceGlb,
        optionsJson: JSON.stringify({
          schemaVersion: 1,
          identity: EXPERIMENTAL_TILE_IDENTITY_V1,
          interior: input.tileOptions.interior,
          terrainName: input.tileOptions.terrainName.trim(),
          surface: input.tileOptions.surface,
        }),
      },
      transfer: [input.sourceGlb],
    };
  }

  if (input.target === "PLACEABLE") {
    return {
      request: {
        requestId: input.requestId,
        type: "BUILD_PLACEABLE_PACKAGE",
        sourceGlb: input.sourceGlb,
        placeablesTwoDa: input.baseTwoDa,
        projectIdentityJson: input.projectIdentityJson,
        placementJson: JSON.stringify(PLACEABLE_PLACEMENT_V1),
        paletteId: 7,
        authoringJson: input.authoringJson,
      },
      transfer: [input.sourceGlb, input.baseTwoDa],
    };
  }

  return {
    request: input.packageLane === "H1_SKINNED_FULL_42_EDITED"
      ? {
          requestId: input.requestId,
          type: "BUILD_MODEL_PACKAGE",
          sourceGlb: input.sourceGlb,
          appearanceTwoDa: input.baseTwoDa,
          packageLane: input.packageLane,
          animationAuthoringJson: input.animationAuthoringJson,
          animationStudioDocumentJson:
            input.animationStudioDocumentJson ?? "",
          projectIdentityJson: input.projectIdentityJson,
          eventAuthoringJson: input.eventAuthoringJson,
          heldWeapon: input.heldWeapon,
        }
      : {
          requestId: input.requestId,
          type: "BUILD_MODEL_PACKAGE",
          sourceGlb: input.sourceGlb,
          appearanceTwoDa: input.baseTwoDa,
          packageLane: input.packageLane,
          animationAuthoringJson: input.animationAuthoringJson,
          projectIdentityJson: input.projectIdentityJson,
          eventAuthoringJson: input.eventAuthoringJson,
        },
    transfer: input.heldWeapon
      ? [input.sourceGlb, input.baseTwoDa, input.heldWeapon.glb]
      : [input.sourceGlb, input.baseTwoDa],
  };
}
