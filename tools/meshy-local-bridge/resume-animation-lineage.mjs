import { createHash } from "node:crypto";
import { access, mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const API_ORIGIN = "https://api.meshy.ai";

function required(name) {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required.`);
  return value;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function sleep(milliseconds) {
  return new Promise((resolvePromise) => setTimeout(resolvePromise, milliseconds));
}

async function requireAbsent(path) {
  try {
    await access(path);
    throw new Error(`Refusing to overwrite existing output ${path}`);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
}

function parseActions(value) {
  const actions = JSON.parse(value);
  if (!Array.isArray(actions) || actions.length < 1 || actions.length > 10) throw new Error("MESHY_RESUME_ACTIONS must contain one to ten actions.");
  for (const action of actions) {
    if (!Number.isInteger(action?.actionId) || action.actionId < 0) throw new Error("Every resume actionId must be non-negative.");
    if (typeof action.fileStem !== "string" || !/^[a-z0-9][a-z0-9_-]{0,47}$/.test(action.fileStem)) throw new Error("Every resume fileStem must be filesystem-safe.");
    if (typeof action.clipName !== "string" || !/^[a-z][a-z0-9_]{0,15}$/i.test(action.clipName)) throw new Error("Every resume clipName must be resref-safe.");
    if (action.taskId !== undefined && (typeof action.taskId !== "string" || !/^[a-z0-9-]{1,128}$/i.test(action.taskId))) throw new Error("Existing taskId is invalid.");
    if (action.expectedFileName !== undefined && (typeof action.expectedFileName !== "string" || action.expectedFileName.includes("/") || action.expectedFileName.includes("\\"))) throw new Error("expectedFileName must be one filename.");
  }
  if (new Set(actions.map((action) => action.actionId)).size !== actions.length) throw new Error("Resume actionIds must be unique.");
  return actions;
}

async function main() {
  if (process.env.MESHY_REAL_E2E !== "1") throw new Error("Refusing paid recovery. Set MESHY_REAL_E2E=1 only after owner approval.");
  const apiKey = required("MESHY_API_KEY");
  const ownerCap = Number(required("MESHY_MAX_CREDITS"));
  const alreadySpentCredits = Number(required("MESHY_ALREADY_SPENT_CREDITS"));
  const rigTaskId = required("MESHY_RIG_TASK_ID");
  const modelTaskId = required("MESHY_MODEL_TASK_ID");
  const outputDirectory = resolve(required("MESHY_REAL_E2E_OUTPUT_DIR"));
  const actions = parseActions(required("MESHY_RESUME_ACTIONS"));
  if (!Number.isFinite(ownerCap) || !Number.isFinite(alreadySpentCredits) || ownerCap <= 0 || alreadySpentCredits < 0) throw new Error("Credit caps must be finite non-negative values.");
  const missingActions = actions.filter((action) => !action.taskId);
  const maximumAdditionalCredits = missingActions.length * 3;
  if (alreadySpentCredits + maximumAdditionalCredits > ownerCap) throw new Error("Refusing recovery because its worst-case total exceeds the owner credit cap.");
  const outputPaths = actions.map((action) => resolve(outputDirectory, `${action.fileStem}.glb`));
  const provenancePath = resolve(outputDirectory, "meshy-run-provenance.json");
  for (const path of [...outputPaths, provenancePath]) await requireAbsent(path);

  const apiJson = async (path, options = {}) => {
    const response = await fetch(`${API_ORIGIN}${path}`, {
      ...options,
      headers: { Authorization: `Bearer ${apiKey}`, ...(options.body ? { "Content-Type": "application/json" } : {}) },
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(`Meshy API rejected ${path} with ${response.status}: ${payload.message ?? "no message"}`);
    return payload;
  };
  const rig = await apiJson(`/openapi/v1/rigging/${encodeURIComponent(rigTaskId)}`);
  if (rig.status !== "SUCCEEDED") throw new Error("The exact recovery rig task is not SUCCEEDED.");
  const balance = await apiJson("/openapi/v1/balance");
  if (Number(balance.balance) < maximumAdditionalCredits) throw new Error("Available balance is below the recovery maximum.");

  const waitForAnimation = async (taskId) => {
    for (let attempt = 0; attempt < 180; attempt += 1) {
      const task = await apiJson(`/openapi/v1/animations/${encodeURIComponent(taskId)}`);
      if (task.status === "SUCCEEDED") return task;
      if (["FAILED", "CANCELED"].includes(task.status)) throw new Error(`Animation task ${taskId} ended with ${task.status}.`);
      await sleep(3_000);
    }
    throw new Error(`Animation task ${taskId} timed out.`);
  };
  const downloadAnimation = async (task, action) => {
    const assetUrl = task.result?.animation_glb_url;
    if (typeof assetUrl !== "string") throw new Error(`Animation task ${task.id} has no GLB URL.`);
    const fileName = decodeURIComponent(new URL(assetUrl).pathname.split("/").at(-1));
    if (action.expectedFileName && fileName !== action.expectedFileName) {
      throw new Error(`Animation task ${task.id} returned ${fileName}, expected ${action.expectedFileName}.`);
    }
    for (let attempt = 1; attempt <= 5; attempt += 1) {
      const response = await fetch(assetUrl);
      if (response.ok) {
        const bytes = new Uint8Array(await response.arrayBuffer());
        if (bytes.byteLength < 4 || bytes[0] !== 0x67 || bytes[1] !== 0x6c || bytes[2] !== 0x54 || bytes[3] !== 0x46) {
          throw new Error(`Animation task ${task.id} returned invalid GLB bytes.`);
        }
        return { bytes, sourceFileName: fileName };
      }
      if (attempt < 5) await sleep(attempt * 1_000);
    }
    throw new Error(`Could not download the exact animation task ${task.id} after five attempts.`);
  };

  const downloaded = [];
  for (const action of actions) {
    let taskId = action.taskId;
    if (!taskId) {
      const created = await apiJson("/openapi/v1/animations", {
        method: "POST",
        body: JSON.stringify({ rig_task_id: rigTaskId, action_id: action.actionId }),
      });
      taskId = created.result;
      process.stdout.write(`${JSON.stringify({ createdAnimationActionId: action.actionId, taskId })}\n`);
    }
    const task = await waitForAnimation(taskId);
    const artifact = await downloadAnimation(task, action);
    downloaded.push({
      ...action,
      taskId,
      sourceFileName: artifact.sourceFileName,
      bytes: artifact.bytes,
      sha256: sha256(artifact.bytes),
      byteLength: artifact.bytes.byteLength,
    });
  }

  await mkdir(outputDirectory, { recursive: true });
  for (let index = 0; index < downloaded.length; index += 1) {
    await writeFile(outputPaths[index], downloaded[index].bytes, { flag: "wx" });
  }
  const provenance = {
    schemaVersion: 1,
    profileId: "H1-humanoid-animated/v1",
    recovery: {
      reason: "signed_animation_glb_download_failed",
      exactModelTaskId: modelTaskId,
      exactRigTaskId: rigTaskId,
      reusedExactLineage: true,
      generatedSecondModel: false,
    },
    credits: {
      ownerCap,
      spentBeforeRecovery: alreadySpentCredits,
      maximumAdditionalCredits,
      expectedTotal: alreadySpentCredits + maximumAdditionalCredits,
      balanceBeforeRecovery: Number(balance.balance),
    },
    animations: downloaded.map(({ actionId, clipName, fileStem, taskId, sourceFileName, sha256: digest, byteLength }) => ({
      actionId, clipName, file: `${fileStem}.glb`, taskId, sourceFileName, sha256: digest, byteLength,
    })),
  };
  await writeFile(provenancePath, `${JSON.stringify(provenance, null, 2)}\n`, { flag: "wx" });
  process.stdout.write(`${JSON.stringify({ outputDirectory, provenancePath, credits: provenance.credits, animations: provenance.animations }, null, 2)}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
