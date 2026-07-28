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
      <h3>Bones</h3>
      <div role="tree">
        {nodes.map((node, index) => (
          <button
            key={node.id}
            type="button"
            role="treeitem"
            aria-selected={selectedNodeId === node.id}
            aria-level={treeNodeDepthV1(node.id, nodes)}
            tabIndex={selectedNodeId === node.id || (selectedNodeId === null && index === 0)
              ? 0
              : -1}
            data-parent-node-id={node.parentId ?? "root"}
            onClick={() => onSelect(node.id)}
            onKeyDown={moveTreeFocus}
          >
            {node.name} <small>#{node.id}</small>
          </button>
        ))}
      </div>
    </nav>
  );
}

function moveTreeFocus(event: KeyboardEvent<HTMLButtonElement>) {
  if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
  const items = Array.from(
    event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>(
      ':scope > [role="treeitem"]',
    ) ?? [],
  );
  const current = items.indexOf(event.currentTarget);
  if (current < 0 || items.length === 0) return;
  event.preventDefault();
  const next = event.key === "Home"
    ? 0
    : event.key === "End"
      ? items.length - 1
      : event.key === "ArrowDown"
        ? Math.min(current + 1, items.length - 1)
        : Math.max(current - 1, 0);
  items[next]?.focus();
  items[next]?.click();
}

function treeNodeDepthV1(
  nodeId: number,
  nodes: readonly AnimationRigNodeV1[],
) {
  const byId = new Map(nodes.map((node) => [node.id, node]));
  let depth = 1;
  let current = byId.get(nodeId);
  const visited = new Set<number>();
  while (current?.parentId !== null && current?.parentId !== undefined) {
    if (visited.has(current.id)) break;
    visited.add(current.id);
    depth += 1;
    current = byId.get(current.parentId);
  }
  return depth;
}
import type { KeyboardEvent } from "react";
