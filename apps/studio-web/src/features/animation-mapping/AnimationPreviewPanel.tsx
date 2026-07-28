import { useMemo, useState } from "react";
import { AuroraReadbackViewport } from "../preview/AuroraReadbackViewport";
import { SourceViewport } from "../preview/SourceViewport";
import type {
  BinaryMdlInspectionReport,
  SourcePreviewInput,
} from "../preview/types";
import {
  compareAnimationPreviewTimingsV1,
  createAnimationPreviewSelectionV1,
  getAnimationPreviewDiagnosticsV1,
  loadReadbackAnimationPreviewV1,
  loadSourceAnimationPreviewV1,
} from "./preview";
import type {
  AnimationCatalogRowV1,
  CreatureAnimationAuthoringV1,
  CreatureAnimationInspectionV1,
} from "./types";

interface Props {
  row: AnimationCatalogRowV1 | null;
  authoring: CreatureAnimationAuthoringV1;
  inspection: CreatureAnimationInspectionV1;
  sourceInput?: SourcePreviewInput;
  readback?: BinaryMdlInspectionReport;
  builtAuthoringRevision?: number | null;
  onError?: (message: string) => void;
}

export function AnimationPreviewPanel({
  row,
  authoring,
  inspection,
  sourceInput,
  readback,
  builtAuthoringRevision,
  onError,
}: Props) {
  const [mode, setMode] = useState<"SOURCE" | "READBACK">("SOURCE");
  const selection = useMemo(
    () => row ? createAnimationPreviewSelectionV1(row, authoring) : null,
    [authoring, row],
  );
  const sourceClip = selection
    ? loadSourceAnimationPreviewV1(selection, inspection.sourceClips)
    : null;
  const readbackAnimation = selection
    ? loadReadbackAnimationPreviewV1(readback, selection.resultClipName)
    : null;
  const diagnostics = selection
    ? getAnimationPreviewDiagnosticsV1(selection, {
        sourceClip,
        readbackAnimation,
        builtAuthoringRevision,
        currentAuthoringRevision: authoring.authoringRevision,
      })
    : [];
  const timings = sourceClip && readbackAnimation
    ? compareAnimationPreviewTimingsV1(sourceClip, readbackAnimation)
    : null;
  const canShowSource = Boolean(
    sourceInput
    && selection?.source?.sourceKind === "SOURCE_CLIP"
    && sourceClip,
  );
  const canShowReadback = Boolean(readback && readbackAnimation);

  return (
    <aside className="animation-preview-panel" aria-label="Animation preview">
      <header>
        <span>{mode === "SOURCE" ? "Source preview" : "Built readback preview"}</span>
        <strong>{mode === "SOURCE"
          ? selection?.sourceClipName ?? "No source clip selected"
          : selection?.resultClipName ?? "No result clip selected"}</strong>
        <div role="tablist" aria-label="Animation preview provenance">
          <button
            type="button"
            role="tab"
            aria-selected={mode === "SOURCE"}
            onClick={() => setMode("SOURCE")}
          >
            Source
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={mode === "READBACK"}
            onClick={() => setMode("READBACK")}
          >
            Built readback
          </button>
        </div>
      </header>

      {mode === "SOURCE" && canShowSource && sourceInput ? (
        <SourceViewport
          key={`source:${selection?.rowId}:${selection?.sourceClipName}`}
          input={sourceInput}
          initialAnimationName={selection?.sourceClipName}
          onError={onError}
        />
      ) : mode === "READBACK" && canShowReadback && readback ? (
        <AuroraReadbackViewport
          key={`readback:${selection?.rowId}:${selection?.resultClipName}`}
          report={readback}
          initialAnimationName={selection?.resultClipName}
          onSelectPart={() => undefined}
          onError={onError}
        />
      ) : (
        <div
          className="animation-preview-panel__viewport"
          role="status"
          aria-label="Animation preview unavailable"
        >
          <span>
            {mode === "SOURCE"
              ? selection?.source?.sourceKind === "PROCEDURAL"
                ? "This slot is generated during build; it has no source GLB clip."
                : "Select a compatible source clip to preview it."
              : "Build the current mapping to preview its canonical binary MDL readback."}
          </span>
        </div>
      )}

      {timings ? (
        <dl className="animation-preview-panel__timings">
          <div><dt>Source</dt><dd>{timings.sourceDurationSeconds.toFixed(2)} s</dd></div>
          <div><dt>Readback</dt><dd>{timings.resultDurationSeconds.toFixed(2)} s</dd></div>
          <div><dt>Delta</dt><dd>{timings.durationDeltaSeconds.toFixed(2)} s</dd></div>
          <div><dt>Transtime</dt><dd>{timings.resultTransitionSeconds.toFixed(2)} s</dd></div>
        </dl>
      ) : null}

      {diagnostics.length > 0 ? (
        <ul className="animation-preview-panel__diagnostics" aria-label="Preview limitations">
          {diagnostics.map(({ code, message }) => (
            <li key={code}><strong>{code}</strong> {message}</li>
          ))}
        </ul>
      ) : null}
      <p>Preview is local inspection evidence, not proof of behavior in Aurora Toolset or NWN.</p>
    </aside>
  );
}
