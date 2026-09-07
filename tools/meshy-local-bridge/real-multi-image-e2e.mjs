import { createHash, randomUUID } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { createLocalBridge } from "./index.mjs";
import { inspectGlbTriangleCount } from "./merge-animation-glbs.mjs";
import { AURORA_MODEL_TRIANGLE_BUDGET_V1 } from "./remesh-rig-animation-recovery-options.mjs";

const PROFILES = new Set(["N1-quadruped/v1", "S1-static-prop/v1"]);
const TERMINAL_STATUSES = new Set(["READY", "FAILED", "CANCELED"]);

function required(name) {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required.`);
  return value;
}

function exactImageDataUrl(bytes, path) {
  const png = bytes.length >= 8
    && bytes.subarray(0, 8).equals(Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]));
  const jpeg = bytes.length >= 3 && bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff;
  if (!png && !jpeg) throw new Error(`Multi-image input is not an exact PNG or JPEG: ${path}`);
  return `data:image/${png ? "png" : "jpeg"};base64,${bytes.toString("base64")}`;
}

function parseImagePaths() {
  let values;
  try {
    values = JSON.parse(required("MESHY_REAL_E2E_IMAGE_PATHS_JSON"));
  } catch (error) {
    throw new Error(`MESHY_REAL_E2E_IMAGE_PATHS_JSON must be a JSON array: ${error.message}`);
  }
  if (!Array.isArray(values) || values.length < 1 || values.length > 4 || values.some((value) => typeof value !== "string" || !value)) {
    throw new Error("MESHY_REAL_E2E_IMAGE_PATHS_JSON must contain one to four paths.");
  }
  return values.map((value) => resolve(value));
}

async function main() {
  if (process.env.MESHY_REAL_E2E !== "1") {
    throw new Error("Refusing to create a paid task. Set MESHY_REAL_E2E=1 only after owner approval.");
  }

  const apiKey = required("MESHY_API_KEY");
  const profileId = process.env.MESHY_REAL_E2E_PROFILE ?? "N1-quadruped/v1";
  const maxCredits = Number(required("MESHY_MAX_CREDITS"));
  const targetPolycount = Number(process.env.MESHY_REAL_E2E_TARGET_POLYCOUNT ?? "60000");
  const imagePaths = parseImagePaths();
  const outputPath = resolve(required("MESHY_REAL_E2E_OUTPUT_PATH"));
  const provenancePath = resolve(required("MESHY_REAL_E2E_PROVENANCE_PATH"));
  const prompt = process.env.MESHY_REAL_E2E_PROMPT ?? "";
  const ownerAuthorization = process.env.MESHY_OWNER_AUTHORIZATION
    ?? "Direct owner instruction on 2026-08-20 to use Meshy and prepare a c_wolf-derived dog model and NWN demo";

  if (!PROFILES.has(profileId)) throw new Error("The multi-image runner supports N1-quadruped/v1 or S1-static-prop/v1.");
  if (!Number.isFinite(maxCredits) || maxCredits <= 0) throw new Error("MESHY_MAX_CREDITS must be positive.");
  if (!Number.isInteger(targetPolycount) || targetPolycount < 100 || targetPolycount > AURORA_MODEL_TRIANGLE_BUDGET_V1) {
    throw new Error(`Target polycount must be an integer in 100..=${AURORA_MODEL_TRIANGLE_BUDGET_V1}.`);
  }

  const inputs = await Promise.all(imagePaths.map(async (path) => {
    const bytes = await readFile(path);
    return {
      path,
      name: basename(path),
      bytes,
      sizeBytes: bytes.byteLength,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      dataUrl: exactImageDataUrl(bytes, path),
    };
  }));

  const apiOptions = {
    modelType: "standard",
    aiModel: "meshy-6",
    shouldRemesh: true,
    topology: "triangle",
    targetPolycount,
    poseMode: "",
    moderation: true,
    targetFormats: ["glb"],
    alphaThumbnail: false,
    autoSize: false,
    originAt: "bottom",
    enablePbr: false,
    shouldTexture: true,
    hdTexture: false,
    texturePrompt: prompt,
    textureImageUrl: "",
    removeLighting: true,
    imageEnhancement: false,
    multiViewThumbnails: false,
    rigHumanoid: false,
    rigHeightMeters: 1.7,
  };

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

    const balanceBeforeResponse = await request("/v1/balance", { headers: sessionHeaders });
    if (!balanceBeforeResponse.ok) throw new Error("Could not retrieve the Meshy balance.");
    const balanceBefore = await balanceBeforeResponse.json();

    const previewResponse = await request("/v1/runs/preview", {
      method: "POST",
      headers: sessionHeaders,
      body: JSON.stringify({
        profileId,
        prompt,
        source: "MULTI_IMAGE",
        imageDataUrls: inputs.map(({ dataUrl }) => dataUrl),
        geometryTarget: "HIGHER_DETAIL",
        apiOptions,
      }),
    });
    if (!previewResponse.ok) throw new Error(`Local Bridge rejected the preview: ${await previewResponse.text()}`);
    const preview = await previewResponse.json();
    if (preview.maximumCredits > maxCredits || balanceBefore.availableCredits < preview.maximumCredits) {
      throw new Error(`Refusing paid run: maximum ${preview.maximumCredits}, balance ${balanceBefore.availableCredits}, owner cap ${maxCredits}.`);
    }

    process.stdout.write(`${JSON.stringify({
      stage: "PAID_RUN_CONFIRMED",
      maximumCredits: preview.maximumCredits,
      balanceBefore: balanceBefore.availableCredits,
      targetPolycount,
      inputs: inputs.map(({ name, sizeBytes, sha256 }) => ({ name, sizeBytes, sha256 })),
    })}\n`);

    const runResponse = await request("/v1/runs", {
      method: "POST",
      headers: sessionHeaders,
      body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: randomUUID() }),
    });
    if (!runResponse.ok) throw new Error(`Meshy run was not created: ${await runResponse.text()}`);
    let run = await runResponse.json();
    const deadline = Date.now() + 20 * 60_000;
    while (!TERMINAL_STATUSES.has(run.status)) {
      if (Date.now() >= deadline) throw new Error("Timed out while waiting for the multi-image Meshy run.");
      await new Promise((resolveTimeout) => setTimeout(resolveTimeout, 3_000));
      const statusResponse = await request(`/v1/runs/${encodeURIComponent(run.id)}`, {
        headers: { "X-Meshy-Session": pairing.sessionToken },
      });
      if (!statusResponse.ok) throw new Error("Could not refresh the Meshy run.");
      run = await statusResponse.json();
    }
    if (run.status !== "READY") throw new Error(`Meshy ended with ${run.status}: ${JSON.stringify(run.error ?? {})}`);

    const provenanceResponse = await request(`/v1/runs/${encodeURIComponent(run.id)}/provenance`, {
      headers: { "X-Meshy-Session": pairing.sessionToken },
    });
    const artifactResponse = await request(`/v1/runs/${encodeURIComponent(run.id)}/artifact`, {
      headers: { "X-Meshy-Session": pairing.sessionToken },
    });
    if (!provenanceResponse.ok || !artifactResponse.ok) throw new Error("Meshy did not yield a verified GLB.");
    const bridgeProvenance = await provenanceResponse.json();
    const artifact = new Uint8Array(await artifactResponse.arrayBuffer());
    const sha256 = createHash("sha256").update(artifact).digest("hex");
    if (sha256 !== bridgeProvenance.sha256) throw new Error("Downloaded GLB hash differs from Bridge provenance.");
    const triangles = inspectGlbTriangleCount(artifact, "multi-image generated GLB");

    const balanceAfterResponse = await request("/v1/balance", { headers: sessionHeaders });
    if (!balanceAfterResponse.ok) throw new Error("Could not retrieve the post-run Meshy balance.");
    const balanceAfter = await balanceAfterResponse.json();
    const createdAt = new Date().toISOString();
    const persistedProvenance = {
      schemaVersion: 1,
      ownerAuthorization,
      profileId,
      sourceMode: "MULTI_IMAGE",
      localBridgeRunId: run.id,
      meshyTaskIds: bridgeProvenance.taskIds,
      status: run.status,
      createdAt,
      creditReview: {
        maximumCredits: preview.maximumCredits,
        balanceBefore: balanceBefore.availableCredits,
        balanceAfter: balanceAfter.availableCredits,
        spentCredits: balanceBefore.availableCredits - balanceAfter.availableCredits,
      },
      inputs: inputs.map(({ name: path, sizeBytes, sha256: inputSha256 }) => ({ path, sizeBytes, sha256: inputSha256 })),
      request: apiOptions,
      artifact: {
        path: basename(outputPath),
        sizeBytes: artifact.byteLength,
        sha256,
        measuredTriangleCount: triangles,
        manualMeshOrTextureEdits: false,
      },
      redaction: { apiKeyPersisted: false, sessionTokenPersisted: false, signedUrlsPersisted: false },
    };

    await mkdir(dirname(outputPath), { recursive: true });
    await writeFile(outputPath, artifact, { flag: "wx" });
    await writeFile(provenancePath, `${JSON.stringify(persistedProvenance, null, 2)}\n`, { flag: "wx" });
    process.stdout.write(`${JSON.stringify({
      stage: "READY",
      runId: run.id,
      taskIds: bridgeProvenance.taskIds,
      maximumCredits: preview.maximumCredits,
      balanceBefore: balanceBefore.availableCredits,
      balanceAfter: balanceAfter.availableCredits,
      spentCredits: persistedProvenance.creditReview.spentCredits,
      sha256,
      byteLength: artifact.byteLength,
      triangles,
      savedTo: outputPath,
      provenancePath,
    }, null, 2)}\n`);
  } finally {
    await bridge.close();
  }
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
