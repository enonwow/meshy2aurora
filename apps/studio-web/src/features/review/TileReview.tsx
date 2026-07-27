import { useState } from "react";
import type { BinaryMdlInspectionReport } from "../preview/types";
import type { TileResultSnapshot } from "../results/projectTileResult";
import "./TileReview.css";

interface TileReviewProps {
  result: TileResultSnapshot;
  readback: BinaryMdlInspectionReport;
}

export function TileReview({ result, readback }: TileReviewProps) {
  const [renderVisible, setRenderVisible] = useState(true);
  const [wokVisible, setWokVisible] = useState(true);
  const [aabbVisible, setAabbVisible] = useState(true);
  const readbackStatus = readback.validation?.status ?? "UNAVAILABLE";
  return (
    <section className="tile-review" aria-labelledby="tile-review-heading">
      <header className="tile-review__header">
        <div>
          <p className="eyebrow">Review output</p>
          <h2 id="tile-review-heading">TileStaticV1 Details</h2>
          <p>{result.moduleFileName} · {result.areaName} · 2×2 custom tileset</p>
        </div>
        <strong data-status={readbackStatus.toLowerCase()}>
          MDL {readbackStatus} · owner proof not tested
        </strong>
      </header>

      <div className="tile-review__controls" aria-label="Tile preview overlays">
        <label><input type="checkbox" checked={renderVisible} onChange={(event) => setRenderVisible(event.currentTarget.checked)} />Render MDL</label>
        <label><input type="checkbox" checked={wokVisible} onChange={(event) => setWokVisible(event.currentTarget.checked)} />Walkmesh WOK</label>
        <label><input type="checkbox" checked={aabbVisible} onChange={(event) => setAabbVisible(event.currentTarget.checked)} />AABB tree</label>
      </div>

      <figure className="tile-review__footprint">
        <svg viewBox="0 0 500 500" role="img" aria-label="Ten metre square tile footprint with render, walkmesh and AABB overlays">
          <rect x="25" y="25" width="450" height="450" className="tile-review__boundary" />
          {renderVisible ? <path d="M25 25H475V475H25Z M25 25L475 475 M475 25L25 475" className="tile-review__render" /> : null}
          {wokVisible ? (
            <g className="tile-review__wok">
              <path d="M25 25H475V475H25Z M250 25V475 M25 250H475 M25 25L250 250L475 25 M25 475L250 250L475 475" />
            </g>
          ) : null}
          {aabbVisible ? (
            <g className="tile-review__aabb">
              <rect x="25" y="25" width="450" height="450" />
              <rect x="25" y="25" width="225" height="450" />
              <rect x="250" y="25" width="225" height="450" />
              <rect x="25" y="25" width="225" height="225" />
              <rect x="250" y="250" width="225" height="225" />
            </g>
          ) : null}
          <circle cx="250" cy="250" r="7" className="tile-review__spawn" />
          <text x="250" y="492" textAnchor="middle">10 m (−5 … +5)</text>
        </svg>
        <figcaption>
          Contract footprint preview. Spawn (5,5,0) is walkable; all four seams are covered by the WOK.
        </figcaption>
      </figure>

      <div className="tile-review__metrics" aria-label="Tile geometry metrics">
        <article><span>Render triangles</span><strong>{result.modelTriangleCount.toLocaleString()}</strong></article>
        <article><span>WOK triangles</span><strong>{result.wokTriangleCount.toLocaleString()}</strong></article>
        <article><span>AABB entries</span><strong>{result.aabbEntryCount.toLocaleString()}</strong></article>
        <article><span>Walk surface</span><strong>{result.surfaceId}</strong></article>
      </div>

      <table className="tile-review__bindings">
        <caption>Authoritative tile bindings</caption>
        <tbody>
          <tr><th scope="row">SET tileset</th><td><code>{result.tilesetResref}.set</code></td></tr>
          <tr><th scope="row">Tile_ID</th><td>{result.tileId}</td></tr>
          <tr><th scope="row">SET Model → MDL</th><td><code>{result.modelResref}.mdl</code></td></tr>
          <tr><th scope="row">Resolved WOK</th><td><code>{result.wokResref}.wok</code></td></tr>
          <tr><th scope="row">WalkMesh token</th><td><code>{result.walkmeshClassToken}</code></td></tr>
          <tr><th scope="row">Ordered HAK</th><td><code>{result.hakResref}</code></td></tr>
        </tbody>
      </table>

      <p className="tile-review__status">
        Offline navigation: spawn {result.navigationSpawnWalkable}, seam {result.navigationSeamWalkable}.
        Visual status remains modelVisibility={result.modelVisibility}, proofCompleteness={result.proofCompleteness}.
      </p>
    </section>
  );
}
