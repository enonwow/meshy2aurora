import type { WorkerArtifact } from "../../worker/types";

function extension(kind: WorkerArtifact["kind"]) {
  switch (kind) {
    case "HAK": return ".hak";
    case "MODEL": return ".mdl";
    case "JSON_REPORT": return ".json";
  }
}

function validate(artifact: WorkerArtifact) {
  if (artifact.provenance !== "M2A_WASM_WORKER") {
    throw new Error("Only canonical Worker artifacts may be downloaded");
  }
  if (
    artifact.fileName.includes("/") ||
    artifact.fileName.includes("\\") ||
    !artifact.fileName.toLowerCase().endsWith(extension(artifact.kind))
  ) {
    throw new Error(`Invalid artifact filename: ${artifact.fileName}`);
  }
  if (artifact.bytes.byteLength !== artifact.byteLength) {
    throw new Error(`Byte-length mismatch for ${artifact.fileName}`);
  }
  if (!/^[a-f0-9]{64}$/.test(artifact.sha256)) {
    throw new Error(`Invalid SHA-256 metadata for ${artifact.fileName}`);
  }
}

async function sha256(bytes: ArrayBuffer) {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

export async function downloadWorkerArtifact(artifact: WorkerArtifact) {
  validate(artifact);
  if (await sha256(artifact.bytes) !== artifact.sha256) {
    throw new Error(`SHA-256 mismatch for ${artifact.fileName}`);
  }
  const url = URL.createObjectURL(new Blob([artifact.bytes], { type: artifact.mediaType }));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = artifact.fileName;
  anchor.rel = "noopener";
  anchor.click();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

interface Props {
  artifacts: WorkerArtifact[];
  onError: (message: string) => void;
}

export function ArtifactDownloads({ artifacts, onError }: Props) {
  const download = (artifact: WorkerArtifact) => {
    void downloadWorkerArtifact(artifact).catch((error: unknown) => {
      onError(error instanceof Error ? error.message : String(error));
    });
  };
  return (
    <section className="panel" aria-label="Canonical Worker artifact downloads">
      <div className="status">
        <strong>GENERATED ARTIFACTS</strong>
        <span>Exact bytes returned by m2a-wasm Worker</span>
      </div>
      {artifacts.length === 0 ? <p>No canonical artifacts are available yet.</p> : (
        <ul>{artifacts.map((artifact) => (
          <li key={artifact.artifactId}>
            <span><strong>{artifact.fileName}</strong><br />{artifact.byteLength.toLocaleString()} bytes</span>
            <code title={artifact.sha256}>{artifact.sha256}</code>
            <button type="button" onClick={() => download(artifact)}>Download</button>
          </li>
        ))}</ul>
      )}
    </section>
  );
}
