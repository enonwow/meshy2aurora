import type {
  AnimationStudioDiagnosticV1,
  AuthoredAnimationEventV1,
} from "../animation-studio/types";

export function AnimationEventEditor({
  event,
  diagnostics,
  onChange,
  onRemove,
  onClose,
}: {
  event: AuthoredAnimationEventV1;
  diagnostics: readonly AnimationStudioDiagnosticV1[];
  onChange: (event: AuthoredAnimationEventV1) => void;
  onRemove: () => void;
  onClose: () => void;
}) {
  return (
    <section
      className="animation-event-editor"
      role="dialog"
      aria-modal="true"
      aria-labelledby="animation-event-title"
      aria-describedby="animation-event-description"
      onKeyDown={(input) => {
        if (input.key === "Escape") {
          input.preventDefault();
          onClose();
        }
      }}
    >
      <header>
        <h3 id="animation-event-title">Animation event</h3>
        <button type="button" onClick={onClose} aria-label="Close event editor">×</button>
      </header>
      <label>
        Time (s)
        <input
          autoFocus
          type="number"
          min={0}
          step={0.001}
          value={event.timeSeconds}
          onChange={(input) => onChange({
            ...event,
            timeSeconds: input.currentTarget.valueAsNumber,
          })}
        />
      </label>
      <label>
        Callback name
        <input
          type="text"
          maxLength={31}
          pattern="[\x20-\x7e]+"
          value={event.name}
          onChange={(input) => onChange({ ...event, name: input.currentTarget.value })}
        />
      </label>
      <p id="animation-event-description">
        Free-text callback names are advanced authoring data. Runtime semantics
        remain subject to owner proof.
      </p>
      {diagnostics.length > 0 ? (
        <ul role="alert">
          {diagnostics.map((item) => <li key={`${item.code}:${item.path}`}>{item.message}</li>)}
        </ul>
      ) : null}
      <button type="button" onClick={onRemove}>Delete event</button>
    </section>
  );
}
