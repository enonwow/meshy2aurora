import { createHash, randomBytes, randomUUID } from "node:crypto";
import { createServer } from "node:http";
import { fileURLToPath } from "node:url";
import {
  MESHY_ANIMATION_ACTION_IDS_V1 as ANIMATION_ACTION_IDS,
  MESHY_ANIMATION_CATALOG_V1 as ANIMATION_CATALOG,
} from "./animation-catalog.mjs";
import { inspectGlbArtifactV1 } from "./glb-artifact-inspection.mjs";

const PROTOCOL_VERSION = 1;
const API_ORIGIN = "https://api.meshy.ai";
const SESSION_TTL_MS = 15 * 60_000;
const MAX_BODY_BYTES = 100 * 1024 * 1024;
const MAX_IMAGE_DATA_URL_CHARS = 27 * 1024 * 1024;
const MAX_THUMBNAIL_BYTES = 8 * 1024 * 1024;
const HISTORY_PAGE_SIZE_MAX = 50;
const POLL_INTERVAL_MS = 3_000;
const MAX_POLLS = 180;
const AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000;
const GEOMETRY_TARGETS = new Set(["AURORA_PROOF", "LOWER_DETAIL", "BALANCED", "HIGHER_DETAIL"]);
const IMAGE_MODELS = new Set(["nano-banana", "nano-banana-2", "nano-banana-pro", "gpt-image-2"]);
const IMAGE_RATIOS = new Set(["1:1", "16:9", "9:16", "4:3", "3:4"]);
const GPT_IMAGE_RATIOS = new Set(["1:1", "3:2", "2:3"]);
const TEXTURE_RESOLUTIONS = new Set(["2k", "4k", "8k"]);
const MESHY_HUMANOID_RIG_TRIANGLE_LIMIT_V1 =
  AURORA_MODEL_TRIANGLE_BUDGET_V1;

const PROFILES = [
  { id: "H1-humanoid-animated/v1", label: "Humanoid Animated", description: "Textured standard humanoid with rigging and one Idle animation proof.", stages: ["PREVIEW", "REFINE", "RIG", "ANIMATE"], expectedOutput: { texture: true, rigging: true, animation: "IDLE" } },
  { id: "N1-quadruped/v1", label: "Quadruped", description: "Textured non-humanoid proof asset without auto-rigging.", stages: ["PREVIEW", "REFINE"], expectedOutput: { texture: true, rigging: false, animation: null } },
  { id: "S1-static-prop/v1", label: "Static Prop", description: "Textured static proof asset without a skeleton or animation.", stages: ["PREVIEW", "REFINE"], expectedOutput: { texture: true, rigging: false, animation: null } },
];

function bridgeError(code, message, status = 400) {
  return { code, message, status };
}

function json(response, status, body, origin) {
  response.writeHead(status, {
    "Content-Type": "application/json; charset=utf-8",
    "Cache-Control": "no-store",
    "Access-Control-Allow-Origin": origin,
    "Access-Control-Allow-Headers": "Content-Type, X-Meshy-Session",
    "Vary": "Origin",
  });
  response.end(JSON.stringify(body));
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function profileById(id) {
  return PROFILES.find((profile) => profile.id === id);
}

function animationActionIds(options) {
  if (Array.isArray(options?.animationActionIds)) return options.animationActionIds;
  if (Number.isInteger(options?.animationActionId)) return [options.animationActionId];
  return [0];
}

function maximumCredits(profile, options, source) {
  const textureCredits = options?.textureResolution === "8k" ? 15 : 10;
  let generationCredits;
  if (source === "TEXT") {
    const previewCredits = (
      options?.aiModel === "meshy-5"
      && options?.modelType !== "lowpoly"
    ) ? 5 : 20;
    generationCredits = previewCredits + textureCredits;
  } else {
    const geometryCredits = options?.aiModel === "meshy-t2"
      || options?.aiModel === "meshy-5"
      ? 5
      : 20;
    generationCredits = geometryCredits
      + (options?.shouldTexture === false ? 0 : textureCredits);
  }
  return generationCredits + (
    profile.id.startsWith("H1")
      ? 5 + 3 * animationActionIds(options).length
      : 0
  );
}

function targetPolycount(target) {
  return target === "AURORA_PROOF"
    ? 1_500
    : target === "LOWER_DETAIL"
      ? 10_000
      : target === "BALANCED"
        ? 15_000
        : AURORA_MODEL_TRIANGLE_BUDGET_V1;
}

function validImageDataUri(value) {
  return typeof value === "string" && /^data:image\/(?:png|jpeg);base64,[a-z0-9+/]+=*$/i.test(value) && value.length <= MAX_IMAGE_DATA_URL_CHARS;
}

function validImageRunRequest(body) {
  if (!body || !["TEXT_TO_IMAGE", "IMAGE_TO_IMAGE"].includes(body.mode) || !IMAGE_MODELS.has(body.aiModel)) return false;
  if (typeof body.prompt !== "string" || !body.prompt.trim()) return false;
  if (typeof body.generateMultiView !== "boolean" || !["", "a-pose", "t-pose"].includes(body.poseMode ?? "")) return false;
  if (body.generateMultiView && body.aspectRatio !== undefined) return false;
  if (!body.generateMultiView && body.aspectRatio !== undefined) {
    const allowedRatios = body.aiModel === "gpt-image-2" ? GPT_IMAGE_RATIOS : IMAGE_RATIOS;
    if (!allowedRatios.has(body.aspectRatio)) return false;
  }
  if (body.mode === "TEXT_TO_IMAGE") return true;
  return Array.isArray(body.referenceImageDataUrls) && body.referenceImageDataUrls.length >= 1 && body.referenceImageDataUrls.length <= 5 && body.referenceImageDataUrls.every(validImageDataUri);
}

function validRetextureRequest(body) {
  return Boolean(body)
    && validInputTaskId(body.inputTaskId)
    && typeof body.textStylePrompt === "string"
    && body.textStylePrompt.trim().length >= 1
    && body.textStylePrompt.length <= 600
    && ["meshy-5", "meshy-6"].includes(body.aiModel)
    && TEXTURE_RESOLUTIONS.has(body.textureResolution)
    && (body.aiModel !== "meshy-5" || body.textureResolution === "2k")
    && ["enableOriginalUv", "enablePbr", "removeLighting", "alphaThumbnail"].every((key) => typeof body[key] === "boolean");
}

function imageRunCredits(aiModel, mode) {
  if (mode === "IMAGE_TO_IMAGE" && aiModel === "gpt-image-2") return 12;
  return aiModel === "nano-banana" ? 3 : aiModel === "nano-banana-2" ? 6 : 9;
}

function validApiOptions(options, source) {
  if (options === undefined) return true;
  if (!options || typeof options !== "object") return false;
  if (!["standard", "lowpoly", "smart-topology"].includes(options.modelType)) return false;
  if (options.modelType === "smart-topology" && (source !== "IMAGE" || !["meshy-t1", "meshy-t2"].includes(options.aiModel))) return false;
  if (options.modelType !== "smart-topology" && !["latest", "meshy-5", "meshy-6"].includes(options.aiModel)) return false;
  if (source === "MULTI_IMAGE" && options.modelType !== "standard") return false;
  if (typeof options.shouldRemesh !== "boolean" || !["triangle", "quad"].includes(options.topology)) return false;
  const maxPolycount = options.modelType === "smart-topology" && options.aiModel === "meshy-t2"
    ? 15_000
    : AURORA_MODEL_TRIANGLE_BUDGET_V1;
  if (!Number.isInteger(options.targetPolycount) || options.targetPolycount < 100 || options.targetPolycount > maxPolycount) return false;
  if (options.decimationMode !== undefined && ![1, 2, 3, 4].includes(options.decimationMode)) return false;
  if (!["", "a-pose", "t-pose"].includes(options.poseMode) || typeof options.moderation !== "boolean") return false;
  if (!Array.isArray(options.targetFormats) || options.targetFormats.length !== 1 || options.targetFormats[0] !== "glb") return false;
  if (!["alphaThumbnail", "autoSize", "enablePbr", "shouldTexture", "removeLighting", "imageEnhancement", "multiViewThumbnails", "rigHumanoid"].every((key) => typeof options[key] === "boolean")) return false;
  if (source === "TEXT" && !options.shouldTexture) return false;
  if (!TEXTURE_RESOLUTIONS.has(options.textureResolution)) return false;
  if (options.aiModel === "meshy-5" && options.textureResolution !== "2k") return false;
  if (options.textureResolution === "8k" && options.topology !== "triangle") return false;
  if (!["bottom", "center"].includes(options.originAt) || typeof options.texturePrompt !== "string" || options.texturePrompt.length > 600 || typeof options.textureImageUrl !== "string") return false;
  if (!Number.isFinite(options.rigHeightMeters) || options.rigHeightMeters < 0.5 || options.rigHeightMeters > 3) return false;
  if (options.rigHumanoid) {
    const actionIds = animationActionIds(options);
    if (
      actionIds.length < 1
      || actionIds.length > 10
      || actionIds.some((value) => !ANIMATION_ACTION_IDS.has(value))
      || new Set(actionIds).size !== actionIds.length
      || options.animationCatalogSnapshotId !== ANIMATION_CATALOG.snapshotId
      || options.animationCatalogSha256 !== ANIMATION_CATALOG.curatedActionsSha256
      || !["a-pose", "t-pose"].includes(options.poseMode)
      || options.shouldTexture !== true
      || options.targetPolycount > MESHY_HUMANOID_RIG_TRIANGLE_LIMIT_V1
    ) return false;
  }
  return !options.rigHumanoid || source !== "MULTI_IMAGE";
}

function safeRun(run) {
  const { artifactBytes, artifacts, imageDataUrls, apiOptions, inputTaskId, ...safe } = run;
  return safe;
}

function safePreview(preview) {
  const { imageDataUrls, apiOptions, ...safe } = preview;
  return safe;
}

function validInputTaskId(value) {
  return typeof value === "string" && /^[A-Za-z0-9_-]{1,128}$/.test(value);
}

function safeImageRun(run) {
  return {
    id: run.id,
    mode: run.mode,
    prompt: run.prompt,
    aiModel: run.aiModel,
    generateMultiView: run.generateMultiView,
    status: run.status,
    progress: run.progress,
    ...(run.taskId ? { taskId: run.taskId } : {}),
    ...(run.error ? { error: run.error } : {}),
  };
}

function safeRetextureRun(run) {
  const { artifactBytes, provenance, ...safe } = run;
  return safe;
}

function verifyGlbBytes(bytes) {
  if (bytes.byteLength < 4 || bytes.byteLength > 512 * 1024 * 1024) {
    throw bridgeError("ARTIFACT_INVALID", "The GLB violates the local Bridge size gate.");
  }
  if (bytes[0] !== 0x67 || bytes[1] !== 0x6c || bytes[2] !== 0x54 || bytes[3] !== 0x46) {
    throw bridgeError("ARTIFACT_INVALID", "The downloaded artifact is not a binary glTF file.");
  }
  try {
    const inspection = inspectGlbArtifactV1(bytes);
    if (inspection.status !== "PARSED") {
      throw new Error(inspection.reason);
    }
  } catch (error) {
    throw bridgeError(
      "ARTIFACT_INVALID",
      `The downloaded GLB failed structural readback: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
}

function historyArtifacts(task, stage) {
  if (stage === "REFINE" && typeof task.model_urls?.glb === "string") {
    return [{ key: "model", role: "MODEL" }];
  }
  if (stage === "RIG") {
    return [
      ...(typeof (task.rigged_character_glb_url ?? task.result?.rigged_character_glb_url) === "string"
        ? [{ key: "rigged-character", role: "RIGGED_CHARACTER" }]
        : []),
      ...(typeof (task.basic_animations?.walking_glb_url ?? task.result?.basic_animations?.walking_glb_url) === "string"
        ? [{ key: "basic-walking", role: "BASIC_WALKING" }]
        : []),
      ...(typeof (task.basic_animations?.running_glb_url ?? task.result?.basic_animations?.running_glb_url) === "string"
        ? [{ key: "basic-running", role: "BASIC_RUNNING" }]
        : []),
    ];
  }
  if (
    stage === "ANIMATE"
    && typeof (task.animation_glb_url ?? task.result?.animation_glb_url) === "string"
  ) {
    return [{
      key: `animation-${task.action_id}`,
      role: "ANIMATION",
      ...(Number.isSafeInteger(task.action_id) ? { actionId: task.action_id } : {}),
    }];
  }
  return [];
}

function historyItem(task, forcedStage) {
  const stage = forcedStage
    ?? (task.type === "text-to-3d-refine" ? "REFINE" : "PREVIEW");
  const artifacts = historyArtifacts(task, stage);
  return {
    taskId: task.id,
    stage,
    status: task.status,
    prompt: typeof task.prompt === "string" ? task.prompt : "",
    createdAt: Number.isFinite(task.created_at) ? new Date(task.created_at).toISOString() : undefined,
    finishedAt: Number.isFinite(task.finished_at) ? new Date(task.finished_at).toISOString() : undefined,
    consumedCredits: Number.isFinite(task.consumed_credits) ? task.consumed_credits : undefined,
    glbAvailable: artifacts.length > 0,
    thumbnailAvailable: task.type === "text-to-3d-refine" && task.status === "SUCCEEDED" && typeof historyThumbnailUrl(task) === "string",
    ...(Number.isSafeInteger(task.action_id) ? { actionId: task.action_id } : {}),
    ...(artifacts.length ? { artifacts } : {}),
  };
}

function historyArtifactUrl(task, stage, key) {
  if (stage === "REFINE" && key === "model") return task.model_urls?.glb;
  if (stage === "RIG" && key === "rigged-character") {
    return task.rigged_character_glb_url ?? task.result?.rigged_character_glb_url;
  }
  if (stage === "RIG" && key === "basic-walking") {
    return task.basic_animations?.walking_glb_url
      ?? task.result?.basic_animations?.walking_glb_url;
  }
  if (stage === "RIG" && key === "basic-running") {
    return task.basic_animations?.running_glb_url
      ?? task.result?.basic_animations?.running_glb_url;
  }
  if (stage === "ANIMATE" && key === `animation-${task.action_id}`) {
    return task.animation_glb_url ?? task.result?.animation_glb_url;
  }
  return undefined;
}

function historyThumbnailUrl(task) {
  const candidates = [task.thumbnail_url, task.thumbnail_urls?.[0], task.model_urls?.thumbnail, task.model_urls?.thumbnail_url];
  return candidates.find((candidate) => typeof candidate === "string");
}

/**
 * Owner-operated loopback proxy. This module purposefully exposes only the
 * Meshy Lab contract; it is never a generic HTTP proxy.
 */
export function createLocalBridge({
  apiKey,
  pairingCode = randomBytes(18).toString("base64url"),
  allowedOrigin,
  bindHost = "127.0.0.1",
  allowContainerBind = false,
  meshFetch = fetch,
  startRuns = true,
  requestRestart,
  allowAutomaticPairing = false,
} = {}) {
  if (!apiKey) throw new Error("MESHY_API_KEY is required by the local Bridge process.");
  if (!allowedOrigin) throw new Error("MESHY_BRIDGE_ALLOWED_ORIGIN is required and must be an exact Studio origin.");
  const loopback = bindHost === "127.0.0.1" || bindHost === "::1";
  const explicitContainerBind = bindHost === "0.0.0.0" && allowContainerBind === true;
  if (!loopback && !explicitContainerBind) {
    throw new Error("MESHY_BRIDGE_BIND_HOST must be loopback, unless the explicit Docker container bind is enabled.");
  }

  const sessions = new Map();
  const previews = new Map();
  const runs = new Map();
  const imagePreviews = new Map();
  const imageRuns = new Map();
  const retexturePreviews = new Map();
  const retextureRuns = new Map();
  const recoveredArtifacts = new Map();
  const recoveredThumbnails = new Map();
  const meshResponseMetadata = new WeakMap();
  const usedNonces = new Set();
  let pairingCodeUsed = false;

  const mintSession = () => {
    const sessionToken = randomBytes(32).toString("base64url");
    const expiresAt = Date.now() + SESSION_TTL_MS;
    sessions.set(sessionToken, expiresAt);
    return { sessionToken, expiresAt: new Date(expiresAt).toISOString() };
  };

  const requireOrigin = (request) => {
    const origin = request.headers.origin;
    if (origin !== allowedOrigin) throw bridgeError("BRIDGE_UNAVAILABLE", "This browser origin is not paired with the local Bridge.", 403);
    return origin;
  };

  const requireSession = (request) => {
    const token = request.headers["x-meshy-session"];
    const expiresAt = typeof token === "string" ? sessions.get(token) : undefined;
    if (!expiresAt || expiresAt <= Date.now()) {
      if (typeof token === "string") sessions.delete(token);
      throw bridgeError("PAIRING_REQUIRED", "Pair this browser session with the local Bridge first.", 401);
    }
  };

  const meshJson = async (path, options = {}) => {
    const safeGet = (options.method ?? "GET") === "GET";
    for (let attempt = 0; attempt < (safeGet ? 3 : 1); attempt += 1) {
      const response = await meshFetch(`${API_ORIGIN}${path}`, {
        ...options,
        headers: {
          Authorization: `Bearer ${apiKey}`,
          ...(options.body ? { "Content-Type": "application/json" } : {}),
        },
      });
      if (response.status === 429 && safeGet && attempt < 2) {
        const retryAfterSeconds = Number(response.headers.get("retry-after"));
        const delay = Number.isFinite(retryAfterSeconds)
          ? Math.min(5_000, Math.max(0, retryAfterSeconds * 1_000))
          : 250 * 2 ** attempt + Math.floor(Math.random() * 75);
        await sleep(delay);
        continue;
      }
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) {
        const code = response.status === 402 ? "INSUFFICIENT_CREDITS" : "TASK_REJECTED";
        throw bridgeError(code, payload.message || `Meshy request failed with status ${response.status}.`, response.status);
      }
      if (typeof payload === "object" && payload !== null) {
        meshResponseMetadata.set(payload, {
          apiVersion: response.headers.get("x-api-version") ?? undefined,
        });
      }
      return payload;
    }
    throw bridgeError(
      "TASK_REJECTED",
      "Meshy GET retry budget was exhausted.",
      429,
    );
  };

  const downloadVerifiedGlb = async (assetUrl, failureMessage) => {
    for (let attempt = 1; attempt <= 3; attempt += 1) {
      const response = await meshFetch(assetUrl);
      if (response.ok) {
        const bytes = new Uint8Array(await response.arrayBuffer());
        verifyGlbBytes(bytes);
        return bytes;
      }
      if (attempt < 3) await sleep(1_000 * attempt);
    }
    throw bridgeError("ARTIFACT_INVALID", failureMessage);
  };

  const assertTrackingActive = (run) => {
    if (run.status === "STOPPED_LOCAL") {
      throw bridgeError(
        "STOPPED_LOCAL",
        "Local tracking stopped. Already-created Meshy tasks may continue remotely.",
      );
    }
  };

  const taskTimestamp = (value) => {
    if (typeof value === "string" && value.trim()) return value;
    if (!Number.isFinite(value)) return undefined;
    const milliseconds = value < 10_000_000_000 ? value * 1_000 : value;
    return new Date(milliseconds).toISOString();
  };

  const taskMetadata = (task) => ({
    ...(meshResponseMetadata.get(task)?.apiVersion
      ? { apiVersion: meshResponseMetadata.get(task).apiVersion }
      : {}),
    ...(taskTimestamp(task?.created_at)
      ? { createdAt: taskTimestamp(task.created_at) }
      : {}),
    ...(taskTimestamp(task?.finished_at)
      ? { finishedAt: taskTimestamp(task.finished_at) }
      : {}),
    ...(taskTimestamp(task?.expires_at)
      ? { expiresAt: taskTimestamp(task.expires_at) }
      : {}),
    ...(Number.isFinite(task?.consumed_credits)
      ? { consumedCredits: task.consumed_credits }
      : {}),
  });

  const recordRunTask = (run, {
    stage,
    taskId,
    task,
    actionId,
  }) => {
    run.taskLedger ??= [];
    const existingIndex = run.taskLedger.findIndex(
      (entry) => entry.taskId === taskId,
    );
    const existing = existingIndex >= 0 ? run.taskLedger[existingIndex] : {};
    const next = {
      ...existing,
      stage,
      taskId,
      ...(actionId === undefined ? {} : { actionId }),
      status: typeof task?.status === "string"
        ? task.status
        : existing.status ?? "CREATED",
      ...taskMetadata(task),
    };
    if (existingIndex >= 0) run.taskLedger[existingIndex] = next;
    else run.taskLedger.push(next);
    return next;
  };

  const assertArtifactRoleReadback = (role, glbReadback) => {
    if (glbReadback.status !== "PARSED") {
      throw bridgeError(
        "ARTIFACT_INVALID",
        `Meshy ${role.toLowerCase()} GLB did not pass structural readback.`,
      );
    }
    if (
      ["RIGGED_CHARACTER", "BASIC_WALKING", "BASIC_RUNNING", "ANIMATION"]
        .includes(role)
      && (
        glbReadback.skeleton.jointCount === 0
        || glbReadback.skeleton.skinnedMeshNodeCount === 0
        || glbReadback.skeleton.weightedVertexCount === 0
      )
    ) {
      throw bridgeError(
        "ARTIFACT_INVALID",
        `Meshy ${role.toLowerCase()} GLB contains no actual weighted skeleton binding.`,
      );
    }
    if (
      ["BASIC_WALKING", "BASIC_RUNNING", "ANIMATION"].includes(role)
      && !glbReadback.clipInventory.clips.some((clip) => (
        clip.channelCount > 0
        && clip.jointChannelCount > 0
        && clip.samplerCount > 0
        && clip.durationSeconds > 0
      ))
    ) {
      throw bridgeError(
        "ARTIFACT_INVALID",
        `Meshy ${role.toLowerCase()} GLB contains no playable animation clip.`,
      );
    }
  };

  const recordRunArtifact = (run, {
    key,
    role,
    taskId,
    bytes,
    actionId,
    task,
  }) => {
    const glbReadback = inspectGlbArtifactV1(bytes);
    assertArtifactRoleReadback(role, glbReadback);
    const baseSkeleton = run.artifacts.find(
      ({ role, glbReadback: readback }) => (
        role === "RIGGED_CHARACTER" && readback.status === "PARSED"
      ),
    )?.glbReadback.skeleton;
    if (
      baseSkeleton
      && glbReadback.status === "PARSED"
      && role !== "MODEL"
      && glbReadback.skeleton.signatureSha256
        !== baseSkeleton.signatureSha256
    ) {
      throw bridgeError(
        "ARTIFACT_INVALID",
        `Meshy ${role.toLowerCase()} GLB uses a different skeleton from the preserved rigged base.`,
      );
    }
    const artifact = {
      key,
      role,
      taskId,
      bytes,
      ...(actionId === undefined ? {} : { actionId }),
      sha256: createHash("sha256").update(bytes).digest("hex"),
      byteLength: bytes.byteLength,
      glbReadback,
      task: taskMetadata(task),
    };
    run.artifacts.push(artifact);
    return artifact;
  };

  const refreshRunProvenance = (run) => {
    const root = run.artifacts.find(({ role }) => role === "RIGGED_CHARACTER")
      ?? run.artifacts.find(({ role }) => role === "MODEL")
      ?? run.artifacts[0];
    if (!root) return;
    const sourceModel = run.artifacts.find(({ role }) => role === "MODEL");
    const riggedBase = run.artifacts.find(
      ({ role }) => role === "RIGGED_CHARACTER",
    );
    const uniqueTasks = run.taskLedger ?? [];
    run.artifactBytes = root.bytes;
    run.provenance = {
      profileId: run.profile.id,
      bridgeProtocolVersion: PROTOCOL_VERSION,
      sha256: root.sha256,
      byteLength: root.byteLength,
      taskIds: run.taskIds,
      sourceTaskId: sourceModel?.taskId,
      sourceModelSha256: sourceModel?.sha256,
      rigTaskId: riggedBase?.taskId,
      riggedBaseSha256: riggedBase?.sha256,
      rigHeightMeters: run.apiOptions?.rigHeightMeters,
      poseMode: run.apiOptions?.poseMode,
      createdAt: run.createdAt,
      updatedAt: run.updatedAt,
      apiVersions: [...new Set(
        uniqueTasks.map(({ apiVersion }) => apiVersion).filter(Boolean),
      )],
      consumedCredits: uniqueTasks.reduce(
        (total, task) => total + (task.consumedCredits ?? 0),
        0,
      ),
      taskLedger: uniqueTasks.map((task) => ({ ...task })),
      animationCatalog: {
        snapshotId: ANIMATION_CATALOG.snapshotId,
        sourceCatalogSha256: ANIMATION_CATALOG.sourceCatalogSha256,
        curatedActionsSha256: ANIMATION_CATALOG.curatedActionsSha256,
        sourceUrl: ANIMATION_CATALOG.sourceUrl,
        capturedAt: ANIMATION_CATALOG.capturedAt,
      },
      artifacts: run.artifacts.map(({
        key,
        role,
        taskId,
        sha256,
        byteLength,
        actionId,
        glbReadback,
        task,
      }) => ({
        key,
        role,
        taskId,
        sha256,
        byteLength,
        ...(actionId === undefined ? {} : { actionId }),
        glbReadback,
        task,
      })),
      animationArtifacts: run.artifacts
        .filter(({ role }) => role === "ANIMATION")
        .map(({ actionId, sha256, byteLength, taskId, glbReadback, task }) => {
          const catalogAction = ANIMATION_CATALOG.actions.find(
            ({ id }) => id === actionId,
          );
          return {
          actionId,
          actionName: catalogAction.name,
          actionCategory: catalogAction.category,
          auroraCandidate: catalogAction.auroraCandidate,
          taskId,
          sha256,
          byteLength,
          glbReadback,
          task,
          };
        }),
    };
  };

  const waitForTask = async (
    run,
    stage,
    endpoint,
    taskId,
    actionId,
  ) => {
    for (let poll = 0; poll < MAX_POLLS; poll += 1) {
      assertTrackingActive(run);
      const task = await meshJson(`${endpoint}/${encodeURIComponent(taskId)}`);
      recordRunTask(run, { stage, taskId, task, actionId });
      assertTrackingActive(run);
      run.progress = Math.min(99, Math.max(run.progress, Number(task.progress) || 0));
      run.updatedAt = new Date().toISOString();
      if (task.status === "SUCCEEDED") return task;
      if (task.status === "FAILED") throw bridgeError("TASK_FAILED", task.task_error?.message || `Meshy ${stage.toLowerCase()} task failed.`);
      if (task.status === "CANCELED") {
        throw bridgeError(
          "TASK_CANCELED",
          `Meshy ${stage.toLowerCase()} task was canceled remotely.`,
        );
      }
      await sleep(POLL_INTERVAL_MS);
    }
    throw bridgeError("TASK_FAILED", `Meshy ${stage.toLowerCase()} task timed out in the local Bridge.`);
  };

  const recoverArtifact = async (stage, taskId, requestedKey) => {
    const endpoint = stage === "REFINE"
      ? "/openapi/v2/text-to-3d"
      : stage === "RIG"
        ? "/openapi/v1/rigging"
        : stage === "ANIMATE"
          ? "/openapi/v1/animations"
          : undefined;
    if (!endpoint) {
      throw bridgeError(
        "ARTIFACT_NOT_READY",
        "Preview tasks do not expose a recoverable GLB.",
        409,
      );
    }
    const task = await meshJson(`${endpoint}/${encodeURIComponent(taskId)}`);
    const available = historyArtifacts(task, stage);
    const defaultKey = stage === "REFINE"
      ? "model"
      : stage === "RIG"
        ? "rigged-character"
        : available[0]?.key;
    const artifactKey = requestedKey ?? defaultKey;
    const cacheKey = `${stage}:${taskId}:${artifactKey}`;
    const cached = recoveredArtifacts.get(cacheKey);
    if (cached) return cached;
    if (task.status !== "SUCCEEDED" || !artifactKey) {
      throw bridgeError("ARTIFACT_NOT_READY", "Only a completed Meshy task with a verified GLB can be recovered.", 409);
    }
    const selected = available.find(({ key }) => key === artifactKey);
    const assetUrl = historyArtifactUrl(task, stage, artifactKey);
    if (!selected || typeof assetUrl !== "string") {
      throw bridgeError("ARTIFACT_NOT_READY", "The requested Meshy task artifact is not available.", 404);
    }
    const artifactBytes = await downloadVerifiedGlb(
      assetUrl,
      "The signed Meshy history GLB download failed.",
    );
    const sha256 = createHash("sha256").update(artifactBytes).digest("hex");
    const glbReadback = inspectGlbArtifactV1(artifactBytes);
    const recoveredTaskMetadata = taskMetadata(task);
    const profileId = stage === "REFINE"
      ? "RECOVERED-text-to-3d/v1"
      : stage === "RIG"
        ? "RECOVERED-rigging/v1"
        : "RECOVERED-animation/v1";
    const recovered = {
      artifactBytes,
      provenance: {
        profileId,
        bridgeProtocolVersion: PROTOCOL_VERSION,
        sha256,
        byteLength: artifactBytes.byteLength,
        taskIds: { [stage]: taskId },
        artifacts: [{
          ...selected,
          taskId,
          sha256,
          byteLength: artifactBytes.byteLength,
          glbReadback,
          task: recoveredTaskMetadata,
        }],
        selectedArtifactKey: artifactKey,
        sourceTaskId: stage === "REFINE" ? taskId : undefined,
        sourceModelSha256: stage === "REFINE" ? sha256 : undefined,
        rigTaskId: stage === "RIG" || stage === "ANIMATE"
          ? (task.rig_task_id ?? (stage === "RIG" ? taskId : undefined))
          : undefined,
        riggedBaseSha256: stage === "RIG"
          && selected.role === "RIGGED_CHARACTER"
          ? sha256
          : undefined,
        createdAt: recoveredTaskMetadata.createdAt,
        updatedAt: recoveredTaskMetadata.finishedAt,
        apiVersions: recoveredTaskMetadata.apiVersion
          ? [recoveredTaskMetadata.apiVersion]
          : [],
        consumedCredits: recoveredTaskMetadata.consumedCredits ?? 0,
        ...(stage === "ANIMATE" ? {
          animationCatalog: {
            snapshotId: ANIMATION_CATALOG.snapshotId,
            sourceCatalogSha256: ANIMATION_CATALOG.sourceCatalogSha256,
            curatedActionsSha256: ANIMATION_CATALOG.curatedActionsSha256,
            sourceUrl: ANIMATION_CATALOG.sourceUrl,
            capturedAt: ANIMATION_CATALOG.capturedAt,
          },
          animationArtifacts: [{
            actionId: selected.actionId,
            actionName: ANIMATION_CATALOG.actions.find(
              ({ id }) => id === selected.actionId,
            )?.name,
            actionCategory: ANIMATION_CATALOG.actions.find(
              ({ id }) => id === selected.actionId,
            )?.category,
            auroraCandidate: ANIMATION_CATALOG.actions.find(
              ({ id }) => id === selected.actionId,
            )?.auroraCandidate,
            taskId,
            sha256,
            byteLength: artifactBytes.byteLength,
            glbReadback,
            task: recoveredTaskMetadata,
          }],
          selectedAnimationActionId: selected.actionId,
        } : {}),
      },
    };
    recoveredArtifacts.set(cacheKey, recovered);
    return recovered;
  };

  const recoverHistoryThumbnail = async (taskId) => {
    const cached = recoveredThumbnails.get(taskId);
    if (cached) return cached;
    const task = await meshJson(`/openapi/v2/text-to-3d/${encodeURIComponent(taskId)}`);
    if (task.type !== "text-to-3d-refine" || task.status !== "SUCCEEDED") {
      throw bridgeError("ARTIFACT_NOT_READY", "Only a completed Text-to-3D refine task has a library thumbnail.", 409);
    }
    const thumbnailUrl = historyThumbnailUrl(task);
    if (!thumbnailUrl) throw bridgeError("ARTIFACT_NOT_READY", "Meshy did not expose a thumbnail for this task.", 404);
    const thumbnailResponse = await meshFetch(thumbnailUrl);
    const contentType = thumbnailResponse.headers.get("content-type")?.split(";", 1)[0].toLowerCase();
    if (!thumbnailResponse.ok || !contentType || !["image/png", "image/jpeg", "image/webp"].includes(contentType)) {
      throw bridgeError("ARTIFACT_INVALID", "The Meshy thumbnail is not a supported image.");
    }
    const bytes = new Uint8Array(await thumbnailResponse.arrayBuffer());
    if (!bytes.byteLength || bytes.byteLength > MAX_THUMBNAIL_BYTES) throw bridgeError("ARTIFACT_INVALID", "The Meshy thumbnail violates the local Bridge size gate.");
    const recovered = { bytes, contentType };
    recoveredThumbnails.set(taskId, recovered);
    return recovered;
  };

  const executeRun = async (run) => {
    try {
      const options = run.apiOptions;
      const source = run.source || "TEXT";
      run.artifacts = [];
      run.animationOutcomes = [];
      run.taskLedger = [];
      run.status = "PREVIEWING";
      const generationEndpoint = source === "TEXT" ? "/openapi/v2/text-to-3d" : source === "IMAGE" ? "/openapi/v1/image-to-3d" : "/openapi/v1/multi-image-to-3d";
      const preview = await meshJson(generationEndpoint, {
        method: "POST",
        body: JSON.stringify(source === "TEXT" ? {
          mode: "preview", prompt: run.prompt,
          ai_model: options?.aiModel ?? "meshy-6",
          ...(source === "IMAGE" ? { model_type: options?.modelType ?? "standard" } : {}),
          moderation: options?.moderation ?? true,
          should_remesh: options?.shouldRemesh ?? true,
          topology: options?.topology ?? "triangle",
          target_polycount: options?.targetPolycount ?? targetPolycount(run.geometryTarget),
          ...(options?.decimationMode ? { decimation_mode: options.decimationMode } : {}),
          pose_mode: options?.poseMode ?? (run.profile.id.startsWith("H1") ? "a-pose" : ""),
          target_formats: options?.targetFormats ?? ["glb"],
          alpha_thumbnail: options?.alphaThumbnail ?? false,
          multi_view_thumbnails: options?.multiViewThumbnails ?? false,
          auto_size: options?.autoSize ?? false,
          ...(options?.autoSize ? { origin_at: options.originAt } : {}),
        } : {
          ...(run.inputTaskId ? { input_task_id: run.inputTaskId } : source === "IMAGE" ? { image_url: run.imageDataUrls[0] } : { image_urls: run.imageDataUrls }),
          ai_model: options?.aiModel ?? "meshy-6",
          model_type: options?.modelType ?? "standard",
          should_texture: options?.shouldTexture ?? true,
          enable_pbr: options?.enablePbr ?? true,
          texture_resolution: options?.textureResolution ?? "2k",
          ...(options?.texturePrompt ? { texture_prompt: options.texturePrompt } : options?.textureImageUrl ? { texture_image_url: options.textureImageUrl } : {}),
          moderation: options?.moderation ?? true,
          image_enhancement: options?.imageEnhancement ?? true,
          remove_lighting: options?.removeLighting ?? true,
          should_remesh: options?.shouldRemesh ?? true,
          topology: options?.topology ?? "triangle",
          target_polycount: options?.targetPolycount ?? targetPolycount(run.geometryTarget),
          ...(options?.decimationMode ? { decimation_mode: options.decimationMode } : {}),
          pose_mode: options?.poseMode ?? "",
          target_formats: options?.targetFormats ?? ["glb"],
          alpha_thumbnail: options?.alphaThumbnail ?? false,
          multi_view_thumbnails: options?.multiViewThumbnails ?? false,
          auto_size: options?.autoSize ?? false,
          ...(options?.autoSize ? { origin_at: options.originAt } : {}),
        }),
      });
      run.taskIds.PREVIEW = preview.result;
      recordRunTask(run, {
        stage: "PREVIEW",
        taskId: preview.result,
        task: preview,
      });
      assertTrackingActive(run);
      let output = await waitForTask(run, "PREVIEW", generationEndpoint, preview.result);
      let inputTaskId = preview.result;

      if (source === "TEXT") {
        run.status = "REFINING";
        const refined = await meshJson("/openapi/v2/text-to-3d", {
          method: "POST",
          body: JSON.stringify({
            mode: "refine", preview_task_id: preview.result,
            ai_model: options?.aiModel ?? "meshy-6",
            enable_pbr: options?.enablePbr ?? true,
            texture_resolution: options?.textureResolution ?? "2k",
            ...(options?.texturePrompt ? { texture_prompt: options.texturePrompt } : options?.textureImageUrl ? { texture_image_url: options.textureImageUrl } : {}),
            moderation: options?.moderation ?? true,
            remove_lighting: options?.removeLighting ?? true,
            target_formats: options?.targetFormats ?? ["glb"],
            alpha_thumbnail: options?.alphaThumbnail ?? false,
            auto_size: options?.autoSize ?? false,
            ...(options?.autoSize ? { origin_at: options.originAt } : {}),
          }),
        });
        run.taskIds.REFINE = refined.result;
        recordRunTask(run, {
          stage: "REFINE",
          taskId: refined.result,
          task: refined,
        });
        assertTrackingActive(run);
        inputTaskId = refined.result;
        output = await waitForTask(run, "REFINE", "/openapi/v2/text-to-3d", refined.result);
      }

      const shouldRig = options?.rigHumanoid ?? run.profile.id.startsWith("H1");
      if (shouldRig) {
        const sourceModelUrl = output.model_urls?.glb;
        if (typeof sourceModelUrl !== "string") {
          throw bridgeError(
            "ARTIFACT_INVALID",
            "Meshy did not return the source GLB required for the H1 lineage.",
          );
        }
        const sourceModelBytes = await downloadVerifiedGlb(
          sourceModelUrl,
          "The signed Meshy source GLB download failed.",
        );
        assertTrackingActive(run);
        recordRunArtifact(run, {
          key: "source-model",
          role: "MODEL",
          taskId: inputTaskId,
          bytes: sourceModelBytes,
          task: output,
        });
        run.status = "RIGGING";
        const rig = await meshJson("/openapi/v1/rigging", {
          method: "POST", body: JSON.stringify({ input_task_id: inputTaskId, height_meters: options?.rigHeightMeters ?? 1.7 }),
        });
        run.taskIds.RIG = rig.result;
        recordRunTask(run, {
          stage: "RIG",
          taskId: rig.result,
          task: rig,
        });
        assertTrackingActive(run);
        output = await waitForTask(run, "RIG", "/openapi/v1/rigging", rig.result);
        const riggedCharacterUrl = output.rigged_character_glb_url
          ?? output.result?.rigged_character_glb_url;
        if (typeof riggedCharacterUrl !== "string") {
          throw bridgeError(
            "ARTIFACT_INVALID",
            "Meshy Rigging did not return rigged_character_glb_url.",
          );
        }
        const riggedBytes = await downloadVerifiedGlb(
          riggedCharacterUrl,
          "The signed Meshy rigged-character GLB download failed.",
        );
        assertTrackingActive(run);
        recordRunArtifact(run, {
          key: "rigged-character",
          role: "RIGGED_CHARACTER",
          taskId: rig.result,
          bytes: riggedBytes,
          task: output,
        });
        for (const [key, role, url] of [
          [
            "basic-walking",
            "BASIC_WALKING",
            output.basic_animations?.walking_glb_url
              ?? output.result?.basic_animations?.walking_glb_url,
          ],
          [
            "basic-running",
            "BASIC_RUNNING",
            output.basic_animations?.running_glb_url
              ?? output.result?.basic_animations?.running_glb_url,
          ],
        ]) {
          if (typeof url !== "string") continue;
          const bytes = await downloadVerifiedGlb(
            url,
            `The signed Meshy ${role.toLowerCase()} GLB download failed.`,
          );
          assertTrackingActive(run);
          recordRunArtifact(run, {
            key,
            role,
            taskId: rig.result,
            bytes,
            task: output,
          });
        }
        refreshRunProvenance(run);

        const selectedActionIds = animationActionIds(options);
        for (const animationActionId of selectedActionIds) {
          try {
            assertTrackingActive(run);
            run.status = "ANIMATING";
            const animation = await meshJson("/openapi/v1/animations", {
              method: "POST", body: JSON.stringify({ rig_task_id: rig.result, action_id: animationActionId }),
            });
            const taskKey = selectedActionIds.length === 1 ? "ANIMATE" : `ANIMATE_${animationActionId}`;
            run.taskIds[taskKey] = animation.result;
            recordRunTask(run, {
              stage: "ANIMATE",
              taskId: animation.result,
              task: animation,
              actionId: animationActionId,
            });
            assertTrackingActive(run);
            output = await waitForTask(
              run,
              "ANIMATE",
              "/openapi/v1/animations",
              animation.result,
              animationActionId,
            );
            const animationAssetUrl = output.result?.animation_glb_url
              ?? output.animation_glb_url;
            if (typeof animationAssetUrl !== "string") throw bridgeError("ARTIFACT_INVALID", `Meshy did not return the GLB for animation action ${animationActionId}.`);
            const animationBytes = await downloadVerifiedGlb(
              animationAssetUrl,
              `The signed Meshy animation GLB download failed for action ${animationActionId}.`,
            );
            assertTrackingActive(run);
            recordRunArtifact(run, {
              key: `animation-${animationActionId}`,
              role: "ANIMATION",
              taskId: animation.result,
              actionId: animationActionId,
              bytes: animationBytes,
              task: output,
            });
            run.animationOutcomes.push({
              actionId: animationActionId,
              taskId: animation.result,
              status: "READY",
            });
            refreshRunProvenance(run);
          } catch (error) {
            if (run.status === "STOPPED_LOCAL") throw error;
            run.animationOutcomes.push({
              actionId: animationActionId,
              status: error.code === "TASK_CANCELED" ? "CANCELED" : "FAILED",
              error: {
                code: error.code || "TASK_FAILED",
                message: error.message || "Meshy animation task failed.",
              },
            });
          }
        }
      } else {
        const assetUrl = output.model_urls?.glb;
        if (typeof assetUrl !== "string") throw bridgeError("ARTIFACT_INVALID", "Meshy did not return a GLB artifact for this run.");
        const modelBytes = await downloadVerifiedGlb(
          assetUrl,
          "The signed Meshy GLB download failed.",
        );
        assertTrackingActive(run);
        recordRunArtifact(run, {
          key: "model",
          role: "MODEL",
          taskId: inputTaskId,
          bytes: modelBytes,
          task: output,
        });
      }

      run.status = "VERIFYING";
      assertTrackingActive(run);
      refreshRunProvenance(run);
      if (!run.provenance) {
        throw bridgeError("ARTIFACT_INVALID", "No verified GLB artifact was preserved for this run.");
      }
      const failedAnimationCount = run.animationOutcomes.filter(
        ({ status }) => status !== "READY",
      ).length;
      run.status = failedAnimationCount > 0 ? "PARTIAL" : "READY";
      if (failedAnimationCount > 0) {
        run.error = {
          code: "PARTIAL_FAILURE",
          message: `${failedAnimationCount} animation action(s) failed; verified successful artifacts remain downloadable.`,
        };
      }
      run.progress = 100;
      run.updatedAt = new Date().toISOString();
      refreshRunProvenance(run);
    } catch (error) {
      if (run.status !== "STOPPED_LOCAL") {
        run.status = error.code === "TASK_CANCELED" ? "CANCELED" : "FAILED";
        run.error = { code: error.code || "TASK_FAILED", message: error.message || "Meshy run failed." };
        run.updatedAt = new Date().toISOString();
      }
    }
  };

  const executeImageRun = async (run) => {
    try {
      run.status = "GENERATING";
      const endpoint = run.mode === "TEXT_TO_IMAGE" ? "/openapi/v1/text-to-image" : "/openapi/v1/image-to-image";
      const requestBody = {
        ai_model: run.aiModel,
        prompt: run.prompt,
        ...(run.generateMultiView ? { generate_multi_view: true } : { ...(run.aspectRatio ? { aspect_ratio: run.aspectRatio } : {}) }),
        ...(run.poseMode ? { pose_mode: run.poseMode } : {}),
        ...(run.mode === "IMAGE_TO_IMAGE" ? { reference_image_urls: run.referenceImageDataUrls } : {}),
      };
      const created = await meshJson(endpoint, { method: "POST", body: JSON.stringify(requestBody) });
      run.taskId = created.result;
      assertTrackingActive(run);
      const output = await waitForTask(run, "IMAGE", endpoint, created.result);
      if (!Array.isArray(output.image_urls) || !output.image_urls.length || !output.image_urls.every((value) => typeof value === "string")) {
        throw bridgeError("ARTIFACT_INVALID", "Meshy did not return image output for this run.");
      }
      run.imageUrls = output.image_urls;
      run.status = "READY";
      run.progress = 100;
    } catch (error) {
      if (run.status !== "STOPPED_LOCAL") {
        run.status = error.code === "TASK_CANCELED" ? "CANCELED" : "FAILED";
        run.error = { code: error.code || "TASK_FAILED", message: error.message || "Meshy image run failed." };
      }
    } finally {
      run.updatedAt = new Date().toISOString();
    }
  };

  const executeRetextureRun = async (run) => {
    try {
      run.status = "TEXTURING";
      const created = await meshJson("/openapi/v1/retexture", {
        method: "POST",
        body: JSON.stringify({
          input_task_id: run.inputTaskId,
          text_style_prompt: run.textStylePrompt,
          ai_model: run.aiModel,
          enable_original_uv: run.enableOriginalUv,
          enable_pbr: run.enablePbr,
          texture_resolution: run.textureResolution,
          remove_lighting: run.removeLighting,
          target_formats: ["glb"],
          alpha_thumbnail: run.alphaThumbnail,
        }),
      });
      run.taskId = created.result;
      assertTrackingActive(run);
      const output = await waitForTask(run, "RETEXTURE", "/openapi/v1/retexture", created.result);
      const assetUrl = output.model_urls?.glb;
      if (typeof assetUrl !== "string") throw bridgeError("ARTIFACT_INVALID", "Meshy did not return a GLB artifact for ReTexture.");
      run.status = "VERIFYING";
      const assetResponse = await meshFetch(assetUrl);
      if (!assetResponse.ok) throw bridgeError("ARTIFACT_INVALID", "The signed Meshy ReTexture GLB download failed.");
      const artifactBytes = new Uint8Array(await assetResponse.arrayBuffer());
      verifyGlbBytes(artifactBytes);
      assertTrackingActive(run);
      run.artifactBytes = artifactBytes;
      run.provenance = {
        profileId: "RETEXTURED-model/v1",
        bridgeProtocolVersion: PROTOCOL_VERSION,
        sha256: createHash("sha256").update(artifactBytes).digest("hex"),
        byteLength: artifactBytes.byteLength,
        taskIds: { REFINE: created.result },
      };
      run.status = "READY";
      run.progress = 100;
    } catch (error) {
      if (run.status !== "STOPPED_LOCAL") {
        run.status = error.code === "TASK_CANCELED" ? "CANCELED" : "FAILED";
        run.error = { code: error.code || "TASK_FAILED", message: error.message || "Meshy ReTexture task failed." };
      }
    } finally {
      run.updatedAt = new Date().toISOString();
    }
  };

  const server = createServer(async (request, response) => {
    let origin;
    try {
      origin = requireOrigin(request);
      if (request.method === "OPTIONS") {
        response.writeHead(204, { "Access-Control-Allow-Origin": origin, "Access-Control-Allow-Headers": "Content-Type, X-Meshy-Session", "Access-Control-Allow-Methods": "GET, POST, OPTIONS", "Vary": "Origin" });
        response.end();
        return;
      }
      const url = new URL(request.url || "/", "http://127.0.0.1");
      const readBody = async () => {
        let body = "";
        for await (const chunk of request) {
          body += chunk;
          if (Buffer.byteLength(body) > MAX_BODY_BYTES) throw bridgeError("TASK_REJECTED", "Bridge request body is too large.", 413);
        }
        return body ? JSON.parse(body) : {};
      };

      if (request.method === "GET" && url.pathname === "/v1/health") {
        json(response, 200, { protocolVersion: PROTOCOL_VERSION, bridge: "LOCAL", status: "READY", restartSupported: typeof requestRestart === "function", automaticPairingSupported: allowAutomaticPairing }, origin); return;
      }
      if (request.method === "POST" && url.pathname === "/v1/bridge/restart") {
        if (typeof requestRestart !== "function") throw bridgeError("BRIDGE_UNAVAILABLE", "This local Bridge was not started by a restart-capable supervisor.", 501);
        json(response, 202, { status: "RESTARTING" }, origin);
        const timer = setTimeout(requestRestart, 50);
        timer.unref?.();
        return;
      }
      if (request.method === "POST" && url.pathname === "/v1/pair/automatic") {
        if (!allowAutomaticPairing) throw bridgeError("BRIDGE_UNAVAILABLE", "This local Bridge requires a manual pairing code.", 501);
        json(response, 200, mintSession(), origin); return;
      }
      if (request.method === "POST" && url.pathname === "/v1/pair") {
        const body = await readBody();
        if (body.pairingCode !== pairingCode) throw bridgeError("PAIRING_REQUIRED", "The local Bridge pairing code is invalid.", 401);
        if (pairingCodeUsed) throw bridgeError("PAIRING_REQUIRED", "The local Bridge pairing code has already been used.", 401);
        pairingCodeUsed = true;
        json(response, 200, mintSession(), origin); return;
      }

      requireSession(request);
      if (request.method === "GET" && url.pathname === "/v1/balance") {
        const result = await meshJson("/openapi/v1/balance");
        json(response, 200, { availableCredits: result.balance }, origin); return;
      }
      if (request.method === "GET" && url.pathname === "/v1/profiles") {
        json(response, 200, PROFILES, origin); return;
      }
      if (request.method === "POST" && url.pathname === "/v1/image-runs/preview") {
        const body = await readBody();
        if (!validImageRunRequest(body)) {
          throw bridgeError("TASK_REJECTED", "Image generation options are invalid or incompatible with the selected model.");
        }
        const preview = {
          previewId: randomUUID(),
          mode: body.mode,
          prompt: body.prompt.trim(),
          aiModel: body.aiModel,
          generateMultiView: body.generateMultiView,
          ...(body.aspectRatio ? { aspectRatio: body.aspectRatio } : {}),
          ...(body.poseMode ? { poseMode: body.poseMode } : {}),
          ...(body.mode === "IMAGE_TO_IMAGE" ? { referenceImageDataUrls: body.referenceImageDataUrls } : {}),
          maximumCredits: imageRunCredits(body.aiModel, body.mode),
        };
        imagePreviews.set(preview.previewId, preview);
        const { referenceImageDataUrls, ...safePreview } = preview;
        json(response, 200, safePreview, origin); return;
      }
      if (request.method === "POST" && url.pathname === "/v1/image-runs") {
        const body = await readBody();
        if (typeof body.confirmationNonce !== "string" || !body.confirmationNonce.trim()) throw bridgeError("CONFIRMATION_REQUIRED", "Confirm image generation before creating a Meshy task.");
        if (usedNonces.has(body.confirmationNonce)) throw bridgeError("CONFIRMATION_ALREADY_USED", "This confirmation was already used.", 409);
        const preview = imagePreviews.get(body.previewId);
        if (!preview) throw bridgeError("PREVIEW_NOT_FOUND", "The image-generation preview is no longer available.", 404);
        const balance = await meshJson("/openapi/v1/balance");
        if (Number(balance.balance) < preview.maximumCredits) throw bridgeError("INSUFFICIENT_CREDITS", "Available Meshy credits are below this image run's maximum cost.", 402);
        usedNonces.add(body.confirmationNonce);
        const timestamp = new Date().toISOString();
        const run = { id: randomUUID(), ...preview, status: "QUEUED", progress: 0, createdAt: timestamp, updatedAt: timestamp };
        imageRuns.set(run.id, run);
        json(response, 200, safeImageRun(run), origin);
        if (startRuns) void executeImageRun(run);
        return;
      }
      if (request.method === "POST" && url.pathname === "/v1/retexture/preview") {
        const body = await readBody();
        if (!validRetextureRequest(body)) {
          throw bridgeError("TASK_REJECTED", "ReTexture options are invalid or outside the supported local Bridge contract.");
        }
        const preview = {
          previewId: randomUUID(), inputTaskId: body.inputTaskId, textStylePrompt: body.textStylePrompt.trim(), aiModel: body.aiModel,
          enableOriginalUv: body.enableOriginalUv, enablePbr: body.enablePbr,
          textureResolution: body.textureResolution,
          removeLighting: body.removeLighting, alphaThumbnail: body.alphaThumbnail,
          maximumCredits: body.textureResolution === "8k" ? 15 : 10,
        };
        retexturePreviews.set(preview.previewId, preview);
        json(response, 200, preview, origin); return;
      }
      if (request.method === "POST" && url.pathname === "/v1/retexture") {
        const body = await readBody();
        if (typeof body.confirmationNonce !== "string" || !body.confirmationNonce.trim()) throw bridgeError("CONFIRMATION_REQUIRED", "Confirm ReTexture before creating a Meshy task.");
        if (usedNonces.has(body.confirmationNonce)) throw bridgeError("CONFIRMATION_ALREADY_USED", "This confirmation was already used.", 409);
        const preview = retexturePreviews.get(body.previewId);
        if (!preview) throw bridgeError("PREVIEW_NOT_FOUND", "The ReTexture preview is no longer available.", 404);
        const balance = await meshJson("/openapi/v1/balance");
        if (Number(balance.balance) < preview.maximumCredits) throw bridgeError("INSUFFICIENT_CREDITS", "Available Meshy credits are below this ReTexture run's maximum cost.", 402);
        usedNonces.add(body.confirmationNonce);
        const timestamp = new Date().toISOString();
        const run = { id: randomUUID(), ...preview, status: "QUEUED", progress: 0, createdAt: timestamp, updatedAt: timestamp };
        retextureRuns.set(run.id, run);
        json(response, 200, safeRetextureRun(run), origin);
        if (startRuns) void executeRetextureRun(run);
        return;
      }
      if (request.method === "GET" && url.pathname === "/v1/history") {
        const pageNum = Number(url.searchParams.get("page_num") || "1");
        const requestedPageSize = Number(url.searchParams.get("page_size") || `${HISTORY_PAGE_SIZE_MAX}`);
        if (!Number.isInteger(pageNum) || pageNum < 1 || !Number.isInteger(requestedPageSize) || requestedPageSize < 1 || requestedPageSize > HISTORY_PAGE_SIZE_MAX) {
          throw bridgeError("TASK_REJECTED", `History pages require page_num >= 1 and page_size between 1 and ${HISTORY_PAGE_SIZE_MAX}.`);
        }
        const [textTasks, rigTasks, animationTasks] = await Promise.all([
          meshJson(`/openapi/v2/text-to-3d?page_num=${pageNum}&page_size=${requestedPageSize}&sort_by=-created_at`),
          meshJson(`/openapi/v1/rigging?page_num=${pageNum}&page_size=${requestedPageSize}`),
          meshJson(`/openapi/v1/animations?page_num=${pageNum}&page_size=${requestedPageSize}`),
        ]);
        if (![textTasks, rigTasks, animationTasks].every(Array.isArray)) {
          throw bridgeError("BRIDGE_UNAVAILABLE", "Meshy returned an invalid task history response.", 502);
        }
        const items = [
          ...textTasks
            .filter((task) => typeof task?.id === "string" && (task.type === "text-to-3d-preview" || task.type === "text-to-3d-refine"))
            .map((task) => historyItem(task)),
          ...rigTasks
            .filter((task) => typeof task?.id === "string")
            .map((task) => historyItem(task, "RIG")),
          ...animationTasks
            .filter((task) => typeof task?.id === "string")
            .map((task) => historyItem(task, "ANIMATE")),
        ].sort((left, right) => Date.parse(right.createdAt ?? "") - Date.parse(left.createdAt ?? ""));
        json(response, 200, {
          pageNum,
          pageSize: requestedPageSize,
          items,
          hasNext: [textTasks, rigTasks, animationTasks].some(
            (tasks) => tasks.length === requestedPageSize,
          ),
        }, origin);
        return;
      }
      if (request.method === "POST" && url.pathname === "/v1/runs/preview") {
        const body = await readBody();
        const profile = profileById(body.profileId);
        const source = body.source || "TEXT";
        const imageDataUrls = Array.isArray(body.imageDataUrls) ? body.imageDataUrls : [];
        const inputTaskId = body.inputTaskId;
        if (!profile || !["TEXT", "IMAGE", "MULTI_IMAGE"].includes(source)) {
          throw bridgeError("TASK_REJECTED", "Select a supported generation source and output profile.");
        }
        if (source === "TEXT" && (typeof body.prompt !== "string" || !body.prompt.trim() || body.prompt.length > 600)) {
          throw bridgeError("TASK_REJECTED", "Text to 3D requires a 1-600 character prompt.");
        }
        const localImageTask = validInputTaskId(inputTaskId) && Array.from(imageRuns.values()).some((run) => run.taskId === inputTaskId && run.status === "READY");
        if (inputTaskId !== undefined && !localImageTask) {
          throw bridgeError("TASK_REJECTED", "Use a completed 2D task created through this local Bridge as the 3D input.");
        }
        if (source === "IMAGE" && !localImageTask && (imageDataUrls.length !== 1 || !imageDataUrls.every(validImageDataUri))) {
          throw bridgeError("TASK_REJECTED", "Image to 3D requires exactly one PNG or JPEG data URI.");
        }
        if (source === "MULTI_IMAGE" && !localImageTask && (imageDataUrls.length < 1 || imageDataUrls.length > 4 || !imageDataUrls.every(validImageDataUri))) {
          throw bridgeError("TASK_REJECTED", "Multi-image to 3D requires one to four PNG or JPEG data URIs.");
        }
        if (!GEOMETRY_TARGETS.has(body.geometryTarget)) {
          throw bridgeError("TASK_REJECTED", "Select a supported geometry target.");
        }
        if (!validApiOptions(body.apiOptions, source)) {
          throw bridgeError("TASK_REJECTED", "Meshy generation options are invalid or outside the supported range.");
        }
        if (profile.id.startsWith("H1") && !(
          body.h1Preflight?.standardHumanoid === true
          && body.h1Preflight?.clearLimbs === true
          && body.h1Preflight?.noWeapon === true
          && body.h1Preflight?.aOrTPose === true
        )) {
          throw bridgeError("H1_PREFLIGHT_REQUIRED", "Confirm standard humanoid, clear limbs, no weapon, and a verified A/T pose before H1 rigging.");
        }
        const previewId = randomUUID();
          const preview = { previewId, profile, prompt: typeof body.prompt === "string" ? body.prompt.trim() : "", source, imageDataUrls, ...(localImageTask ? { inputTaskId } : {}), geometryTarget: body.geometryTarget, apiOptions: body.apiOptions, maximumCredits: maximumCredits(profile, body.apiOptions, source), stages: profile.stages };
        previews.set(previewId, preview);
        json(response, 200, safePreview(preview), origin); return;
      }
      if (request.method === "POST" && url.pathname === "/v1/runs") {
        const body = await readBody();
        if (typeof body.confirmationNonce !== "string" || !body.confirmationNonce.trim()) throw bridgeError("CONFIRMATION_REQUIRED", "Confirm generation before creating a Meshy task.");
        if (usedNonces.has(body.confirmationNonce)) throw bridgeError("CONFIRMATION_ALREADY_USED", "This confirmation was already used.", 409);
        const preview = previews.get(body.previewId);
        if (!preview) throw bridgeError("PREVIEW_NOT_FOUND", "The generation preview is no longer available.", 404);
        const balance = await meshJson("/openapi/v1/balance");
        if (Number(balance.balance) < preview.maximumCredits) {
          throw bridgeError("INSUFFICIENT_CREDITS", "Available Meshy credits are below this profile's maximum cost.", 402);
        }
        usedNonces.add(body.confirmationNonce);
        const timestamp = new Date().toISOString();
        const run = { id: randomUUID(), profile: preview.profile, prompt: preview.prompt, source: preview.source, imageDataUrls: preview.imageDataUrls, inputTaskId: preview.inputTaskId, geometryTarget: preview.geometryTarget, apiOptions: preview.apiOptions, status: "QUEUED", progress: 0, taskIds: {}, createdAt: timestamp, updatedAt: timestamp };
        runs.set(run.id, run);
        json(response, 200, safeRun(run), origin);
        if (startRuns) void executeRun(run);
        return;
      }

      const historyMatch = url.pathname.match(/^\/v1\/history\/(preview|refine|rig|animate)\/([^/]+)\/(artifact|provenance|thumbnail)(?:\/([a-z0-9-]+))?$/);
      if (historyMatch && request.method === "GET") {
        const stage = historyMatch[1].toUpperCase();
        const taskId = decodeURIComponent(historyMatch[2]);
        const operation = historyMatch[3];
        const artifactKey = historyMatch[4];
        if (operation === "thumbnail") {
          if (stage !== "REFINE") {
            throw bridgeError("ARTIFACT_NOT_READY", "Only Text-to-3D refine history has a thumbnail.", 404);
          }
          const thumbnail = await recoverHistoryThumbnail(taskId);
          response.writeHead(200, { "Content-Type": thumbnail.contentType, "Content-Length": thumbnail.bytes.byteLength, "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff", "Access-Control-Allow-Origin": origin, "Vary": "Origin" });
          response.end(thumbnail.bytes); return;
        }
        const recovered = await recoverArtifact(stage, taskId, artifactKey);
        if (operation === "provenance") {
          json(response, 200, recovered.provenance, origin); return;
        }
        response.writeHead(200, { "Content-Type": "model/gltf-binary", "Content-Length": recovered.artifactBytes.byteLength, "Cache-Control": "no-store", "Access-Control-Allow-Origin": origin, "Vary": "Origin" });
        response.end(recovered.artifactBytes); return;
      }

      const imageRunMatch = url.pathname.match(/^\/v1\/image-runs\/([^/]+)(?:\/(cancel))?$/);
      if (imageRunMatch) {
        const run = imageRuns.get(decodeURIComponent(imageRunMatch[1]));
        if (!run) throw bridgeError("RUN_NOT_FOUND", "The requested image run does not exist.", 404);
        if (request.method === "GET" && !imageRunMatch[2]) { json(response, 200, safeImageRun(run), origin); return; }
        if (request.method === "POST" && imageRunMatch[2] === "cancel") {
          if (!["READY", "FAILED", "CANCELED", "STOPPED_LOCAL"].includes(run.status)) {
            run.status = "STOPPED_LOCAL";
            run.error = {
              code: "STOPPED_LOCAL",
              message: "Local tracking stopped. The Meshy image task may continue remotely and consume credits.",
            };
            run.updatedAt = new Date().toISOString();
          }
          json(response, 200, safeImageRun(run), origin); return;
        }
      }

      const retextureMatch = url.pathname.match(/^\/v1\/retexture\/([^/]+)(?:\/(cancel|artifact|provenance))?$/);
      if (retextureMatch) {
        const run = retextureRuns.get(decodeURIComponent(retextureMatch[1]));
        if (!run) throw bridgeError("RUN_NOT_FOUND", "The requested ReTexture run does not exist.", 404);
        const suffix = retextureMatch[2];
        if (request.method === "GET" && !suffix) { json(response, 200, safeRetextureRun(run), origin); return; }
        if (request.method === "POST" && suffix === "cancel") {
          if (!["READY", "FAILED", "CANCELED", "STOPPED_LOCAL"].includes(run.status)) {
            run.status = "STOPPED_LOCAL";
            run.error = {
              code: "STOPPED_LOCAL",
              message: "Local tracking stopped. The Meshy ReTexture task may continue remotely and consume credits.",
            };
            run.updatedAt = new Date().toISOString();
          }
          json(response, 200, safeRetextureRun(run), origin); return;
        }
        if (request.method === "GET" && suffix === "provenance") {
          if (!run.provenance) throw bridgeError("ARTIFACT_NOT_READY", "The verified ReTexture GLB is not ready.", 409);
          json(response, 200, run.provenance, origin); return;
        }
        if (request.method === "GET" && suffix === "artifact") {
          if (!run.artifactBytes || !run.provenance) throw bridgeError("ARTIFACT_NOT_READY", "The verified ReTexture GLB is not ready.", 409);
          response.writeHead(200, { "Content-Type": "model/gltf-binary", "Content-Length": run.artifactBytes.byteLength, "Cache-Control": "no-store", "Access-Control-Allow-Origin": origin, "Vary": "Origin" });
          response.end(run.artifactBytes); return;
        }
      }

      const match = url.pathname.match(/^\/v1\/runs\/([^/]+)(?:\/(cancel|artifact|provenance)(?:\/([A-Za-z0-9-]+))?)?$/);
      if (match) {
        const run = runs.get(decodeURIComponent(match[1]));
        if (!run) throw bridgeError("RUN_NOT_FOUND", "The requested Meshy run does not exist.", 404);
        const suffix = match[2];
        if (request.method === "GET" && !suffix) { json(response, 200, safeRun(run), origin); return; }
        if (request.method === "POST" && suffix === "cancel") {
          if (!["READY", "PARTIAL", "FAILED", "CANCELED", "STOPPED_LOCAL"].includes(run.status)) {
            run.status = "STOPPED_LOCAL";
            run.error = {
              code: "STOPPED_LOCAL",
              message: "Local tracking stopped. Already-created Meshy tasks may continue remotely and can be recovered from history.",
            };
            run.updatedAt = new Date().toISOString();
          }
          json(response, 200, safeRun(run), origin); return;
        }
        if (request.method === "GET" && suffix === "provenance") {
          if (!run.provenance) throw bridgeError("ARTIFACT_NOT_READY", "The verified GLB is not ready to import.", 409);
          json(response, 200, run.provenance, origin); return;
        }
        if (request.method === "GET" && suffix === "artifact") {
          const selector = match[3];
          const requestedActionId = selector !== undefined && /^[0-9]+$/.test(selector)
            ? Number(selector)
            : undefined;
          const selectedArtifact = selector === undefined
            ? run.artifactBytes
            : requestedActionId === undefined
              ? run.artifacts?.find((artifact) => artifact.key === selector)?.bytes
              : run.artifacts?.find((artifact) => artifact.actionId === requestedActionId)?.bytes;
          if (!selectedArtifact || !run.provenance) throw bridgeError("ARTIFACT_NOT_READY", "The requested verified GLB is not ready to import.", 409);
          response.writeHead(200, { "Content-Type": "model/gltf-binary", "Content-Length": selectedArtifact.byteLength, "Cache-Control": "no-store", "Access-Control-Allow-Origin": origin, "Vary": "Origin" });
          response.end(selectedArtifact); return;
        }
      }
      throw bridgeError("BRIDGE_UNAVAILABLE", "Bridge route not found.", 404);
    } catch (error) {
      const status = error.status || 500;
      const code = error.code || "BRIDGE_UNAVAILABLE";
      const message = status >= 500 ? "The local Meshy Bridge could not complete the request." : error.message;
      json(response, status, { code, message }, origin === allowedOrigin ? origin : allowedOrigin);
    }
  });

  return {
    pairingCode,
    async listen(port = 43119) {
      await new Promise((resolve, reject) => server.once("error", reject).listen(port, bindHost, resolve));
      const address = server.address();
      const urlHost = bindHost.includes(":") ? `[${bindHost}]` : bindHost;
      return `http://${urlHost}:${address.port}`;
    },
    async close() { await new Promise((resolve, reject) => server.close((error) => error ? reject(error) : resolve())); },
  };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const bridge = createLocalBridge({
    apiKey: process.env.MESHY_API_KEY,
    pairingCode: process.env.MESHY_BRIDGE_PAIRING_CODE,
    allowedOrigin: process.env.MESHY_BRIDGE_ALLOWED_ORIGIN,
    bindHost: process.env.MESHY_BRIDGE_BIND_HOST,
    allowContainerBind: process.env.MESHY_BRIDGE_ALLOW_CONTAINER_BIND === "1",
    requestRestart: process.env.MESHY_BRIDGE_RESTARTABLE === "1" ? () => process.exit(0) : undefined,
    allowAutomaticPairing: process.env.MESHY_BRIDGE_AUTOMATIC_PAIRING === "1",
  });
  const origin = await bridge.listen(Number(process.env.MESHY_BRIDGE_PORT || 43119));
  process.stdout.write(`Meshy Local Bridge listening at ${origin}\nPairing code: ${bridge.pairingCode}\n`);
}
