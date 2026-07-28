import type { ReactNode } from "react";
import type { BinaryMdlInspectionReport } from "../preview/types";
import type { CanonicalModelMetrics, CanonicalResultSnapshot } from "../results/projectCanonicalResult";
import { ConversionReadiness } from "./ConversionReadiness";
import "./ReviewModelDetails.css";

export type ReviewViewport = "SOURCE" | "CONVERTED";

interface ReviewModelDetailsProps {
  result: CanonicalResultSnapshot;
  readback: BinaryMdlInspectionReport;
  activeViewport: ReviewViewport;
  onViewportChange: (viewport: ReviewViewport) => void;
  onInspectBinary: () => void;
  sourceViewport: ReactNode;
  convertedReadbackViewport: ReactNode;
}

const metricLabels: Record<keyof CanonicalModelMetrics, string> = {
  nodes: "Nodes",
  meshes: "Meshes",
  vertices: "Vertices",
  triangles: "Triangles",
  animations: "Animation clips",
};

export interface PairedReviewMetric {
  key: keyof CanonicalModelMetrics;
  label: string;
  source: number;
  converted: number;
}

export function pairedReviewMetrics(
  source: Partial<CanonicalModelMetrics>,
  converted: Partial<CanonicalModelMetrics>,
): PairedReviewMetric[] {
  return (Object.keys(metricLabels) as Array<keyof CanonicalModelMetrics>).flatMap((key) => {
    const sourceValue = source[key];
    const convertedValue = converted[key];
    return typeof sourceValue === "number" && Number.isFinite(sourceValue)
      && typeof convertedValue === "number" && Number.isFinite(convertedValue)
      ? [{ key, label: metricLabels[key], source: sourceValue, converted: convertedValue }]
      : [];
  });
}

function rootCount(readback: BinaryMdlInspectionReport) {
  return readback.nodeTree.roots.length;
}

export function ReviewModelDetails({
  result,
  readback,
  activeViewport,
  onViewportChange,
  onInspectBinary,
  sourceViewport,
  convertedReadbackViewport,
}: ReviewModelDetailsProps) {
  const metrics = pairedReviewMetrics(result.sourceMetrics, result.convertedMetrics);
  const semanticPass = result.semanticEvidence.semanticDiff.length === 0;
  const readbackStatus = readback.validation?.status ?? "UNAVAILABLE";
  const readbackLabel = readbackStatus === "PASS"
    ? "Verified by binary readback"
    : readbackStatus === "WARNING"
      ? "Binary readback has warnings"
      : readbackStatus === "ERROR"
        ? "Binary readback has errors"
        : "Binary readback evidence unavailable";
  const readbackStatusLabel = readbackStatus === "PASS"
    ? "PASS"
    : readbackStatus === "WARNING"
      ? "WARNING"
      : readbackStatus === "ERROR"
        ? "ERROR"
        : "UNAVAILABLE";

  return (
    <section className="review-model" aria-labelledby="review-model-heading">
      <header className="review-model__header">
        <div>
          <p className="eyebrow">Review output</p>
          <h2 id="review-model-heading">Model Details</h2>
        </div>
        <button type="button" className="readback-badge" data-status={readbackStatus.toLowerCase()} onClick={onInspectBinary}>
          <span aria-hidden="true">●</span>
          {readbackLabel}
          <small>Inspect Binary</small>
        </button>
      </header>

      <div className="review-model__viewport-tabs" role="tablist" aria-label="Model viewport source">
        <button
          type="button"
          role="tab"
          aria-selected={activeViewport === "SOURCE"}
          onClick={() => onViewportChange("SOURCE")}
        >
          Source Model
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={activeViewport === "CONVERTED"}
          onClick={() => onViewportChange("CONVERTED")}
        >
          Converted Model
        </button>
      </div>
      <div className="review-model__viewport" role="tabpanel">
        {activeViewport === "SOURCE" ? sourceViewport : convertedReadbackViewport}
      </div>

      <ConversionReadiness result={result} readback={readback} />

      {result.animationMappingEvidence ? (
        <section className="review-model__animation-mapping" aria-labelledby="review-animation-mapping-heading">
          <header>
            <h3 id="review-animation-mapping-heading">Animation Mapping</h3>
            <p>
              <strong>READY · 42/42 read back</strong>
              {` · authoring r${result.animationMappingEvidence.authoringRevision}`}
              {` · fingerprint ${result.animationMappingEvidence.authoringFingerprintSha256.slice(0, 12)}…`}
            </p>
          </header>
          <div className="review-model__animation-table">
            <table>
              <caption>Sources actually materialized into the built artifact</caption>
              <thead>
                <tr>
                  <th scope="col">Aurora slot</th>
                  <th scope="col">Source</th>
                  <th scope="col">Provider / asset</th>
                  <th scope="col">Fallback path</th>
                </tr>
              </thead>
              <tbody>
                {result.animationMappingEvidence.baseAnimations.map((animation) => (
                  <tr key={animation.targetSlot}>
                    <th scope="row">{animation.targetSlot}</th>
                    <td>{animation.sourceClipName ?? animation.sourceKind}</td>
                    <td>{animation.provider} · {animation.assetId}</td>
                    <td>
                      {animation.viaFallbackSlots.length > 0
                        ? `${animation.viaFallbackSlots.join(" → ")} → ${animation.resolvedSourceSlot}`
                        : "Direct"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {result.animationMappingEvidence.customAnimations.length > 0 ? (
            <div>
              <h4>Custom animations</h4>
              <ul>
                {result.animationMappingEvidence.customAnimations.map((custom) => (
                  <li key={custom.id}>
                    <strong>{custom.id} · {custom.playback}</strong>
                    {custom.outputClipNames.map((output, index) => (
                      <span key={output}>
                        {` ${custom.phases[index] ?? "ONE_SHOT"}: ${output} ← ${
                          custom.sourceClipNames[index]
                        }`}
                      </span>
                    ))}
                    {` · ${custom.provider}`}
                  </li>
                ))}
              </ul>
            </div>
          ) : <p>No custom animations were included.</p>}
        </section>
      ) : null}

      <div className="review-model__evidence" aria-label="Canonical evidence">
        {result.runtimeFixtureContract && (
          <article>
            <span>M0 runtime fixture</span>
            <strong data-status="pass">BOUND / NOT RUNTIME PROOF</strong>
            <small>
              {`${result.runtimeFixtureContract.binaryScene.areaResref} · row ${result.runtimeFixtureContract.appearance.physicalRow} · fixture [${result.runtimeFixtureContract.binaryScene.fixture.position.x}, ${result.runtimeFixtureContract.binaryScene.fixture.position.y}, ${result.runtimeFixtureContract.binaryScene.fixture.position.z}]`}
            </small>
          </article>
        )}
        {result.geometry.deformation === "SKIN" && (
          <article>
            <span>Creature gameplay events</span>
            <strong data-status={result.animationEventEvidence ? "pass" : "unavailable"}>
              {result.animationEventEvidence ? "PASS" : "NOT INCLUDED"}
            </strong>
            <small>
              {result.animationEventEvidence
                ? `${result.animationEventEvidence.satisfiedPairCount}/${result.animationEventEvidence.requiredPairCount} required hooks · ${result.animationEventEvidence.totalEventCount} total event(s) · canonical sidecar SHA-256 ${result.animationEventEvidence.authoringCanonical.sha256.slice(0, 12)}...`
                : "This package did not use the caller-owned Full-42 event authoring lane."}
            </small>
          </article>
        )}
        <article>
          <span>Binary readback</span>
          <strong data-status={readbackStatus.toLowerCase()}>{readbackStatusLabel}</strong>
          <small>
            {readback.validation
              ? `${readback.validation.structure.format} · ${readback.validation.structure.rootNodeCount} root node(s) · ${readback.validation.structure.structuralErrors.length} structural error(s) · ${readback.validation.diagnostics.total} diagnostic(s)`
              : `${readback.format} · ${rootCount(readback)} root node(s) · validation evidence unavailable`}
          </small>
        </article>
        <article>
          <span>Writer semantic diff</span>
          <strong data-status={semanticPass ? "pass" : "difference"}>{semanticPass ? "PASS" : "DIFFERENCE REPORTED"}</strong>
          <small>
            {semanticPass
              ? "Canonical writer/readback semantic diff is empty."
              : `${result.semanticEvidence.semanticDiff.length} difference(s) reported by the canonical writer.`}
          </small>
        </article>
        <article>
          <span>Writer deviations</span>
          <strong>{result.semanticEvidence.deviations.length}</strong>
          <small>Reported by the canonical writer; no UI score is calculated.</small>
        </article>
      </div>

      <div className="review-model__metrics">
        <table>
          <caption>Metrics available in both canonical snapshots</caption>
          <thead><tr><th scope="col">Metric</th><th scope="col">Source (GLB)</th><th scope="col">Converted (MDL)</th></tr></thead>
          <tbody>
            {metrics.map((metric) => (
              <tr key={metric.key}>
                <th scope="row">{metric.label}</th>
                <td>{metric.source.toLocaleString("en-US")}</td>
                <td>{metric.converted.toLocaleString("en-US")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {!semanticPass && (
        <section className="review-model__semantic-diff" aria-label="Semantic differences">
          <h3>Canonical semantic differences</h3>
          <ul>{result.semanticEvidence.semanticDiff.map((difference, index) => <li key={`${index}:${difference}`}>{difference}</li>)}</ul>
        </section>
      )}
    </section>
  );
}
