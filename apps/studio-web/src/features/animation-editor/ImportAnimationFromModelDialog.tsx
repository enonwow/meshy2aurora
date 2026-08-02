import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import {
  type AnimationTransferModeV1,
  type AnimationTransferBatchResultV2,
  type ExternalAnimationSourceInspectionV1,
  type HumanoidSemanticBoneMapV2,
  type HumanoidSemanticOverrideV2,
} from "./animationImport";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import "./animation-library.base.css";

export interface AnimationModelDonorV1 {
  readonly id: string;
  readonly file: File;
  readonly label: string;
  readonly detail?: string;
}

interface Props {
  availableDonors?: readonly AnimationModelDonorV1[];
  targetRig?: readonly AnimationRigNodeV1[];
  onInspect: (
    file: File,
    overrides?: readonly HumanoidSemanticOverrideV2[],
    manualMappingConfirmed?: boolean,
  ) => Promise<ExternalAnimationSourceInspectionV1>;
  onPrepare: (
    file: File,
    clipName: string,
    donorSourceRevision: string,
    mode: AnimationTransferModeV1,
    semanticMap?: HumanoidSemanticBoneMapV2,
  ) => Promise<AuthoredAnimationClipV1>;
  onPrepareBatch?: (
    file: File,
    inspection: ExternalAnimationSourceInspectionV1,
    clipNames: readonly string[],
    mode: AnimationTransferModeV1,
    semanticMap?: HumanoidSemanticBoneMapV2,
  ) => Promise<AnimationTransferBatchResultV2>;
  onCommit: (
    clip: AuthoredAnimationClipV1,
    clipName: string,
    donorSourceRevision: string,
    mode: AnimationTransferModeV1,
    semanticMap?: HumanoidSemanticBoneMapV2,
  ) => void;
  onCommitBatch?: (
    batch: AnimationTransferBatchResultV2,
    donorSourceRevision: string,
    mode: AnimationTransferModeV1,
  ) => void;
  renderPreview?: (
    clip: AuthoredAnimationClipV1,
    donorFile: File,
    donorClipName: string,
    donorSourceRevision: string,
  ) => ReactNode;
  onClose: () => void;
}

export function ImportAnimationFromModelDialog({
  availableDonors = [],
  targetRig = [],
  onInspect,
  onPrepare,
  onPrepareBatch,
  onCommit,
  onCommitBatch,
  renderPreview,
  onClose,
}: Props) {
  const titleId = useId();
  const descriptionId = useId();
  const generationRef = useRef(0);
  const cardRef = useRef<HTMLDivElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [inspection, setInspection] =
    useState<ExternalAnimationSourceInspectionV1 | null>(null);
  const [selectedClipName, setSelectedClipName] = useState("");
  const [selectedMode, setSelectedMode] = useState<AnimationTransferModeV1 | "">("");
  const [preparedClip, setPreparedClip] = useState<AuthoredAnimationClipV1 | null>(null);
  const [preparedBatch, setPreparedBatch] = useState<AnimationTransferBatchResultV2 | null>(null);
  const [batchProgress, setBatchProgress] = useState<{ readonly completed: number; readonly total: number } | null>(null);
  const [batchSelection, setBatchSelection] = useState<readonly string[]>([]);
  const [clipSearch, setClipSearch] = useState("");
  const [mappingOpen, setMappingOpen] = useState(false);
  const [mappingDraft, setMappingDraft] = useState<Record<string, { donorNodeId: number | null; targetNodeId: number | null }>>({});
  const [busy, setBusy] = useState<
    "INSPECTING" | "PREPARING" | "COMMITTING" | null
  >(null);
  const [error, setError] = useState<string | null>(null);
  const compatibility = inspection?.transferCompatibility ?? null;
  const semanticCompatibility = inspection?.semanticCompatibility ?? null;
  const allowedModes: readonly AnimationTransferModeV1[] = [
    ...(compatibility?.allowedModes ?? []),
    ...(semanticCompatibility?.status === "COMPATIBLE"
      && !compatibility?.allowedModes.includes("HUMANOID_SEMANTIC_RETARGET_V2")
      ? ["HUMANOID_SEMANTIC_RETARGET_V2" as const]
      : []),
  ];

  useEffect(() => () => {
    generationRef.current += 1;
  }, []);

  const inspect = async (nextFile: File) => {
    const generation = generationRef.current + 1;
    generationRef.current = generation;
    setFile(nextFile);
    setInspection(null);
    setSelectedClipName("");
    setSelectedMode("");
    setPreparedClip(null);
    setPreparedBatch(null);
    setBatchProgress(null);
    setError(null);
    setBusy("INSPECTING");
    try {
      const result = await onInspect(nextFile);
      if (generationRef.current !== generation) return;
      setInspection(result);
      setMappingDraft(Object.fromEntries(result.semanticCompatibility?.entries.map((entry) => [
        entry.semantic,
        { donorNodeId: entry.donorNodeId, targetNodeId: entry.targetNodeId },
      ]) ?? []));
      setMappingOpen(result.semanticCompatibility?.status === "MANUAL_CONFIRMATION_REQUIRED");
      setSelectedClipName(result.clips[0]?.name ?? "");
      setBatchSelection(result.clips.map(({ name }) => name));
      setSelectedMode(
        result.transferCompatibility.allowedModes[0]
        ?? (result.semanticCompatibility?.status === "COMPATIBLE"
          ? "HUMANOID_SEMANTIC_RETARGET_V2"
          : ""),
      );
      if (result.clips.length === 0) {
        setError("This GLB contains no animation clips.");
      }
    } catch (cause: unknown) {
      if (generationRef.current !== generation) return;
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (generationRef.current === generation) setBusy(null);
    }
  };

  const prepareBatch = async () => {
    if (!file || !inspection || !selectedMode || !onPrepareBatch || batchSelection.length === 0) return;
    const generation = generationRef.current + 1;
    generationRef.current = generation;
    setBusy("PREPARING"); setError(null); setPreparedClip(null); setPreparedBatch(null);
    setBatchProgress({ completed: 0, total: batchSelection.length });
    try {
      const result = await onPrepareBatch(file, inspection, batchSelection, selectedMode,
        selectedMode === "HUMANOID_SEMANTIC_RETARGET_V2" ? inspection.semanticCompatibility : undefined);
      if (generationRef.current !== generation) return;
      if (result.clips.length !== batchSelection.length) {
        throw new Error(`Core prepared ${result.clips.length}/${batchSelection.length} clips; atomic batch was rejected.`);
      }
      setPreparedBatch(result);
      setBatchProgress({ completed: result.clips.length, total: batchSelection.length });
    } catch (cause: unknown) {
      if (generationRef.current !== generation) return;
      setError(cause instanceof Error ? cause.message : String(cause));
      setBatchProgress({ completed: 0, total: batchSelection.length });
    }
    finally { if (generationRef.current === generation) setBusy(null); }
  };

  const cancelPending = () => {
    generationRef.current += 1;
    setBusy(null);
    setPreparedClip(null);
    setPreparedBatch(null);
    setBatchProgress(null);
    setError("Preparation cancelled safely. No animation was committed; retry is available.");
  };

  const validateManualMapping = async () => {
    if (!file) return;
    const overrides: HumanoidSemanticOverrideV2[] = Object.entries(mappingDraft)
      .filter((entry): entry is [string, { donorNodeId: number; targetNodeId: number }] => entry[1].donorNodeId !== null && entry[1].targetNodeId !== null)
      .map(([semantic, value]) => ({ semantic, donorNodeId: value.donorNodeId, targetNodeId: value.targetNodeId }));
    setBusy("INSPECTING");
    setError(null);
    setPreparedClip(null);
    setPreparedBatch(null);
    try {
      const result = await onInspect(file, overrides, true);
      setInspection(result);
      if (result.semanticCompatibility?.status !== "COMPATIBLE") {
        throw new Error("The manual semantic map is incomplete, duplicate, or stale. Core kept the transfer blocked.");
      }
      setSelectedMode("HUMANOID_SEMANTIC_RETARGET_V2");
      setMappingOpen(false);
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(null);
    }
  };

  const prepareSelected = async () => {
    if (!file || !inspection || !selectedClipName || !selectedMode) return;
    const generation = generationRef.current;
    setError(null);
    setPreparedClip(null);
    setBusy("PREPARING");
    try {
      const clip = await onPrepare(
        file,
        selectedClipName,
        inspection.sourceRevision,
        selectedMode,
        selectedMode === "HUMANOID_SEMANTIC_RETARGET_V2"
          ? inspection.semanticCompatibility
          : undefined,
      );
      if (generationRef.current !== generation) return;
      setPreparedClip(clip);
    } catch (cause: unknown) {
      if (generationRef.current !== generation) return;
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (generationRef.current === generation) setBusy(null);
    }
  };

  const commitSelected = () => {
    if (!preparedClip || !inspection || !selectedMode) return;
    setError(null);
    setBusy("COMMITTING");
    try {
      onCommit(
        preparedClip,
        selectedClipName,
        inspection.sourceRevision,
        selectedMode,
        selectedMode === "HUMANOID_SEMANTIC_RETARGET_V2"
          ? inspection.semanticCompatibility
          : undefined,
      );
      onClose();
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
      setPreparedClip(null);
      setBusy(null);
    }
  };

  const invalidatePreview = () => {
    generationRef.current += 1;
    setPreparedClip(null);
    setError(null);
  };

  return (
    <div
      className="animation-import-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
      onKeyDown={(event) => {
        if (event.key === "Escape" && busy === null) {
          event.preventDefault();
          onClose();
          return;
        }
        if (event.key === "Tab") {
          const focusable = Array.from(
            cardRef.current?.querySelectorAll<HTMLElement>(
              "button:not(:disabled), input:not(:disabled), select:not(:disabled)",
            ) ?? [],
          );
          if (focusable.length === 0) return;
          const first = focusable[0]!;
          const last = focusable[focusable.length - 1]!;
          if (event.shiftKey && globalThis.document.activeElement === first) {
            event.preventDefault();
            last.focus();
          } else if (!event.shiftKey && globalThis.document.activeElement === last) {
            event.preventDefault();
            first.focus();
          }
        }
      }}
    >
      <div
        ref={cardRef}
        className={`animation-import-dialog__card${
          preparedClip ? " animation-import-dialog__card--preview" : ""
        }`}
      >
        <header>
          <div>
            <span className="animation-import-dialog__eyebrow">Custom library</span>
            <h3 id={titleId}>Copy animation from another model</h3>
          </div>
          <button
            type="button"
            aria-label="Close animation import"
            disabled={busy !== null}
            onClick={onClose}
          >
            ×
          </button>
        </header>

        <p id={descriptionId}>
          Choose a verified Meshy donor or a local animated GLB. Core inspects
          both exact rigs; only the selected motion and provenance can enter
          this project.
        </p>

        {availableDonors.length > 0 ? (
          <section
            className="animation-import-dialog__donors"
            aria-labelledby="animation-import-donors-title"
          >
            <div>
              <strong id="animation-import-donors-title">Available from Meshy Bridge</strong>
              <small>Every GLB was hash-verified before this browser session.</small>
            </div>
            <ul>
              {availableDonors.map((donor, index) => (
                <li key={donor.id}>
                  <button
                    type="button"
                    autoFocus={index === 0}
                    disabled={busy !== null}
                    aria-pressed={file === donor.file}
                    onClick={() => void inspect(donor.file)}
                  >
                    <strong>{donor.label}</strong>
                    <small>{donor.detail ?? donor.file.name}</small>
                  </button>
                </li>
              ))}
            </ul>
          </section>
        ) : null}

        <label className="animation-import-dialog__file">
          <span>
            {availableDonors.length > 0
              ? "Or choose another donor (.glb)"
              : "Donor model (.glb)"}
          </span>
          <input
            autoFocus={availableDonors.length === 0}
            type="file"
            accept=".glb,model/gltf-binary"
            disabled={busy !== null}
            onChange={(event) => {
              const nextFile = event.currentTarget.files?.[0];
              if (nextFile) void inspect(nextFile);
            }}
          />
          <small>{file?.name ?? "No donor model selected"}</small>
        </label>

        {busy === "INSPECTING" ? (
          <p className="animation-import-dialog__status" role="status">
            Inspecting donor and target rigs…
          </p>
        ) : null}
        {busy === "PREPARING" ? (
          <div className="animation-import-dialog__status" role="status">
            Core is preparing the target-model preview…
            {batchProgress ? <span>{batchProgress.completed}/{batchProgress.total} clips prepared; commit remains locked until all are ready.</span> : null}
            <button type="button" onClick={cancelPending}>Cancel safely</button>
          </div>
        ) : null}

        {inspection ? (
          <section className="animation-import-dialog__inspection">
            <div
              className="animation-import-dialog__compatibility"
              data-compatible={
                compatibility?.status !== "INCOMPATIBLE"
                || semanticCompatibility?.status === "COMPATIBLE"
              }
              role="status"
            >
              <strong>
                {compatibility?.status === "EXACT_COPY"
                  ? "Exact rig copy"
                  : compatibility?.status === "RETARGETABLE_SAME_HIERARCHY"
                    ? "Retargetable rig"
                    : semanticCompatibility?.status === "COMPATIBLE"
                      ? "Semantic humanoid retarget"
                      : "Incompatible rig"}
              </strong>
              <span>
                {compatibility?.status === "EXACT_COPY"
                  ? "Rest pose and hierarchy match. Core can copy tracks exactly."
                  : compatibility?.status === "RETARGETABLE_SAME_HIERARCHY"
                    ? "Bone names and parent graph match. Core will apply target rest-pose deltas."
                    : semanticCompatibility?.status === "COMPATIBLE"
                      ? `Mapped ${semanticCompatibility.entries.length} humanoid semantics with ${semanticCompatibility.aliasDictionaryVersion}.`
                      : "Core blocked this donor because the hierarchy cannot be mapped safely."}
              </span>
              {compatibility && compatibility.diagnostics.length > 0 ? (
                <details>
                  <summary>
                    {compatibility.diagnostics.length} rig difference{
                      compatibility.diagnostics.length === 1 ? "" : "s"
                    }
                  </summary>
                  <ul>
                    {compatibility.diagnostics.map((diagnostic) => (
                      <li key={`${diagnostic.code}:${diagnostic.path}`}>
                        {diagnostic.message}<small>{diagnostic.action}</small>
                      </li>
                    ))}
                  </ul>
                </details>
              ) : null}
              {semanticCompatibility?.status === "MANUAL_CONFIRMATION_REQUIRED" ? (
                <p role="status">Semantic retarget needs explicit bone overrides. Use the mapper below; transfer remains blocked until Core confirms the exact donor/target pair.</p>
              ) : null}
              {semanticCompatibility && semanticCompatibility.diagnostics.length > 0 ? (
                <details>
                  <summary>
                    {semanticCompatibility.diagnostics.length} semantic mapping issue{
                      semanticCompatibility.diagnostics.length === 1 ? "" : "s"
                    }
                  </summary>
                  <ul>
                    {semanticCompatibility.diagnostics.map((diagnostic) => (
                      <li key={`semantic:${diagnostic.code}:${diagnostic.path}`}>
                        {diagnostic.message}<small>{diagnostic.action}</small>
                      </li>
                    ))}
                  </ul>
                </details>
              ) : null}
            </div>

            {semanticCompatibility ? (
              <section className="animation-semantic-mapper" aria-labelledby="semantic-mapper-title">
                <header><div><strong id="semantic-mapper-title">Manual humanoid mapper</strong><small>Donor bone â†’ semantic â†’ target bone</small></div><button type="button" onClick={() => setMappingOpen((value) => !value)}>{mappingOpen ? "Hide" : "Edit map"}</button></header>
                {mappingOpen ? <>
                  <div className="animation-semantic-mapper__table" role="table">
                    {HUMANOID_SEMANTICS.map((semantic) => {
                      const draft = mappingDraft[semantic] ?? { donorNodeId: null, targetNodeId: null };
                      const usedDonor = new Set(Object.entries(mappingDraft).filter(([key]) => key !== semantic).map(([, value]) => value.donorNodeId));
                      const usedTarget = new Set(Object.entries(mappingDraft).filter(([key]) => key !== semantic).map(([, value]) => value.targetNodeId));
                      return <div role="row" key={semantic}>
                        <select aria-label={`${semantic} donor bone`} value={draft.donorNodeId ?? ""} onChange={(event) => setMappingDraft({ ...mappingDraft, [semantic]: { ...draft, donorNodeId: event.currentTarget.value === "" ? null : Number(event.currentTarget.value) } })}><option value="">Ignore / unresolved</option>{inspection.rig.map((bone) => <option key={bone.id} value={bone.id} disabled={usedDonor.has(bone.id)}>{bone.name}</option>)}</select>
                        <strong>{semantic.replaceAll("_", " ")}</strong>
                        <select aria-label={`${semantic} target bone`} value={draft.targetNodeId ?? ""} onChange={(event) => setMappingDraft({ ...mappingDraft, [semantic]: { ...draft, targetNodeId: event.currentTarget.value === "" ? null : Number(event.currentTarget.value) } })}><option value="">Ignore / unresolved</option>{targetRig.map((bone) => <option key={bone.id} value={bone.id} disabled={usedTarget.has(bone.id)}>{bone.name}</option>)}</select>
                      </div>;
                    })}
                  </div>
                  <button type="button" disabled={busy !== null} onClick={() => void validateManualMapping()}>Confirm exact semantic map in Core</button>
                </> : null}
              </section>
            ) : null}

            <label>
              Animation clip
              <select
                value={selectedClipName}
                disabled={busy !== null || inspection.clips.length === 0}
                onChange={(event) => {
                  setSelectedClipName(event.currentTarget.value);
                  invalidatePreview();
                }}
              >
                {inspection.clips.map((clip) => (
                  <option key={clip.name} value={clip.name}>
                    {clip.name} · {clip.durationSeconds.toFixed(2)} s · {clip.trackCount} tracks
                  </option>
                ))}
              </select>
            </label>

            {onPrepareBatch ? <section className="animation-batch-picker" aria-labelledby="animation-batch-title">
              <header><div><strong id="animation-batch-title">Batch import</strong><small>Transactional: all selected clips or zero document changes.</small></div><span>{batchSelection.length}/{inspection.clips.length}</span></header>
              <label>Find donor clips <input type="search" value={clipSearch} onChange={(event) => setClipSearch(event.currentTarget.value)} /></label>
              <div className="animation-workbench-tools__actions"><button type="button" onClick={() => setBatchSelection(inspection.clips.map(({ name }) => name))}>Select all</button><button type="button" onClick={() => setBatchSelection([])}>Select none</button></div>
              <div className="animation-batch-picker__clips">{inspection.clips.filter(({ name }) => name.toLowerCase().includes(clipSearch.toLowerCase())).map((clip) => <label key={clip.name}><input type="checkbox" checked={batchSelection.includes(clip.name)} onChange={() => setBatchSelection(batchSelection.includes(clip.name) ? batchSelection.filter((name) => name !== clip.name) : [...batchSelection, clip.name])} />{clip.name}<small>{clip.durationSeconds.toFixed(2)} s</small></label>)}</div>
              <button type="button" disabled={busy !== null || batchSelection.length === 0 || !selectedMode} onClick={() => void prepareBatch()}>Prepare {batchSelection.length} selected clips</button>
              {batchProgress && busy === null ? <p role="status">Batch progress: {batchProgress.completed}/{batchProgress.total}. {preparedBatch ? "Ready for one atomic commit." : "No document changes."}</p> : null}
              {error && busy === null && batchSelection.length > 0 && selectedMode ? <button type="button" onClick={() => void prepareBatch()}>Retry batch from zero</button> : null}
            </section> : null}

            {allowedModes.length > 0 ? (
              <label>
                Transfer mode
                <select
                  value={selectedMode}
                  disabled={busy !== null}
                  onChange={(event) => {
                    setSelectedMode(event.currentTarget.value as AnimationTransferModeV1);
                    invalidatePreview();
                  }}
                >
                  {allowedModes.map((mode) => (
                    <option key={mode} value={mode}>
                      {mode === "EXACT_RIG_COPY_V1"
                        ? "Exact rig copy"
                        : mode === "SAME_HIERARCHY_RETARGET_V1"
                          ? "Same hierarchy retarget"
                          : "Humanoid semantic retarget V2"}
                    </option>
                  ))}
                </select>
                <small>
                  {selectedMode === "SAME_HIERARCHY_RETARGET_V1"
                    ? "Transfers local rest deltas and scales root motion to this model."
                    : selectedMode === "HUMANOID_SEMANTIC_RETARGET_V2"
                      ? "Maps versioned humanoid semantics across differently named skeletons; ambiguous rigs are blocked."
                      : "Preserves donor motion and remaps stable bone identities."}
                </small>
              </label>
            ) : null}

            <dl>
              <div><dt>Animations</dt><dd>{inspection.clips.length}</dd></div>
              <div><dt>Bones</dt><dd>{inspection.rig.length}</dd></div>
              <div>
                <dt>Mapped bones</dt>
                <dd>
                  {selectedMode === "HUMANOID_SEMANTIC_RETARGET_V2"
                    ? semanticCompatibility?.entries.length ?? 0
                    : compatibility?.mapping?.entries.length ?? 0}
                </dd>
              </div>
              <div>
                <dt>Donor SHA-256</dt>
                <dd title={inspection.sourceRevision}>
                  {inspection.sourceRevision.slice(0, 12)}…
                </dd>
              </div>
              <div>
                <dt>Target SHA-256</dt>
                <dd title={compatibility?.targetSourceRevision}>
                  {compatibility?.targetSourceRevision.slice(0, 12)}…
                </dd>
              </div>
            </dl>
          </section>
        ) : null}

        {preparedBatch ? <section className="animation-import-dialog__preview" aria-label="Prepared animation batch preview"><header><div><strong>{preparedBatch.clips.length}/{batchSelection.length} clips ready</strong><small>One atomic commit Â· {preparedBatch.batchFingerprintSha256.slice(0, 12)}â€¦</small></div></header><ul>{preparedBatch.clips.map((item) => <li key={item.clip.id}>{item.donorClipName} â†’ {item.clip.name} Â· {item.clip.tracks.length} tracks</li>)}</ul></section> : null}

        {preparedClip && file && inspection ? (
          <section
            className="animation-import-dialog__preview"
            aria-label="Prepared animation transfer preview"
          >
            <header>
              <div>
                <strong>Preview ready — project not changed</strong>
                <small>Review donor motion and target result before copying.</small>
              </div>
              <span>{preparedClip.lengthSeconds.toFixed(2)} s</span>
            </header>
            {renderPreview?.(
              preparedClip,
              file,
              selectedClipName,
              inspection.sourceRevision,
            ) ?? <p>{preparedClip.tracks.length} target tracks are ready.</p>}
            {preparedClip.source.retarget ? (
              <dl>
                <div>
                  <dt>Mode</dt>
                  <dd>
                    {preparedClip.source.retarget.mode === "HUMANOID_SEMANTIC_RETARGET_V2"
                      ? "Humanoid semantic V2"
                      : "Same hierarchy"}
                  </dd>
                </div>
                <div>
                  <dt>Root scale</dt>
                  <dd>{preparedClip.source.retarget.rootMotionScale.toFixed(4)}</dd>
                </div>
                <div>
                  <dt>Motion SHA</dt>
                  <dd title={preparedClip.source.retarget.outputMotionFingerprintSha256}>
                    {preparedClip.source.retarget.outputMotionFingerprintSha256.slice(0, 12)}…
                  </dd>
                </div>
              </dl>
            ) : null}
          </section>
        ) : null}

        {error ? (
          <p className="animation-import-dialog__error" role="alert">{error}</p>
        ) : null}

        <footer>
          <button type="button" disabled={busy !== null} onClick={onClose}>
            Cancel
          </button>
          {preparedBatch && inspection && selectedMode && onCommitBatch ? (
            <button className="animation-studio-action animation-studio-action--primary" type="button" disabled={busy !== null} onClick={() => { try { onCommitBatch(preparedBatch, inspection.sourceRevision, selectedMode); onClose(); } catch (cause: unknown) { setError(cause instanceof Error ? cause.message : String(cause)); setPreparedBatch(null); } }}>Commit {preparedBatch.clips.length} clips atomically</button>
          ) : preparedClip ? (
            <button
              className="animation-studio-action animation-studio-action--primary"
              type="button"
              disabled={busy !== null}
              onClick={commitSelected}
            >
              {busy === "COMMITTING"
                ? "Copying…"
                : selectedMode !== "EXACT_RIG_COPY_V1"
                  ? "Copy retargeted to Custom"
                  : "Copy exact to Custom"}
            </button>
          ) : (
            <button
              className="animation-studio-action animation-studio-action--primary"
              type="button"
              disabled={
                busy !== null
                || !file
                || !selectedClipName
                || !selectedMode
              }
              onClick={() => void prepareSelected()}
            >
              {busy === "PREPARING"
                ? "Preparing preview…"
                : selectedMode !== "EXACT_RIG_COPY_V1"
                  ? "Preview retarget"
                  : "Preview exact copy"}
            </button>
          )}
        </footer>
      </div>
    </div>
  );
}

const HUMANOID_SEMANTICS = [
  "ROOT", "HIPS", "SPINE", "CHEST", "NECK", "HEAD",
  "LEFT_CLAVICLE", "LEFT_UPPER_ARM", "LEFT_LOWER_ARM", "LEFT_HAND",
  "RIGHT_CLAVICLE", "RIGHT_UPPER_ARM", "RIGHT_LOWER_ARM", "RIGHT_HAND",
  "LEFT_UPPER_LEG", "LEFT_LOWER_LEG", "LEFT_FOOT", "LEFT_TOE",
  "RIGHT_UPPER_LEG", "RIGHT_LOWER_LEG", "RIGHT_FOOT", "RIGHT_TOE",
] as const;
