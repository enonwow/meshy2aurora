// @vitest-environment jsdom

import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import { animationStudioClipFixtureV1 } from "../animation-studio/testFixtures";
import { ImportAnimationFromModelDialog } from "./ImportAnimationFromModelDialog";
import type { AnimationTransferBatchResultV2, ExternalAnimationSourceInspectionV1 } from "./animationImport";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const donorRevision = "a".repeat(64);
const targetRevision = "b".repeat(64);
const inspection: ExternalAnimationSourceInspectionV1 = {
  sourceRevision: donorRevision,
  rig: [{ id: 1, name: "Root", parentId: null, translation: [0, 0, 0], rotation: [0, 0, 0, 1] }],
  clips: [{ name: "attack", durationSeconds: 1, trackCount: 1 }],
  transferCompatibility: {
    schemaVersion: 1,
    status: "EXACT_COPY",
    donorSourceRevision: donorRevision,
    targetSourceRevision: targetRevision,
    donorRigSignatureSha256: "c".repeat(64),
    targetRigSignatureSha256: "c".repeat(64),
    compatibilityFingerprintSha256: "d".repeat(64),
    allowedModes: ["EXACT_RIG_COPY_V1"],
    mapping: { schemaVersion: 1, rootName: "Root", entries: [{ boneName: "Root", parentName: null, donorNodeId: 1, targetNodeId: 1 }] },
    diagnostics: [],
  },
};
const clip = animationStudioClipFixtureV1({ id: "copied-attack", name: "copied_attack" });
const batch: AnimationTransferBatchResultV2 = {
  schemaVersion: 2,
  commit: "ALL_OR_NOTHING",
  donorSourceRevision: donorRevision,
  targetSourceRevision: targetRevision,
  semanticMapFingerprintSha256: null,
  clips: [{ donorClipName: "attack", mode: "EXACT_RIG_COPY_V1", status: "READY", clip }],
  batchFingerprintSha256: "e".repeat(64),
};

function findButton(container: HTMLElement, text: string) {
  return Array.from(container.querySelectorAll("button")).find((button) => button.textContent?.includes(text));
}

describe("transactional animation batch controls", () => {
  it("ignores a cancelled result and retries from zero before one atomic commit", async () => {
    let resolveFirst!: (value: AnimationTransferBatchResultV2) => void;
    const first = new Promise<AnimationTransferBatchResultV2>((resolve) => { resolveFirst = resolve; });
    const onPrepareBatch = vi.fn()
      .mockImplementationOnce(() => first)
      .mockResolvedValueOnce(batch);
    const onCommitBatch = vi.fn();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    await act(async () => root.render(<ImportAnimationFromModelDialog
      availableDonors={[{ id: "donor", file: new File([new Uint8Array([1])], "donor.glb"), label: "Donor" }]}
      onInspect={vi.fn().mockResolvedValue(inspection)}
      onPrepare={vi.fn().mockResolvedValue(clip)}
      onPrepareBatch={onPrepareBatch}
      onCommit={vi.fn()}
      onCommitBatch={onCommitBatch}
      onClose={vi.fn()}
    />));
    await act(async () => findButton(container, "Donor")?.click());
    await vi.waitFor(() => expect(findButton(container, "Prepare 1 selected clips")).toBeTruthy());
    await act(async () => findButton(container, "Prepare 1 selected clips")?.click());
    expect(findButton(container, "Cancel safely")).toBeTruthy();
    await act(async () => findButton(container, "Cancel safely")?.click());
    await act(async () => resolveFirst(batch));
    expect(container.textContent).not.toContain("1/1 clips ready");
    expect(onCommitBatch).not.toHaveBeenCalled();
    await act(async () => findButton(container, "Retry batch from zero")?.click());
    await vi.waitFor(() => expect(container.textContent).toContain("1/1 clips ready"));
    await act(async () => findButton(container, "Commit 1 clips atomically")?.click());
    expect(onPrepareBatch).toHaveBeenCalledTimes(2);
    expect(onCommitBatch).toHaveBeenCalledTimes(1);
    await act(async () => root.unmount());
  });
});
