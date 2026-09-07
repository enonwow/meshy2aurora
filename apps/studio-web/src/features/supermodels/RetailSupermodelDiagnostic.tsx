import { useEffect, useMemo, useState } from "react";
import { projectCanonicalReadback } from "../results/projectReadback";
import type { BinaryMdlInspectionReport, ReadbackNode } from "../preview/types";
import { StudioWorkerClient } from "../../worker/client";
import type { StudioWorkerResponse } from "../../worker/types";
import {
  parseNwnBifIndexPlanV1,
  parseNwnBifIndexV1,
  parseNwnKeyModelIndexV1,
} from "./types";
import { SupermodelPreviewViewport } from "./SupermodelPreviewViewport";
import "./supermodel-library.css";

export function retailNwnResourceUrlV1(
  rootUrl: string,
  resourcePath: string,
  studioUrl = window.location.href,
) {
  const studio = new URL(studioUrl);
  const root = new URL(rootUrl.replace(/\/$/, ""), studio);
  if (root.origin !== studio.origin) throw new Error("Retail resources must use the same Studio origin");
  const decodedRoot = decodeURIComponent(root.pathname).replace(/\\/g, "/");
  const devReferenceEndpoint = decodedRoot === "/__m2a_nwn_reference";
  const directLocalReference = decodedRoot.startsWith("/@fs/") && /\/Neverwinter Nights$/i.test(decodedRoot);
  if (!devReferenceEndpoint && !directLocalReference) {
    throw new Error("Retail resources must resolve to a selected local Neverwinter Nights root");
  }
  const parts = resourcePath.replace(/\\/g, "/").split("/").filter(Boolean);
  if (!parts.length || parts.some((part) => part === "." || part === "..")) {
    throw new Error("Retail resource must use a safe retail resource path");
  }
  return new URL(`${decodedRoot}/${parts.join("/")}`, studio).toString();
}

function exactResponse<T extends StudioWorkerResponse["type"]>(
  response: StudioWorkerResponse,
  expected: T,
): Extract<StudioWorkerResponse, { ok: true; type: T }> {
  if (!response.ok) throw new Error(response.message);
  if (response.type !== expected) throw new Error(`Unexpected Studio Worker response: ${response.type}`);
  return response as Extract<StudioWorkerResponse, { ok: true; type: T }>;
}

async function fetchWholeFile(url: string, signal: AbortSignal) {
  const response = await fetch(url, { signal });
  if (!response.ok) throw new Error(`${response.status} ${response.statusText}: ${url}`);
  return response.arrayBuffer();
}

async function fetchExactRange(url: string, offset: number, byteLength: number, signal: AbortSignal) {
  const end = offset + byteLength - 1;
  const response = await fetch(url, { headers: { Range: `bytes=${offset}-${end}` }, signal });
  if (response.status !== 206) throw new Error(`Retail server did not honor byte range ${offset}-${end}: ${url}`);
  const bytes = await response.arrayBuffer();
  if (bytes.byteLength !== byteLength) throw new Error(`Short retail resource range: expected ${byteLength}, received ${bytes.byteLength}`);
  return bytes;
}

async function sha256Hex(bytes: ArrayBuffer) {
  return [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))]
    .map((value) => value.toString(16).padStart(2, "0"))
    .join("");
}

interface RetailSupermodelReadbackV1 {
  readonly report: BinaryMdlInspectionReport;
  readonly sha256: string;
  readonly containerName: string;
  readonly payloadSize: number;
}

async function loadRetailSupermodelV1(
  worker: StudioWorkerClient,
  rootUrl: string,
  requestedResref: string,
  signal: AbortSignal,
): Promise<RetailSupermodelReadbackV1> {
  const resref = requestedResref.trim().toLocaleLowerCase();
  if (!/^[a-z0-9_]{1,16}$/.test(resref)) throw new Error("Invalid retail supermodel resref");
  const keyUrl = retailNwnResourceUrlV1(rootUrl, "data/nwn_base.key");
  const keyBytes = await fetchWholeFile(keyUrl, signal);
  const keyResponse = exactResponse(await worker.request({
    requestId: crypto.randomUUID(),
    type: "INDEX_NWN_KEY_MODELS",
    keyBytes,
  }, [keyBytes]), "NWN_KEY_MODELS_INDEXED");
  const key = parseNwnKeyModelIndexV1(keyResponse.indexJson);
  const locator = key.models.find((model) => model.resref.toLocaleLowerCase() === resref);
  if (!locator) throw new Error(`Retail model ${resref} is absent from nwn_base.key`);
  const bif = key.bifs[locator.bifIndex];
  if (!bif) throw new Error(`Retail model ${resref} references an absent BIF index`);
  const bifUrl = retailNwnResourceUrlV1(rootUrl, bif.logicalName);
  const headerBytes = await fetchExactRange(bifUrl, 0, 20, signal);
  const planResponse = exactResponse(await worker.request({
    requestId: crypto.randomUUID(),
    type: "PLAN_NWN_BIF_INDEX",
    headerBytes,
  }), "NWN_BIF_INDEX_PLANNED");
  const plan = parseNwnBifIndexPlanV1(planResponse.planJson);
  const tableBytes = await fetchExactRange(bifUrl, plan.tableOffset, plan.tableByteLength, signal);
  const indexResponse = exactResponse(await worker.request({
    requestId: crypto.randomUUID(),
    type: "INDEX_NWN_BIF_TABLE",
    headerBytes,
    tableBytes,
  }), "NWN_BIF_TABLE_INDEXED");
  const resource = parseNwnBifIndexV1(indexResponse.indexJson).resources[locator.resourceIndex];
  if (!resource || resource.resourceType !== 2002) throw new Error(`Retail model ${resref} has an invalid BIF resource entry`);
  const mdlBytes = await fetchExactRange(bifUrl, resource.payloadOffset, resource.payloadSize, signal);
  const hash = await sha256Hex(mdlBytes);
  const inspectResponse = exactResponse(await worker.request({
    requestId: crypto.randomUUID(),
    type: "INSPECT_BINARY_MDL",
    mdlBytes,
  }, [mdlBytes]), "BINARY_MDL_INSPECTED");
  return {
    report: projectCanonicalReadback(inspectResponse.reportJson),
    sha256: hash,
    containerName: bif.logicalName,
    payloadSize: resource.payloadSize,
  };
}

function countNodes(roots: readonly ReadbackNode[]) {
  let count = 0;
  const visit = (node: ReadbackNode) => {
    count += 1;
    node.children.forEach(visit);
  };
  roots.forEach(visit);
  return count;
}

interface Props {
  readonly nwnRootUrl: string;
  readonly resref: string;
}

export function RetailSupermodelDiagnostic({ nwnRootUrl, resref }: Props) {
  const [readback, setReadback] = useState<RetailSupermodelReadbackV1>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    const worker = new StudioWorkerClient();
    const controller = new AbortController();
    setReadback(undefined);
    setError(undefined);
    void loadRetailSupermodelV1(worker, nwnRootUrl, resref, controller.signal)
      .then(setReadback)
      .catch((reason: unknown) => {
        if (!controller.signal.aborted) setError(reason instanceof Error ? reason.message : String(reason));
      });
    return () => {
      controller.abort();
      worker.dispose();
    };
  }, [nwnRootUrl, resref]);

  const nodeCount = useMemo(
    () => readback ? countNodes(readback.report.nodeTree.roots) : 0,
    [readback],
  );

  return (
    <main className="saved-supermodel-diagnostic">
      <header className="supermodel-library__header">
        <div>
          <p className="eyebrow">Meshy2Aurora Studio · local retail readback</p>
          <h1>Model bazowy {resref}</h1>
          <p>Dokładny zasób NWN odczytany zakresami z KEY/BIF, bez kopiowania go do projektu.</p>
        </div>
        <a className="button button--secondary" href="./">Wróć do Studio</a>
      </header>

      {!readback && !error ? <p role="status">Odczytywanie KEY/BIF i budowanie podglądu…</p> : null}
      {error ? <p role="alert" className="supermodel-library__error">Nie można odczytać modelu retail: {error}</p> : null}

      {readback ? (
        <section className="saved-supermodel-diagnostic__workspace" aria-label="Retail supermodel evidence">
          <section className="supermodel-library__applied">
            <strong>RETAIL READBACK</strong>
            <span>{readback.report.model?.name ?? resref}</span>
            <span>{readback.containerName} · {readback.payloadSize.toLocaleString("pl-PL")} B</span>
            <span>SHA-256 {readback.sha256.slice(0, 16)}…</span>
            <span>{nodeCount} nodów · {readback.report.animations.length} lokalnych animacji</span>
          </section>
          <SupermodelPreviewViewport
            reports={[readback.report]}
            carrier={readback.report}
            detail={`Retail ${resref}: model, szkielet i jointy`}
            onError={setError}
          />
          <p className="supermodel-library__truth">
            Kości, jointy i X-Ray są włączone. Możesz przełączać animacje oraz etykiety bez modyfikowania zasobu gry.
          </p>
        </section>
      ) : null}
    </main>
  );
}
