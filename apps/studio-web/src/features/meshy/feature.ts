export function isMeshyLabEnabled(value = import.meta.env.VITE_MESHY_LAB): boolean {
  return value === "1";
}
