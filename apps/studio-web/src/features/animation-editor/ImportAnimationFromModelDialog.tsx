import { useEffect, useId, useRef, useState } from "react";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import {
  compareAnimationImportRigsV1,
  type ExternalAnimationSourceInspectionV1,
} from "./animationImport";
import "./animation-library.base.css";

export function ImportAnimationFromModelDialog({
  currentRig,
  onInspect,
  onImport,
  onClose,
}: {
  currentRig: readonly AnimationRigNodeV1[];
  onInspect: (file: File) => Promise<ExternalAnimationSourceInspectionV1>;
  onImport: (file: File, clipName: string) => Promise<void>;
  onClose: () => void;
}) {
  const titleId = useId();
  const descriptionId = useId();
  const generationRef = useRef(0);
  const cardRef = useRef<HTMLDivElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [inspection, setInspection] =
    useState<ExternalAnimationSourceInspectionV1 | null>(null);
  const [selectedClipName, setSelectedClipName] = useState("");
  const [busy, setBusy] = useState<"INSPECTING" | "IMPORTING" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const compatibility = inspection
    ? compareAnimationImportRigsV1(currentRig, inspection.rig)
    : null;

  useEffect(() => () => {
    generationRef.current += 1;
  }, []);

  const inspect = async (nextFile: File) => {
    const generation = generationRef.current + 1;
    generationRef.current = generation;
    setFile(nextFile);
    setInspection(null);
    setSelectedClipName("");
    setError(null);
    setBusy("INSPECTING");
    try {
      const result = await onInspect(nextFile);
      if (generationRef.current !== generation) return;
      setInspection(result);
      setSelectedClipName(result.clips[0]?.name ?? "");
      if (result.clips.length === 0) {
        setError("This GLB contains no animation clips.");
      }
    } catch (cause: unknown) {
      if (generationRef.current !== generation) return;
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (generationRef.current === generation) setBusy(null);
    }
  };

  const importSelected = async () => {
    if (!file || !selectedClipName || !compatibility?.compatible) return;
    setError(null);
    setBusy("IMPORTING");
    try {
      await onImport(file, selectedClipName);
      onClose();
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
      setBusy(null);
    }
  };

  return (
    <div
      className="animation-import-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
      onKeyDown={(event) => {
        if (event.key === "Escape" && busy === null) {
          event.preventDefault();
          onClose();
          return;
        }
        if (event.key === "Tab") {
          const focusable = Array.from(
            cardRef.current?.querySelectorAll<HTMLElement>(
              "button:not(:disabled), input:not(:disabled), select:not(:disabled)",
            ) ?? [],
          );
          if (focusable.length === 0) return;
          const first = focusable[0]!;
          const last = focusable[focusable.length - 1]!;
          if (event.shiftKey && globalThis.document.activeElement === first) {
            event.preventDefault();
            last.focus();
          } else if (!event.shiftKey && globalThis.document.activeElement === last) {
            event.preventDefault();
            first.focus();
          }
        }
      }}
    >
      <div ref={cardRef} className="animation-import-dialog__card">
        <header>
          <div>
            <span className="animation-import-dialog__eyebrow">Custom library</span>
            <h3 id={titleId}>Copy animation from another model</h3>
          </div>
          <button
            type="button"
            aria-label="Close animation import"
            disabled={busy !== null}
            onClick={onClose}
          >
            ×
          </button>
        </header>

        <p id={descriptionId}>
          Choose a local animated GLB. The model is inspected in memory; only the
          selected animation tracks and their provenance are stored in this project.
        </p>

        <label className="animation-import-dialog__file">
          <span>Donor model (.glb)</span>
          <input
            autoFocus
            type="file"
            accept=".glb,model/gltf-binary"
            disabled={busy !== null}
            onChange={(event) => {
              const nextFile = event.currentTarget.files?.[0];
              if (nextFile) void inspect(nextFile);
            }}
          />
          <small>{file?.name ?? "No donor model selected"}</small>
        </label>

        {busy === "INSPECTING" ? (
          <p className="animation-import-dialog__status" role="status">
            Inspecting donor model…
          </p>
        ) : null}

        {inspection ? (
          <section className="animation-import-dialog__inspection">
            <div
              className="animation-import-dialog__compatibility"
              data-compatible={compatibility?.compatible}
              role="status"
            >
              <strong>
                {compatibility?.compatible ? "Compatible rig" : "Different rig"}
              </strong>
              <span>{compatibility?.message}</span>
              {!compatibility?.compatible && compatibility?.mismatches[0] ? (
                <small>{compatibility.mismatches[0]}</small>
              ) : null}
            </div>

            <label>
              Animation clip
              <select
                value={selectedClipName}
                disabled={busy !== null || inspection.clips.length === 0}
                onChange={(event) => setSelectedClipName(event.currentTarget.value)}
              >
                {inspection.clips.map((clip) => (
                  <option key={clip.name} value={clip.name}>
                    {clip.name} · {clip.durationSeconds.toFixed(2)} s · {clip.trackCount} tracks
                  </option>
                ))}
              </select>
            </label>

            <dl>
              <div><dt>Animations</dt><dd>{inspection.clips.length}</dd></div>
              <div><dt>Bones</dt><dd>{inspection.rig.length}</dd></div>
              <div>
                <dt>Donor SHA-256</dt>
                <dd title={inspection.sourceRevision}>
                  {inspection.sourceRevision.slice(0, 12)}…
                </dd>
              </div>
            </dl>
          </section>
        ) : null}

        {error ? (
          <p className="animation-import-dialog__error" role="alert">{error}</p>
        ) : null}

        <footer>
          <button type="button" disabled={busy !== null} onClick={onClose}>
            Cancel
          </button>
          <button
            className="animation-studio-action animation-studio-action--primary"
            type="button"
            disabled={
              busy !== null
              || !file
              || !selectedClipName
              || !compatibility?.compatible
            }
            onClick={() => void importSelected()}
          >
            {busy === "IMPORTING" ? "Copying…" : "Copy to Custom"}
          </button>
        </footer>
      </div>
    </div>
  );
}
