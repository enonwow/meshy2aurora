import { createHash, randomBytes, randomUUID } from "node:crypto";
import { createServer } from "node:http";
import { fileURLToPath } from "node:url";
import { inspectGlbTriangleCount, mergeMeshyAnimationGlbs } from "./merge-animation-glbs.mjs";
import {
  AURORA_MODEL_TRIANGLE_BUDGET_V1,
  MESHY_RIG_FACE_LIMIT,
  planAutomaticRigFaceRecovery,
} from "./remesh-rig-animation-recovery-options.mjs";

const PROTOCOL_VERSION = 1;
const API_ORIGIN = "https://api.meshy.ai";
const SESSION_TTL_MS = 15 * 60_000;
const MAX_BODY_BYTES = 100 * 1024 * 1024;
const MAX_IMAGE_DATA_URL_CHARS = 27 * 1024 * 1024;
const MAX_THUMBNAIL_BYTES = 8 * 1024 * 1024;
const HISTORY_PAGE_SIZE_MAX = 50;
const POLL_INTERVAL_MS = 3_000;
const MAX_POLLS = 180;
const GEOMETRY_TARGETS = new Set(["AURORA_PROOF", "LOWER_DETAIL", "BALANCED", "HIGHER_DETAIL"]);
const IMAGE_MODELS = new Set(["nano-banana", "nano-banana-2", "nano-banana-pro", "gpt-image-2"]);
const IMAGE_RATIOS = new Set(["1:1", "16:9", "9:16", "4:3", "3:4"]);
const GPT_IMAGE_RATIOS = new Set(["1:1", "3:2", "2:3"]);
const RETEXTURE_MAXIMUM_CREDITS = 10;
const NWN_DIRECT_CREATURE_CLIPS = new Set([
  "ca1slashl", "ca1slashr", "ca1stab", "creach", "cconjure1", "ccastout",
  "cparryl", "cparryr", "cdodgelr", "cdodges", "creadyr", "creadyl",
  "cdamagel", "cdamager", "cdamages", "ckdbck", "ckdbckps", "ckdbckdie",
  "cguptokdb", "cgustandb", "cwalk", "crun", "ccwalkf", "ccwalkb",
  "ccwalkl", "ccwalkr", "cpause1", "chturnl", "chturnr", "ctaunt",
  "cclosel", "ccloseh", "cgetmid", "ckdbckdmg", "ccastoutlp", "cspasm",
  "cappear", "cdisappear", "cgetmidlp", "cdead", "cdisappearlp", "ccturnr",
]);

const PROFILES = [
  { id: "H1-humanoid-animated/v1", label: "Humanoid Animated", description: "Textured standard humanoid with rigging and one to ten explicitly mapped Meshy animations.", stages: ["PREVIEW", "REFINE", "RIG", "ANIMATE"], expectedOutput: { texture: true, rigging: true, animation: "MAPPED_SET" } },
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

function animationActions(options) {
  if (Array.isArray(options?.animationActions)) return options.animationActions;
  if (Number.isInteger(options?.animationActionId)) {
    return [{ actionId: options.animationActionId, clipName: "cpause1" }];
  }
  if (Array.isArray(options?.animationActionIds) && options.animationActionIds.length === 1) {
    return [{ actionId: options.animationActionIds[0], clipName: "cpause1" }];
  }
  return [{ actionId: 0, clipName: "cpause1" }];
}

function maximumCredits(profile, options) {
  return profile.id.startsWith("H1") ? 40 + 3 * animationActions(options).length : 30;
}

function targetPolycount(target) {
  return target === "AURORA_PROOF" ? 1_500 : target === "LOWER_DETAIL" ? 10_000 : target === "HIGHER_DETAIL" ? 60_000 : 30_000;
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
    && ["enableOriginalUv", "enablePbr", "hdTexture", "removeLighting", "alphaThumbnail"].every((key) => typeof body[key] === "boolean");
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
  if (!["alphaThumbnail", "autoSize", "enablePbr", "shouldTexture", "hdTexture", "removeLighting", "imageEnhancement", "multiViewThumbnails", "rigHumanoid"].every((key) => typeof options[key] === "boolean")) return false;
  if (!["bottom", "center"].includes(options.originAt) || typeof options.texturePrompt !== "string" || options.texturePrompt.length > 600 || typeof options.textureImageUrl !== "string") return false;
  if (!Number.isFinite(options.rigHeightMeters) || options.rigHeightMeters < 0.5 || options.rigHeightMeters > 3) return false;
  if (options.rigHumanoid) {
    const actions = animationActions(options);
    if (actions.length < 1 || actions.length > 10) return false;
    if (actions.some(({ actionId, clipName }) => !Number.isInteger(actionId) || actionId < 0 || !NWN_DIRECT_CREATURE_CLIPS.has(clipName))) return false;
    if (new Set(actions.map(({ actionId }) => actionId)).size !== actions.length) return false;
    if (new Set(actions.map(({ clipName }) => clipName.toLowerCase())).size !== actions.length) return false;
    if (actions.filter(({ clipName }) => clipName === "cpause1").length !== 1) return false;
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
}

function historyItem(task) {
  const stage = task.type === "text-to-3d-refine" ? "REFINE" : "PREVIEW";
  return {
    taskId: task.id,
    stage,
    status: task.status,
    prompt: typeof task.prompt === "string" ? task.prompt : "",
    createdAt: Number.isFinite(task.created_at) ? new Date(task.created_at).toISOString() : undefined,
    finishedAt: Number.isFinite(task.finished_at) ? new Date(task.finished_at).toISOString() : undefined,
    consumedCredits: Number.isFinite(task.consumed_credits) ? task.consumed_credits : undefined,
    glbAvailable: typeof task.model_urls?.glb === "string",
    thumbnailAvailable: task.type === "text-to-3d-refine" && task.status === "SUCCEEDED" && typeof historyThumbnailUrl(task) === "string",
  };
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
    const response = await meshFetch(`${API_ORIGIN}${path}`, {
      ...options,
      headers: {
        Authorization: `Bearer ${apiKey}`,
        ...(options.body ? { "Content-Type": "application/json" } : {}),
      },
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) {
      const code = response.status === 402 ? "INSUFFICIENT_CREDITS" : "TASK_REJECTED";
      throw bridgeError(code, payload.message || `Meshy request failed with status ${response.status}.`, response.status);
    }
    return payload;
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

  const waitForTask = async (run, stage, endpoint, taskId) => {
    for (let poll = 0; poll < MAX_POLLS; poll += 1) {
      if (run.status === "CANCELED") throw bridgeError("CANCELED", "The local run was canceled before the next Meshy stage.");
      const task = await meshJson(`${endpoint}/${encodeURIComponent(taskId)}`);
      run.progress = Math.min(99, Math.max(run.progress, Number(task.progress) || 0));
      run.updatedAt = new Date().toISOString();
      if (task.status === "SUCCEEDED") return task;
      if (task.status === "FAILED") throw bridgeError("TASK_FAILED", task.task_error?.message || `Meshy ${stage.toLowerCase()} task failed.`);
      await sleep(POLL_INTERVAL_MS);
    }
    throw bridgeError("TASK_FAILED", `Meshy ${stage.toLowerCase()} task timed out in the local Bridge.`);
  };

  const recoverArtifact = async (taskId) => {
    const cached = recoveredArtifacts.get(taskId);
    if (cached) return cached;
    const task = await meshJson(`/openapi/v2/text-to-3d/${encodeURIComponent(taskId)}`);
    if (task.type !== "text-to-3d-refine" || task.status !== "SUCCEEDED") {
      throw bridgeError("ARTIFACT_NOT_READY", "Only a completed Text-to-3D refine task can be recovered.", 409);
    }
    const assetUrl = task.model_urls?.glb;
    if (typeof assetUrl !== "string") throw bridgeError("ARTIFACT_NOT_READY", "Meshy did not expose a GLB for this completed task.", 409);
    const assetResponse = await meshFetch(assetUrl);
    if (!assetResponse.ok) throw bridgeError("ARTIFACT_INVALID", "The signed Meshy GLB download failed.");
    const artifactBytes = new Uint8Array(await assetResponse.arrayBuffer());
    verifyGlbBytes(artifactBytes);
    const recovered = {
      artifactBytes,
      provenance: {
        profileId: "RECOVERED-text-to-3d/v1",
        bridgeProtocolVersion: PROTOCOL_VERSION,
        sha256: createHash("sha256").update(artifactBytes).digest("hex"),
        byteLength: artifactBytes.byteLength,
        taskIds: { REFINE: taskId },
      },
    };
    recoveredArtifacts.set(taskId, recovered);
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
          hd_texture: options?.hdTexture ?? false,
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
            hd_texture: options?.hdTexture ?? false,
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
        inputTaskId = refined.result;
        output = await waitForTask(run, "REFINE", "/openapi/v2/text-to-3d", refined.result);
      }

      const shouldRig = options?.rigHumanoid ?? run.profile.id.startsWith("H1");
      if (shouldRig) {
        const generatedAssetUrl = output.model_urls?.glb;
        if (typeof generatedAssetUrl !== "string") {
          throw bridgeError(
            "ARTIFACT_INVALID",
            "Meshy did not expose the generated GLB required for the pre-rig face-budget gate.",
          );
        }
        const generatedBytes = await downloadVerifiedGlb(
          generatedAssetUrl,
          "The signed pre-rig Meshy GLB download failed.",
        );
        const requestedTargetPolycount = options?.targetPolycount ?? targetPolycount(run.geometryTarget);
        const recovery = planAutomaticRigFaceRecovery({
          observedFaceCount: inspectGlbTriangleCount(generatedBytes, "generated pre-rig GLB"),
          requestedTargetPolycount,
        });
        run.geometryAdmission = {
          generatedFaceCount: recovery.observedFaceCount,
          rigFaceLimit: MESHY_RIG_FACE_LIMIT,
          automaticRemeshApplied: recovery.required,
        };
        if (recovery.required) {
          run.status = "REMESHING";
          const remesh = await meshJson("/openapi/v1/remesh", {
            method: "POST",
            body: JSON.stringify({
              input_task_id: inputTaskId,
              target_formats: ["glb"],
              topology: "triangle",
              target_polycount: recovery.targetPolycount,
              alpha_thumbnail: false,
            }),
          });
          run.taskIds.REMESH = remesh.result;
          inputTaskId = remesh.result;
          output = await waitForTask(run, "REMESH", "/openapi/v1/remesh", remesh.result);
          const remeshedAssetUrl = output.model_urls?.glb;
          if (typeof remeshedAssetUrl !== "string") {
            throw bridgeError("ARTIFACT_INVALID", "Meshy did not expose the automatically remeshed GLB.");
          }
          const remeshedBytes = await downloadVerifiedGlb(
            remeshedAssetUrl,
            "The signed automatically remeshed Meshy GLB download failed.",
          );
          const remeshedFaceCount = inspectGlbTriangleCount(remeshedBytes, "automatically remeshed GLB");
          run.geometryAdmission = {
            ...run.geometryAdmission,
            remeshTaskId: remesh.result,
            remeshTargetPolycount: recovery.targetPolycount,
            remeshedFaceCount,
          };
          if (remeshedFaceCount > MESHY_RIG_FACE_LIMIT) {
            throw bridgeError(
              "ARTIFACT_INVALID",
              `Automatic remesh still has ${remeshedFaceCount} faces; Meshy rigging requires at most ${MESHY_RIG_FACE_LIMIT}.`,
            );
          }
        }
        run.status = "RIGGING";
        const rig = await meshJson("/openapi/v1/rigging", {
          method: "POST", body: JSON.stringify({ input_task_id: inputTaskId, height_meters: options?.rigHeightMeters ?? 1.7 }),
        });
        run.taskIds.RIG = rig.result;
        output = await waitForTask(run, "RIG", "/openapi/v1/rigging", rig.result);

        const selectedActions = animationActions(options);
        run.artifacts = [];
        for (const { actionId: animationActionId, clipName } of selectedActions) {
          run.status = "ANIMATING";
          const animation = await meshJson("/openapi/v1/animations", {
            method: "POST", body: JSON.stringify({ rig_task_id: rig.result, action_id: animationActionId }),
          });
          const taskKey = selectedActions.length === 1 ? "ANIMATE" : `ANIMATE_${animationActionId}`;
          run.taskIds[taskKey] = animation.result;
          output = await waitForTask(run, "ANIMATE", "/openapi/v1/animations", animation.result);
          const animationAssetUrl = output.result?.animation_glb_url;
          if (typeof animationAssetUrl !== "string") throw bridgeError("ARTIFACT_INVALID", `Meshy did not return the GLB for animation action ${animationActionId}.`);
          const animationBytes = await downloadVerifiedGlb(
            animationAssetUrl,
            `The signed Meshy animation GLB download failed for action ${animationActionId}.`,
          );
          run.artifacts.push({
            actionId: animationActionId,
            clipName,
            bytes: animationBytes,
            sha256: createHash("sha256").update(animationBytes).digest("hex"),
            byteLength: animationBytes.byteLength,
          });
        }
      }

      run.status = "VERIFYING";
      let artifactBytes;
      if (run.artifacts?.length) {
        artifactBytes = mergeMeshyAnimationGlbs(run.artifacts.map(({ actionId, clipName, bytes }) => ({
          clipName,
          label: `Meshy action ${actionId}`,
          bytes,
        })));
      } else {
        const assetUrl = output.model_urls?.glb;
        if (typeof assetUrl !== "string") throw bridgeError("ARTIFACT_INVALID", "Meshy did not return a GLB artifact for this run.");
        artifactBytes = await downloadVerifiedGlb(assetUrl, "The signed Meshy GLB download failed.");
      }
      run.artifactBytes = artifactBytes;
      run.provenance = {
        profileId: run.profile.id,
        bridgeProtocolVersion: PROTOCOL_VERSION,
        sha256: createHash("sha256").update(artifactBytes).digest("hex"),
        byteLength: artifactBytes.byteLength,
        taskIds: run.taskIds,
        artifactKind: run.artifacts?.length ? "MERGED_ANIMATION_GLTF" : "GENERATED_GLTF",
        ...(run.artifacts?.length ? {
          animationArtifacts: run.artifacts.map(({ actionId, clipName, sha256, byteLength }) => ({
            actionId, clipName, sha256, byteLength,
          })),
        } : {}),
        ...(run.geometryAdmission ? { geometryAdmission: run.geometryAdmission } : {}),
      };
      run.status = "READY";
      run.progress = 100;
      run.updatedAt = new Date().toISOString();
    } catch (error) {
      if (run.status !== "CANCELED") {
        run.status = "FAILED";
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
      const output = await waitForTask(run, "IMAGE", endpoint, created.result);
      if (!Array.isArray(output.image_urls) || !output.image_urls.length || !output.image_urls.every((value) => typeof value === "string")) {
        throw bridgeError("ARTIFACT_INVALID", "Meshy did not return image output for this run.");
      }
      run.imageUrls = output.image_urls;
      run.status = "READY";
      run.progress = 100;
    } catch (error) {
      if (run.status !== "CANCELED") {
        run.status = "FAILED";
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
          hd_texture: run.hdTexture,
          remove_lighting: run.removeLighting,
          target_formats: ["glb"],
          alpha_thumbnail: run.alphaThumbnail,
        }),
      });
      run.taskId = created.result;
      const output = await waitForTask(run, "RETEXTURE", "/openapi/v1/retexture", created.result);
      const assetUrl = output.model_urls?.glb;
      if (typeof assetUrl !== "string") throw bridgeError("ARTIFACT_INVALID", "Meshy did not return a GLB artifact for ReTexture.");
      run.status = "VERIFYING";
      const assetResponse = await meshFetch(assetUrl);
      if (!assetResponse.ok) throw bridgeError("ARTIFACT_INVALID", "The signed Meshy ReTexture GLB download failed.");
      const artifactBytes = new Uint8Array(await assetResponse.arrayBuffer());
      verifyGlbBytes(artifactBytes);
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
      if (run.status !== "CANCELED") {
        run.status = "FAILED";
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
          enableOriginalUv: body.enableOriginalUv, enablePbr: body.enablePbr, hdTexture: body.hdTexture,
          removeLighting: body.removeLighting, alphaThumbnail: body.alphaThumbnail, maximumCredits: RETEXTURE_MAXIMUM_CREDITS,
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
        const tasks = await meshJson(`/openapi/v2/text-to-3d?page_num=${pageNum}&page_size=${requestedPageSize}&sort_by=-created_at`);
        if (!Array.isArray(tasks)) throw bridgeError("BRIDGE_UNAVAILABLE", "Meshy returned an invalid Text-to-3D history response.", 502);
        json(response, 200, {
          pageNum,
          pageSize: requestedPageSize,
          items: tasks.filter((task) => typeof task?.id === "string" && (task.type === "text-to-3d-preview" || task.type === "text-to-3d-refine")).map(historyItem),
          hasNext: tasks.length === requestedPageSize,
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
        if (profile.id.startsWith("H1") && !(body.h1Preflight?.standardHumanoid === true && body.h1Preflight?.clearLimbs === true && body.h1Preflight?.noWeapon === true)) {
          throw bridgeError("H1_PREFLIGHT_REQUIRED", "Confirm standard humanoid, clear limbs, and no weapon before H1 rigging.");
        }
        const previewId = randomUUID();
          const preview = { previewId, profile, prompt: typeof body.prompt === "string" ? body.prompt.trim() : "", source, imageDataUrls, ...(localImageTask ? { inputTaskId } : {}), geometryTarget: body.geometryTarget, apiOptions: body.apiOptions, maximumCredits: maximumCredits(profile, body.apiOptions), stages: profile.stages };
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

      const historyMatch = url.pathname.match(/^\/v1\/history\/([^/]+)\/(artifact|provenance|thumbnail)$/);
      if (historyMatch && request.method === "GET") {
        const taskId = decodeURIComponent(historyMatch[1]);
        if (historyMatch[2] === "thumbnail") {
          const thumbnail = await recoverHistoryThumbnail(taskId);
          response.writeHead(200, { "Content-Type": thumbnail.contentType, "Content-Length": thumbnail.bytes.byteLength, "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff", "Access-Control-Allow-Origin": origin, "Vary": "Origin" });
          response.end(thumbnail.bytes); return;
        }
        const recovered = await recoverArtifact(taskId);
        if (historyMatch[2] === "provenance") {
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
          if (run.status !== "READY" && run.status !== "FAILED") { run.status = "CANCELED"; run.updatedAt = new Date().toISOString(); }
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
          if (run.status !== "READY" && run.status !== "FAILED") { run.status = "CANCELED"; run.updatedAt = new Date().toISOString(); }
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

      const match = url.pathname.match(/^\/v1\/runs\/([^/]+)(?:\/(cancel|artifact|provenance)(?:\/([0-9]+))?)?$/);
      if (match) {
        const run = runs.get(decodeURIComponent(match[1]));
        if (!run) throw bridgeError("RUN_NOT_FOUND", "The requested Meshy run does not exist.", 404);
        const suffix = match[2];
        if (request.method === "GET" && !suffix) { json(response, 200, safeRun(run), origin); return; }
        if (request.method === "POST" && suffix === "cancel") {
          if (run.status !== "READY" && run.status !== "FAILED") { run.status = "CANCELED"; run.updatedAt = new Date().toISOString(); }
          json(response, 200, safeRun(run), origin); return;
        }
        if (request.method === "GET" && suffix === "provenance") {
          if (!run.provenance) throw bridgeError("ARTIFACT_NOT_READY", "The verified GLB is not ready to import.", 409);
          json(response, 200, run.provenance, origin); return;
        }
        if (request.method === "GET" && suffix === "artifact") {
          const requestedActionId = match[3] === undefined ? undefined : Number(match[3]);
          const selectedArtifact = requestedActionId === undefined
            ? run.artifactBytes
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
