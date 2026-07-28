export function AnimationKeyframeMarker({
  id,
  timeSeconds,
  selected,
  leftPercent,
  boneName,
  path,
  onSelect,
}: {
  id: string;
  timeSeconds: number;
  selected: boolean;
  leftPercent: number;
  boneName: string;
  path: string;
  onSelect: (id: string, additive: boolean) => void;
}) {
  return (
    <button
      type="button"
      className="animation-keyframe-marker"
      data-selected={selected}
      aria-pressed={selected}
      style={{ left: `${leftPercent}%` }}
      aria-label={`${boneName} ${path} key at ${timeSeconds.toFixed(3)} seconds${selected ? ", selected" : ""}`}
      onClick={(event) => onSelect(id, event.shiftKey || event.ctrlKey || event.metaKey)}
    >
      ◆
    </button>
  );
}
