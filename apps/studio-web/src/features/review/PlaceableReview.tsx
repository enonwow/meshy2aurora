import type { BinaryMdlInspectionReport, ReadbackNode } from "../preview/types";
import type { PlaceableResultSnapshot } from "../results/projectPlaceableResult";

const nodeCount = (nodes: readonly ReadbackNode[]): number =>
  nodes.reduce((count, node) => count + 1 + nodeCount(node.children), 0);

export function PlaceableReview({
  result,
  readback,
}: {
  result: PlaceableResultSnapshot;
  readback: BinaryMdlInspectionReport;
}) {
  const statuses = Object.entries(result.componentStatuses);
  return (
    <section className="review-model" aria-labelledby="placeable-review-heading">
      <header className="review-model__header">
        <div>
          <p className="eyebrow">Review static placeable</p>
          <h2 id="placeable-review-heading">{result.moduleDisplayName}</h2>
        </div>
        <span className="readback-badge" data-status="pass">
          Offline admission passed
          <small>Owner visual proof not performed</small>
        </span>
      </header>

      <div className="review-model__evidence" aria-label="Placeable pipeline status">
        {statuses.map(([name, status]) => (
          <article key={name}>
            <span>{name}</span>
            <strong data-status={status === "passed" ? "pass" : "unavailable"}>{status}</strong>
          </article>
        ))}
      </div>

      <div className="review-model__metrics">
        <dl className="build-input-summary__grid">
          <div><dt>Profile</dt><dd><strong>{result.profile}</strong></dd></div>
          <div><dt>Appearance row</dt><dd><strong>{result.appearanceRow}</strong></dd></div>
          <div><dt>Model / texture</dt><dd><code>{result.modelResref}</code><code>{result.textureResref}</code></dd></div>
          <div><dt>Blueprint / object</dt><dd><code>{result.blueprintResref}</code><code>{result.objectTag}</code></dd></div>
          <div><dt>Module / Area</dt><dd><strong>{result.moduleFileName}</strong><span>{result.areaName} ({result.areaResref})</span></dd></div>
          <div>
            <dt>Placement</dt>
            <dd><code>{`${result.placement.x}, ${result.placement.y}, ${result.placement.z} / ${result.placement.bearing}`}</code></dd>
          </div>
          <div><dt>Model visibility</dt><dd><strong>{result.modelVisibility}</strong></dd></div>
          <div><dt>Proof completeness</dt><dd><strong>{result.proofCompleteness}</strong></dd></div>
          <div><dt>Palette</dt><dd><strong>{result.paletteCompleteness}</strong></dd></div>
          <div><dt>Collision</dt><dd><strong>{result.collisionCompleteness}</strong></dd></div>
          {result.authoring ? (
            <>
              <div>
                <dt>Authored elements</dt>
                <dd>
                  <strong>{result.authoring.renderableElementCount} render</strong>
                  <span>{result.authoring.collisionElementCount} collision · {result.authoring.shadowElementCount} shadow</span>
                </dd>
              </div>
              <div>
                <dt>Authored topology</dt>
                <dd>
                  <strong>{result.authoring.outputTriangleCount.toLocaleString("en-US")} triangles</strong>
                  <code title={result.authoring.authoringSha256}>{result.authoring.authoringSha256.slice(0, 12)}...</code>
                  <code title={result.authoring.sourceSha256}>source {result.authoring.sourceSha256.slice(0, 12)}...</code>
                </dd>
              </div>
            </>
          ) : null}
          {result.collision ? (
            <>
              <div>
                <dt>PWK footprint</dt>
                <dd>
                  <strong>{result.collision.mode}</strong>
                  <span>{result.collision.sourceVertices.length} vertices · {result.collision.triangles.length} faces · surface {result.collision.surfaceId}</span>
                </dd>
              </div>
              <div>
                <dt>PWK bounds</dt>
                <dd>
                  <code>{result.collision.boundsMin.join(", ")} → {result.collision.boundsMax.join(", ")}</code>
                  <code title={result.collision.collisionSha256}>collision {result.collision.collisionSha256.slice(0, 12)}...</code>
                  <code title={result.collision.pwkSha256}>PWK {result.collision.pwkSha256.slice(0, 12)}...</code>
                </dd>
              </div>
            </>
          ) : null}
          {result.textureAuthoring ? (
            <>
              <div>
                <dt>Material textures</dt>
                <dd>
                  <strong>{result.textureAuthoring.bindings.length} bindings</strong>
                  <span>{result.textureAuthoring.resources.length} exact TGA resources · OPAQUE_ONLY</span>
                </dd>
              </div>
              <div>
                <dt>Texture recipe</dt>
                <dd>
                  <code title={result.textureAuthoring.authoringSha256}>
                    {result.textureAuthoring.authoringSha256.slice(0, 12)}...
                  </code>
                  <span>{result.textureAuthoring.bindings.filter((binding) => binding.mode === "OVERRIDE").length} overrides</span>
                </dd>
              </div>
            </>
          ) : null}
          {result.modelTextureAuthoring ? (
            <>
              <div>
                <dt>Material quality gate</dt>
                <dd>
                  <strong>{result.modelTextureAuthoring.bindings.some(
                    (binding) => binding.mipReadability.status === "FLAT",
                  )
                    ? "FLAT"
                    : result.modelTextureAuthoring.bindings.some(
                      (binding) => binding.mipReadability.status === "LOW_CONTRAST",
                    )
                      ? "LOW_CONTRAST"
                      : "READABLE"}</strong>
                  <span>{result.modelTextureAuthoring.uvPolicy}</span>
                </dd>
              </div>
              {result.modelTextureAuthoring.bindings.map((binding) => (
                <div key={`material-quality:${binding.materialSlot}`}>
                  <dt>{binding.authoredMaterialId}</dt>
                  <dd>
                    <strong>{binding.mipReadability.status}</strong>
                    <span>
                      mip16 contrast {(binding.mipReadability.mip16LumaStddevMilli / 1_000).toFixed(2)}
                      {binding.sourceDoubleSided ? " · double-sided target unproven" : ""}
                    </span>
                  </dd>
                </div>
              ))}
              {result.modelTextureAuthoring.warnings.map((warning) => (
                <div key={warning}>
                  <dt>Warning</dt>
                  <dd><code>{warning}</code></dd>
                </div>
              ))}
            </>
          ) : null}
          <div>
            <dt>Binary readback</dt>
            <dd><strong>{readback.validation?.status ?? "UNAVAILABLE"}</strong><span>{nodeCount(readback.nodeTree.roots)} nodes</span></dd>
          </div>
          <div>
            <dt>Export preview lineage</dt>
            <dd>
              <strong>Exact binary MDL + exported texture resources</strong>
              <code title={result.readbackSha256}>readback {result.readbackSha256.slice(0, 12)}...</code>
            </dd>
          </div>
        </dl>
      </div>

      <div className="review-model__metrics">
        <table>
          <caption>Exact generated resource bindings</caption>
          <thead>
            <tr><th>Container</th><th>Role</th><th>Resref</th><th>Type</th><th>Bytes</th><th>SHA-256</th></tr>
          </thead>
          <tbody>
            {result.resources.map((resource) => (
              <tr key={`${resource.container}:${resource.resref}:${resource.resourceType}`}>
                <td>{resource.container}</td>
                <td>{resource.role}</td>
                <td><code>{resource.resref}</code></td>
                <td>{resource.resourceType}</td>
                <td>{resource.byteLength.toLocaleString("en-US")}</td>
                <td><code title={resource.sha256}>{resource.sha256.slice(0, 12)}...</code></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
