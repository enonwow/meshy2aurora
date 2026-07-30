import { createHash, randomUUID } from "node:crypto";
import { access, mkdir, readFile, writeFile } from "node:fs/promises";
import { extname, resolve } from "node:path";

import { createLocalBridge } from "./index.mjs";
import { parseModerationFlag } from "./real-image-multi-animation-options.mjs";
import { AURORA_MODEL_TRIANGLE_BUDGET_V1 } from "./remesh-rig-animation-recovery-options.mjs";

function required(name) {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required.`);
  return value;
}

function terminal(status) {
  return status === "READY" || status === "FAILED" || status === "CANCELED";
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function requireAbsent(path) {
  try {
    await access(path);
    throw new Error(`Refusing to overwrite existing output ${path}`);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
}

function parseAnimations(value) {
  let parsed;
  try {
    parsed = JSON.parse(value);
  } catch {
    throw new Error("MESHY_REAL_E2E_ANIMATIONS must be valid JSON.");
  }
  if (!Array.isArray(parsed) || parsed.length < 1 || parsed.length > 10) {
    throw new Error("MESHY_REAL_E2E_ANIMATIONS must contain one to ten actions.");
  }
  const actionIds = new Set();
  const fileStems = new Set();
  for (const animation of parsed) {
    if (!Number.isInteger(animation?.actionId) || animation.actionId < 0) throw new Error("Every animation actionId must be a non-negative integer.");
    if (typeof animation.fileStem !== "string" || !/^[a-z0-9][a-z0-9_-]{0,47}$/.test(animation.fileStem)) throw new Error("Every animation fileStem must be a lowercase filesystem-safe name.");
    if (typeof animation.clipName !== "string" || !/^[a-z][a-z0-9_]{0,15}$/i.test(animation.clipName)) throw new Error("Every animation clipName must be a 1-16 character ASCII resref-style name.");
    if (actionIds.has(animation.actionId) || fileStems.has(animation.fileStem)) throw new Error("Animation actionIds and fileStems must be unique.");
    actionIds.add(animation.actionId);
    fileStems.add(animation.fileStem);
  }
  return parsed;
}

async function main() {
  if (process.env.MESHY_REAL_E2E !== "1") throw new Error("Refusing to create a paid task. Set MESHY_REAL_E2E=1 only after owner approval.");
  const apiKey = required("MESHY_API_KEY");
  const maxCredits = Number(required("MESHY_MAX_CREDITS"));
  const sourceImagePath = resolve(required("MESHY_REAL_E2E_SOURCE_IMAGE"));
  const outputDirectory = resolve(required("MESHY_REAL_E2E_OUTPUT_DIR"));
  const animations = parseAnimations(required("MESHY_REAL_E2E_ANIMATIONS"));
  const targetPolycount = Number(process.env.MESHY_REAL_E2E_TARGET_POLYCOUNT ?? "20000");
  const rigHeightMeters = Number(process.env.MESHY_REAL_E2E_RIG_HEIGHT_METERS ?? "1.85");
  const moderation = parseModerationFlag(process.env.MESHY_REAL_E2E_MODERATION);
  if (!Number.isFinite(maxCredits) || maxCredits <= 0) throw new Error("MESHY_MAX_CREDITS must be a positive number.");
  if (
    !Number.isInteger(targetPolycount)
    || targetPolycount < 100
    || targetPolycount > AURORA_MODEL_TRIANGLE_BUDGET_V1
  ) {
    throw new Error(
      `MESHY_REAL_E2E_TARGET_POLYCOUNT must be an integer in 100..=${AURORA_MODEL_TRIANGLE_BUDGET_V1}.`,
    );
  }
  if (!Number.isFinite(rigHeightMeters) || rigHeightMeters < 0.5 || rigHeightMeters > 3) throw new Error("MESHY_REAL_E2E_RIG_HEIGHT_METERS must be in 0.5..=3.");
  const extension = extname(sourceImagePath).toLowerCase();
  const mimeType = extension === ".png" ? "image/png" : [".jpg", ".jpeg"].includes(extension) ? "image/jpeg" : undefined;
  if (!mimeType) throw new Error("MESHY_REAL_E2E_SOURCE_IMAGE must be PNG or JPEG.");
  const sourceImage = await readFile(sourceImagePath);
  const imageDataUrl = `data:${mimeType};base64,${sourceImage.toString("base64")}`;
  const outputPaths = animations.map((animation) => resolve(outputDirectory, `${animation.fileStem}.glb`));
  const combinedOutputPath = resolve(outputDirectory, "source.glb");
  const provenancePath = resolve(outputDirectory, "meshy-run-provenance.json");
  for (const path of [...outputPaths, combinedOutputPath, provenancePath]) await requireAbsent(path);

  const localOrigin = "http://127.0.0.1";
  const bridge = createLocalBridge({ apiKey, allowedOrigin: localOrigin });
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
    const apiOptions = {
      modelType: "standard",
      aiModel: "meshy-6",
      shouldRemesh: true,
      topology: "triangle",
      targetPolycount,
      poseMode: "t-pose",
      moderation,
      targetFormats: ["glb"],
      alphaThumbnail: false,
      autoSize: true,
      originAt: "bottom",
      enablePbr: true,
      shouldTexture: true,
      hdTexture: false,
      texturePrompt: "",
      textureImageUrl: "",
      removeLighting: true,
      imageEnhancement: true,
      multiViewThumbnails: false,
      rigHumanoid: true,
      rigHeightMeters,
      animationActions: animations.map(({ actionId, clipName }) => ({ actionId, clipName })),
    };
    const previewResponse = await request("/v1/runs/preview", {
      method: "POST",
      headers: sessionHeaders,
      body: JSON.stringify({
        profileId: "H1-humanoid-animated/v1",
        prompt: "",
        source: "IMAGE",
        imageDataUrls: [imageDataUrl],
        geometryTarget: "BALANCED",
        h1Preflight: { standardHumanoid: true, clearLimbs: true, noWeapon: true },
        apiOptions,
      }),
    });
    if (!previewResponse.ok) throw new Error(`Local Bridge rejected the image-to-3D preview: ${await previewResponse.text()}`);
    const preview = await previewResponse.json();
    if (preview.maximumCredits > maxCredits || balance.availableCredits < preview.maximumCredits) {
      throw new Error(`Refusing paid run: maximum ${preview.maximumCredits} credits, balance ${balance.availableCredits}, owner cap ${maxCredits}.`);
    }
    const runResponse = await request("/v1/runs", {
      method: "POST",
      headers: sessionHeaders,
      body: JSON.stringify({ previewId: preview.previewId, confirmationNonce: randomUUID() }),
    });
    if (!runResponse.ok) throw new Error(`Meshy run was not created: ${await runResponse.text()}`);
    let run = await runResponse.json();
    while (!terminal(run.status)) {
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 3_000));
      const statusResponse = await request(`/v1/runs/${encodeURIComponent(run.id)}`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
      if (!statusResponse.ok) throw new Error("Could not refresh the Meshy multi-animation run.");
      run = await statusResponse.json();
    }
    if (run.status !== "READY") throw new Error(`Meshy multi-animation run ended with ${run.status}: ${JSON.stringify(run.error ?? {})}`);
    const provenanceResponse = await request(`/v1/runs/${encodeURIComponent(run.id)}/provenance`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
    if (!provenanceResponse.ok) throw new Error("Meshy run did not yield provenance.");
    const bridgeProvenance = await provenanceResponse.json();
    if (bridgeProvenance.artifactKind !== "MERGED_ANIMATION_GLTF") {
      throw new Error("Meshy run did not yield the required merged animation GLB.");
    }
    const combinedResponse = await request(`/v1/runs/${encodeURIComponent(run.id)}/artifact`, {
      headers: { "X-Meshy-Session": pairing.sessionToken },
    });
    if (!combinedResponse.ok) throw new Error("Meshy run did not yield its merged animation GLB.");
    const combinedBytes = new Uint8Array(await combinedResponse.arrayBuffer());
    if (sha256(combinedBytes) !== bridgeProvenance.sha256 || combinedBytes.byteLength !== bridgeProvenance.byteLength) {
      throw new Error("Merged animation GLB does not match Bridge provenance.");
    }
    const downloaded = [];
    for (let index = 0; index < animations.length; index += 1) {
      const animation = animations[index];
      const response = await request(`/v1/runs/${encodeURIComponent(run.id)}/artifact/${animation.actionId}`, { headers: { "X-Meshy-Session": pairing.sessionToken } });
      if (!response.ok) throw new Error(`Meshy run did not yield action ${animation.actionId}.`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      const digest = sha256(bytes);
      const expected = bridgeProvenance.animationArtifacts?.find((artifact) => artifact.actionId === animation.actionId);
      if (!expected || expected.sha256 !== digest || expected.byteLength !== bytes.byteLength) {
        throw new Error(`Downloaded action ${animation.actionId} does not match Bridge provenance.`);
      }
      downloaded.push({ ...animation, bytes, sha256: digest, byteLength: bytes.byteLength, outputPath: outputPaths[index] });
    }
    await mkdir(outputDirectory, { recursive: true });
    for (const artifact of downloaded) await writeFile(artifact.outputPath, artifact.bytes, { flag: "wx" });
    await writeFile(combinedOutputPath, combinedBytes, { flag: "wx" });
    const durableProvenance = {
      schemaVersion: 1,
      profileId: "H1-humanoid-animated/v1",
      source: {
        kind: "IMAGE",
        path: sourceImagePath,
        sha256: sha256(sourceImage),
        byteLength: sourceImage.byteLength,
      },
      options: {
        aiModel: "meshy-6",
        targetPolycount,
        topology: "triangle",
        poseMode: "t-pose",
        rigHeightMeters,
        moderation,
      },
      credits: {
        ownerCap: maxCredits,
        maximumForRun: preview.maximumCredits,
        balanceBeforeRun: balance.availableCredits,
      },
      runId: run.id,
      taskIds: bridgeProvenance.taskIds,
      canonicalSource: {
        file: "source.glb",
        sha256: bridgeProvenance.sha256,
        byteLength: bridgeProvenance.byteLength,
        artifactKind: bridgeProvenance.artifactKind,
      },
      animations: downloaded.map(({ actionId, clipName, fileStem, sha256: digest, byteLength }) => ({
        actionId, clipName, file: `${fileStem}.glb`, sha256: digest, byteLength,
      })),
    };
    await writeFile(provenancePath, `${JSON.stringify(durableProvenance, null, 2)}\n`, { flag: "wx" });
    process.stdout.write(`${JSON.stringify({
      runId: run.id,
      maximumCredits: preview.maximumCredits,
      balanceBeforeRun: balance.availableCredits,
      outputDirectory,
      combinedOutputPath,
      provenancePath,
      animations: durableProvenance.animations,
    }, null, 2)}\n`);
  } finally {
    await bridge.close();
  }
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
