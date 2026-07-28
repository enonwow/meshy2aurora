import type { ReactNode } from "react";

export function AnimationEditorViewport({
  viewport,
  selectedBoneName,
  playheadSeconds,
  path,
  onGestureBegin,
  onGestureUpdate,
  onGestureCommit,
  onGestureCancel,
}: {
  viewport: ReactNode;
  selectedBoneName: string | null;
  playheadSeconds: number;
  path: "ROTATION" | "TRANSLATION";
  onGestureBegin: () => void;
  onGestureUpdate: (delta: number) => void;
  onGestureCommit: () => void;
  onGestureCancel: () => void;
}) {
  return (
    <section className="animation-editor-viewport" aria-label="Edited result viewport">
      <header>
        <strong>Edited result</strong>
        <span>
          {selectedBoneName ?? "No bone selected"} · {playheadSeconds.toFixed(2)} s
        </span>
      </header>
      <div className="animation-editor-viewport__canvas">
        {viewport}
        {selectedBoneName ? (
          <div
            className="animation-editor-gizmo"
            aria-label={`${path.toLocaleLowerCase("en-US")} gizmo for ${selectedBoneName}`}
          >
            <label>
              <span>Gizmo delta</span>
              <input
                type="range"
                aria-label={`${path === "ROTATION" ? "Rotation" : "Translation"} gizmo delta for ${selectedBoneName}`}
                aria-describedby="animation-gizmo-keyboard-help"
                min={path === "ROTATION" ? -180 : -1}
                max={path === "ROTATION" ? 180 : 1}
                step={path === "ROTATION" ? 1 : 0.01}
                defaultValue={0}
                onPointerDown={onGestureBegin}
                onInput={(event) => onGestureUpdate(event.currentTarget.valueAsNumber)}
                onPointerUp={onGestureCommit}
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    event.preventDefault();
                    onGestureCancel();
                  } else if (isRangeAdjustmentKeyV1(event.key)) {
                    onGestureBegin();
                  }
                }}
                onKeyUp={(event) => {
                  if (isRangeAdjustmentKeyV1(event.key)) onGestureCommit();
                }}
              />
            </label>
            <p id="animation-gizmo-keyboard-help" className="sr-only">
              Use Arrow, Page Up, Page Down, Home or End to preview and commit
              a transform. Press Escape to cancel. Equivalent numeric fields
              are available in Bone and clip.
            </p>
          </div>
        ) : null}
      </div>
    </section>
  );
}

function isRangeAdjustmentKeyV1(key: string) {
  return [
    "ArrowLeft",
    "ArrowRight",
    "ArrowUp",
    "ArrowDown",
    "PageUp",
    "PageDown",
    "Home",
    "End",
  ].includes(key);
}
