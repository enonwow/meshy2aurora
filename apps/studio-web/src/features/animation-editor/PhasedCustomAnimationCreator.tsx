import { useEffect, useMemo, useState } from "react";
import type {
  AnimationStudioDocumentV1,
  CustomAnimationDefinitionV2,
} from "../animation-studio/types";
import {
  createPhasedCustomDefinitionFromAuthoredClipsV1,
  nextPhasedCustomAnimationNameV1,
} from "./editing";

export function PhasedCustomAnimationCreator({
  studio,
  existingCustomIds,
  existingCustomNames,
  onCreate,
}: {
  studio: AnimationStudioDocumentV1;
  existingCustomIds: readonly string[];
  existingCustomNames: readonly string[];
  onCreate: (definition: CustomAnimationDefinitionV2) => void;
}) {
  const validIds = useMemo(() => studio.authoredClips
    .filter(({ status }) => status === "VALID")
    .map(({ id }) => id), [studio.authoredClips]);
  const validIdsKey = validIds.join("\0");
  const [phaseClipIds, setPhaseClipIds] = useState<readonly string[]>(
    () => reconcilePhaseSelection([], validIds),
  );
  const [name, setName] = useState(
    () => nextPhasedCustomAnimationNameV1(existingCustomNames),
  );
  const [error, setError] = useState<string | null>(null);
  const [createdName, setCreatedName] = useState<string | null>(null);

  useEffect(() => {
    setPhaseClipIds((current) => reconcilePhaseSelection(current, validIds));
  }, [validIdsKey]);

  const selectedAreValid = phaseClipIds.length === 3
    && new Set(phaseClipIds).size === 3
    && phaseClipIds.every((id) => validIds.includes(id));
  const normalizedName = name.trim().toLocaleLowerCase("en-US");
  const nameIsValid = /^[a-z][a-z0-9_]{0,13}$/.test(normalizedName)
    && !existingCustomNames.some(
      (candidate) => candidate.toLocaleLowerCase("en-US") === normalizedName,
    );
  const canCreate = selectedAreValid && nameIsValid;

  const create = () => {
    try {
      const definition = createPhasedCustomDefinitionFromAuthoredClipsV1(
        studio,
        {
          name,
          startAuthoredClipId: phaseClipIds[0] ?? "",
          loopAuthoredClipId: phaseClipIds[1] ?? "",
          endAuthoredClipId: phaseClipIds[2] ?? "",
          existingIds: existingCustomIds,
          existingNames: existingCustomNames,
        },
      );
      onCreate(definition);
      setCreatedName(definition.name);
      setName(nextPhasedCustomAnimationNameV1([
        ...existingCustomNames,
        definition.name,
      ]));
      setError(null);
    } catch (caught: unknown) {
      setCreatedName(null);
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  };

  return (
    <fieldset
      className="phased-custom-animation-creator"
      aria-describedby="phased-custom-animation-help"
    >
      <legend>Create Start / Loop / End Custom</legend>
      <p id="phased-custom-animation-help">
        Choose three distinct authored clips. Draft and Invalid clips stay
        visible but cannot be selected.
      </p>
      <label>
        Phased Custom name
        <input
          aria-label="Phased Custom name"
          aria-invalid={!nameIsValid}
          value={name}
          maxLength={14}
          onChange={(event) => {
            setName(event.currentTarget.value);
            setCreatedName(null);
          }}
        />
      </label>
      {(["START", "LOOP", "END"] as const).map((phase, index) => (
        <label key={phase}>
          {phase[0] + phase.slice(1).toLocaleLowerCase("en-US")} clip
          <select
            aria-label={`${phase[0] + phase.slice(1).toLocaleLowerCase("en-US")} clip`}
            value={phaseClipIds[index] ?? ""}
            onChange={(event) => {
              const next = [...phaseClipIds];
              next[index] = event.currentTarget.value;
              setPhaseClipIds(next);
              setCreatedName(null);
            }}
          >
            <option value="">Choose a Valid clip</option>
            {studio.authoredClips.map((clip) => {
              const selectedByAnotherPhase = phaseClipIds.some(
                (selectedId, selectedIndex) => (
                  selectedIndex !== index && selectedId === clip.id
                ),
              );
              return (
                <option
                  key={clip.id}
                  value={clip.id}
                  disabled={clip.status !== "VALID" || selectedByAnotherPhase}
                >
                  {clip.name} — {clip.status}
                </option>
              );
            })}
          </select>
        </label>
      ))}
      {validIds.length < 3 ? (
        <p role="status">
          Save at least three distinct clips as Valid to create phased Custom.
        </p>
      ) : null}
      {error ? <p role="alert">{error}</p> : null}
      {createdName ? (
        <p role="status">
          {createdName} was added to Custom. Assign it below when ready.
        </p>
      ) : null}
      <button type="button" disabled={!canCreate} onClick={create}>
        Create phased Custom
      </button>
    </fieldset>
  );
}

function reconcilePhaseSelection(
  current: readonly string[],
  validIds: readonly string[],
) {
  const used = new Set<string>();
  return (["START", "LOOP", "END"] as const).map((_, index) => {
    const candidate = current[index];
    if (candidate && validIds.includes(candidate) && !used.has(candidate)) {
      used.add(candidate);
      return candidate;
    }
    const replacement = validIds.find((id) => !used.has(id)) ?? "";
    if (replacement) used.add(replacement);
    return replacement;
  });
}
