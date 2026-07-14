import { afterEach, describe, expect, it } from "vitest";
import sourceUrl from "../.generated/owned-package/generated/source-owned.glb?url";
import expectedHakUrl from "../.generated/owned-package/generated/m2a_m6p01.hak?url";
import appearanceUrl from "../fixtures/appearance.2da?url";
import { StudioWorkerClient } from "../../src/worker/client";

const clients: StudioWorkerClient[] = [];

async function fetchBytes(url: string): Promise<ArrayBuffer> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`fixture fetch failed: ${response.status} ${url}`);
  }
  return response.arrayBuffer();
}

async function fixtureFile(url: string, name: string, type: string): Promise<File> {
  return new File([await fetchBytes(url)], name, { type });
}

async function sha256(bytes: ArrayBuffer): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

afterEach(() => {
  while (clients.length) clients.pop()?.dispose();
});

describe("local file to canonical web-WASM Worker integration", () => {
  it("materializes the owned synthetic GLB as the native-identical HAK and reports", async () => {
    const source = await fixtureFile(sourceUrl, "source-owned.synthetic.glb", "model/gltf-binary");
    const appearance = await fixtureFile(appearanceUrl, "appearance.2da", "text/plain");
    const expectedHak = await fetchBytes(expectedHakUrl);
    const sourceGlb = await source.arrayBuffer();
    const appearanceTwoDa = await appearance.arrayBuffer();
    const client = new StudioWorkerClient();
    clients.push(client);

    const initialized = await client.request({ requestId: "integration-init", type: "INITIALIZE" });
    expect(initialized).toMatchObject({ ok: true, type: "INITIALIZED" });

    const response = await client.request(
      {
        requestId: "integration-build",
        type: "BUILD_MODEL_PACKAGE",
        sourceGlb,
        appearanceTwoDa,
      },
      [sourceGlb, appearanceTwoDa],
    );

    expect(sourceGlb.byteLength).toBe(0);
    expect(appearanceTwoDa.byteLength).toBe(0);
    expect(response.ok).toBe(true);
    expect(response.type).toBe("MODEL_PACKAGE_BUILT");
    if (!response.ok || response.type !== "MODEL_PACKAGE_BUILT") {
      throw new Error("real Worker did not return a model package");
    }

    const hak = response.artifacts.find((artifact) => artifact.kind === "HAK");
    const reportArtifact = response.artifacts.find(
      (artifact) => artifact.fileName === "inspection.json",
    );
    expect(hak).toBeDefined();
    expect(reportArtifact).toBeDefined();
    expect(hak?.provenance).toBe("M2A_WASM_WORKER");
    expect(new Uint8Array(hak!.bytes)).toEqual(new Uint8Array(expectedHak));
    expect(new TextDecoder().decode(hak!.bytes.slice(0, 8))).toBe("HAK V1.0");
    expect(hak?.byteLength).toBe(hak?.bytes.byteLength);
    expect(hak?.sha256).toBe(await sha256(hak!.bytes));

    const report = JSON.parse(response.reportJson) as {
      geometry?: { triangleCount?: number };
      texture?: { width?: number; height?: number };
    };
    const summary = JSON.parse(response.summaryJson) as {
      status?: string;
      inputGlb?: { byteLength?: number };
    };
    const manifest = JSON.parse(response.manifestJson) as {
      packageManifest?: { resources?: unknown[] };
    };
    const readback = JSON.parse(response.readbackJson) as {
      nodeTree?: { roots?: unknown[] };
    };
    expect(report.geometry?.triangleCount).toBeGreaterThan(1);
    expect(report.texture).toMatchObject({ width: 2, height: 2 });
    expect(summary.status).toBe("M6_MODEL_PACKAGE_MATERIALIZED");
    expect(summary.inputGlb?.byteLength).toBe(source.size);
    expect(manifest.packageManifest?.resources).toHaveLength(3);
    expect(readback.nodeTree?.roots?.length).toBeGreaterThan(0);
    expect(new TextDecoder().decode(reportArtifact!.bytes)).toBe(response.reportJson);
  }, 30_000);
});
