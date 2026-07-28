import { useEffect, useState, type ReactNode } from "react";
import {
  getAvailableAnimationSourcesV1,
  getIncompatibleAnimationSourcesV1,
} from "./authoring";
import type {
  CreatureAnimationAuthoringEventV1,
} from "./state";
import type {
  AnimationSourceKindV1,
  CreatureAnimationAuthoringV1,
  CreatureAnimationInspectionV1,
  DirectCreatureBaseSlotV1,
} from "./types";

export interface AnimationSourcePickerProps {
  slot: DirectCreatureBaseSlotV1;
  authoring: CreatureAnimationAuthoringV1;
  inspection: CreatureAnimationInspectionV1;
  onEvent: (event: CreatureAnimationAuthoringEventV1) => void;
  customPanel?: ReactNode;
  customSelected?: boolean;
}

export function AnimationSourcePicker({
  slot,
  authoring,
  inspection,
  onEvent,
  customPanel,
  customSelected = false,
}: AnimationSourcePickerProps) {
  const sources = getAvailableAnimationSourcesV1(slot, inspection);
  const incompatible = getIncompatibleAnimationSourcesV1(slot, inspection);
  const assignment = authoring.assignments.find(({ targetSlot }) => targetSlot === slot);
  const selected = sources.find(({ clipName }) => clipName === assignment?.sourceClipName);
  const assignedKind: AnimationSourceKindV1 = customSelected
    ? "CUSTOM"
    : assignment?.sourceKind ?? "SOURCE_CLIP";
  const [realization, setRealization] = useState<AnimationSourceKindV1>(
    assignedKind,
  );
  const customAvailable = customPanel !== undefined && customPanel !== null;

  useEffect(() => {
    setRealization(assignedKind);
  }, [assignedKind, slot]);

  const realizationOptions: readonly {
    kind: AnimationSourceKindV1;
    label: string;
    disabled: boolean;
    reason: string | null;
  }[] = [
    {
      kind: "SOURCE_CLIP",
      label: "Source clip",
      disabled: false,
      reason: null,
    },
    {
      kind: "INHERITED_SUPERMODEL",
      label: "Inherited supermodel",
      disabled: true,
      reason: "A compatible versioned supermodel provider is not packaged in this build lane.",
    },
    {
      kind: "PROCEDURAL",
      label: "Procedural",
      disabled: true,
      reason: "A procedural mapping provider is not available in this build lane.",
    },
    {
      kind: "CUSTOM",
      label: "Custom",
      disabled: !customAvailable,
      reason: customAvailable
        ? null
        : "Custom requires the saved project animation library.",
    },
  ];

  return (
    <div className="animation-source-picker">
      <div
        className="animation-source-picker__realization"
        role="radiogroup"
        aria-label="Source realization"
      >
        {realizationOptions.map((option) => {
          const descriptionId =
            `source-realization-${slot}-${option.kind.toLowerCase()}-reason`;
          return (
            <div key={option.kind}>
              <label>
                <input
                  type="radio"
                  name={`source-realization-${slot}`}
                  value={option.kind}
                  checked={realization === option.kind}
                  disabled={option.disabled}
                  aria-describedby={option.reason ? descriptionId : undefined}
                  onChange={() => setRealization(option.kind)}
                />
                <span>{option.label}</span>
              </label>
              {option.reason ? (
                <small id={descriptionId}>{option.reason}</small>
              ) : null}
            </div>
          );
        })}
      </div>

      {realization === "SOURCE_CLIP" ? (
        <>
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
        </>
      ) : null}

      {realization === "CUSTOM" && customAvailable ? customPanel : null}
    </div>
  );
}
