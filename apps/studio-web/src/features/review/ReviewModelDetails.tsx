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
        {result.demo && (
          <article>
            <span>Demo module / UTC</span>
            <strong data-status="pass">{`${result.demo.moduleResref}.mod`}</strong>
            <small>
              {`Module “${result.demo.moduleDisplayName}” · Area “${result.demo.areaDisplayName}” (${result.demo.areaResref}) · UTC ${result.demo.creatureResref} · appearance row ${result.demo.appearanceRow}`}
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

      {result.skinAccessoryStabilization && (
        <section className="review-model__accessory-audit" aria-label="Accessory skinning audit">
          <header>
            <div>
              <h3>Accessory skinning audit</h3>
              <p>
                {`${result.skinAccessoryStabilization.auditedClipCount} animation clips audited · mode ${result.skinAccessoryStabilization.mode}`}
              </p>
            </div>
            <strong>
              {`${result.skinAccessoryStabilization.stabilizedComponentCount}/${result.skinAccessoryStabilization.detachedComponentCount} detached components stabilized`}
            </strong>
          </header>
          <dl>
            <div><dt>All components</dt><dd>{result.skinAccessoryStabilization.componentCount}</dd></div>
            <div><dt>Risky</dt><dd>{result.skinAccessoryStabilization.riskyComponentCount}</dd></div>
            <div><dt>Changed vertices</dt><dd>{result.skinAccessoryStabilization.changedVertexCount.toLocaleString("en-US")}</dd></div>
            <div><dt>Spatial weld</dt><dd>{result.skinAccessoryStabilization.weldTolerance}</dd></div>
          </dl>
          {result.skinAccessoryStabilization.components.some((component) => !component.isPrimaryBody) && (
            <div className="review-model__accessory-components">
              {result.skinAccessoryStabilization.components
                .filter((component) => !component.isPrimaryBody)
                .map((component) => (
                  <article key={`${component.segmentIndex}:${component.componentIndex}`}>
                    <div>
                      <strong>{`Segment ${component.segmentIndex} · component ${component.componentIndex}`}</strong>
                      <small>
                        {`${component.triangleCount.toLocaleString("en-US")} triangles · ${component.vertexCount.toLocaleString("en-US")} vertices · ${component.action}`}
                      </small>
                    </div>
                    <div>
                      <span>Bone</span>
                      <strong>
                        {`${component.dominantBoneName ?? "none"} → ${component.selectedBoneName ?? "source weights"}`}
                      </strong>
                    </div>
                    <div>
                      <span>Max pair stretch</span>
                      <strong>
                        {`${component.before.maxPairDistanceRatio.toFixed(3)} → ${component.after.maxPairDistanceRatio.toFixed(3)}`}
                      </strong>
                    </div>
                    <div>
                      <span>Metric coverage</span>
                      <strong>
                        {`${component.before.sampledVertexCount}/${component.vertexCount} vertices · ${component.before.vertexSamplingMode}`}
                      </strong>
                    </div>
                    <div>
                      <span>Risk</span>
                      <strong>{component.riskReasons.length > 0 ? component.riskReasons.join(", ") : "none"}</strong>
                    </div>
                  </article>
                ))}
            </div>
          )}
          {result.skinAccessoryStabilization.warnings.length > 0 && (
            <ul className="review-model__accessory-warnings">
              {result.skinAccessoryStabilization.warnings.map((warning, index) => (
                <li key={`${index}:${warning}`}>{warning}</li>
              ))}
            </ul>
          )}
        </section>
      )}

      {result.materialFidelity && (
        <section className="review-model__material-fidelity" aria-label="Material fidelity">
          <header>
            <div>
              <h3>Material fidelity</h3>
              <p>{result.materialFidelity.auroraMaterialProfile}</p>
            </div>
            <strong data-status={result.materialFidelity.unsupportedFields.length === 0 ? "pass" : "warning"}>
              {result.materialFidelity.baseColorFactorBaked
                ? "Base-color factor baked"
                : "Base-color texture preserved"}
            </strong>
          </header>
          <div>
            <article>
              <span>Mapped safely</span>
              <ul>
                {result.materialFidelity.mappedFields.map((field) => (
                  <li key={field}>{field.replaceAll("->", "→")}</li>
                ))}
              </ul>
            </article>
            <article>
              <span>Unsupported in safe classic profile</span>
              {result.materialFidelity.unsupportedFields.length > 0
                ? <ul>{result.materialFidelity.unsupportedFields.map((field) => <li key={field}>{field}</li>)}</ul>
                : <p>None</p>}
            </article>
          </div>
        </section>
      )}

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
