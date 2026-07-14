import { useId, type DragEvent } from "react";
import {
  formatFileSize,
  shortSha256,
  type FileIdentityValue,
  type SourceInputProps,
} from "./InputsPanel";

export interface SourceStepProps extends SourceInputProps {
  onContinue: () => void;
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
            <strong>Drag &amp; drop {label === "Meshy model" ? "a GLB file" : "an appearance.2da file"} here</strong>
            <span>or click to browse</span>
          </span>
        )}
      </label>

      <input
        id={inputId}
        className="source-file-input"
        type="file"
        accept={accept}
        required
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
  onContinue,
}: SourceStepProps) {
  const headingId = useId();
  const sourceInputId = useId();
  const appearanceInputId = useId();
  const ready = Boolean(source && appearance && !sourceError && !appearanceError);
  const hasSelection = Boolean(source || appearance);
  const readinessMessage = ready
    ? "Both required files are selected. Continue to inspect their contents."
    : source
      ? "Select the base appearance.2da file to continue."
      : appearance
        ? "Select a Meshy GLB model to continue."
        : "Select both required files to continue.";

  return (
    <section className="source-step" aria-labelledby={headingId}>
      <header className="source-step__intro">
        <h1 id={headingId}>Start a new conversion</h1>
        <p>Select a Meshy GLB model and the base appearance.2da file.</p>
        <p className="source-step__privacy">All processing stays in your browser. Files are not uploaded.</p>
      </header>

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
        <DropZone
          inputId={appearanceInputId}
          label="Base appearance table"
          description="Required file: appearance.2da"
          accept=".2da"
          file={appearance}
          identity={appearanceIdentity}
          error={appearanceError}
          onSelect={onSelectAppearance}
          onRemove={onRemoveAppearance}
          selectLabel="Select appearance.2da"
        />
      </div>

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
