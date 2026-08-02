import { describe, expect, it } from "vitest";
import {
  AURORA_MODEL_TRIANGLE_BUDGET_V1,
  DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
  InMemoryMeshyBridgeClient,
  MESHY_PROFILES,
  MeshyBridgeError,
  findMeshyProfile,
  maximumMeshyTargetPolycount,
  resolveLocalMeshyBridgeOrigin,
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

  it("uses the shared Aurora triangle budget for Studio generation inputs", async () => {
    expect(AURORA_MODEL_TRIANGLE_BUDGET_V1).toBe(300_000);
    expect(DEFAULT_MESHY_TEXT_TO_3D_OPTIONS.targetPolycount).toBe(300_000);
    expect(maximumMeshyTargetPolycount(DEFAULT_MESHY_TEXT_TO_3D_OPTIONS)).toBe(300_000);
    expect(maximumMeshyTargetPolycount({
      modelType: "smart-topology",
      aiModel: "meshy-t2",
      rigHumanoid: false,
    }))
      .toBe(15_000);

    const bridge = new InMemoryMeshyBridgeClient();
    const { sessionToken } = await bridge.pair({ pairingCode: "local-proof" });
    const request = {
      profileId: "S1-static-prop/v1" as const,
      prompt: "A small wooden treasure chest",
      geometryTarget: "BALANCED" as const,
      apiOptions: DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
    };
    await expect(bridge.previewRun(sessionToken, request)).resolves.toMatchObject({});
    await expect(bridge.previewRun(sessionToken, {
      ...request,
      apiOptions: {
        ...DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
        targetPolycount: AURORA_MODEL_TRIANGLE_BUDGET_V1 + 1,
      },
    })).rejects.toMatchObject({ code: "PREVIEW_NOT_FOUND" });
  });

  it("accepts image and multi-image preview contracts without inventing a text prompt", async () => {
    const bridge = new InMemoryMeshyBridgeClient();
    const { sessionToken } = await bridge.pair({ pairingCode: "local-proof" });
    await expect(bridge.previewRun(sessionToken, {
      profileId: "S1-static-prop/v1",
      prompt: "",
      geometryTarget: "BALANCED",
      source: "IMAGE",
      imageDataUrls: ["data:image/png;base64,AAAA"],
    })).resolves.toMatchObject({ profile: { id: "S1-static-prop/v1" } });
    await expect(bridge.previewRun(sessionToken, {
      profileId: "S1-static-prop/v1",
      prompt: "",
      geometryTarget: "BALANCED",
      source: "MULTI_IMAGE",
      imageDataUrls: [],
    })).rejects.toMatchObject({ code: "PREVIEW_NOT_FOUND" });
  });

  it("quotes the exact generation ceiling for source, model, and texture resolution", async () => {
    const bridge = new InMemoryMeshyBridgeClient();
    const { sessionToken } = await bridge.pair({ pairingCode: "local-proof" });
    const smartTopology = await bridge.previewRun(sessionToken, {
      profileId: "S1-static-prop/v1",
      prompt: "",
      geometryTarget: "BALANCED",
      source: "IMAGE",
      imageDataUrls: ["data:image/png;base64,AAAA"],
      apiOptions: {
        ...DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
        modelType: "smart-topology",
        aiModel: "meshy-t2",
        targetPolycount: 15_000,
        textureResolution: "8k",
      },
    });
    expect(smartTopology.maximumCredits).toBe(20);

    const meshy5Text = await bridge.previewRun(sessionToken, {
      profileId: "S1-static-prop/v1",
      prompt: "A compact game prop",
      geometryTarget: "BALANCED",
      source: "TEXT",
      apiOptions: {
        ...DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
        aiModel: "meshy-5",
        textureResolution: "2k",
      },
    });
    expect(meshy5Text.maximumCredits).toBe(15);
    await expect(bridge.previewRun(sessionToken, {
      profileId: "S1-static-prop/v1",
      prompt: "A misleading untextured text request",
      geometryTarget: "BALANCED",
      source: "TEXT",
      apiOptions: {
        ...DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
        shouldTexture: false,
      },
    })).rejects.toMatchObject({ code: "PREVIEW_NOT_FOUND" });
  });
});

describe("local Bridge origin", () => {
  it("allows the synthetic proof Bridge only in a development proof session", () => {
    expect(resolveLocalMeshyBridgeOrigin({ configured: "http://127.0.0.1:43119", development: true, search: "?meshyProofBridge=1" }))
      .toBe("http://127.0.0.1:43120");
    expect(resolveLocalMeshyBridgeOrigin({ configured: "http://127.0.0.1:43119", development: false, search: "?meshyProofBridge=1" }))
      .toBe("http://127.0.0.1:43119");
  });
});

describe("InMemoryMeshyBridgeClient", () => {
  it("keeps Material Matching behind a preview and one-time paid confirmation", async () => {
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 120 });
    const { sessionToken } = await bridge.pair({ pairingCode: "local-proof" });
    const preview = await bridge.previewRetexture(sessionToken, {
      inputTaskId: "verified-refine-task",
      textStylePrompt: "aged brass and green patina",
      aiModel: "meshy-6",
      enableOriginalUv: true,
      enablePbr: true,
      textureResolution: "2k",
      removeLighting: true,
      alphaThumbnail: false,
    });
    expect(preview.maximumCredits).toBe(10);
    await expect(bridge.createRetexture(sessionToken, { previewId: preview.previewId, confirmationNonce: "" }))
      .rejects.toMatchObject({ code: "CONFIRMATION_REQUIRED" });
    await expect(bridge.createRetexture(sessionToken, { previewId: preview.previewId, confirmationNonce: "confirm-retexture" }))
      .resolves.toMatchObject({ status: "QUEUED", inputTaskId: "verified-refine-task" });
  });

  it("requires an explicit confirmation for 2D generation and never echoes Image-to-Image input data", async () => {
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 20 });
    const { sessionToken } = await bridge.pair({ pairingCode: "local-proof" });
    const preview = await bridge.previewImageRun(sessionToken, {
      mode: "IMAGE_TO_IMAGE",
      prompt: "Turn this lantern into a clean game concept",
      aiModel: "nano-banana",
      generateMultiView: false,
      aspectRatio: "1:1",
      referenceImageDataUrls: ["data:image/png;base64,AAAA"],
    });
    expect(preview.maximumCredits).toBe(3);
    expect(JSON.stringify(preview)).not.toContain("data:image");
    await expect(bridge.createImageRun(sessionToken, { previewId: preview.previewId, confirmationNonce: "" }))
      .rejects.toMatchObject({ code: "CONFIRMATION_REQUIRED" });
    await expect(bridge.createImageRun(sessionToken, { previewId: preview.previewId, confirmationNonce: "confirm-image" }))
      .resolves.toMatchObject({ status: "QUEUED", mode: "IMAGE_TO_IMAGE" });
  });

  it("never returns a credential and requires a fresh confirmation nonce before creating a run", async () => {
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 120 });
    const pairing = await bridge.pair({ pairingCode: "local-proof" });
    const health = await bridge.health();
    const preview = await bridge.previewRun(pairing.sessionToken, {
      profileId: "H1-humanoid-animated/v1",
      prompt: "A neutral humanoid adventurer in A-pose",
      geometryTarget: "BALANCED",
      h1Preflight: {
        standardHumanoid: true,
        clearLimbs: true,
        noWeapon: true,
        aOrTPose: true,
      },
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

  it("keeps every requested animation action as a separately verified GLB", async () => {
    const bridge = new InMemoryMeshyBridgeClient();
    const { sessionToken } = await bridge.pair({ pairingCode: "local-proof" });
    const preview = await bridge.previewRun(sessionToken, {
      profileId: "H1-humanoid-animated/v1",
      prompt: "A humanoid with two animation donors",
      geometryTarget: "BALANCED",
      h1Preflight: {
        standardHumanoid: true,
        clearLimbs: true,
        noWeapon: true,
        aOrTPose: true,
      },
      apiOptions: {
        ...DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
        rigHumanoid: true,
        targetPolycount: 300_000,
        poseMode: "a-pose",
        animationActionIds: [0, 92],
      },
    });
    expect(preview.maximumCredits).toBe(41);
    const run = await bridge.createRun(sessionToken, {
      previewId: preview.previewId,
      confirmationNonce: "confirm-multi-animation",
    });
    await bridge.completeAnimationRunForTest(run.id, [{
      actionId: 0,
      bytes: new Uint8Array([0x67, 0x6c, 0x54, 0x46, 0]),
    }, {
      actionId: 92,
      bytes: new Uint8Array([0x67, 0x6c, 0x54, 0x46, 92]),
    }]);

    const provenance = await bridge.provenance(sessionToken, run.id);
    expect(provenance.animationArtifacts?.map(({ actionId }) => actionId))
      .toEqual([0, 92]);
    const attack = await bridge.downloadAnimationArtifact(
      sessionToken,
      run.id,
      92,
    );
    expect(attack.file.name).toBe("meshy-animation-92.glb");
    expect(attack.provenance.selectedAnimationActionId).toBe(92);
    expect(attack.provenance.sha256)
      .toBe(provenance.animationArtifacts?.[1]?.sha256);
  });
});
