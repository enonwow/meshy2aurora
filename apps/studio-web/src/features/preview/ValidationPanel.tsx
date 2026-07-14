import type { ModelPartRef, StudioDiagnostic } from "./types";

interface Props {
  diagnostics: StudioDiagnostic[];
  selectedPart?: ModelPartRef;
  onSelectPart: (part?: ModelPartRef) => void;
}

const samePart = (left?: ModelPartRef, right?: ModelPartRef) =>
  Boolean(left && right && left.kind === right.kind && String(left.id) === String(right.id));

export function ValidationPanel({ diagnostics, selectedPart, onSelectPart }: Props) {
  const visible = selectedPart
    ? diagnostics.filter((diagnostic) => !diagnostic.target || samePart(diagnostic.target, selectedPart))
    : diagnostics;
  return (
    <section className="panel" aria-label="Canonical readback validation">
      <div className="status">
        <strong>READBACK VALIDATION</strong>
        <span>{selectedPart ? selectedPart.label : "all model parts"}</span>
      </div>
      {selectedPart && <button type="button" onClick={() => onSelectPart(undefined)}>Show all</button>}
      {visible.length === 0 ? <p>No diagnostics for the current selection.</p> : (
        <ul>{visible.map((diagnostic) => (
          <li key={diagnostic.id}>
            <strong>{diagnostic.severity}</strong>
            <span><code>{diagnostic.code}</code><br />{diagnostic.message}</span>
            {diagnostic.target && <button type="button" onClick={() => onSelectPart(diagnostic.target)}>Select {diagnostic.target.label}</button>}
          </li>
        ))}</ul>
      )}
    </section>
  );
}
