export type ModelPackageLaneV1 =
  | "H1_SKINNED_FULL_42"
  | "H1_SKINNED_FULL_42_EVENTS"
  | "H1_SKINNED_FULL_42_AUTHORED"
  | "H1_SKINNED_FULL_42_EDITED"
  | "SKINNED_PROCEDURAL_HUMANOID_42"
  | "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT"
  | "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT"
  | "M0_STATIC_RIGID";

export type SkinAccessoryStabilizationOptionsV1 = {
  mode: "AUTO" | "KEEP_SOURCE_WEIGHTS" | "SELECT_BONE";
  selectedBoneName?: string;
};

export type StudioWorkerRequest =
  | { requestId: string; type: "INITIALIZE" }
  | {
      requestId: string;
      type: "INSPECT_SOURCE";
      sourceGlb: ArrayBuffer;
      target?: "CREATURE" | "PLACEABLE" | "TILE";
      creatureProfile?: "PRODUCT_300K" | "EXPERIMENTAL_P100K" | "EXPERIMENTAL_P300K";
    }
  | { requestId: string; type: "INSPECT_APPEARANCE"; appearanceTwoDa: ArrayBuffer }
  | {
      requestId: string;
      type: "VALIDATE_CREATURE_ANIMATION_MAPPING";
      animationAuthoringJson: string;
    }
  | {
      requestId: string;
      type: "INSPECT_EDITABLE_ANIMATION_SOURCE";
      sourceGlb: ArrayBuffer;
      clipName?: string;
    }
  | {
      requestId: string;
      type: "VALIDATE_ANIMATION_STUDIO_DOCUMENT";
      sourceGlb: ArrayBuffer;
      animationStudioDocumentJson: string;
    }
  | {
      requestId: string;
      type: "MATERIALIZE_ANIMATION_STUDIO_DOCUMENT";
      sourceGlb: ArrayBuffer;
      animationStudioDocumentJson: string;
    }
  | {
      requestId: string;
      type: "PREVIEW_AUTHORED_ANIMATION_CLIP";
      animationStudioDocumentJson: string;
      clipId: string;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "H1_SKINNED_FULL_42" | "M0_STATIC_RIGID";
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_42";
      identityJson: string;
      textureArtifactCleanup: boolean;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV1;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT";
      identityJson: string;
      textureArtifactCleanup: boolean;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV1;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT";
      identityJson: string;
      textureArtifactCleanup: boolean;
      skinAccessoryStabilization?: SkinAccessoryStabilizationOptionsV1;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "H1_SKINNED_FULL_42_EVENTS";
      eventAuthoringJson: string;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "H1_SKINNED_FULL_42_AUTHORED";
      animationAuthoringJson: string;
      projectIdentityJson: string;
      eventAuthoringJson?: string;
    }
  | {
      requestId: string;
      type: "BUILD_MODEL_PACKAGE";
      sourceGlb: ArrayBuffer;
      appearanceTwoDa: ArrayBuffer;
      packageLane: "H1_SKINNED_FULL_42_EDITED";
      animationAuthoringJson: string;
      animationStudioDocumentJson: string;
      projectIdentityJson: string;
      eventAuthoringJson?: string;
    }
  | {
      requestId: string;
      type: "BUILD_PLACEABLE_PACKAGE";
      sourceGlb: ArrayBuffer;
      placeablesTwoDa: ArrayBuffer;
      projectIdentityJson: string;
      placementJson: string;
      paletteId: number;
      authoringJson?: string;
    }
  | {
      requestId: string;
      type: "BUILD_TILE_PACKAGE";
      sourceGlb: ArrayBuffer;
      optionsJson: string;
    }
  | { requestId: string; type: "VALIDATE_M7_CORPUS"; manifestJson: string }
  | {
      requestId: string;
      type: "INSPECT_M7_CORPUS_INTAKE";
      manifestJson: string;
      payloadBlob: ArrayBuffer;
      descriptorsJson: string;
    }
  | {
      requestId: string;
      type: "BUILD_M7_CORPUS_BATCH";
      manifestJson: string;
      payloadBlob: ArrayBuffer;
      descriptorsJson: string;
    };

export interface WorkerArtifact {
  artifactId: string;
  kind: "HAK" | "MODEL" | "MODULE" | "WOK" | "SET" | "TEXTURE" | "JSON_REPORT";
  fileName: string;
  mediaType: string;
  byteLength: number;
  sha256: string;
  bytes: ArrayBuffer;
  provenance: "M2A_WASM_WORKER";
}

export type StudioWorkerSuccess =
  | { requestId: string; ok: true; type: "INITIALIZED" }
  | {
      requestId: string;
      ok: true;
      type: "SOURCE_INSPECTED";
      ingestJson: string;
      placeableAuthoringJson?: string;
    }
  | { requestId: string; ok: true; type: "APPEARANCE_INSPECTED"; inspectionJson: string }
  | {
      requestId: string;
      ok: true;
      type: "CREATURE_ANIMATION_MAPPING_VALIDATED";
      validationJson: string;
      resolutionJson: string;
      catalogJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "EDITABLE_ANIMATION_SOURCE_INSPECTED";
      inspectionJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "ANIMATION_STUDIO_DOCUMENT_VALIDATED";
      validationJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "ANIMATION_STUDIO_DOCUMENT_MATERIALIZED";
      materializationJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "AUTHORED_ANIMATION_CLIP_PREVIEWED";
      previewJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "MODEL_PACKAGE_BUILT";
      artifacts: WorkerArtifact[];
      reportJson: string;
      manifestJson: string;
      summaryJson: string;
      readbackJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "PLACEABLE_PACKAGE_BUILT";
      artifacts: WorkerArtifact[];
      reportJson: string;
      readbackJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "TILE_PACKAGE_BUILT";
      artifacts: WorkerArtifact[];
      reportJson: string;
      modelReadbackJson: string;
      wokReadbackJson: string;
      setReadbackJson: string;
    }
  | {
      requestId: string;
      ok: true;
      type: "M7_CORPUS_VALIDATED";
      manifestJson: string;
      artifacts: WorkerArtifact[];
    }
  | {
      requestId: string;
      ok: true;
      type: "M7_CORPUS_INTAKE_INSPECTED";
      intakeJson: string;
      artifacts: WorkerArtifact[];
    }
  | {
      requestId: string;
      ok: true;
      type: "M7_CORPUS_BATCH_BUILT";
      batchJson: string;
      artifacts: WorkerArtifact[];
    };

export interface StudioWorkerFailure {
  requestId: string;
  ok: false;
  type: "FAILED";
  message: string;
}

export type StudioWorkerResponse = StudioWorkerSuccess | StudioWorkerFailure;
