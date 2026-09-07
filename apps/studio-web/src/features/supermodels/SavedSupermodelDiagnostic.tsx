import { useEffect, useMemo, useState } from "react";
import { projectCanonicalReadback } from "../results/projectReadback";
import { SupermodelPreviewViewport } from "./SupermodelPreviewViewport";
import {
  parseAppliedSupermodelReportV2,
  type AppliedSupermodelReportV2,
} from "./appliedPreview";
import {
  parseReferenceSupermodelRigAuthoringV1,
  parseReferenceSupermodelRigAuthoringV2,
  parseReferenceSupermodelTargetRigV1,
  type ReferenceSupermodelRigAuthoringDocumentV1,
  type ReferenceSupermodelRigAuthoringDocumentV2,
  type ReferenceSupermodelTargetRigV1,
} from "./rigAuthoring";
import type { BinaryMdlInspectionReport } from "../preview/types";
import { classifyReadbackRigV1 } from "../preview/rigOverlay";
import "./supermodel-library.css";

interface SavedSupermodelDiagnosticUrlsV1 {
  readonly report: string;
  readonly readback: string;
  readonly authoring: string;
  readonly targetRig: string;
}

interface SavedSupermodelDiagnosticDetailsV1 {
  readonly perClipDeformationCoverage: boolean;
  readonly perComponentDeformationCoverage: boolean;
  readonly failedComponentSampleCount: number;
  readonly worstMaxEdgeRatio: number | null;
  readonly worstMinEdgeRatio: number | null;
  readonly worstMaxTriangleAreaRatio: number | null;
  readonly worstMinTriangleAreaRatio: number | null;
}

const CANONICAL_DIAGNOSTIC_PATH = /^\/@fs\/C:\/Projects\/meshy2aurora\/artifacts\/diagnostics\/[A-Za-z0-9._-]+$/i;

export function savedSupermodelDiagnosticUrlsV1(
  baseUrl: string,
  studioUrl = window.location.href,
): SavedSupermodelDiagnosticUrlsV1 {
  const studio = new URL(studioUrl);
  const diagnostic = new URL(baseUrl.replace(/\/$/, ""), studio);
  if (diagnostic.origin !== studio.origin) {
    throw new Error("Saved diagnostics must use the same Studio origin");
  }
  const path = decodeURIComponent(diagnostic.pathname).replace(/\\/g, "/");
  if (!CANONICAL_DIAGNOSTIC_PATH.test(path)) {
    throw new Error("Saved diagnostics must resolve to one canonical diagnostic directory");
  }
  const file = (name: string) => new URL(`${path}/${name}`, studio).toString();
  return {
    report: file("base-preview-report.json"),
    readback: file("base-preview-readback.json"),
    authoring: file("base-authoring.json"),
    targetRig: file("base-target-rig.json"),
  };
}

function diagnosticRecord(value: unknown, path: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Invalid saved supermodel diagnostic at ${path}`);
  }
  return value as Record<string, unknown>;
}

function diagnosticBoolean(value: unknown, path: string) {
  if (typeof value !== "boolean") throw new Error(`Invalid saved supermodel diagnostic at ${path}`);
  return value;
}

function diagnosticFinite(value: unknown, path: string) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`Invalid saved supermodel diagnostic at ${path}`);
  }
  return value;
}

export function parseSavedSupermodelDiagnosticDetailsV1(
  reportJson: string,
): SavedSupermodelDiagnosticDetailsV1 {
  let parsed: unknown;
  try { parsed = JSON.parse(reportJson); } catch { throw new Error("Invalid saved supermodel diagnostic JSON"); }
  const report = diagnosticRecord(parsed, "report");
  const quality = diagnosticRecord(report.motionQuality, "report.motionQuality");
  if (!Array.isArray(quality.clips)) throw new Error("Invalid saved supermodel diagnostic at report.motionQuality.clips");
  const failed = quality.clips.flatMap((clip, clipIndex) => {
    const clipValue = diagnosticRecord(clip, `report.motionQuality.clips[${clipIndex}]`);
    if (!Array.isArray(clipValue.components)) {
      throw new Error(`Invalid saved supermodel diagnostic at report.motionQuality.clips[${clipIndex}].components`);
    }
    return clipValue.components.flatMap((component, componentIndex) => {
      const path = `report.motionQuality.clips[${clipIndex}].components[${componentIndex}]`;
      const value = diagnosticRecord(component, path);
      const pass = diagnosticBoolean(value.pass, `${path}.pass`);
      if (pass) return [];
      return [{
        maxEdgeRatio: diagnosticFinite(value.maxEdgeRatio, `${path}.maxEdgeRatio`),
        minEdgeRatio: diagnosticFinite(value.minEdgeRatio, `${path}.minEdgeRatio`),
        maxTriangleAreaRatio: diagnosticFinite(value.maxTriangleAreaRatio, `${path}.maxTriangleAreaRatio`),
        minTriangleAreaRatio: diagnosticFinite(value.minTriangleAreaRatio, `${path}.minTriangleAreaRatio`),
      }];
    });
  });
  const maximum = (values: readonly number[]) => values.length ? Math.max(...values) : null;
  const minimum = (values: readonly number[]) => values.length ? Math.min(...values) : null;
  return {
    perClipDeformationCoverage: diagnosticBoolean(quality.perClipDeformationCoverage, "report.motionQuality.perClipDeformationCoverage"),
    perComponentDeformationCoverage: diagnosticBoolean(quality.perComponentDeformationCoverage, "report.motionQuality.perComponentDeformationCoverage"),
    failedComponentSampleCount: failed.length,
    worstMaxEdgeRatio: maximum(failed.map((item) => item.maxEdgeRatio)),
    worstMinEdgeRatio: minimum(failed.map((item) => item.minEdgeRatio)),
    worstMaxTriangleAreaRatio: maximum(failed.map((item) => item.maxTriangleAreaRatio)),
    worstMinTriangleAreaRatio: minimum(failed.map((item) => item.minTriangleAreaRatio)),
  };
}

async function fetchEvidenceFile(url: string, signal: AbortSignal) {
  const response = await fetch(url, { signal });
  if (!response.ok) throw new Error(`${response.status} ${response.statusText}: ${url}`);
  return response.text();
}

async function loadSavedPreviewV1(
  baseUrl: string,
  signal: AbortSignal,
): Promise<SavedSupermodelPreviewV1> {
  const urls = savedSupermodelDiagnosticUrlsV1(baseUrl);
  const [reportJson, readbackJson, authoringJson, targetRigJson] = await Promise.all([
    fetchEvidenceFile(urls.report, signal),
    fetchEvidenceFile(urls.readback, signal),
    fetchEvidenceFile(urls.authoring, signal),
    fetchEvidenceFile(urls.targetRig, signal),
  ]);
  const report = parseAppliedSupermodelReportV2(reportJson);
  const diagnosticDetails = parseSavedSupermodelDiagnosticDetailsV1(reportJson);
  const authoringSchemaVersion = (JSON.parse(authoringJson) as { schemaVersion?: unknown }).schemaVersion;
  const rigAuthoring = authoringSchemaVersion === 1
    ? parseReferenceSupermodelRigAuthoringV1(authoringJson)
    : parseReferenceSupermodelRigAuthoringV2(authoringJson);
  return {
    sourceName: "source.glb",
    sourceSha256: rigAuthoring.sourceSha256,
    supermodelResref: report.supermodelResref.toLocaleLowerCase(),
    target: projectCanonicalReadback(readbackJson),
    report,
    diagnosticDetails,
    rigAuthoring,
    targetRig: parseReferenceSupermodelTargetRigV1(targetRigJson),
  };
}

interface SavedSupermodelPreviewV1 {
  readonly sourceName: string;
  readonly sourceSha256: string;
  readonly supermodelResref: string;
  readonly target: BinaryMdlInspectionReport;
  readonly report: AppliedSupermodelReportV2;
  readonly diagnosticDetails: SavedSupermodelDiagnosticDetailsV1;
  readonly rigAuthoring: ReferenceSupermodelRigAuthoringDocumentV1 | ReferenceSupermodelRigAuthoringDocumentV2;
  readonly targetRig: ReferenceSupermodelTargetRigV1;
}

interface Props {
  readonly baseUrl: string;
}

export function SavedSupermodelDiagnostic({ baseUrl }: Props) {
  const [preview, setPreview] = useState<SavedSupermodelPreviewV1>();
  const [error, setError] = useState<string>();
  const rigInventory = useMemo(
    () => preview ? classifyReadbackRigV1(preview.target.nodeTree.roots) : undefined,
    [preview],
  );

  useEffect(() => {
    const controller = new AbortController();
    setPreview(undefined);
    setError(undefined);
    void loadSavedPreviewV1(baseUrl, controller.signal)
      .then(setPreview)
      .catch((reason: unknown) => {
        if (!controller.signal.aborted) setError(reason instanceof Error ? reason.message : String(reason));
      });
    return () => controller.abort();
  }, [baseUrl]);

  return (
    <main className="saved-supermodel-diagnostic">
      <header className="supermodel-library__header">
        <div>
          <p className="eyebrow">Meshy2Aurora Studio · immutable offline evidence</p>
          <h1>Diagnostyka nałożenia supermodelu</h1>
          <p>Ten ekran odczytuje zapisany wynik normalnego pipeline’u; nie zastępuje wyniku danymi demonstracyjnymi.</p>
        </div>
        <a className="button button--secondary" href="./">Wróć do Studio</a>
      </header>

      {!preview && !error ? <p role="status">Wczytywanie raportu, readbacku i rigu…</p> : null}
      {error ? <p role="alert" className="supermodel-library__error">Nie można wczytać diagnostyki: {error}</p> : null}

      {preview ? (
        <section className="saved-supermodel-diagnostic__workspace" aria-label="Saved supermodel diagnostic evidence">
          <section className="supermodel-library__applied" aria-label="Applied supermodel evidence">
            <strong>{preview.report.admissionV3?.status ?? "LEGACY DIAGNOSTIC — NO CENTRAL ADMISSION"}</strong>
            <span>{preview.sourceName} → {preview.supermodelResref}</span>
            <span>source {preview.sourceSha256.slice(0, 12)}… · MDL {preview.report.modelSha256.slice(0, 12)}…</span>
            <span>{preview.report.inheritedAnimationCount} odziedziczonych animacji · motion quality {preview.report.motionQualityStatus}</span>
            <span>
              Rig: {preview.report.rigAnalysis.activeWeightedBoneCount}/{preview.report.rigAnalysis.allowedBoneCount} aktywnych kości ważonych;
              {" "}{preview.targetRig.nodes.length} jointów;
              {" "}joint×clip {preview.report.motionQuality.jointClipPassCount}/{preview.report.motionQuality.jointClipRequiredCount}.
            </span>
            {preview.report.admissionV3?.status !== "PASS" ? (
              <span className="supermodel-library__applied-warning">
                Eksport jest zablokowany: {preview.report.admissionV3?.blockingCodes.join(", ") || "ten starszy raport nie zawiera wymaganej bramki admission V3"}.
              </span>
            ) : null}
          </section>

          <section className="saved-supermodel-diagnostic__gates" aria-label="Centralny admission supermodelu">
            <h2>Jedna decyzja pipeline’u</h2>
            <div>
              {(preview.report.admissionV3?.stages ?? []).map((stage) => (
                <article key={stage.stage} data-status={stage.status === "PASS" ? "pass" : "blocked"}>
                  <span>{stage.stage}</span>
                  <strong>{stage.status}</strong>
                  <small>{stage.sourceStatus}{stage.blockingCodes.length ? ` · ${stage.blockingCodes.join(", ")}` : ""}</small>
                </article>
              ))}
              {!preview.report.admissionV3 ? (
                <article data-status="blocked">
                  <span>central admission</span>
                  <strong>BLOCKED</strong>
                  <small>raport sprzed kontraktu V3 — może służyć tylko jako negatywny oracle</small>
                </article>
              ) : null}
            </div>
          </section>

          {rigInventory ? (
            <section className="saved-supermodel-diagnostic__gates" aria-label="Rozdzielony inwentarz rigu">
              <h2>Co naprawdę zawiera model</h2>
              <div>
                <article data-status="pass"><span>Exact carriers</span><strong>{preview.report.structuralAnalysis.carrierNodeCount}</strong><small>topologia wybranego chain</small></article>
                <article data-status="pass"><span>Scene nodes</span><strong>{rigInventory.nodes.length}</strong><small>wszystkie transformy readbacku</small></article>
                <article data-status="pass"><span>Skin bones</span><strong>{rigInventory.counts.SKIN_BONE}</strong><small>aktywne w node-to-bone map</small></article>
                <article data-status="pass"><span>Helpers</span><strong>{rigInventory.counts.HELPER + rigInventory.counts.ATTACHMENT}</strong><small>helper + attachment, poza skin bones</small></article>
                <article data-status="pass"><span>Geometry nodes</span><strong>{rigInventory.nodes.filter((node) => node.ownsMesh).length}</strong><small>węzły posiadające mesh</small></article>
              </div>
            </section>
          ) : null}

          {preview.report.rigAnalysis.jointFit ? (
            <section className="saved-supermodel-diagnostic__joints" aria-label="Joint fit details">
              <header>
                <div>
                  <h2>Jointy i constrainty</h2>
                  <p>
                    {preview.report.rigAnalysis.jointFit.constraintPassCount}/{preview.report.rigAnalysis.jointFit.constraintRequiredCount} constraintów · minimum confidence {preview.report.rigAnalysis.jointFit.minimumConfidence.toLocaleString("pl-PL", { maximumFractionDigits: 3 })}
                  </p>
                </div>
                <strong data-status={preview.report.rigAnalysis.jointFit.status === "READY" ? "pass" : "blocked"}>{preview.report.rigAnalysis.jointFit.status}</strong>
              </header>
              {preview.report.rigAnalysis.jointFit.constraintViolations.length ? (
                <ul>
                  {preview.report.rigAnalysis.jointFit.constraintViolations.map((constraint) => (
                    <li key={`${constraint.code}:${constraint.partNumbers.join("-")}`}>
                      <code>{constraint.code}</code> · jointy {constraint.partNumbers.join(", ") || "globalne"} · {constraint.message}
                    </li>
                  ))}
                </ul>
              ) : null}
              <div className="saved-supermodel-diagnostic__joint-table-wrap">
                <table>
                  <thead><tr><th>Joint</th><th>Region</th><th>Reference XYZ</th><th>Fitted XYZ</th><th>Residual</th><th>Confidence</th><th>Constraint</th><th>Provenance</th></tr></thead>
                  <tbody>
                    {preview.report.rigAnalysis.jointFit.joints.map((joint) => (
                      <tr key={joint.partNumber} data-status={joint.constraintVerdict === "PASS" ? "pass" : "blocked"}>
                        <th>{joint.partNumber} · {joint.jointName}</th>
                        <td>{joint.semanticRegion}</td>
                        <td>{joint.referenceWorldPosition.map((value) => value.toFixed(3)).join(" / ")}</td>
                        <td>{joint.targetWorldPosition.map((value) => value.toFixed(3)).join(" / ")}</td>
                        <td>{(joint.residualFraction * 100).toFixed(2)}%</td>
                        <td>{joint.confidence.toFixed(3)}</td>
                        <td>{joint.constraintVerdict}</td>
                        <td>{joint.provenance}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          ) : null}

          <section className="saved-supermodel-diagnostic__gates" aria-label="Decydujące bramki jakości">
            <h2>Dlaczego wynik jest zablokowany</h2>
            <div>
              <article data-status={preview.diagnosticDetails.perComponentDeformationCoverage ? "pass" : "blocked"}>
                <span>Komponent × klip</span>
                <strong>{preview.diagnosticDetails.perComponentDeformationCoverage ? "PASS" : "BLOCKED"}</strong>
                <small>{preview.diagnosticDetails.failedComponentSampleCount.toLocaleString("pl-PL")} próbek nie przeszło</small>
              </article>
              <article data-status="blocked">
                <span>Najgorsza zmiana krawędzi</span>
                <strong>×{preview.diagnosticDetails.worstMaxEdgeRatio?.toLocaleString("pl-PL", { maximumFractionDigits: 2 }) ?? "—"}</strong>
                <small>minimum ×{preview.diagnosticDetails.worstMinEdgeRatio?.toLocaleString("pl-PL", { maximumFractionDigits: 3 }) ?? "—"}</small>
              </article>
              <article data-status="blocked">
                <span>Najgorsza zmiana pola</span>
                <strong>×{preview.diagnosticDetails.worstMaxTriangleAreaRatio?.toLocaleString("pl-PL", { maximumFractionDigits: 2 }) ?? "—"}</strong>
                <small>minimum ×{preview.diagnosticDetails.worstMinTriangleAreaRatio?.toLocaleString("pl-PL", { maximumFractionDigits: 3 }) ?? "—"}</small>
              </article>
              <article data-status={preview.report.motionQuality.jointClipCoverage ? "pass" : "blocked"}>
                <span>Joint × clip</span>
                <strong>{preview.report.motionQuality.jointClipPassCount}/{preview.report.motionQuality.jointClipRequiredCount}</strong>
                <small>{preview.report.motionQuality.jointClipCoverage ? "pełne pokrycie" : "niepełne pokrycie"}</small>
              </article>
            </div>
            <p>
              Globalne budżety są spełnione: hard edges {preview.report.motionQuality.edgeOutsideHardLimitCount.toLocaleString("pl-PL")}/{preview.report.motionQuality.edgeOutsideHardAllowedCount.toLocaleString("pl-PL")},
              {" "}collapsed triangles {preview.report.motionQuality.triangleAreaCollapseCount.toLocaleString("pl-PL")}/{preview.report.motionQuality.triangleAreaCollapseAllowedCount.toLocaleString("pl-PL")}.
              Mimo tego bramka per-komponent wykrywa lokalne, katastrofalne deformacje. Szkielet, jointy i klipy są obecne, ale skinning powierzchni jest błędny.
            </p>
          </section>

          <SupermodelPreviewViewport
            reports={[]}
            carrier={preview.target}
            detail={`${preview.sourceName} po nałożeniu ${preview.supermodelResref}`}
            onError={setError}
          />
          <p className="supermodel-library__truth">
            Podgląd Three.js i raport są dowodem offline aplikacji. Końcowy werdykt wizualny Toolset/NWN pozostaje po stronie właściciela.
          </p>
        </section>
      ) : null}
    </main>
  );
}
