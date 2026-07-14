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
  sha256: "a".repeat(64),
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

  it("downloads exact bytes and releases the object URL", () => {
    const artifact = validArtifact();
    downloadWorkerArtifact(artifact);
    expect(URL.createObjectURL).toHaveBeenCalledOnce();
    expect(HTMLAnchorElement.prototype.click).toHaveBeenCalledOnce();
    vi.runAllTimers();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:test");
  });

  it("rejects non-Worker provenance, length drift and invalid extension", () => {
    expect(() => downloadWorkerArtifact({ ...validArtifact(), provenance: "OTHER" } as unknown as WorkerArtifact))
      .toThrow("canonical Worker");
    expect(() => downloadWorkerArtifact({ ...validArtifact(), byteLength: 4 }))
      .toThrow("Byte-length mismatch");
    expect(() => downloadWorkerArtifact({ ...validArtifact(), fileName: "model.zip" }))
      .toThrow("Invalid artifact filename");
  });
});
