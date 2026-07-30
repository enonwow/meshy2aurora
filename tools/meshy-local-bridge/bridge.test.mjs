import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { after, before, test } from "node:test";
import { createLocalBridge } from "./index.mjs";
import { parseModerationFlag } from "./real-image-multi-animation-options.mjs";
import {
  calculateRemeshRecoveryCreditGate,
  parseRemeshRecoveryOptions,
  planAutomaticRigFaceRecovery,
} from "./remesh-rig-animation-recovery-options.mjs";

const GLB_MAGIC = 0x46546c67;
const JSON_CHUNK = 0x4e4f534a;
const BIN_CHUNK = 0x004e4942;

function encodeAnimationGlb(document, bin) {
  document.buffers = [{ byteLength: bin.length }];
  const rawJson = Buffer.from(JSON.stringify(document));
  const json = Buffer.concat([rawJson, Buffer.alloc((4 - rawJson.length % 4) % 4, 0x20)]);
  const paddedBin = Buffer.concat([bin, Buffer.alloc((4 - bin.length % 4) % 4)]);
  const output = Buffer.alloc(12 + 8 + json.length + 8 + paddedBin.length);
  output.writeUInt32LE(GLB_MAGIC, 0);
  output.writeUInt32LE(2, 4);
  output.writeUInt32LE(output.length, 8);
  output.writeUInt32LE(json.length, 12);
  output.writeUInt32LE(JSON_CHUNK, 16);
  json.copy(output, 20);
  const binHeader = 20 + json.length;
  output.writeUInt32LE(paddedBin.length, binHeader);
  output.writeUInt32LE(BIN_CHUNK, binHeader + 4);
  paddedBin.copy(output, binHeader + 8);
  return output;
}

function animationGlb(actionId) {
  const bin = Buffer.alloc(40);
  bin.writeFloatLE(0, 0);
  bin.writeFloatLE(1, 4);
  bin.writeFloatLE(0, 8);
  bin.writeFloatLE(0, 12);
  bin.writeFloatLE(0, 16);
  bin.writeFloatLE(1, 20);
  bin.writeFloatLE(0, 24);
  bin.writeFloatLE(actionId / 1000, 28);
  bin.writeFloatLE(0, 32);
  bin.writeFloatLE(1, 36);
  return encodeAnimationGlb({
    asset: { version: "2.0" },
    scene: 0,
    scenes: [{ nodes: [0] }],
    nodes: [{ name: "Armature", children: [1] }, { name: "Hips" }],
    skins: [{ joints: [1], skeleton: 1 }],
    bufferViews: [{ buffer: 0, byteOffset: 0, byteLength: 8 }, { buffer: 0, byteOffset: 8, byteLength: 32 }],
    accessors: [
      { bufferView: 0, componentType: 5126, count: 2, type: "SCALAR", min: [0], max: [1] },
      { bufferView: 1, componentType: 5126, count: 2, type: "VEC4" },
    ],
    animations: [{
      name: `MeshyAction${actionId}`,
      samplers: [{ input: 0, output: 1, interpolation: "LINEAR" }],
      channels: [{ sampler: 0, target: { node: 1, path: "rotation" } }],
    }],
  }, bin);
}

function parseGlbJson(bytes) {
  const buffer = Buffer.from(bytes);
  const jsonLength = buffer.readUInt32LE(12);
  return JSON.parse(buffer.toString("utf8", 20, 20 + jsonLength).trimEnd());
}

function triangleCountGlb(triangleCount) {
  const indexCount = triangleCount * 3;
  const bin = Buffer.alloc(indexCount * 4);
  return encodeAnimationGlb({
    asset: { version: "2.0" },
    scene: 0,
    scenes: [{ nodes: [0] }],
    nodes: [{ name: "Model", mesh: 0 }],
    meshes: [{ primitives: [{ attributes: {}, indices: 0, mode: 4 }] }],
    bufferViews: [{ buffer: 0, byteOffset: 0, byteLength: bin.length }],
    accessors: [{ bufferView: 0, componentType: 5125, count: indexCount, type: "SCALAR" }],
  }, bin);
}

let bridge;
let origin;
const meshCalls = [];

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
  assert.deepEqual(
    planAutomaticRigFaceRecovery({
      observedFaceCount: 300_000,
      requestedTargetPolycount: 300_000,
    }),
    { required: false, observedFaceCount: 300_000 },
  );
  assert.deepEqual(
    planAutomaticRigFaceRecovery({
      observedFaceCount: 300_844,
      requestedTargetPolycount: 300_000,
    }),
    {
      required: true,
      observedFaceCount: 300_844,
      targetPolycount: 295_000,
      reason: "generated_model_exceeds_meshy_rig_face_limit",
    },
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
    const previewResponse = await request("/v1/retexture/preview", { method: "POST", headers, body: JSON.stringify({ inputTaskId: "verified-refine-task", textStylePrompt: "aged brass and green patina", aiModel: "meshy-6", enableOriginalUv: true, enablePbr: true, hdTexture: false, removeLighting: true, alphaThumbnail: false }) });
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
    modelType: "standard", aiModel: "latest", shouldRemesh: true, topology: "triangle", targetPolycount: 30_000,
    poseMode: "", moderation: true, targetFormats: ["glb"], alphaThumbnail: false, autoSize: false, originAt: "bottom",
    enablePbr: true, shouldTexture: true, hdTexture: false, texturePrompt: "", textureImageUrl: "", removeLighting: true, imageEnhancement: true,
    multiViewThumbnails: false, rigHumanoid: false, rigHeightMeters: 1.7,
  };
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "image-contract-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const valid = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "", geometryTarget: "BALANCED", source: "IMAGE", imageDataUrls: ["data:image/png;base64,AAAA"], apiOptions: options }) });
    assert.equal(valid.status, 200);
    const invalid = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "", geometryTarget: "BALANCED", source: "IMAGE", imageDataUrls: ["data:image/png;base64,AAAA"], apiOptions: { ...options, targetPolycount: 1 } }) });
    assert.equal(invalid.status, 400);
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
    enablePbr: true, shouldTexture: true, hdTexture: false, texturePrompt: "", textureImageUrl: "", removeLighting: true, imageEnhancement: true,
    multiViewThumbnails: false, rigHumanoid: false, rigHeightMeters: 1.7,
  };
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "smart-topology-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const valid = await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "S1-static-prop/v1", prompt: "", geometryTarget: "BALANCED", source: "IMAGE", imageDataUrls: ["data:image/jpeg;base64,AAAA"], apiOptions: options }) });
    assert.equal(valid.status, 200);
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
    if (url === "https://assets.meshy.ai/chained.glb") return new Response(new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
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
  const apiOptions = { modelType: "standard", aiModel: "latest", shouldRemesh: true, topology: "triangle", targetPolycount: 30_000, poseMode: "", moderation: true, targetFormats: ["glb"], alphaThumbnail: false, autoSize: false, originAt: "bottom", enablePbr: true, shouldTexture: true, hdTexture: false, texturePrompt: "", textureImageUrl: "", removeLighting: true, imageEnhancement: true, multiViewThumbnails: false, rigHumanoid: false, rigHeightMeters: 1.7 };
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
      return new Response(animationGlb(0));
    }
    if (url.endsWith("/openapi/v1/balance")) return new Response(JSON.stringify({ balance: 120 }));
    if (options.method === "POST" && url.endsWith("/openapi/v2/text-to-3d")) {
      const request = JSON.parse(options.body);
      return new Response(JSON.stringify({ result: request.mode === "preview" ? "preview-task" : "refine-task" }), { headers: { "Content-Type": "application/json" } });
    }
    if (url.endsWith("/openapi/v2/text-to-3d/preview-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100 }));
    if (url.endsWith("/openapi/v2/text-to-3d/refine-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, model_urls: { glb: "https://assets.meshy.ai/proof.glb" } }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/rigging")) return new Response(JSON.stringify({ result: "rig-task" }));
    if (url.endsWith("/openapi/v1/rigging/rig-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100 }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/animations")) return new Response(JSON.stringify({ result: "animation-task" }));
    if (url.endsWith("/openapi/v1/animations/animation-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, result: { animation_glb_url: "https://assets.meshy.ai/proof.glb" } }));
    throw new Error(`Unexpected Meshy test URL ${url}`);
  };
  const local = createLocalBridge({ apiKey: "h1-test-key", pairingCode: "h1-pair", allowedOrigin: "http://localhost:5173", meshFetch: fakeMeshy });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, { ...options, headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) } });
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "h1-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const preview = await (await request("/v1/runs/preview", { method: "POST", headers, body: JSON.stringify({ profileId: "H1-humanoid-animated/v1", prompt: "humanoid in A-pose", geometryTarget: "AURORA_PROOF", h1Preflight: { standardHumanoid: true, clearLimbs: true, noWeapon: true } }) })).json();
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
    const provenance = await (await request(`/v1/runs/${run.id}/provenance`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
    assert.equal(provenance.artifactKind, "MERGED_ANIMATION_GLTF");
    assert.deepEqual(provenance.animationArtifacts.map(({ actionId, clipName }) => ({ actionId, clipName })), [
      { actionId: 0, clipName: "cpause1" },
    ]);
    assert.match(provenance.sha256, /^[a-f0-9]{64}$/);
    const artifact = await request(`/v1/runs/${run.id}/artifact`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
    const artifactBytes = new Uint8Array(await artifact.arrayBuffer());
    assert.equal(artifactBytes.byteLength, provenance.byteLength);
    assert.deepEqual(parseGlbJson(artifactBytes).animations.map((animation) => animation.name), ["cpause1"]);
    assert.equal(calls.every((call) => call.url === "https://assets.meshy.ai/proof.glb" || call.options.headers.Authorization === "Bearer h1-test-key"), true);
  } finally {
    await local.close();
  }
});

test("merges up to ten explicitly mapped H1 actions into the canonical import GLB and preserves each source artifact", async () => {
  const calls = [];
  const fakeMeshy = async (url, options = {}) => {
    calls.push({ url, options });
    const assetMatch = url.match(/^https:\/\/assets\.meshy\.ai\/action-([0-9]+)\.glb$/);
    if (assetMatch) return new Response(animationGlb(Number(assetMatch[1])));
    if (url === "https://assets.meshy.ai/model.glb") return new Response(animationGlb(0));
    if (url.endsWith("/openapi/v1/balance")) return new Response(JSON.stringify({ balance: 120 }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/image-to-3d")) return new Response(JSON.stringify({ result: "model-task" }));
    if (url.endsWith("/openapi/v1/image-to-3d/model-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100, model_urls: { glb: "https://assets.meshy.ai/model.glb" } }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/rigging")) return new Response(JSON.stringify({ result: "rig-task" }));
    if (url.endsWith("/openapi/v1/rigging/rig-task")) return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100 }));
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
    modelType: "standard", aiModel: "meshy-6", shouldRemesh: true, topology: "triangle", targetPolycount: 20_000,
    poseMode: "t-pose", moderation: true, targetFormats: ["glb"], alphaThumbnail: false, autoSize: true, originAt: "bottom",
    enablePbr: true, shouldTexture: true, hdTexture: false, texturePrompt: "", textureImageUrl: "", removeLighting: true,
    imageEnhancement: true, multiViewThumbnails: false, rigHumanoid: true, rigHeightMeters: 1.85,
    animationActions: [{ actionId: 0, clipName: "cpause1" }, { actionId: 198, clipName: "ca1slashl" }],
  };
  try {
    const pairing = await (await request("/v1/pair", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ pairingCode: "multi-animation-pair" }) })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const previewResponse = await request("/v1/runs/preview", {
      method: "POST", headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1", prompt: "", source: "IMAGE",
        imageDataUrls: ["data:image/png;base64,AAAA"], geometryTarget: "BALANCED",
        h1Preflight: { standardHumanoid: true, clearLimbs: true, noWeapon: true }, apiOptions,
      }),
    });
    assert.equal(previewResponse.status, 200);
    const preview = await previewResponse.json();
    assert.equal(preview.maximumCredits, 46);
    const created = await (await request("/v1/runs", { method: "POST", headers, body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: "multi-animation-confirmation" }) })).json();
    let status;
    for (let attempt = 0; attempt < 30; attempt += 1) {
      status = await (await request(`/v1/runs/${created.id}`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
      if (status.status === "READY") break;
      await new Promise((resolve) => setTimeout(resolve, 5));
    }
    assert.equal(status.status, "READY");
    assert.deepEqual(Object.keys(status.taskIds).sort(), ["ANIMATE_0", "ANIMATE_198", "PREVIEW", "RIG"]);
    const provenance = await (await request(`/v1/runs/${created.id}/provenance`, { headers: { "X-Meshy-Session": pairing.sessionToken } })).json();
    assert.equal(provenance.artifactKind, "MERGED_ANIMATION_GLTF");
    assert.deepEqual(provenance.animationArtifacts.map(({ actionId, clipName }) => ({ actionId, clipName })), [
      { actionId: 0, clipName: "cpause1" },
      { actionId: 198, clipName: "ca1slashl" },
    ]);
    const merged = await request(`/v1/runs/${created.id}/artifact`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
    const mergedBytes = new Uint8Array(await merged.arrayBuffer());
    assert.equal(mergedBytes.byteLength, provenance.byteLength);
    assert.deepEqual(parseGlbJson(mergedBytes).animations.map((animation) => animation.name), ["cpause1", "ca1slashl"]);
    const attack = await request(`/v1/runs/${created.id}/artifact/198`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
    assert.deepEqual(new Uint8Array(await attack.arrayBuffer()), new Uint8Array(animationGlb(198)));
    assert.equal(calls.filter((call) => call.url.endsWith("/openapi/v1/animations") && call.options.method === "POST").length, 2);
  } finally {
    await local.close();
  }
});

test("automatically remeshes an over-300K generated humanoid before rigging", async () => {
  const calls = [];
  const original = triangleCountGlb(300_844);
  const remeshed = triangleCountGlb(295_000);
  const fakeMeshy = async (url, options = {}) => {
    calls.push({ url, options });
    if (url === "https://assets.meshy.ai/overshoot.glb") return new Response(original);
    if (url === "https://assets.meshy.ai/remeshed.glb") return new Response(remeshed);
    if (url === "https://assets.meshy.ai/idle.glb") return new Response(animationGlb(0));
    if (url.endsWith("/openapi/v1/balance")) return new Response(JSON.stringify({ balance: 120 }));
    if (options.method === "POST" && url.endsWith("/openapi/v1/image-to-3d")) {
      return new Response(JSON.stringify({ result: "overshoot-model-task" }));
    }
    if (url.endsWith("/openapi/v1/image-to-3d/overshoot-model-task")) {
      return new Response(JSON.stringify({
        status: "SUCCEEDED",
        progress: 100,
        model_urls: { glb: "https://assets.meshy.ai/overshoot.glb" },
      }));
    }
    if (options.method === "POST" && url.endsWith("/openapi/v1/remesh")) {
      return new Response(JSON.stringify({ result: "automatic-remesh-task" }));
    }
    if (url.endsWith("/openapi/v1/remesh/automatic-remesh-task")) {
      return new Response(JSON.stringify({
        status: "SUCCEEDED",
        progress: 100,
        model_urls: { glb: "https://assets.meshy.ai/remeshed.glb" },
      }));
    }
    if (options.method === "POST" && url.endsWith("/openapi/v1/rigging")) {
      return new Response(JSON.stringify({ result: "recovered-rig-task" }));
    }
    if (url.endsWith("/openapi/v1/rigging/recovered-rig-task")) {
      return new Response(JSON.stringify({ status: "SUCCEEDED", progress: 100 }));
    }
    if (options.method === "POST" && url.endsWith("/openapi/v1/animations")) {
      return new Response(JSON.stringify({ result: "recovered-idle-task" }));
    }
    if (url.endsWith("/openapi/v1/animations/recovered-idle-task")) {
      return new Response(JSON.stringify({
        status: "SUCCEEDED",
        progress: 100,
        result: { animation_glb_url: "https://assets.meshy.ai/idle.glb" },
      }));
    }
    throw new Error(`Unexpected Meshy automatic recovery URL ${url}`);
  };
  const local = createLocalBridge({
    apiKey: "automatic-recovery-key",
    pairingCode: "automatic-recovery-pair",
    allowedOrigin: "http://localhost:5173",
    meshFetch: fakeMeshy,
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
    ...options,
    headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) },
  });
  const apiOptions = {
    modelType: "standard", aiModel: "meshy-6", shouldRemesh: true, topology: "triangle", targetPolycount: 300_000,
    poseMode: "t-pose", moderation: true, targetFormats: ["glb"], alphaThumbnail: false, autoSize: true, originAt: "bottom",
    enablePbr: true, shouldTexture: true, hdTexture: false, texturePrompt: "", textureImageUrl: "", removeLighting: true,
    imageEnhancement: true, multiViewThumbnails: false, rigHumanoid: true, rigHeightMeters: 2.4,
    animationActions: [{ actionId: 0, clipName: "cpause1" }],
  };
  try {
    const pairing = await (await request("/v1/pair", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pairingCode: "automatic-recovery-pair" }),
    })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const preview = await (await request("/v1/runs/preview", {
      method: "POST",
      headers,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "",
        source: "IMAGE",
        imageDataUrls: ["data:image/png;base64,AAAA"],
        geometryTarget: "HIGHER_DETAIL",
        h1Preflight: { standardHumanoid: true, clearLimbs: true, noWeapon: true },
        apiOptions,
      }),
    })).json();
    assert.equal(preview.maximumCredits, 43);
    const created = await (await request("/v1/runs", {
      method: "POST",
      headers,
      body: JSON.stringify({
        previewId: preview.previewId,
        confirmationNonce: "automatic-recovery-confirmation",
      }),
    })).json();
    let status;
    for (let attempt = 0; attempt < 50; attempt += 1) {
      status = await (await request(`/v1/runs/${created.id}`, {
        headers: { "X-Meshy-Session": pairing.sessionToken },
      })).json();
      if (["READY", "FAILED"].includes(status.status)) break;
      await new Promise((resolve) => setTimeout(resolve, 5));
    }
    assert.equal(status.status, "READY");
    assert.deepEqual(Object.keys(status.taskIds).sort(), [
      "ANIMATE", "PREVIEW", "REMESH", "RIG",
    ]);
    const remeshRequest = calls.find((call) => (
      call.url.endsWith("/openapi/v1/remesh") && call.options.method === "POST"
    ));
    assert.deepEqual(JSON.parse(remeshRequest.options.body), {
      input_task_id: "overshoot-model-task",
      target_formats: ["glb"],
      topology: "triangle",
      target_polycount: 295_000,
      alpha_thumbnail: false,
    });
    const rigRequest = calls.find((call) => (
      call.url.endsWith("/openapi/v1/rigging") && call.options.method === "POST"
    ));
    assert.equal(JSON.parse(rigRequest.options.body).input_task_id, "automatic-remesh-task");
    const provenance = await (await request(`/v1/runs/${created.id}/provenance`, {
      headers: { "X-Meshy-Session": pairing.sessionToken },
    })).json();
    assert.deepEqual(provenance.geometryAdmission, {
      generatedFaceCount: 300_844,
      rigFaceLimit: 300_000,
      automaticRemeshApplied: true,
      remeshTaskId: "automatic-remesh-task",
      remeshTargetPolycount: 295_000,
      remeshedFaceCount: 295_000,
    });
  } finally {
    await local.close();
  }
});

test("rejects H1 animation mappings without cpause1 or with duplicate NWN clip names before creating a paid run", async () => {
  const local = createLocalBridge({
    apiKey: "mapping-validation-key",
    pairingCode: "mapping-validation-pair",
    allowedOrigin: "http://localhost:5173",
    startRuns: false,
  });
  const localOrigin = await local.listen(0);
  const request = (path, options = {}) => fetch(`${localOrigin}${path}`, {
    ...options,
    headers: { Origin: "http://localhost:5173", ...(options.headers ?? {}) },
  });
  const baseOptions = {
    modelType: "standard", aiModel: "meshy-6", shouldRemesh: true, topology: "triangle", targetPolycount: 20_000,
    poseMode: "t-pose", moderation: true, targetFormats: ["glb"], alphaThumbnail: false, autoSize: true, originAt: "bottom",
    enablePbr: true, shouldTexture: true, hdTexture: false, texturePrompt: "", textureImageUrl: "", removeLighting: true,
    imageEnhancement: true, multiViewThumbnails: false, rigHumanoid: true, rigHeightMeters: 1.85,
  };
  try {
    const pairing = await (await request("/v1/pair", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pairingCode: "mapping-validation-pair" }),
    })).json();
    const headers = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    for (const animationActions of [
      [{ actionId: 198, clipName: "ca1slashl" }],
      [{ actionId: 0, clipName: "cpause1" }, { actionId: 198, clipName: "cpause1" }],
    ]) {
      const response = await request("/v1/runs/preview", {
        method: "POST",
        headers,
        body: JSON.stringify({
          profileId: "H1-humanoid-animated/v1",
          prompt: "humanoid",
          source: "TEXT",
          geometryTarget: "BALANCED",
          h1Preflight: { standardHumanoid: true, clearLimbs: true, noWeapon: true },
          apiOptions: { ...baseOptions, animationActions },
        }),
      });
      assert.equal(response.status, 400);
    }
  } finally {
    await local.close();
  }
});

test("runs N1 and S1 without calling humanoid-only rigging or animation endpoints", async () => {
  for (const profileId of ["N1-quadruped/v1", "S1-static-prop/v1"]) {
    const calls = [];
    const fakeMeshy = async (url, options = {}) => {
      calls.push({ url, options });
      if (url === "https://assets.meshy.ai/static.glb") return new Response(new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
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
    } finally {
      await local.close();
    }
  }
});

test("lists prior Text-to-3D work without signed URLs and recovers a verified refined GLB", async () => {
  const calls = [];
  const fakeMeshy = async (url, options = {}) => {
    calls.push({ url, options });
    if (url === "https://assets.meshy.ai/recovered.glb") return new Response(new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
    if (url === "https://assets.meshy.ai/recovered-thumbnail.png?Expires=secret") return new Response(new Uint8Array([0x89, 0x50, 0x4e, 0x47]), { headers: { "Content-Type": "image/png" } });
    if (url.includes("/openapi/v2/text-to-3d?page_num=1&page_size=50&sort_by=-created_at")) {
      return new Response(JSON.stringify([
        { id: "preview-history-task", type: "text-to-3d-preview", status: "SUCCEEDED", prompt: "preview only", created_at: 10, finished_at: 20, consumed_credits: 10, model_urls: { glb: "https://assets.meshy.ai/must-not-leak.glb" } },
        { id: "refine-history-task", type: "text-to-3d-refine", status: "SUCCEEDED", prompt: "recovered stone golem", created_at: 30, finished_at: 40, consumed_credits: 20, thumbnail_url: "https://assets.meshy.ai/recovered-thumbnail.png?Expires=secret", model_urls: { glb: "https://assets.meshy.ai/recovered.glb" } },
      ]), { headers: { "Content-Type": "application/json" } });
    }
    if (url.endsWith("/openapi/v2/text-to-3d/refine-history-task")) {
      return new Response(JSON.stringify({ id: "refine-history-task", type: "text-to-3d-refine", status: "SUCCEEDED", prompt: "recovered stone golem", created_at: 30, finished_at: 40, consumed_credits: 20, thumbnail_url: "https://assets.meshy.ai/recovered-thumbnail.png?Expires=secret", model_urls: { glb: "https://assets.meshy.ai/recovered.glb" } }), { headers: { "Content-Type": "application/json" } });
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
    assert.deepEqual(history.items.map((item) => item.taskId), ["preview-history-task", "refine-history-task"]);
    assert.equal(JSON.stringify(history).includes("must-not-leak"), false);
    assert.equal(JSON.stringify(history).includes("Expires=secret"), false);
    assert.equal(history.items[1].glbAvailable, true);
    assert.equal(history.items[1].thumbnailAvailable, true);

    const thumbnail = await request("/v1/history/refine-history-task/thumbnail", { headers });
    assert.equal(thumbnail.headers.get("content-type"), "image/png");
    assert.deepEqual([...new Uint8Array(await thumbnail.arrayBuffer())], [0x89, 0x50, 0x4e, 0x47]);

    const provenance = await (await request("/v1/history/refine-history-task/provenance", { headers })).json();
    assert.equal(provenance.profileId, "RECOVERED-text-to-3d/v1");
    assert.deepEqual(provenance.taskIds, { REFINE: "refine-history-task" });
    const artifact = await request("/v1/history/refine-history-task/artifact", { headers });
    assert.deepEqual([...new Uint8Array(await artifact.arrayBuffer())], [0x67, 0x6c, 0x54, 0x46]);
    assert.equal(calls.some((call) => call.url === "https://assets.meshy.ai/recovered.glb"), true);
    assert.equal(calls.every((call) => ["https://assets.meshy.ai/recovered.glb", "https://assets.meshy.ai/recovered-thumbnail.png?Expires=secret"].includes(call.url) || call.options.headers.Authorization === "Bearer history-test-key"), true);
  } finally {
    await local.close();
  }
});
