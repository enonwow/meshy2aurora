// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { WorkerArtifact } from "../../worker/types";
import { ArtifactDownloads } from "./ArtifactDownloads";
import {
  createDownloadManifestV1,
  projectDownloadReadinessV1,
} from "./downloadManifest";

const roots: Root[] = [];
const projectIdentity = {
  schemaVersion: 1,
  projectId: "project-download-01",
  projectName: "Download project",
  projectRevision: 7,
} as const;
const artifacts: WorkerArtifact[] = [{
  artifactId: "proof-module",
  kind: "MODULE",
  fileName: "m2abcdef12345678.mod",
  mediaType: "application/octet-stream",
  byteLength: 1,
  sha256: "a".repeat(64),
  bytes: new Uint8Array([1]).buffer,
  provenance: "M2A_WASM_WORKER",
}, {
  artifactId: "package-hak",
  kind: "HAK",
  fileName: "m2abcdef12345678.hak",
  mediaType: "application/octet-stream",
  byteLength: 1,
  sha256: "b".repeat(64),
  bytes: new Uint8Array([2]).buffer,
  provenance: "M2A_WASM_WORKER",
}];
const manifest = createDownloadManifestV1({
  projectIdentity,
  target: "CREATURE",
  inputs: [{
    role: "SOURCE_GLB",
    fileName: "source.glb",
    byteLength: 1,
    sha256: "c".repeat(64),
  }],
  artifacts,
  offlineReconciled: true,
});

async function render(readiness: ReturnType<typeof projectDownloadReadinessV1>) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  await act(async () => root.render(
    <ArtifactDownloads
      artifacts={artifacts}
      manifest={manifest}
      readiness={readiness}
      onError={vi.fn()}
    />,
  ));
  return container;
}

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
});

describe("project-bound artifact downloads", () => {
  it("locks every download when the current project revision differs", async () => {
    const container = await render(projectDownloadReadinessV1({
      currentProjectIdentity: { ...projectIdentity, projectRevision: 8 },
      builtProjectIdentity: projectIdentity,
      offlineReconciled: true,
    }));

    expect(container.textContent).toContain("different project revision");
    expect(container.textContent).toContain("owner proof pending");
    const buttons = Array.from(container.querySelectorAll<HTMLButtonElement>("button"));
    expect(buttons).toHaveLength(3);
    expect(buttons.every(({ disabled }) => disabled)).toBe(true);
  });

  it("enables the manifest and exact artifacts only for the reconciled current revision", async () => {
    const container = await render(projectDownloadReadinessV1({
      currentProjectIdentity: projectIdentity,
      builtProjectIdentity: projectIdentity,
      offlineReconciled: true,
    }));

    expect(container.textContent).toContain("Exact current project revision");
    expect(container.textContent).toContain("m2abcdef12345678-download-manifest.json");
    const buttons = Array.from(container.querySelectorAll<HTMLButtonElement>("button"));
    expect(buttons).toHaveLength(3);
    expect(buttons.every(({ disabled }) => !disabled)).toBe(true);
  });
});
