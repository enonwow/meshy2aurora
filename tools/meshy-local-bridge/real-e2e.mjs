import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { createLocalBridge } from "./index.mjs";

const profiles = new Set(["H1-humanoid-animated/v1", "N1-quadruped/v1", "S1-static-prop/v1"]);
const geometryTargets = new Set(["AURORA_PROOF", "LOWER_DETAIL", "BALANCED", "HIGHER_DETAIL"]);

function required(name) {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required.`);
  return value;
}

function terminal(status) {
  return status === "READY" || status === "FAILED" || status === "CANCELED";
}

function glbTriangles(bytes) {
  const buffer = Buffer.from(bytes);
  if (buffer.length < 20 || buffer.toString("ascii", 0, 4) !== "glTF" || buffer.readUInt32LE(4) !== 2 || buffer.toString("ascii", 16, 20) !== "JSON") {
    throw new Error("Meshy did not return a valid GLB 2.0 artifact.");
  }
  const jsonLength = buffer.readUInt32LE(12);
  const gltf = JSON.parse(buffer.toString("utf8", 20, 20 + jsonLength).replace(/\0+$/, ""));
  return (gltf.meshes ?? []).flatMap((mesh) => mesh.primitives ?? []).reduce((total, primitive) => {
    if ((primitive.mode ?? 4) !== 4 || typeof primitive.indices !== "number") return total;
    return total + (gltf.accessors?.[primitive.indices]?.count ?? 0) / 3;
  }, 0);
}

async function main() {
  if (process.env.MESHY_REAL_E2E !== "1") {
    throw new Error("Refusing to create a paid task. Set MESHY_REAL_E2E=1 only after owner approval.");
  }
  const apiKey = required("MESHY_API_KEY");
  const maxCredits = Number(required("MESHY_MAX_CREDITS"));
  const profileId = required("MESHY_REAL_E2E_PROFILE");
  const prompt = required("MESHY_REAL_E2E_PROMPT");
  const geometryTarget = process.env.MESHY_REAL_E2E_GEOMETRY_TARGET ?? "AURORA_PROOF";
  const outputPath = process.env.MESHY_REAL_E2E_OUTPUT_PATH;
  if (!Number.isFinite(maxCredits) || maxCredits <= 0) throw new Error("MESHY_MAX_CREDITS must be a positive number.");
  if (!profiles.has(profileId)) throw new Error("MESHY_REAL_E2E_PROFILE must be H1-humanoid-animated/v1, N1-quadruped/v1, or S1-static-prop/v1.");
  if (!geometryTargets.has(geometryTarget)) throw new Error("MESHY_REAL_E2E_GEOMETRY_TARGET must be AURORA_PROOF, LOWER_DETAIL, BALANCED, or HIGHER_DETAIL.");

  const localOrigin = "http://127.0.0.1";
  const bridge = createLocalBridge({
    apiKey,
    pairingCode: process.env.MESHY_BRIDGE_PAIRING_CODE,
    allowedOrigin: localOrigin,
  });
  const bridgeOrigin = await bridge.listen(0);
  const request = (path, options = {}) => fetch(`${bridgeOrigin}${path}`, {
    ...options,
    headers: { Origin: localOrigin, ...(options.headers ?? {}) },
  });

  try {
    const pairingResponse = await request("/v1/pair", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pairingCode: bridge.pairingCode }),
    });
    if (!pairingResponse.ok) throw new Error("Local Bridge pairing failed.");
    const pairing = await pairingResponse.json();
    const sessionHeaders = { "X-Meshy-Session": pairing.sessionToken, "Content-Type": "application/json" };
    const balanceResponse = await request("/v1/balance", { headers: sessionHeaders });
    if (!balanceResponse.ok) throw new Error("Could not retrieve Meshy balance.");
    const balance = await balanceResponse.json();
    const preflight = profileId.startsWith("H1")
      ? { h1Preflight: { standardHumanoid: true, clearLimbs: true, noWeapon: true } }
      : {};
    const previewResponse = await request("/v1/runs/preview", {
      method: "POST", headers: sessionHeaders,
      body: JSON.stringify({ profileId, prompt, geometryTarget, ...preflight }),
    });
    if (!previewResponse.ok) throw new Error("Local Bridge rejected the E2E preview request.");
    const preview = await previewResponse.json();
    if (preview.maximumCredits > maxCredits || balance.availableCredits < preview.maximumCredits) {
      throw new Error(`Refusing paid run: maximum ${preview.maximumCredits} credits, balance ${balance.availableCredits}, owner cap ${maxCredits}.`);
    }
    const runResponse = await request("/v1/runs", {
      method: "POST", headers: sessionHeaders,
      body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: crypto.randomUUID() }),
    });
    if (!runResponse.ok) throw new Error(`Meshy run was not created: ${await runResponse.text()}`);
    const initialRun = await runResponse.json();
    let run = initialRun;
    while (!terminal(run.status)) {
      await new Promise((resolve) => setTimeout(resolve, 3_000));
      const statusResponse = await request(`/v1/runs/${encodeURIComponent(run.id)}`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
      if (!statusResponse.ok) throw new Error("Could not refresh the Meshy E2E run.");
      run = await statusResponse.json();
    }
    if (run.status !== "READY") throw new Error(`Meshy E2E ended with ${run.status}.`);
    const provenanceResponse = await request(`/v1/runs/${encodeURIComponent(run.id)}/provenance`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
    const artifactResponse = await request(`/v1/runs/${encodeURIComponent(run.id)}/artifact`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
    if (!provenanceResponse.ok || !artifactResponse.ok) throw new Error("Meshy E2E did not yield a verified GLB.");
    const provenance = await provenanceResponse.json();
    const artifact = new Uint8Array(await artifactResponse.arrayBuffer());
    const triangles = glbTriangles(artifact);
    if (outputPath) {
      const absoluteOutputPath = resolve(outputPath);
      await mkdir(dirname(absoluteOutputPath), { recursive: true });
      await writeFile(absoluteOutputPath, artifact, { flag: "wx" });
    }
    process.stdout.write(`${JSON.stringify({ profileId, geometryTarget, runId: run.id, taskIds: provenance.taskIds, sha256: provenance.sha256, byteLength: artifact.byteLength, triangles, ...(outputPath ? { savedTo: resolve(outputPath) } : {}) }, null, 2)}\n`);
  } finally {
    await bridge.close();
  }
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
