import type {
  StudioWorkerRequest,
  StudioWorkerResponse,
} from "./types";

type Pending = {
  resolve: (response: StudioWorkerResponse) => void;
  reject: (error: Error) => void;
  lane: "MAIN" | "PREVIEW";
};

export class StudioWorkerClient {
  private readonly pending = new Map<string, Pending>();
  private readonly worker: Worker;
  private previewWorker: Worker | null = null;
  private animationPreviewRequestId: string | null = null;
  private animationLibraryRequestId: string | null = null;

  constructor() {
    this.worker = this.createWorker("MAIN");
  }

  private createWorker(lane: Pending["lane"]) {
    const worker = new Worker(
      new URL("./m2a.worker.ts", import.meta.url),
      { type: "module" },
    );
    worker.addEventListener("message", (event: MessageEvent<StudioWorkerResponse>) => {
      const pending = this.pending.get(event.data.requestId);
      if (!pending || pending.lane !== lane) return;
      this.pending.delete(event.data.requestId);
      if (event.data.ok) pending.resolve(event.data);
      else pending.reject(new Error(event.data.message));
    });
    worker.addEventListener("error", (event) => {
      for (const [requestId, pending] of this.pending) {
        if (pending.lane !== lane) continue;
        pending.reject(new Error(event.message || "Studio Worker failed"));
        this.pending.delete(requestId);
      }
      if (lane === "PREVIEW" && this.previewWorker === worker) {
        this.previewWorker = null;
        this.animationPreviewRequestId = null;
      }
    });
    return worker;
  }

  request(
    request: StudioWorkerRequest,
    transfer: Transferable[] = [],
    lane: Pending["lane"] = "MAIN",
  ) {
    const worker = lane === "PREVIEW"
      ? (this.previewWorker ??= this.createWorker("PREVIEW"))
      : this.worker;
    return new Promise<StudioWorkerResponse>((resolve, reject) => {
      this.pending.set(request.requestId, { resolve, reject, lane });
      worker.postMessage(request, transfer);
    });
  }

  validateCreatureAnimationMapping(
    animationAuthoringJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "VALIDATE_CREATURE_ANIMATION_MAPPING",
      animationAuthoringJson,
    });
  }

  inspectEditableAnimationSource(
    sourceGlb: ArrayBuffer,
    clipName?: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "INSPECT_EDITABLE_ANIMATION_SOURCE",
      sourceGlb,
      clipName,
    }, [sourceGlb]);
  }

  inspectAnimationTransferCompatibility(
    targetGlb: ArrayBuffer,
    donorGlb: ArrayBuffer,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "INSPECT_ANIMATION_TRANSFER_COMPATIBILITY",
      targetGlb,
      donorGlb,
    }, [targetGlb, donorGlb]);
  }

  retargetAnimationModelClip(
    targetGlb: ArrayBuffer,
    donorGlb: ArrayBuffer,
    clipName: string,
    optionsJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "RETARGET_ANIMATION_MODEL_CLIP",
      targetGlb,
      donorGlb,
      clipName,
      optionsJson,
    }, [targetGlb, donorGlb]);
  }

  inspectHumanoidRetargetCompatibilityV2(
    targetGlb: ArrayBuffer,
    donorGlb: ArrayBuffer,
    overridesJson = "[]",
    manualMappingConfirmed = false,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "INSPECT_HUMANOID_RETARGET_COMPATIBILITY_V2",
      targetGlb,
      donorGlb,
      overridesJson,
      manualMappingConfirmed,
    }, [targetGlb, donorGlb]);
  }

  retargetAnimationClipHumanoidV2(
    targetGlb: ArrayBuffer,
    donorGlb: ArrayBuffer,
    clipName: string,
    semanticMapJson: string,
    clipId: string,
    outputName: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "RETARGET_ANIMATION_CLIP_HUMANOID_V2",
      targetGlb,
      donorGlb,
      clipName,
      semanticMapJson,
      clipId,
      outputName,
    }, [targetGlb, donorGlb]);
  }

  prepareAnimationTransferBatchV2(
    targetGlb: ArrayBuffer,
    donorGlb: ArrayBuffer,
    batchRequestJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "PREPARE_ANIMATION_TRANSFER_BATCH_V2",
      targetGlb,
      donorGlb,
      batchRequestJson,
    }, [targetGlb, donorGlb]);
  }

  buildAnimationSequencePreview(
    sequenceRequestJson: string,
    availableClipsJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "BUILD_ANIMATION_SEQUENCE_PREVIEW",
      sequenceRequestJson,
      availableClipsJson,
    });
  }

  applyAnimationWorkbenchOperationV1(
    operationJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "APPLY_ANIMATION_WORKBENCH_OPERATION_V1",
      operationJson,
    });
  }

  resampleAnimationCurveToLinear(
    curveJson: string,
    policyJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "RESAMPLE_ANIMATION_CURVE_TO_LINEAR",
      curveJson,
      policyJson,
    });
  }

  bakeAnimationLayers(
    layersJson: string,
    rigJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "BAKE_ANIMATION_LAYERS",
      layersJson,
      rigJson,
    });
  }

  validateAnimationStudioDocument(
    sourceGlb: ArrayBuffer,
    animationStudioDocumentJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "VALIDATE_ANIMATION_STUDIO_DOCUMENT",
      sourceGlb,
      animationStudioDocumentJson,
    }, [sourceGlb]);
  }

  materializeAnimationStudioDocument(
    sourceGlb: ArrayBuffer,
    animationStudioDocumentJson: string,
    requestId = workerRequestId(),
  ) {
    if (this.animationPreviewRequestId) {
      this.cancelAnimationStudioPreviewBuild();
    }
    this.animationPreviewRequestId = requestId;
    return this.request({
      requestId,
      type: "MATERIALIZE_ANIMATION_STUDIO_DOCUMENT",
      sourceGlb,
      animationStudioDocumentJson,
    }, [sourceGlb], "PREVIEW").finally(() => {
      if (this.animationPreviewRequestId === requestId) {
        this.animationPreviewRequestId = null;
      }
    });
  }

  previewAuthoredAnimationClip(
    animationStudioDocumentJson: string,
    clipId: string,
    requestId = workerRequestId(),
  ) {
    if (this.animationPreviewRequestId) {
      this.cancelAnimationStudioPreviewBuild();
    }
    this.animationPreviewRequestId = requestId;
    return this.request({
      requestId,
      type: "PREVIEW_AUTHORED_ANIMATION_CLIP",
      animationStudioDocumentJson,
      clipId,
    }, [], "PREVIEW").finally(() => {
      if (this.animationPreviewRequestId === requestId) {
        this.animationPreviewRequestId = null;
      }
    });
  }

  applyAnimationEditCommandBatch(
    sourceGlb: ArrayBuffer,
    animationStudioDocumentJson: string,
    commandBatchJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "APPLY_ANIMATION_EDIT_COMMAND_BATCH",
      sourceGlb,
      animationStudioDocumentJson,
      commandBatchJson,
    }, [sourceGlb]);
  }

  analyzeAnimationMotionQuality(
    sourceGlb: ArrayBuffer,
    authoredClipJson: string,
    policyJson: string,
    contextJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "ANALYZE_ANIMATION_MOTION_QUALITY",
      sourceGlb,
      authoredClipJson,
      policyJson,
      contextJson,
    }, [sourceGlb]);
  }

  applyAnimationAuthoringTool(
    sourceGlb: ArrayBuffer,
    authoredClipJson: string,
    toolRequestJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "APPLY_ANIMATION_AUTHORING_TOOL",
      sourceGlb,
      authoredClipJson,
      toolRequestJson,
    }, [sourceGlb]);
  }

  inspectHeldWeaponSource(
    weaponGlb: ArrayBuffer,
    filename: string,
    provenance = "LOCAL_FILE",
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "INSPECT_HELD_WEAPON_SOURCE",
      filename,
      provenance,
      weaponGlb,
    }, [weaponGlb]);
  }

  evaluateAnimationPoseParity(
    sourceGlb: ArrayBuffer,
    expectedClipJson: string,
    actualClipJson: string,
    policyJson: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "EVALUATE_ANIMATION_POSE_PARITY",
      sourceGlb,
      expectedClipJson,
      actualClipJson,
      policyJson,
    }, [sourceGlb]);
  }

  validateAnimationPreset(
    manifestJson: string,
    animationJson: string,
    catalogSha256: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "VALIDATE_ANIMATION_PRESET",
      manifestJson,
      animationJson,
      catalogSha256,
    });
  }

  inspectAnimationPresetCompatibility(
    manifestJson: string,
    animationJson: string,
    catalogSha256: string,
    sourceGlb: ArrayBuffer,
    requestId = workerRequestId(),
  ) {
    this.beginAnimationLibraryRequest(requestId);
    return this.request({
      requestId,
      type: "INSPECT_ANIMATION_PRESET_COMPATIBILITY",
      manifestJson,
      animationJson,
      catalogSha256,
      sourceGlb,
    }, [sourceGlb]).finally(() => this.finishAnimationLibraryRequest(requestId));
  }

  instantiateAnimationPreset(
    manifestJson: string,
    animationJson: string,
    catalogSha256: string,
    sourceGlb: ArrayBuffer,
    clipId: string,
    outputName: string,
    requestId = workerRequestId(),
  ) {
    this.beginAnimationLibraryRequest(requestId);
    return this.request({
      requestId,
      type: "INSTANTIATE_ANIMATION_PRESET",
      manifestJson,
      animationJson,
      catalogSha256,
      sourceGlb,
      clipId,
      outputName,
    }, [sourceGlb]).finally(() => this.finishAnimationLibraryRequest(requestId));
  }

  exportAnimationContribution(
    authoredClipJson: string,
    sourceGlb: ArrayBuffer,
    metadataJson: string,
    requestId = workerRequestId(),
  ) {
    this.beginAnimationLibraryRequest(requestId);
    return this.request({
      requestId,
      type: "EXPORT_ANIMATION_CONTRIBUTION",
      authoredClipJson,
      sourceGlb,
      metadataJson,
    }, [sourceGlb]).finally(() => this.finishAnimationLibraryRequest(requestId));
  }

  private beginAnimationLibraryRequest(requestId: string) {
    if (this.animationLibraryRequestId) this.cancel(this.animationLibraryRequestId);
    this.animationLibraryRequestId = requestId;
  }

  private finishAnimationLibraryRequest(requestId: string) {
    if (this.animationLibraryRequestId === requestId) {
      this.animationLibraryRequestId = null;
    }
  }

  buildAuthoredCreatureModelPackage(
    sourceGlb: ArrayBuffer,
    appearanceTwoDa: ArrayBuffer,
    animationAuthoringJson: string,
    projectIdentityJson: string,
    eventAuthoringJson?: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "BUILD_MODEL_PACKAGE",
      sourceGlb,
      appearanceTwoDa,
      packageLane: "H1_SKINNED_FULL_42_AUTHORED",
      animationAuthoringJson,
      projectIdentityJson,
      eventAuthoringJson,
    }, [sourceGlb, appearanceTwoDa]);
  }

  buildEditedCreatureModelPackage(
    sourceGlb: ArrayBuffer,
    appearanceTwoDa: ArrayBuffer,
    animationAuthoringJson: string,
    animationStudioDocumentJson: string,
    projectIdentityJson: string,
    eventAuthoringJson?: string,
    requestId = workerRequestId(),
  ) {
    return this.request({
      requestId,
      type: "BUILD_MODEL_PACKAGE",
      sourceGlb,
      appearanceTwoDa,
      packageLane: "H1_SKINNED_FULL_42_EDITED",
      animationAuthoringJson,
      animationStudioDocumentJson,
      projectIdentityJson,
      eventAuthoringJson,
    }, [sourceGlb, appearanceTwoDa]);
  }

  cancelAnimationStudioPreviewBuild() {
    const worker = this.previewWorker;
    const pendingPreviewIds = [...this.pending.entries()]
      .filter(([, pending]) => pending.lane === "PREVIEW")
      .map(([requestId]) => requestId);
    if (pendingPreviewIds.length === 0) return false;
    worker?.terminate();
    if (this.previewWorker === worker) this.previewWorker = null;
    this.animationPreviewRequestId = null;
    this.animationLibraryRequestId = null;
    for (const requestId of pendingPreviewIds) {
      const pending = this.pending.get(requestId);
      this.pending.delete(requestId);
      pending?.reject(new DOMException(
        "Animation preview Worker was terminated",
        "AbortError",
      ));
    }
    return true;
  }

  cancel(requestId: string) {
    const pending = this.pending.get(requestId);
    if (!pending) return false;
    if (pending.lane === "PREVIEW") {
      return this.cancelAnimationStudioPreviewBuild();
    }
    this.pending.delete(requestId);
    pending.reject(new DOMException("Studio Worker request cancelled", "AbortError"));
    return true;
  }

  dispose() {
    this.worker.terminate();
    this.previewWorker?.terminate();
    this.previewWorker = null;
    this.animationPreviewRequestId = null;
    for (const pending of this.pending.values()) {
      pending.reject(new Error("Studio Worker disposed"));
    }
    this.pending.clear();
  }
}

function workerRequestId() {
  return typeof crypto.randomUUID === "function"
    ? crypto.randomUUID()
    : `worker-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}
