import type { AnimationStudioLibraryItemV1 } from "./editing";

export function AnimationClipStatusBadge({
  origin,
  status,
}: Pick<AnimationStudioLibraryItemV1, "origin" | "status">) {
  const originLabel = origin === "SOURCE"
    ? "Source"
    : origin === "GENERATED"
      ? "Generated"
      : "Edited";
  const statusLabel = status === "SOURCE"
    ? null
    : status[0] + status.slice(1).toLocaleLowerCase("en-US");
  return (
    <span
      className="animation-clip-status"
      data-status={status}
      role="status"
      aria-label={`Animation origin: ${originLabel}${
        statusLabel ? `; status: ${statusLabel}` : ""
      }`}
    >
      <span>{originLabel}</span>
      {statusLabel ? <span>{statusLabel}</span> : null}
    </span>
  );
}
