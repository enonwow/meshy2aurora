import { describe, expect, it } from "vitest";
import type { WorkerArtifact } from "../../worker/types";
import {
  createDownloadManifestV1,
  downloadManifestFileNameV1,
  projectDownloadReadinessV1,
  serializeDownloadManifestV1,
} from "./downloadManifest";

const projectIdentity = {
  schemaVersion: 1,
  projectId: "project-download-01",
  projectName: "Download project",
  projectRevision: 7,
} as const;

const artifact = (
  artifactId: string,
  kind: WorkerArtifact["kind"],
  fileName: string,
  marker: string,
): WorkerArtifact => ({
  artifactId,
  kind,
  fileName,
  mediaType: kind === "JSON_REPORT" ? "application/json" : "application/octet-stream",
  byteLength: 1,
  sha256: marker.repeat(64),
  bytes: new Uint8Array([1]).buffer,
  provenance: "M2A_WASM_WORKER",
});

describe("DownloadManifestV1", () => {
  it("records exact project, input, output and pending owner-proof identities deterministically", () => {
    const artifacts = [
      artifact("proof-module", "MODULE", "m2abcdef12345678.mod", "a"),
      artifact("package-hak", "HAK", "m2abcdef12345678.hak", "b"),
    ];
    const create = () => createDownloadManifestV1({
      projectIdentity,
      target: "CREATURE",
      inputs: [{
        role: "SOURCE_GLB",
        fileName: "source.glb",
        byteLength: 123,
        sha256: "c".repeat(64),
      }, {
        role: "ANIMATION_MAPPING_V2",
        fileName: "animation-mapping-v2.json",
        byteLength: null,
        sha256: "d".repeat(64),
      }],
      artifacts,
      offlineReconciled: true,
    });

    expect(create()).toEqual(create());
    expect(create()).toMatchObject({
      schemaVersion: 1,
      projectIdentity,
      target: "CREATURE",
      buildStatus: "OFFLINE_READBACK_RECONCILED",
      ownerProof: {
        status: "PENDING_OWNER",
        modelVisibility: "not_tested",
        proofCompleteness: "missing",
      },
      zipPolicy: "NOT_INCLUDED_IN_E4_CANONICAL_DELIVERY",
    });
    expect(create().outputs).toEqual([
      expect.objectContaining({ artifactId: "package-hak", sha256: "b".repeat(64) }),
      expect.objectContaining({ artifactId: "proof-module", sha256: "a".repeat(64) }),
    ]);
    expect(serializeDownloadManifestV1(create())).toBe(
      serializeDownloadManifestV1(create()),
    );
    expect(downloadManifestFileNameV1(artifacts))
      .toBe("m2abcdef12345678-download-manifest.json");
  });

  it("locks stale project revisions and incomplete readback", () => {
    expect(projectDownloadReadinessV1({
      currentProjectIdentity: { ...projectIdentity, projectRevision: 8 },
      builtProjectIdentity: projectIdentity,
      offlineReconciled: true,
    })).toMatchObject({ allowed: false, status: "LOCKED" });
    expect(projectDownloadReadinessV1({
      currentProjectIdentity: projectIdentity,
      builtProjectIdentity: projectIdentity,
      offlineReconciled: false,
    })).toMatchObject({ allowed: false, status: "LOCKED" });
    expect(projectDownloadReadinessV1({
      currentProjectIdentity: projectIdentity,
      builtProjectIdentity: projectIdentity,
      offlineReconciled: true,
    })).toMatchObject({ allowed: true, status: "READY" });
  });

  it("names a product-only manifest from its exact HAK when no demo module exists", () => {
    expect(downloadManifestFileNameV1([
      artifact("package-hak", "HAK", "m2a_product01.hak", "a"),
      artifact("model-mdl", "MODEL", "m2a_product01.mdl", "b"),
    ])).toBe("m2a_product01-download-manifest.json");
  });

  it("rejects malformed hashes and unverified output identities", () => {
    expect(() => createDownloadManifestV1({
      projectIdentity,
      target: "CREATURE",
      inputs: [{
        role: "SOURCE_GLB",
        fileName: "source.glb",
        byteLength: 1,
        sha256: "not-a-hash",
      }],
      artifacts: [artifact("proof-module", "MODULE", "model.mod", "a")],
      offlineReconciled: false,
    })).toThrow("input identity");
    expect(() => createDownloadManifestV1({
      projectIdentity,
      target: "CREATURE",
      inputs: [{
        role: "SOURCE_GLB",
        fileName: "source.glb",
        byteLength: 1,
        sha256: "c".repeat(64),
      }],
      artifacts: [{
        ...artifact("proof-module", "MODULE", "model.mod", "a"),
        byteLength: 2,
      }],
      offlineReconciled: false,
    })).toThrow("output identity");
  });
});
