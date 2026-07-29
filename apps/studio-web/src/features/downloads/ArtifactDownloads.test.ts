// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  downloadManifestJsonV1,
  downloadWorkerArtifact,
} from "./ArtifactDownloads";
import type { WorkerArtifact } from "../../worker/types";
import { createDownloadManifestV1 } from "./downloadManifest";

const validArtifact = (): WorkerArtifact => ({
  artifactId: "hak",
  kind: "HAK",
  fileName: "model.hak",
  mediaType: "application/octet-stream",
  byteLength: 3,
  sha256: "039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81",
  bytes: new Uint8Array([1, 2, 3]).buffer,
  provenance: "M2A_WASM_WORKER",
});

describe("canonical Worker downloads", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    Object.defineProperty(URL, "createObjectURL", { configurable: true, value: vi.fn(() => "blob:test") });
    Object.defineProperty(URL, "revokeObjectURL", { configurable: true, value: vi.fn() });
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (this: HTMLAnchorElement) {
      expect(document.body.contains(this)).toBe(true);
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it("downloads exact hash-verified bytes and releases the object URL", async () => {
    const artifact = validArtifact();
    await downloadWorkerArtifact(artifact);
    expect(URL.createObjectURL).toHaveBeenCalledOnce();
    expect(HTMLAnchorElement.prototype.click).toHaveBeenCalledOnce();
    vi.runAllTimers();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:test");
  });

  it("rejects non-Worker provenance, length drift and invalid extension", async () => {
    await expect(downloadWorkerArtifact({ ...validArtifact(), provenance: "OTHER" } as unknown as WorkerArtifact))
      .rejects.toThrow("canonical Worker");
    await expect(downloadWorkerArtifact({ ...validArtifact(), byteLength: 4 }))
      .rejects.toThrow("Byte-length mismatch");
    await expect(downloadWorkerArtifact({ ...validArtifact(), fileName: "model.zip" }))
      .rejects.toThrow("Invalid artifact filename");
  });

  it("accepts a generated MOD proof artifact", async () => {
    await downloadWorkerArtifact({
      ...validArtifact(),
      artifactId: "proof-module",
      kind: "MODULE",
      fileName: "m2a_codex_aproof.mod",
    });
    expect(HTMLAnchorElement.prototype.click).toHaveBeenCalledOnce();
  });

  it("rejects byte corruption even when SHA-256 metadata has a valid shape", async () => {
    await expect(downloadWorkerArtifact({
      ...validArtifact(),
      bytes: new Uint8Array([1, 2, 4]).buffer,
    }))
      .rejects.toThrow("SHA-256 mismatch");
    expect(URL.createObjectURL).not.toHaveBeenCalled();
  });

  it("downloads the deterministic project manifest only for the exact artifact inventory", () => {
    const artifacts = [
      {
        ...validArtifact(),
        artifactId: "proof-module",
        kind: "MODULE" as const,
        fileName: "m2abcdef12345678.mod",
      },
      validArtifact(),
    ];
    const manifest = createDownloadManifestV1({
      projectIdentity: {
        schemaVersion: 1,
        projectId: "project-download-01",
        projectName: "Download project",
        projectRevision: 7,
      },
      target: "CREATURE",
      inputs: [{
        role: "SOURCE_GLB",
        fileName: "source.glb",
        byteLength: 3,
        sha256: "a".repeat(64),
      }],
      artifacts,
      offlineReconciled: true,
    });

    downloadManifestJsonV1(manifest, artifacts);
    const click = vi.mocked(HTMLAnchorElement.prototype.click);
    expect(click).toHaveBeenCalledOnce();
    expect(() => downloadManifestJsonV1(manifest, artifacts.slice(0, 1)))
      .toThrow("exact artifact inventory");
  });
});
