import { useMemo, useState, type KeyboardEvent } from "react";
import { AnimationClipLibraryRow } from "./AnimationClipLibraryRow";
import {
  filterAnimationStudioLibraryV1,
  type AnimationStudioLibraryItemV1,
} from "./editing";
import { NewAnimationMenu } from "./NewAnimationMenu";

export function AnimationClipLibrary({
  items,
  selectedId,
  onSelect,
  onCreateBlank,
  onCreateProcedural,
  onImportFromModel,
  onEditCopy,
  onDuplicate,
  invalidRepairActions = {},
}: {
  items: readonly AnimationStudioLibraryItemV1[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onCreateBlank: () => void;
  onCreateProcedural: () => void;
  onImportFromModel?: () => void;
  onEditCopy: (id: string) => void;
  onDuplicate: (id: string) => void;
  invalidRepairActions?: Readonly<Record<string, string>>;
}) {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<"BASE_42" | "CUSTOM">("CUSTOM");
  const visible = useMemo(
    () => filterAnimationStudioLibraryV1(items, query, filter),
    [filter, items, query],
  );
  const visibleHasSelection = visible.some(({ id }) => id === selectedId);
  const selected = items.find(({ id }) => id === selectedId);
  return (
    <aside
      className="animation-clip-library"
      aria-label="Animation clips"
      data-panel="clip-library"
    >
      <h2>Animation clips</h2>
      <div role="tablist" aria-label="Animation clip source">
        {(["BASE_42", "CUSTOM"] as const).map((value) => (
          <button
            type="button"
            role="tab"
            aria-selected={filter === value}
            tabIndex={filter === value ? 0 : -1}
            key={value}
            onClick={() => setFilter(value)}
            onKeyDown={(event) => moveCollectionFocus(
              event,
              ':scope > [role="tab"]',
            )}
          >
            {value === "BASE_42" ? "Base 42" : "Custom"}
          </button>
        ))}
      </div>
      <div className="animation-clip-library__actions">
        <NewAnimationMenu
          onCreateBlank={onCreateBlank}
          onCreateProcedural={onCreateProcedural}
          onImportFromModel={onImportFromModel}
        />
        <button
          type="button"
          disabled={!selected || selected.origin !== "SOURCE"}
          onClick={() => selected && onEditCopy(selected.id)}
        >
          Edit copy
        </button>
        <button
          type="button"
          disabled={!selected || selected.origin === "SOURCE"}
          onClick={() => selected && onDuplicate(selected.id)}
        >
          Duplicate
        </button>
      </div>
      <label>
        <span className="sr-only">Search animations</span>
        <input
          type="search"
          value={query}
          placeholder="Search animations…"
          onChange={(event) => setQuery(event.target.value)}
        />
      </label>
      <div
        className="animation-clip-library__list"
        role="listbox"
        aria-label={`${filter === "BASE_42" ? "Base 42" : "Custom"} animation clips`}
      >
        {visible.map((item, index) => (
          <AnimationClipLibraryRow
            key={item.id}
            item={item}
            selected={item.id === selectedId}
            onSelect={() => onSelect(item.id)}
            firstRepairAction={invalidRepairActions[item.id] ?? null}
            tabIndex={item.id === selectedId || (!visibleHasSelection && index === 0)
              ? 0
              : -1}
            onKeyDown={(event) => moveCollectionFocus(
              event,
              ':scope > [role="option"]',
            )}
          />
        ))}
        {visible.length === 0 ? <p>No animations in this view.</p> : null}
      </div>
      <p className="animation-studio-source-note">
        Source GLB stays unchanged. Edits are stored as a local project layer.
      </p>
    </aside>
  );
}

function moveCollectionFocus(
  event: KeyboardEvent<HTMLButtonElement>,
  selector: string,
) {
  if (!["ArrowDown", "ArrowRight", "ArrowUp", "ArrowLeft", "Home", "End"].includes(
    event.key,
  )) return;
  const items = Array.from(
    event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>(
      selector,
    ) ?? [],
  );
  const current = items.indexOf(event.currentTarget);
  if (current < 0 || items.length === 0) return;
  event.preventDefault();
  const next = event.key === "Home"
    ? 0
    : event.key === "End"
      ? items.length - 1
      : ["ArrowDown", "ArrowRight"].includes(event.key)
        ? (current + 1) % items.length
        : (current - 1 + items.length) % items.length;
  items[next]?.focus();
  items[next]?.click();
}
