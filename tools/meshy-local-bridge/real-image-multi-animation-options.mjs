export function parseModerationFlag(value) {
  if (value === undefined || value === "1") return true;
  if (value === "0") return false;
  throw new Error("MESHY_REAL_E2E_MODERATION must be 0 or 1.");
}
