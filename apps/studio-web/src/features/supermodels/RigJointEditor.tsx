import { useEffect, useMemo, useState } from "react";
import type { AppliedSupermodelPreviewV2 } from "./appliedPreview";
import {
  removeWeightOverrideV2,
  upsertComponentBindingV2,
  upsertRegionWeightConstraintV2,
  upsertWeightOverrideV2,
  type ReferenceSupermodelRigAuthoringDocumentV2,
} from "./rigAuthoring";

interface Props {
  preview: AppliedSupermodelPreviewV2;
  selectedNodeName?: string;
  busy: boolean;
  onSelectNodeName: (name: string) => void;
  onApply: (document: ReferenceSupermodelRigAuthoringDocumentV2) => Promise<void>;
}

function numeric(value: string, label: string) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) throw new Error(`${label}: oczekiwano skończonej liczby.`);
  return parsed;
}

function parseWeightText(value: string) {
  const influences = value.split(",").map((row) => row.trim()).filter(Boolean).map((row) => {
    const [bone, weight, ...rest] = row.split(":").map((field) => field.trim());
    if (!bone || !weight || rest.length) throw new Error("Wagi zapisuj jako boneId:wartość, np. 12:0.75, 8:0.25.");
    return { boneNodeId: numeric(bone, "Bone id"), value: numeric(weight, "Waga") };
  });
  const sum = influences.reduce((total, influence) => total + influence.value, 0);
  if (influences.length < 1 || influences.length > 4 || Math.abs(sum - 1) > 1e-4) {
    throw new Error("Wiersz musi mieć 1–4 wpływy, których suma wynosi 1.0.");
  }
  return influences;
}

function weightText(influences: readonly { readonly boneNodeId: number; readonly value: number }[]) {
  return influences.map((influence) => `${influence.boneNodeId}:${influence.value}`).join(", ");
}

function parseIntegerList(value: string, label: string) {
  const rows = value.split(",").map((row) => row.trim()).filter(Boolean).map((row) => {
    const parsed = Number(row);
    if (!Number.isSafeInteger(parsed) || parsed < 0) throw new Error(`${label}: oczekiwano listy nieujemnych liczb całkowitych.`);
    return parsed;
  });
  if (new Set(rows).size !== rows.length) throw new Error(`${label}: lista zawiera duplikaty.`);
  return rows;
}

export function RigJointEditor({
  preview,
  selectedNodeName,
  busy,
  onSelectNodeName,
  onApply,
}: Props) {
  const selectedNode = useMemo(() => {
    const normalized = selectedNodeName?.trim().toLocaleLowerCase();
    return preview.targetRig.nodes.find((node) => node.name.trim().toLocaleLowerCase() === normalized)
      ?? preview.targetRig.nodes[0];
  }, [preview.targetRig.nodes, selectedNodeName]);
  const [editorError, setEditorError] = useState<string>();

  const initialSegment = preview.targetRig.segments[0];
  const [segmentId, setSegmentId] = useState(initialSegment?.id ?? 0);
  const [vertexIndex, setVertexIndex] = useState(0);
  const [weights, setWeights] = useState("");
  const [componentIndex, setComponentIndex] = useState(0);
  const [componentRegionId, setComponentRegionId] = useState("");
  const [componentAllowedBones, setComponentAllowedBones] = useState("");
  const [componentLocked, setComponentLocked] = useState(false);
  const [constraintRegionId, setConstraintRegionId] = useState("");
  const [constraintVertices, setConstraintVertices] = useState("");
  const [constraintAllowedBones, setConstraintAllowedBones] = useState("");
  const [constraintForbiddenBones, setConstraintForbiddenBones] = useState("");
  const [constraintMaximumInfluences, setConstraintMaximumInfluences] = useState(4);
  const [constraintLocked, setConstraintLocked] = useState(false);
  const selectedSegment = preview.targetRig.segments.find((segment) => segment.id === segmentId) ?? initialSegment;
  const selectedWeightOverride = selectedSegment
    ? preview.rigAuthoring.weightOverrides.find((row) => row.segmentId === selectedSegment.id && row.vertexIndex === vertexIndex)
    : undefined;

  useEffect(() => {
    if (!selectedSegment) {
      setWeights("");
      return;
    }
    const override = preview.rigAuthoring.weightOverrides.find((row) => (
      row.segmentId === selectedSegment.id && row.vertexIndex === vertexIndex
    ));
    const base = selectedSegment.referenceWeights[vertexIndex] ?? [];
    setWeights(weightText(override?.influences ?? base));
  }, [preview.rigAuthoring.contentSha256, selectedSegment, vertexIndex]);

  if (!selectedNode) {
    return <section className="supermodel-rig-editor"><strong>Brak edytowalnych jointów w target rigu.</strong></section>;
  }

  const applyWeights = async () => {
    if (!selectedSegment) return;
    setEditorError(undefined);
    try {
      if (vertexIndex < 0 || vertexIndex >= selectedSegment.referenceWeights.length) {
        throw new Error(`Vertex musi być w zakresie 0–${Math.max(selectedSegment.referenceWeights.length - 1, 0)}.`);
      }
      const influences = parseWeightText(weights);
      const disallowed = influences.find((influence) => !selectedSegment.allowedBoneNodeIds.includes(influence.boneNodeId));
      if (disallowed) throw new Error(`Bone ${disallowed.boneNodeId} nie jest dozwolony dla segmentu ${selectedSegment.name}.`);
      await onApply(upsertWeightOverrideV2(preview.rigAuthoring, {
        segmentId: selectedSegment.id,
        vertexIndex,
        influences,
      }));
    } catch (error) {
      setEditorError(error instanceof Error ? error.message : String(error));
    }
  };

  const applyComponentBinding = async () => {
    if (!selectedSegment) return;
    setEditorError(undefined);
    try {
      if (!componentRegionId.trim()) throw new Error("Binding komponentu wymaga regionId.");
      const allowedBoneNodeIds = parseIntegerList(componentAllowedBones, "Dozwolone bone IDs");
      if (allowedBoneNodeIds.length === 0) throw new Error("Binding komponentu wymaga co najmniej jednego bone ID.");
      await onApply(upsertComponentBindingV2(preview.rigAuthoring, {
        segmentId: selectedSegment.id,
        componentIndex,
        regionId: componentRegionId.trim(),
        allowedBoneNodeIds,
        locked: componentLocked,
      }));
    } catch (error) {
      setEditorError(error instanceof Error ? error.message : String(error));
    }
  };

  const applyRegionConstraint = async () => {
    if (!selectedSegment) return;
    setEditorError(undefined);
    try {
      if (!constraintRegionId.trim()) throw new Error("Constraint wag wymaga regionId.");
      if (constraintMaximumInfluences < 1 || constraintMaximumInfluences > 4) throw new Error("Limit wpływów musi być w zakresie 1–4.");
      await onApply(upsertRegionWeightConstraintV2(preview.rigAuthoring, {
        segmentId: selectedSegment.id,
        regionId: constraintRegionId.trim(),
        vertexIndices: parseIntegerList(constraintVertices, "Vertex indices"),
        allowedBoneNodeIds: parseIntegerList(constraintAllowedBones, "Dozwolone bone IDs"),
        forbiddenBoneNodeIds: parseIntegerList(constraintForbiddenBones, "Zabronione bone IDs"),
        maximumInfluenceCount: constraintMaximumInfluences,
        locked: constraintLocked,
      }));
    } catch (error) {
      setEditorError(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <section className="supermodel-rig-editor" aria-label="Edytor target riga">
      <header>
        <div><p className="eyebrow">Immutable supermodel bind · authoring V2</p><h3>Szkielet i wagi</h3></div>
        <span>{preview.targetRig.nodes.length} jointów read-only · {preview.rigAuthoring.weightOverrides.length} weight overrides</span>
      </header>
      <p>Jointy, ich hierarchia oraz bind pose są dokładną, niezmienną kopią kontraktu wybranego supermodelu. Dopasowanie dotyczy mesha i wag, nie szkieletu.</p>
      <label>Joint
        <select value={selectedNode.name} onChange={(event) => onSelectNodeName(event.target.value)}>
          {preview.targetRig.nodes.map((node) => <option key={node.id} value={node.name}>#{node.id} {node.name}</option>)}
        </select>
      </label>
      <dl className="supermodel-rig-editor__identity">
        <div><dt>Carrier part</dt><dd>#{selectedNode.id}</dd></div>
        <div><dt>Parent</dt><dd>{selectedNode.parentId === null ? "—" : `#${selectedNode.parentId}`}</dd></div>
        <div><dt>Bind pose</dt><dd>retail 1:1 · tylko odczyt</dd></div>
      </dl>
      {selectedSegment ? (
        <details>
          <summary>Sparse weight override</summary>
          <label>Segment<select value={selectedSegment.id} onChange={(event) => { setSegmentId(Number(event.target.value)); setVertexIndex(0); }}>
            {preview.targetRig.segments.map((segment) => <option key={segment.id} value={segment.id}>#{segment.id} {segment.name}</option>)}
          </select></label>
          <label>Vertex<input type="number" min={0} max={Math.max(selectedSegment.referenceWeights.length - 1, 0)} value={vertexIndex} onChange={(event) => setVertexIndex(Math.max(0, Math.trunc(Number(event.target.value) || 0)))} /></label>
          <label>Bone:weight<input value={weights} onChange={(event) => setWeights(event.target.value)} placeholder="12:0.75, 8:0.25" /></label>
          <small>Dozwolone bone IDs: {selectedSegment.allowedBoneNodeIds.join(", ")}. Maksymalnie 4 wpływy, suma 1.0.</small>
          <div className="supermodel-rig-editor__actions">
            <button type="button" className="button button--primary" disabled={busy} onClick={() => void applyWeights()}>Zastosuj wagi i przelicz preview</button>
            <button type="button" className="button button--secondary" disabled={busy || !selectedWeightOverride} onClick={() => void onApply(removeWeightOverrideV2(preview.rigAuthoring, selectedSegment.id, vertexIndex))}>Przywróć bazowe wagi</button>
          </div>
        </details>
      ) : null}
      {selectedSegment ? (
        <details>
          <summary>Component binding</summary>
          <label>Component index<input type="number" min={0} value={componentIndex} onChange={(event) => setComponentIndex(Math.max(0, Math.trunc(Number(event.target.value) || 0)))} /></label>
          <label>Region ID<input value={componentRegionId} placeholder="np. tail_fur" onChange={(event) => setComponentRegionId(event.target.value)} /></label>
          <label>Dozwolone bone IDs<input value={componentAllowedBones} placeholder="18, 19" onChange={(event) => setComponentAllowedBones(event.target.value)} /></label>
          <label><input type="checkbox" checked={componentLocked} disabled={busy} onChange={(event) => setComponentLocked(event.target.checked)} />Zablokuj binding</label>
          <button type="button" className="button button--primary" disabled={busy} onClick={() => void applyComponentBinding()}>Zastosuj binding i przelicz preview</button>
        </details>
      ) : null}
      {selectedSegment ? (
        <details>
          <summary>Region weight constraint</summary>
          <label>Region ID<input value={constraintRegionId} placeholder="np. left_front_lower" onChange={(event) => setConstraintRegionId(event.target.value)} /></label>
          <label>Vertex indices<input value={constraintVertices} placeholder="12, 18, 27" onChange={(event) => setConstraintVertices(event.target.value)} /></label>
          <label>Dozwolone bone IDs<input value={constraintAllowedBones} placeholder="13, 14, 15" onChange={(event) => setConstraintAllowedBones(event.target.value)} /></label>
          <label>Zabronione bone IDs<input value={constraintForbiddenBones} placeholder="20, 24" onChange={(event) => setConstraintForbiddenBones(event.target.value)} /></label>
          <label>Maks. wpływów<input type="number" min={1} max={4} value={constraintMaximumInfluences} onChange={(event) => setConstraintMaximumInfluences(Math.max(1, Math.min(4, Math.trunc(Number(event.target.value) || 1))))} /></label>
          <label><input type="checkbox" checked={constraintLocked} disabled={busy} onChange={(event) => setConstraintLocked(event.target.checked)} />Zablokuj constraint</label>
          <button type="button" className="button button--primary" disabled={busy} onClick={() => void applyRegionConstraint()}>Zastosuj constraint i przelicz preview</button>
        </details>
      ) : null}
      {editorError ? <p role="alert" className="supermodel-library__error">{editorError}</p> : null}
    </section>
  );
}
