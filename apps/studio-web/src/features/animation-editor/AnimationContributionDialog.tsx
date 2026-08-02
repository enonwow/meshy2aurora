import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { getAnimationPresetCatalogV1 } from "../animation-library/catalog";
import type {
  AnimationContributionMetadataV1,
  AnimationPresetPlaybackV1,
} from "../animation-library/types";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";

export function AnimationContributionDialog({
  clip,
  onExport,
  onClose,
}: {
  clip: AuthoredAnimationClipV1;
  onExport: (
    clip: AuthoredAnimationClipV1,
    metadata: AnimationContributionMetadataV1,
  ) => Promise<unknown>;
  onClose: () => void;
}) {
  const availableTags = useMemo(() => [...new Set(
    getAnimationPresetCatalogV1().flatMap(({ tags }) => tags),
  )].sort(), []);
  const [presetId, setPresetId] = useState(`community_${clip.name}`.slice(0, 64));
  const [outputName, setOutputName] = useState(clip.name);
  const [label, setLabel] = useState(clip.name.replaceAll("_", " "));
  const [summary, setSummary] = useState("");
  const [author, setAuthor] = useState("");
  const [license, setLicense] = useState<AnimationContributionMetadataV1["license"]>("CC0-1.0");
  const [playback, setPlayback] = useState<AnimationPresetPlaybackV1>("ONE_SHOT");
  const [tags, setTags] = useState<readonly string[]>([]);
  const [rightsConfirmed, setRightsConfirmed] = useState(false);
  const [state, setState] = useState<"IDLE" | "EXPORTING">("IDLE");
  const [error, setError] = useState<string | null>(null);
  const firstInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    firstInputRef.current?.focus();
  }, []);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setState("EXPORTING");
    setError(null);
    try {
      const contribution = await onExport(clip, {
        presetId,
        presetVersion: 1,
        outputName,
        label,
        summary,
        authors: [{ name: author }],
        license,
        tags: [...tags].sort(),
        playback,
        rigProfile: "M2A_HUMANOID_STRICT_V1",
        validationStatus: "PIPELINE_VERIFIED",
      });
      downloadContribution(`${presetId}-v1.contribution.json`, contribution);
      onClose();
    } catch (caught: unknown) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setState("IDLE");
    }
  };

  return (
    <div className="animation-dialog-backdrop">
      <section
        className="animation-dialog animation-contribution-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="animation-contribution-title"
        onKeyDown={(event) => {
          if (event.key !== "Escape" || state === "EXPORTING") return;
          event.preventDefault();
          onClose();
        }}
      >
        <header>
          <div>
            <h2 id="animation-contribution-title">Export animation contribution</h2>
            <p>Creates portable motion JSON only. The source model is never included.</p>
          </div>
          <button type="button" onClick={onClose} aria-label="Close contribution export">×</button>
        </header>
        <form onSubmit={(event) => void submit(event)}>
          <div className="animation-contribution-dialog__grid">
            <label>Preset ID<input ref={firstInputRef} required pattern="[A-Za-z0-9_-]{1,64}" value={presetId} onChange={(event) => setPresetId(event.target.value)} /></label>
            <label>Output name<input required pattern="[A-Za-z0-9_]{1,16}" value={outputName} onChange={(event) => setOutputName(event.target.value)} /></label>
            <label>Label<input required maxLength={120} value={label} onChange={(event) => setLabel(event.target.value)} /></label>
            <label>Author<input required maxLength={120} value={author} onChange={(event) => setAuthor(event.target.value)} /></label>
            <label>License<select value={license} onChange={(event) => setLicense(event.target.value as AnimationContributionMetadataV1["license"])}><option>CC0-1.0</option><option>CC-BY-4.0</option><option>LicenseRef-Meshy2Aurora-Project-Generated</option></select></label>
            <label>Playback<select value={playback} onChange={(event) => setPlayback(event.target.value as AnimationPresetPlaybackV1)}><option value="ONE_SHOT">One shot</option><option value="LOOP">Loop</option></select></label>
          </div>
          <label>Summary<textarea required maxLength={500} value={summary} onChange={(event) => setSummary(event.target.value)} /></label>
          <fieldset>
            <legend>Tags</legend>
            <div className="animation-library-tags">
              {availableTags.map((tag) => (
                <button
                  type="button"
                  aria-pressed={tags.includes(tag)}
                  key={tag}
                  onClick={() => setTags((current) => current.includes(tag)
                    ? current.filter((value) => value !== tag)
                    : [...current, tag].sort())}
                >
                  {tag}
                </button>
              ))}
            </div>
          </fieldset>
          <label className="animation-contribution-dialog__rights">
            <input type="checkbox" required checked={rightsConfirmed} onChange={(event) => setRightsConfirmed(event.target.checked)} />
            I created this motion or have explicit rights to redistribute it under the selected license.
          </label>
          <p className="animation-contribution-dialog__notice">
            External public PRs remain closed until the repository owner records the root LICENSE decision.
          </p>
          {error ? <p role="alert" className="animation-library-preset-details__error">{error}</p> : null}
          <footer>
            <button type="button" onClick={onClose}>Cancel</button>
            <button type="submit" disabled={state === "EXPORTING" || tags.length === 0 || !rightsConfirmed}>
              {state === "EXPORTING" ? "Validating in Core…" : "Download contribution JSON"}
            </button>
          </footer>
        </form>
      </section>
    </div>
  );
}

function downloadContribution(fileName: string, contribution: unknown) {
  const blob = new Blob([`${JSON.stringify(contribution, null, 2)}\n`], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName;
  anchor.click();
  queueMicrotask(() => URL.revokeObjectURL(url));
}
