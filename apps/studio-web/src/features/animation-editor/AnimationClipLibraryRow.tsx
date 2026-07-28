import type { KeyboardEventHandler } from "react";
import { AnimationClipStatusBadge } from "./AnimationClipStatusBadge";
import type { AnimationStudioLibraryItemV1 } from "./editing";

export function AnimationClipLibraryRow({
  item,
  selected,
  onSelect,
  firstRepairAction,
  tabIndex,
  onKeyDown,
}: {
  item: AnimationStudioLibraryItemV1;
  selected: boolean;
  onSelect: () => void;
  firstRepairAction: string | null;
  tabIndex: number;
  onKeyDown: KeyboardEventHandler<HTMLButtonElement>;
}) {
  return (
    <button
      type="button"
      role="option"
      aria-selected={selected}
      tabIndex={tabIndex}
      onClick={onSelect}
      onKeyDown={onKeyDown}
    >
      <span>
        <strong>{item.name}</strong>
        <small>
          {item.durationSeconds.toFixed(2)} s · {item.keyframeCount} keys
        </small>
      </span>
      <AnimationClipStatusBadge origin={item.origin} status={item.status} />
      {item.status === "INVALID" ? (
        <span className="animation-clip-repair-action">
          <strong>First fix:</strong>{" "}
          {firstRepairAction
            ?? "Add at least one valid LINEAR transform key, then save again."}
        </span>
      ) : null}
    </button>
  );
}
