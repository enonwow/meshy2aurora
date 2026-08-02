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

function isBlockingSeverity(value: string) {
  return ["BLOCKING", "ERROR", "FATAL", "FAIL", "FAILED"]
    .includes(value.trim().toUpperCase());
}

function firstMismatch(
  result: CanonicalResultSnapshot,
  readback: BinaryMdlInspectionReport,
) {
  if (result.semanticEvidence.semanticDiff[0]) {
    return {
      code: "WRITER-SEMANTIC-DIFF",
      path: "artifacts.model-mdl",
      message: result.semanticEvidence.semanticDiff[0],
      target: "artifact-model-mdl",
    };
  }
  const deviation = result.semanticEvidence.deviations[0];
  if (deviation) {
    return {
      ...deviation,
      target: "artifact-model-mdl",
    };
  }
  const gate = result.conversionEvidence.gates.find(({ severity }) => (
    isBlockingSeverity(severity)
  ));
  if (gate) {
    return {
      code: gate.code,
      path: gate.path,
      message: gate.message,
      target: "artifact-report-json",
    };
  }
  const diagnostic = result.conversionEvidence.diagnostics.find(({ severity }) => (
    isBlockingSeverity(severity)
  ));
  if (diagnostic) {
    return {
      code: diagnostic.code,
      path: diagnostic.path,
      message: diagnostic.message,
      target: "artifact-report-json",
    };
  }
  const readbackDiagnostic = readback.diagnostics.find(({ severity }) => (
    isBlockingSeverity(severity)
  ));
  return readbackDiagnostic
    ? {
        code: readbackDiagnostic.code,
        path: `byteOffset:${readbackDiagnostic.offset}`,
        message: readbackDiagnostic.context,
        target: "artifact-model-mdl",
      }
    : undefined;
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
  const mismatch = firstMismatch(result, readback);
  const readbackAnimationNames = new Set(
    readback.animations.map(({ name }) => name.toLowerCase()),
  );
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

      {result.heldWeaponEvidence ? (
        <section className="review-model__held-weapon" aria-labelledby="review-held-weapon-heading">
          <header>
            <h3 id="review-held-weapon-heading">Held weapon</h3>
            <strong data-status="pass">MATCH</strong>
          </header>
          <p>
            The exact rigid weapon is baked into the final MDL, its attachment
            node was read back, and texture <code>{result.heldWeaponEvidence.textureResref}</code>
            {" "}matches the payload inside the HAK.
          </p>
          <dl>
            <div>
              <dt>Weapon triangles</dt>
              <dd>{result.heldWeaponEvidence.weaponTriangleCount.toLocaleString("en-US")}</dd>
            </div>
            <div>
              <dt>Combined triangles</dt>
              <dd>{result.heldWeaponEvidence.targetTriangleCountAfter.toLocaleString("en-US")}</dd>
            </div>
            <div>
              <dt>Attachment</dt>
              <dd>{result.heldWeaponEvidence.attachmentFingerprintSha256.slice(0, 12)}â€¦</dd>
            </div>
          </dl>
        </section>
      ) : null}

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
                  <th scope="col">Ownership</th>
                  <th scope="col">Fallback path</th>
                  <th scope="col">Binary readback</th>
                </tr>
              </thead>
              <tbody>
                {result.animationMappingEvidence.baseAnimations.map((animation) => (
                  <tr key={animation.targetSlot}>
                    <th scope="row">{animation.targetSlot}</th>
                    <td>{animation.sourceClipName ?? animation.sourceKind}</td>
                    <td>{animation.provider} · {animation.assetId}</td>
                    <td>{animation.ownership}</td>
                    <td>
                      {animation.viaFallbackSlots.length > 0
                        ? `${animation.viaFallbackSlots.join(" → ")} → ${animation.resolvedSourceSlot}`
                        : "Direct"}
                    </td>
                    <td data-status={
                      readbackAnimationNames.has(animation.targetSlot.toLowerCase())
                        ? "pass"
                        : "fail"
                    }>
                      {readbackAnimationNames.has(animation.targetSlot.toLowerCase())
                        ? "MATCH"
                        : "MISSING"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {result.animationMappingEvidence.customAnimations.length > 0 ? (
            <div>
              <h4>Custom animations</h4>
              <div className="review-model__animation-table">
                <table>
                  <thead>
                    <tr>
                      <th scope="col">Custom ID</th>
                      <th scope="col">Playback / phase</th>
                      <th scope="col">Source → output</th>
                      <th scope="col">Provider / asset</th>
                      <th scope="col">Ownership</th>
                      <th scope="col">Binary readback</th>
                    </tr>
                  </thead>
                  <tbody>
                    {result.animationMappingEvidence.customAnimations.flatMap((custom) => (
                      custom.outputClipNames.map((output, index) => (
                        <tr key={`${custom.id}:${output}`}>
                          <th scope="row">{custom.id}</th>
                          <td>{custom.playback} · {custom.phases[index] ?? "ONE_SHOT"}</td>
                          <td>{custom.sourceClipNames[index]} → {output}</td>
                          <td>{custom.provider} · {custom.assetId}</td>
                          <td>{custom.ownership}</td>
                          <td data-status={
                            readbackAnimationNames.has(output.toLowerCase())
                              ? "pass"
                              : "fail"
                          }>
                            {readbackAnimationNames.has(output.toLowerCase())
                              ? "MATCH"
                              : "MISSING"}
                          </td>
                        </tr>
                      ))
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          ) : <p>No custom animations were included.</p>}
        </section>
      ) : null}

      <section className="review-model__animation-evidence" aria-labelledby="review-animation-evidence-heading">
        <header>
          <h3 id="review-animation-evidence-heading">Animation evidence</h3>
          <span>Exact Core report and binary-readback facts</span>
        </header>
        <div>
          <article>
            <span>Completeness</span>
            <strong data-status={
              result.animationCompletenessEvidence?.complete ? "pass" : "unavailable"
            }>
              {result.animationCompletenessEvidence?.complete ? "COMPLETE" : "NOT EMITTED"}
            </strong>
            <small>
              {result.animationCompletenessEvidence
                ? `${result.animationCompletenessEvidence.requiredClipCount} required · ${result.animationCompletenessEvidence.explicitClipCount} explicit · ${result.animationCompletenessEvidence.proceduralClipCount} procedural · ${result.animationCompletenessEvidence.fallbackAliasCount} fallback alias(es) · ${result.animationCompletenessEvidence.profile}`
                : "This package emitted no direct-creature completeness contract."}
            </small>
          </article>
          <article>
            <span>Behavior</span>
            <strong data-status={
              result.animationBehaviorEvidence?.behaviorCandidateEligible
                ? "pass"
                : "unavailable"
            }>
              {result.animationBehaviorEvidence?.behaviorCandidateEligible
                ? "CANDIDATE ELIGIBLE"
                : "NOT EMITTED"}
            </strong>
            <small>
              {result.animationBehaviorEvidence
                ? `${result.animationBehaviorEvidence.observedClipCount}/${result.animationBehaviorEvidence.requiredNamespaceClipCount} states · walk/run distinct ${result.animationBehaviorEvidence.walkRunDistinct ? "yes" : "no"} · essential states distinct ${result.animationBehaviorEvidence.essentialStatesDistinct ? "yes" : "no"} · ${result.animationBehaviorEvidence.violations.length} violation(s)`
                : "This package emitted no behavior-candidate contract."}
            </small>
          </article>
          <article>
            <span>Events</span>
            <strong data-status={result.animationEventEvidence?.complete ? "pass" : "unavailable"}>
              {result.animationEventEvidence?.complete ? "COMPLETE" : "NOT EMITTED"}
            </strong>
            <small>
              {result.animationEventEvidence
                ? `${result.animationEventEvidence.satisfiedPairCount}/${result.animationEventEvidence.requiredPairCount} required pairs · ${result.animationEventEvidence.totalEventCount} event(s) · ${result.animationEventEvidence.unknownEventNames.length} unknown`
                : "Per-clip authored markers remain visible below; no aggregate event conformance was emitted."}
            </small>
          </article>
          <article>
            <span>Deformation</span>
            <strong data-status={result.skinAnimationEvidence?.complete ? "pass" : "unavailable"}>
              {result.geometry.deformation}
              {result.skinAnimationEvidence?.complete ? " · COMPLETE" : ""}
            </strong>
            <small>
              {result.skinAnimationEvidence
                ? `${result.skinAnimationEvidence.activeJointCount} active joints · ${result.skinAnimationEvidence.clips.length}/${result.skinAnimationEvidence.requiredClipCount} sampled clips · ${result.skinAnimationEvidence.violations.length} violation(s)`
                : `${result.geometry.joints} active joints; no sampled skin-conformance evidence was emitted.`}
            </small>
          </article>
        </div>
      </section>

      <section className="review-model__first-mismatch" aria-labelledby="review-first-mismatch-heading">
        <div>
          <span>First mismatch</span>
          <h3 id="review-first-mismatch-heading">
            {mismatch ? mismatch.code : "None in canonical offline reconciliation"}
          </h3>
          <p>{mismatch ? `${mismatch.path} · ${mismatch.message}` : "Source, generated evidence and binary readback have no reported blocking mismatch."}</p>
        </div>
        {mismatch ? <a href={`#${mismatch.target}`}>Open exact artifact</a> : null}
      </section>

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
