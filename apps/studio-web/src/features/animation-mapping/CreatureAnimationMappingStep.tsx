import {
  useEffect,
  useMemo,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import {
  filterAnimationCatalogV1,
  projectAnimationCatalogRowsV1,
} from "./catalog";
import {
  AnimationMappingStatusBar,
  type AnimationMappingSaveStateV1,
} from "./AnimationMappingStatusBar";
import type {
  AnimationCatalogFilterV1,
  AnimationCatalogRowV1,
  CreatureAnimationAuthoringV1,
  CreatureAnimationInspectionV1,
  CreatureAnimationMappingStatusV1,
} from "./types";
import type { CreatureAnimationAuthoringEventV1 } from "./state";
import { AnimationSourcePicker } from "./AnimationSourcePicker";
import { FallbackReview } from "./FallbackReview";
import { AnimationPreviewPanel } from "./AnimationPreviewPanel";
import type {
  BinaryMdlInspectionReport,
  SourcePreviewInput,
} from "../preview/types";
import {
  AnimationMappingModeSwitch,
  type AnimationMappingModeV1,
} from "../animation-editor/AnimationMappingModeSwitch";
import { CustomAnimationMappingPanel } from "../animation-editor/CustomAnimationMappingPanel";
import type {
  AnimationStudioDocumentV1,
  CreatureAnimationAuthoringV2,
} from "../animation-studio/types";
import "./CreatureAnimationMappingStep.css";

export interface CreatureAnimationMappingStepProps {
  authoring: CreatureAnimationAuthoringV1;
  inspection: CreatureAnimationInspectionV1;
  saveState: AnimationMappingSaveStateV1;
  canContinue: boolean;
  canonicalValidationPending?: boolean;
  validationStatus?: CreatureAnimationMappingStatusV1;
  onBack: () => void;
  onContinue: () => void;
  onApplySuggestions: () => void;
  canUseGeneratedProfile?: boolean;
  onUseGeneratedProfile?: () => void;
  onAuthoringEvent?: (event: CreatureAnimationAuthoringEventV1) => void;
  sourcePreviewInput?: SourcePreviewInput;
  readback?: BinaryMdlInspectionReport;
  builtAuthoringRevision?: number | null;
  onPreviewError?: (message: string) => void;
  animationStudio?: ReactNode;
  animationMode?: AnimationMappingModeV1;
  onAnimationModeChange?: (mode: AnimationMappingModeV1) => void;
  animationStudioDocument?: AnimationStudioDocumentV1;
  animationAuthoringV2?: CreatureAnimationAuthoringV2;
  onAnimationAuthoringV2Change?: (authoring: CreatureAnimationAuthoringV2) => void;
  onCreateAuthoredAnimation?: () => void;
  onOpenAuthoredAnimation?: (clipId: string) => void;
}

const FILTERS: readonly {
  id: AnimationCatalogFilterV1;
  label: string;
}[] = [
  { id: "NEEDS_ATTENTION", label: "Needs attention" },
  { id: "BASE_42", label: "Base 42" },
  { id: "CUSTOM", label: "Custom" },
];

export function CreatureAnimationMappingStep({
  authoring,
  inspection,
  saveState,
  canContinue,
  canonicalValidationPending = false,
  validationStatus,
  onBack,
  onContinue,
  onApplySuggestions,
  canUseGeneratedProfile = false,
  onUseGeneratedProfile,
  onAuthoringEvent,
  sourcePreviewInput,
  readback,
  builtAuthoringRevision,
  onPreviewError,
  animationStudio,
  animationMode,
  onAnimationModeChange,
  animationStudioDocument,
  animationAuthoringV2,
  onAnimationAuthoringV2Change,
  onCreateAuthoredAnimation,
  onOpenAuthoredAnimation,
}: CreatureAnimationMappingStepProps) {
  const [internalMode, setInternalMode] = useState<AnimationMappingModeV1>(
    readAnimationMappingModeV1,
  );
  const mode = animationMode ?? internalMode;
  const setMode = (next: AnimationMappingModeV1) => {
    setInternalMode(next);
    onAnimationModeChange?.(next);
  };
  const rows = useMemo(
    () => projectAnimationCatalogRowsV1(authoring, inspection),
    [authoring, inspection],
  );
  const [filter, setFilter] = useState<AnimationCatalogFilterV1>("NEEDS_ATTENTION");
  const [query, setQuery] = useState("");
  const visibleRows = useMemo(
    () => filterAnimationCatalogV1(rows, query, filter),
    [rows, query, filter],
  );
  const [selectedId, setSelectedId] = useState<string | null>(
    visibleRows[0]?.id ?? null,
  );
  const [selectionNotice, setSelectionNotice] = useState<string | null>(null);

  useEffect(() => {
    if (!visibleRows.some(({ id }) => id === selectedId)) {
      if (selectedId !== null) {
        setSelectionNotice(
          "The previous row no longer matches this filter. The first visible row was selected.",
        );
      }
      setSelectedId(visibleRows[0]?.id ?? null);
    } else {
      setSelectionNotice(null);
    }
  }, [selectedId, visibleRows]);

  const selected = rows.find(({ id }) => id === selectedId) ?? visibleRows[0] ?? null;
  const status = validationStatus ?? mappingStatus(rows);
  const counts = Object.fromEntries(FILTERS.map(({ id }) => [
    id,
    filterAnimationCatalogV1(rows, "", id).length,
  ])) as Record<AnimationCatalogFilterV1, number>;

  useEffect(() => {
    persistAnimationMappingModeV1(mode);
  }, [mode]);

  return (
    <section className="animation-mapping-step" aria-labelledby="animation-mapping-title">
      <header className="animation-mapping-step__header">
        <div>
          <p className="animation-mapping-step__eyebrow">Creature profile · MODELTYPE {authoring.modelType}</p>
          <h1 id="animation-mapping-title">Creature Animation Mapping</h1>
          <p>
            Map 42 Aurora base slots to source, inherited or generated motion.
            Derived Aurora fields stay read-only.
          </p>
        </div>
        <div className="animation-mapping-step__header-actions">
          <button type="button" onClick={onApplySuggestions}>Apply safe suggestions</button>
          {onUseGeneratedProfile ? (
            <button
              type="button"
              disabled={!canUseGeneratedProfile}
              onClick={onUseGeneratedProfile}
            >
              Use generated Base 42
            </button>
          ) : null}
        </div>
      </header>

      <AnimationMappingModeSwitch value={mode} onChange={setMode} />

      {mode === "EDIT" && animationStudio ? animationStudio : (
        <>
      <div className="animation-mapping-step__workspace">
        <aside className="animation-catalog" aria-label="Animation catalog">
          <label>
            <span>Search</span>
            <input
              type="search"
              aria-label="Search animation states"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="State, slot or source"
            />
          </label>
          <div className="animation-catalog__tabs" role="tablist" aria-label="Catalog views">
            {FILTERS.map(({ id, label }) => (
              <button
                key={id}
                type="button"
                role="tab"
                aria-selected={filter === id}
                onClick={() => setFilter(id)}
                onKeyDown={moveKeyboardSelection}
              >
                {label} <span>{counts[id]}</span>
              </button>
            ))}
          </div>
          <div
            className="animation-catalog__list"
            role="listbox"
            aria-label={`${filter.replace("_", " ")} animation rows`}
          >
            {visibleRows.length > 0 ? visibleRows.map((row) => (
              <button
                key={row.id}
                type="button"
                role="option"
                aria-selected={row.id === selected?.id}
                onClick={() => setSelectedId(row.id)}
                onKeyDown={moveKeyboardSelection}
              >
                <span>
                  <strong>{row.label}</strong>
                  <small>{row.slot ?? row.id}</small>
                </span>
                <StatusPill row={row} />
              </button>
            )) : (
              <p className="animation-catalog__empty">No animations match this view.</p>
            )}
          </div>
          {selectionNotice ? <p role="status">{selectionNotice}</p> : null}
        </aside>

        <main className="animation-mapping-detail">
          {selected ? (
            <>
              <header>
                <span>{selected.kind === "BASE" ? "Aurora state" : "Custom animation"}</span>
                <h2>{selected.label}</h2>
                <p>{selected.description}</p>
              </header>
              <dl className="animation-mapping-detail__derived">
                <DerivedField label="State ID" value={selected.stateId ?? "Custom"} />
                <DerivedField label="MODELTYPE" value={selected.modelType} />
                <DerivedField label="Resolved slot" value={selected.slot ?? "Custom namespace"} />
                <DerivedField label="Playback" value={selected.playbackPolicy} />
              </dl>
              <section aria-labelledby="animation-source-heading">
                <h3 id="animation-source-heading">Source realization</h3>
                <dl className="animation-mapping-detail__source">
                  <DerivedField label="Source" value={selected.sourceLabel ?? "Not assigned"} />
                  <DerivedField
                    label="Provider"
                    value={selected.provenance?.provider ?? "—"}
                  />
                  <DerivedField
                    label="Asset"
                    value={selected.provenance?.assetId ?? "—"}
                  />
                  <DerivedField
                    label="Ownership"
                    value={selected.provenance?.ownership ?? "—"}
                  />
                </dl>
                {selected.slot && onAuthoringEvent ? (
                  <AnimationSourcePicker
                    slot={selected.slot}
                    authoring={authoring}
                    inspection={inspection}
                    onEvent={onAuthoringEvent}
                    customSelected={animationAuthoringV2?.assignments.some(
                      (assignment) => (
                        assignment.targetSlot === selected.slot
                        && assignment.sourceKind === "CUSTOM"
                      ),
                    )}
                    customPanel={animationStudioDocument
                      && animationAuthoringV2
                      && onAnimationAuthoringV2Change
                      && onCreateAuthoredAnimation
                      && onOpenAuthoredAnimation ? (
                      <CustomAnimationMappingPanel
                        slot={selected.slot}
                        authoring={animationAuthoringV2}
                        studio={animationStudioDocument}
                        sourceInventory={inspection.sourceClips}
                        onAuthoringChange={onAnimationAuthoringV2Change}
                        onCreate={onCreateAuthoredAnimation}
                        onOpenClip={onOpenAuthoredAnimation}
                      />
                    ) : undefined}
                  />
                ) : null}
                {selected.diagnosticCodes.length > 0 ? (
                  <div className="animation-mapping-detail__diagnostics" role="status">
                    <strong>Needs attention</strong>
                    <ul>
                      {selected.diagnosticCodes.map((code) => (
                        <li
                          key={code}
                          data-animation-diagnostic-code={code}
                          tabIndex={-1}
                        >
                          {code}
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : null}
              </section>
              {onAuthoringEvent ? (
                <FallbackReview authoring={authoring} onEvent={onAuthoringEvent} />
              ) : null}
            </>
          ) : (
            <div className="animation-mapping-detail__empty">
              <h2>No row selected</h2>
              <p>Change the filter or clear the search query.</p>
            </div>
          )}
        </main>

        <AnimationPreviewPanel
          row={selected}
          authoring={authoring}
          inspection={inspection}
          sourceInput={sourcePreviewInput}
          readback={readback}
          builtAuthoringRevision={builtAuthoringRevision}
          onError={onPreviewError}
        />
      </div>

      <footer className="animation-mapping-step__footer">
        <AnimationMappingStatusBar
          rows={rows}
          status={status}
          saveState={saveState}
          canonicalValidationPending={canonicalValidationPending}
        />
        <div>
          <button type="button" onClick={onBack}>Back to Inspect</button>
          <button type="button" disabled={!canContinue} onClick={onContinue}>
            Continue to Build
          </button>
        </div>
      </footer>
        </>
      )}
    </section>
  );
}

const ANIMATION_MAPPING_MODE_STORAGE_KEY = "m2a.animationMapping.mode.v1";

function readAnimationMappingModeV1(): AnimationMappingModeV1 {
  if (typeof window === "undefined") return "MAP";
  const query = new URLSearchParams(window.location.search).get("animationMode");
  if (query === "edit") return "EDIT";
  if (query === "map") return "MAP";
  return window.localStorage.getItem(ANIMATION_MAPPING_MODE_STORAGE_KEY) === "EDIT"
    ? "EDIT"
    : "MAP";
}

function persistAnimationMappingModeV1(mode: AnimationMappingModeV1) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(ANIMATION_MAPPING_MODE_STORAGE_KEY, mode);
  const url = new URL(window.location.href);
  url.searchParams.set("animationMode", mode === "EDIT" ? "edit" : "map");
  window.history.replaceState(window.history.state, "", url);
}

function DerivedField({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function StatusPill({ row }: { row: AnimationCatalogRowV1 }) {
  return (
    <span className="animation-catalog__status" data-status={row.status}>
      {row.status.replace("_", " ")}
    </span>
  );
}

function mappingStatus(
  rows: readonly AnimationCatalogRowV1[],
): CreatureAnimationMappingStatusV1 {
  if (rows.some(({ status }) => status === "BLOCKED")) return "BLOCKED";
  if (rows.some(({ status }) => status === "NEEDS_REVIEW")) return "NEEDS_REVIEW";
  return "READY";
}

function moveKeyboardSelection(event: KeyboardEvent<HTMLButtonElement>) {
  if (!["ArrowDown", "ArrowRight", "ArrowUp", "ArrowLeft", "Home", "End"].includes(event.key)) {
    return;
  }
  const items = Array.from(
    event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>(
      ':scope > button[role="tab"], :scope > button[role="option"]',
    ) ?? [],
  );
  const currentIndex = items.indexOf(event.currentTarget);
  if (currentIndex < 0 || items.length === 0) return;
  event.preventDefault();
  const nextIndex = event.key === "Home"
    ? 0
    : event.key === "End"
      ? items.length - 1
      : ["ArrowDown", "ArrowRight"].includes(event.key)
        ? (currentIndex + 1) % items.length
        : (currentIndex - 1 + items.length) % items.length;
  items[nextIndex]?.focus();
  items[nextIndex]?.click();
}
