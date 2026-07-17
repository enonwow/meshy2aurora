import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { after, before, test } from "node:test";
import { createLocalBridge } from "./index.mjs";

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
  assert.deepEqual(await health.json(), { protocolVersion: 1, bridge: "LOCAL", status: "READY" });

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

test("real E2E runner refuses before any paid operation unless explicitly armed", () => {
  const result = spawnSync(process.execPath, ["real-e2e.mjs"], { cwd: new URL(".", import.meta.url), encoding: "utf8", env: {} });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /MESHY_REAL_E2E=1/);
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

test("runs the constrained H1 pipeline and proxies only the verified GLB", async () => {
  const calls = [];
  const fakeMeshy = async (url, options = {}) => {
    calls.push({ url, options });
    if (url === "https://assets.meshy.ai/proof.glb") {
      return new Response(new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
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
    assert.equal(provenance.byteLength, 4);
    assert.match(provenance.sha256, /^[a-f0-9]{64}$/);
    const artifact = await request(`/v1/runs/${run.id}/artifact`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
    assert.deepEqual([...new Uint8Array(await artifact.arrayBuffer())], [0x67, 0x6c, 0x54, 0x46]);
    assert.equal(calls.every((call) => call.url === "https://assets.meshy.ai/proof.glb" || call.options.headers.Authorization === "Bearer h1-test-key"), true);
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
