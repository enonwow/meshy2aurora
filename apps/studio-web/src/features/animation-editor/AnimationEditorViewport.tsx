import { useRef, useState, type ReactNode } from "react";

export function AnimationEditorViewport({
  viewport,
  sourceViewport,
  selectedBoneName,
  playheadSeconds,
  path,
  onGestureBegin,
  onGestureUpdate,
  onGestureCommit,
  onGestureCancel,
}: {
  viewport: ReactNode;
  sourceViewport?: ReactNode;
  selectedBoneName: string | null;
  playheadSeconds: number;
  path: "ROTATION" | "TRANSLATION";
  onGestureBegin: () => void;
  onGestureUpdate: (delta: number) => void;
  onGestureCommit: () => void;
  onGestureCancel: () => void;
}) {
  const [display, setDisplay] = useState<"SOURCE" | "EDITED">("EDITED");
  const viewportRef = useRef<HTMLDivElement>(null);
  const showingSource = display === "SOURCE" && sourceViewport !== undefined;

  return (
    <section className="animation-editor-viewport" aria-label="Edited result viewport">
      <header>
        <strong>Edited result</strong>
        <div
          className="animation-editor-viewport__toolbar"
          role="toolbar"
          aria-label="Animation viewport display"
        >
          <span role="tablist" aria-label="Animation comparison">
            <button
              type="button"
              role="tab"
              aria-label="Show source animation"
              aria-selected={showingSource}
              disabled={sourceViewport === undefined}
              onClick={() => setDisplay("SOURCE")}
            >
              Source
            </button>
            <button
              type="button"
              role="tab"
              aria-label="Show edited animation"
              aria-selected={!showingSource}
              onClick={() => setDisplay("EDITED")}
            >
              Edited
            </button>
          </span>
          <button
            type="button"
            aria-label="Toggle animation viewport fullscreen"
            onClick={() => void toggleFullscreen(viewportRef.current)}
          >
            Fullscreen
          </button>
        </div>
        <span>
          {selectedBoneName ?? "No bone selected"}
          {" \u00b7 "}
          {playheadSeconds.toFixed(2)} s
        </span>
      </header>
      <div
        className="animation-editor-viewport__canvas"
        data-animation-result={showingSource ? "SOURCE" : "EDITED"}
        ref={viewportRef}
      >
        {showingSource ? sourceViewport : viewport}
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

async function toggleFullscreen(target: HTMLElement | null) {
  if (!target) return;
  if (document.fullscreenElement) {
    await document.exitFullscreen();
    return;
  }
  await target.requestFullscreen();
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
