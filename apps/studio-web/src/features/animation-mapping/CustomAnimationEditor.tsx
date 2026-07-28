import { useState } from "react";
import {
  createCustomAnimationV1,
  validateCustomAnimationNameV1,
} from "./customAnimations";
import type { CreatureAnimationAuthoringEventV1 } from "./state";
import type {
  CreatureAnimationAuthoringV1,
  CreatureAnimationInspectionV1,
  CustomAnimationPlaybackV1,
} from "./types";

export function CustomAnimationEditor({
  authoring,
  inspection,
  onEvent,
}: {
  authoring: CreatureAnimationAuthoringV1;
  inspection: CreatureAnimationInspectionV1;
  onEvent: (event: CreatureAnimationAuthoringEventV1) => void;
}) {
  const [name, setName] = useState("");
  const [playback, setPlayback] = useState<CustomAnimationPlaybackV1>("ONE_SHOT");
  const [primaryClip, setPrimaryClip] = useState("");
  const [startClip, setStartClip] = useState("");
  const [loopClip, setLoopClip] = useState("");
  const [endClip, setEndClip] = useState("");
  const [error, setError] = useState<string>();

  const add = () => {
    try {
      const custom = createCustomAnimationV1({
        name,
        playback,
        sourceClipName: playback === "ONE_SHOT" ? primaryClip : null,
        phases: playback === "ONE_SHOT"
          ? []
          : [
              ...(startClip ? [{ phase: "START" as const, sourceClipName: startClip }] : []),
              { phase: "LOOP" as const, sourceClipName: loopClip },
              ...(endClip ? [{ phase: "END" as const, sourceClipName: endClip }] : []),
            ],
        provenance: {
          provider: "USER_CUSTOM",
          assetId: authoring.sourceRevision,
          ownership: "USER_OWNED",
        },
      });
      onEvent({ type: "CUSTOM_ANIMATION_ADDED", custom });
      setName("");
      setPrimaryClip("");
      setStartClip("");
      setLoopClip("");
      setEndClip("");
      setError(undefined);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  };

  return (
    <details className="custom-animation-editor">
      <summary>Custom animations ({authoring.customAnimations.length})</summary>
      <div>
        {authoring.customAnimations.map((custom) => (
          <ExistingCustomAnimation
            key={custom.id}
            custom={custom}
            authoring={authoring}
            onEvent={onEvent}
          />
        ))}
        <h4>Add custom animation</h4>
        <label>
          <span>Name</span>
          <input
            aria-label="Custom animation name"
            aria-invalid={error ? true : undefined}
            aria-describedby={error ? "custom-animation-add-error" : undefined}
            value={name}
            maxLength={16}
            onChange={(event) => setName(event.target.value)}
            placeholder="custom_wave"
          />
        </label>
        <label>
          <span>Playback</span>
          <select
            aria-label="Custom animation playback"
            value={playback}
            onChange={(event) => setPlayback(event.target.value as CustomAnimationPlaybackV1)}
          >
            <option value="ONE_SHOT">One-shot</option>
            <option value="LOOPING_PHASED">Start / Loop / End</option>
          </select>
        </label>
        {playback === "ONE_SHOT" ? (
          <ClipSelect
            label="Source clip"
            value={primaryClip}
            required
            clips={inspection.sourceClips.map(({ name }) => name)}
            onChange={setPrimaryClip}
          />
        ) : (
          <>
            <ClipSelect
              label="Start clip"
              value={startClip}
              clips={inspection.sourceClips.map(({ name }) => name)}
              onChange={setStartClip}
            />
            <ClipSelect
              label="Loop clip"
              value={loopClip}
              required
              clips={inspection.sourceClips.map(({ name }) => name)}
              onChange={setLoopClip}
            />
            <ClipSelect
              label="End clip"
              value={endClip}
              clips={inspection.sourceClips.map(({ name }) => name)}
              onChange={setEndClip}
            />
          </>
        )}
        {error ? <p id="custom-animation-add-error" role="alert">{error}</p> : null}
        <button type="button" onClick={add}>Add custom</button>
      </div>
    </details>
  );
}

function ExistingCustomAnimation({
  custom,
  authoring,
  onEvent,
}: {
  custom: CreatureAnimationAuthoringV1["customAnimations"][number];
  authoring: CreatureAnimationAuthoringV1;
  onEvent: (event: CreatureAnimationAuthoringEventV1) => void;
}) {
  const [name, setName] = useState(custom.name);
  const [error, setError] = useState<string>();
  const inUse = authoring.assignments.some(
    ({ customAnimationId }) => customAnimationId === custom.id,
  );
  const save = () => {
    const diagnostics = validateCustomAnimationNameV1(
      name,
      authoring.customAnimations
        .filter(({ id }) => id !== custom.id)
        .map(({ name: candidate }) => candidate),
    );
    if (diagnostics.length > 0) {
      setError(diagnostics[0].message);
      return;
    }
    onEvent({
      type: "CUSTOM_ANIMATION_UPDATED",
      id: custom.id,
      patch: { name: name.trim().toLocaleLowerCase("en-US") },
    });
    setError(undefined);
  };
  const remove = () => {
    const confirmed = !inUse || window.confirm(
      `${custom.name} is assigned to a base slot. Remove it and clear those assignments?`,
    );
    if (!confirmed) return;
    onEvent({ type: "CUSTOM_ANIMATION_REMOVED", id: custom.id, confirmed });
  };
  return (
    <article>
      <label>
        <span>{custom.playback} name</span>
        <input
          aria-label={`Name for custom animation ${custom.name}`}
          aria-invalid={error ? true : undefined}
          aria-describedby={error ? `custom-animation-error-${custom.id}` : undefined}
          value={name}
          maxLength={custom.playback === "LOOPING_PHASED" ? 14 : 16}
          onChange={(event) => setName(event.target.value)}
        />
      </label>
      <p>{custom.playback === "ONE_SHOT"
        ? custom.sourceClipName
        : custom.phases.map(({ phase, sourceClipName }) => `${phase}: ${sourceClipName}`).join(" · ")}</p>
      {inUse ? <p>In use by a Base 42 assignment.</p> : null}
      {error ? (
        <p id={`custom-animation-error-${custom.id}`} role="alert">{error}</p>
      ) : null}
      <div>
        <button type="button" onClick={save}>Save custom</button>
        <button type="button" onClick={remove}>Remove custom</button>
      </div>
    </article>
  );
}

function ClipSelect({
  label,
  value,
  clips,
  required = false,
  onChange,
}: {
  label: string;
  value: string;
  clips: readonly string[];
  required?: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <label>
      <span>{label}</span>
      <select
        aria-label={label}
        value={value}
        required={required}
        onChange={(event) => onChange(event.target.value)}
      >
        <option value="">None</option>
        {clips.map((clip) => <option key={clip} value={clip}>{clip}</option>)}
      </select>
    </label>
  );
}
