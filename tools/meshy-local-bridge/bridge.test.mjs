import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { after, before, test } from "node:test";
import { createLocalBridge } from "./index.mjs";
import { parseModerationFlag } from "./real-image-multi-animation-options.mjs";
import {
  calculateRemeshRecoveryCreditGate,
  parseRemeshRecoveryOptions,
} from "./remesh-rig-animation-recovery-options.mjs";
import { createTestGlb } from "./test-glb-fixture.mjs";

let bridge;
let origin;
const meshCalls = [];
const H1_PREFLIGHT = {
  standardHumanoid: true,
  clearLimbs: true,
  noWeapon: true,
  aOrTPose: true,
};

function fakeGlb(marker = 0) {
  return createTestGlb({ marker });
}

function h1ApiOptions(animationActionIds = [0]) {
  return {
    modelType: "standard",
    aiModel: "meshy-6",
    shouldRemesh: true,
    topology: "triangle",
    targetPolycount: 300_000,
    poseMode: "a-pose",
    moderation: true,
    targetFormats: ["glb"],
    alphaThumbnail: false,
    autoSize: true,
    originAt: "bottom",
    enablePbr: true,
    shouldTexture: true,
    textureResolution: "2k",
    texturePrompt: "",
    textureImageUrl: "",
    removeLighting: true,
    imageEnhancement: true,
    multiViewThumbnails: false,
    rigHumanoid: true,
    rigHeightMeters: 1.85,
    animationActionIds,
    animationCatalogSnapshotId: "meshy-animation-library-2026-07-28-curated-v1",
    animationCatalogSha256: "661b837e858235d0c375726e123d6963298144d5e40e7c81379ad26d0fba5f4e",
  };
}

before(async () => {
  bridge = createLocalBridge({
    apiKey: "test-key-that-must-not-leak",
    pairingCode: "pair-local-proof",
    allowedOrigin: "http://localhost:5173",
    meshFetch: async (url, options) => {
      meshCalls.push({ url, options });
      return new Response(JSON.stringify({ balance: 120 }), { headers: { "Content-Type": "application/json" } });
    },
  });
  origin = await bridge.listen(0);
});

after(async () => bridge.close());

async function call(path, options = {}) {
  return fetch(`${origin}${path}`, {
    ...options,
    headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) },
  });
}

test("exposes only a paired loopback contract and never returns the API key", async () => {
  const health = await call("/v1/health");
  assert.equal(health.status, 200);
  assert.deepEqual(await health.json(), { protocolVersion: 1, bridge: "LOCAL", status: "READY", restartSupported: false, automaticPairingSupported: false });

  const pair = await call("/v1/pair", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ pairingCode: "pair-local-proof" }),
  });
  const pairing = await pair.json();
  assert.equal(pair.status, 200);
  assert.equal(JSON.stringify(pairing).includes("test-key-that-must-not-leak"), false);

  const balance = await call("/v1/balance", { headers: { "X-Meshy-Session": pairing.sessionToken } });
  assert.deepEqual(await balance.json(), { availableCredits: 120 });
  assert.equal(meshCalls.at(-1).url, "https://api.meshy.ai/openapi/v1/balance");
  assert.equal(meshCalls.at(-1).options.headers.Authorization, "Bearer test-key-that-must-not-leak");

  const profiles = await call("/v1/profiles", { headers: { "X-Meshy-Session": pairing.sessionToken } });
  assert.deepEqual((await profiles.json()).map((profile) => profile.id), [
    "H1-humanoid-animated/v1", "N1-quadruped/v1", "S1-static-prop/v1",
  ]);
  const secondPair = await call("/v1/pair", {
    method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "pair-local-proof" }),
  });
  assert.equal(secondPair.status, 401);
});

test("accepts a same-origin Bridge restart only when a local supervisor is configured", async () => {
  let restartRequests = 0;
  const local = createLocalBridge({
    apiKey: "restart-test-key", pairingCode: "restart-test-pair", allowedOrigin: "http://localhost:5173", startRuns: false,
    requestRestart: () => { restartRequests += 1; },
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
    ...options,
    headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) },
  });
  try {
    const health = await request("/v1/health");
    assert.deepEqual(await health.json(), { protocolVersion: 1, bridge: "LOCAL", status: "READY", restartSupported: true, automaticPairingSupported: false });
    const restart = await request("/v1/bridge/restart", { method: "POST" });
    assert.equal(restart.status, 202);
    assert.deepEqual(await restart.json(), { status: "RESTARTING" });
    await new Promise((resolve) => setTimeout(resolve, 80));
    assert.equal(restartRequests, 1);
  } finally {
    await local.close();
  }
});

test("mints a short-lived same-origin session without exposing a pairing code only when explicitly enabled", async () => {
  const local = createLocalBridge({
    apiKey: "automatic-pair-test-key", pairingCode: "manual-code-must-not-be-returned", allowedOrigin: "http://localhost:5173", startRuns: false,
    allowAutomaticPairing: true,
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
    ...options,
    headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) },
  });
  try {
    const health = await request("/v1/health");
    assert.equal((await health.json()).automaticPairingSupported, true);
    const pairing = await request("/v1/pair/automatic", { method: "POST" });
    assert.equal(pairing.status, 200);
    const payload = await pairing.json();
    assert.equal(typeof payload.sessionToken, "string");
    assert.equal(JSON.stringify(payload).includes("manual-code-must-not-be-returned"), false);
    const foreign = await fetch(`${localOrigin}/v1/pair/automatic`, { method: "POST", headers: { Origin: "https://attacker.invalid" } });
    assert.equal(foreign.status, 403);
  } finally {
    await local.close();
  }
});

test("rejects an unpaired or foreign-origin request before it can create a task", async () => {
  const unpaired = await call("/v1/runs/preview", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "stone lantern", geometryTarget: "BALANCED" }),
  });
  assert.equal(unpaired.status, 401);

  const foreign = await fetch(`${origin}/v1/health`, { headers: { Origin: "https://attacker.invalid" } });
  assert.equal(foreign.status, 403);
});

test("accepts only loopback bind hosts", () => {
  assert.throws(() => createLocalBridge({
    apiKey: "bind-host-test-key", pairingCode: "bind-host-pair", allowedOrigin: "http://localhost:5173", bindHost: "10.0.0.5",
  }), /MESHY_BRIDGE_BIND_HOST/);
  assert.throws(() => createLocalBridge({
    apiKey: "bind-host-test-key", pairingCode: "bind-host-pair", allowedOrigin: "http://localhost:5173", bindHost: "0.0.0.0",
  }), /loopback/);
  assert.doesNotThrow(() => createLocalBridge({
    apiKey: "bind-host-test-key", pairingCode: "docker-bind-pair", allowedOrigin: "http://localhost:5173", bindHost: "0.0.0.0", allowContainerBind: true,
  }));
});

test("formats an IPv6 loopback listener as a usable URL", async () => {
  const local = createLocalBridge({
    apiKey: "ipv6-bind-test-key", pairingCode: "ipv6-bind-pair", allowedOrigin: "http://localhost:5173", bindHost: "::1", startRuns: false,
  });
  try {
    assert.match(await local.listen(0), /^http:\/\/\[::1\]:\d+$/);
  } finally {
    await local.close();
  }
});

test("real E2E runner refuses before any paid operation unless explicitly armed", () => {
  const result = spawnSync(process.execPath, ["real-e2e.mjs"], { cwd: new URL(".", import.meta.url), encoding: "utf8", env: {} });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /MESHY_REAL_E2E=1/);
});

test("multi-animation runner enables moderation by default and accepts only explicit boolean overrides", () => {
  assert.equal(parseModerationFlag(undefined), true);
  assert.equal(parseModerationFlag("1"), true);
  assert.equal(parseModerationFlag("0"), false);
  assert.throws(() => parseModerationFlag("false"), /MESHY_REAL_E2E_MODERATION/);
  assert.throws(() => parseModerationFlag("2"), /MESHY_REAL_E2E_MODERATION/);
});

test("bounds exact-lineage remesh recovery below the owner cap and Meshy rig face limit", () => {
  const options = parseRemeshRecoveryOptions({
    targetPolycount: "299000",
    rigHeightMeters: "2.4",
    actionCount: 10,
  });
  assert.deepEqual(options, {
    targetPolycount: 299_000,
    rigHeightMeters: 2.4,
    actionCount: 10,
  });
  assert.deepEqual(
    calculateRemeshRecoveryCreditGate({
      ownerCap: 250,
      alreadySpentCredits: 30,
      actionCount: options.actionCount,
    }),
    {
      remeshCredits: 5,
      rigCredits: 5,
      animationCredits: 30,
      maximumAdditionalCredits: 40,
      maximumTotalCredits: 70,
    },
  );
  assert.throws(
    () => parseRemeshRecoveryOptions({
      targetPolycount: "300001",
      rigHeightMeters: "2.4",
      actionCount: 10,
    }),
    /100..=300000/,
  );
  assert.throws(
    () => calculateRemeshRecoveryCreditGate({
      ownerCap: 50,
      alreadySpentCredits: 30,
      actionCount: 10,
    }),
    /owner credit cap/,
  );
});

test("checks the live balance before it creates a paid run", async () => {
  const local = createLocalBridge({
    apiKey: "low-credit-test-key", pairingCode: "low-credit-pair", allowedOrigin: "http://localhost:5173",
    meshFetch: async () => new Response(JSON.stringify({ balance: 1 })), startRuns: false,
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "low-credit-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const preview = await (await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "stone lantern", geometryTarget: "BALANCED" }) })).json();
    const create = await request("/v1/runs", { method: "POST", headers, body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: "low-credit-confirmation" }) });
    assert.equal(create.status, 402);
    assert.deepEqual(await create.json(), { code: "INSUFFICIENT_CREDITS", message: "Available Meshy credits are below this profile's maximum cost." });
  } finally {
    await local.close();
  }
});

test("retries only safe Meshy GET requests after bounded 429 responses", async () => {
  let balanceGets = 0;
  const local = createLocalBridge({
    apiKey: "rate-limit-key",
    pairingCode: "rate-limit-pair",
    allowedOrigin: "http://localhost:5173",
    meshFetch: async (url) => {
      if (!url.endsWith("/openapi/v1/balance")) {
        throw new Error(`Unexpected rate-limit URL ${url}`);
      }
      balanceGets += 1;
      if (balanceGets < 3) {
        return new Response(JSON.stringify({ message: "slow down" }), {
          status: 429,
          headers: {
            "Content-Type": "application/json",
            "Retry-After": "0",
          },
        });
      }
      return new Response(JSON.stringify({ balance: 17 }), {
        headers: { "Content-Type": "application/json" },
      });
    },
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
    ...options,
    headers: {
      Origin: "http://localhost:5173",
      ...(options.headers ?? {}),
    },
  });
  try {
    const pairing = await (await request("/v1/pair", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pairingCode: "rate-limit-pair" }),
    })).json();
    const balance = await (await request("/v1/balance", {
      headers: { "X-Meshy-Session": pairing.sessionToken },
    })).json();
    assert.equal(balance.availableCredits, 17);
    assert.equal(balanceGets, 3);
  } finally {
    await local.close();
  }
});

test("requires explicit confirmation before ReTexture and never sends a signed model URL", async () => {
  const calls = [];
  const local = createLocalBridge({
    apiKey: "retexture-test-key", pairingCode: "retexture-pair", allowedOrigin: "http://localhost:5173", startRuns: false,
    meshFetch: async (url, options) => { calls.push({ url, options }); return new Response(JSON.stringify({ balance: 120 }), { headers: { "Content-Type": "application/json" } }); },
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "retexture-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const previewResponse = await request("/v1/retexture/preview", { method: "POST", headers, body: JSON.stringify({ inputTaskId: "verified-refine-task", textStylePrompt: "aged brass and green patina", aiModel: "meshy-6", enableOriginalUv: true, enablePbr: true, textureResolution: "2k", removeLighting: true, alphaThumbnail: false }) });
    assert.equal(previewResponse.status, 200);
    const preview = await previewResponse.json();
    assert.equal(preview.maximumCredits, 10);
    assert.equal(JSON.stringify(preview).includes("http"), false);
    const rejected = await request("/v1/retexture", { method: "POST", headers, body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: "" }) });
    assert.equal(rejected.status, 400);
    const confirmed = await request("/v1/retexture", { method: "POST", headers, body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: "confirm-retexture-once" }) });
    assert.equal(confirmed.status, 200);
    const run = await confirmed.json();
    assert.equal(run.inputTaskId, "verified-refine-task");
    assert.equal(run.textStylePrompt, "aged brass and green patina");
    assert.equal(run.status, "QUEUED");
    assert.equal(run.progress, 0);
    assert.equal(typeof run.id, "string");
    assert.equal(typeof run.createdAt, "string");
    assert.equal(calls.at(-1).url, "https://api.meshy.ai/openapi/v1/balance");
  } finally {
    await local.close();
  }
});

test("accepts a bounded Image to 3D request and rejects invalid API option ranges before a paid run", async () => {
  const local = createLocalBridge({
    apiKey: "image-contract-test-key", pairingCode: "image-contract-pair", allowedOrigin: "http://localhost:5173", startRuns: false,
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  const options = {
    modelType: "standard", aiModel: "latest", shouldRemesh: true, topology: "triangle", targetPolycount: 300_000,
    poseMode: "", moderation: true, targetFormats: ["glb"], alphaThumbnail: false, autoSize: false, originAt: "bottom",
    enablePbr: true, shouldTexture: true, textureResolution: "2k", texturePrompt: "", textureImageUrl: "", removeLighting: true, imageEnhancement: true,
    multiViewThumbnails: false, rigHumanoid: false, rigHeightMeters: 1.7,
  };
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "image-contract-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const valid = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "", geometryTarget: "BALANCED", source: "IMAGE", imageDataUrls: ["data:image/png;base64,AAAA"], apiOptions: options }) });
    assert.equal(valid.status, 200);
    const invalid = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "", geometryTarget: "BALANCED", source: "IMAGE", imageDataUrls: ["data:image/png;base64,AAAA"], apiOptions: { ...options, targetPolycount: 1 } }) });
    assert.equal(invalid.status, 400);
    const overAuroraBudget = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "", geometryTarget: "BALANCED", source: "IMAGE", imageDataUrls: ["data:image/png;base64,AAAA"], apiOptions: { ...options, targetPolycount: 300_001 } }) });
    assert.equal(overAuroraBudget.status, 400);
    const untexturedText = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "A misleading untextured text request", geometryTarget: "BALANCED", source: "TEXT", imageDataUrls: [], apiOptions: { ...options, shouldTexture: false } }) });
    assert.equal(untexturedText.status, 400);
  } finally {
    await local.close();
  }
});

test("accepts Image-to-3D Smart Topology only with its Meshy T model and rejects unsupported data URIs", async () => {
  const local = createLocalBridge({
    apiKey: "smart-topology-test-key", pairingCode: "smart-topology-pair", allowedOrigin: "http://localhost:5173", startRuns: false,
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  const options = {
    modelType: "smart-topology", aiModel: "meshy-t2", shouldRemesh: false, topology: "triangle", targetPolycount: 4_000,
    poseMode: "", moderation: true, targetFormats: ["glb"], alphaThumbnail: false, autoSize: false, originAt: "bottom",
    enablePbr: true, shouldTexture: true, textureResolution: "2k", texturePrompt: "", textureImageUrl: "", removeLighting: true, imageEnhancement: true,
    multiViewThumbnails: false, rigHumanoid: false, rigHeightMeters: 1.7,
  };
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "smart-topology-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const valid = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "", geometryTarget: "BALANCED", source: "IMAGE", imageDataUrls: ["data:image/jpeg;base64,AAAA"], apiOptions: options }) });
    assert.equal(valid.status, 200);
    assert.equal((await valid.json()).maximumCredits, 15);
    const eightKilobyte = await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "S1-static-prop/v1",
        prompt: "",
        geometryTarget: "BALANCED",
        source: "IMAGE",
        imageDataUrls: ["data:image/jpeg;base64,AAAA"],
        apiOptions: { ...options, textureResolution: "8k" },
      }),
    });
    assert.equal(eightKilobyte.status, 200);
    assert.equal((await eightKilobyte.json()).maximumCredits, 20);
    const wrongSource = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "", geometryTarget: "BALANCED", source: "MULTI_IMAGE", imageDataUrls: ["data:image/jpeg;base64,AAAA"], apiOptions: options }) });
    assert.equal(wrongSource.status, 400);
    const unsupportedImage = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "", geometryTarget: "BALANCED", source: "IMAGE", imageDataUrls: ["data:image/webp;base64,AAAA"], apiOptions: options }) });
    assert.equal(unsupportedImage.status, 400);
  } finally {
    await local.close();
  }
});

test("creates a confirmed Text-to-Image task and exposes only its task identity for 3D chaining", async () => {
  const calls = [];
  const fakeMeshy = async (url, options = {}) => {
    calls.push({ url, options });
    if (url.endsWith("/openapi/v1/balance")) return new Response(JSON.stringify({ balance: 20 }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/text-to-image")) return new Response(JSON.stringify({ result: "concept-task" }));
    if (url.endsWith("/openapi/v1/text-to-image/concept-task")) return new Response(JSON.stringify({ id: "concept-task", type: "text-to-image", status: "SUCCEEDED", progress: 100, image_urls: ["https://assets.meshy.ai/signed-concept.png?Expires=secret"] }));
    throw new Error(`Unexpected Meshy URL ${url}`);
  };
  const local = createLocalBridge({ apiKey: "image-run-test-key", pairingCode: "image-run-pair", allowedOrigin: "http://localhost:5173", meshFetch: fakeMeshy });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "image-run-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const preview = await (await request("/v1/image-runs/preview", { method: "POST", headers, body: JSON.stringify({ mode: "TEXT_TO_IMAGE", prompt: "a clear concept image of a wooden lantern", aiModel: "nano-banana", generateMultiView: false, aspectRatio: "1:1" }) })).json();
    assert.equal(preview.maximumCredits, 3);
    const created = await (await request("/v1/image-runs", { method: "POST", headers, body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: "concept-confirmation" }) })).json();
    let status;
    for (let attempt = 0; attempt < 20; attempt += 1) {
      status = await (await request(`/v1/image-runs/${created.id}`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
      if (status.status === "READY") break;
      await new Promise((resolve) => setTimeout(resolve, 5));
    }
    assert.deepEqual(status, { id: created.id, mode: "TEXT_TO_IMAGE", prompt: "a clear concept image of a wooden lantern", aiModel: "nano-banana", generateMultiView: false, status: "READY", progress: 100, taskId: "concept-task" });
    assert.equal(JSON.stringify(status).includes("signed-concept"), false);
    assert.equal(calls.some((call) => call.url.endsWith("/openapi/v1/text-to-image") && JSON.parse(call.options.body).prompt.includes("wooden lantern")), true);
  } finally {
    await local.close();
  }
});

test("chains a completed private 2D task into Image to 3D without sending its signed image URL", async () => {
  const calls = [];
  const fakeMeshy = async (url, options = {}) => {
    calls.push({ url, options });
    if (url === "https://assets.meshy.ai/chained.glb") return new Response(fakeGlb());
    if (url.endsWith("/openapi/v1/balance")) return new Response(JSON.stringify({ balance: 40 }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/text-to-image")) return new Response(JSON.stringify({ result: "concept-task" }));
    if (url.endsWith("/openapi/v1/text-to-image/concept-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, image_urls: ["https://assets.meshy.ai/signed-concept.png?Expires=secret"] }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/image-to-3d")) return new Response(JSON.stringify({ result: "model-task" }));
    if (url.endsWith("/openapi/v1/image-to-3d/model-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, model_urls: { glb: "https://assets.meshy.ai/chained.glb" } }));
    throw new Error(`Unexpected Meshy URL ${url}`);
  };
  const local = createLocalBridge({ apiKey: "chain-test-key", pairingCode: "chain-pair", allowedOrigin: "http://localhost:5173", meshFetch: fakeMeshy });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  const apiOptions = { modelType: "standard", aiModel: "latest", shouldRemesh: true, topology: "triangle", targetPolycount: 300_000, poseMode: "", moderation: true, targetFormats: ["glb"], alphaThumbnail: false, autoSize: false, originAt: "bottom", enablePbr: true, shouldTexture: true, textureResolution: "2k", texturePrompt: "", textureImageUrl: "", removeLighting: true, imageEnhancement: true, multiViewThumbnails: false, rigHumanoid: false, rigHeightMeters: 1.7 };
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "chain-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const imagePreview = await (await request("/v1/image-runs/preview", { method: "POST", headers, body: JSON.stringify({ mode: "TEXT_TO_IMAGE", prompt: "a stone lantern", aiModel: "nano-banana", generateMultiView: false, aspectRatio: "1:1" }) })).json();
    const imageCreated = await (await request("/v1/image-runs", { method: "POST", headers, body: JSON.stringify({ previewId: imagePreview.previewId, confirmationNonce: "chain-image-confirmation" }) })).json();
    let imageStatus;
    for (let attempt = 0; attempt < 20; attempt += 1) {
      imageStatus = await (await request(`/v1/image-runs/${imageCreated.id}`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
      if (imageStatus.status === "READY") break;
      await new Promise((resolve) => setTimeout(resolve, 5));
    }
    const modelPreview = await (await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "a stone lantern", geometryTarget: "BALANCED", source: "IMAGE", inputTaskId: imageStatus.taskId, apiOptions }) })).json();
    const modelCreated = await (await request("/v1/runs", { method: "POST", headers, body: JSON.stringify({ previewId: modelPreview.previewId, confirmationNonce: "chain-model-confirmation" }) })).json();
    for (let attempt = 0; attempt < 20; attempt += 1) {
      const status = await (await request(`/v1/runs/${modelCreated.id}`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
      if (status.status === "READY") break;
      await new Promise((resolve) => setTimeout(resolve, 5));
    }
    const modelRequest = calls.find((call) => call.url.endsWith("/openapi/v1/image-to-3d") && call.options.method === "POST");
    const payload = JSON.parse(modelRequest.options.body);
    assert.equal(payload.input_task_id, "concept-task");
    assert.equal("image_url" in payload, false);
    assert.equal("image_urls" in payload, false);
    assert.equal(JSON.stringify(payload).includes("signed-concept"), false);
  } finally {
    await local.close();
  }
});

test("runs the constrained H1 pipeline and proxies only the verified GLB", async () => {
  const calls = [];
  const fakeMeshy = async (url, options = {}) => {
    calls.push({ url, options });
    if (url === "https://assets.meshy.ai/proof.glb") {
      return new Response(fakeGlb());
    }
    if (url.endsWith("/openapi/v1/balance")) return new Response(JSON.stringify({ balance: 120 }));
    if (options.method === "POST" && url.endsWith("/openapi/v2/text-to-3d")) {
      const request = JSON.parse(options.body);
      return new Response(JSON.stringify({ result: request.mode === "preview" ? "preview-task" : "refine-task" }), { headers: { "Content-Type": "application/json" } });
    }
    if (url.endsWith("/openapi/v2/text-to-3d/preview-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, consumed_credits: 20 }));
    if (url.endsWith("/openapi/v2/text-to-3d/refine-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, consumed_credits: 10, model_urls: { glb: "https://assets.meshy.ai/proof.glb" } }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/rigging")) return new Response(JSON.stringify({ result: "rig-task" }));
    if (url.endsWith("/openapi/v1/rigging/rig-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, consumed_credits: 5, rigged_character_glb_url: "https://assets.meshy.ai/proof.glb" }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/animations")) return new Response(JSON.stringify({ result: "animation-task" }));
    if (url.endsWith("/openapi/v1/animations/animation-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, consumed_credits: 3, result: { animation_glb_url: "https://assets.meshy.ai/proof.glb" } }));
    throw new Error(`Unexpected Meshy test URL ${url}`);
  };
  const local = createLocalBridge({ apiKey: "h1-test-key", pairingCode: "h1-pair", allowedOrigin: "http://localhost:5173", meshFetch: fakeMeshy });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "h1-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const preview = await (await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "H1-humanoid-animated/v1", prompt: "humanoid in A-pose", geometryTarget: "AURORA_PROOF", h1Preflight: { standardHumanoid: true, clearLimbs: true, noWeapon: true, aOrTPose: true } }) })).json();
    const run = await (await request("/v1/runs", { method: "POST", headers, body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: "h1-confirmation" }) })).json();

    let status;
    for (let attempt = 0; attempt < 20; attempt += 1) {
      status = await (await request(`/v1/runs/${run.id}`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
      if (status.status === "READY") break;
      await new Promise((resolve) => setTimeout(resolve, 5));
    }
    assert.equal(status.status, "READY");
    assert.deepEqual(Object.keys(status.taskIds).sort(), ["ANIMATE", "PREVIEW", "REFINE", "RIG"]);
    const previewRequest = calls.find((call) => call.url.endsWith("/openapi/v2/text-to-3d") && JSON.parse(call.options.body).mode === "preview");
    assert.equal(JSON.parse(previewRequest.options.body).target_polycount, 1_500);
    const refineRequest = calls.find((call) => call.url.endsWith("/openapi/v2/text-to-3d") && JSON.parse(call.options.body).mode === "refine");
    assert.equal(JSON.parse(refineRequest.options.body).texture_resolution, "2k");
    assert.equal("hd_texture" in JSON.parse(refineRequest.options.body), false);
    const provenance = await (await request(`/v1/runs/${run.id}/provenance`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
    assert.equal(provenance.byteLength, fakeGlb().byteLength);
    assert.match(provenance.sha256, /^[a-f0-9]{64}$/);
    assert.equal(provenance.consumedCredits, 38);
    assert.deepEqual(
      provenance.taskLedger.map(({ stage, taskId, consumedCredits }) => [
        stage,
        taskId,
        consumedCredits,
      ]),
      [
        ["PREVIEW", "preview-task", 20],
        ["REFINE", "refine-task", 10],
        ["RIG", "rig-task", 5],
        ["ANIMATE", "animation-task", 3],
      ],
    );
    const artifact = await request(`/v1/runs/${run.id}/artifact`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
    assert.deepEqual(
      new Uint8Array(await artifact.arrayBuffer()),
      fakeGlb(),
    );
    assert.equal(calls.every((call) => call.url === "https://assets.meshy.ai/proof.glb" || call.options.headers.Authorization === "Bearer h1-test-key"), true);
  } finally {
    await local.close();
  }
});

test("rejects H1 rig and animation artifacts without an actual skeleton or playable clip", async () => {
  const runCase = async ({ pairingCode, invalidRole }) => {
    const fakeMeshy = async (url, options = {}) => {
      if (url.endsWith("/openapi/v1/balance")) {
        return new Response(JSON.stringify({ balance: 120 }));
      }
      if (url === "https://assets.meshy.ai/model.glb") {
        return new Response(fakeGlb(31));
      }
      if (url === "https://assets.meshy.ai/rigged.glb") {
        if (invalidRole === "RIGGED_CHARACTER") {
          return new Response(createTestGlb({
            marker: 32,
            withSkin: false,
            withAnimation: false,
          }));
        }
        if (invalidRole === "RIGGED_BINDING") {
          return new Response(createTestGlb({
            marker: 32,
            bindSkin: false,
            withAnimation: false,
          }));
        }
        return new Response(fakeGlb(32));
      }
      if (url === "https://assets.meshy.ai/animation.glb") {
        return new Response(createTestGlb({
          marker: 33,
          withAnimation: invalidRole !== "ANIMATION",
          animateJoint: invalidRole !== "NON_JOINT_ANIMATION",
        }));
      }
      if (
        options.method === "POST"
        && url.endsWith("/openapi/v1/image-to-3d")
      ) {
        return new Response(JSON.stringify({ result: "model-task" }));
      }
      if (url.endsWith("/openapi/v1/image-to-3d/model-task")) {
        return new Response(JSON.stringify({
          status: "SUCCEEDED",
          progress: 100,
          model_urls: { glb: "https://assets.meshy.ai/model.glb" },
        }));
      }
      if (options.method === "POST" && url.endsWith("/openapi/v1/rigging")) {
        return new Response(JSON.stringify({ result: "rig-task" }));
      }
      if (url.endsWith("/openapi/v1/rigging/rig-task")) {
        return new Response(JSON.stringify({
          status: "SUCCEEDED",
          progress: 100,
          rigged_character_glb_url: "https://assets.meshy.ai/rigged.glb",
        }));
      }
      if (
        options.method === "POST"
        && url.endsWith("/openapi/v1/animations")
      ) {
        return new Response(JSON.stringify({ result: "animation-task" }));
      }
      if (url.endsWith("/openapi/v1/animations/animation-task")) {
        return new Response(JSON.stringify({
          status: "SUCCEEDED",
          progress: 100,
          result: {
            animation_glb_url: "https://assets.meshy.ai/animation.glb",
          },
        }));
      }
      throw new Error(`Unexpected H1 readback URL ${url}`);
    };
    const local = createLocalBridge({
      apiKey: "h1-readback-key",
      pairingCode,
      allowedOrigin: "http://localhost:5173",
      meshFetch: fakeMeshy,
    });
    const localOrigin = await local.listen(0);
    const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
      ...options,
      headers: {
        Origin: "http://localhost:5173",
        ...(options.headers ?? {}),
      },
    });
    try {
      const pairing = await (await request("/v1/pair", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ pairingCode }),
      })).json();
      const headers = {
        "X-Meshy-Session": pairing.sessionToken,
        "Content-Type": "application/json",
      };
      const preview = await (await request("/v1/runs/preview", {
        method: "POST",
        headers,
        body: JSON.stringify({
          profileId: "H1-humanoid-animated/v1",
          prompt: "",
          source: "IMAGE",
          imageDataUrls: ["data:image/png;base64,AAAA"],
          geometryTarget: "HIGHER_DETAIL",
          h1Preflight: H1_PREFLIGHT,
          apiOptions: h1ApiOptions(),
        }),
      })).json();
      const created = await (await request("/v1/runs", {
        method: "POST",
        headers,
        body: JSON.stringify({
          previewId: preview.previewId,
          confirmationNonce: `confirm-${pairingCode}`,
        }),
      })).json();
      for (let attempt = 0; attempt < 30; attempt += 1) {
        const status = await (await request(`/v1/runs/${created.id}`, {
          headers: { "X-Meshy-Session": pairing.sessionToken },
        })).json();
        if (["FAILED", "PARTIAL"].includes(status.status)) return status;
        await new Promise((resolve) => setTimeout(resolve, 5));
      }
      throw new Error("H1 invalid artifact case did not finish.");
    } finally {
      await local.close();
    }
  };

  const missingSkeleton = await runCase({
    pairingCode: "h1-missing-skeleton",
    invalidRole: "RIGGED_CHARACTER",
  });
  assert.equal(missingSkeleton.status, "FAILED");
  assert.match(
    missingSkeleton.error.message,
    /no actual weighted skeleton binding/,
  );

  const missingSkinBinding = await runCase({
    pairingCode: "h1-missing-skin-binding",
    invalidRole: "RIGGED_BINDING",
  });
  assert.equal(missingSkinBinding.status, "FAILED");
  assert.match(
    missingSkinBinding.error.message,
    /no actual weighted skeleton binding/,
  );

  const missingClip = await runCase({
    pairingCode: "h1-missing-clip",
    invalidRole: "ANIMATION",
  });
  assert.equal(missingClip.status, "PARTIAL");
  assert.match(
    missingClip.animationOutcomes[0].error.message,
    /no playable animation clip/,
  );

  const missingJointChannel = await runCase({
    pairingCode: "h1-missing-joint-channel",
    invalidRole: "NON_JOINT_ANIMATION",
  });
  assert.equal(missingJointChannel.status, "PARTIAL");
  assert.match(
    missingJointChannel.animationOutcomes[0].error.message,
    /no playable animation clip/,
  );
});

test("runs up to ten distinct H1 animation actions and exposes each verified GLB by action id", async () => {
  const calls = [];
  const fakeMeshy = async (url, options = {}) => {
    calls.push({ url, options });
    const assetMatch = url.match(/^https:\/\/assets\.meshy\.ai\/action-([0-9]+)\.glb$/);
    if (assetMatch) return new Response(fakeGlb(Number(assetMatch[1])));
    if (url === "https://assets.meshy.ai/rigged.glb") return new Response(fakeGlb(77));
    if (url === "https://assets.meshy.ai/model.glb") return new Response(fakeGlb(76));
    if (url.endsWith("/openapi/v1/balance")) return new Response(JSON.stringify({ balance: 120 }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/image-to-3d")) return new Response(JSON.stringify({ result: "model-task" }));
    if (url.endsWith("/openapi/v1/image-to-3d/model-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, model_urls: { glb: "https://assets.meshy.ai/model.glb" } }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/rigging")) return new Response(JSON.stringify({ result: "rig-task" }));
    if (url.endsWith("/openapi/v1/rigging/rig-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, rigged_character_glb_url: "https://assets.meshy.ai/rigged.glb" }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/animations")) {
      const { action_id: actionId } = JSON.parse(options.body);
      return new Response(JSON.stringify({ result: `animation-${actionId}` }));
    }
    const animationMatch = url.match(/\/openapi\/v1\/animations\/animation-([0-9]+)$/);
    if (animationMatch) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, result: { animation_glb_url: `https://assets.meshy.ai/action-${animationMatch[1]}.glb` } }));
    throw new Error(`Unexpected Meshy multi-animation URL ${url}`);
  };
  const local = createLocalBridge({ apiKey: "multi-animation-key", pairingCode: "multi-animation-pair", allowedOrigin: "http://localhost:5173", meshFetch: fakeMeshy });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  const apiOptions = {
    modelType: "standard", aiModel: "meshy-6", shouldRemesh: true, topology: "triangle", targetPolycount: 300_000,
    poseMode: "t-pose", moderation: true, targetFormats: ["glb"], alphaThumbnail: false, autoSize: true, originAt: "bottom",
    enablePbr: true, shouldTexture: true, textureResolution: "2k", texturePrompt: "", textureImageUrl: "", removeLighting: true,
    imageEnhancement: true, multiViewThumbnails: false, rigHumanoid: true, rigHeightMeters: 1.85, animationActionIds: [0, 92],
    animationCatalogSnapshotId: "meshy-animation-library-2026-07-28-curated-v1",
    animationCatalogSha256: "661b837e858235d0c375726e123d6963298144d5e40e7c81379ad26d0fba5f4e",
  };
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "multi-animation-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const previewResponse = await request("/v1/runs/preview", {
      method: "POST", headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1", prompt: "", source: "IMAGE",
        imageDataUrls: ["data:image/png;base64,AAAA"], geometryTarget: "BALANCED",
        h1Preflight: { standardHumanoid: true, clearLimbs: true, noWeapon: true, aOrTPose: true }, apiOptions,
      }),
    });
    assert.equal(previewResponse.status, 200);
    const preview = await previewResponse.json();
    assert.equal(preview.maximumCredits, 41);
    const created = await (await request("/v1/runs", { method: "POST", headers, body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: "multi-animation-confirmation" }) })).json();
    let status;
    for (let attempt = 0; attempt < 30; attempt += 1) {
      status = await (await request(`/v1/runs/${created.id}`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
      if (status.status === "READY") break;
      await new Promise((resolve) => setTimeout(resolve, 5));
    }
    assert.equal(status.status, "READY");
    assert.deepEqual(Object.keys(status.taskIds).sort(), ["ANIMATE_0", "ANIMATE_92", "PREVIEW", "RIG"]);
    const provenance = await (await request(`/v1/runs/${created.id}/provenance`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
    assert.deepEqual(provenance.animationArtifacts.map((artifact) => artifact.actionId), [0, 92]);
    const attack = await request(`/v1/runs/${created.id}/artifact/92`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
    assert.deepEqual(
      new Uint8Array(await attack.arrayBuffer()),
      fakeGlb(92),
    );
    assert.equal(calls.filter((call) => call.url.endsWith("/openapi/v1/animations") && call.options.method === "POST").length, 2);
  } finally {
    await local.close();
  }
});

test("rejects unknown animation actions and incomplete H1 confirmation before a paid run", async () => {
  const local = createLocalBridge({
    apiKey: "catalog-test-key",
    pairingCode: "catalog-pair",
    allowedOrigin: "http://localhost:5173",
    meshFetch: async () => new Response(JSON.stringify({ balance: 120 })),
    startRuns: false,
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
    ...options,
    headers: {
      Origin: "http://localhost:5173",
      ...(options.headers ?? {}),
    },
  });
  try {
    const pairing = await (await request("/v1/pair", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pairingCode: "catalog-pair" }),
    })).json();
    const headers = {
      "X-Meshy-Session": pairing.sessionToken,
      "Content-Type": "application/json",
    };
    const unknownAction = await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "humanoid",
        geometryTarget: "BALANCED",
        h1Preflight: H1_PREFLIGHT,
        apiOptions: h1ApiOptions([198]),
      }),
    });
    assert.equal(unknownAction.status, 400);

    const exactBudget = await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "humanoid",
        geometryTarget: "HIGHER_DETAIL",
        h1Preflight: H1_PREFLIGHT,
        apiOptions: h1ApiOptions(),
      }),
    });
    assert.equal(exactBudget.status, 200);

    const overBudget = await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "humanoid",
        geometryTarget: "HIGHER_DETAIL",
        h1Preflight: H1_PREFLIGHT,
        apiOptions: { ...h1ApiOptions(), targetPolycount: 300_001 },
      }),
    });
    assert.equal(overBudget.status, 400);

    const staleCatalogHash = await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "humanoid",
        geometryTarget: "BALANCED",
        h1Preflight: H1_PREFLIGHT,
        apiOptions: {
          ...h1ApiOptions(),
          animationCatalogSha256: "0".repeat(64),
        },
      }),
    });
    assert.equal(staleCatalogHash.status, 400);

    const unsupportedMeshy5Texture = await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "humanoid",
        geometryTarget: "BALANCED",
        h1Preflight: H1_PREFLIGHT,
        apiOptions: {
          ...h1ApiOptions(),
          aiModel: "meshy-5",
          textureResolution: "4k",
        },
      }),
    });
    assert.equal(unsupportedMeshy5Texture.status, 400);

    const incompletePreflight = await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "humanoid",
        geometryTarget: "BALANCED",
        h1Preflight: { ...H1_PREFLIGHT, noWeapon: false },
        apiOptions: h1ApiOptions(),
      }),
    });
    assert.equal(incompletePreflight.status, 400);
    assert.equal((await incompletePreflight.json()).code, "H1_PREFLIGHT_REQUIRED");
  } finally {
    await local.close();
  }
});

test("preserves rig, basic clips, and successful animations when a later action fails", async () => {
  const fakeMeshy = async (url, options = {}) => {
    const assets = new Map([
      ["https://assets.meshy.ai/model.glb", 10],
      ["https://assets.meshy.ai/rigged.glb", 11],
      ["https://assets.meshy.ai/walking.glb", 12],
      ["https://assets.meshy.ai/running.glb", 13],
      ["https://assets.meshy.ai/action-0.glb", 14],
    ]);
    if (assets.has(url)) {
      return new Response(fakeGlb(assets.get(url)));
    }
    if (url.endsWith("/openapi/v1/balance")) {
      return new Response(JSON.stringify({ balance: 120 }));
    }
    if (options.method === "POST" && url.endsWith("/openapi/v1/image-to-3d")) {
      return new Response(JSON.stringify({ result: "model-task" }));
    }
    if (url.endsWith("/openapi/v1/image-to-3d/model-task")) {
      return new Response(JSON.stringify({
        status: "SUCCEEDED",
        progress: 100,
        model_urls: { glb: "https://assets.meshy.ai/model.glb" },
      }));
    }
    if (options.method === "POST" && url.endsWith("/openapi/v1/rigging")) {
      return new Response(JSON.stringify({ result: "rig-task" }));
    }
    if (url.endsWith("/openapi/v1/rigging/rig-task")) {
      return new Response(JSON.stringify({
        status: "SUCCEEDED",
        progress: 100,
        created_at: 1_785_400_000,
        finished_at: 1_785_400_010,
        expires_at: 1_788_000_000,
        consumed_credits: 35,
        rigged_character_glb_url: "https://assets.meshy.ai/rigged.glb",
        basic_animations: {
          walking_glb_url: "https://assets.meshy.ai/walking.glb",
          running_glb_url: "https://assets.meshy.ai/running.glb",
        },
      }), { headers: { "x-api-version": "2026-07-01" } });
    }
    if (options.method === "POST" && url.endsWith("/openapi/v1/animations")) {
      const { action_id: actionId } = JSON.parse(options.body);
      return new Response(JSON.stringify({ result: `animation-${actionId}` }));
    }
    if (url.endsWith("/openapi/v1/animations/animation-0")) {
      return new Response(JSON.stringify({
        status: "SUCCEEDED",
        progress: 100,
        result: { animation_glb_url: "https://assets.meshy.ai/action-0.glb" },
      }));
    }
    if (url.endsWith("/openapi/v1/animations/animation-92")) {
      return new Response(JSON.stringify({
        status: "FAILED",
        progress: 100,
        task_error: { message: "second action failed" },
      }));
    }
    if (url.endsWith("/openapi/v1/animations/animation-178")) {
      return new Response(JSON.stringify({
        status: "CANCELED",
        progress: 100,
      }));
    }
    throw new Error(`Unexpected partial-success URL ${url}`);
  };
  const local = createLocalBridge({
    apiKey: "partial-key",
    pairingCode: "partial-pair",
    allowedOrigin: "http://localhost:5173",
    meshFetch: fakeMeshy,
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
    ...options,
    headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) },
  });
  try {
    const pairing = await (await request("/v1/pair", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pairingCode: "partial-pair" }),
    })).json();
    const headers = {
      "X-Meshy-Session": pairing.sessionToken,
      "Content-Type": "application/json",
    };
    const preview = await (await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "",
        source: "IMAGE",
        imageDataUrls: ["data:image/png;base64,AAAA"],
        geometryTarget: "BALANCED",
        h1Preflight: H1_PREFLIGHT,
        apiOptions: h1ApiOptions([0, 92, 178]),
      }),
    })).json();
    const created = await (await request("/v1/runs", {
      method: "POST",
      headers,
      body: JSON.stringify({
        previewId: preview.previewId,
        confirmationNonce: "partial-confirmation",
      }),
    })).json();
    let status;
    for (let attempt = 0; attempt < 30; attempt += 1) {
      status = await (await request(`/v1/runs/${created.id}`, {
        headers: { "X-Meshy-Session": pairing.sessionToken },
      })).json();
      if (status.status === "PARTIAL") break;
      await new Promise((resolve) => setTimeout(resolve, 5));
    }
    assert.equal(status.status, "PARTIAL");
    assert.deepEqual(
      status.animationOutcomes.map(({ actionId, status: outcome }) => [
        actionId,
        outcome,
      ]),
      [[0, "READY"], [92, "FAILED"], [178, "CANCELED"]],
    );
    const provenance = await (await request(`/v1/runs/${created.id}/provenance`, {
      headers: { "X-Meshy-Session": pairing.sessionToken },
    })).json();
    assert.deepEqual(
      provenance.artifacts.map(({ key }) => key),
      ["source-model", "rigged-character", "basic-walking", "basic-running", "animation-0"],
    );
    assert.deepEqual(
      provenance.animationArtifacts.map(({ actionId }) => actionId),
      [0],
    );
    assert.equal(provenance.sourceModelSha256, provenance.artifacts[0].sha256);
    assert.equal(provenance.riggedBaseSha256, provenance.sha256);
    assert.equal(provenance.rigTaskId, "rig-task");
    assert.deepEqual(provenance.apiVersions, ["2026-07-01"]);
    assert.equal(provenance.consumedCredits, 35);
    assert.deepEqual(
      Object.fromEntries([
        "actionId",
        "actionName",
        "actionCategory",
        "auroraCandidate",
        "taskId",
      ].map((key) => [key, provenance.animationArtifacts[0][key]])),
      {
        actionId: 0,
        actionName: "Idle",
        actionCategory: "BodyMovements",
        auroraCandidate: "cpause1",
        taskId: "animation-0",
      },
    );
    assert.equal(
      provenance.artifacts.every(({ glbReadback }) => (
        glbReadback.status === "PARSED"
      )),
      true,
    );
    for (const [key, marker] of [
      ["basic-walking", 12],
      ["basic-running", 13],
      ["0", 14],
    ]) {
      const artifact = await request(`/v1/runs/${created.id}/artifact/${key}`, {
        headers: { "X-Meshy-Session": pairing.sessionToken },
      });
      assert.equal(new Uint8Array(await artifact.arrayBuffer()).at(-4), marker);
    }
    const root = await request(`/v1/runs/${created.id}/artifact`, {
      headers: { "X-Meshy-Session": pairing.sessionToken },
    });
    assert.equal(new Uint8Array(await root.arrayBuffer()).at(-4), 11);
  } finally {
    await local.close();
  }
});

test("local stop is explicit and cannot be overwritten by an in-flight Meshy response", async () => {
  let resolveRigPoll;
  let animationCreates = 0;
  const fakeMeshy = async (url, options = {}) => {
    if (url === "https://assets.meshy.ai/model.glb") {
      return new Response(fakeGlb(9));
    }
    if (url.endsWith("/openapi/v1/balance")) {
      return new Response(JSON.stringify({ balance: 120 }));
    }
    if (options.method === "POST" && url.endsWith("/openapi/v1/image-to-3d")) {
      return new Response(JSON.stringify({ result: "model-task" }));
    }
    if (url.endsWith("/openapi/v1/image-to-3d/model-task")) {
      return new Response(JSON.stringify({
        status: "SUCCEEDED",
        progress: 100,
        model_urls: { glb: "https://assets.meshy.ai/model.glb" },
      }));
    }
    if (options.method === "POST" && url.endsWith("/openapi/v1/rigging")) {
      return new Response(JSON.stringify({ result: "rig-task" }));
    }
    if (url.endsWith("/openapi/v1/rigging/rig-task")) {
      return new Promise((resolve) => {
        resolveRigPoll = () => resolve(new Response(JSON.stringify({
          status: "SUCCEEDED",
          progress: 100,
          rigged_character_glb_url: "https://assets.meshy.ai/should-not-download.glb",
        })));
      });
    }
    if (options.method === "POST" && url.endsWith("/openapi/v1/animations")) {
      animationCreates += 1;
    }
    throw new Error(`Unexpected stop-tracking URL ${url}`);
  };
  const local = createLocalBridge({
    apiKey: "stop-key",
    pairingCode: "stop-pair",
    allowedOrigin: "http://localhost:5173",
    meshFetch: fakeMeshy,
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
    ...options,
    headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) },
  });
  try {
    const pairing = await (await request("/v1/pair", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pairingCode: "stop-pair" }),
    })).json();
    const headers = {
      "X-Meshy-Session": pairing.sessionToken,
      "Content-Type": "application/json",
    };
    const preview = await (await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "",
        source: "IMAGE",
        imageDataUrls: ["data:image/png;base64,AAAA"],
        geometryTarget: "BALANCED",
        h1Preflight: H1_PREFLIGHT,
        apiOptions: h1ApiOptions(),
      }),
    })).json();
    const created = await (await request("/v1/runs", {
      method: "POST",
      headers,
      body: JSON.stringify({
        previewId: preview.previewId,
        confirmationNonce: "stop-confirmation",
      }),
    })).json();
    for (let attempt = 0; attempt < 30 && !resolveRigPoll; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 5));
    }
    assert.equal(typeof resolveRigPoll, "function");
    const stopped = await (await request(`/v1/runs/${created.id}/cancel`, {
      method: "POST",
      headers,
    })).json();
    assert.equal(stopped.status, "STOPPED_LOCAL");
    assert.match(stopped.error.message, /may continue remotely/);
    resolveRigPoll();
    await new Promise((resolve) => setTimeout(resolve, 10));
    const final = await (await request(`/v1/runs/${created.id}`, {
      headers: { "X-Meshy-Session": pairing.sessionToken },
    })).json();
    assert.equal(final.status, "STOPPED_LOCAL");
    assert.equal(animationCreates, 0);
  } finally {
    await local.close();
  }
});

test("image and ReTexture stop actions remain local and terminal", async () => {
  for (const lane of ["IMAGE", "RETEXTURE"]) {
    let resolvePoll;
    const fakeMeshy = async (url, options = {}) => {
      if (url.endsWith("/openapi/v1/balance")) {
        return new Response(JSON.stringify({ balance: 120 }));
      }
      if (
        options.method === "POST"
        && url.endsWith(lane === "IMAGE"
          ? "/openapi/v1/text-to-image"
          : "/openapi/v1/retexture")
      ) {
        return new Response(JSON.stringify({ result: `${lane.toLowerCase()}-task` }));
      }
      if (url.endsWith(
        lane === "IMAGE"
          ? "/openapi/v1/text-to-image/image-task"
          : "/openapi/v1/retexture/retexture-task",
      )) {
        return new Promise((resolve) => {
          resolvePoll = () => resolve(new Response(JSON.stringify(
            lane === "IMAGE"
              ? {
                  status: "SUCCEEDED",
                  progress: 100,
                  image_urls: ["https://assets.meshy.ai/concept.png"],
                }
              : {
                  status: "SUCCEEDED",
                  progress: 100,
                  model_urls: {
                    glb: "https://assets.meshy.ai/retexture.glb",
                  },
                },
          )));
        });
      }
      throw new Error(`Unexpected ${lane} stop URL ${url}`);
    };
    const local = createLocalBridge({
      apiKey: `${lane.toLowerCase()}-stop-key`,
      pairingCode: `${lane.toLowerCase()}-stop-pair`,
      allowedOrigin: "http://localhost:5173",
      meshFetch: fakeMeshy,
    });
    const localOrigin = await local.listen(0);
    const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
      ...options,
      headers: {
        Origin: "http://localhost:5173",
        ...(options.headers ?? {}),
      },
    });
    try {
      const pairing = await (await request("/v1/pair", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          pairingCode: `${lane.toLowerCase()}-stop-pair`,
        }),
      })).json();
      const headers = {
        "X-Meshy-Session": pairing.sessionToken,
        "Content-Type": "application/json",
      };
      const preview = await (await request(
        lane === "IMAGE"
          ? "/v1/image-runs/preview"
          : "/v1/retexture/preview",
        {
          method: "POST",
          headers,
          body: JSON.stringify(
            lane === "IMAGE"
              ? {
                  mode: "TEXT_TO_IMAGE",
                  prompt: "owner concept",
                  aiModel: "nano-banana",
                  generateMultiView: false,
                  aspectRatio: "1:1",
                }
              : {
                  inputTaskId: "source-task",
                  textStylePrompt: "owner material",
                  aiModel: "meshy-6",
                  enableOriginalUv: true,
                  enablePbr: true,
                  textureResolution: "2k",
                  removeLighting: true,
                  alphaThumbnail: false,
                },
          ),
        },
      )).json();
      const base = lane === "IMAGE" ? "/v1/image-runs" : "/v1/retexture";
      const created = await (await request(base, {
        method: "POST",
        headers,
        body: JSON.stringify({
          previewId: preview.previewId,
          confirmationNonce: `${lane.toLowerCase()}-stop-confirmation`,
        }),
      })).json();
      for (let attempt = 0; attempt < 30 && !resolvePoll; attempt += 1) {
        await new Promise((resolve) => setTimeout(resolve, 5));
      }
      assert.equal(typeof resolvePoll, "function");
      const stopped = await (await request(`${base}/${created.id}/cancel`, {
        method: "POST",
        headers,
      })).json();
      assert.equal(stopped.status, "STOPPED_LOCAL");
      assert.match(stopped.error.message, /may continue remotely/);
      resolvePoll();
      await new Promise((resolve) => setTimeout(resolve, 10));
      const final = await (await request(`${base}/${created.id}`, {
        headers: { "X-Meshy-Session": pairing.sessionToken },
      })).json();
      assert.equal(final.status, "STOPPED_LOCAL");
    } finally {
      await local.close();
    }
  }
});

test("runs N1 and S1 without calling humanoid-only rigging or animation endpoints", async () => {
  for (const profileId of ["N1-quadruped/v1", "S1-static-prop/v1"]) {
    const calls = [];
    const fakeMeshy = async (url, options = {}) => {
      calls.push({ url, options });
      if (url === "https://assets.meshy.ai/static.glb") return new Response(fakeGlb());
      if (url.endsWith("/openapi/v1/balance")) return new Response(JSON.stringify({ balance: 120 }));
      if (options.method === "POST" && url.endsWith("/openapi/v2/text-to-3d")) {
        return new Response(JSON.stringify({ result: JSON.parse(options.body).mode === "preview" ? "preview-task" : "refine-task" }));
      }
      if (url.endsWith("/preview-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100 }));
      if (url.endsWith("/refine-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, model_urls: { glb: "https://assets.meshy.ai/static.glb" } }));
      throw new Error(`Unexpected URL ${url}`);
    };
    const local = createLocalBridge({ apiKey: "profile-test-key", pairingCode: profileId, allowedOrigin: "http://localhost:5173", meshFetch: fakeMeshy });
    const localOrigin = await local.listen(0);
    const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
    try {
      const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: profileId }) })).json();
      const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
      const preview = await (await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId, prompt: "proof asset", geometryTarget: "BALANCED" }) })).json();
      const run = await (await request("/v1/runs", { method: "POST", headers, body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: `${profileId}-nonce` }) })).json();
      let status;
      for (let attempt = 0; attempt < 20; attempt += 1) {
        status = await (await request(`/v1/runs/${run.id}`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
        if (status.status === "READY") break;
        await new Promise((resolve) => setTimeout(resolve, 5));
      }
      assert.equal(status.status, "READY");
      assert.equal(calls.some((call) => call.url.includes("/rigging") || call.url.includes("/animations")), false);
      const previewRequest = calls.find((call) => call.url.endsWith("/openapi/v2/text-to-3d") && JSON.parse(call.options.body).mode === "preview");
      assert.equal(JSON.parse(previewRequest.options.body).target_polycount, 15_000);
    } finally {
      await local.close();
    }
  }
});

test("lists prior Text-to-3D work without signed URLs and recovers a verified refined GLB", async () => {
  const calls = [];
  const fakeMeshy = async (url, options = {}) => {
    calls.push({ url, options });
    if (url === "https://assets.meshy.ai/recovered.glb") return new Response(fakeGlb());
    if (url === "https://assets.meshy.ai/recovered-rigged.glb") return new Response(fakeGlb(21));
    if (url === "https://assets.meshy.ai/recovered-walking.glb") return new Response(fakeGlb(22));
    if (url === "https://assets.meshy.ai/recovered-attack.glb") return new Response(fakeGlb(23));
    if (url === "https://assets.meshy.ai/recovered-thumbnail.png?Expires=secret") return new Response(new Uint8Array([0x89, 0x50, 0x4e, 0x47]), { headers: { "Content-Type": "image/png" } });
    if (url.includes("/openapi/v2/text-to-3d?page_num=1&page_size=50&sort_by=-created_at")) {
      return new Response(JSON.stringify([
        { id: "preview-history-task", type: "text-to-3d-preview", status: "SUCCEEDED", prompt: "preview only", created_at: 10, finished_at: 20, consumed_credits: 10, model_urls: { glb: "https://assets.meshy.ai/must-not-leak.glb" } },
        { id: "refine-history-task", type: "text-to-3d-refine", status: "SUCCEEDED", prompt: "recovered stone golem", created_at: 30, finished_at: 40, consumed_credits: 20, thumbnail_url: "https://assets.meshy.ai/recovered-thumbnail.png?Expires=secret", model_urls: { glb: "https://assets.meshy.ai/recovered.glb" } },
      ]), { headers: { "Content-Type": "application/json" } });
    }
    if (url.includes("/openapi/v1/rigging?page_num=1&page_size=50")) {
      return new Response(JSON.stringify([{
        id: "rig-history-task",
        status: "SUCCEEDED",
        created_at: 50,
        finished_at: 60,
        consumed_credits: 5,
        rigged_character_glb_url: "https://assets.meshy.ai/recovered-rigged.glb",
        basic_animations: {
          walking_glb_url: "https://assets.meshy.ai/recovered-walking.glb",
        },
      }]), { headers: { "Content-Type": "application/json" } });
    }
    if (url.includes("/openapi/v1/animations?page_num=1&page_size=50")) {
      return new Response(JSON.stringify([{
        id: "animation-history-task",
        status: "SUCCEEDED",
        action_id: 4,
        created_at: 70,
        finished_at: 80,
        consumed_credits: 3,
        result: {
          animation_glb_url: "https://assets.meshy.ai/recovered-attack.glb",
        },
      }]), { headers: { "Content-Type": "application/json" } });
    }
    if (url.endsWith("/openapi/v2/text-to-3d/refine-history-task")) {
      return new Response(JSON.stringify({ id: "refine-history-task", type: "text-to-3d-refine", status: "SUCCEEDED", prompt: "recovered stone golem", created_at: 30, finished_at: 40, consumed_credits: 20, thumbnail_url: "https://assets.meshy.ai/recovered-thumbnail.png?Expires=secret", model_urls: { glb: "https://assets.meshy.ai/recovered.glb" } }), { headers: { "Content-Type": "application/json" } });
    }
    if (url.endsWith("/openapi/v1/rigging/rig-history-task")) {
      return new Response(JSON.stringify({
        id: "rig-history-task",
        status: "SUCCEEDED",
        rigged_character_glb_url: "https://assets.meshy.ai/recovered-rigged.glb",
        basic_animations: {
          walking_glb_url: "https://assets.meshy.ai/recovered-walking.glb",
        },
      }), { headers: { "Content-Type": "application/json" } });
    }
    if (url.endsWith("/openapi/v1/animations/animation-history-task")) {
      return new Response(JSON.stringify({
        id: "animation-history-task",
        status: "SUCCEEDED",
        action_id: 4,
        result: {
          animation_glb_url: "https://assets.meshy.ai/recovered-attack.glb",
        },
      }), { headers: { "Content-Type": "application/json" } });
    }
    throw new Error(`Unexpected Meshy history URL ${url}`);
  };
  const local = createLocalBridge({ apiKey: "history-test-key", pairingCode: "history-pair", allowedOrigin: "http://localhost:5173", meshFetch: fakeMeshy, startRuns: false });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "history-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken };
    const history = await (await request("/v1/history?page_num=1&page_size=50", { headers })).json();
    assert.deepEqual(history.items.map((item) => item.taskId), [
      "animation-history-task",
      "rig-history-task",
      "refine-history-task",
      "preview-history-task",
    ]);
    assert.equal(JSON.stringify(history).includes("must-not-leak"), false);
    assert.equal(JSON.stringify(history).includes("Expires=secret"), false);
    assert.equal(history.items[0].glbAvailable, true);
    assert.equal(history.items[2].thumbnailAvailable, true);

    const thumbnail = await request("/v1/history/refine/refine-history-task/thumbnail", { headers });
    assert.equal(thumbnail.headers.get("content-type"), "image/png");
    assert.deepEqual([...new Uint8Array(await thumbnail.arrayBuffer())], [0x89, 0x50, 0x4e, 0x47]);

    const provenance = await (await request("/v1/history/refine/refine-history-task/provenance", { headers })).json();
    assert.equal(provenance.profileId, "RECOVERED-text-to-3d/v1");
    assert.deepEqual(provenance.taskIds, { REFINE: "refine-history-task" });
    const artifact = await request("/v1/history/refine/refine-history-task/artifact", { headers });
    assert.deepEqual(
      new Uint8Array(await artifact.arrayBuffer()),
      fakeGlb(),
    );
    const rigged = await request("/v1/history/rig/rig-history-task/artifact", { headers });
    assert.equal(new Uint8Array(await rigged.arrayBuffer()).at(-4), 21);
    const walking = await request("/v1/history/rig/rig-history-task/artifact/basic-walking", { headers });
    assert.equal(new Uint8Array(await walking.arrayBuffer()).at(-4), 22);
    const animation = await request("/v1/history/animate/animation-history-task/artifact", { headers });
    assert.equal(new Uint8Array(await animation.arrayBuffer()).at(-4), 23);
    assert.equal(calls.some((call) => call.url === "https://assets.meshy.ai/recovered.glb"), true);
    assert.equal(calls.every((call) => call.url.startsWith("https://assets.meshy.ai/recovered") || call.options.headers.Authorization === "Bearer history-test-key"), true);
  } finally {
    await local.close();
  }
});
