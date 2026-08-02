import { memo } from "react";
import type { AuthoredAnimationTrackV1 } from "../animation-studio/types";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import { AnimationKeyframeMarker } from "./AnimationKeyframeMarker";
import { animationKeySelectionIdV1 } from "./editing";

export const AnimationTrackRow = memo(function AnimationTrackRow({
  track,
  rig,
  lengthSeconds,
  selectedKeyIds,
  onSelectKey,
}: {
  track: AuthoredAnimationTrackV1;
  rig: readonly AnimationRigNodeV1[];
  lengthSeconds: number;
  selectedKeyIds: ReadonlySet<string>;
  onSelectKey: (id: string, additive: boolean) => void;
}) {
  const boneName = rig.find(({ id }) => id === track.targetNodeId)?.name
    ?? `Node ${track.targetNodeId}`;
  return (
    <div
      className="animation-track-row"
      role="row"
      aria-label={`${boneName} ${track.path} track`}
    >
      <span role="rowheader">{boneName} · {track.path.toLocaleLowerCase("en-US")}</span>
      <div role="gridcell">
        {track.keyframes.map((keyframe) => (
          <AnimationKeyframeMarker
            key={keyframe.id}
            id={animationKeySelectionIdV1(track.id, keyframe.id)}
            timeSeconds={keyframe.timeSeconds}
            selected={selectedKeyIds.has(
              animationKeySelectionIdV1(track.id, keyframe.id),
            )}
            leftPercent={lengthSeconds <= 0 ? 0 : keyframe.timeSeconds / lengthSeconds * 100}
            boneName={boneName}
            path={track.path}
            onSelect={onSelectKey}
          />
        ))}
      </div>
    </div>
  );
});
