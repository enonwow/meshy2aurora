export interface SourceInspectionMetrics {
  readonly meshCount?: number;
  readonly vertexCount?: number;
  readonly triangleCount?: number;
  readonly materialCount?: number;
  readonly textureCount?: number;
  readonly boneCount?: number;
  readonly animationClipCount?: number;
}

export interface SourceInspectionPanelProps {
  readonly metrics: SourceInspectionMetrics;
}

const METRICS: ReadonlyArray<{
  key: keyof SourceInspectionMetrics;
  label: string;
}> = [
  { key: "meshCount", label: "Meshes" },
  { key: "vertexCount", label: "Vertices" },
  { key: "triangleCount", label: "Triangles" },
  { key: "materialCount", label: "Materials" },
  { key: "textureCount", label: "Textures" },
  { key: "boneCount", label: "Bones" },
  { key: "animationClipCount", label: "Animation clips" },
];

const numberFormat = new Intl.NumberFormat("en-US");

export function SourceInspectionPanel({ metrics }: SourceInspectionPanelProps) {
  return (
    <section className="inspect-panel inspect-source-inspection" aria-labelledby="source-inspection-heading">
      <header className="inspect-panel__header">
        <h2 id="source-inspection-heading">Source Inspection</h2>
      </header>
      <dl className="inspect-source-inspection__metrics">
        {METRICS.map(({ key, label }) => {
          const value = metrics[key];
          const available = typeof value === "number" && Number.isFinite(value);
          return (
            <div className="inspect-source-inspection__metric" key={key}>
              <dt>{label}</dt>
              <dd data-available={available}>{available ? numberFormat.format(value) : "Unavailable"}</dd>
            </div>
          );
        })}
      </dl>
    </section>
  );
}
