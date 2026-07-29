import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import { resolveOwnerRuntimeProofV1 } from "./ownerRuntimeProofs";
import { projectCanonicalResult } from "./projectCanonicalResult";
import type { WorkerArtifact } from "../../worker/types";

const v5Outputs = {
  model: {
    byteLength: 1,
    sha256: "c8b173a59a576911ff938b7287f823694474be838a81cb55c370c8c3258eb6e9",
  },
  hak: {
    byteLength: 1,
    sha256: "90d79114f0bc725e91f54657dc39f7491862ab4c0f54e032703ac4fddd6c9853",
  },
  texture: {
    byteLength: 1,
    sha256: "ac6aceb3f2809c5ffe1170d2525df672fb077855cf8bc1d78f24241596f05fbc",
  },
  proofModule: {
    byteLength: 1,
    sha256: "da60bc35b45f7ce53753ed91d6c2c13ce09192b6d63143acf2fffca2c94cf1df",
  },
};

describe("owner runtime proof registry", () => {
  it("promotes only the exact owner-verified Stoneback V5 lineage", () => {
    expect(resolveOwnerRuntimeProofV1(v5Outputs)).toMatchObject({
      status: "OWNER_VERIFIED",
      evidenceId: "tlc-stoneback-brute-p300k-geometry-ab-v5-2026-07-29",
      toolset: { modelVisibility: "visible", proofCompleteness: "verified" },
      nwn: { modelVisibility: "visible", proofCompleteness: "verified" },
      geometryArtifactResult: "fixed",
    });
  });

  it("keeps OPEN_M6 when any immutable lineage hash differs", () => {
    expect(resolveOwnerRuntimeProofV1({
      ...v5Outputs,
      model: { ...v5Outputs.model, sha256: "0".repeat(64) },
    })).toEqual({
      status: "OPEN_M6",
      reason: "No exact owner-verified Aurora/NWN lineage matches these output hashes.",
    });
  });

  it("does not promote a product-only result without an exact proof MOD", () => {
    expect(resolveOwnerRuntimeProofV1({
      model: v5Outputs.model,
      hak: v5Outputs.hak,
    })).toEqual({
      status: "OPEN_M6",
      reason: "No exact owner-verified Aurora/NWN lineage matches these output hashes.",
    });
  });
});

const v5Packet = resolve(
  process.cwd(),
  "../../proof-output/tlc-stoneback-brute-p300k-geometry-ab-v5/studio-export",
);

function artifact(
  artifactId: string,
  kind: WorkerArtifact["kind"],
  fileName: string,
): WorkerArtifact {
  const bytes = readFileSync(resolve(v5Packet, fileName));
  return {
    artifactId,
    kind,
    fileName,
    mediaType: "application/octet-stream",
    byteLength: bytes.byteLength,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    bytes: bytes.buffer.slice(
      bytes.byteOffset,
      bytes.byteOffset + bytes.byteLength,
    ) as ArrayBuffer,
    provenance: "M2A_WASM_WORKER",
  };
}

describe.skipIf(!existsSync(v5Packet))("exact local Stoneback V5 Studio packet", () => {
  it("projects the immutable packet as owner verified instead of OPEN_M6", () => {
    const reportJson = readFileSync(resolve(v5Packet, "inspection.json"), "utf8");
    const manifestJson = readFileSync(
      resolve(v5Packet, "conversion-manifest.json"),
      "utf8",
    );
    const summaryJson = readFileSync(resolve(v5Packet, "summary.json"), "utf8");
    const snapshot = projectCanonicalResult(
      reportJson,
      summaryJson,
      manifestJson,
      [
        artifact("package-hak", "HAK", "m2p3jh0eeb135c.hak"),
        artifact("model-mdl", "MODEL", "m2p3jm0eeb135c.mdl"),
        artifact("texture-tga", "TEXTURE", "m2p3jt0eeb135c.tga"),
        artifact("proof-module", "MODULE", "m2p3jd0eeb135c.mod"),
        artifact("report-json", "JSON_REPORT", "inspection.json"),
        artifact("manifest-json", "JSON_REPORT", "conversion-manifest.json"),
        artifact("summary-json", "JSON_REPORT", "summary.json"),
      ],
    );

    expect(snapshot.runtimeAcceptance).toMatchObject({
      status: "OWNER_VERIFIED",
      toolset: { modelVisibility: "visible", proofCompleteness: "verified" },
      nwn: { modelVisibility: "visible", proofCompleteness: "verified" },
      geometryArtifactResult: "fixed",
    });
    expect(snapshot.geometry.triangles).toBe(296_276);
    expect(snapshot.convertedMetrics.triangles).toBe(296_276);
  });
});
