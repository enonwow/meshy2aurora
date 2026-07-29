import type { WorkerArtifact } from "../../worker/types";
import {
  downloadManifestFileNameV1,
  serializeDownloadManifestV1,
  validateDownloadManifestInventoryV1,
  type DownloadManifestV1,
  type DownloadReadinessV1,
} from "./downloadManifest";
import "./ArtifactDownloads.css";

function extension(kind: WorkerArtifact["kind"]) {
  switch (kind) {
    case "HAK": return ".hak";
    case "MODEL": return ".mdl";
    case "MODULE": return ".mod";
    case "WOK": return ".wok";
    case "SET": return ".set";
    case "TEXTURE": return ".tga";
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
  // Attach the transient anchor so Chromium treats the click as a real
  // download gesture.  Detached anchors are unreliable in headless and some
  // hardened browser contexts, which would leave a valid Worker artefact
  // visible but not actually downloadable.
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

export function downloadManifestJsonV1(
  manifest: DownloadManifestV1,
  artifacts: readonly WorkerArtifact[],
) {
  validateDownloadManifestInventoryV1(manifest, artifacts);
  const fileName = downloadManifestFileNameV1(artifacts);
  const url = URL.createObjectURL(new Blob(
    [serializeDownloadManifestV1(manifest)],
    { type: "application/json" },
  ));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName;
  anchor.rel = "noopener";
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

interface Props {
  artifacts: WorkerArtifact[];
  manifest?: DownloadManifestV1;
  readiness?: DownloadReadinessV1;
  onError: (message: string) => void;
}

export function ArtifactDownloads({
  artifacts,
  manifest,
  readiness,
  onError,
}: Props) {
  const allowed = readiness?.allowed ?? true;
  const download = (artifact: WorkerArtifact) => {
    if (!allowed) {
      onError(readiness?.reason ?? "Download is locked.");
      return;
    }
    void downloadWorkerArtifact(artifact).catch((error: unknown) => {
      onError(error instanceof Error ? error.message : String(error));
    });
  };
  const downloadManifest = () => {
    if (!manifest) {
      onError("No project-bound download manifest is available.");
      return;
    }
    if (!allowed) {
      onError(readiness?.reason ?? "Download is locked.");
      return;
    }
    try {
      downloadManifestJsonV1(manifest, artifacts);
    } catch (error) {
      onError(error instanceof Error ? error.message : String(error));
    }
  };
  return (
    <section className="panel" aria-label="Canonical Worker artifact downloads">
      <div className="status">
        <strong>GENERATED ARTIFACTS</strong>
        <span>
          Exact bytes returned by m2a-wasm Worker
          {readiness ? ` · ${readiness.status}` : ""}
        </span>
      </div>
      {readiness ? <p role="status">{readiness.reason}</p> : null}
      {manifest ? (
        <div className="download-manifest">
          <div>
            <strong>{downloadManifestFileNameV1(artifacts)}</strong>
            <span>
              Project {manifest.projectIdentity.projectId} · revision{" "}
              {manifest.projectIdentity.projectRevision} · owner proof pending
            </span>
          </div>
          <button
            type="button"
            onClick={downloadManifest}
            disabled={!allowed}
          >
            Download manifest
          </button>
        </div>
      ) : null}
      {artifacts.length === 0 ? <p>No canonical artifacts are available yet.</p> : (
        <ul>{artifacts.map((artifact) => (
          <li id={`artifact-${artifact.artifactId}`} key={artifact.artifactId}>
            <span><strong>{artifact.fileName}</strong><br />{artifact.byteLength.toLocaleString()} bytes</span>
            <code title={artifact.sha256}>{artifact.sha256}</code>
            <button type="button" onClick={() => download(artifact)} disabled={!allowed}>
              Download
            </button>
          </li>
        ))}</ul>
      )}
    </section>
  );
}
