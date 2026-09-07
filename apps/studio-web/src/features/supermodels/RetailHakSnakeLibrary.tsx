import { useEffect, useMemo, useState } from "react";
import * as THREE from "three";
import type { BinaryMdlInspectionReport, ReadbackNode } from "../preview/types";
import type { AuroraReadbackMaterialResolver } from "../preview/AuroraReadbackViewport";
import { projectCanonicalReadback } from "../results/projectReadback";
import { StudioWorkerClient } from "../../worker/client";
import type { StudioWorkerResponse } from "../../worker/types";
import { locateHakModelResourceV1, locateHakResourceV1, parseHakRangeIndexPlanV1 } from "./hakRange";
import { parseNwnDdsV1 } from "./nwnDds";
import { SupermodelPreviewViewport } from "./SupermodelPreviewViewport";
import "./supermodel-library.css";

export interface SnakeModelLineageV1 {
  readonly resref: string;
  readonly supermodel: string;
  readonly family: "Cobra" | "Viper" | "Worm" | "Naga";
}

export const SNAKE_MODEL_LINEAGES_V1: readonly SnakeModelLineageV1[] = [
  { resref: "fwp_cobr_sap", supermodel: "c_cobra01", family: "Cobra" },
  { resref: "c_viper_desert_h", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_desert_m", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_desert_t", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_forest_m", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_forest_t", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_jungle_h", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_jungle_m", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_jungle_t", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_swamp_h", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_swamp_m", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_viper_swamp_t", supermodel: "c_viper_forest_h", family: "Viper" },
  { resref: "c_worm2", supermodel: "c_worm", family: "Worm" },
  { resref: "c_worm3", supermodel: "c_worm", family: "Worm" },
  { resref: "c_worm4", supermodel: "c_worm", family: "Worm" },
  { resref: "zcp_nagab", supermodel: "zcp_nagaa", family: "Naga" },
  { resref: "zcp_nagac", supermodel: "zcp_nagaa", family: "Naga" },
  { resref: "zcp_nagad", supermodel: "zcp_nagaa", family: "Naga" },
];

export function diffuseTextureResrefsV1(report: BinaryMdlInspectionReport) {
  const result = new Set<string>();
  const visit = (node: ReadbackNode) => {
    const resref = node.mesh?.textures?.[0]?.trim().toLocaleLowerCase() ?? "";
    if (resref && resref !== "null") result.add(resref);
    node.children.forEach(visit);
  };
  report.nodeTree.roots.forEach(visit);
  return [...result];
}

export function retailHakResourceUrlV1(
  rootUrl: string,
  resourcePath: string,
  studioUrl = window.location.href,
) {
  const studio = new URL(studioUrl);
  const root = new URL(rootUrl.replace(/\/$/, ""), studio);
  if (root.origin !== studio.origin || decodeURIComponent(root.pathname) !== "/__m2a_nwn_user_reference") {
    throw new Error("HAK resources must use the dedicated same-origin local reference endpoint");
  }
  const parts = resourcePath.replace(/\\/g, "/").split("/").filter(Boolean);
  if (!parts.length || parts.some((part) => part === "." || part === "..")) {
    throw new Error("HAK resource must use a safe HAK resource path");
  }
  return new URL(`${root.pathname}/${parts.join("/")}`, studio).toString();
}

function exactResponse<T extends StudioWorkerResponse["type"]>(
  response: StudioWorkerResponse,
  expected: T,
): Extract<StudioWorkerResponse, { ok: true; type: T }> {
  if (!response.ok) throw new Error(response.message);
  if (response.type !== expected) throw new Error(`Unexpected Studio Worker response: ${response.type}`);
  return response as Extract<StudioWorkerResponse, { ok: true; type: T }>;
}

async function fetchExactRange(url: string, offset: number, byteLength: number, signal: AbortSignal) {
  if (!Number.isSafeInteger(offset) || !Number.isSafeInteger(byteLength) || offset < 0 || byteLength <= 0) {
    throw new Error("Invalid HAK byte range");
  }
  const end = offset + byteLength - 1;
  const response = await fetch(url, { headers: { Range: `bytes=${offset}-${end}` }, signal });
  if (response.status !== 206) throw new Error(`Local HAK server did not honor byte range ${offset}-${end}`);
  const bytes = await response.arrayBuffer();
  if (bytes.byteLength !== byteLength) {
    throw new Error(`Short HAK resource range: expected ${byteLength}, received ${bytes.byteLength}`);
  }
  return bytes;
}

async function sha256Hex(bytes: ArrayBuffer) {
  return [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))]
    .map((value) => value.toString(16).padStart(2, "0"))
    .join("");
}

interface HakModelReadbackV1 {
  readonly report: BinaryMdlInspectionReport;
  readonly sha256: string;
  readonly payloadSize: number;
  readonly textures: ReadonlyMap<string, THREE.Texture>;
  readonly textureResrefs: readonly string[];
  readonly missingTextureResrefs: readonly string[];
}

interface HakRangeIndexV1 {
  readonly entryCount: number;
  readonly keyTableBytes: ArrayBuffer;
  readonly resourceTableBytes: ArrayBuffer;
}

async function loadHakRangeIndexV1(hakUrl: string, signal: AbortSignal): Promise<HakRangeIndexV1> {
  const headerBytes = await fetchExactRange(hakUrl, 0, 160, signal);
  const plan = parseHakRangeIndexPlanV1(headerBytes);
  const [keyTableBytes, resourceTableBytes] = await Promise.all([
    fetchExactRange(hakUrl, plan.keyTableOffset, plan.keyTableByteLength, signal),
    fetchExactRange(hakUrl, plan.resourceTableOffset, plan.resourceTableByteLength, signal),
  ]);
  return { entryCount: plan.entryCount, keyTableBytes, resourceTableBytes };
}

async function loadDdsTexturesV1(
  textureHakUrl: string,
  resrefs: readonly string[],
  signal: AbortSignal,
) {
  const textures = new Map<string, THREE.Texture>();
  const missingTextureResrefs: string[] = [];
  if (resrefs.length === 0) return { textures, missingTextureResrefs };
  const index = await loadHakRangeIndexV1(textureHakUrl, signal);
  for (const resref of resrefs) {
    try {
      const locator = locateHakResourceV1(
        index.keyTableBytes,
        index.resourceTableBytes,
        index.entryCount,
        resref,
        2033,
      );
      const bytes = await fetchExactRange(textureHakUrl, locator.payloadOffset, locator.payloadSize, signal);
      const dds = parseNwnDdsV1(bytes);
      const texture = new THREE.CompressedTexture(
        dds.mipmaps.map(({ data, width, height }) => ({ data, width, height })),
        dds.width,
        dds.height,
        dds.format === "DXT1" ? THREE.RGB_S3TC_DXT1_Format : THREE.RGBA_S3TC_DXT5_Format,
      );
      texture.name = `${resref}.dds`;
      texture.colorSpace = THREE.SRGBColorSpace;
      texture.flipY = false;
      texture.generateMipmaps = false;
      texture.minFilter = dds.mipmaps.length > 1 ? THREE.LinearMipmapLinearFilter : THREE.LinearFilter;
      texture.magFilter = THREE.LinearFilter;
      texture.needsUpdate = true;
      textures.set(resref, texture);
    } catch {
      missingTextureResrefs.push(resref);
    }
  }
  return { textures, missingTextureResrefs };
}

async function loadHakModelV1(
  worker: StudioWorkerClient,
  hakUrl: string,
  textureHakUrl: string,
  lineage: SnakeModelLineageV1,
  signal: AbortSignal,
): Promise<HakModelReadbackV1> {
  const index = await loadHakRangeIndexV1(hakUrl, signal);
  const locator = locateHakModelResourceV1(
    index.keyTableBytes,
    index.resourceTableBytes,
    index.entryCount,
    lineage.resref,
  );
  const mdlBytes = await fetchExactRange(hakUrl, locator.payloadOffset, locator.payloadSize, signal);
  const hash = await sha256Hex(mdlBytes);
  const inspectResponse = exactResponse(await worker.request({
    requestId: crypto.randomUUID(),
    type: "INSPECT_BINARY_MDL",
    mdlBytes,
  }, [mdlBytes]), "BINARY_MDL_INSPECTED");
  const report = projectCanonicalReadback(inspectResponse.reportJson);
  const actualName = report.model?.name.trim().toLocaleLowerCase();
  const actualSupermodel = report.model?.supermodelName.trim().toLocaleLowerCase();
  if (actualName !== lineage.resref || actualSupermodel !== lineage.supermodel) {
    throw new Error(
      `Lineage mismatch: expected ${lineage.resref} → ${lineage.supermodel}, read ${actualName ?? "?"} → ${actualSupermodel ?? "?"}`,
    );
  }
  const textureResrefs = diffuseTextureResrefsV1(report);
  const { textures, missingTextureResrefs } = await loadDdsTexturesV1(textureHakUrl, textureResrefs, signal);
  return { report, sha256: hash, payloadSize: locator.payloadSize, textures, textureResrefs, missingTextureResrefs };
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
  readonly hakRootUrl: string;
  readonly hakPath: string;
  readonly textureHakPath: string;
}

export function RetailHakSnakeLibrary({ hakRootUrl, hakPath, textureHakPath }: Props) {
  const [selectedResref, setSelectedResref] = useState("c_viper_desert_h");
  const [showTextures, setShowTextures] = useState(true);
  const [readback, setReadback] = useState<HakModelReadbackV1>();
  const [error, setError] = useState<string>();
  const selected = useMemo(
    () => SNAKE_MODEL_LINEAGES_V1.find(({ resref }) => resref === selectedResref) ?? SNAKE_MODEL_LINEAGES_V1[0],
    [selectedResref],
  );
  const hakUrl = useMemo(() => retailHakResourceUrlV1(hakRootUrl, hakPath), [hakPath, hakRootUrl]);
  const textureHakUrl = useMemo(
    () => retailHakResourceUrlV1(hakRootUrl, textureHakPath),
    [hakRootUrl, textureHakPath],
  );

  useEffect(() => {
    if (!selected) return;
    const worker = new StudioWorkerClient();
    const controller = new AbortController();
    setReadback(undefined);
    setError(undefined);
    void loadHakModelV1(worker, hakUrl, textureHakUrl, selected, controller.signal)
      .then(setReadback)
      .catch((reason: unknown) => {
        if (!controller.signal.aborted) setError(reason instanceof Error ? reason.message : String(reason));
      });
    return () => {
      controller.abort();
      worker.dispose();
    };
  }, [hakUrl, selected, textureHakUrl]);

  const nodeCount = useMemo(
    () => readback ? countNodes(readback.report.nodeTree.roots) : 0,
    [readback],
  );
  const materialResolver = useMemo<AuroraReadbackMaterialResolver | undefined>(() => {
    if (!readback) return undefined;
    return (mesh, _node, selectedPart) => {
      const textureResref = mesh.textures?.[0]?.trim().toLocaleLowerCase() ?? "";
      const diffuse = mesh.diffuse ?? [1, 1, 1];
      return new THREE.MeshPhongMaterial({
        map: showTextures ? readback.textures.get(textureResref) : undefined,
        color: new THREE.Color(...diffuse),
        emissive: selectedPart ? new THREE.Color(0x164f42) : new THREE.Color(0x000000),
        shininess: mesh.shininess ?? 8,
        side: THREE.DoubleSide,
      });
    };
  }, [readback, showTextures]);

  return (
    <main className="saved-supermodel-diagnostic retail-hak-library">
      <header className="supermodel-library__header">
        <div>
          <p className="eyebrow">Meshy2Aurora Studio · istniejące modele CEP</p>
          <h1>Modele używające supermodeli węży</h1>
          <p>18 faktycznych modeli potomnych odczytanych bezpośrednio z <strong>cep3_core1.hak</strong>.</p>
        </div>
        <a className="button button--secondary" href="./">Wróć do Studio</a>
      </header>

      <section className="supermodel-library__workspace" aria-label="Snake supermodel child library">
        <aside className="supermodel-library__catalog">
          <p className="supermodel-library__truth">Kliknij model, aby zobaczyć jego mesh, szkielet i jointy.</p>
          <div className="supermodel-library__entries">
            {SNAKE_MODEL_LINEAGES_V1.map((lineage) => (
              <button
                type="button"
                key={lineage.resref}
                data-selected={lineage.resref === selected?.resref}
                onClick={() => setSelectedResref(lineage.resref)}
              >
                <span>
                  <strong>{lineage.resref}</strong>
                  <small>{lineage.resref} → {lineage.supermodel}</small>
                </span>
                <small>{lineage.family}</small>
                <small>MDL</small>
              </button>
            ))}
          </div>
        </aside>

        <section className="supermodel-library__preview">
          <header className="supermodel-library__selection">
            <div>
              <p className="eyebrow">Wybrany istniejący model</p>
              <h2>{selected?.resref}</h2>
            </div>
          </header>

          <section className="supermodel-library__preview-mode" aria-label="Opcje wyglądu modelu">
            <label>
              <span>Materiał</span>
              <span>
                <input
                  type="checkbox"
                  checked={showTextures}
                  onChange={(event) => setShowTextures(event.currentTarget.checked)}
                />
                Pokaż tekstury z cep3_core0.hak
              </span>
            </label>
          </section>

          {!readback && !error ? <p role="status">Odczytywanie modelu z HAK i budowanie podglądu…</p> : null}
          {error ? <p role="alert" className="supermodel-library__error">Nie można odczytać modelu: {error}</p> : null}

          {readback && selected ? (
            <>
              <section className="supermodel-library__applied" aria-label="Verified snake supermodel lineage">
                <strong>ZWERYFIKOWANE DZIEDZICZENIE</strong>
                <span>{readback.report.model?.name}</span>
                <span>→</span>
                <span data-testid="actual-supermodel">{readback.report.model?.supermodelName}</span>
                <span>{readback.payloadSize.toLocaleString("pl-PL")} B</span>
                <span>{nodeCount} nodów</span>
                <span>{readback.textures.size}/{readback.textureResrefs.length} tekstur DDS</span>
                <span>SHA-256 {readback.sha256.slice(0, 16)}…</span>
              </section>
              <SupermodelPreviewViewport
                reports={[readback.report]}
                carrier={readback.report}
                detail={`${selected.resref} → ${selected.supermodel}: istniejący model, szkielet i jointy`}
                onError={setError}
                materialResolver={materialResolver}
              />
              {readback.missingTextureResrefs.length ? (
                <p className="supermodel-library__limitations">
                  Brak w cep3_core0.hak: {readback.missingTextureResrefs.join(", ")}
                </p>
              ) : null}
              <p className="supermodel-library__truth">
                Relacja pochodzi z nagłówka tego konkretnego MDL, a geometria z dokładnego zasobu HAK. Podgląd niczego nie generuje ani nie podmienia.
              </p>
            </>
          ) : null}
        </section>
      </section>
    </main>
  );
}
