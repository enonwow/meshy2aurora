import { useId, type ChangeEvent } from "react";

export interface FileIdentity {
  sha256?: string | null;
}

export type FileIdentityValue = FileIdentity | string | null;

export interface SourceInputProps {
  source?: File;
  appearance?: File;
  sourceIdentity?: FileIdentityValue;
  appearanceIdentity?: FileIdentityValue;
  sourceError?: string;
  appearanceError?: string;
  onSelectSource: (file: File) => void;
  onSelectAppearance: (file: File) => void;
  onRemoveSource: () => void;
  onRemoveAppearance: () => void;
  onClear: () => void;
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
          required
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
  source,
  appearance,
  sourceIdentity,
  appearanceIdentity,
  sourceError,
  appearanceError,
  onSelectSource,
  onSelectAppearance,
  onRemoveSource,
  onRemoveAppearance,
  onClear,
}: SourceInputProps) {
  const headingId = useId();
  const sourceInputId = useId();
  const appearanceInputId = useId();
  const hasSelection = Boolean(source || appearance);

  return (
    <aside className="panel inputs-panel" aria-labelledby={headingId}>
      <header className="panel__header">
        <h2 id={headingId}>Inputs</h2>
        <button type="button" className="button button--quiet" onClick={onClear} disabled={!hasSelection}>
          Clear
        </button>
      </header>

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
        <InputRow
          inputId={appearanceInputId}
          label="Base appearance table"
          description="appearance.2da required"
          accept=".2da"
          file={appearance}
          identity={appearanceIdentity}
          error={appearanceError}
          onSelect={onSelectAppearance}
          onRemove={onRemoveAppearance}
        />
      </ul>

      {!hasSelection ? (
        <p className="inputs-panel__empty">No files selected yet. Add the two required local inputs to get started.</p>
      ) : null}
    </aside>
  );
}
