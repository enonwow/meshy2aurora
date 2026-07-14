import type { CanonicalResultSnapshot } from "./projectCanonicalResult";

export function CanonicalResultSummary({ result }: { result: CanonicalResultSnapshot }) {
  return (
    <section className="panel result-workspace" aria-label="Canonical model result">
      <div>
        <p className="eyebrow">Canonical structural output</p>
        <h2>{result.status}</h2>
        <p><strong>Runtime acceptance:</strong> OPEN_M6 — Aurora/NWN runtime proof is a later gate.</p>
      </div>
      <div className="result-grid">
        <article><h3>Geometry</h3><dl>
          <div><dt>Vertices</dt><dd>{result.geometry.vertices}</dd></div>
          <div><dt>Triangles</dt><dd>{result.geometry.triangles}</dd></div>
          <div><dt>Active joints</dt><dd>{result.geometry.joints}</dd></div>
          <div><dt>Deformation</dt><dd>{result.geometry.deformation}</dd></div>
        </dl></article>
        <article><h3>Animation</h3><dl>
          <div><dt>Mapping</dt><dd>{result.animation.sourceName} → {result.animation.outputName}</dd></div>
          <div><dt>Duration</dt><dd>{result.animation.durationSeconds} s</dd></div>
          <div><dt>Motion</dt><dd>{result.animation.hasMotion ? "yes" : "no"}</dd></div>
        </dl></article>
        <article><h3>Texture and 2DA</h3><dl>
          <div><dt>Texture</dt><dd>{result.texture.width} × {result.texture.height} · {result.texture.pixelFormat}</dd></div>
          <div><dt>Texture bytes</dt><dd>{result.texture.byteLength}</dd></div>
          <div><dt>Appended row</dt><dd>{result.appearance.appendedRow}</dd></div>
          <div><dt>Prefix preserved</dt><dd>{result.appearance.sourcePrefixPreserved ? "yes" : "no"}</dd></div>
        </dl></article>
        <article><h3>Package</h3><dl>
          <div><dt>Model resref</dt><dd>{result.resrefs.model}</dd></div>
          <div><dt>Texture resref</dt><dd>{result.resrefs.texture}</dd></div>
          <div><dt>HAK</dt><dd>{result.hak.byteLength} bytes · {result.hak.entryCount} resources</dd></div>
          <div><dt>HAK SHA-256</dt><dd><code>{result.hak.sha256}</code></dd></div>
        </dl></article>
      </div>
      <h3>HAK resources</h3>
      <ul>{result.resources.map((resource) => (
        <li key={`${resource.role}:${resource.resref}`}>
          <strong>{resource.role}</strong><span>{resource.resref} · type {resource.type}</span><span>{resource.byteLength} bytes</span>
        </li>
      ))}</ul>
      <h3>Byte identities</h3>
      <ul>{Object.entries(result.outputs).map(([name, identity]) => (
        <li key={name}><strong>{name}</strong><span>{identity.byteLength} bytes</span><code>{identity.sha256}</code></li>
      ))}</ul>
    </section>
  );
}
