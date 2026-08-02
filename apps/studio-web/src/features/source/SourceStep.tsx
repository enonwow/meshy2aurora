import { useId, type DragEvent } from "react";
import {
  formatFileSize,
  shortSha256,
  type FileIdentityValue,
  type CreatureConversionProfileV1,
  type CreatureSourceForwardV1,
  type SkinAccessoryStabilizationModeV1,
  type SourceInputProps,
  type TileSurface,
} from "./InputsPanel";
import type { StudioTarget } from "../../app/studioSession";
import type { MeshyArtifactProvenance } from "../meshy/bridge";
import { parseSkinAccessoryComponentBoneOverridesV2 } from "./skinAccessoryOverrides";

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
  creatureProfile = "PRODUCT_300K",
  creatureSourceForward = "POSITIVE_Z",
  textureArtifactCleanup = false,
  experimentalAggressiveGeometryCleanup = false,
  skinAccessoryStabilizationMode = "AUTO",
  skinAccessorySelectedBoneName = "",
  skinAccessoryComponentBoneOverrides = "",
  sourceIdentity,
  appearanceIdentity,
  sourceError,
  appearanceError,
  animationEventsError,
  onSelectSource,
  onSelectAppearance,
  onSelectAnimationEvents,
  onCreatureProfileChange,
  onCreatureSourceForwardChange,
  onTextureArtifactCleanupChange,
  onExperimentalAggressiveGeometryCleanupChange,
  onSkinAccessoryStabilizationModeChange,
  onSkinAccessorySelectedBoneNameChange,
  onSkinAccessoryComponentBoneOverridesChange,
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
  let componentOverridesValid = true;
  let componentOverrideCount = 0;
  try {
    componentOverrideCount = parseSkinAccessoryComponentBoneOverridesV2(
      skinAccessoryComponentBoneOverrides,
    ).length;
  } catch {
    componentOverridesValid = false;
  }
  const ready = Boolean(
    source
    && (target === "TILE" ? tileOptionsValid : appearance)
    && (
      target !== "CREATURE"
      || skinAccessoryStabilizationMode !== "SELECT_BONE"
      || (
        componentOverridesValid
        && (
          skinAccessorySelectedBoneName.trim().length > 0
          || componentOverrideCount > 0
        )
      )
    )
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

      {target === "CREATURE" && onCreatureProfileChange ? (
        <fieldset className="source-step__creature-profile">
          <legend>Creature conversion profile</legend>
          <label>
            Pipeline profile
            <select
              aria-label="Creature conversion profile"
              value={creatureProfile}
              onChange={(event) => onCreatureProfileChange(
                event.currentTarget.value as CreatureConversionProfileV1,
              )}
            >
              <option value="PRODUCT_300K">Product (300,000 triangle limit)</option>
              <option value="EXPERIMENTAL_P100K">Legacy P100K compatibility</option>
              <option value="EXPERIMENTAL_P300K">Legacy P300K compatibility</option>
            </select>
          </label>
          {onCreatureSourceForwardChange ? (
            <>
              <label>
                Model front in source GLB
                <select
                  aria-label="Model front in source GLB"
                  value={creatureSourceForward}
                  onChange={(event) => onCreatureSourceForwardChange(
                    event.currentTarget.value as CreatureSourceForwardV1,
                  )}
                >
                  <option value="POSITIVE_Z">+Z</option>
                  <option value="NEGATIVE_Z">-Z</option>
                  <option value="POSITIVE_X">+X</option>
                  <option value="NEGATIVE_X">-X</option>
                </select>
              </label>
              <p role="note">
                Choose the axis the character faces in the source GLB. The pipeline rotates that
                axis to Aurora/NWN forward (-Y) without mirroring the model.
              </p>
            </>
          ) : null}
          {creatureProfile === "EXPERIMENTAL_P100K" ? (
            <p role="note">
              Historical P100K replay profile. New conversions should use the shared 300,000-triangle product profile.
            </p>
          ) : null}
          {creatureProfile === "EXPERIMENTAL_P300K" ? (
            <p role="note">
              Historical P300K replay profile. New conversions should use the shared 300,000-triangle product profile.
            </p>
          ) : null}
          {onTextureArtifactCleanupChange ? (
            <>
              <label>
                <input
                  type="checkbox"
                  aria-label="Repair texture artifacts"
                  checked={textureArtifactCleanup}
                  onChange={(event) => onTextureArtifactCleanupChange(event.currentTarget.checked)}
                />
                Repair texture artifacts
              </label>
              <p role="note">
                Removes isolated bright or dark speckles and repairs one-pixel alpha holes in the
                base-color texture. Larger details, edges, and transparent regions are preserved.
              </p>
            </>
          ) : null}
          {onSkinAccessoryStabilizationModeChange ? (
            <>
              <label>
                Detached accessory skinning
                <select
                  aria-label="Detached accessory skinning"
                  value={skinAccessoryStabilizationMode}
                  onChange={(event) => onSkinAccessoryStabilizationModeChange(
                    event.currentTarget.value as SkinAccessoryStabilizationModeV1,
                  )}
                >
                  <option value="AUTO">Auto</option>
                  <option value="KEEP_SOURCE_WEIGHTS">Keep source weights</option>
                  <option value="SELECT_BONE">Select bone</option>
                </select>
              </label>
              {skinAccessoryStabilizationMode === "SELECT_BONE" ? (
                <>
                  <label>
                    Default accessory bone
                    <input
                      aria-label="Accessory bone name"
                      value={skinAccessorySelectedBoneName}
                      onChange={(event) => onSkinAccessorySelectedBoneNameChange?.(
                        event.currentTarget.value,
                      )}
                      placeholder="Optional fallback, e.g. Spine02"
                    />
                  </label>
                  <label>
                    Component bone overrides
                    <textarea
                      aria-label="Accessory component bone overrides"
                      value={skinAccessoryComponentBoneOverrides}
                      onChange={(event) => onSkinAccessoryComponentBoneOverridesChange?.(
                        event.currentTarget.value,
                      )}
                      aria-invalid={!componentOverridesValid || undefined}
                      placeholder={"0:1=Spine02\n0:2=Spine"}
                    />
                  </label>
                  {!componentOverridesValid ? (
                    <p role="alert">Each override must use segment:component=BoneName.</p>
                  ) : null}
                </>
              ) : null}
              <p role="note">
                Auto audits spatially welded detached parts across animation clips and stabilizes
                only risky accessories on a nearby torso bone. Keep source weights records the
                risk without changing it. Select bone accepts a default bone plus optional
                per-component overrides such as 0:1=Spine02. Geometry, UVs, materials, and
                animation clips are preserved.
              </p>
            </>
          ) : null}
        </fieldset>
      ) : null}

      {target === "PLACEABLE" && onExperimentalAggressiveGeometryCleanupChange ? (
        <fieldset className="source-step__placeable-geometry-cleanup">
          <legend>Placeable geometry</legend>
          <label>
            <input
              type="checkbox"
              aria-label="Experimental aggressive geometry cleanup"
              checked={experimentalAggressiveGeometryCleanup}
              onChange={(event) => onExperimentalAggressiveGeometryCleanupChange(
                event.currentTarget.checked,
              )}
            />
            Experimental aggressive geometry cleanup
          </label>
          <p role="note">
            Trial option, disabled by default. It removes triangles below the legacy fixed-area
            threshold and may create holes or erase dense detail. When disabled, the pipeline
            preserves all finite non-collinear triangles and removes only faces that cannot form
            a valid Aurora face plane.
          </p>
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
