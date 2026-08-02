import { useEffect, useMemo, useRef } from "react";
import type { WorkerArtifact } from "../../worker/types";
import {
  compositeItemIconLayers,
  decodeItemIconTga,
  type ItemIconPixels,
} from "./itemIconPreview";

function PixelCanvas({
  pixels,
  label,
}: {
  readonly pixels: ItemIconPixels;
  readonly label: string;
}) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const canvas = ref.current;
    if (!canvas || navigator.userAgent.toLowerCase().includes("jsdom")) return;
    canvas.width = pixels.width;
    canvas.height = pixels.height;
    const context = canvas.getContext("2d");
    if (!context) return;
    const image = context.createImageData(pixels.width, pixels.height);
    image.data.set(pixels.rgba);
    context.putImageData(image, 0, 0);
  }, [pixels]);
  return <canvas ref={ref} width={pixels.width} height={pixels.height} aria-label={label} />;
}

export function ItemIconArtifactPreview({
  artifacts,
}: {
  readonly artifacts: readonly WorkerArtifact[];
}) {
  const layers = useMemo(() => artifacts
    .filter((artifact) => (
      artifact.kind === "TEXTURE"
      && artifact.artifactId.startsWith("item-part-")
      && artifact.artifactId.endsWith("-icon")
    ))
    .map((artifact) => ({
      artifact,
      pixels: decodeItemIconTga(artifact.bytes),
    })), [artifacts]);
  const composite = useMemo(
    () => layers.length > 0
      ? compositeItemIconLayers(layers.map(({ pixels }) => pixels))
      : undefined,
    [layers],
  );
  if (!composite) return null;
  return (
    <section className="panel item-icon-artifact-preview">
      <header>
        <div><h2>Generated geometry icon layers</h2><p>Exact emitted TGA pixels, shared projection frame, UTI slot order.</p></div>
        <span>{composite.width} × {composite.height}</span>
      </header>
      <div className="item-icon-composite">
        <PixelCanvas pixels={composite} label="Composited generated item icon layers" />
        <strong>Authoring composite</strong>
      </div>
      <div className="item-icon-layer-list">
        {layers.map(({ artifact, pixels }) => (
          <figure key={artifact.artifactId}>
            <PixelCanvas pixels={pixels} label={`Generated icon layer ${artifact.fileName}`} />
            <figcaption>{artifact.fileName}</figcaption>
          </figure>
        ))}
      </div>
    </section>
  );
}
