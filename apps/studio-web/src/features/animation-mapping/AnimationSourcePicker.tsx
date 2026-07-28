import {
  getAvailableAnimationSourcesV1,
  getIncompatibleAnimationSourcesV1,
} from "./authoring";
import type {
  CreatureAnimationAuthoringEventV1,
} from "./state";
import type {
  CreatureAnimationAuthoringV1,
  CreatureAnimationInspectionV1,
  DirectCreatureBaseSlotV1,
} from "./types";

export interface AnimationSourcePickerProps {
  slot: DirectCreatureBaseSlotV1;
  authoring: CreatureAnimationAuthoringV1;
  inspection: CreatureAnimationInspectionV1;
  onEvent: (event: CreatureAnimationAuthoringEventV1) => void;
}

export function AnimationSourcePicker({
  slot,
  authoring,
  inspection,
  onEvent,
}: AnimationSourcePickerProps) {
  const sources = getAvailableAnimationSourcesV1(slot, inspection);
  const incompatible = getIncompatibleAnimationSourcesV1(slot, inspection);
  const assignment = authoring.assignments.find(({ targetSlot }) => targetSlot === slot);
  const selected = sources.find(({ clipName }) => clipName === assignment?.sourceClipName);

  return (
    <div className="animation-source-picker">
      <label>
        <span>Source clip</span>
        <select
          aria-label={`Source clip for ${slot}`}
          value={selected?.clipId ?? ""}
          onChange={(event) => {
            const source = sources.find(({ clipId }) => clipId === event.target.value);
            if (!source) {
              onEvent({ type: "ANIMATION_SOURCE_CLEARED", slot });
              return;
            }
            onEvent({
              type: "ANIMATION_SOURCE_ASSIGNED",
              assignment: {
                targetSlot: slot,
                sourceKind: "SOURCE_CLIP",
                sourceClipName: source.clipName,
                customAnimationId: null,
                provenance: {
                  provider: "SOURCE_GLB",
                  assetId: authoring.sourceRevision,
                  ownership: "USER_OWNED",
                },
              },
            });
          }}
        >
          <option value="">Not assigned</option>
          {sources.map((source) => (
            <option key={source.clipId} value={source.clipId}>
              {source.clipName} · {Math.round(source.score * 100)}%
              {source.requiresReview ? " · review" : ""}
            </option>
          ))}
        </select>
      </label>
      <p>
        Only clips with animation tracks and a semantic candidate for this slot
        are listed. Supermodels require a versioned portable provider, never a
        filesystem path; no compatible provider is packaged in this build lane yet.
      </p>
      {incompatible.length > 0 ? (
        <details>
          <summary>{incompatible.length} incompatible source clip(s)</summary>
          <ul>
            {incompatible.map((source) => (
              <li key={source.clipId}>
                <strong>{source.clipName}</strong>: {source.reason}
              </li>
            ))}
          </ul>
        </details>
      ) : null}
    </div>
  );
}
