import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { StudioWorkerClient } from "../../worker/client";
import type { StudioWorkerResponse } from "../../worker/types";
import { projectCanonicalReadback } from "../results/projectReadback";
import type { BinaryMdlInspectionReport } from "../preview/types";
import type { CreatureSourceForwardV1 } from "../../worker/types";
import {
  addHakFilesToSupermodelCatalogV1,
  addLooseMdlFilesToSupermodelCatalogV1,
  directoryNwnResourceSourceV1,
  effectiveCatalogModelV1,
  fileListNwnResourceSourceV1,
  loadSupermodelModelBytesV1,
  scanNwnSupermodelsV1,
  type SupermodelCatalogProgressV1,
  type SupermodelCatalogSessionV1,
  type SupermodelWorkerClientV1,
} from "./filesystem";
import { SupermodelPreviewViewport } from "./SupermodelPreviewViewport";
import { RigJointEditor } from "./RigJointEditor";
import {
  parseAppliedSupermodelReportV2,
  type AppliedSupermodelPreviewV2,
} from "./appliedPreview";
import {
  parseReferenceSupermodelRigAuthoringV2,
  parseReferenceSupermodelTargetRigV1,
  type ReferenceSupermodelRigAuthoringDocumentV2,
} from "./rigAuthoring";
import { loadExactReferenceSupermodelChainV2 } from "./exactChain";
import {
  filterSupermodelCatalogEntriesV1,
  type SupermodelCatalogEntryV1,
  type SupermodelCatalogFilterV1,
  type SupermodelModelV1,
} from "./types";
import "./supermodel-library.css";

interface DisposableSupermodelWorkerV1 extends SupermodelWorkerClientV1 {
  dispose(): void;
}

interface Props {
  onBack: () => void;
  onSelectCandidate: (entry: SupermodelCatalogEntryV1) => void;
  onAppliedPreview?: (preview: AppliedSupermodelPreviewV2) => void;
  selectedCandidateResref?: string;
  sourceFile?: File;
  sourceSha256?: string;
  sourceForward?: CreatureSourceForwardV1;
  initialAppliedPreview?: AppliedSupermodelPreviewV2;
  initialSession?: SupermodelCatalogSessionV1;
  onSessionChange?: (session: SupermodelCatalogSessionV1) => void;
  createWorker?: () => DisposableSupermodelWorkerV1;
}

const defaultCreateWorkerV1 = () => new StudioWorkerClient();

type LibraryPhaseV1 = "IDLE" | "SCANNING" | "READY" | "ERROR";
type PreviewModeV1 = "NATIVE" | "REPRESENTATIVE" | "APPLIED_SOURCE";

let previewRequestSequence = 0;
function previewRequestId() {
  previewRequestSequence += 1;
  return `supermodel-preview-${previewRequestSequence}`;
}

function successful<T extends StudioWorkerResponse["type"]>(
  response: StudioWorkerResponse,
  expected: T,
): Extract<StudioWorkerResponse, { ok: true; type: T }> {
  if (!response.ok) throw new Error(response.message);
  if (response.type !== expected) throw new Error(`Unexpected Studio Worker response: ${response.type}`);
  return response as Extract<StudioWorkerResponse, { ok: true; type: T }>;
}

async function inspectPreviewModel(
  worker: SupermodelWorkerClientV1,
  session: SupermodelCatalogSessionV1,
  model: SupermodelModelV1,
) {
  if (model.header.format !== "BINARY") {
    throw new Error(`${model.resref} is an ASCII MDL. It is catalogued, but the binary preview currently supports binary MDL resources only.`);
  }
  const mdlBytes = await loadSupermodelModelBytesV1(session, model);
  const response = successful(await worker.request({
    requestId: previewRequestId(),
    type: "INSPECT_BINARY_MDL",
    mdlBytes,
  }, [mdlBytes]), "BINARY_MDL_INSPECTED");
  return projectCanonicalReadback(response.reportJson);
}

export function SupermodelLibrary({
  onBack,
  onSelectCandidate,
  onAppliedPreview,
  selectedCandidateResref,
  sourceFile,
  sourceSha256,
  sourceForward = "POSITIVE_Z",
  initialAppliedPreview,
  initialSession,
  onSessionChange,
  createWorker = defaultCreateWorkerV1,
}: Props) {
  const workerRef = useRef<DisposableSupermodelWorkerV1 | undefined>(undefined);
  const scanAbortRef = useRef<AbortController | undefined>(undefined);
  const [phase, setPhase] = useState<LibraryPhaseV1>(initialSession ? "READY" : "IDLE");
  const [session, setSession] = useState(initialSession);
  const [progress, setProgress] = useState<SupermodelCatalogProgressV1>();
  const [error, setError] = useState<string>();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<SupermodelCatalogFilterV1>("ALL");
  const [selected, setSelected] = useState<SupermodelCatalogEntryV1>();
  const [reports, setReports] = useState<BinaryMdlInspectionReport[]>([]);
  const [carrierReport, setCarrierReport] = useState<BinaryMdlInspectionReport>();
  const [carrierResref, setCarrierResref] = useState("");
  const [previewMode, setPreviewMode] = useState<PreviewModeV1>("NATIVE");
  const [previewLoading, setPreviewLoading] = useState(false);
  const [extensionLoading, setExtensionLoading] = useState(false);
  const [applicationLoading, setApplicationLoading] = useState(false);
  const [appliedPreview, setAppliedPreview] = useState(initialAppliedPreview);
  const [previewError, setPreviewError] = useState<string>();
  const [previewLimitations, setPreviewLimitations] = useState<string[]>([]);
  const [carrierLimitation, setCarrierLimitation] = useState<string>();
  const [selectedRigNodeName, setSelectedRigNodeName] = useState<string>();
  const [experimentalAllowExcessiveSkinBranchRepair, setExperimentalAllowExcessiveSkinBranchRepair] = useState(
    initialAppliedPreview?.experimentalAllowExcessiveSkinBranchRepair ?? false,
  );

  useEffect(() => {
    const worker = createWorker();
    workerRef.current = worker;
    return () => {
      scanAbortRef.current?.abort();
      worker.dispose();
      if (workerRef.current === worker) workerRef.current = undefined;
    };
  }, [createWorker]);

  useEffect(() => {
    setAppliedPreview(initialAppliedPreview);
    if (initialAppliedPreview) {
      setExperimentalAllowExcessiveSkinBranchRepair(
        initialAppliedPreview.experimentalAllowExcessiveSkinBranchRepair,
      );
    }
  }, [initialAppliedPreview]);

  const startScan = async (source: Parameters<typeof scanNwnSupermodelsV1>[0]) => {
    const worker = workerRef.current;
    if (!worker) return;
    scanAbortRef.current?.abort();
    const controller = new AbortController();
    scanAbortRef.current = controller;
    setPhase("SCANNING");
    setSession(undefined);
    setSelected(undefined);
    setReports([]);
    setCarrierReport(undefined);
    setError(undefined);
    try {
      const result = await scanNwnSupermodelsV1(source, worker, setProgress, controller.signal);
      if (controller.signal.aborted) return;
      setSession(result);
      onSessionChange?.(result);
      setPhase("READY");
    } catch (scanError) {
      if (controller.signal.aborted) return;
      setError(scanError instanceof Error ? scanError.message : String(scanError));
      setPhase("ERROR");
    }
  };

  useEffect(() => {
    if (!selected || !session) return;
    const worker = workerRef.current;
    if (!worker) return;
    let active = true;
    setReports([]);
    setCarrierReport(undefined);
    setPreviewError(undefined);
    setPreviewLimitations([]);
    setCarrierLimitation(undefined);
    setPreviewLoading(true);
    setCarrierResref(selected.children[0] ?? "");
    setPreviewMode(selected.children.length > 0 ? "REPRESENTATIVE" : "NATIVE");
    void (async () => {
      const resolved: BinaryMdlInspectionReport[] = [];
      const limitations: string[] = [];
      for (const resref of selected.chain) {
        const model = effectiveCatalogModelV1(session, resref);
        if (!model) {
          limitations.push(`Brak zasobu ${resref} w odziedziczonym łańcuchu.`);
          continue;
        }
        try {
          resolved.push(await inspectPreviewModel(worker, session, model));
        } catch (loadError) {
          limitations.push(loadError instanceof Error ? loadError.message : String(loadError));
        }
      }
      if (active) {
        setReports(resolved);
        setPreviewLimitations(limitations);
      }
    })().catch((loadError: unknown) => {
      if (active) setPreviewError(loadError instanceof Error ? loadError.message : String(loadError));
    }).finally(() => { if (active) setPreviewLoading(false); });
    return () => { active = false; };
  }, [selected, session]);

  useEffect(() => {
    if (previewMode !== "REPRESENTATIVE" || !carrierResref || !session) {
      setCarrierReport(undefined);
      setCarrierLimitation(undefined);
      return;
    }
    const worker = workerRef.current;
    const model = effectiveCatalogModelV1(session, carrierResref);
    if (!worker || !model) return;
    let active = true;
    setCarrierReport(undefined);
    setCarrierLimitation(undefined);
    void inspectPreviewModel(worker, session, model)
      .then((report) => { if (active) setCarrierReport(report); })
      .catch((loadError: unknown) => {
        if (active) setCarrierLimitation(loadError instanceof Error ? loadError.message : String(loadError));
      });
    return () => { active = false; };
  }, [carrierResref, previewMode, session]);

  const entries = useMemo(
    () => filterSupermodelCatalogEntriesV1(session?.catalog.entries ?? [], query, filter),
    [filter, query, session?.catalog.entries],
  );
  const percent = progress?.totalModels
    ? Math.min(100, Math.round(progress.scannedModels / progress.totalModels * 100))
    : 0;

  const extendCatalog = async (
    operation: (session: SupermodelCatalogSessionV1, worker: SupermodelWorkerClientV1) => Promise<SupermodelCatalogSessionV1>,
  ) => {
    const worker = workerRef.current;
    if (!session || !worker) return;
    setExtensionLoading(true);
    setError(undefined);
    try {
      const result = await operation(session, worker);
      setSession(result);
      onSessionChange?.(result);
      setSelected(undefined);
    } catch (extensionError) {
      setError(extensionError instanceof Error ? extensionError.message : String(extensionError));
    } finally {
      setExtensionLoading(false);
    }
  };

  const applySelectedSupermodel = async () => {
    const worker = workerRef.current;
    if (!worker || !session || !selected || !sourceFile || !sourceSha256) return;
    setApplicationLoading(true);
    setPreviewError(undefined);
    try {
      const [sourceGlb, referenceChain] = await Promise.all([
        sourceFile.arrayBuffer(),
        loadExactReferenceSupermodelChainV2(session, selected.resref),
      ]);
      const response = successful(await worker.request({
        requestId: previewRequestId(),
        type: "BUILD_REFERENCE_SUPERMODEL_APPLIED_PREVIEW",
        selectedSupermodelResref: selected.resref,
        sourceGlb,
        referenceChainBlob: referenceChain.blob,
        referenceChainJson: referenceChain.descriptorsJson,
        sourceForward,
        experimentalAllowExcessiveSkinBranchRepair,
      }, [sourceGlb, referenceChain.blob]), "REFERENCE_SUPERMODEL_APPLIED_PREVIEW_BUILT");
      const report = parseAppliedSupermodelReportV2(response.applyReportJson);
      const target = projectCanonicalReadback(response.readbackJson);
      if (target.model?.supermodelName.toLocaleLowerCase() !== selected.resref.toLocaleLowerCase()) {
        throw new Error(`Applied preview readback does not reference ${selected.resref}`);
      }
      const preview: AppliedSupermodelPreviewV2 = {
        sourceName: sourceFile.name,
        sourceSha256,
        supermodelResref: selected.resref.toLocaleLowerCase(),
        experimentalAllowExcessiveSkinBranchRepair:
          report.experimentalAllowExcessiveSkinBranchRepair,
        target,
        report,
        rigAuthoring: parseReferenceSupermodelRigAuthoringV2(response.authoringJson),
        targetRig: parseReferenceSupermodelTargetRigV1(response.targetRigJson),
        diagnosticArtifacts: response.artifacts,
      };
      setAppliedPreview(preview);
      onSelectCandidate(selected);
      onAppliedPreview?.(preview);
      setPreviewMode("APPLIED_SOURCE");
    } catch (applyError) {
      setPreviewError(applyError instanceof Error ? applyError.message : String(applyError));
    } finally {
      setApplicationLoading(false);
    }
  };

  const applyAuthoredRig = async (document: ReferenceSupermodelRigAuthoringDocumentV2) => {
    const worker = workerRef.current;
    if (!worker || !session || !selected || !sourceFile || !sourceSha256) return;
    setApplicationLoading(true);
    setPreviewError(undefined);
    try {
      const [sourceGlb, referenceChain] = await Promise.all([
        sourceFile.arrayBuffer(),
        loadExactReferenceSupermodelChainV2(session, selected.resref, appliedPreview?.report.exactChain),
      ]);
      const response = successful(await worker.request({
        requestId: previewRequestId(),
        type: "BUILD_REFERENCE_SUPERMODEL_AUTHORED_PREVIEW",
        selectedSupermodelResref: selected.resref,
        sourceGlb,
        referenceChainBlob: referenceChain.blob,
        referenceChainJson: referenceChain.descriptorsJson,
        sourceForward,
        authoringJson: JSON.stringify(document),
        experimentalAllowExcessiveSkinBranchRepair,
      }, [sourceGlb, referenceChain.blob]), "REFERENCE_SUPERMODEL_APPLIED_PREVIEW_BUILT");
      const target = projectCanonicalReadback(response.readbackJson);
      if (target.model?.supermodelName.toLocaleLowerCase() !== selected.resref.toLocaleLowerCase()) {
        throw new Error(`Authored preview readback does not reference ${selected.resref}`);
      }
      const report = parseAppliedSupermodelReportV2(response.applyReportJson);
      const preview: AppliedSupermodelPreviewV2 = {
        sourceName: sourceFile.name,
        sourceSha256,
        supermodelResref: selected.resref.toLocaleLowerCase(),
        experimentalAllowExcessiveSkinBranchRepair:
          report.experimentalAllowExcessiveSkinBranchRepair,
        target,
        report,
        rigAuthoring: parseReferenceSupermodelRigAuthoringV2(response.authoringJson),
        targetRig: parseReferenceSupermodelTargetRigV1(response.targetRigJson),
        diagnosticArtifacts: response.artifacts,
      };
      setAppliedPreview(preview);
      onAppliedPreview?.(preview);
      setPreviewMode("APPLIED_SOURCE");
    } catch (applyError) {
      const message = applyError instanceof Error ? applyError.message : String(applyError);
      setPreviewError(message);
      throw applyError;
    } finally {
      setApplicationLoading(false);
    }
  };

  const handleSelectRigNode = useCallback((node?: { readonly name: string }) => {
    if (node && appliedPreview?.targetRig.nodes.some((targetNode) => (
      targetNode.name.toLocaleLowerCase() === node.name.toLocaleLowerCase()
    ))) setSelectedRigNodeName(node.name);
  }, [appliedPreview?.targetRig.nodes]);

  return (
    <section className="supermodel-library" aria-labelledby="supermodel-library-heading">
      <header className="supermodel-library__header">
        <div>
          <p className="eyebrow">Creature authoring · local read-only reference</p>
          <h1 id="supermodel-library-heading">Biblioteka supermodeli</h1>
          <p>Przeglądaj supermodele z lokalnych zasobów przed wyborem i bez modyfikowania modelu.</p>
        </div>
        <button type="button" className="button button--secondary" onClick={onBack}>Wróć do źródeł</button>
      </header>

      {phase === "IDLE" || phase === "ERROR" ? (
        <section className="supermodel-library__connect">
          <h2>Połącz lokalną instalację NWN</h2>
          <p>Studio odczyta wyłącznie wybrane pliki `nwn_base.key` i BIF. Pliki nie są wysyłane ani kopiowane do projektu.</p>
          {error ? <p role="alert" className="supermodel-library__error">{error}</p> : null}
          {window.showDirectoryPicker ? (
            <button
              type="button"
              className="button button--primary"
              onClick={() => void window.showDirectoryPicker?.({ mode: "read" })
                .then((handle) => startScan(directoryNwnResourceSourceV1(handle)))
                .catch((pickerError: unknown) => {
                  if (pickerError instanceof DOMException && pickerError.name === "AbortError") return;
                  setError(pickerError instanceof Error ? pickerError.message : String(pickerError));
                })}
            >Wybierz folder instalacji NWN</button>
          ) : null}
          <label className="button button--secondary supermodel-library__folder-fallback">
            Wybierz folder jako zestaw plików
            <input
              type="file"
              multiple
              {...({ webkitdirectory: "", directory: "" } as Record<string, string>)}
              onChange={(event) => {
                const files = event.currentTarget.files;
                if (files?.length) void startScan(fileListNwnResourceSourceV1(files));
                event.currentTarget.value = "";
              }}
            />
          </label>
        </section>
      ) : null}

      {phase === "SCANNING" ? (
        <section className="supermodel-library__scanning" aria-live="polite">
          <h2>Indeksowanie lokalnych modeli</h2>
          <progress max={100} value={percent}>{percent}%</progress>
          <strong>{percent}% · {progress?.scannedModels.toLocaleString("pl-PL") ?? 0}/{progress?.totalModels.toLocaleString("pl-PL") ?? "?"} MDL</strong>
          <span>{progress?.currentContainer ?? "Odczyt indeksu KEY"}</span>
          <button type="button" className="button button--secondary" onClick={() => {
            scanAbortRef.current?.abort();
            setPhase("IDLE");
          }}>Anuluj</button>
        </section>
      ) : null}

      {phase === "READY" && session ? (
        <>
          <section className="supermodel-library__summary" aria-label="Kompletność katalogu">
            <article><span>Status</span><strong data-status={session.catalog.completeness.toLocaleLowerCase()}>{session.catalog.completeness}</strong></article>
            <article><span>Supermodele</span><strong>{session.catalog.supermodelCount.toLocaleString("pl-PL")}</strong></article>
            <article><span>Przeskanowane MDL</span><strong>{session.catalog.scannedModelCount.toLocaleString("pl-PL")}/{session.catalog.declaredModelCount.toLocaleString("pl-PL")}</strong></article>
            <article><span>Błędy nagłówków</span><strong>{session.catalog.failedModelCount.toLocaleString("pl-PL")}</strong></article>
            <article><span>Źródło</span><strong>{session.sourceLabel}</strong></article>
          </section>
          <section className="supermodel-library__sources" aria-label="Dodatkowe źródła supermodeli">
            <div><strong>Rozszerz katalog</strong><span>Pierwszy wybrany HAK ma najwyższy priorytet w podglądzie; jest to jawna kolejność biblioteki, nie deklaracja parity kolejności wielu HAK w silniku.</span></div>
            <label className="button button--secondary">
              Dodaj HAK-i
              <input type="file" accept=".hak" multiple disabled={extensionLoading} onChange={(event) => {
                const files = [...(event.currentTarget.files ?? [])];
                if (files.length) void extendCatalog((current, worker) => addHakFilesToSupermodelCatalogV1(current, files, worker));
                event.currentTarget.value = "";
              }} />
            </label>
            <label className="button button--secondary">
              Dodaj folder override
              <input type="file" accept=".mdl" multiple disabled={extensionLoading} {...({ webkitdirectory: "", directory: "" } as Record<string, string>)} onChange={(event) => {
                const files = [...(event.currentTarget.files ?? [])];
                if (files.length) void extendCatalog((current, worker) => addLooseMdlFilesToSupermodelCatalogV1(current, files, worker));
                event.currentTarget.value = "";
              }} />
            </label>
            <label className="button button--secondary">
              Dodaj luźne MDL
              <input type="file" accept=".mdl" multiple disabled={extensionLoading} onChange={(event) => {
                const files = [...(event.currentTarget.files ?? [])];
                if (files.length) void extendCatalog((current, worker) => addLooseMdlFilesToSupermodelCatalogV1(current, files, worker));
                event.currentTarget.value = "";
              }} />
            </label>
            {extensionLoading ? <span>Indeksowanie dodatkowych źródeł…</span> : null}
            {error ? <p role="alert" className="supermodel-library__error">{error}</p> : null}
          </section>
          <div className="supermodel-library__workspace">
            <aside className="supermodel-library__catalog">
              <div className="supermodel-library__filters">
                <label>Szukaj<input aria-label="Szukaj supermodelu" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="np. horror, wolf…" /></label>
                <label>Status<select aria-label="Filtr supermodeli" value={filter} onChange={(event) => setFilter(event.target.value as SupermodelCatalogFilterV1)}>
                  <option value="ALL">Wszystkie</option>
                  <option value="WITH_ANIMATIONS">Z lokalnymi animacjami</option>
                  <option value="RESOLVED">Rozwiązane</option>
                  <option value="MISSING">Brakujące</option>
                  <option value="CYCLIC">Cykliczne</option>
                </select></label>
              </div>
              <p>{entries.length.toLocaleString("pl-PL")} wyników</p>
              <div className="supermodel-library__entries">
                {entries.map((entry) => (
                  <button
                    type="button"
                    key={entry.resref.toLocaleLowerCase()}
                    data-supermodel={entry.resref}
                    data-selected={selected?.resref === entry.resref || undefined}
                    onClick={() => setSelected(entry)}
                  >
                    <span><strong>{entry.resref}</strong><small>{entry.status} · {entry.resource?.header.format ?? "UNRESOLVED"}</small></span>
                    <span><strong>{entry.resource?.header.localAnimationCount ?? 0}</strong><small>animacji</small></span>
                    <span><strong>{entry.children.length}</strong><small>modeli potomnych</small></span>
                  </button>
                ))}
              </div>
            </aside>

            <main className="supermodel-library__preview">
              {!selected ? (
                <div className="empty-state"><strong>Wybierz supermodel z katalogu</strong><span>Podgląd nie zmienia konfiguracji eksportu.</span></div>
              ) : (
                <>
                  <header className="supermodel-library__selection">
                    <div><p className="eyebrow">Wybrany wpis</p><h2>{selected.resref}</h2><p>{selected.chain.join(" → ")}</p></div>
                    <div className="supermodel-library__selection-actions">
                      <button
                        type="button"
                        className="button button--secondary"
                        disabled={selected.status !== "RESOLVED"}
                        onClick={() => onSelectCandidate(selected)}
                      >{selectedCandidateResref?.toLocaleLowerCase() === selected.resref.toLocaleLowerCase() ? "Wybrany kandydat" : "Wybierz jako kandydata (bez nakładania)"}</button>
                      <button
                        type="button"
                        className="button button--primary"
                        disabled={
                          applicationLoading
                          || selected.status !== "RESOLVED"
                          || !sourceFile
                          || !sourceSha256
                        }
                        onClick={() => void applySelectedSupermodel()}
                      >{applicationLoading ? "Analiza, nakładanie i walidacja…" : `Analizuj i zastosuj ${selected.resref}`}</button>
                    </div>
                  </header>
                  <label className="supermodel-library__experimental-skinning">
                    <input
                      type="checkbox"
                      checked={experimentalAllowExcessiveSkinBranchRepair}
                      onChange={(event) => setExperimentalAllowExcessiveSkinBranchRepair(event.target.checked)}
                    />
                    <span>
                      Pozwól na eksperymentalną naprawę skinningu ponad limit 5%
                      <small>
                        Omija wyłącznie limit liczby przepisywanych wierzchołków. Walidacja szkieletu,
                        wag, bind pose i animacji nadal może zablokować wynik.
                      </small>
                    </span>
                  </label>
                  <dl className="supermodel-library__metadata">
                    <div><dt>Status</dt><dd>{selected.status}</dd></div>
                    <div><dt>Lokalne animacje</dt><dd>{selected.resource?.header.localAnimationCount ?? 0}</dd></div>
                    <div><dt>Animation scale</dt><dd>{selected.resource?.header.animationScale ?? "—"}</dd></div>
                    <div><dt>Źródło</dt><dd>{selected.resource?.containerName ?? "nierozwiązane"}</dd></div>
                  </dl>
                  {!sourceFile || !sourceSha256 ? (
                    <p className="supermodel-library__truth">Wczytaj i poczekaj na inspekcję nowego pliku GLB, aby przeanalizować oraz zastosować wybrany supermodel.</p>
                  ) : null}
                  {appliedPreview
                    && appliedPreview.sourceSha256 === sourceSha256
                    && appliedPreview.supermodelResref === selected.resref.toLocaleLowerCase() ? (
                      <section className="supermodel-library__applied" aria-label="Applied supermodel evidence">
                        <strong>{appliedPreview.report.motionCompatible ? "APPLIED TO SOURCE" : "DIAGNOSTIC APPLIED PREVIEW"}</strong>
                        <span>{appliedPreview.sourceName} → {appliedPreview.supermodelResref}</span>
                        <span>MDL {appliedPreview.report.modelSha256.slice(0, 12)}… · reference {appliedPreview.report.referenceSha256.slice(0, 12)}…</span>
                        <span>{appliedPreview.report.inheritedAnimationCount} odziedziczonych animacji · motion quality {appliedPreview.report.motionQualityStatus}</span>
                        {appliedPreview.report.rigAnalysis.skinning?.branchBoundaryRepairLimitExceeded ? (
                          <span className="supermodel-library__applied-warning">
                            Tryb eksperymentalny dopuścił naprawę do {appliedPreview.report.rigAnalysis.skinning.branchBoundaryRepairMaximumObservedVertexCount.toLocaleString("pl-PL")} wierzchołków
                            {" "}przy limicie {appliedPreview.report.rigAnalysis.skinning.branchBoundaryRepairLimitVertexCount.toLocaleString("pl-PL")}.
                          </span>
                        ) : appliedPreview.experimentalAllowExcessiveSkinBranchRepair ? (
                          <span>Tryb eksperymentalny był aktywny, ale raport nie wykazał przekroczenia limitu 5%.</span>
                        ) : null}
                        <span>
                          Analiza struktury: {appliedPreview.report.structuralAnalysis.carrierNodeCount} nodów,
                          {" "}{appliedPreview.report.structuralAnalysis.carrierControllerCount} kontrolerów,
                          {" "}rig: {appliedPreview.report.rigAnalysis.activeWeightedBoneCount}/{appliedPreview.report.rigAnalysis.allowedBoneCount} aktywnych kości ważonych,
                          {" "}format {appliedPreview.report.referenceFormat}.
                        </span>
                        <span>
                          Carrier {appliedPreview.report.fullCarrierCoverage ? "pełny" : "NIEPEŁNY"};
                          {" "}jointy wymagane {appliedPreview.report.requiredJointCoverage ? "pełne" : "NIEPEŁNE"};
                          {" "}wagi skóry {appliedPreview.report.skinInfluenceCoverage ? "pełne" : "NIEPEŁNE"};
                          {" "}klipy {appliedPreview.report.motionQuality.sampledClipCount}/{appliedPreview.report.motionQuality.requiredClipCount};
                          {" "}joint×clip {appliedPreview.report.motionQuality.jointClipPassCount}/{appliedPreview.report.motionQuality.jointClipRequiredCount}.
                        </span>
                        {appliedPreview.report.rigAnalysis.unweightedRequiredJointNames.length ? (
                          <span className="supermodel-library__applied-warning">
                            Brak aktywnych wpływów: {appliedPreview.report.rigAnalysis.unweightedRequiredJointNames.join(", ")}.
                          </span>
                        ) : null}
                        {appliedPreview.report.rigAnalysis.passiveUnweightedJointNames.length ? (
                          <span>Jointy pasywne bez wag (legalne): {appliedPreview.report.rigAnalysis.passiveUnweightedJointNames.join(", ")}.</span>
                        ) : null}
                        {!appliedPreview.report.motionCompatible ? (
                          <span className="supermodel-library__applied-warning">
                            Model można oglądać z animacjami, kośćmi i jointami, ale nie przeszedł bramki jakości ruchu i nie jest gotowy do eksportu.
                            {" "}Hard edges {appliedPreview.report.motionQuality.edgeOutsideHardLimitCount}/{appliedPreview.report.motionQuality.edgeOutsideHardAllowedCount},
                            {" "}collapsed triangles {appliedPreview.report.motionQuality.triangleAreaCollapseCount}/{appliedPreview.report.motionQuality.triangleAreaCollapseAllowedCount},
                            {" "}seams {appliedPreview.report.motionQuality.seamPairViolationCount}/{appliedPreview.report.motionQuality.seamPairAllowedCount}.
                            {" "}Surface gate {appliedPreview.report.motionQuality.surfaceSeamGateStatus};
                            {" "}visible anchor trajectory violations {appliedPreview.report.motionQuality.visibleAnchorMotionViolationCount}.
                          </span>
                        ) : null}
                      </section>
                    ) : null}
                  {appliedPreview
                    && appliedPreview.sourceSha256 === sourceSha256
                    && appliedPreview.supermodelResref === selected.resref.toLocaleLowerCase() ? (
                      <RigJointEditor
                        preview={appliedPreview}
                        selectedNodeName={selectedRigNodeName}
                        busy={applicationLoading}
                        onSelectNodeName={setSelectedRigNodeName}
                        onApply={applyAuthoredRig}
                      />
                    ) : null}
                  {selected.children.length > 0 ? (
                    <div className="supermodel-library__preview-mode">
                      <label><input type="radio" checked={previewMode === "NATIVE"} onChange={() => setPreviewMode("NATIVE")} />Sam supermodel</label>
                      <label><input type="radio" checked={previewMode === "REPRESENTATIVE"} onChange={() => setPreviewMode("REPRESENTATIVE")} />Na modelu reprezentatywnym</label>
                      {appliedPreview
                        && appliedPreview.sourceSha256 === sourceSha256
                        && appliedPreview.supermodelResref === selected.resref.toLocaleLowerCase() ? (
                          <label><input type="radio" checked={previewMode === "APPLIED_SOURCE"} onChange={() => setPreviewMode("APPLIED_SOURCE")} />Na nowym modelu</label>
                        ) : null}
                      {previewMode === "REPRESENTATIVE" ? <label>Model potomny<select aria-label="Model reprezentatywny" value={carrierResref} onChange={(event) => setCarrierResref(event.target.value)}>{selected.children.map((child) => <option key={child} value={child}>{child}</option>)}</select></label> : null}
                    </div>
                  ) : null}
                  {previewLoading ? <p>Wczytywanie dokładnego MDL i łańcucha animacji…</p> : null}
                  {previewError ? <p role="alert" className="supermodel-library__error">{previewError}</p> : null}
                  {previewLimitations.length > 0 || carrierLimitation ? (
                    <div className="supermodel-library__limitations" role="status">
                      <strong>Ograniczenia tego podglądu</strong>
                      <ul>
                        {previewLimitations.map((limitation) => <li key={limitation}>{limitation}</li>)}
                        {carrierLimitation ? <li>{carrierLimitation}</li> : null}
                      </ul>
                    </div>
                  ) : null}
                  {reports.length > 0 || carrierReport || (previewMode === "APPLIED_SOURCE" && appliedPreview) ? (
                    <SupermodelPreviewViewport
                      reports={reports}
                      carrier={previewMode === "APPLIED_SOURCE"
                        ? appliedPreview?.target
                        : previewMode === "REPRESENTATIVE" ? carrierReport : undefined}
                      detail={previewMode === "APPLIED_SOURCE" && appliedPreview
                        ? `${appliedPreview.sourceName} po nałożeniu ${appliedPreview.supermodelResref}`
                        : previewMode === "REPRESENTATIVE"
                          ? `Model ${carrierResref || "potomny"} z animacjami ${selected.resref}`
                          : `Natywny zasób ${selected.resref}; geometria może być nieobecna`}
                      onError={setPreviewError}
                      onSelectRigNode={handleSelectRigNode}
                    />
                  ) : null}
                  <p className="supermodel-library__truth">Podgląd Three.js jest lokalnym odczytem MDL, nie dowodem zachowania w Toolset/NWN.</p>
                </>
              )}
            </main>
          </div>
        </>
      ) : null}
    </section>
  );
}
