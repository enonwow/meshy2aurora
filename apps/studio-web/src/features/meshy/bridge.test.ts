import { describe, expect, it } from "vitest";
import {
  InMemoryMeshyBridgeClient,
  MESHY_PROFILES,
  MeshyBridgeError,
  findMeshyProfile,
} from "./bridge";

describe("Meshy Lab profiles", () => {
  it("keeps the supported proof pipelines constrained and versioned", () => {
    expect(MESHY_PROFILES.map((profile) => profile.id)).toEqual([
      "H1-humanoid-animated/v1",
      "N1-quadruped/v1",
      "S1-static-prop/v1",
    ]);
    expect(findMeshyProfile("H1-humanoid-animated/v1")?.stages).toEqual([
      "PREVIEW", "REFINE", "RIG", "ANIMATE",
    ]);
    expect(findMeshyProfile("N1-quadruped/v1")?.stages).toEqual(["PREVIEW", "REFINE"]);
    expect(findMeshyProfile("S1-static-prop/v1")?.stages).toEqual(["PREVIEW", "REFINE"]);
  });

  it("offers an Aurora proof geometry target instead of forcing the Meshy default budget", async () => {
    const bridge = new InMemoryMeshyBridgeClient();
    const { sessionToken } = await bridge.pair({ pairingCode: "local-proof" });
    await expect(bridge.previewRun(sessionToken, {
      profileId: "S1-static-prop/v1",
      prompt: "A small wooden treasure chest",
      geometryTarget: "AURORA_PROOF",
    })).resolves.toMatchObject({});
  });
});

describe("InMemoryMeshyBridgeClient", () => {
  it("never returns a credential and requires a fresh confirmation nonce before creating a run", async () => {
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 120 });
    const pairing = await bridge.pair({ pairingCode: "local-proof" });
    const health = await bridge.health();
    const preview = await bridge.previewRun(pairing.sessionToken, {
      profileId: "H1-humanoid-animated/v1",
      prompt: "A neutral humanoid adventurer in A-pose",
      geometryTarget: "BALANCED",
      h1Preflight: { standardHumanoid: true, clearLimbs: true, noWeapon: true },
    });

    expect(JSON.stringify({ pairing, health, preview })).not.toContain("MESHY_API_KEY");
    await expect(bridge.createRun(pairing.sessionToken, {
      previewId: preview.previewId,
      confirmationNonce: "",
    })).rejects.toMatchObject({ code: "CONFIRMATION_REQUIRED" });

    const run = await bridge.createRun(pairing.sessionToken, {
      previewId: preview.previewId,
      confirmationNonce: "confirm-once",
    });
    expect(run.status).toBe("QUEUED");
    await expect(bridge.createRun(pairing.sessionToken, {
      previewId: preview.previewId,
      confirmationNonce: "confirm-once",
    })).rejects.toMatchObject({ code: "CONFIRMATION_ALREADY_USED" });
  });

  it("requires the explicit H1 rigging preflight but never imposes it on N1 or S1", async () => {
    const bridge = new InMemoryMeshyBridgeClient();
    const { sessionToken } = await bridge.pair({ pairingCode: "local-proof" });
    await expect(bridge.previewRun(sessionToken, {
      profileId: "H1-humanoid-animated/v1",
      prompt: "A humanoid proof asset",
      geometryTarget: "BALANCED",
    })).rejects.toMatchObject({ code: "H1_PREFLIGHT_REQUIRED" });
    await expect(bridge.previewRun(sessionToken, {
      profileId: "N1-quadruped/v1",
      prompt: "A quadruped proof asset",
      geometryTarget: "BALANCED",
    })).resolves.toMatchObject({ profile: { id: "N1-quadruped/v1" } });
  });

  it("does not expose an artifact before a verified run is ready", async () => {
    const bridge = new InMemoryMeshyBridgeClient();
    const { sessionToken } = await bridge.pair({ pairingCode: "local-proof" });
    const preview = await bridge.previewRun(sessionToken, {
      profileId: "S1-static-prop/v1",
      prompt: "A weathered stone lantern, isolated game asset",
      geometryTarget: "BALANCED",
    });
    const run = await bridge.createRun(sessionToken, {
      previewId: preview.previewId,
      confirmationNonce: "confirm-static-prop",
    });

    await expect(bridge.downloadArtifact(sessionToken, run.id)).rejects.toBeInstanceOf(MeshyBridgeError);

    const ready = await bridge.completeRunForTest(run.id, new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
    expect(ready.status).toBe("READY");
    const artifact = await bridge.downloadArtifact(sessionToken, run.id);
    expect(artifact.file.name).toBe("meshy-s1-static-prop.glb");
    expect(artifact.provenance.profileId).toBe("S1-static-prop/v1");
    expect(artifact.provenance.sha256).toMatch(/^[a-f0-9]{64}$/);
  });
});
