import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
} from "react";
import {
  ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1,
  animationPresetCatalogIdentityV1,
  getAnimationPresetCatalogV1,
  loadAnimationPresetAssetsV1,
  projectAnimationPresetCatalogWindowV1,
  searchAnimationPresetCatalogV1,
} from "../animation-library/catalog";
import type {
  AnimationPresetCatalogEntryV1,
  AnimationPresetCompatibilityV1,
} from "../animation-library/types";
import { AnimationClipLibraryRow } from "./AnimationClipLibraryRow";
import {
  filterAnimationStudioLibraryV1,
  type AnimationStudioLibraryItemV1,
} from "./editing";
import { NewAnimationMenu } from "./NewAnimationMenu";

type LibraryTabV1 = "BASE_42" | "BUILT_IN" | "COMMUNITY" | "CUSTOM";

export function AnimationClipLibrary({
  items,
  selectedId,
  onSelect,
  onCreateBlank,
  onImportFromModel,
  onEditCopy,
  onDuplicate,
  onInspectPreset,
  onUsePreset,
  invalidRepairActions = {},
}: {
  items: readonly AnimationStudioLibraryItemV1[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onCreateBlank: () => void;
  onImportFromModel?: () => void;
  onEditCopy: (id: string) => void;
  onDuplicate: (id: string) => void;
  onInspectPreset?: (
    preset: AnimationPresetCatalogEntryV1,
  ) => Promise<AnimationPresetCompatibilityV1>;
  onUsePreset?: (preset: AnimationPresetCatalogEntryV1) => Promise<void>;
  invalidRepairActions?: Readonly<Record<string, string>>;
}) {
  const presets = getAnimationPresetCatalogV1();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<LibraryTabV1>("CUSTOM");
  const [selectedPresetKey, setSelectedPresetKey] = useState<string | null>(null);
  const [selectedTags, setSelectedTags] = useState<readonly string[]>([]);
  const [visiblePresetCount, setVisiblePresetCount] = useState(
    ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1,
  );
  const [compatibility, setCompatibility] = useState<AnimationPresetCompatibilityV1 | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [presetState, setPresetState] = useState<"IDLE" | "CHECKING" | "USING">("IDLE");
  const [presetError, setPresetError] = useState<string | null>(null);
  const inspectionSequence = useRef(0);
  const isRepositoryTab = filter === "BUILT_IN" || filter === "COMMUNITY";
  const visible = useMemo(
    () => isRepositoryTab
      ? []
      : filterAnimationStudioLibraryV1(items, query, filter),
    [filter, isRepositoryTab, items, query],
  );
  const visiblePresets = useMemo(
    () => isRepositoryTab
      ? searchAnimationPresetCatalogV1(
          query,
          filter === "BUILT_IN" ? "BUILT_IN" : "COMMUNITY",
          selectedTags,
        )
      : [],
    [filter, isRepositoryTab, query, selectedTags],
  );
  const renderedPresets = useMemo(
    () => projectAnimationPresetCatalogWindowV1(visiblePresets, visiblePresetCount),
    [visiblePresetCount, visiblePresets],
  );
  useEffect(() => {
    setVisiblePresetCount(ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1);
  }, [filter, query, selectedTags]);
  const availableTags = useMemo(() => [...new Set(presets
    .filter(({ source }) => (
      filter === "BUILT_IN" ? source === "BUILT_IN" : source === "COMMUNITY"
    ))
    .flatMap(({ tags }) => tags))].sort(), [filter, presets]);
  const visibleHasSelection = visible.some(({ id }) => id === selectedId);
  const selected = items.find(({ id }) => id === selectedId);
  const selectedPreset = presets.find((preset) => (
    animationPresetCatalogIdentityV1(preset) === selectedPresetKey
  ))
    ?? null;

  const inspectPreset = async (preset: AnimationPresetCatalogEntryV1) => {
    setSelectedPresetKey(animationPresetCatalogIdentityV1(preset));
    setCompatibility(null);
    setPreviewUrl(null);
    setPresetError(null);
    const sequence = ++inspectionSequence.current;
    setPresetState("CHECKING");
    try {
      const [assets, result] = await Promise.all([
        loadAnimationPresetAssetsV1(preset.presetId, preset.presetVersion),
        onInspectPreset ? onInspectPreset(preset) : Promise.resolve(null),
      ]);
      if (inspectionSequence.current === sequence) {
        setPreviewUrl(assets.previewUrl);
        setCompatibility(result);
      }
    } catch (error: unknown) {
      if (inspectionSequence.current === sequence) {
        setPresetError(error instanceof Error ? error.message : String(error));
      }
    } finally {
      if (inspectionSequence.current === sequence) setPresetState("IDLE");
    }
  };

  const usePreset = async () => {
    if (!selectedPreset || !onUsePreset || compatibility?.status !== "COMPATIBLE") return;
    setPresetState("USING");
    setPresetError(null);
    try {
      await onUsePreset(selectedPreset);
      setFilter("CUSTOM");
      setSelectedPresetKey(null);
      setCompatibility(null);
    } catch (error: unknown) {
      setPresetError(error instanceof Error ? error.message : String(error));
    } finally {
      setPresetState("IDLE");
    }
  };

  return (
    <aside
      className="animation-clip-library"
      aria-label="Animation clips"
      data-panel="clip-library"
    >
      <h2>Animation library</h2>
      <div role="tablist" aria-label="Animation clip source">
        {(["BASE_42", "BUILT_IN", "COMMUNITY", "CUSTOM"] as const).map((value) => (
          <button
            type="button"
            role="tab"
            aria-selected={filter === value}
            tabIndex={filter === value ? 0 : -1}
            key={value}
            onClick={() => {
              inspectionSequence.current += 1;
              setFilter(value);
              setSelectedTags([]);
              setVisiblePresetCount(ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1);
              setSelectedPresetKey(null);
              setCompatibility(null);
              setPreviewUrl(null);
              setPresetError(null);
            }}
            onKeyDown={(event) => moveCollectionFocus(event, ':scope > [role="tab"]')}
          >
            {tabLabel(value)}
          </button>
        ))}
      </div>
      {!isRepositoryTab ? (
        <div className="animation-clip-library__actions">
          <NewAnimationMenu
            onCreateBlank={onCreateBlank}
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
      ) : null}
      <label>
        <span className="sr-only">Search animations</span>
        <input
          type="search"
          value={query}
          placeholder="Search name, ID, author or tag…"
          onChange={(event) => setQuery(event.target.value)}
        />
      </label>
      {isRepositoryTab && availableTags.length > 0 ? (
        <div className="animation-library-tags" aria-label="Filter by tags">
          {availableTags.map((tag) => {
            const selected = selectedTags.includes(tag);
            return (
              <button
                key={tag}
                type="button"
                aria-pressed={selected}
                onClick={() => setSelectedTags((current) => selected
                  ? current.filter((value) => value !== tag)
                  : [...current, tag].sort())}
              >
                {tag}
              </button>
            );
          })}
        </div>
      ) : null}
      <div
        className="animation-clip-library__list"
        role="listbox"
        aria-label={`${tabLabel(filter)} animation clips`}
      >
        {isRepositoryTab ? renderedPresets.map((preset, index) => (
          <button
            type="button"
            role="option"
            aria-selected={animationPresetCatalogIdentityV1(preset) === selectedPresetKey}
            tabIndex={animationPresetCatalogIdentityV1(preset) === selectedPresetKey || (!selectedPresetKey && index === 0) ? 0 : -1}
            className="animation-library-preset-row"
            key={`${preset.presetId}@${preset.presetVersion}`}
            onClick={() => void inspectPreset(preset)}
            onKeyDown={(event) => moveCollectionFocus(event, ':scope > [role="option"]')}
          >
            <span>
              <strong>{preset.label}</strong>
              <small>{preset.presetId} · v{preset.presetVersion}</small>
            </span>
            <span className="animation-library-preset-row__badges">
              <small>{preset.playback === "LOOP" ? "Loop" : "One shot"}</small>
              <small>{preset.validationStatus === "OWNER_NWN_VERIFIED" ? "NWN verified" : "Pipeline verified"}</small>
            </span>
          </button>
        )) : visible.map((item, index) => (
          <AnimationClipLibraryRow
            key={item.id}
            item={item}
            selected={item.id === selectedId}
            onSelect={() => onSelect(item.id)}
            firstRepairAction={invalidRepairActions[item.id] ?? null}
            tabIndex={item.id === selectedId || (!visibleHasSelection && index === 0) ? 0 : -1}
            onKeyDown={(event) => moveCollectionFocus(event, ':scope > [role="option"]')}
          />
        ))}
        {(isRepositoryTab ? visiblePresets.length : visible.length) === 0
          ? <p>No animations match this view.</p>
          : null}
      </div>
      {isRepositoryTab && visiblePresets.length > ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1 ? (
        <div className="animation-library-result-window" aria-live="polite">
          <span>
            Showing {renderedPresets.length} of {visiblePresets.length} matching presets.
          </span>
          {renderedPresets.length < visiblePresets.length ? (
            <button
              type="button"
              onClick={() => setVisiblePresetCount((count) => (
                Math.min(count + ANIMATION_LIBRARY_RESULT_PAGE_SIZE_V1, visiblePresets.length)
              ))}
            >
              Show more presets
            </button>
          ) : null}
        </div>
      ) : null}
      {isRepositoryTab && selectedPreset ? (
        <section className="animation-library-preset-details" aria-label="Selected animation preset details">
          <div className="animation-library-preset-details__heading">
            <h3>{selectedPreset.label}</h3>
            <span data-status={compatibility?.status ?? "NOT_CHECKED"}>
              {presetState === "CHECKING"
                ? "Checking…"
                : compatibility?.status === "COMPATIBLE"
                  ? "Compatible"
                  : compatibility?.status === "INCOMPATIBLE"
                    ? "Incompatible"
                    : "Not checked"}
            </span>
          </div>
          <p>{selectedPreset.summary}</p>
          {previewUrl ? (
            <img
              className="animation-library-preset-details__preview"
              src={previewUrl}
              alt={`${selectedPreset.label} motion phases preview`}
            />
          ) : (
            <p className="animation-library-preset-details__preview-state" role="status">
              {presetState === "CHECKING" ? "Loading preview…" : "Preview unavailable."}
            </p>
          )}
          <dl>
            <div><dt>Author</dt><dd>{selectedPreset.authors.map(({ name }) => name).join(", ")}</dd></div>
            <div><dt>License</dt><dd>{selectedPreset.license}</dd></div>
            <div><dt>Rig</dt><dd>{selectedPreset.rigProfile}</dd></div>
            <div><dt>Proof</dt><dd>{selectedPreset.validationStatus === "OWNER_NWN_VERIFIED" ? "Owner NWN verified" : "Offline pipeline verified"}</dd></div>
          </dl>
          <div className="animation-library-preset-details__tags">
            {selectedPreset.tags.map((tag) => <span key={tag}>{tag}</span>)}
          </div>
          {compatibility?.diagnostics.map((diagnostic) => (
            <p className="animation-library-preset-details__error" key={`${diagnostic.code}:${diagnostic.path}`}>
              {diagnostic.message} {diagnostic.action}
            </p>
          ))}
          {presetError ? <p className="animation-library-preset-details__error" role="alert">{presetError}</p> : null}
          <button
            type="button"
            disabled={
              !onUsePreset
              || compatibility?.status !== "COMPATIBLE"
              || presetState !== "IDLE"
            }
            onClick={() => void usePreset()}
          >
            {presetState === "USING" ? "Creating Custom copy…" : "Use as template"}
          </button>
          <small>Repository presets are immutable. Editing always creates a local Custom copy.</small>
        </section>
      ) : null}
      <p className="animation-studio-source-note">
        Source GLB stays unchanged. Edits are stored as a local project layer.
      </p>
    </aside>
  );
}

function tabLabel(value: LibraryTabV1) {
  if (value === "BASE_42") return "Base 42";
  if (value === "BUILT_IN") return "Built-in";
  if (value === "COMMUNITY") return "Community";
  return "Custom";
}

function moveCollectionFocus(
  event: KeyboardEvent<HTMLButtonElement>,
  selector: string,
) {
  if (!["ArrowDown", "ArrowRight", "ArrowUp", "ArrowLeft", "Home", "End"].includes(event.key)) return;
  const items = Array.from(
    event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>(selector) ?? [],
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
