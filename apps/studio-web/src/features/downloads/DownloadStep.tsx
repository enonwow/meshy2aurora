import type { WorkerArtifact } from "../../worker/types";
import { ArtifactDownloads } from "./ArtifactDownloads";
import type {
  DownloadManifestV1,
  DownloadReadinessV1,
} from "./downloadManifest";
import "./DownloadStep.css";

interface DownloadStepProps {
  readonly artifacts: WorkerArtifact[];
  readonly manifest: DownloadManifestV1;
  readonly readiness: DownloadReadinessV1;
  readonly onBack: () => void;
  readonly onError: (message: string) => void;
}

export function DownloadStep({
  artifacts,
  manifest,
  readiness,
  onBack,
  onError,
}: DownloadStepProps) {
  return (
    <section
      className="download-step"
      aria-labelledby="download-step-heading"
    >
      <header className="download-step__header">
        <div>
          <p className="eyebrow">Exact current revision</p>
          <h1 id="download-step-heading">Download</h1>
          <p>
            Verify the manifest and SHA-256 inventory, then save the exact
            Worker/WASM artifacts. Owner visual proof remains a separate step.
          </p>
        </div>
        <button type="button" onClick={onBack}>
          Back to Review
        </button>
      </header>
      <ArtifactDownloads
        artifacts={artifacts}
        manifest={manifest}
        readiness={readiness}
        onError={onError}
      />
    </section>
  );
}
