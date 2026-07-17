export const MESHY_BRIDGE_PROTOCOL_VERSION = 1 as const;

export type MeshyProfileId =
  | "H1-humanoid-animated/v1"
  | "N1-quadruped/v1"
  | "S1-static-prop/v1";

export type MeshyPipelineStage = "PREVIEW" | "REFINE" | "RIG" | "ANIMATE";
export type MeshyGeometryTarget = "AURORA_PROOF" | "LOWER_DETAIL" | "BALANCED" | "HIGHER_DETAIL";

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
  readonly h1Preflight?: {
    readonly standardHumanoid: true;
    readonly clearLimbs: true;
    readonly noWeapon: true;
  };
}

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
  readonly profileId: MeshyProfileId;
  readonly bridgeProtocolVersion: typeof MESHY_BRIDGE_PROTOCOL_VERSION;
  readonly sha256: string;
  readonly byteLength: number;
  readonly taskIds: Readonly<Partial<Record<MeshyPipelineStage, string>>>;
}

export interface MeshyRunArtifact {
  readonly file: File;
  readonly provenance: MeshyArtifactProvenance;
}

export interface MeshyBridgeClient {
  health(): Promise<MeshyBridgeHealth>;
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
}

interface StoredPreview extends MeshyRunPreviewRequest, MeshyRunPreview {}

interface StoredRun {
  run: MeshyRun;
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
  private readonly usedConfirmationNonces = new Set<string>();
  private readonly sessions = new Set<string>();
  private pairingCodeUsed = false;
  private readonly availableCredits: number;

  constructor(options: { readonly availableCredits?: number } = {}) {
    this.availableCredits = options.availableCredits ?? 0;
  }

  async health(): Promise<MeshyBridgeHealth> {
    return { protocolVersion: MESHY_BRIDGE_PROTOCOL_VERSION, bridge: "LOCAL", status: "READY" };
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
    if (!profile || !input.prompt.trim()) {
      throw new MeshyBridgeError("PREVIEW_NOT_FOUND", "Select a supported profile and provide an asset prompt.");
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
    return preview;
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

  private maximumCredits(profile: MeshyProfile) {
    return profile.id.startsWith("H1") ? 38 : 30;
  }
}

function isH1PreflightComplete(value: MeshyRunPreviewRequest["h1Preflight"]) {
  return value?.standardHumanoid === true && value.clearLimbs === true && value.noWeapon === true;
}

/** Browser adapter for the owner-operated loopback Bridge. It never talks to api.meshy.ai. */
export class LocalMeshyBridgeClient implements MeshyBridgeClient {
  constructor(private readonly origin = "http://127.0.0.1:43119") {}

  async health(): Promise<MeshyBridgeHealth> {
    return this.json("/v1/health") as Promise<MeshyBridgeHealth>;
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
    const encodedRunId = encodeURIComponent(runId);
    const provenance = await this.provenance(sessionToken, runId);
    const response = await fetch(`${this.origin}/v1/runs/${encodedRunId}/artifact`, {
      headers: { "X-Meshy-Session": sessionToken },
    });
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
