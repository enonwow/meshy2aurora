import { useEffect, useState } from "react";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import type { AnimationLayerBakeReportV1, AnimationLayerModeV1 } from "./animationLayers";

interface Props {
  readonly clips: readonly AuthoredAnimationClipV1[];
  readonly selectedClip: AuthoredAnimationClipV1 | null;
  readonly selectedBoneId: number | null;
  readonly busy: boolean;
  readonly error: string | null;
  readonly report: AnimationLayerBakeReportV1 | null;
  readonly onBake: (input: {
    readonly layerClipId: string;
    readonly mode: Exclude<AnimationLayerModeV1, "BASE">;
    readonly weight: number;
    readonly selectedBoneOnly: boolean;
  }) => void;
}

export function AnimationLayerPanel({
  clips,
  selectedClip,
  selectedBoneId,
  busy,
  error,
  report,
  onBake,
}: Props) {
  const available = clips.filter(({ id }) => id !== selectedClip?.id);
  const [layerClipId, setLayerClipId] = useState(available[0]?.id ?? "");
  const [mode, setMode] = useState<"ADDITIVE" | "OVERRIDE">("ADDITIVE");
  const [weight, setWeight] = useState(1);
  const [selectedBoneOnly, setSelectedBoneOnly] = useState(false);
  useEffect(() => {
    if (!available.some(({ id }) => id === layerClipId)) {
      setLayerClipId(available[0]?.id ?? "");
    }
  }, [available, layerClipId]);

  return (
    <section className="animation-layer-panel" aria-labelledby="animation-layer-title">
      <h3 id="animation-layer-title">Correction layer</h3>
      <p>Bake one masked Additive or Override layer into the selected clip.</p>
      <label>
        Layer clip
        <select
          value={layerClipId}
          disabled={busy || available.length === 0}
          onChange={(event) => setLayerClipId(event.currentTarget.value)}
        >
          {available.map((clip) => <option key={clip.id} value={clip.id}>{clip.name}</option>)}
        </select>
      </label>
      <label>
        Mode
        <select
          value={mode}
          disabled={busy}
          onChange={(event) => setMode(event.currentTarget.value as typeof mode)}
        >
          <option value="ADDITIVE">Additive</option>
          <option value="OVERRIDE">Override</option>
        </select>
      </label>
      <label>
        Weight {weight.toFixed(2)}
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={weight}
          disabled={busy}
          onChange={(event) => setWeight(event.currentTarget.valueAsNumber)}
        />
      </label>
      <label>
        <input
          type="checkbox"
          checked={selectedBoneOnly}
          disabled={busy || selectedBoneId === null}
          onChange={(event) => setSelectedBoneOnly(event.currentTarget.checked)}
        />
        Mask to selected bone
      </label>
      <button
        type="button"
        disabled={busy || !selectedClip || !layerClipId}
        onClick={() => onBake({ layerClipId, mode, weight, selectedBoneOnly })}
      >
        {busy ? "Baking layers…" : "Bake correction layer"}
      </button>
      {report ? (
        <p role="status">{report.keyCountBefore} → {report.keyCountAfter} keys baked</p>
      ) : null}
      {error ? <p role="alert">{error}</p> : null}
    </section>
  );
}
