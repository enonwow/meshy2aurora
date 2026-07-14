import { useCallback, useEffect, useRef, useState } from "react";
import { SourceViewport } from "./features/preview/SourceViewport";
import { AuroraReadbackViewport } from "./features/preview/AuroraReadbackViewport";
import { ValidationPanel } from "./features/preview/ValidationPanel";
import { mapReadbackDiagnostics } from "./features/preview/types";
import type { BinaryMdlInspectionReport, ModelPartRef } from "./features/preview/types";
import { ArtifactDownloads } from "./features/downloads/ArtifactDownloads";
import { M7CorpusPanel } from "./features/m7/M7CorpusPanel";
import { CanonicalResultSummary } from "./features/results/CanonicalResultSummary";
import { projectCanonicalResult, type CanonicalResultSnapshot } from "./features/results/projectCanonicalResult";
import { projectCanonicalReadback } from "./features/results/projectReadback";
import { StudioWorkerClient } from "./worker/client";
import type { StudioWorkerRequest, StudioWorkerResponse } from "./worker/types";

type SessionStatus = "EMPTY" | "READY" | "WORKING" | "COMPLETE" | "ERROR";
interface BuiltResult { snapshot: CanonicalResultSnapshot; readback: BinaryMdlInspectionReport }

const requestId = () => crypto.randomUUID();

export function App() {
  const workerRef = useRef<StudioWorkerClient | undefined>(undefined);
  const sessionRevision = useRef(0);
  const sourceRevision = useRef(0);
  const statusRef = useRef<SessionStatus>("EMPTY");
  const [source, setSource] = useState<File>();
  const [appearance, setAppearance] = useState<File>();
  const [status, setStatus] = useState<SessionStatus>("EMPTY");
  const [message, setMessage] = useState("Select local files to begin.");
  const [sourceSha256, setSourceSha256] = useState<string>();
  const [result, setResult] = useState<BuiltResult>();
  const [selectedPart, setSelectedPart] = useState<ModelPartRef>();

  const updateStatus = useCallback((next: SessionStatus) => {
    statusRef.current = next;
    setStatus(next);
  }, []);

  useEffect(() => {
    const worker = new StudioWorkerClient();
    workerRef.current = worker;
    return () => {
      sessionRevision.current += 1;
      worker.dispose();
      if (workerRef.current === worker) workerRef.current = undefined;
    };
  }, []);
  useEffect(() => {
    setResult(undefined);
    setSelectedPart(undefined);
    updateStatus(source && appearance ? "READY" : "EMPTY");
    setMessage(source && appearance ? "Local files ready; no upload occurred." : "Select local files to begin.");
  }, [source, appearance, updateStatus]);

  useEffect(() => {
    setSourceSha256(undefined);
    if (!source) return;
    const worker = workerRef.current;
    if (!worker) return;
    const inspectRevision = sourceRevision.current;
    let cancelled = false;
    void source.arrayBuffer().then((sourceGlb) => {
      if (cancelled || inspectRevision !== sourceRevision.current) return undefined;
      return worker.request(
        { requestId: requestId(), type: "INSPECT_SOURCE", sourceGlb },
        [sourceGlb],
      );
    }).then((response) => {
      if (!response || cancelled || inspectRevision !== sourceRevision.current) return;
      if (!response.ok || response.type !== "SOURCE_INSPECTED") throw new Error("Unexpected source inspection response");
      const ingest = JSON.parse(response.ingestJson) as { ir?: { source?: { sha256?: string } } };
      setSourceSha256(ingest.ir?.source?.sha256);
    }).catch((error: unknown) => {
      if (!cancelled && inspectRevision === sourceRevision.current && statusRef.current !== "WORKING" && statusRef.current !== "COMPLETE") {
        updateStatus("ERROR");
        setMessage(error instanceof Error ? error.message : String(error));
      }
    });
    return () => { cancelled = true; };
  }, [source, updateStatus]);

  const replaceSource = useCallback((file?: File) => {
    sessionRevision.current += 1;
    sourceRevision.current += 1;
    setSource(file);
  }, []);

  const replaceAppearance = useCallback((file?: File) => {
    sessionRevision.current += 1;
    setAppearance(file);
  }, []);

  const reportUiError = useCallback((error: string) => {
    updateStatus("ERROR");
    setMessage(error);
  }, [updateStatus]);

  const requestFromWorker = useCallback((request: StudioWorkerRequest, transfer: Transferable[] = []) => {
    const worker = workerRef.current;
    return worker
      ? worker.request(request, transfer)
      : Promise.reject(new Error("Studio Worker is not ready"));
  }, []);

  const build = async () => {
    if (!source || !appearance) return;
    const worker = workerRef.current;
    if (!worker) {
      reportUiError("Studio Worker is not ready");
      return;
    }
    const buildRevision = sessionRevision.current;
    setResult(undefined);
    setSelectedPart(undefined);
    updateStatus("WORKING");
    setMessage("Canonical Rust/WASM pipeline is running in a Worker.");
    try {
      const sourceGlb = await source.arrayBuffer();
      const appearanceTwoDa = await appearance.arrayBuffer();
      if (buildRevision !== sessionRevision.current) return;
      const response = await worker.request(
        { requestId: requestId(), type: "BUILD_MODEL_PACKAGE", sourceGlb, appearanceTwoDa },
        [sourceGlb, appearanceTwoDa],
      );
      if (!response.ok || response.type !== "MODEL_PACKAGE_BUILT") {
        throw new Error("Unexpected Worker response");
      }
      if (buildRevision !== sessionRevision.current) return;
      const snapshot = projectCanonicalResult(
        response.reportJson,
        response.summaryJson,
        response.manifestJson,
        response.artifacts,
      );
      const readback = projectCanonicalReadback(response.readbackJson);
      if (buildRevision !== sessionRevision.current) return;
      setResult({ snapshot, readback });
      updateStatus("COMPLETE");
      setMessage("Canonical Worker returned model-package bytes and reports.");
    } catch (error) {
      if (buildRevision !== sessionRevision.current) return;
      setResult(undefined);
      updateStatus("ERROR");
      setMessage(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <main>
      <header className="hero">
        <p className="eyebrow">Local-first model converter</p>
        <h1>Meshy2Aurora Studio</h1>
        <p>Selected bytes stay in this browser. There is no upload or backend.</p>
      </header>

      <section className="panel" aria-label="Local file session">
        <div className="status"><strong>{status}</strong><span>{message}</span></div>
        <label>Source Meshy GLB<input type="file" accept=".glb,model/gltf-binary" onChange={(event) => replaceSource(event.target.files?.[0])} /></label>
        <label>Base appearance.2da<input type="file" accept=".2da,text/plain" onChange={(event) => replaceAppearance(event.target.files?.[0])} /></label>
        <dl>
          <div><dt>Source</dt><dd>{source ? `${source.name} · ${source.size} bytes` : "not selected"}</dd></div>
          <div><dt>Table</dt><dd>{appearance ? `${appearance.name} · ${appearance.size} bytes` : "not selected"}</dd></div>
          <div><dt>Execution</dt><dd>Web Worker → public m2a-wasm adapter</dd></div>
        </dl>
        <button type="button" disabled={status !== "READY"} onClick={() => void build()}>Build model package</button>
      </section>

      {result && <CanonicalResultSummary result={result.snapshot} />}

      {source && sourceSha256 && (
        <SourceViewport input={{ provenance: "SOURCE", file: source, sourceSha256 }} onError={reportUiError} />
      )}

      {result && (
        <>
          <AuroraReadbackViewport report={result.readback} selectedPart={selectedPart} onSelectPart={setSelectedPart} onError={reportUiError} />
          <ValidationPanel diagnostics={mapReadbackDiagnostics(result.readback)} selectedPart={selectedPart} onSelectPart={setSelectedPart} />
        </>
      )}

      {result && <ArtifactDownloads artifacts={result.snapshot.artifacts} onError={reportUiError} />}

      <M7CorpusPanel request={requestFromWorker} />
    </main>
  );
}
