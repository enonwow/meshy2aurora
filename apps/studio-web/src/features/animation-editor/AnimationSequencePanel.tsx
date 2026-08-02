import { useEffect, useMemo, useState } from "react";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import type { AnimationSequencePreviewV1 } from "./animationSequence";

interface Props {
  readonly clips: readonly AuthoredAnimationClipV1[];
  readonly selectedClipId: string | null;
  readonly preview: AnimationSequencePreviewV1 | null;
  readonly busy: boolean;
  readonly error: string | null;
  readonly onBuild: (clipIds: readonly string[]) => void;
  readonly onClear: () => void;
}

export function AnimationSequencePanel({
  clips,
  selectedClipId,
  preview,
  busy,
  error,
  onBuild,
  onClear,
}: Props) {
  const idleCandidates = useMemo(() => clips.filter((clip) => (
    /(?:idle|ready|rest)/iu.test(clip.name)
  )), [clips]);
  const defaultIdle = idleCandidates[0]?.id ?? clips[0]?.id ?? "";
  const [idleClipId, setIdleClipId] = useState(defaultIdle);
  const [actionClipId, setActionClipId] = useState(selectedClipId ?? clips[0]?.id ?? "");

  useEffect(() => {
    if (selectedClipId && clips.some(({ id }) => id === selectedClipId)) {
      setActionClipId(selectedClipId);
    }
  }, [clips, selectedClipId]);
  useEffect(() => {
    if (!clips.some(({ id }) => id === idleClipId)) setIdleClipId(defaultIdle);
  }, [clips, defaultIdle, idleClipId]);

  return (
    <section className="animation-sequence-panel" aria-labelledby="sequence-preview-title">
      <header>
        <div>
          <h3 id="sequence-preview-title">Sequence preview</h3>
          <small>Preview idle → action → idle without changing the project.</small>
        </div>
        {preview ? <button type="button" onClick={onClear}>Stop sequence</button> : null}
      </header>
      <label>
        Idle clip
        <select
          value={idleClipId}
          disabled={busy || clips.length === 0}
          onChange={(event) => setIdleClipId(event.currentTarget.value)}
        >
          {clips.map((clip) => <option key={clip.id} value={clip.id}>{clip.name}</option>)}
        </select>
      </label>
      <label>
        Action clip
        <select
          value={actionClipId}
          disabled={busy || clips.length === 0}
          onChange={(event) => setActionClipId(event.currentTarget.value)}
        >
          {clips.map((clip) => <option key={clip.id} value={clip.id}>{clip.name}</option>)}
        </select>
      </label>
      <button
        type="button"
        disabled={busy || !idleClipId || !actionClipId || clips.length < 2}
        onClick={() => onBuild([idleClipId, actionClipId, idleClipId])}
      >
        {busy ? "Building sequence…" : "Preview idle → action → idle"}
      </button>
      {preview ? (
        <div role="status">
          <strong>{preview.previewClip.lengthSeconds.toFixed(2)} s sequence ready</strong>
          <small>
            {preview.segments.map(({ clipName }) => clipName).join(" → ")}
          </small>
          <ul>
            {preview.transitionJumps.map((jump) => (
              <li key={`${jump.fromClipId}:${jump.toClipId}:${jump.boundarySeconds}`}>
                t={jump.boundarySeconds.toFixed(2)} s · Δp {jump.maxTranslationDelta.toFixed(4)} · Δr {jump.maxAngularDeltaRadians.toFixed(4)} rad
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      {error ? <p role="alert">{error}</p> : null}
    </section>
  );
}
