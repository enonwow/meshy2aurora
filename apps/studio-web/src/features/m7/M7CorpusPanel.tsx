import { useRef, useState } from "react";
import { ArtifactDownloads } from "../downloads/ArtifactDownloads";
import type { StudioWorkerRequest, StudioWorkerResponse, WorkerArtifact } from "../../worker/types";
import { buildM7PayloadEnvelope } from "./envelope";

interface ManifestSample {
  role: string;
  sampleId: string;
  source?: { relativePath?: string } | null;
}

interface ManifestView {
  corpusId?: string;
  samples?: ManifestSample[];
}

interface Props {
  request: (request: StudioWorkerRequest, transfer?: Transferable[]) => Promise<StudioWorkerResponse>;
}

const nextRequestId = () => crypto.randomUUID();

export function M7CorpusPanel({ request }: Props) {
  const revision = useRef(0);
  const [manifestFile, setManifestFile] = useState<File>();
  const [manifestJson, setManifestJson] = useState("");
  const [manifest, setManifest] = useState<ManifestView>();
  const [sources, setSources] = useState<Record<string, File>>({});
  const [appearances, setAppearances] = useState<Record<string, File>>({});
  const [result, setResult] = useState("No M7 operation has run.");
  const [artifacts, setArtifacts] = useState<WorkerArtifact[]>([]);
  const [working, setWorking] = useState(false);

  const resetForInputChange = () => {
    revision.current += 1;
    setWorking(false);
    setArtifacts([]);
    setResult("Inputs changed; run the M7 operation again.");
    return revision.current;
  };

  const selectManifest = async (file?: File) => {
    const selectedRevision = resetForInputChange();
    setManifestFile(file);
    setManifest(undefined);
    setManifestJson("");
    setSources({});
    setAppearances({});
    if (!file) return;
    try {
      const json = await file.text();
      if (selectedRevision !== revision.current) return;
      setManifestJson(json);
      setResult("Local manifest loaded; canonical WASM validation is required.");
    } catch (error) {
      if (selectedRevision !== revision.current) return;
      const message = error instanceof Error ? error.message : String(error);
      setResult(`Manifest read failed: ${message}`);
    }
  };

  const run = async (operation: "VALIDATE" | "INTAKE" | "BUILD") => {
    if (!manifestJson || (operation !== "VALIDATE" && !manifest)) return;
    const runRevision = revision.current;
    setWorking(true);
    setArtifacts([]);
    setResult(`Running M7 ${operation.toLowerCase()}...`);
    try {
      let response: StudioWorkerResponse;
      if (operation === "VALIDATE") {
        response = await request({
          requestId: nextRequestId(),
          type: "VALIDATE_M7_CORPUS",
          manifestJson,
        });
      } else {
        const samples = manifest?.samples ?? [];
        const envelope = await buildM7PayloadEnvelope(
          samples.flatMap((sample) => {
            const relativePath = sample.source?.relativePath;
            const file = relativePath ? sources[relativePath] : undefined;
            return relativePath && file
              ? [{ role: "SOURCE" as const, relativePath, file }]
              : [];
          }),
          samples.flatMap((sample) => {
            const file = appearances[sample.sampleId];
            return sample.role === "RIGGED_HUMANOID_SOURCE_CLIPS" && file
              ? [{ role: "RIGGED_HUMANOID_APPEARANCE_2DA" as const, sampleId: sample.sampleId, file }]
              : [];
          }),
        );
        if (runRevision !== revision.current) return;
        response = await request(
          operation === "INTAKE"
            ? {
                requestId: nextRequestId(),
                type: "INSPECT_M7_CORPUS_INTAKE",
                manifestJson,
                ...envelope,
              }
            : {
                requestId: nextRequestId(),
                type: "BUILD_M7_CORPUS_BATCH",
                manifestJson,
                ...envelope,
              },
          [envelope.payloadBlob],
        );
      }
      if (runRevision !== revision.current) return;
      if (!response.ok) throw new Error(response.message);
      if (response.type === "M7_CORPUS_VALIDATED") {
        const parsed = JSON.parse(response.manifestJson) as ManifestView & { code?: string };
        if (parsed.code) {
          setManifest(undefined);
          setResult(`Manifest: ${parsed.code}`);
        } else {
          setManifest(parsed);
          setResult(`VALID manifest: ${parsed.corpusId ?? "unnamed corpus"}`);
        }
        setArtifacts(response.artifacts);
      } else if (response.type === "M7_CORPUS_INTAKE_INSPECTED") {
        const parsed = JSON.parse(response.intakeJson) as { status?: string; code?: string };
        setResult(`Intake: ${parsed.status ?? parsed.code ?? "UNKNOWN"}`);
        setArtifacts(response.artifacts);
      } else if (response.type === "M7_CORPUS_BATCH_BUILT") {
        const parsed = JSON.parse(response.batchJson) as { report?: { status?: string } };
        setResult(`Batch: ${parsed.report?.status ?? "UNKNOWN"}`);
        setArtifacts(response.artifacts);
      } else {
        throw new Error("Unexpected M7 Worker response");
      }
    } catch (error) {
      if (runRevision === revision.current) {
        const message = error instanceof Error ? error.message : String(error);
        setResult(`M7 Worker error: ${message}`);
      }
    } finally {
      if (runRevision === revision.current) setWorking(false);
    }
  };

  const samples = Array.isArray(manifest?.samples) ? manifest.samples : [];
  return (
    <section className="panel m7-panel" aria-label="M7 corpus session">
      <p className="eyebrow">M7 corpus reports</p>
      <h2>Local manifest and explicit payloads</h2>
      <p>Files stay local. Roles, paths and provenance come only from the selected manifest.</p>
      <label>
        M7 corpus manifest JSON
        <input type="file" accept=".json,application/json" onChange={(event) => void selectManifest(event.target.files?.[0])} />
      </label>
      <p>{manifestFile ? `${manifestFile.name} · ${manifestFile.size} bytes` : "manifest not selected"}</p>

      {samples.map((sample) => (
        <fieldset key={sample.sampleId}>
          <legend>{sample.role} · {sample.sampleId}</legend>
          <p>Declared source: {sample.source?.relativePath ?? "INPUT_DEFERRED"}</p>
          {sample.source?.relativePath && (
            <label>
              Payload for {sample.source.relativePath}
              <input type="file" accept=".glb,model/gltf-binary" onChange={(event) => {
                const file = event.target.files?.[0];
                resetForInputChange();
                setSources((current) => {
                  const next = { ...current };
                  if (file) next[sample.source!.relativePath!] = file;
                  else delete next[sample.source!.relativePath!];
                  return next;
                });
              }} />
            </label>
          )}
          {sample.role === "RIGGED_HUMANOID_SOURCE_CLIPS" && (
            <label>
              appearance.2da for {sample.sampleId}
              <input type="file" accept=".2da,text/plain" onChange={(event) => {
                const file = event.target.files?.[0];
                resetForInputChange();
                setAppearances((current) => {
                  const next = { ...current };
                  if (file) next[sample.sampleId] = file;
                  else delete next[sample.sampleId];
                  return next;
                });
              }} />
            </label>
          )}
        </fieldset>
      ))}

      <div className="button-row">
        <button type="button" disabled={!manifestJson || working} onClick={() => void run("VALIDATE")}>Validate manifest</button>
        <button type="button" disabled={!manifest || working} onClick={() => void run("INTAKE")}>Inspect intake</button>
        <button type="button" disabled={!manifest || working} onClick={() => void run("BUILD")}>Build M7 reports</button>
      </div>
      <div className="status"><strong>{working ? "WORKING" : "REPORT"}</strong><span>{result}</span></div>
      <p>M7-V5 acceptance remains deferred. This panel never claims M7 DONE.</p>
      <ArtifactDownloads
        artifacts={artifacts}
        onError={(message) => setResult(`M7 artifact error: ${message}`)}
      />
    </section>
  );
}
