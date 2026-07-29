import { createHash } from "node:crypto";
import { access, mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

import {
  calculateRemeshRecoveryCreditGate,
  parseRemeshRecoveryOptions,
} from "./remesh-rig-animation-recovery-options.mjs";

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
  let actions;
  try {
    actions = JSON.parse(value);
  } catch {
    throw new Error("MESHY_RECOVERY_ACTIONS must be valid JSON.");
  }
  if (!Array.isArray(actions) || actions.length < 1 || actions.length > 10) {
    throw new Error("MESHY_RECOVERY_ACTIONS must contain one to ten actions.");
  }
  for (const action of actions) {
    if (!Number.isInteger(action?.actionId) || action.actionId < 0) {
      throw new Error("Every recovery actionId must be non-negative.");
    }
    if (
      typeof action.fileStem !== "string"
      || !/^[a-z0-9][a-z0-9_-]{0,47}$/.test(action.fileStem)
    ) {
      throw new Error("Every recovery fileStem must be filesystem-safe.");
    }
    if (
      typeof action.clipName !== "string"
      || !/^[a-z][a-z0-9_]{0,15}$/i.test(action.clipName)
    ) {
      throw new Error("Every recovery clipName must be resref-safe.");
    }
  }
  if (
    new Set(actions.map((action) => action.actionId)).size !== actions.length
    || new Set(actions.map((action) => action.fileStem)).size !== actions.length
  ) {
    throw new Error("Recovery actionIds and fileStems must be unique.");
  }
  return actions;
}

async function main() {
  if (process.env.MESHY_REAL_E2E !== "1") {
    throw new Error(
      "Refusing paid recovery. Set MESHY_REAL_E2E=1 only after owner approval.",
    );
  }
  const apiKey = required("MESHY_API_KEY");
  const modelTaskId = required("MESHY_MODEL_TASK_ID");
  const expectedModelCreatedAt = Number(required("MESHY_EXPECTED_MODEL_CREATED_AT"));
  const outputDirectory = resolve(required("MESHY_REAL_E2E_OUTPUT_DIR"));
  const actions = parseActions(required("MESHY_RECOVERY_ACTIONS"));
  const options = parseRemeshRecoveryOptions({
    targetPolycount: required("MESHY_REMESH_TARGET_POLYCOUNT"),
    rigHeightMeters: required("MESHY_REAL_E2E_RIG_HEIGHT_METERS"),
    actionCount: actions.length,
  });
  const ownerCap = Number(required("MESHY_MAX_CREDITS"));
  const alreadySpentCredits = Number(required("MESHY_ALREADY_SPENT_CREDITS"));
  const creditGate = calculateRemeshRecoveryCreditGate({
    ownerCap,
    alreadySpentCredits,
    actionCount: actions.length,
  });
  if (!Number.isInteger(expectedModelCreatedAt) || expectedModelCreatedAt < 1) {
    throw new Error("MESHY_EXPECTED_MODEL_CREATED_AT must be a positive timestamp.");
  }

  const outputPaths = actions.map(
    (action) => resolve(outputDirectory, `${action.fileStem}.glb`),
  );
  const provenancePath = resolve(outputDirectory, "meshy-run-provenance.json");
  for (const path of [...outputPaths, provenancePath]) await requireAbsent(path);

  const apiJson = async (path, requestOptions = {}) => {
    const response = await fetch(`${API_ORIGIN}${path}`, {
      ...requestOptions,
      headers: {
        Authorization: `Bearer ${apiKey}`,
        ...(requestOptions.body ? { "Content-Type": "application/json" } : {}),
      },
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(
        `Meshy API rejected ${path} with ${response.status}: ${payload.message ?? "no message"}`,
      );
    }
    return payload;
  };

  const waitForTask = async (endpoint, taskId, label) => {
    for (let attempt = 0; attempt < 240; attempt += 1) {
      const task = await apiJson(`${endpoint}/${encodeURIComponent(taskId)}`);
      if (task.status === "SUCCEEDED") return task;
      if (["FAILED", "CANCELED"].includes(task.status)) {
        throw new Error(
          `${label} task ${taskId} ended with ${task.status}: ${task.task_error?.message ?? "no message"}`,
        );
      }
      await sleep(3_000);
    }
    throw new Error(`${label} task ${taskId} timed out.`);
  };

  const sourceTask = await apiJson(
    `/openapi/v1/image-to-3d/${encodeURIComponent(modelTaskId)}`,
  );
  if (
    sourceTask.status !== "SUCCEEDED"
    || sourceTask.type !== "image-to-3d"
    || sourceTask.created_at !== expectedModelCreatedAt
    || typeof sourceTask.model_urls?.glb !== "string"
  ) {
    throw new Error("The exact Image-to-3D source task identity does not match.");
  }
  const balance = await apiJson("/openapi/v1/balance");
  if (Number(balance.balance) < creditGate.maximumAdditionalCredits) {
    throw new Error("Available Meshy balance is below the recovery maximum.");
  }

  const remeshCreated = await apiJson("/openapi/v1/remesh", {
    method: "POST",
    body: JSON.stringify({
      input_task_id: modelTaskId,
      target_formats: ["glb"],
      topology: "triangle",
      target_polycount: options.targetPolycount,
      alpha_thumbnail: false,
    }),
  });
  const remeshTaskId = remeshCreated.result;
  process.stdout.write(`${JSON.stringify({ createdRemeshTaskId: remeshTaskId })}\n`);
  const remeshTask = await waitForTask("/openapi/v1/remesh", remeshTaskId, "Remesh");

  const rigCreated = await apiJson("/openapi/v1/rigging", {
    method: "POST",
    body: JSON.stringify({
      input_task_id: remeshTaskId,
      height_meters: options.rigHeightMeters,
    }),
  });
  const rigTaskId = rigCreated.result;
  process.stdout.write(`${JSON.stringify({ createdRigTaskId: rigTaskId })}\n`);
  const rigTask = await waitForTask("/openapi/v1/rigging", rigTaskId, "Rig");

  const animationTasks = [];
  for (const action of actions) {
    const created = await apiJson("/openapi/v1/animations", {
      method: "POST",
      body: JSON.stringify({
        rig_task_id: rigTaskId,
        action_id: action.actionId,
      }),
    });
    const taskId = created.result;
    process.stdout.write(
      `${JSON.stringify({ createdAnimationActionId: action.actionId, taskId })}\n`,
    );
    const task = await waitForTask("/openapi/v1/animations", taskId, "Animation");
    animationTasks.push({ action, task });
  }

  const downloaded = [];
  for (const { action, task } of animationTasks) {
    const assetUrl = task.result?.animation_glb_url;
    if (typeof assetUrl !== "string") {
      throw new Error(`Animation task ${task.id} has no GLB URL.`);
    }
    const response = await fetch(assetUrl);
    if (!response.ok) {
      throw new Error(`Could not download animation task ${task.id}.`);
    }
    const bytes = new Uint8Array(await response.arrayBuffer());
    if (
      bytes.byteLength < 4
      || bytes[0] !== 0x67
      || bytes[1] !== 0x6c
      || bytes[2] !== 0x54
      || bytes[3] !== 0x46
    ) {
      throw new Error(`Animation task ${task.id} returned invalid GLB bytes.`);
    }
    downloaded.push({
      ...action,
      taskId: task.id,
      bytes,
      sha256: sha256(bytes),
      byteLength: bytes.byteLength,
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
      reason: "image_to_3d_exceeded_meshy_rig_300000_face_limit",
      exactModelTaskId: modelTaskId,
      exactModelCreatedAt: expectedModelCreatedAt,
      exactRemeshTaskId: remeshTaskId,
      exactRigTaskId: rigTaskId,
      reusedExactGeneratedModel: true,
      generatedSecondModel: false,
    },
    options: {
      originalRequestedPolycount: 300_000,
      observedOriginalFaceCount: 300_844,
      remeshTargetPolycount: options.targetPolycount,
      topology: "triangle",
      rigHeightMeters: options.rigHeightMeters,
    },
    credits: {
      ownerCap,
      spentBeforeRecovery: alreadySpentCredits,
      ...creditGate,
      balanceBeforeRecovery: Number(balance.balance),
      remeshConsumedCredits: remeshTask.consumed_credits,
      rigConsumedCredits: rigTask.consumed_credits,
      animationConsumedCredits: animationTasks.reduce(
        (sum, item) => sum + Number(item.task.consumed_credits ?? 0),
        0,
      ),
    },
    taskIds: {
      IMAGE_TO_3D: modelTaskId,
      REMESH: remeshTaskId,
      RIG: rigTaskId,
      ...Object.fromEntries(
        downloaded.map((animation) => [
          `ANIMATE_${animation.actionId}`,
          animation.taskId,
        ]),
      ),
    },
    animations: downloaded.map(
      ({ actionId, clipName, fileStem, taskId, sha256: digest, byteLength }) => ({
        actionId,
        clipName,
        file: `${fileStem}.glb`,
        taskId,
        sha256: digest,
        byteLength,
      }),
    ),
  };
  await writeFile(
    provenancePath,
    `${JSON.stringify(provenance, null, 2)}\n`,
    { flag: "wx" },
  );
  process.stdout.write(
    `${JSON.stringify({
      outputDirectory,
      provenancePath,
      taskIds: provenance.taskIds,
      credits: provenance.credits,
      animations: provenance.animations,
    }, null, 2)}\n`,
  );
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
