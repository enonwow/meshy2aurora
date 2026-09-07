import { useId, type ChangeEvent } from "react";
import type { StudioTarget } from "../../app/studioSession";
import { WeaponGripControls } from "../preview/WeaponGripControls";
import { HeldWeaponControls } from "./HeldWeaponControls";
import type { CreatureHeldWeaponModeV1 } from "./heldWeapon";
import {
  defaultCreatureWeaponGripOptionsV1,
  type CreatureWeaponGripOptionsV1,
} from "./weaponGrip";

export type TileSurface = "DIRT" | "GRASS" | "STONE" | "WOOD";
export type CreatureConversionProfileV1 =
  | "PRODUCT_300K"
  | "EXPERIMENTAL_P100K"
  | "EXPERIMENTAL_P300K";
export type CreatureSourceForwardV1 =
  | "POSITIVE_Z"
  | "NEGATIVE_Z"
  | "POSITIVE_X"
  | "NEGATIVE_X";
export type SkinAccessoryStabilizationModeV1 =
  | "AUTO"
  | "KEEP_SOURCE_WEIGHTS"
  | "SELECT_BONE";

export interface TileAuthoringOptions {
  readonly terrainName: string;
  readonly surface: TileSurface;
  readonly interior: boolean;
}

export interface FileIdentity {
  sha256?: string | null;
}

export type FileIdentityValue = FileIdentity | string | null;

export interface SourceInputProps {
  target?: StudioTarget;
  tileTargetEnabled?: boolean;
  tileOptions?: TileAuthoringOptions;
  source?: File;
  appearance?: File;
  animationEvents?: File;
  creatureProfile?: CreatureConversionProfileV1;
  unsafeHighPolyInspection?: boolean;
  creatureSourceForward?: CreatureSourceForwardV1;
  textureArtifactCleanup?: boolean;
  experimentalAggressiveGeometryCleanup?: boolean;
  skinAccessoryStabilizationMode?: SkinAccessoryStabilizationModeV1;
  skinAccessorySelectedBoneName?: string;
  skinAccessoryComponentBoneOverrides?: string;
  creatureWeaponGrip?: CreatureWeaponGripOptionsV1;
  creatureHeldWeaponMode?: CreatureHeldWeaponModeV1;
  sourceIdentity?: FileIdentityValue;
  appearanceIdentity?: FileIdentityValue;
  sourceError?: string;
  appearanceError?: string;
  animationEventsError?: string;
  onSelectSource: (file: File) => void;
  onSelectAppearance: (file: File) => void;
  onSelectAnimationEvents: (file: File) => void;
  onCreatureProfileChange?: (profile: CreatureConversionProfileV1) => void;
  onUnsafeHighPolyInspectionChange?: (enabled: boolean) => void;
  onCreatureSourceForwardChange?: (sourceForward: CreatureSourceForwardV1) => void;
  onTextureArtifactCleanupChange?: (enabled: boolean) => void;
  onExperimentalAggressiveGeometryCleanupChange?: (enabled: boolean) => void;
  onSkinAccessoryStabilizationModeChange?: (
    mode: SkinAccessoryStabilizationModeV1,
  ) => void;
  onSkinAccessorySelectedBoneNameChange?: (boneName: string) => void;
  onSkinAccessoryComponentBoneOverridesChange?: (overrides: string) => void;
  onCreatureWeaponGripChange?: (weaponGrip: CreatureWeaponGripOptionsV1) => void;
  onCreatureHeldWeaponModeChange?: (mode: CreatureHeldWeaponModeV1) => void;
  onRemoveSource: () => void;
  onRemoveAppearance: () => void;
  onRemoveAnimationEvents: () => void;
  onClear: () => void;
  onTargetChange?: (target: StudioTarget) => void;
  onTileOptionsChange?: (options: TileAuthoringOptions) => void;
}

interface InputRowProps {
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
}

export function formatFileSize(size: number): string {
  if (size < 1_000) return `${size} B`;
  if (size < 1_000_000) return `${(size / 1_000).toFixed(size < 10_000 ? 1 : 0)} KB`;
  return `${(size / 1_000_000).toFixed(size < 10_000_000 ? 1 : 0)} MB`;
}

export function shortSha256(identity?: FileIdentityValue): string | undefined {
  const sha256 = typeof identity === "string" ? identity : identity?.sha256 ?? undefined;
  if (!sha256) return undefined;
  return sha256.length > 12 ? `${sha256.slice(0, 12)}...` : sha256;
}

function selectedFile(event: ChangeEvent<HTMLInputElement>, onSelect: (file: File) => void) {
  const file = event.currentTarget.files?.[0];
  if (file) onSelect(file);
}

function InputRow({
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
}: InputRowProps) {
  const detailsId = `${inputId}-details`;
  const errorId = `${inputId}-error`;
  const hash = shortSha256(identity);

  return (
    <li className="inputs-panel__item">
      <div className="inputs-panel__item-copy">
        <strong>{label}</strong>
        {file ? (
          <div id={detailsId} className="inputs-panel__file-details">
            <span>{file.name}</span>
            <span>{formatFileSize(file.size)}</span>
            {hash ? <code title={typeof identity === "string" ? identity : identity?.sha256 ?? undefined}>SHA-256 {hash}</code> : null}
            <span className="status-badge status-badge--neutral">Selected</span>
          </div>
        ) : (
          <span id={detailsId}>No file selected / {description}</span>
        )}
        {error ? <span id={errorId} role="alert">{error}</span> : null}
      </div>

      <div className="inputs-panel__item-actions">
        <label className="button button--secondary" htmlFor={inputId}>
          {file ? "Replace" : "Select"}
        </label>
        <input
          id={inputId}
          className="source-file-input"
          type="file"
          accept={accept}
          required={required}
          aria-describedby={`${detailsId}${error ? ` ${errorId}` : ""}`}
          aria-invalid={error ? true : undefined}
          onClick={(event) => { event.currentTarget.value = ""; }}
          onChange={(event) => selectedFile(event, onSelect)}
        />
        {file ? (
          <button type="button" className="button button--quiet" onClick={onRemove} aria-label={`Remove ${label}`}>
            Remove
          </button>
        ) : null}
      </div>
    </li>
  );
}

export function InputsPanel({
  target = "CREATURE",
  tileTargetEnabled = false,
  source,
  appearance,
  animationEvents,
  creatureProfile = "PRODUCT_300K",
  unsafeHighPolyInspection = false,
  creatureSourceForward = "POSITIVE_Z",
  textureArtifactCleanup = false,
  experimentalAggressiveGeometryCleanup = false,
  skinAccessoryStabilizationMode = "AUTO",
  skinAccessorySelectedBoneName = "",
  skinAccessoryComponentBoneOverrides = "",
  creatureWeaponGrip = defaultCreatureWeaponGripOptionsV1(),
  creatureHeldWeaponMode = "NONE",
  sourceIdentity,
  appearanceIdentity,
  sourceError,
  appearanceError,
  animationEventsError,
  onSelectSource,
  onSelectAppearance,
  onSelectAnimationEvents,
  onCreatureProfileChange,
  onUnsafeHighPolyInspectionChange,
  onCreatureSourceForwardChange,
  onTextureArtifactCleanupChange,
  onExperimentalAggressiveGeometryCleanupChange,
  onSkinAccessoryStabilizationModeChange,
  onSkinAccessorySelectedBoneNameChange,
  onSkinAccessoryComponentBoneOverridesChange,
  onCreatureWeaponGripChange,
  onCreatureHeldWeaponModeChange,
  onRemoveSource,
  onRemoveAppearance,
  onRemoveAnimationEvents,
  onClear,
  onTargetChange,
}: SourceInputProps) {
  const headingId = useId();
  const sourceInputId = useId();
  const appearanceInputId = useId();
  const animationEventsInputId = useId();
  const hasSelection = Boolean(source || appearance || animationEvents);

  return (
    <aside className="panel inputs-panel" aria-labelledby={headingId}>
      <header className="panel__header">
        <h2 id={headingId}>Inputs</h2>
        <button type="button" className="button button--quiet" onClick={onClear} disabled={!hasSelection}>
          Clear
        </button>
      </header>

      {onTargetChange ? (
        <label>
          Conversion target
          <select
            aria-label="Conversion target"
            value={target}
            onChange={(event) => onTargetChange(event.currentTarget.value as StudioTarget)}
          >
            <option value="CREATURE">Creature</option>
            <option value="PLACEABLE">Placeable</option>
            {tileTargetEnabled ? <option value="TILE">Tile</option> : null}
          </select>
        </label>
      ) : null}

      {target === "CREATURE" && onCreatureProfileChange ? (
        <>
          <label>
            Creature conversion profile
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
          {creatureProfile !== "PRODUCT_300K" ? (
            <p role="note">
              Material Separation is available only in the Product 300K profile.
            </p>
          ) : null}
          {creatureProfile === "PRODUCT_300K" && onUnsafeHighPolyInspectionChange ? (
            <>
              <label>
                <input
                  type="checkbox"
                  aria-label="Unsafe high-poly inspection"
                  checked={unsafeHighPolyInspection}
                  onChange={(event) => onUnsafeHighPolyInspectionChange(event.currentTarget.checked)}
                />
                Unsafe high-poly inspection
              </label>
              <p role="note">
                Local preview and diagnostics only. Product export remains blocked above 300,000
                triangles. This mode may use substantial memory.
              </p>
            </>
          ) : null}
          {onCreatureSourceForwardChange ? (
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
          ) : null}
          {onTextureArtifactCleanupChange ? (
            <label>
              <input
                type="checkbox"
                aria-label="Repair texture artifacts"
                checked={textureArtifactCleanup}
                onChange={(event) => onTextureArtifactCleanupChange(event.currentTarget.checked)}
              />
              Repair texture artifacts
            </label>
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
                      placeholder={"0:1=Spine02\n0:2=Spine"}
                    />
                  </label>
                </>
              ) : null}
            </>
          ) : null}
          {onCreatureWeaponGripChange ? (
            <WeaponGripControls value={creatureWeaponGrip} onChange={onCreatureWeaponGripChange} compact />
          ) : null}
          {onCreatureHeldWeaponModeChange ? (
            <HeldWeaponControls
              value={creatureHeldWeaponMode}
              onChange={onCreatureHeldWeaponModeChange}
              compact
            />
          ) : null}
        </>
      ) : null}

      {target === "PLACEABLE" && onExperimentalAggressiveGeometryCleanupChange ? (
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
      ) : null}

      <ul className="inputs-panel__list">
        <InputRow
          inputId={sourceInputId}
          label="Meshy GLB model"
          description="GLB required"
          accept=".glb,model/gltf-binary"
          file={source}
          identity={sourceIdentity}
          error={sourceError}
          onSelect={onSelectSource}
          onRemove={onRemoveSource}
        />
        {target !== "TILE" ? <InputRow
          inputId={appearanceInputId}
          label="Base model table"
          description="appearance.2da for creatures or placeables.2da for placeables"
          accept=".2da"
          file={appearance}
          identity={appearanceIdentity}
          error={appearanceError}
          onSelect={onSelectAppearance}
          onRemove={onRemoveAppearance}
        /> : null}
        {target === "CREATURE" ? <InputRow
          inputId={animationEventsInputId}
          label="Creature animation events"
          description="optional strict V1 JSON with caller-owned event timings for exact 42-state creatures"
          accept=".json,application/json"
          file={animationEvents}
          error={animationEventsError}
          onSelect={onSelectAnimationEvents}
          onRemove={onRemoveAnimationEvents}
          required={false}
        /> : null}
      </ul>

      {!hasSelection ? (
        <p className="inputs-panel__empty">
          No files selected yet. Add the required local {target === "TILE" ? "GLB" : "inputs"} to get started.
        </p>
      ) : null}
    </aside>
  );
}
