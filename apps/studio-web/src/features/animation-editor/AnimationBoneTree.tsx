export interface AnimationRigNodeV1 {
  readonly id: number;
  readonly name: string;
  readonly parentId: number | null;
  readonly translation: readonly [number, number, number];
  readonly rotation: readonly [number, number, number, number];
}

export function AnimationBoneTree({
  nodes,
  selectedNodeId,
  onSelect,
}: {
  nodes: readonly AnimationRigNodeV1[];
  selectedNodeId: number | null;
  onSelect: (id: number) => void;
}) {
  return (
    <nav className="animation-bone-tree" aria-label="Output rig bones">
      <label>
        <span>Selected bone</span>
        <select
          aria-label="Output rig bone"
          value={selectedNodeId ?? ""}
          disabled={nodes.length === 0}
          onChange={(event) => onSelect(Number(event.currentTarget.value))}
        >
          {nodes.length === 0 ? <option value="">No bones available</option> : null}
          {nodes.map((node) => (
            <option key={node.id} value={node.id}>
              {node.name}
            </option>
          ))}
        </select>
      </label>
    </nav>
  );
}
