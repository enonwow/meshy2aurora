import type { KeyboardEventHandler } from "react";
import type { ProjectedCustomAnimationLibraryItemV1 } from "./editing";

export function CustomAnimationPickerRow({
  item,
  selected,
  onSelect,
  tabIndex,
  onKeyDown,
}: {
  item: ProjectedCustomAnimationLibraryItemV1;
  selected: boolean;
  onSelect: () => void;
  tabIndex: number;
  onKeyDown: KeyboardEventHandler<HTMLButtonElement>;
}) {
  return (
    <button
      type="button"
      role="option"
      aria-selected={selected}
      aria-disabled={!item.assignable}
      tabIndex={tabIndex}
      aria-label={[
        item.name,
        item.status,
        `${item.durationSeconds.toFixed(2)} seconds`,
        item.sourceClipCount > 0
          ? `${item.sourceClipCount} read-only source clips`
          : `${item.keyframeCount} keyframes`,
        item.playback === "LOOPING_PHASED" ? "Start Loop End phases" : "One shot",
        `provider ${item.provenance.provider}`,
        `asset ${item.provenance.assetId}`,
        `ownership ${item.provenance.ownership}`,
      ].join(", ")}
      data-status={item.status}
      onClick={() => item.assignable && onSelect()}
      onKeyDown={onKeyDown}
    >
      <span aria-hidden="true">{selected ? "◉" : "○"}</span>
      <strong>{item.name}</strong>
      <span>{item.status[0] + item.status.slice(1).toLocaleLowerCase("en-US")}</span>
      <small>
        {item.playback === "LOOPING_PHASED"
          ? "Start / Loop / End"
          : "One shot"}
        {" · "}
        {item.durationSeconds.toFixed(2)} s
        {" · "}
        {item.sourceClipCount > 0
          ? `${item.sourceClipCount} source clip${item.sourceClipCount === 1 ? "" : "s"}`
          : `${item.keyframeCount} keys`}
      </small>
      {item.assignedSlots.length > 0 ? (
        <small>Used by Base 42: {item.assignedSlots.join(", ")}</small>
      ) : null}
      <small className="custom-animation-picker__provenance">
        Provider: {item.provenance.provider}
        {" · "}Asset: {item.provenance.assetId}
        {" · "}Ownership: {item.provenance.ownership}
      </small>
      {item.lineage.map(({ reference, authoredSource }, index) => (
        <small
          className="custom-animation-picker__lineage"
          key={`${reference.sourceKind}:${reference.authoredClipId ?? reference.sourceClipName}:${index}`}
        >
          Lineage {index + 1}: {reference.sourceKind}
          {reference.sourceClipName ? ` · source ${reference.sourceClipName}` : ""}
          {reference.authoredClipId ? ` · clip ${reference.authoredClipId}` : ""}
          {authoredSource
            ? ` · ${authoredSource.kind} · revision ${authoredSource.sourceRevision.slice(0, 12)}${
                authoredSource.sourceClipFingerprint
                  ? ` · fingerprint ${authoredSource.sourceClipFingerprint.slice(0, 12)}`
                  : ""
              }`
            : ""}
        </small>
      ))}
      {!item.assignable ? <span>Finish editing to assign</span> : null}
    </button>
  );
}
