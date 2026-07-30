export const AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000;
export const MESHY_RIG_FACE_LIMIT = AURORA_MODEL_TRIANGLE_BUDGET_V1;
export const MESHY_AUTOMATIC_RECOVERY_TARGET = AURORA_MODEL_TRIANGLE_BUDGET_V1 - 5_000;

export function parseRemeshRecoveryOptions({
  targetPolycount,
  rigHeightMeters,
  actionCount,
}) {
  const parsedTarget = Number(targetPolycount);
  const parsedHeight = Number(rigHeightMeters);
  if (
    !Number.isInteger(parsedTarget)
    || parsedTarget < 100
    || parsedTarget > AURORA_MODEL_TRIANGLE_BUDGET_V1
  ) {
    throw new Error(
      `remesh targetPolycount must be an integer in 100..=${AURORA_MODEL_TRIANGLE_BUDGET_V1}`,
    );
  }
  if (!Number.isFinite(parsedHeight) || parsedHeight < 0.5 || parsedHeight > 3) {
    throw new Error("rigHeightMeters must be in 0.5..=3");
  }
  if (!Number.isInteger(actionCount) || actionCount < 1 || actionCount > 10) {
    throw new Error("actionCount must be an integer in 1..=10");
  }
  return {
    targetPolycount: parsedTarget,
    rigHeightMeters: parsedHeight,
    actionCount,
  };
}

export function calculateRemeshRecoveryCreditGate({
  ownerCap,
  alreadySpentCredits,
  actionCount,
}) {
  const remeshCredits = 5;
  const rigCredits = 5;
  const animationCredits = actionCount * 3;
  const maximumAdditionalCredits = remeshCredits + rigCredits + animationCredits;
  const maximumTotalCredits = alreadySpentCredits + maximumAdditionalCredits;
  if (
    !Number.isFinite(ownerCap)
    || !Number.isFinite(alreadySpentCredits)
    || ownerCap <= 0
    || alreadySpentCredits < 0
  ) {
    throw new Error("credit values must be finite and non-negative");
  }
  if (maximumTotalCredits > ownerCap) {
    throw new Error(
      `refusing recovery because maximum ${maximumTotalCredits} exceeds the owner credit cap ${ownerCap}`,
    );
  }
  return {
    remeshCredits,
    rigCredits,
    animationCredits,
    maximumAdditionalCredits,
    maximumTotalCredits,
  };
}

export function planAutomaticRigFaceRecovery({
  observedFaceCount,
  requestedTargetPolycount,
}) {
  if (!Number.isInteger(observedFaceCount) || observedFaceCount < 0) {
    throw new Error("observedFaceCount must be a non-negative integer");
  }
  if (
    !Number.isInteger(requestedTargetPolycount)
    || requestedTargetPolycount < 100
    || requestedTargetPolycount > MESHY_RIG_FACE_LIMIT
  ) {
    throw new Error(
      `requestedTargetPolycount must be an integer in 100..=${AURORA_MODEL_TRIANGLE_BUDGET_V1}`,
    );
  }
  return observedFaceCount <= MESHY_RIG_FACE_LIMIT
    ? { required: false, observedFaceCount }
    : {
        required: true,
        observedFaceCount,
        targetPolycount: Math.min(requestedTargetPolycount, MESHY_AUTOMATIC_RECOVERY_TARGET),
        reason: "generated_model_exceeds_meshy_rig_face_limit",
      };
}
