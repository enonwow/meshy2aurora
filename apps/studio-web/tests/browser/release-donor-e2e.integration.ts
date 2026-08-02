import { afterEach, describe, expect, it } from "vitest";
import ownerH1Url from
  "@m2a-canonical-repository/sample-3d/h1-humanoid-1500/source.glb?url";
import type { AnimationRigNodeV1 } from
  "../../src/features/animation-editor/AnimationBoneTree";
import {
  serializeAnimationStudioDocumentV1,
} from "../../src/features/animation-studio/schema";
import {
  serializeCreatureAnimationAuthoringV2,
} from "../../src/features/animation-studio/mappingV2Persistence";
import {
  createMeshy2AuroraProjectV1,
  projectBuildIdentityV1,
  projectFileReferenceV1,
  reviseMeshy2AuroraProjectV1,
} from "../../src/features/project";
import type {
  AnimationStudioDocumentV1,
  AuthoredAnimationClipV1,
  CreatureAnimationAuthoringV2,
} from "../../src/features/animation-studio/types";
import {
  FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
} from "../../src/features/source/directCreatureAnimationProfile";
import { StudioWorkerClient } from "../../src/worker/client";

const OWNER_H1_SHA256 =
  "3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f";
const SOURCE_CLIP_NAME = "Armature|Idle|baselayer";
const DONOR_CLIP_NAME = "donor_wave";
const clients: StudioWorkerClient[] = [];

interface EditableSourceInspectionV1 {
  readonly sourceRevision: string;
  readonly rig: AnimationRigNodeV1[];
  readonly clips: Array<{
    name: string;
    durationSeconds: number;
    trackCount: number;
  }>;
  readonly clip: AuthoredAnimationClipV1 | null;
}

async function fetchBytes(url: string) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`fixture fetch failed: ${response.status} ${url}`);
  }
  return response.arrayBuffer();
}

async function sha256(bytes: ArrayBuffer) {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function utf8Bytes(value: string): ArrayBuffer {
  return new TextEncoder().encode(value).buffer as ArrayBuffer;
}

function replaceJsonStringOnce(
  source: ArrayBuffer,
  expected: string,
  replacement: string,
) {
  const expectedBytes = new TextEncoder().encode(JSON.stringify(expected));
  const replacementBytes = new TextEncoder().encode(JSON.stringify(replacement));
  if (replacementBytes.byteLength > expectedBytes.byteLength) {
    throw new Error("The deterministic donor label must fit the GLB JSON field.");
  }
  const output = new Uint8Array(source.slice(0));
  let matchOffset = -1;
  for (
    let offset = 0;
    offset <= output.byteLength - expectedBytes.byteLength;
    offset += 1
  ) {
    if (expectedBytes.every((byte, index) => output[offset + index] === byte)) {
      if (matchOffset !== -1) {
        throw new Error("The source clip label is not unique in the owner GLB.");
      }
      matchOffset = offset;
    }
  }
  if (matchOffset === -1) {
    throw new Error("The expected owner clip label is absent from the GLB.");
  }
  output.fill(
    " ".charCodeAt(0),
    matchOffset,
    matchOffset + expectedBytes.byteLength,
  );
  output.set(replacementBytes, matchOffset);
  return output.buffer;
}

afterEach(() => {
  while (clients.length) clients.pop()?.dispose();
});

describe("E7 real owner-asset donor import boundary", () => {
  it("imports, validates, previews and materializes a distinct compatible donor without freezing V5 artifacts", async () => {
    const ownerBytes = await fetchBytes(ownerH1Url);
    expect(ownerBytes.byteLength).toBe(7_944_380);
    expect(await sha256(ownerBytes)).toBe(OWNER_H1_SHA256);
    const donorBytes = replaceJsonStringOnce(
      ownerBytes,
      SOURCE_CLIP_NAME,
      DONOR_CLIP_NAME,
    );
    const donorSha256 = await sha256(donorBytes);
    expect(donorSha256).not.toBe(OWNER_H1_SHA256);

    const client = new StudioWorkerClient();
    clients.push(client);
    await client.request({
      requestId: "e7-donor-initialize",
      type: "INITIALIZE",
    });

    const currentResponse = await client.inspectEditableAnimationSource(
      ownerBytes.slice(0),
      undefined,
      "e7-current-owner-h1",
    );
    const donorResponse = await client.inspectEditableAnimationSource(
      donorBytes.slice(0),
      DONOR_CLIP_NAME,
      "e7-derived-donor-h1",
    );
    if (
      !currentResponse.ok
      || currentResponse.type !== "EDITABLE_ANIMATION_SOURCE_INSPECTED"
      || !donorResponse.ok
      || donorResponse.type !== "EDITABLE_ANIMATION_SOURCE_INSPECTED"
    ) {
      throw new Error("The exact current/donor owner GLBs were not inspected.");
    }
    const current = JSON.parse(
      currentResponse.inspectionJson,
    ) as EditableSourceInspectionV1;
    const donor = JSON.parse(
      donorResponse.inspectionJson,
    ) as EditableSourceInspectionV1;
    expect(current.sourceRevision).toBe(OWNER_H1_SHA256);
    expect(donor.sourceRevision).toBe(donorSha256);
    expect(donor.clips).toEqual([
      expect.objectContaining({ name: DONOR_CLIP_NAME }),
    ]);
    const compatibilityResponse = await client.inspectAnimationTransferCompatibility(
      ownerBytes.slice(0),
      donorBytes.slice(0),
      "e7-owner-donor-compatibility",
    );
    if (
      !compatibilityResponse.ok
      || compatibilityResponse.type
        !== "ANIMATION_TRANSFER_COMPATIBILITY_INSPECTED"
    ) {
      throw new Error("Core did not classify the exact owner/donor pair.");
    }
    expect(JSON.parse(compatibilityResponse.compatibilityJson)).toMatchObject({
      status: "EXACT_COPY",
      donorSourceRevision: donorSha256,
      targetSourceRevision: OWNER_H1_SHA256,
      allowedModes: expect.arrayContaining(["EXACT_RIG_COPY_V1"]),
    });
    const transferResponse = await client.retargetAnimationModelClip(
      ownerBytes.slice(0),
      donorBytes.slice(0),
      DONOR_CLIP_NAME,
      JSON.stringify({
        mode: "EXACT_RIG_COPY_V1",
        clipId: "e7-owner-donor-idle",
        outputName: "e7_donor_idle",
      }),
      "e7-owner-donor-transfer",
    );
    if (
      !transferResponse.ok
      || transferResponse.type !== "ANIMATION_MODEL_CLIP_RETARGETED"
    ) {
      throw new Error("Core did not transfer the exact donor clip.");
    }
    const transferred = JSON.parse(transferResponse.resultJson) as {
      clip: AuthoredAnimationClipV1;
    };
    const imported = {
      ...transferred.clip,
      status: "VALID" as const,
    };
    const document: AnimationStudioDocumentV1 = {
      schemaVersion: 1,
      sourceRevision: OWNER_H1_SHA256,
      authoringRevision: 1,
      status: "VALID",
      authoredClips: [imported],
    };
    const documentJson = serializeAnimationStudioDocumentV1(document);
    const animationMapping: CreatureAnimationAuthoringV2 = {
      schemaVersion: 2,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      modelType: "S",
      sourceRevision: OWNER_H1_SHA256,
      authoringRevision: 1,
      assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((targetSlot) => (
        targetSlot === "cpause1"
          ? {
              targetSlot,
              sourceKind: "CUSTOM",
              sourceClipName: null,
              customAnimationId: "e7-owner-donor-custom",
              provenance: {
                provider: "USER_CUSTOM",
                assetId: imported.id,
                ownership: "USER_OWNED",
              },
            } as const
          : {
              targetSlot,
              sourceKind: "PROCEDURAL",
              sourceClipName: null,
              customAnimationId: null,
              provenance: {
                provider: "PROCEDURAL_GENERATOR",
                assetId: "m2a:procedural-humanoid-v1",
                ownership: "PROJECT_GENERATED",
              },
            } as const
      )),
      fallbacks: [],
      customAnimations: [{
        id: "e7-owner-donor-custom",
        name: "e7donor",
        playback: "ONE_SHOT",
        clipReference: {
          sourceKind: "AUTHORED_CLIP",
          sourceClipName: null,
          authoredClipId: imported.id,
        },
        phases: [],
        provenance: {
          provider: "USER_CUSTOM",
          assetId: imported.id,
          ownership: "USER_OWNED",
        },
      }],
    };
    const animationMappingJson =
      serializeCreatureAnimationAuthoringV2(animationMapping);
    const animationMappingFingerprint = await sha256(
      utf8Bytes(animationMappingJson),
    );
    const releaseProjectSource = createMeshy2AuroraProjectV1({
      projectId: "e7-owner-h1-donor",
      name: "E7 Owner H1 Donor",
      target: "CREATURE",
      now: "2026-07-29T00:00:00.000Z",
    });
    const releaseProjectWithSource = reviseMeshy2AuroraProjectV1(
      releaseProjectSource,
      {
        files: {
          ...releaseProjectSource.files,
          sourceGlb: projectFileReferenceV1({
            name: "source.glb",
            size: ownerBytes.byteLength,
            lastModified: Date.parse("2026-07-29T00:00:00.000Z"),
          }, OWNER_H1_SHA256),
        },
      },
      "2026-07-29T00:00:01.000Z",
    );
    const projectIdentity = projectBuildIdentityV1(
      reviseMeshy2AuroraProjectV1(
        releaseProjectWithSource,
        {
          animationMappingV2: animationMapping,
          animationStudio: document,
        },
        "2026-07-29T00:00:02.000Z",
      ),
    );

    const validation = await client.validateAnimationStudioDocument(
      ownerBytes.slice(0),
      documentJson,
      "e7-owner-donor-validate",
    );
    expect(validation).toMatchObject({
      ok: true,
      type: "ANIMATION_STUDIO_DOCUMENT_VALIDATED",
    });
    if (
      !validation.ok
      || validation.type !== "ANIMATION_STUDIO_DOCUMENT_VALIDATED"
    ) {
      throw new Error("The imported owner donor document was not validated.");
    }
    const validationResult = JSON.parse(validation.validationJson) as {
      status: string;
      sourceRevision: string;
      documentFingerprintSha256: string;
      diagnostics: unknown[];
    };
    expect(validationResult).toMatchObject({
      status: "READY",
      sourceRevision: OWNER_H1_SHA256,
      diagnostics: [],
    });
    expect(validationResult.documentFingerprintSha256)
      .toMatch(/^[0-9a-f]{64}$/);

    const preview = await client.previewAuthoredAnimationClip(
      documentJson,
      imported.id,
      "e7-owner-donor-preview",
    );
    expect(preview).toMatchObject({
      ok: true,
      type: "AUTHORED_ANIMATION_CLIP_PREVIEWED",
    });

    const materialized = await client.materializeAnimationStudioDocument(
      ownerBytes.slice(0),
      documentJson,
      "e7-owner-donor-materialize",
    );
    expect(materialized).toMatchObject({
      ok: true,
      type: "ANIMATION_STUDIO_DOCUMENT_MATERIALIZED",
    });
    if (
      !materialized.ok
      || materialized.type !== "ANIMATION_STUDIO_DOCUMENT_MATERIALIZED"
    ) {
      throw new Error("The imported owner donor document was not materialized.");
    }
    expect(JSON.parse(materialized.materializationJson)).toMatchObject({
      clips: [expect.objectContaining({
        name: imported.name,
        animationRoot: imported.animationRoot,
      })],
    });
    const materializationBytes = utf8Bytes(materialized.materializationJson);
    const materializationSha256 = await sha256(materializationBytes);
    const repeatedMaterialized =
      await client.materializeAnimationStudioDocument(
        ownerBytes.slice(0),
        documentJson,
        "e7-owner-donor-materialize-repeat",
      );
    expect(repeatedMaterialized).toMatchObject({
      ok: true,
      type: "ANIMATION_STUDIO_DOCUMENT_MATERIALIZED",
    });
    if (
      !repeatedMaterialized.ok
      || repeatedMaterialized.type
        !== "ANIMATION_STUDIO_DOCUMENT_MATERIALIZED"
    ) {
      throw new Error(
        "The repeated imported owner donor document was not materialized.",
      );
    }
    expect(repeatedMaterialized.materializationJson)
      .toBe(materialized.materializationJson);

    console.info("E7_OWNER_DONOR_E2E_V1", JSON.stringify({
      schemaVersion: 1,
      sourceAssetId: "h1-humanoid-1500",
      sourceSha256: OWNER_H1_SHA256,
      projectIdentity,
      animationStudioFingerprint:
        validationResult.documentFingerprintSha256,
      animationMappingFingerprint,
      animationMappingFingerprintKind:
        "SHA256_OF_CANONICAL_SERIALIZED_CREATURE_ANIMATION_AUTHORING_V2",
      donorDerivation: {
        kind: "FIXED_LENGTH_GLB_JSON_CLIP_LABEL_MUTATION",
        sourceClipName: SOURCE_CLIP_NAME,
        donorClipName: DONOR_CLIP_NAME,
        donorSha256,
      },
      importedClipId: imported.id,
      importedClipName: imported.name,
      importedTrackCount: imported.tracks.length,
      materializationArtifact: {
        fileName: "e7-owner-h1-donor-animation-materialization.json",
        byteLength: materializationBytes.byteLength,
        sha256: materializationSha256,
      },
      repeatMaterializationMatched: true,
      status: "READY_MATERIALIZED_NO_PACKAGE",
      newModelIterationCreated: false,
      v5ProofArtifactsFrozen: false,
      modelVisibility: "not_tested",
      proofCompleteness: "missing",
    }));
  }, 30_000);
});
