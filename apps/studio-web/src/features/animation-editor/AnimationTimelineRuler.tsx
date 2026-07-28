export function AnimationTimelineRuler({
  lengthSeconds,
}: {
  lengthSeconds: number;
}) {
  const count = Math.min(12, Math.max(2, Math.ceil(lengthSeconds * 10)));
  return (
    <div className="animation-timeline-ruler" aria-hidden="true">
      {Array.from({ length: count + 1 }, (_, index) => (
        <span key={index}>{(lengthSeconds * index / count).toFixed(2)}</span>
      ))}
    </div>
  );
}
