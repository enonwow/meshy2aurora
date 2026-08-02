import { memo } from "react";
import type { AuthoredAnimationEventV1 } from "../animation-studio/types";

export const AnimationEventTrack = memo(function AnimationEventTrack({
  events,
  lengthSeconds,
  onSelect,
}: {
  events: readonly AuthoredAnimationEventV1[];
  lengthSeconds: number;
  onSelect: (id: string) => void;
}) {
  return (
    <div className="animation-event-track" role="row" aria-label="Animation events">
      <span role="rowheader">Events</span>
      <div role="gridcell">
        {events.map((event) => (
          <button
            type="button"
            key={event.id}
            style={{
              left: `${lengthSeconds <= 0 ? 0 : event.timeSeconds / lengthSeconds * 100}%`,
            }}
            aria-label={`Event ${event.name || "unnamed"} at ${event.timeSeconds.toFixed(3)} seconds`}
            onClick={() => onSelect(event.id)}
          >
            ◆ <span>{event.name || "new event"}</span>
          </button>
        ))}
      </div>
    </div>
  );
});
