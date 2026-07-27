import { useId, type DragEvent } from "react";
import {
  formatFileSize,
  shortSha256,
  type FileIdentityValue,
  type SourceInputProps,
  type TileSurface,
} from "./InputsPanel";
import type { StudioTarget } from "../../app/studioSession";
import type { MeshyArtifactProvenance } from "../meshy/bridge";

export interface SourceStepProps extends SourceInputProps {
  onContinue: () => void;
  onOpenMeshyLab?: () => void;
  meshyProvenance?: MeshyArtifactProvenance;
}

interface DropZoneProps {
  accept: string;
  description: string;
  error?: string;
  file?: File;
  identity?: FileIdentityValue;
  inputId: string;
  label: string;
  onRemove: () => void;
  onSelect: (file: File) => void;
  required?: boolean;
  selectLabel: string;
}

function DropZone({
  accept,
  description,
  error,
  file,
  identity,
  inputId,
  label,
  onRemove,
  onSelect,
  required = true,
  selectLabel,
}: DropZoneProps) {
  const descriptionId = `${inputId}-description`;
  const errorId = `${inputId}-error`;
  const hash = shortSha256(identity);

  const handleDrop = (event: DragEvent<HTMLLabelElement>) => {
    event.preventDefault();
    const droppedFile = event.dataTransfer.files[0];
    if (droppedFile) onSelect(droppedFile);
  };

  return (
    <article className={`source-input-card${error ? " source-input-card--error" : ""}`}>
      <header>
        <h2>{label}</h2>
        <p id={descriptionId}>{description}</p>
      </header>

      <label
        className="source-drop-zone"
        htmlFor={inputId}
        onDragEnter={(event) => event.preventDefault()}
        onDragOver={(event) => event.preventDefault()}
        onDrop={handleDrop}
      >
        {file ? (
          <span className="source-drop-zone__selection">
            <strong>{file.name}</strong>
            <span>{formatFileSize(file.size)}</span>
            {hash ? <code title={typeof identity === "string" ? identity : identity?.sha256 ?? undefined}>SHA-256 {hash}</code> : null}
            <span className="status-badge status-badge--neutral">Selected</span>
            <span>Drop a replacement here or choose another file</span>
          </span>
        ) : (
          <span className="source-drop-zone__empty">
            <strong>
              Drag &amp; drop {
                label === "Meshy model"
                  ? "a GLB file"
                  : label === "Creature animation events"
                    ? "an event JSON file"
                    : "a base 2DA file"
              } here
            </strong>
            <span>or click to browse</span>
          </span>
        )}
      </label>

      <input
        id={inputId}
        className="source-file-input"
        type="file"
        accept={accept}
        required={required}
        aria-describedby={`${descriptionId}${error ? ` ${errorId}` : ""}`}
        aria-invalid={error ? true : undefined}
        onClick={(event) => { event.currentTarget.value = ""; }}
        onChange={(event) => {
          const selected = event.currentTarget.files?.[0];
          if (selected) onSelect(selected);
        }}
      />

      <div className="source-input-card__actions">
        <label className="button button--secondary" htmlFor={inputId}>{file ? "Replace file" : selectLabel}</label>
        {file ? <button type="button" className="button button--quiet" onClick={onRemove}>Remove</button> : null}
      </div>
      {error ? <p id={errorId} role="alert">{error}</p> : null}
    </article>
  );
}

export function SourceStep({
  target = "CREATURE",
  tileTargetEnabled = false,
  tileOptions = { terrainName: "Grass", surface: "GRASS", interior: false },
  source,
  appearance,
  animationEvents,
  sourceIdentity,
  appearanceIdentity,
  sourceError,
  appearanceError,
  animationEventsError,
  onSelectSource,
  onSelectAppearance,
  onSelectAnimationEvents,
  onRemoveSource,
  onRemoveAppearance,
  onRemoveAnimationEvents,
  onClear,
  onTargetChange,
  onTileOptionsChange,
  onContinue,
  onOpenMeshyLab,
  meshyProvenance,
}: SourceStepProps) {
  const headingId = useId();
  const sourceInputId = useId();
  const appearanceInputId = useId();
  const animationEventsInputId = useId();
  const tileOptionsValid = tileOptions.terrainName.trim().length > 0
    && tileOptions.terrainName.length <= 64
    && /^[\x20-\x7e]+$/.test(tileOptions.terrainName);
  const ready = Boolean(
    source
    && (target === "TILE" ? tileOptionsValid : appearance)
    && !sourceError
    && !appearanceError
    && !animationEventsError
  );
  const hasSelection = Boolean(source || appearance || animationEvents);
  const readinessMessage = ready
    ? target === "TILE"
      ? "The tile GLB and authoring options are ready for local inspection."
      : "Both required files are selected. Continue to inspect their contents."
    : source
      ? target === "TILE"
        ? "Choose valid tile authoring options to continue."
        : "Select appearance.2da or placeables.2da to continue."
      : appearance
        ? "Select a Meshy GLB model to continue."
        : "Select both required files to continue.";

  return (
    <section className="source-step" aria-labelledby={headingId}>
      <header className="source-step__intro">
        <h1 id={headingId}>Start a new conversion</h1>
        <p>
          Select a Meshy GLB model and choose {
            tileTargetEnabled ? "Creature, Placeable, or Tile." : "Creature or Placeable."
          }
        </p>
        <p className="source-step__privacy">All processing stays in your browser. Files are not uploaded.</p>
      </header>

      {onTargetChange ? (
        <fieldset className="source-step__target">
          <legend>Conversion target</legend>
          {(tileTargetEnabled
            ? ["CREATURE", "PLACEABLE", "TILE"]
            : ["CREATURE", "PLACEABLE"]
          ).map((value) => (
            <label key={value}>
              <input
                type="radio"
                name="conversion-target"
                value={value as StudioTarget}
                checked={target === value}
                onChange={() => onTargetChange(value as StudioTarget)}
              />
              {value === "CREATURE" ? "Creature" : value === "PLACEABLE" ? "Placeable" : "Tile"}
            </label>
          ))}
        </fieldset>
      ) : null}

      {target === "TILE" && onTileOptionsChange ? (
        <fieldset className="source-step__tile-options">
          <legend>Tile authoring</legend>
          <label>
            Terrain name
            <input
              aria-label="Tile terrain name"
              value={tileOptions.terrainName}
              maxLength={32}
              onChange={(event) => onTileOptionsChange({
                ...tileOptions,
                terrainName: event.currentTarget.value,
              })}
            />
          </label>
          <label>
            Walk surface
            <select
              aria-label="Tile walk surface"
              value={tileOptions.surface}
              onChange={(event) => onTileOptionsChange({
                ...tileOptions,
                surface: event.currentTarget.value as TileSurface,
              })}
            >
              <option value="DIRT">Dirt (1)</option>
              <option value="GRASS">Grass (3)</option>
              <option value="STONE">Stone (4)</option>
              <option value="WOOD">Wood (5)</option>
            </select>
          </label>
          <label>
            <input
              type="checkbox"
              checked={tileOptions.interior}
              onChange={(event) => onTileOptionsChange({
                ...tileOptions,
                interior: event.currentTarget.checked,
              })}
            />
            Interior tileset
          </label>
          <output>Footprint: 10 m × 10 m · seam: ±5 m · Tile_ID 0</output>
        </fieldset>
      ) : null}

      <div className="source-step__inputs">
        <DropZone
          inputId={sourceInputId}
          label="Meshy model"
          description="Required format: GLB"
          accept=".glb,model/gltf-binary"
          file={source}
          identity={sourceIdentity}
          error={sourceError}
          onSelect={onSelectSource}
          onRemove={onRemoveSource}
          selectLabel="Select GLB"
        />
        {target !== "TILE" ? <DropZone
          inputId={appearanceInputId}
          label="Base model table"
          description={target === "PLACEABLE" ? "Required file: placeables.2da" : "Required file: appearance.2da"}
          accept=".2da"
          file={appearance}
          identity={appearanceIdentity}
          error={appearanceError}
          onSelect={onSelectAppearance}
          onRemove={onRemoveAppearance}
          selectLabel="Select base 2DA"
        /> : null}
        {target === "CREATURE" ? <DropZone
          inputId={animationEventsInputId}
          label="Creature animation events"
          description="Optional strict V1 JSON. Required only for the event-complete exact 42-state lane; all times remain caller-owned."
          accept=".json,application/json"
          file={animationEvents}
          error={animationEventsError}
          onSelect={onSelectAnimationEvents}
          onRemove={onRemoveAnimationEvents}
          required={false}
          selectLabel="Select event JSON"
        /> : null}
      </div>

      {onOpenMeshyLab ? (
        <aside className="source-step__meshy-lab" aria-label="Optional Meshy Lab integration">
          <div><strong>Need a reproducible proof asset?</strong><span>Use the optional local Meshy Lab, then import its verified GLB here.</span></div>
          <button type="button" className="button button--secondary" onClick={onOpenMeshyLab}>Open Meshy Lab</button>
        </aside>
      ) : null}

      {meshyProvenance ? (
        <p className="source-step__provenance">
          Imported from Meshy Lab: {meshyProvenance.profileId} · SHA-256 {meshyProvenance.sha256.slice(0, 12)}...
        </p>
      ) : null}

      <footer className="source-step__actions">
        <p className="source-step__readiness" aria-live="polite">{readinessMessage}</p>
        <button type="button" className="button button--secondary" onClick={onClear} disabled={!hasSelection}>
          Clear files
        </button>
        <button type="button" className="button button--primary" onClick={onContinue} disabled={!ready}>
          Continue to Inspect
        </button>
      </footer>
    </section>
  );
}
