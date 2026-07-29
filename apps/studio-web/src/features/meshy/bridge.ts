export const MESHY_BRIDGE_PROTOCOL_VERSION = 1 as const;

export type MeshyProfileId =
  | "H1-humanoid-animated/v1"
  | "N1-quadruped/v1"
  | "S1-static-prop/v1";

export type MeshyArtifactProfileId = MeshyProfileId | "RECOVERED-text-to-3d/v1" | "RETEXTURED-model/v1";

export type MeshyPipelineStage = "PREVIEW" | "REFINE" | "RIG" | "ANIMATE";
export type MeshyGeometryTarget = "AURORA_PROOF" | "LOWER_DETAIL" | "BALANCED" | "HIGHER_DETAIL";
export type MeshyAiModel = "latest" | "meshy-5" | "meshy-6" | "meshy-t1" | "meshy-t2";
export type MeshyModelType = "standard" | "lowpoly" | "smart-topology";
export type MeshyTopology = "triangle" | "quad";
export type MeshyPoseMode = "" | "a-pose" | "t-pose";
export type MeshyTargetFormat = "glb" | "obj" | "fbx" | "stl" | "usdz" | "3mf";
export type MeshyGenerationSource = "TEXT" | "IMAGE" | "MULTI_IMAGE";

export const MESHY_GEOMETRY_TARGETS: readonly MeshyGeometryTarget[] = [
  "AURORA_PROOF",
  "LOWER_DETAIL",
  "BALANCED",
  "HIGHER_DETAIL",
] as const;
export type MeshyRunStatus =
  | "QUEUED"
  | "PREVIEWING"
  | "REFINING"
  | "RIGGING"
  | "ANIMATING"
  | "VERIFYING"
  | "READY"
  | "FAILED"
  | "CANCELED";

export type MeshyImageAiModel = "nano-banana" | "nano-banana-2" | "nano-banana-pro" | "gpt-image-2";
export type MeshyImageRunMode = "TEXT_TO_IMAGE" | "IMAGE_TO_IMAGE";
export type MeshyImageRunStatus = "QUEUED" | "GENERATING" | "READY" | "FAILED" | "CANCELED";
export type MeshyRetextureRunStatus = "QUEUED" | "TEXTURING" | "VERIFYING" | "READY" | "FAILED" | "CANCELED";

/** A ReTexture source is always a task identity held by the local Bridge, never a signed model URL. */
export interface MeshyRetexturePreviewRequest {
  readonly inputTaskId: string;
  readonly textStylePrompt: string;
  readonly aiModel: "meshy-5" | "meshy-6";
  readonly enableOriginalUv: boolean;
  readonly enablePbr: boolean;
  readonly hdTexture: boolean;
  readonly removeLighting: boolean;
  readonly alphaThumbnail: boolean;
}

export interface MeshyRetexturePreview extends MeshyRetexturePreviewRequest {
  readonly previewId: string;
  readonly maximumCredits: number;
}

export interface MeshyRetextureRun {
  readonly id: string;
  readonly inputTaskId: string;
  readonly textStylePrompt: string;
  readonly status: MeshyRetextureRunStatus;
  readonly progress: number;
  readonly taskId?: string;
  readonly error?: { readonly code: MeshyBridgeErrorCode; readonly message: string };
}

export interface MeshyImageRunPreviewRequest {
  readonly mode: MeshyImageRunMode;
  readonly prompt: string;
  readonly aiModel: MeshyImageAiModel;
  readonly generateMultiView: boolean;
  readonly aspectRatio?: "1:1" | "16:9" | "9:16" | "4:3" | "3:4" | "3:2" | "2:3";
  readonly poseMode?: Exclude<MeshyPoseMode, "">;
  readonly referenceImageDataUrls?: readonly string[];
}

export interface MeshyImageRunPreview extends MeshyImageRunPreviewRequest {
  readonly previewId: string;
  readonly maximumCredits: number;
}

export interface MeshyImageRun {
  readonly id: string;
  readonly mode: MeshyImageRunMode;
  readonly prompt: string;
  readonly aiModel: MeshyImageAiModel;
  readonly generateMultiView: boolean;
  readonly status: MeshyImageRunStatus;
  readonly progress: number;
  readonly taskId?: string;
  readonly error?: { readonly code: MeshyBridgeErrorCode; readonly message: string };
}

export interface MeshyProfile {
  readonly id: MeshyProfileId;
  readonly label: string;
  readonly description: string;
  readonly stages: readonly MeshyPipelineStage[];
  readonly expectedOutput: {
    readonly texture: boolean;
    readonly rigging: boolean;
    readonly animation: "IDLE" | null;
  };
}

export const MESHY_PROFILES: readonly MeshyProfile[] = [
  {
    id: "H1-humanoid-animated/v1",
    label: "Humanoid Animated",
    description: "Textured standard humanoid with rigging and one Idle animation proof.",
    stages: ["PREVIEW", "REFINE", "RIG", "ANIMATE"],
    expectedOutput: { texture: true, rigging: true, animation: "IDLE" },
  },
  {
    id: "N1-quadruped/v1",
    label: "Quadruped",
    description: "Textured non-humanoid proof asset without auto-rigging.",
    stages: ["PREVIEW", "REFINE"],
    expectedOutput: { texture: true, rigging: false, animation: null },
  },
  {
    id: "S1-static-prop/v1",
    label: "Static Prop",
    description: "Textured static proof asset without a skeleton or animation.",
    stages: ["PREVIEW", "REFINE"],
    expectedOutput: { texture: true, rigging: false, animation: null },
  },
] as const;

export function findMeshyProfile(id: MeshyProfileId): MeshyProfile | undefined {
  return MESHY_PROFILES.find((profile) => profile.id === id);
}

export type MeshyBridgeErrorCode =
  | "BRIDGE_UNAVAILABLE"
  | "PAIRING_REQUIRED"
  | "CONFIRMATION_REQUIRED"
  | "CONFIRMATION_ALREADY_USED"
  | "H1_PREFLIGHT_REQUIRED"
  | "PREVIEW_NOT_FOUND"
  | "RUN_NOT_FOUND"
  | "ARTIFACT_NOT_READY"
  | "ARTIFACT_INVALID";

export class MeshyBridgeError extends Error {
  readonly code: MeshyBridgeErrorCode;

  constructor(code: MeshyBridgeErrorCode, message: string) {
    super(message);
    this.name = "MeshyBridgeError";
    this.code = code;
  }
}

export interface MeshyBridgeHealth {
  readonly protocolVersion: typeof MESHY_BRIDGE_PROTOCOL_VERSION;
  readonly bridge: "LOCAL";
  readonly status: "READY";
  /** True only when the owner started Bridge under a local supervisor that can recreate it. */
  readonly restartSupported: boolean;
  /** True only for the local Compose exact-origin handoff; no code or API key is returned. */
  readonly automaticPairingSupported: boolean;
}

export interface MeshyBridgePairing {
  readonly sessionToken: string;
  readonly expiresAt: string;
}

export interface MeshyBalance {
  readonly availableCredits: number;
}

export interface MeshyRunPreviewRequest {
  readonly profileId: MeshyProfileId;
  readonly prompt: string;
  readonly geometryTarget: MeshyGeometryTarget;
  readonly source?: MeshyGenerationSource;
  readonly imageDataUrls?: readonly string[];
  /** A completed Meshy 2D task ID. The Bridge exchanges it directly with Meshy; no signed image URL reaches Studio. */
  readonly inputTaskId?: string;
  readonly apiOptions?: MeshyTextTo3DOptions;
  readonly h1Preflight?: {
    readonly standardHumanoid: true;
    readonly clearLimbs: true;
    readonly noWeapon: true;
  };
}

export interface MeshyTextTo3DOptions {
  readonly modelType: MeshyModelType;
  readonly aiModel: MeshyAiModel;
  readonly shouldRemesh: boolean;
  readonly topology: MeshyTopology;
  readonly targetPolycount: number;
  readonly decimationMode?: 1 | 2 | 3 | 4;
  readonly poseMode: MeshyPoseMode;
  readonly moderation: boolean;
  readonly targetFormats: readonly MeshyTargetFormat[];
  readonly alphaThumbnail: boolean;
  readonly autoSize: boolean;
  readonly originAt: "bottom" | "center";
  readonly enablePbr: boolean;
  readonly shouldTexture: boolean;
  readonly hdTexture: boolean;
  readonly texturePrompt: string;
  readonly textureImageUrl: string;
  readonly removeLighting: boolean;
  readonly imageEnhancement: boolean;
  readonly multiViewThumbnails: boolean;
  readonly rigHumanoid: boolean;
  readonly rigHeightMeters: number;
  readonly animationActionId?: number;
}

export const AURORA_MODEL_TRIANGLE_BUDGET_V1 = 20_000;

export function maximumMeshyTargetPolycount(
  options: Pick<MeshyTextTo3DOptions, "modelType" | "aiModel">,
): number {
  return options.modelType === "smart-topology" && options.aiModel === "meshy-t2"
    ? 15_000
    : AURORA_MODEL_TRIANGLE_BUDGET_V1;
}

export const DEFAULT_MESHY_TEXT_TO_3D_OPTIONS: MeshyTextTo3DOptions = {
  modelType: "standard",
  // Keep generation reproducible in the Studio. "latest" is still accepted by
  // the Bridge for older callers, but it is a moving Meshy alias rather than a
  // meaningful choice in the UI.
  aiModel: "meshy-6",
  shouldRemesh: true,
  topology: "triangle",
  targetPolycount: AURORA_MODEL_TRIANGLE_BUDGET_V1,
  poseMode: "",
  moderation: true,
  targetFormats: ["glb"],
  alphaThumbnail: false,
  autoSize: false,
  originAt: "bottom",
  enablePbr: true,
  shouldTexture: true,
  hdTexture: false,
  texturePrompt: "",
  textureImageUrl: "",
  removeLighting: true,
  imageEnhancement: true,
  multiViewThumbnails: false,
  rigHumanoid: false,
  rigHeightMeters: 1.7,
  animationActionId: 0,
};

export interface MeshyRunPreview {
  readonly previewId: string;
  readonly profile: MeshyProfile;
  readonly maximumCredits: number;
  readonly stages: readonly MeshyPipelineStage[];
}

export interface MeshyRun {
  readonly id: string;
  readonly profile: MeshyProfile;
  readonly prompt: string;
  readonly geometryTarget: MeshyGeometryTarget;
  readonly status: MeshyRunStatus;
  readonly progress: number;
  readonly taskIds: Readonly<Partial<Record<MeshyPipelineStage, string>>>;
  readonly createdAt: string;
  readonly updatedAt: string;
  readonly error?: { readonly code: MeshyBridgeErrorCode; readonly message: string };
}

export interface MeshyArtifactProvenance {
  readonly profileId: MeshyArtifactProfileId;
  readonly bridgeProtocolVersion: typeof MESHY_BRIDGE_PROTOCOL_VERSION;
  readonly sha256: string;
  readonly byteLength: number;
  readonly taskIds: Readonly<Partial<Record<MeshyPipelineStage, string>>>;
}

export interface MeshyRunArtifact {
  readonly file: File;
  readonly provenance: MeshyArtifactProvenance;
}

export interface MeshyHistoryItem {
  readonly taskId: string;
  readonly stage: "PREVIEW" | "REFINE";
  readonly status: string;
  readonly prompt: string;
  readonly createdAt?: string;
  readonly finishedAt?: string;
  readonly consumedCredits?: number;
  readonly glbAvailable: boolean;
  readonly thumbnailAvailable?: boolean;
}

export interface MeshyHistoryPage {
  readonly pageNum: number;
  readonly pageSize: number;
  readonly items: readonly MeshyHistoryItem[];
  readonly hasNext: boolean;
}

export interface MeshyHistoryThumbnail {
  readonly file: File;
}

export interface MeshyBridgeClient {
  health(): Promise<MeshyBridgeHealth>;
  restart(): Promise<void>;
  pairAutomatically(): Promise<MeshyBridgePairing>;
  pair(input: { readonly pairingCode: string }): Promise<MeshyBridgePairing>;
  balance(sessionToken: string): Promise<MeshyBalance>;
  profiles(sessionToken: string): Promise<readonly MeshyProfile[]>;
  previewRun(sessionToken: string, input: MeshyRunPreviewRequest): Promise<MeshyRunPreview>;
  createRun(sessionToken: string, input: {
    readonly previewId: string;
    readonly confirmationNonce: string;
  }): Promise<MeshyRun>;
  getRun(sessionToken: string, runId: string): Promise<MeshyRun>;
  cancelRun(sessionToken: string, runId: string): Promise<MeshyRun>;
  provenance(sessionToken: string, runId: string): Promise<MeshyArtifactProvenance>;
  downloadArtifact(sessionToken: string, runId: string): Promise<MeshyRunArtifact>;
  listHistory(sessionToken: string, pageNum?: number): Promise<MeshyHistoryPage>;
  downloadHistoryArtifact(sessionToken: string, taskId: string): Promise<MeshyRunArtifact>;
  downloadHistoryThumbnail(sessionToken: string, taskId: string): Promise<MeshyHistoryThumbnail>;
  previewRetexture(sessionToken: string, input: MeshyRetexturePreviewRequest): Promise<MeshyRetexturePreview>;
  createRetexture(sessionToken: string, input: { readonly previewId: string; readonly confirmationNonce: string }): Promise<MeshyRetextureRun>;
  getRetexture(sessionToken: string, runId: string): Promise<MeshyRetextureRun>;
  cancelRetexture(sessionToken: string, runId: string): Promise<MeshyRetextureRun>;
  downloadRetextureArtifact(sessionToken: string, runId: string): Promise<MeshyRunArtifact>;
  previewImageRun(sessionToken: string, input: MeshyImageRunPreviewRequest): Promise<MeshyImageRunPreview>;
  createImageRun(sessionToken: string, input: { readonly previewId: string; readonly confirmationNonce: string }): Promise<MeshyImageRun>;
  getImageRun(sessionToken: string, runId: string): Promise<MeshyImageRun>;
  cancelImageRun(sessionToken: string, runId: string): Promise<MeshyImageRun>;
}

interface StoredPreview extends MeshyRunPreviewRequest, MeshyRunPreview {}

interface StoredRun {
  run: MeshyRun;
  artifact?: MeshyRunArtifact;
}

interface StoredHistoryItem {
  item: MeshyHistoryItem;
  artifact?: MeshyRunArtifact;
}

interface StoredImageRun {
  run: MeshyImageRun;
}

interface StoredRetextureRun {
  run: MeshyRetextureRun;
  artifact?: MeshyRunArtifact;
}

function now() {
  return new Date().toISOString();
}

function identifier(prefix: string) {
  return `${prefix}-${crypto.randomUUID()}`;
}

async function sha256(bytes: Uint8Array) {
  const digest = await crypto.subtle.digest("SHA-256", bytes.slice().buffer);
  return Array.from(new Uint8Array(digest), (value) => value.toString(16).padStart(2, "0")).join("");
}

/**
 * Test/dev adapter. It has no network implementation and deliberately cannot
 * receive an API key. Production uses the same MeshyBridgeClient boundary.
 */
export class InMemoryMeshyBridgeClient implements MeshyBridgeClient {
  private readonly previews = new Map<string, StoredPreview>();
  private readonly runs = new Map<string, StoredRun>();
  private readonly history = new Map<string, StoredHistoryItem>();
  private readonly imagePreviews = new Map<string, MeshyImageRunPreview>();
  private readonly imageRuns = new Map<string, StoredImageRun>();
  private readonly retexturePreviews = new Map<string, MeshyRetexturePreview>();
  private readonly retextureRuns = new Map<string, StoredRetextureRun>();
  private readonly usedConfirmationNonces = new Set<string>();
  private readonly sessions = new Set<string>();
  private pairingCodeUsed = false;
  private readonly availableCredits: number;
  private readonly automaticPairingSupported: boolean;

  constructor(options: { readonly availableCredits?: number; readonly automaticPairingSupported?: boolean } = {}) {
    this.availableCredits = options.availableCredits ?? 0;
    this.automaticPairingSupported = options.automaticPairingSupported ?? false;
  }

  async health(): Promise<MeshyBridgeHealth> {
    return { protocolVersion: MESHY_BRIDGE_PROTOCOL_VERSION, bridge: "LOCAL", status: "READY", restartSupported: false, automaticPairingSupported: this.automaticPairingSupported };
  }

  async restart(): Promise<void> {
    throw new MeshyBridgeError("BRIDGE_UNAVAILABLE", "The in-memory Bridge cannot be restarted.");
  }

  async pairAutomatically(): Promise<MeshyBridgePairing> {
    if (!this.automaticPairingSupported) throw new MeshyBridgeError("BRIDGE_UNAVAILABLE", "The in-memory Bridge requires a manual pairing code.");
    const sessionToken = identifier("meshy-session");
    this.sessions.add(sessionToken);
    return { sessionToken, expiresAt: new Date(Date.now() + 15 * 60_000).toISOString() };
  }

  async pair(input: { readonly pairingCode: string }): Promise<MeshyBridgePairing> {
    if (!input.pairingCode.trim()) {
      throw new MeshyBridgeError("PAIRING_REQUIRED", "A local Bridge pairing code is required.");
    }
    if (this.pairingCodeUsed) {
      throw new MeshyBridgeError("PAIRING_REQUIRED", "This local Bridge pairing code has already been used.");
    }
    this.pairingCodeUsed = true;
    const sessionToken = identifier("meshy-session");
    this.sessions.add(sessionToken);
    return {
      sessionToken,
      expiresAt: new Date(Date.now() + 15 * 60_000).toISOString(),
    };
  }

  async balance(sessionToken: string): Promise<MeshyBalance> {
    this.requireSession(sessionToken);
    return { availableCredits: this.availableCredits };
  }

  async profiles(sessionToken: string): Promise<readonly MeshyProfile[]> {
    this.requireSession(sessionToken);
    return MESHY_PROFILES;
  }

  async previewRun(sessionToken: string, input: MeshyRunPreviewRequest): Promise<MeshyRunPreview> {
    this.requireSession(sessionToken);
    const profile = findMeshyProfile(input.profileId);
    const source = input.source ?? "TEXT";
    const images = input.imageDataUrls ?? [];
    const hasTaskInput = Boolean(input.inputTaskId?.trim());
    if (!profile) {
      throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "Select a supported generation profile.");
    }
    if (
      input.apiOptions
      && (
        !Number.isInteger(input.apiOptions.targetPolycount)
        || input.apiOptions.targetPolycount < 100
        || input.apiOptions.targetPolycount > maximumMeshyTargetPolycount(input.apiOptions)
      )
    ) {
      throw new MeshyBridgeError(
        "PREVIEW_NOT_FOUND",
        `Target polycount must be between 100 and ${maximumMeshyTargetPolycount(input.apiOptions)}.`,
      );
    }
    if (source === "TEXT" && !input.prompt.trim()) {
      throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "Text to 3D requires an asset prompt.");
    }
    if (source === "IMAGE" && !hasTaskInput && images.length !== 1) {
      throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "Image to 3D requires exactly one reference image.");
    }
    if (source === "MULTI_IMAGE" && !hasTaskInput && (images.length < 1 || images.length > 4)) {
      throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "Multi-image to 3D requires one to four reference images.");
    }
    if (profile.id.startsWith("H1") && !isH1PreflightComplete(input.h1Preflight)) {
      throw new MeshyBridgeError("H1_PREFLIGHT_REQUIRED", "Confirm standard humanoid, clear limbs, and no weapon before H1 rigging.");
    }
    const preview: StoredPreview = {
      ...input,
      previewId: identifier("meshy-preview"),
      profile,
      maximumCredits: this.maximumCredits(profile),
      stages: profile.stages,
    };
    this.previews.set(preview.previewId, preview);
    const { imageDataUrls: _imageDataUrls, ...safePreview } = preview;
    return safePreview;
  }

  async createRun(sessionToken: string, input: {
    readonly previewId: string;
    readonly confirmationNonce: string;
  }): Promise<MeshyRun> {
    this.requireSession(sessionToken);
    if (!input.confirmationNonce.trim()) {
      throw new MeshyBridgeError("CONFIRMATION_REQUIRED", "Confirm generation before creating a Meshy task.");
    }
    if (this.usedConfirmationNonces.has(input.confirmationNonce)) {
      throw new MeshyBridgeError("CONFIRMATION_ALREADY_USED", "This confirmation was already used for a Meshy run.");
    }
    const preview = this.previews.get(input.previewId);
    if (!preview) throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "The generation preview is no longer available.");

    this.usedConfirmationNonces.add(input.confirmationNonce);
    const timestamp = now();
    const run: MeshyRun = {
      id: identifier("meshy-run"),
      profile: preview.profile,
      prompt: preview.prompt,
      geometryTarget: preview.geometryTarget,
      status: "QUEUED",
      progress: 0,
      taskIds: {},
      createdAt: timestamp,
      updatedAt: timestamp,
    };
    this.runs.set(run.id, { run });
    return run;
  }

  async getRun(sessionToken: string, runId: string): Promise<MeshyRun> {
    this.requireSession(sessionToken);
    return this.storedRun(runId).run;
  }

  async cancelRun(sessionToken: string, runId: string): Promise<MeshyRun> {
    this.requireSession(sessionToken);
    const stored = this.storedRun(runId);
    if (stored.run.status === "READY") return stored.run;
    stored.run = { ...stored.run, status: "CANCELED", updatedAt: now() };
    return stored.run;
  }

  async downloadArtifact(sessionToken: string, runId: string): Promise<MeshyRunArtifact> {
    this.requireSession(sessionToken);
    const artifact = this.storedRun(runId).artifact;
    if (!artifact) throw new MeshyBridgeError("ARTIFACT_NOT_READY", "The verified GLB is not ready to import.");
    return artifact;
  }

  async provenance(sessionToken: string, runId: string): Promise<MeshyArtifactProvenance> {
    this.requireSession(sessionToken);
    const artifact = this.storedRun(runId).artifact;
    if (!artifact) throw new MeshyBridgeError("ARTIFACT_NOT_READY", "The verified GLB is not ready to inspect.");
    return artifact.provenance;
  }

  async listHistory(sessionToken: string, pageNum = 1): Promise<MeshyHistoryPage> {
    this.requireSession(sessionToken);
    const pageSize = 50;
    const items = Array.from(this.history.values(), ({ item }) => item);
    const offset = (pageNum - 1) * pageSize;
    return {
      pageNum,
      pageSize,
      items: items.slice(offset, offset + pageSize),
      hasNext: offset + pageSize < items.length,
    };
  }

  async downloadHistoryArtifact(sessionToken: string, taskId: string): Promise<MeshyRunArtifact> {
    this.requireSession(sessionToken);
    const artifact = this.history.get(taskId)?.artifact;
    if (!artifact) throw new MeshyBridgeError("ARTIFACT_NOT_READY", "The selected Meshy history entry cannot be recovered as a GLB.");
    return artifact;
  }

  async downloadHistoryThumbnail(sessionToken: string, taskId: string): Promise<MeshyHistoryThumbnail> {
    this.requireSession(sessionToken);
    const item = this.history.get(taskId)?.item;
    if (!item?.thumbnailAvailable) throw new MeshyBridgeError("ARTIFACT_NOT_READY", "The selected Meshy history entry has no thumbnail.");
    return { file: new File(["<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 160 160\"><rect width=\"160\" height=\"160\" fill=\"#242624\"/><path d=\"M80 24 128 52v56l-48 28-48-28V52z\" fill=\"#7d857d\"/><path d=\"m80 42 30 18v36L80 114 50 96V60z\" fill=\"#bfc7bd\"/></svg>"], "meshy-thumbnail.svg", { type: "image/svg+xml" }) };
  }

  async previewRetexture(sessionToken: string, input: MeshyRetexturePreviewRequest): Promise<MeshyRetexturePreview> {
    this.requireSession(sessionToken);
    if (!/^[A-Za-z0-9_-]{1,128}$/.test(input.inputTaskId) || !input.textStylePrompt.trim() || input.textStylePrompt.length > 600) {
      throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "ReTexture needs a verified Meshy task and a 1-600 character style prompt.");
    }
    const preview: MeshyRetexturePreview = { ...input, textStylePrompt: input.textStylePrompt.trim(), previewId: identifier("meshy-retexture-preview"), maximumCredits: 10 };
    this.retexturePreviews.set(preview.previewId, preview);
    return preview;
  }

  async createRetexture(sessionToken: string, input: { readonly previewId: string; readonly confirmationNonce: string }): Promise<MeshyRetextureRun> {
    this.requireSession(sessionToken);
    if (!input.confirmationNonce.trim()) throw new MeshyBridgeError("CONFIRMATION_REQUIRED", "Confirm ReTexture before creating a Meshy task.");
    if (this.usedConfirmationNonces.has(input.confirmationNonce)) throw new MeshyBridgeError("CONFIRMATION_ALREADY_USED", "This confirmation was already used for a Meshy run.");
    const preview = this.retexturePreviews.get(input.previewId);
    if (!preview) throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "The ReTexture preview is no longer available.");
    this.usedConfirmationNonces.add(input.confirmationNonce);
    const run: MeshyRetextureRun = { id: identifier("meshy-retexture-run"), inputTaskId: preview.inputTaskId, textStylePrompt: preview.textStylePrompt, status: "QUEUED", progress: 0 };
    this.retextureRuns.set(run.id, { run });
    return run;
  }

  async getRetexture(sessionToken: string, runId: string): Promise<MeshyRetextureRun> { this.requireSession(sessionToken); return this.storedRetextureRun(runId).run; }
  async cancelRetexture(sessionToken: string, runId: string): Promise<MeshyRetextureRun> {
    this.requireSession(sessionToken);
    const stored = this.storedRetextureRun(runId);
    if (stored.run.status !== "READY") stored.run = { ...stored.run, status: "CANCELED" };
    return stored.run;
  }
  async downloadRetextureArtifact(sessionToken: string, runId: string): Promise<MeshyRunArtifact> {
    this.requireSession(sessionToken);
    const artifact = this.storedRetextureRun(runId).artifact;
    if (!artifact) throw new MeshyBridgeError("ARTIFACT_NOT_READY", "The verified ReTexture GLB is not ready.");
    return artifact;
  }

  async previewImageRun(sessionToken: string, input: MeshyImageRunPreviewRequest): Promise<MeshyImageRunPreview> {
    this.requireSession(sessionToken);
    if (!input.prompt.trim()) throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "Image generation requires a prompt.");
    if (input.mode === "IMAGE_TO_IMAGE" && (!input.referenceImageDataUrls || input.referenceImageDataUrls.length < 1 || input.referenceImageDataUrls.length > 5)) {
      throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "Image to Image requires one to five reference images.");
    }
    const preview: MeshyImageRunPreview = {
      ...input,
      previewId: identifier("meshy-image-preview"),
      maximumCredits: input.mode === "IMAGE_TO_IMAGE" && input.aiModel === "gpt-image-2" ? 12 : input.aiModel === "nano-banana" ? 3 : input.aiModel === "nano-banana-2" ? 6 : 9,
    };
    this.imagePreviews.set(preview.previewId, preview);
    const { referenceImageDataUrls: _referenceImageDataUrls, ...safePreview } = preview;
    return safePreview;
  }

  async createImageRun(sessionToken: string, input: { readonly previewId: string; readonly confirmationNonce: string }): Promise<MeshyImageRun> {
    this.requireSession(sessionToken);
    if (!input.confirmationNonce.trim()) throw new MeshyBridgeError("CONFIRMATION_REQUIRED", "Confirm image generation before creating a Meshy task.");
    if (this.usedConfirmationNonces.has(input.confirmationNonce)) throw new MeshyBridgeError("CONFIRMATION_ALREADY_USED", "This confirmation was already used for a Meshy run.");
    const preview = this.imagePreviews.get(input.previewId);
    if (!preview) throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "The image-generation preview is no longer available.");
    this.usedConfirmationNonces.add(input.confirmationNonce);
    const run: MeshyImageRun = { id: identifier("meshy-image-run"), mode: preview.mode, prompt: preview.prompt, aiModel: preview.aiModel, generateMultiView: preview.generateMultiView, status: "QUEUED", progress: 0 };
    this.imageRuns.set(run.id, { run });
    return run;
  }

  async getImageRun(sessionToken: string, runId: string): Promise<MeshyImageRun> {
    this.requireSession(sessionToken);
    return this.storedImageRun(runId).run;
  }

  async cancelImageRun(sessionToken: string, runId: string): Promise<MeshyImageRun> {
    this.requireSession(sessionToken);
    const stored = this.storedImageRun(runId);
    if (stored.run.status === "READY") return stored.run;
    stored.run = { ...stored.run, status: "CANCELED" };
    return stored.run;
  }

  async completeRunForTest(runId: string, bytes: Uint8Array): Promise<MeshyRun> {
    const stored = this.storedRun(runId);
    const profileSlug = stored.run.profile.id.startsWith("S1") ? "s1-static-prop"
      : stored.run.profile.id.startsWith("N1") ? "n1-quadruped"
        : "h1-humanoid-animated";
    const hash = await sha256(bytes);
    const taskIds = Object.fromEntries(
      stored.run.profile.stages.map((stage) => [stage, identifier(`task-${stage.toLowerCase()}`)]),
    ) as Readonly<Partial<Record<MeshyPipelineStage, string>>>;
    stored.run = {
      ...stored.run,
      status: "READY",
      progress: 100,
      taskIds,
      updatedAt: now(),
    };
    stored.artifact = {
      file: new File([bytes.slice()], `meshy-${profileSlug}.glb`, { type: "model/gltf-binary" }),
      provenance: {
        profileId: stored.run.profile.id,
        bridgeProtocolVersion: MESHY_BRIDGE_PROTOCOL_VERSION,
        sha256: hash,
        byteLength: bytes.byteLength,
        taskIds,
      },
    };
    return stored.run;
  }

  latestRunIdForTest(): string | undefined {
    return Array.from(this.runs.keys()).at(-1);
  }

  async addRecoverableHistoryForTest(item: MeshyHistoryItem, bytes: Uint8Array): Promise<void> {
    const hash = await sha256(bytes);
    this.history.set(item.taskId, {
      item,
      artifact: {
        file: new File([bytes.slice()], "meshy-recovered-text-to-3d.glb", { type: "model/gltf-binary" }),
        provenance: {
          profileId: "RECOVERED-text-to-3d/v1",
          bridgeProtocolVersion: MESHY_BRIDGE_PROTOCOL_VERSION,
          sha256: hash,
          byteLength: bytes.byteLength,
          taskIds: { REFINE: item.taskId },
        },
      },
    });
  }

  private requireSession(sessionToken: string) {
    if (!this.sessions.has(sessionToken)) {
      throw new MeshyBridgeError("PAIRING_REQUIRED", "Pair this browser session with the local Bridge first.");
    }
  }

  private storedRun(runId: string): StoredRun {
    const run = this.runs.get(runId);
    if (!run) throw new MeshyBridgeError("RUN_NOT_FOUND", "The requested Meshy run does not exist.");
    return run;
  }

  private storedImageRun(runId: string): StoredImageRun {
    const run = this.imageRuns.get(runId);
    if (!run) throw new MeshyBridgeError("RUN_NOT_FOUND", "The requested image run does not exist.");
    return run;
  }

  private storedRetextureRun(runId: string): StoredRetextureRun {
    const run = this.retextureRuns.get(runId);
    if (!run) throw new MeshyBridgeError("RUN_NOT_FOUND", "The requested ReTexture run does not exist.");
    return run;
  }

  private maximumCredits(profile: MeshyProfile) {
    return profile.id.startsWith("H1") ? 38 : 30;
  }
}

function isH1PreflightComplete(value: MeshyRunPreviewRequest["h1Preflight"]) {
  return value?.standardHumanoid === true && value.clearLimbs === true && value.noWeapon === true;
}

export function resolveLocalMeshyBridgeOrigin(input: { readonly configured?: string; readonly development: boolean; readonly search: string }) {
  const configured = input.configured || "http://127.0.0.1:43119";
  // This endpoint is the repository's synthetic visual-proof Bridge.  It is
  // intentionally unreachable in a production build and is not a user-facing
  // Bridge configuration option.
  if (input.development && new URLSearchParams(input.search).has("meshyProofBridge")) return "http://127.0.0.1:43120";
  return configured;
}

/** Browser adapter for the owner-operated loopback Bridge. It never talks to api.meshy.ai. */
export class LocalMeshyBridgeClient implements MeshyBridgeClient {
  constructor(private readonly origin = resolveLocalMeshyBridgeOrigin({
    configured: import.meta.env.VITE_MESHY_BRIDGE_ORIGIN,
    development: import.meta.env.DEV,
    search: window.location.search,
  })) {}

  async health(): Promise<MeshyBridgeHealth> {
    return this.json("/v1/health") as Promise<MeshyBridgeHealth>;
  }

  async restart(): Promise<void> {
    await this.json("/v1/bridge/restart", { method: "POST" });
  }

  async pairAutomatically(): Promise<MeshyBridgePairing> {
    return this.json("/v1/pair/automatic", { method: "POST" }) as Promise<MeshyBridgePairing>;
  }

  async pair(input: { readonly pairingCode: string }): Promise<MeshyBridgePairing> {
    return this.json("/v1/pair", { method: "POST", body: input }) as Promise<MeshyBridgePairing>;
  }

  async balance(sessionToken: string): Promise<MeshyBalance> {
    return this.json("/v1/balance", { sessionToken }) as Promise<MeshyBalance>;
  }

  async profiles(sessionToken: string): Promise<readonly MeshyProfile[]> {
    return this.json("/v1/profiles", { sessionToken }) as Promise<readonly MeshyProfile[]>;
  }

  async previewRun(sessionToken: string, input: MeshyRunPreviewRequest): Promise<MeshyRunPreview> {
    return this.json("/v1/runs/preview", { method: "POST", sessionToken, body: input }) as Promise<MeshyRunPreview>;
  }

  async createRun(sessionToken: string, input: { readonly previewId: string; readonly confirmationNonce: string }): Promise<MeshyRun> {
    return this.json("/v1/runs", { method: "POST", sessionToken, body: input }) as Promise<MeshyRun>;
  }

  async getRun(sessionToken: string, runId: string): Promise<MeshyRun> {
    return this.json(`/v1/runs/${encodeURIComponent(runId)}`, { sessionToken }) as Promise<MeshyRun>;
  }

  async cancelRun(sessionToken: string, runId: string): Promise<MeshyRun> {
    return this.json(`/v1/runs/${encodeURIComponent(runId)}/cancel`, { method: "POST", sessionToken }) as Promise<MeshyRun>;
  }

  async downloadArtifact(sessionToken: string, runId: string): Promise<MeshyRunArtifact> {
    return this.downloadVerifiedArtifact(sessionToken, `/v1/runs/${encodeURIComponent(runId)}`);
  }

  async listHistory(sessionToken: string, pageNum = 1): Promise<MeshyHistoryPage> {
    return this.json(`/v1/history?page_num=${encodeURIComponent(pageNum)}&page_size=50`, { sessionToken }) as Promise<MeshyHistoryPage>;
  }

  async downloadHistoryArtifact(sessionToken: string, taskId: string): Promise<MeshyRunArtifact> {
    return this.downloadVerifiedArtifact(sessionToken, `/v1/history/${encodeURIComponent(taskId)}`);
  }

  async downloadHistoryThumbnail(sessionToken: string, taskId: string): Promise<MeshyHistoryThumbnail> {
    const response = await fetch(`${this.origin}/v1/history/${encodeURIComponent(taskId)}/thumbnail`, { headers: { "X-Meshy-Session": sessionToken } });
    if (!response.ok) throw await this.toError(response);
    const type = response.headers.get("content-type")?.split(";", 1)[0] ?? "";
    if (!["image/png", "image/jpeg", "image/webp"].includes(type)) throw new MeshyBridgeError("ARTIFACT_INVALID", "The local Bridge returned an invalid thumbnail type.");
    const blob = await response.blob();
    if (!blob.size || blob.size > 8 * 1024 * 1024) throw new MeshyBridgeError("ARTIFACT_INVALID", "The local Bridge thumbnail violates the size gate.");
    return { file: new File([blob], `meshy-${taskId}-thumbnail.${type === "image/png" ? "png" : type === "image/jpeg" ? "jpg" : "webp"}`, { type }) };
  }

  async previewRetexture(sessionToken: string, input: MeshyRetexturePreviewRequest): Promise<MeshyRetexturePreview> {
    return this.json("/v1/retexture/preview", { method: "POST", sessionToken, body: input }) as Promise<MeshyRetexturePreview>;
  }

  async createRetexture(sessionToken: string, input: { readonly previewId: string; readonly confirmationNonce: string }): Promise<MeshyRetextureRun> {
    return this.json("/v1/retexture", { method: "POST", sessionToken, body: input }) as Promise<MeshyRetextureRun>;
  }

  async getRetexture(sessionToken: string, runId: string): Promise<MeshyRetextureRun> {
    return this.json(`/v1/retexture/${encodeURIComponent(runId)}`, { sessionToken }) as Promise<MeshyRetextureRun>;
  }

  async cancelRetexture(sessionToken: string, runId: string): Promise<MeshyRetextureRun> {
    return this.json(`/v1/retexture/${encodeURIComponent(runId)}/cancel`, { method: "POST", sessionToken }) as Promise<MeshyRetextureRun>;
  }

  async downloadRetextureArtifact(sessionToken: string, runId: string): Promise<MeshyRunArtifact> {
    return this.downloadVerifiedArtifact(sessionToken, `/v1/retexture/${encodeURIComponent(runId)}`);
  }

  async previewImageRun(sessionToken: string, input: MeshyImageRunPreviewRequest): Promise<MeshyImageRunPreview> {
    return this.json("/v1/image-runs/preview", { method: "POST", sessionToken, body: input }) as Promise<MeshyImageRunPreview>;
  }

  async createImageRun(sessionToken: string, input: { readonly previewId: string; readonly confirmationNonce: string }): Promise<MeshyImageRun> {
    return this.json("/v1/image-runs", { method: "POST", sessionToken, body: input }) as Promise<MeshyImageRun>;
  }

  async getImageRun(sessionToken: string, runId: string): Promise<MeshyImageRun> {
    return this.json(`/v1/image-runs/${encodeURIComponent(runId)}`, { sessionToken }) as Promise<MeshyImageRun>;
  }

  async cancelImageRun(sessionToken: string, runId: string): Promise<MeshyImageRun> {
    return this.json(`/v1/image-runs/${encodeURIComponent(runId)}/cancel`, { method: "POST", sessionToken }) as Promise<MeshyImageRun>;
  }

  private async downloadVerifiedArtifact(sessionToken: string, basePath: string): Promise<MeshyRunArtifact> {
    const provenance = await this.json(`${basePath}/provenance`, { sessionToken }) as MeshyArtifactProvenance;
    const response = await fetch(`${this.origin}${basePath}/artifact`, { headers: { "X-Meshy-Session": sessionToken } });
    if (!response.ok) throw await this.toError(response);
    const bytes = await response.blob();
    if (bytes.size !== provenance.byteLength) {
      throw new MeshyBridgeError("ARTIFACT_INVALID", "The downloaded artifact does not match Bridge provenance.");
    }
    const content = new Uint8Array(await bytes.arrayBuffer());
    if (new TextDecoder().decode(content.slice(0, 4)) !== "glTF") {
      throw new MeshyBridgeError("ARTIFACT_INVALID", "The downloaded artifact is not a binary glTF file.");
    }
    if (await sha256(content) !== provenance.sha256) {
      throw new MeshyBridgeError("ARTIFACT_INVALID", "The downloaded artifact SHA-256 does not match Bridge provenance.");
    }
    return {
      file: new File([bytes], `meshy-${provenance.profileId.split("/")[0].toLowerCase()}.glb`, { type: "model/gltf-binary" }),
      provenance,
    };
  }

  async provenance(sessionToken: string, runId: string): Promise<MeshyArtifactProvenance> {
    return this.json(`/v1/runs/${encodeURIComponent(runId)}/provenance`, { sessionToken }) as Promise<MeshyArtifactProvenance>;
  }

  private async json(path: string, options: {
    readonly method?: "POST";
    readonly sessionToken?: string;
    readonly body?: unknown;
  } = {}): Promise<unknown> {
    const response = await fetch(`${this.origin}${path}`, {
      method: options.method ?? "GET",
      headers: {
        ...(options.sessionToken ? { "X-Meshy-Session": options.sessionToken } : {}),
        ...(options.body === undefined ? {} : { "Content-Type": "application/json" }),
      },
      ...(options.body === undefined ? {} : { body: JSON.stringify(options.body) }),
    });
    if (!response.ok) throw await this.toError(response);
    return response.json();
  }

  private async toError(response: Response): Promise<MeshyBridgeError> {
    const payload = await response.json().catch(() => ({})) as { code?: MeshyBridgeErrorCode; message?: string };
    return new MeshyBridgeError(payload.code ?? "BRIDGE_UNAVAILABLE", payload.message ?? "The local Meshy Bridge request failed.");
  }
}
