// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { downloadWorkerArtifact } from "./ArtifactDownloads";
import type { WorkerArtifact } from "../../worker/types";

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
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => undefined);
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

  it("rejects byte corruption even when SHA-256 metadata has a valid shape", async () => {
    await expect(downloadWorkerArtifact({
      ...validArtifact(),
      bytes: new Uint8Array([1, 2, 4]).buffer,
    }))
      .rejects.toThrow("SHA-256 mismatch");
    expect(URL.createObjectURL).not.toHaveBeenCalled();
  });
});
