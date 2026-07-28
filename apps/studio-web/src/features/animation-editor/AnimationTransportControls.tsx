export function AnimationTransportControls({
  playing,
  playheadSeconds,
  lengthSeconds,
  previousKey,
  nextKey,
  onPlayingChange,
  onSeek,
}: {
  playing: boolean;
  playheadSeconds: number;
  lengthSeconds: number;
  previousKey: () => void;
  nextKey: () => void;
  onPlayingChange: (playing: boolean) => void;
  onSeek: (time: number) => void;
}) {
  return (
    <div className="animation-transport" aria-label="Animation playback controls">
      <button type="button" onClick={() => onSeek(0)} aria-label="Go to start">|◀</button>
      <button type="button" onClick={previousKey} aria-label="Previous keyframe">◀</button>
      <button
        type="button"
        onClick={() => onPlayingChange(!playing)}
        aria-label={playing ? "Pause" : "Play"}
      >
        {playing ? "■" : "▶"}
      </button>
      <button type="button" onClick={nextKey} aria-label="Next keyframe">▶</button>
      <output aria-label="Current animation time">
        {playheadSeconds.toFixed(2)} / {lengthSeconds.toFixed(2)} s
      </output>
    </div>
  );
}
