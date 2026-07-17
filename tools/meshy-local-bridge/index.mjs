import { createHash, randomBytes, randomUUID } from "node:crypto";
import { createServer } from "node:http";
import { fileURLToPath } from "node:url";

const PROTOCOL_VERSION = 1;
const API_ORIGIN = "https://api.meshy.ai";
const SESSION_TTL_MS = 15 * 60_000;
const MAX_BODY_BYTES = 64 * 1024;
const POLL_INTERVAL_MS = 3_000;
const MAX_POLLS = 180;
const GEOMETRY_TARGETS = new Set(["AURORA_PROOF", "LOWER_DETAIL", "BALANCED", "HIGHER_DETAIL"]);

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

function maximumCredits(profile) {
  return profile.id.startsWith("H1") ? 38 : 30;
}

function targetPolycount(target) {
  return target === "AURORA_PROOF" ? 1_500 : target === "LOWER_DETAIL" ? 10_000 : target === "HIGHER_DETAIL" ? 60_000 : 30_000;
}

function safeRun(run) {
  const { artifactBytes, ...safe } = run;
  return safe;
}

/**
 * Owner-operated loopback proxy. This module purposefully exposes only the
 * Meshy Lab contract; it is never a generic HTTP proxy.
 */
export function createLocalBridge({
  apiKey,
  pairingCode = randomBytes(18).toString("base64url"),
  allowedOrigin,
  meshFetch = fetch,
  startRuns = true,
} = {}) {
  if (!apiKey) throw new Error("MESHY_API_KEY is required by the local Bridge process.");
  if (!allowedOrigin) throw new Error("MESHY_BRIDGE_ALLOWED_ORIGIN is required and must be an exact Studio origin.");

  const sessions = new Map();
  const previews = new Map();
  const runs = new Map();
  const usedNonces = new Set();
  let pairingCodeUsed = false;

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

  const executeRun = async (run) => {
    try {
      run.status = "PREVIEWING";
      const preview = await meshJson("/openapi/v2/text-to-3d", {
        method: "POST",
        body: JSON.stringify({
          mode: "preview", prompt: run.prompt, ai_model: "latest", moderation: true,
          should_remesh: true, target_polycount: targetPolycount(run.geometryTarget),
          pose_mode: run.profile.id.startsWith("H1") ? "a-pose" : "", target_formats: ["glb"],
        }),
      });
      run.taskIds.PREVIEW = preview.result;
      await waitForTask(run, "PREVIEW", "/openapi/v2/text-to-3d", preview.result);

      run.status = "REFINING";
      const refined = await meshJson("/openapi/v2/text-to-3d", {
        method: "POST",
        body: JSON.stringify({
          mode: "refine", preview_task_id: preview.result, ai_model: "latest", enable_pbr: true,
          remove_lighting: true, target_formats: ["glb"],
        }),
      });
      run.taskIds.REFINE = refined.result;
      let output = await waitForTask(run, "REFINE", "/openapi/v2/text-to-3d", refined.result);

      if (run.profile.id.startsWith("H1")) {
        run.status = "RIGGING";
        const rig = await meshJson("/openapi/v1/rigging", {
          method: "POST", body: JSON.stringify({ input_task_id: refined.result, height_meters: 1.7 }),
        });
        run.taskIds.RIG = rig.result;
        await waitForTask(run, "RIG", "/openapi/v1/rigging", rig.result);

        run.status = "ANIMATING";
        const animation = await meshJson("/openapi/v1/animations", {
          method: "POST", body: JSON.stringify({ rig_task_id: rig.result, action_id: 0 }),
        });
        run.taskIds.ANIMATE = animation.result;
        output = await waitForTask(run, "ANIMATE", "/openapi/v1/animations", animation.result);
      }

      const assetUrl = output.result?.animation_glb_url ?? output.model_urls?.glb;
      if (typeof assetUrl !== "string") throw bridgeError("ARTIFACT_INVALID", "Meshy did not return a GLB artifact for this run.");
      run.status = "VERIFYING";
      const assetResponse = await meshFetch(assetUrl);
      if (!assetResponse.ok) throw bridgeError("ARTIFACT_INVALID", "The signed Meshy GLB download failed.");
      const artifactBytes = new Uint8Array(await assetResponse.arrayBuffer());
      if (artifactBytes.byteLength < 4 || artifactBytes.byteLength > 512 * 1024 * 1024) {
        throw bridgeError("ARTIFACT_INVALID", "The GLB violates the local Bridge size gate.");
      }
      run.artifactBytes = artifactBytes;
      run.provenance = {
        profileId: run.profile.id,
        bridgeProtocolVersion: PROTOCOL_VERSION,
        sha256: createHash("sha256").update(artifactBytes).digest("hex"),
        byteLength: artifactBytes.byteLength,
        taskIds: run.taskIds,
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
        json(response, 200, { protocolVersion: PROTOCOL_VERSION, bridge: "LOCAL", status: "READY" }, origin); return;
      }
      if (request.method === "POST" && url.pathname === "/v1/pair") {
        const body = await readBody();
        if (body.pairingCode !== pairingCode) throw bridgeError("PAIRING_REQUIRED", "The local Bridge pairing code is invalid.", 401);
        if (pairingCodeUsed) throw bridgeError("PAIRING_REQUIRED", "The local Bridge pairing code has already been used.", 401);
        pairingCodeUsed = true;
        const sessionToken = randomBytes(32).toString("base64url");
        const expiresAt = Date.now() + SESSION_TTL_MS;
        sessions.set(sessionToken, expiresAt);
        json(response, 200, { sessionToken, expiresAt: new Date(expiresAt).toISOString() }, origin); return;
      }

      requireSession(request);
      if (request.method === "GET" && url.pathname === "/v1/balance") {
        const result = await meshJson("/openapi/v1/balance");
        json(response, 200, { availableCredits: result.balance }, origin); return;
      }
      if (request.method === "GET" && url.pathname === "/v1/profiles") {
        json(response, 200, PROFILES, origin); return;
      }
      if (request.method === "POST" && url.pathname === "/v1/runs/preview") {
        const body = await readBody();
        const profile = profileById(body.profileId);
        if (!profile || typeof body.prompt !== "string" || !body.prompt.trim() || body.prompt.length > 600) {
          throw bridgeError("TASK_REJECTED", "Select a supported profile and a 1-600 character prompt.");
        }
        if (!GEOMETRY_TARGETS.has(body.geometryTarget)) {
          throw bridgeError("TASK_REJECTED", "Select a supported geometry target.");
        }
        if (profile.id.startsWith("H1") && !(body.h1Preflight?.standardHumanoid === true && body.h1Preflight?.clearLimbs === true && body.h1Preflight?.noWeapon === true)) {
          throw bridgeError("H1_PREFLIGHT_REQUIRED", "Confirm standard humanoid, clear limbs, and no weapon before H1 rigging.");
        }
        const previewId = randomUUID();
        const preview = { previewId, profile, prompt: body.prompt.trim(), geometryTarget: body.geometryTarget, maximumCredits: maximumCredits(profile), stages: profile.stages };
        previews.set(previewId, preview);
        json(response, 200, preview, origin); return;
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
        const run = { id: randomUUID(), profile: preview.profile, prompt: preview.prompt, geometryTarget: preview.geometryTarget, status: "QUEUED", progress: 0, taskIds: {}, createdAt: timestamp, updatedAt: timestamp };
        runs.set(run.id, run);
        json(response, 200, safeRun(run), origin);
        if (startRuns) void executeRun(run);
        return;
      }

      const match = url.pathname.match(/^\/v1\/runs\/([^/]+)(?:\/(cancel|artifact|provenance))?$/);
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
          if (!run.artifactBytes || !run.provenance) throw bridgeError("ARTIFACT_NOT_READY", "The verified GLB is not ready to import.", 409);
          response.writeHead(200, { "Content-Type": "model/gltf-binary", "Content-Length": run.artifactBytes.byteLength, "Cache-Control": "no-store", "Access-Control-Allow-Origin": origin, "Vary": "Origin" });
          response.end(run.artifactBytes); return;
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
      await new Promise((resolve, reject) => server.once("error", reject).listen(port, "127.0.0.1", resolve));
      const address = server.address();
      return `http://127.0.0.1:${address.port}`;
    },
    async close() { await new Promise((resolve, reject) => server.close((error) => error ? reject(error) : resolve())); },
  };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const bridge = createLocalBridge({
    apiKey: process.env.MESHY_API_KEY,
    pairingCode: process.env.MESHY_BRIDGE_PAIRING_CODE,
    allowedOrigin: process.env.MESHY_BRIDGE_ALLOWED_ORIGIN,
  });
  const origin = await bridge.listen(Number(process.env.MESHY_BRIDGE_PORT || 43119));
  process.stdout.write(`Meshy Local Bridge listening at ${origin}\nPairing code: ${bridge.pairingCode}\n`);
}
