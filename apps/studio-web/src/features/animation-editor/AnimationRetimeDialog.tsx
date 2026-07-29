import { useState } from "react";

export function AnimationRetimeDialog({
  lengthSeconds,
  onApply,
  onClose,
}: {
  lengthSeconds: number;
  onApply: (length: number) => void;
  onClose: () => void;
}) {
  const [length, setLength] = useState(lengthSeconds);
  return (
    <div
      className="animation-clip-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="retime-dialog-title"
      aria-describedby="retime-dialog-description"
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          onClose();
        }
      }}
    >
      <h3 id="retime-dialog-title">Retime animation</h3>
      <label>
        New length (s)
        <input
          autoFocus
          type="number"
          min={0.001}
          step={0.001}
          value={length}
          onChange={(event) => setLength(event.currentTarget.valueAsNumber)}
        />
      </label>
      <p id="retime-dialog-description">Keyframes and events are scaled by the same factor.</p>
      <button
        type="button"
        disabled={!Number.isFinite(length) || length <= 0}
        onClick={() => onApply(length)}
      >
        Apply retime
      </button>
      <button type="button" onClick={onClose}>Cancel</button>
    </div>
  );
}
