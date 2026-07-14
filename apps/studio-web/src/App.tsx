import { useEffect, useMemo, useState } from "react";
import { SourceViewport } from "./features/preview/SourceViewport";
import { AuroraReadbackViewport } from "./features/preview/AuroraReadbackViewport";
import { ValidationPanel } from "./features/preview/ValidationPanel";
import { mapReadbackDiagnostics } from "./features/preview/types";
import type { BinaryMdlInspectionReport, ModelPartRef } from "./features/preview/types";
import { ArtifactDownloads } from "./features/downloads/ArtifactDownloads";
import { StudioWorkerClient } from "./worker/client";
import type { StudioWorkerResponse, WorkerArtifact } from "./worker/types";

type SessionStatus = "EMPTY" | "READY" | "WORKING" | "COMPLETE" | "ERROR";

const requestId = () => crypto.randomUUID();

export function App() {
  const worker = useMemo(() => new StudioWorkerClient(), []);
  const [source, setSource] = useState<File>();
  const [appearance, setAppearance] = useState<File>();
  const [status, setStatus] = useState<SessionStatus>("EMPTY");
  const [message, setMessage] = useState("Select local files to begin.");
  const [artifacts, setArtifacts] = useState<WorkerArtifact[]>([]);
  const [sourceSha256, setSourceSha256] = useState<string>();
  const [readback, setReadback] = useState<BinaryMdlInspectionReport>();
  const [selectedPart, setSelectedPart] = useState<ModelPartRef>();

  useEffect(() => () => worker.dispose(), [worker]);
  useEffect(() => {
    setArtifacts([]);
    setReadback(undefined);
    setSelectedPart(undefined);
    setStatus(source && appearance ? "READY" : "EMPTY");
    setMessage(source && appearance ? "Local files ready; no upload occurred." : "Select local files to begin.");
  }, [source, appearance]);

  useEffect(() => {
    setSourceSha256(undefined);
    if (!source) return;
    let cancelled = false;
    void source.arrayBuffer().then((sourceGlb) =>
      worker.request(
        { requestId: requestId(), type: "INSPECT_SOURCE", sourceGlb },
        [sourceGlb],
      ),
    ).then((response) => {
      if (cancelled || !response.ok || response.type !== "SOURCE_INSPECTED") return;
      const ingest = JSON.parse(response.ingestJson) as { ir?: { source?: { sha256?: string } } };
      setSourceSha256(ingest.ir?.source?.sha256);
    }).catch((error: unknown) => {
      if (!cancelled) {
        setStatus("ERROR");
        setMessage(error instanceof Error ? error.message : String(error));
      }
    });
    return () => { cancelled = true; };
  }, [source, worker]);

  const build = async () => {
    if (!source || !appearance) return;
    setStatus("WORKING");
    setMessage("Canonical Rust/WASM pipeline is running in a Worker.");
    try {
      const sourceGlb = await source.arrayBuffer();
      const appearanceTwoDa = await appearance.arrayBuffer();
      const response = await worker.request(
        { requestId: requestId(), type: "BUILD_MODEL_PACKAGE", sourceGlb, appearanceTwoDa },
        [sourceGlb, appearanceTwoDa],
      );
      if (!response.ok || response.type !== "MODEL_PACKAGE_BUILT") {
        throw new Error("Unexpected Worker response");
      }
      setArtifacts(response.artifacts);
      setReadback(JSON.parse(response.readbackJson) as BinaryMdlInspectionReport);
      setStatus("COMPLETE");
      setMessage("Canonical Worker returned model-package bytes and reports.");
    } catch (error) {
      setStatus("ERROR");
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
        <label>Source Meshy GLB<input type="file" accept=".glb,model/gltf-binary" onChange={(event) => setSource(event.target.files?.[0])} /></label>
        <label>Base appearance.2da<input type="file" accept=".2da,text/plain" onChange={(event) => setAppearance(event.target.files?.[0])} /></label>
        <dl>
          <div><dt>Source</dt><dd>{source ? `${source.name} · ${source.size} bytes` : "not selected"}</dd></div>
          <div><dt>Table</dt><dd>{appearance ? `${appearance.name} · ${appearance.size} bytes` : "not selected"}</dd></div>
          <div><dt>Execution</dt><dd>Web Worker → public m2a-wasm adapter</dd></div>
        </dl>
        <button type="button" disabled={status !== "READY"} onClick={() => void build()}>Build model package</button>
      </section>

      {source && sourceSha256 && (
        <SourceViewport input={{ provenance: "SOURCE", file: source, sourceSha256 }} />
      )}

      {readback && (
        <>
          <AuroraReadbackViewport report={readback} selectedPart={selectedPart} onSelectPart={setSelectedPart} />
          <ValidationPanel diagnostics={mapReadbackDiagnostics(readback)} selectedPart={selectedPart} onSelectPart={setSelectedPart} />
        </>
      )}

      <ArtifactDownloads artifacts={artifacts} onError={(error) => { setStatus("ERROR"); setMessage(error); }} />
    </main>
  );
}
