import { useMemo, useState, type KeyboardEvent } from "react";
import { filterCustomAnimationPickerV1 } from "./editing";
import type { ProjectedCustomAnimationLibraryItemV1 } from "./editing";
import { CustomAnimationPickerRow } from "./CustomAnimationPickerRow";

export function CustomAnimationPicker({
  items,
  selectedId,
  onSelect,
  onCreate,
  onOpenSelected,
}: {
  items: readonly ProjectedCustomAnimationLibraryItemV1[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onCreate: () => void;
  onOpenSelected: (id: string) => void;
}) {
  const [query, setQuery] = useState("");
  const visible = useMemo(
    () => filterCustomAnimationPickerV1(items, query),
    [items, query],
  );
  const visibleHasSelection = visible.some(({ id }) => id === selectedId);
  const selected = items.find(({ id }) => id === selectedId) ?? null;
  return (
    <section className="custom-animation-picker" aria-labelledby="custom-picker-title">
      <h3 id="custom-picker-title">Custom animation</h3>
      <input
        type="search"
        aria-label="Search custom animations"
        placeholder="Search custom animations…"
        value={query}
        onChange={(event) => setQuery(event.currentTarget.value)}
      />
      <div role="listbox" aria-label="Saved Custom animations">
        {visible.map((item, index) => (
          <CustomAnimationPickerRow
            key={item.id}
            item={item}
            selected={selectedId === item.id}
            onSelect={() => onSelect(item.id)}
            tabIndex={item.id === selectedId || (!visibleHasSelection && index === 0)
              ? 0
              : -1}
            onKeyDown={moveCustomOptionFocus}
          />
        ))}
      </div>
      <footer>
        <button type="button" onClick={onCreate}>+ Create new animation</button>
        <button
          type="button"
          disabled={!selected?.openableInEditor}
          title={selected && !selected.openableInEditor
            ? "Preview requires an authored clip. Create an editable copy first."
            : undefined}
          onClick={() => selectedId && onOpenSelected(selectedId)}
        >
          Preview selected Custom
        </button>
        <button
          type="button"
          disabled={!selected?.openableInEditor}
          title={selected && !selected.openableInEditor
            ? "This Custom item references a read-only source clip. Use Edit copy to create an authored clip."
            : undefined}
          onClick={() => selectedId && onOpenSelected(selectedId)}
        >
          Open selected in editor
        </button>
      </footer>
      {selected ? (
        <section
          className="custom-animation-picker__preview"
          aria-label="Selected Custom preview"
        >
          <strong>{selected.name}</strong>
          <span>{selected.durationSeconds.toFixed(2)} s · {selected.status}</span>
          <span>
            {selected.openableInEditor
              ? "Preview opens this exact authored clip without assigning it."
              : "Create an editable copy to preview this read-only source realization."}
          </span>
        </section>
      ) : null}
    </section>
  );
}

function moveCustomOptionFocus(event: KeyboardEvent<HTMLButtonElement>) {
  if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
  const options = Array.from(
    event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>(
      ':scope > [role="option"]',
    ) ?? [],
  );
  const current = options.indexOf(event.currentTarget);
  if (current < 0 || options.length === 0) return;
  event.preventDefault();
  const next = event.key === "Home"
    ? 0
    : event.key === "End"
      ? options.length - 1
      : event.key === "ArrowDown"
        ? (current + 1) % options.length
        : (current - 1 + options.length) % options.length;
  options[next]?.focus();
  // Disabled entries remain focusable so their visible reason can be read.
  if (options[next]?.getAttribute("aria-disabled") !== "true") {
    options[next]?.click();
  }
}
