import type { CustomAnimationDefinitionV2 } from "../animation-studio/types";

export function CustomAnimationProvenanceCard({
  custom,
  sourceRevision,
}: {
  custom: CustomAnimationDefinitionV2;
  sourceRevision: string;
}) {
  return (
    <dl className="custom-animation-provenance">
      <div><dt>Provider</dt><dd>Authored locally</dd></div>
      <div><dt>Custom ID</dt><dd>{custom.id}</dd></div>
      <div><dt>Asset ID</dt><dd>{custom.provenance.assetId}</dd></div>
      <div><dt>Source GLB</dt><dd>unchanged</dd></div>
      <div><dt>Source revision</dt><dd>{sourceRevision.slice(0, 12)}…</dd></div>
    </dl>
  );
}
