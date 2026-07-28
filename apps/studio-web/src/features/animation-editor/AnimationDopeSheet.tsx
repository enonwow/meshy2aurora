import { useEffect, useMemo, useRef, useState } from "react";
import {
  ANIMATION_STUDIO_PRODUCT_LIMITS_V1,
  type AuthoredAnimationClipV1,
} from "../animation-studio";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import { AnimationEventTrack } from "./AnimationEventTrack";
import { AnimationTimelineRuler } from "./AnimationTimelineRuler";
import { AnimationTrackRow } from "./AnimationTrackRow";
import { AnimationTransportControls } from "./AnimationTransportControls";
import {
  animationKeySelectionIdV1,
  panAnimationTimelineV1,
  snapAnimationTimeV1,
  zoomAnimationTimelineV1,
} from "./editing";

export function AnimationDopeSheet({
  clip,
  rig,
  playheadSeconds,
  playing,
  selectedKeyIds,
  onPlayingChange,
  onSeek,
  onSelectKey,
  onSelectEvent,
  onAddEvent,
  onDeleteKeys,
  onMoveSelectedKeys,
  onSelectRange,
  onOpenTrim,
  onOpenRetime,
}: {
  clip: AuthoredAnimationClipV1;
  rig: readonly AnimationRigNodeV1[];
  playheadSeconds: number;
  playing: boolean;
  selectedKeyIds: ReadonlySet<string>;
  onPlayingChange: (playing: boolean) => void;
  onSeek: (time: number) => void;
  onSelectKey: (id: string, additive: boolean) => void;
  onSelectEvent: (id: string) => void;
  onAddEvent: () => void;
  onDeleteKeys: () => void;
  onMoveSelectedKeys: (deltaSeconds: number) => void;
  onSelectRange: (startSeconds: number, endSeconds: number) => void;
  onOpenTrim: () => void;
  onOpenRetime: () => void;
}) {
  const [snap, setSnap] = useState<"NONE" | "FRAME_30">("FRAME_30");
  const [rangeStart, setRangeStart] = useState(0);
  const [rangeEnd, setRangeEnd] = useState(clip.lengthSeconds);
  const [view, setView] = useState({
    pixelsPerSecond: 100,
    offsetSeconds: 0,
  });
  const timelineViewportRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    setRangeStart(0);
    setRangeEnd(clip.lengthSeconds);
    setView((current) => ({
      ...current,
      offsetSeconds: Math.min(current.offsetSeconds, clip.lengthSeconds),
    }));
  }, [clip.id, clip.lengthSeconds]);
  useEffect(() => {
    if (timelineViewportRef.current) {
      timelineViewportRef.current.scrollLeft =
        view.offsetSeconds * view.pixelsPerSecond;
    }
  }, [view]);
  const keyTimes = [
    ...new Set(clip.tracks.flatMap((track) => (
      track.keyframes.map(({ timeSeconds }) => timeSeconds)
    ))),
  ].sort((left, right) => left - right);
  const markerProjection = useMemo(() => {
    const candidates = [
      ...clip.tracks.flatMap((track) => track.keyframes.map((keyframe) => ({
        id: animationKeySelectionIdV1(track.id, keyframe.id),
        kind: "KEY" as const,
        timeSeconds: keyframe.timeSeconds,
      }))),
      ...clip.events.map((event) => ({
        id: event.id,
        kind: "EVENT" as const,
        timeSeconds: event.timeSeconds,
      })),
    ];
    const rendered = candidates
      .sort((left, right) => {
        const leftSelected = left.kind === "KEY" && selectedKeyIds.has(left.id);
        const rightSelected = right.kind === "KEY" && selectedKeyIds.has(right.id);
        if (leftSelected !== rightSelected) return leftSelected ? -1 : 1;
        return Math.abs(left.timeSeconds - playheadSeconds)
          - Math.abs(right.timeSeconds - playheadSeconds)
          || left.timeSeconds - right.timeSeconds
          || left.id.localeCompare(right.id);
      })
      .slice(0, ANIMATION_STUDIO_PRODUCT_LIMITS_V1.maxTimelineDomMarkers);
    return {
      total: candidates.length,
      rendered: rendered.length,
      keyIds: new Set(
        rendered.filter(({ kind }) => kind === "KEY").map(({ id }) => id),
      ),
      eventIds: new Set(
        rendered.filter(({ kind }) => kind === "EVENT").map(({ id }) => id),
      ),
    };
  }, [clip.events, clip.tracks, playheadSeconds, selectedKeyIds]);
  const visibleTracks = useMemo(() => clip.tracks.map((track) => ({
    ...track,
    keyframes: track.keyframes.filter(({ id }) => markerProjection.keyIds.has(
      animationKeySelectionIdV1(track.id, id),
    )),
  })), [clip.tracks, markerProjection.keyIds]);
  const visibleEvents = useMemo(() => clip.events.filter(
    ({ id }) => markerProjection.eventIds.has(id),
  ), [clip.events, markerProjection.eventIds]);
  const previousKey = () => onSeek(
    [...keyTimes].reverse().find((time) => time < playheadSeconds - 1e-7) ?? 0,
  );
  const nextKey = () => onSeek(
    keyTimes.find((time) => time > playheadSeconds + 1e-7) ?? clip.lengthSeconds,
  );
  const moveStep = snap === "FRAME_30" ? 1 / 30 : 0.001;
  const panStep = Math.max(moveStep, Math.min(0.25, clip.lengthSeconds / 4));
  const moveSelection = (direction: -1 | 1) => {
    if (selectedKeyIds.size > 0) {
      onMoveSelectedKeys(snapAnimationTimeV1(
        direction * moveStep,
        snap,
      ));
    }
  };
  const zoom = (direction: -1 | 1) => setView((current) => (
    zoomAnimationTimelineV1(current, direction)
  ));
  const pan = (direction: -1 | 1) => setView((current) => {
    const next = panAnimationTimelineV1(current, direction * panStep);
    return {
      ...next,
      offsetSeconds: Math.min(next.offsetSeconds, clip.lengthSeconds),
    };
  });
  const timeAreaWidth = Math.max(
    480,
    clip.lengthSeconds * view.pixelsPerSecond,
  );
  return (
    <section
      className="animation-dope-sheet"
      aria-labelledby="dope-sheet-title"
      aria-describedby="animation-timeline-keyboard-help"
      tabIndex={0}
      onKeyDown={(event) => {
        if (isEditableTargetV1(event.target)) return;
        if ((event.key === "Delete" || event.key === "Backspace") && selectedKeyIds.size > 0) {
          event.preventDefault();
          onDeleteKeys();
        } else if (
          event.altKey
          && !event.shiftKey
          && (event.key === "ArrowLeft" || event.key === "ArrowRight")
          && selectedKeyIds.size > 0
        ) {
          event.preventDefault();
          moveSelection(event.key === "ArrowLeft" ? -1 : 1);
        } else if (
          event.shiftKey
          && !event.altKey
          && (event.key === "ArrowLeft" || event.key === "ArrowRight")
        ) {
          event.preventDefault();
          pan(event.key === "ArrowLeft" ? -1 : 1);
        } else if (event.key === "+" || event.key === "=") {
          event.preventDefault();
          zoom(1);
        } else if (event.key === "-") {
          event.preventDefault();
          zoom(-1);
        }
      }}
    >
      <header>
        <h2 id="dope-sheet-title">Dope sheet</h2>
        <AnimationTransportControls
          playing={playing}
          playheadSeconds={playheadSeconds}
          lengthSeconds={clip.lengthSeconds}
          previousKey={previousKey}
          nextKey={nextKey}
          onPlayingChange={onPlayingChange}
          onSeek={onSeek}
        />
        <label>
          Snap
          <select value={snap} onChange={(event) => setSnap(event.currentTarget.value as typeof snap)}>
            <option value="NONE">Off</option>
            <option value="FRAME_30">1/30 s</option>
          </select>
        </label>
        <button type="button" onClick={onAddEvent}>+ Event</button>
        <button type="button" onClick={onOpenTrim}>Trim</button>
        <button type="button" onClick={onOpenRetime}>Retime</button>
      </header>
      <div className="animation-timeline-edit-controls" aria-label="Timeline editing controls">
        <fieldset>
          <legend>Range selection</legend>
          <label>
            Start (s)
            <input
              type="number"
              aria-label="Range start seconds"
              min={0}
              max={clip.lengthSeconds}
              step={moveStep}
              value={rangeStart}
              onChange={(event) => setRangeStart(event.currentTarget.valueAsNumber)}
            />
          </label>
          <label>
            End (s)
            <input
              type="number"
              aria-label="Range end seconds"
              min={0}
              max={clip.lengthSeconds}
              step={moveStep}
              value={rangeEnd}
              onChange={(event) => setRangeEnd(event.currentTarget.valueAsNumber)}
            />
          </label>
          <button
            type="button"
            disabled={!validTimelineRangeV1(rangeStart, rangeEnd, clip.lengthSeconds)}
            onClick={() => onSelectRange(rangeStart, rangeEnd)}
          >
            Select keys in range
          </button>
        </fieldset>
        <div role="group" aria-label="Move selected keyframes">
          <button
            type="button"
            aria-label="Move selected keyframes left"
            aria-keyshortcuts="Alt+ArrowLeft"
            disabled={selectedKeyIds.size === 0}
            onClick={() => moveSelection(-1)}
          >
            Move left
          </button>
          <button
            type="button"
            aria-label="Move selected keyframes right"
            aria-keyshortcuts="Alt+ArrowRight"
            disabled={selectedKeyIds.size === 0}
            onClick={() => moveSelection(1)}
          >
            Move right
          </button>
        </div>
        <div role="group" aria-label="Timeline zoom and pan">
          <button
            type="button"
            aria-label="Zoom timeline out"
            aria-keyshortcuts="-"
            onClick={() => zoom(-1)}
          >
            Zoom out
          </button>
          <button
            type="button"
            aria-label="Zoom timeline in"
            aria-keyshortcuts="+"
            onClick={() => zoom(1)}
          >
            Zoom in
          </button>
          <button
            type="button"
            aria-label="Pan timeline left"
            aria-keyshortcuts="Shift+ArrowLeft"
            disabled={view.offsetSeconds <= 0}
            onClick={() => pan(-1)}
          >
            Pan left
          </button>
          <button
            type="button"
            aria-label="Pan timeline right"
            aria-keyshortcuts="Shift+ArrowRight"
            disabled={view.offsetSeconds >= clip.lengthSeconds}
            onClick={() => pan(1)}
          >
            Pan right
          </button>
        </div>
      </div>
      <p id="animation-timeline-keyboard-help" className="sr-only">
        Select a key, then press Alt plus Left or Right Arrow to move it.
        Press Delete to remove selected keys, Plus or Minus to zoom, and
        Shift plus Left or Right Arrow to pan.
      </p>
      <output className="animation-timeline-view-status" aria-live="polite">
        {selectedKeyIds.size} selected · {view.pixelsPerSecond.toFixed(0)} pixels per second
        {" · "}
        pan {view.offsetSeconds.toFixed(2)} seconds
      </output>
      <div
        className="animation-timeline-viewport"
        ref={timelineViewportRef}
        onScroll={(event) => {
          const offsetSeconds = event.currentTarget.scrollLeft
            / view.pixelsPerSecond;
          if (Math.abs(offsetSeconds - view.offsetSeconds) > 1e-4) {
            setView((current) => ({ ...current, offsetSeconds }));
          }
        }}
      >
        <div
          className="animation-timeline-surface"
          data-pixels-per-second={view.pixelsPerSecond.toFixed(2)}
          data-offset-seconds={view.offsetSeconds.toFixed(4)}
          style={{ width: `calc(10.5rem + ${timeAreaWidth}px)` }}
        >
          <AnimationTimelineRuler lengthSeconds={clip.lengthSeconds} />
          <div className="animation-dope-sheet__grid" role="grid" aria-label="Animation keyframes">
            <div
              className="animation-playhead"
              style={{
                left: clip.lengthSeconds <= 0
                  ? "10.5rem"
                  : `calc(10.5rem + ${
                    playheadSeconds / clip.lengthSeconds * timeAreaWidth
                  }px)`,
              }}
              aria-hidden="true"
            />
            {visibleTracks.map((track) => (
              <AnimationTrackRow
                key={track.id}
                track={track}
                rig={rig}
                lengthSeconds={clip.lengthSeconds}
                selectedKeyIds={selectedKeyIds}
                onSelectKey={onSelectKey}
              />
            ))}
            <AnimationEventTrack
              events={visibleEvents}
              lengthSeconds={clip.lengthSeconds}
              onSelect={onSelectEvent}
            />
          </div>
        </div>
      </div>
      {markerProjection.rendered < markerProjection.total ? (
        <p role="status">
          Timeline virtualized: showing {markerProjection.rendered} markers
          nearest the playhead out of {markerProjection.total}.
        </p>
      ) : null}
      <input
        type="range"
        aria-label="Animation playhead"
        min={0}
        max={clip.lengthSeconds}
        step={snap === "FRAME_30" ? 1 / 30 : 0.001}
        value={playheadSeconds}
        onChange={(event) => onSeek(event.currentTarget.valueAsNumber)}
      />
    </section>
  );
}

function isEditableTargetV1(target: EventTarget | null) {
  return target instanceof HTMLInputElement
    || target instanceof HTMLSelectElement
    || target instanceof HTMLTextAreaElement
    || (target instanceof HTMLElement && target.isContentEditable);
}

function validTimelineRangeV1(
  start: number,
  end: number,
  clipLength: number,
) {
  return Number.isFinite(start)
    && Number.isFinite(end)
    && start >= 0
    && end >= 0
    && start <= clipLength
    && end <= clipLength;
}
