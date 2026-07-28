import type { CreatureAnimationAuthoringEventV1 } from "./state";
import type { CreatureAnimationAuthoringV1 } from "./types";
import { detectFallbackCyclesV1 } from "./fallbacks";

export function FallbackReview({
  authoring,
  onEvent,
}: {
  authoring: CreatureAnimationAuthoringV1;
  onEvent: (event: CreatureAnimationAuthoringEventV1) => void;
}) {
  const pending = authoring.fallbacks.filter(({ review }) => review === "PENDING");
  if (pending.length === 0) return null;
  return (
    <section className="animation-fallback-review" aria-labelledby="fallback-review-heading">
      <h3 id="fallback-review-heading">Review fallback</h3>
      {pending.map((fallback) => {
        const wouldCreateCycle = detectFallbackCyclesV1(
          authoring.fallbacks.map((candidate) => (
            candidate.id === fallback.id
              ? { ...candidate, review: "ACCEPTED" as const }
              : candidate
          )),
        ).length > 0;
        const cycleMessageId = `fallback-cycle-${fallback.id}`;
        return (
        <article key={fallback.id}>
          <p>
            <strong>{fallback.targetSlot}</strong> uses <strong>{fallback.sourceSlot}</strong>
          </p>
          <p>{fallback.reason}</p>
          <div>
            <button
              type="button"
              onClick={() => onEvent({
                type: "ANIMATION_FALLBACK_REJECTED",
                fallbackId: fallback.id,
              })}
            >
              Reject
            </button>
            <button
              type="button"
              disabled={wouldCreateCycle}
              aria-describedby={wouldCreateCycle ? cycleMessageId : undefined}
              title={wouldCreateCycle ? "Accepting this fallback would create a cycle." : undefined}
              onClick={() => onEvent({
                type: "ANIMATION_FALLBACK_APPROVED",
                fallbackId: fallback.id,
              })}
            >
              Accept fallback
            </button>
          </div>
          {wouldCreateCycle ? (
            <p id={cycleMessageId} role="alert">
              This fallback cannot be accepted because it creates a cycle.
            </p>
          ) : null}
        </article>
        );
      })}
    </section>
  );
}
