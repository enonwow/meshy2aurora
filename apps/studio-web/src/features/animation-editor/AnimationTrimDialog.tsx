import { useState } from "react";

export function AnimationTrimDialog({
  lengthSeconds,
  onApply,
  onClose,
}: {
  lengthSeconds: number;
  onApply: (start: number, end: number) => void;
  onClose: () => void;
}) {
  const [start, setStart] = useState(0);
  const [end, setEnd] = useState(lengthSeconds);
  return (
    <div
      className="animation-clip-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="trim-dialog-title"
      aria-describedby="trim-dialog-description"
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          onClose();
        }
      }}
    >
      <h3 id="trim-dialog-title">Trim animation</h3>
      <label>Start (s)<input autoFocus type="number" min={0} max={end} step={0.001} value={start} onChange={(event) => setStart(event.currentTarget.valueAsNumber)} /></label>
      <label>End (s)<input type="number" min={start} max={lengthSeconds} step={0.001} value={end} onChange={(event) => setEnd(event.currentTarget.valueAsNumber)} /></label>
      <p id="trim-dialog-description">Boundary poses are sampled linearly. Events outside the range are removed.</p>
      <button
        type="button"
        disabled={!Number.isFinite(start) || !Number.isFinite(end) || start < 0 || start >= end || end > lengthSeconds}
        onClick={() => onApply(start, end)}
      >
        Apply trim
      </button>
      <button type="button" onClick={onClose}>Cancel</button>
    </div>
  );
}
