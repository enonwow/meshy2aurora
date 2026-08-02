import { afterEach, describe, expect, it } from "vitest";
import sourceUrl from "../.generated/owned-full42-package/generated/source.glb?url";
import librarySourceUrl from
  "../.generated/owned-library-humanoid-full42-package/generated/source.glb?url";
import appearanceUrl from "../fixtures/appearance.2da?url";
import animationPresetManifestJson from
  "../../../../animation-library/presets/m2a_right_cross/manifest.json?raw";
import animationPresetPayloadJson from
  "../../../../animation-library/presets/m2a_right_cross/animation.json?raw";
import animationCatalogJson from
  "../../../../contracts/community-animation-catalog-v1.json?raw";
import {
  createProceduralTemplateClipV1,
} from "../../src/features/animation-editor/editing";
import type {
  AnimationStudioDocumentV1,
  AuthoredAnimationClipV1,
  CreatureAnimationAuthoringV2,
} from "../../src/features/animation-studio/types";
import {
  fingerprintAnimationStudioDocumentV1,
  migrateAnimationStudioDocumentV1ToV2,
  serializeAnimationStudioDocumentV1,
} from "../../src/features/animation-studio/schema";
import { serializeCreatureAnimationAuthoringV2 } from
  "../../src/features/animation-studio/mappingV2Persistence";
import { projectCanonicalResult } from
  "../../src/features/results/projectCanonicalResult";
import { projectCanonicalReadback } from
  "../../src/features/results/projectReadback";
import { FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 } from
  "../../src/features/source/directCreatureAnimationProfile";
import { StudioWorkerClient } from "../../src/worker/client";

const clients: StudioWorkerClient[] = [];

async function fetchBytes(url: string): Promise<ArrayBuffer> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`fixture fetch failed: ${response.status} ${url}`);
  }
  return response.arrayBuffer();
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

describe("Animation Studio V5 through the real Worker and WASM", () => {
  it("validates, previews, builds, binary-readbacks and repeats exact authored output", async () => {
    const identityBytes = await fetchBytes(sourceUrl);
    const sourceRevision = await sha256(identityBytes);
    const client = new StudioWorkerClient();
    clients.push(client);
    await expect(client.request({
      requestId: "studio-v5-init",
      type: "INITIALIZE",
    })).resolves.toMatchObject({ ok: true, type: "INITIALIZED" });

    const inspected = await client.inspectEditableAnimationSource(
      await fetchBytes(sourceUrl),
      undefined,
      "studio-v5-inspect",
    );
    expect(inspected).toMatchObject({
      ok: true,
      type: "EDITABLE_ANIMATION_SOURCE_INSPECTED",
    });
    if (!inspected.ok || inspected.type !== "EDITABLE_ANIMATION_SOURCE_INSPECTED") {
      throw new Error("real Worker did not inspect the editable output rig");
    }
    const inspection = JSON.parse(inspected.inspectionJson) as {
      sourceRevision: string;
      rig: Array<{
        id: number;
        name: string;
        parentId: number | null;
        translation: [number, number, number];
        rotation: [number, number, number, number];
      }>;
    };
    expect(inspection.sourceRevision).toBe(sourceRevision);
    expect(inspection.rig.length).toBeGreaterThan(0);
    const animationRoot = inspection.rig.find(({ parentId }) => parentId === null)
      ?.name ?? inspection.rig[0]!.name;
    const proceduralClip = createProceduralTemplateClipV1({
        id: "authored-stable-id",
        name: "m2a_editpulse",
        sourceRevision,
        animationRoot,
        lengthSeconds: 1,
        rig: inspection.rig,
      }, "ROOT_TRANSLATION_PULSE");
    const clip = {
      ...proceduralClip,
      status: "VALID" as const,
      events: [{ id: "event-impact", timeSeconds: 1 / 30, name: "impact" }],
      tracks: proceduralClip.tracks.map((track) => (
        track.path === "ROTATION"
          ? {
              ...track,
              keyframes: track.keyframes.map((keyframe) => ({
                ...keyframe,
                value: [0, 0, Math.SQRT1_2, Math.SQRT1_2],
              })),
            }
          : track
      )),
      revision: 2,
    };
    const studio: AnimationStudioDocumentV1 = {
      schemaVersion: 1,
      sourceRevision,
      authoringRevision: 2,
      status: "VALID",
      authoredClips: [clip],
    };
    const authoring: CreatureAnimationAuthoringV2 = {
      schemaVersion: 2,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      modelType: "S",
      sourceRevision,
      authoringRevision: 2,
      assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((targetSlot) => (
        targetSlot === "cpause1"
          ? {
              targetSlot,
              sourceKind: "CUSTOM",
              sourceClipName: null,
              customAnimationId: "custom-stable-id",
              provenance: {
                provider: "USER_CUSTOM",
                assetId: clip.id,
                ownership: "USER_OWNED",
              },
            } as const
          : {
              targetSlot,
              sourceKind: "SOURCE_CLIP",
              sourceClipName: targetSlot,
              customAnimationId: null,
              provenance: {
                provider: "SOURCE_GLB",
                assetId: sourceRevision,
                ownership: "USER_OWNED",
              },
            } as const
      )),
      fallbacks: [],
      customAnimations: [{
        id: "custom-stable-id",
        name: "custom_pulse",
        playback: "ONE_SHOT",
        clipReference: {
          sourceKind: "AUTHORED_CLIP",
          sourceClipName: null,
          authoredClipId: clip.id,
        },
        phases: [],
        provenance: {
          provider: "USER_CUSTOM",
          assetId: clip.id,
          ownership: "USER_OWNED",
        },
      }],
    };
    const studioJson = serializeAnimationStudioDocumentV1(studio);
    const studioFingerprint = await fingerprintAnimationStudioDocumentV1(studio);
    const authoringJson = serializeCreatureAnimationAuthoringV2(authoring);

    const hugeValueValidation = await client.validateAnimationStudioDocument(
      await fetchBytes(sourceUrl),
      JSON.stringify({
        ...studio,
        authoredClips: [{
          ...clip,
          tracks: clip.tracks.map((track, index) => (
            index === 0
              ? {
                  ...track,
                  keyframes: track.keyframes.map((keyframe, keyIndex) => (
                    keyIndex === 0
                      ? { ...keyframe, value: [1e20, 0, 0] }
                      : keyframe
                  )),
                }
              : track
          )),
        }],
      }),
      "studio-v5-huge-value",
    );
    expect(hugeValueValidation).toMatchObject({
      ok: true,
      type: "ANIMATION_STUDIO_DOCUMENT_VALIDATED",
    });
    if (
      !hugeValueValidation.ok
      || hugeValueValidation.type !== "ANIMATION_STUDIO_DOCUMENT_VALIDATED"
    ) {
      throw new Error("real Worker did not enforce the float product range");
    }
    expect(JSON.parse(hugeValueValidation.validationJson)).toMatchObject({
      status: "BLOCKED",
      diagnostics: expect.arrayContaining([expect.objectContaining({
        code: "M2A-ANIMATION-EDIT-VALUE-NONFINITE",
      })]),
    });

    const foreignBoneValidation = await client.validateAnimationStudioDocument(
      await fetchBytes(sourceUrl),
      serializeAnimationStudioDocumentV1({
        ...studio,
        authoredClips: [{
          ...clip,
          tracks: clip.tracks.map((track, index) => (
            index === 0 ? { ...track, targetNodeId: 2_147_483_647 } : track
          )),
        }],
      }),
      "studio-v5-foreign-bone",
    );
    expect(foreignBoneValidation).toMatchObject({
      ok: true,
      type: "ANIMATION_STUDIO_DOCUMENT_VALIDATED",
    });
    if (
      !foreignBoneValidation.ok
      || foreignBoneValidation.type !== "ANIMATION_STUDIO_DOCUMENT_VALIDATED"
    ) {
      throw new Error("real Worker did not return exact rig diagnostics");
    }
    expect(JSON.parse(foreignBoneValidation.validationJson)).toMatchObject({
      schemaVersion: 1,
      status: "BLOCKED",
      diagnostics: expect.arrayContaining([expect.objectContaining({
        code: "M2A-ANIMATION-EDIT-BONE-MISSING",
      })]),
    });

    const validated = await client.validateAnimationStudioDocument(
      await fetchBytes(sourceUrl),
      studioJson,
      "studio-v5-validate",
    );
    expect(validated).toMatchObject({
      ok: true,
      type: "ANIMATION_STUDIO_DOCUMENT_VALIDATED",
    });
    if (!validated.ok || validated.type !== "ANIMATION_STUDIO_DOCUMENT_VALIDATED") {
      throw new Error("real Worker did not validate the Studio document");
    }
    expect(JSON.parse(validated.validationJson)).toMatchObject({
      schemaVersion: 1,
      status: "READY",
      sourceRevision,
      documentFingerprintSha256: studioFingerprint,
      diagnostics: [],
    });

    const materialized = await client.materializeAnimationStudioDocument(
      await fetchBytes(sourceUrl),
      studioJson,
      "studio-v5-materialize",
    );
    expect(materialized).toMatchObject({
      ok: true,
      type: "ANIMATION_STUDIO_DOCUMENT_MATERIALIZED",
    });
    if (
      !materialized.ok
      || materialized.type !== "ANIMATION_STUDIO_DOCUMENT_MATERIALIZED"
    ) {
      throw new Error("real Worker did not materialize the Studio document");
    }
    const materializedJson = JSON.parse(materialized.materializationJson) as {
      clips: Array<{
        name: string;
        tracks: Array<{
          path: string;
          timesSeconds: number[];
          values: number[][];
        }>;
      }>;
    };
    expect(materializedJson.clips[0]?.name).toBe("m2a_editpulse");
    expect(materializedJson.clips[0]?.tracks).toEqual(expect.arrayContaining([
      expect.objectContaining({
        path: "TRANSLATION",
        timesSeconds: [0, 0.5, 1],
      }),
    ]));

    const previewed = await client.previewAuthoredAnimationClip(
      studioJson,
      clip.id,
      "studio-v5-preview",
    );
    expect(previewed).toMatchObject({
      ok: true,
      type: "AUTHORED_ANIMATION_CLIP_PREVIEWED",
    });
    if (!previewed.ok || previewed.type !== "AUTHORED_ANIMATION_CLIP_PREVIEWED") {
      throw new Error("real Worker did not preview the authored clip");
    }
    expect(JSON.parse(previewed.previewJson)).toMatchObject({
      schemaVersion: 1,
      clip: {
        name: "m2a_editpulse",
        events: [{ timeSeconds: Math.fround(1 / 30), name: "impact" }],
      },
    });

    const sourceGlb = await fetchBytes(sourceUrl);
    const appearanceTwoDa = await fetchBytes(appearanceUrl);
    const projectIdentity = {
      schemaVersion: 1,
      projectId: "browser-edited-project",
      projectName: "Browser edited project",
      projectRevision: 3,
    } as const;
    const built = await client.buildEditedCreatureModelPackage(
      sourceGlb,
      appearanceTwoDa,
      authoringJson,
      studioJson,
      JSON.stringify(projectIdentity),
      undefined,
      "studio-v5-build",
    );
    expect(sourceGlb.byteLength).toBe(0);
    expect(appearanceTwoDa.byteLength).toBe(0);
    expect(built).toMatchObject({ ok: true, type: "MODEL_PACKAGE_BUILT" });
    if (!built.ok || built.type !== "MODEL_PACKAGE_BUILT") {
      throw new Error("real Worker did not build the edited V5 package");
    }
    const canonical = projectCanonicalResult(
      built.reportJson,
      built.summaryJson,
      built.manifestJson,
      built.artifacts,
    );
    expect(canonical.projectIdentity).toEqual(projectIdentity);
    expect(canonical.animationStudioEvidence).toMatchObject({
      animationStudioRevision: 2,
      authoredClipIds: ["authored-stable-id"],
      authoredClipOutputNames: ["m2a_editpulse"],
      authoredEventCount: 1,
      customAssignmentCount: 1,
      sourceRevision,
      animationStudioFingerprintSha256: studioFingerprint,
      readbackStatus: "MATCH",
      sourceGlbUnchanged: true,
      animationStudioReadback: { status: "MATCH", diagnostics: [] },
    });
    expect(canonical.animationStudioEvidence?.authoredClips[0]?.usages)
      .toEqual(expect.arrayContaining([
        expect.objectContaining({
          authoredClipId: "authored-stable-id",
          outputClipName: "cpause1",
          usageKind: "BASE_SLOT",
        }),
        expect.objectContaining({
          authoredClipId: "authored-stable-id",
          outputClipName: "custom_pulse",
          usageKind: "CUSTOM_ONE_SHOT",
        }),
      ]));
    const readback = projectCanonicalReadback(built.readbackJson);
    expect(readback.animations.map(({ name }) => name)).toEqual(
      expect.arrayContaining(["cpause1", "custom_pulse"]),
    );
    expect(readback.animations.find(({ name }) => name === "custom_pulse"))
      .toMatchObject({
        events: [{ time: 0.033333335, name: "impact" }],
      });

    const repeated = await client.buildEditedCreatureModelPackage(
      await fetchBytes(sourceUrl),
      await fetchBytes(appearanceUrl),
      authoringJson,
      studioJson,
      JSON.stringify(projectIdentity),
      undefined,
      "studio-v5-build-repeat",
    );
    expect(repeated).toMatchObject({ ok: true, type: "MODEL_PACKAGE_BUILT" });
    if (!repeated.ok || repeated.type !== "MODEL_PACKAGE_BUILT") {
      throw new Error("real Worker did not repeat the edited V5 package");
    }
    expect(repeated.artifacts.map(({ fileName, sha256 }) => [fileName, sha256]))
      .toEqual(built.artifacts.map(({ fileName, sha256 }) => [fileName, sha256]));
  }, 120_000);

  it("instantiates a portable preset and preserves its exact motion through binary MDL readback", async () => {
    const manifest = JSON.parse(animationPresetManifestJson) as {
      motionSha256: string;
      presetId: string;
      rigProfile: string;
      rigSignatureSha256: string;
    };
    const payload = JSON.parse(animationPresetPayloadJson) as {
      tracks: Array<{
        targetBoneName: string;
        path: "TRANSLATION" | "ROTATION";
        keyframes: Array<{ timeSeconds: number; value: number[] }>;
      }>;
    };
    const catalogSha256 = (JSON.parse(animationCatalogJson) as {
      catalogSha256: string;
    }).catalogSha256;
    const sourceRevision = await sha256(await fetchBytes(librarySourceUrl));
    const client = new StudioWorkerClient();
    clients.push(client);
    await client.request({ requestId: "library-init", type: "INITIALIZE" });

    const validated = await client.validateAnimationPreset(
      animationPresetManifestJson,
      animationPresetPayloadJson,
      catalogSha256,
      "library-validate",
    );
    expect(validated).toMatchObject({ ok: true, type: "ANIMATION_PRESET_VALIDATED" });
    if (!validated.ok || validated.type !== "ANIMATION_PRESET_VALIDATED") {
      throw new Error("real Worker did not validate the portable preset");
    }
    expect(JSON.parse(validated.validationJson)).toMatchObject({
      schemaVersion: 1,
      status: "READY",
      diagnostics: [],
    });

    const compatibility = await client.inspectAnimationPresetCompatibility(
      animationPresetManifestJson,
      animationPresetPayloadJson,
      catalogSha256,
      await fetchBytes(librarySourceUrl),
      "library-compatibility",
    );
    expect(compatibility).toMatchObject({
      ok: true,
      type: "ANIMATION_PRESET_COMPATIBILITY_INSPECTED",
    });
    if (
      !compatibility.ok
      || compatibility.type !== "ANIMATION_PRESET_COMPATIBILITY_INSPECTED"
    ) {
      throw new Error("real Worker did not inspect preset compatibility");
    }
    expect(JSON.parse(compatibility.compatibilityJson)).toMatchObject({
      status: "COMPATIBLE",
      actualRigSignatureSha256: manifest.rigSignatureSha256,
      diagnostics: [],
    });

    const instantiated = await client.instantiateAnimationPreset(
      animationPresetManifestJson,
      animationPresetPayloadJson,
      catalogSha256,
      await fetchBytes(librarySourceUrl),
      "authored-library-right-cross",
      "m2a_rightcross",
      "library-instantiate",
    );
    expect(instantiated).toMatchObject({ ok: true, type: "ANIMATION_PRESET_INSTANTIATED" });
    if (!instantiated.ok || instantiated.type !== "ANIMATION_PRESET_INSTANTIATED") {
      throw new Error("real Worker did not instantiate the portable preset");
    }
    const instance = JSON.parse(instantiated.instantiationJson) as {
      status: "READY" | "BLOCKED";
      clip: AuthoredAnimationClipV1 | null;
      diagnostics: unknown[];
    };
    expect(instance).toMatchObject({ status: "READY", diagnostics: [] });
    if (!instance.clip) throw new Error("preset instantiation returned no clip");
    const clip: AuthoredAnimationClipV1 = { ...instance.clip, status: "VALID" };
    expect(clip.source).toMatchObject({
      kind: "LIBRARY_PRESET_COPY",
      sourceRevision,
      sourceClipFingerprint: manifest.motionSha256,
      libraryPreset: {
        presetId: manifest.presetId,
        catalogSha256,
        instantiationMode: "STRICT_RIG_V1",
      },
    });

    const exported = await client.exportAnimationContribution(
      JSON.stringify(clip),
      await fetchBytes(librarySourceUrl),
      JSON.stringify({
        presetId: "m2a_right_cross_export",
        presetVersion: 1,
        outputName: "m2a_rcexport",
        label: "Right cross export",
        summary: "Portable browser Worker export of the tracked right cross.",
        authors: [{ name: "Meshy2Aurora tests" }],
        license: "LicenseRef-Meshy2Aurora-Project-Generated",
        tags: ["attack", "boxing", "humanoid", "one-shot", "right-hand", "unarmed", "upper-body"],
        playback: "ONE_SHOT",
        rigProfile: manifest.rigProfile,
        validationStatus: "PIPELINE_VERIFIED",
      }),
      "library-export",
    );
    expect(exported).toMatchObject({ ok: true, type: "ANIMATION_CONTRIBUTION_EXPORTED" });
    if (!exported.ok || exported.type !== "ANIMATION_CONTRIBUTION_EXPORTED") {
      throw new Error("real Worker did not export the contribution");
    }
    const contribution = JSON.parse(exported.contributionJson);
    expect(contribution).toMatchObject({ status: "READY", diagnostics: [] });
    expect(exported.contributionJson).not.toContain("sourceRevision");
    expect(exported.contributionJson.toLowerCase()).not.toContain(".glb");

    const studio: AnimationStudioDocumentV1 = {
      ...migrateAnimationStudioDocumentV1ToV2({
        schemaVersion: 1,
        sourceRevision,
        authoringRevision: 1,
        status: "DRAFT",
        authoredClips: [],
      }),
      sourceRevision,
      authoringRevision: 1,
      status: "VALID",
      authoredClips: [clip],
    };
    const authoring: CreatureAnimationAuthoringV2 = {
      schemaVersion: 2,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      modelType: "S",
      sourceRevision,
      authoringRevision: 1,
      assignments: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((targetSlot) => (
        targetSlot === "cpause1"
          ? {
              targetSlot,
              sourceKind: "CUSTOM",
              sourceClipName: null,
              customAnimationId: "custom-library-right-cross",
              provenance: {
                provider: "USER_CUSTOM",
                assetId: clip.id,
                ownership: "USER_OWNED",
              },
            } as const
          : {
              targetSlot,
              sourceKind: "SOURCE_CLIP",
              sourceClipName: targetSlot,
              customAnimationId: null,
              provenance: {
                provider: "SOURCE_GLB",
                assetId: sourceRevision,
                ownership: "USER_OWNED",
              },
            } as const
      )),
      fallbacks: [],
      customAnimations: [{
        id: "custom-library-right-cross",
        name: "lib_rightcross",
        playback: "ONE_SHOT",
        clipReference: {
          sourceKind: "AUTHORED_CLIP",
          sourceClipName: null,
          authoredClipId: clip.id,
        },
        phases: [],
        provenance: {
          provider: "USER_CUSTOM",
          assetId: clip.id,
          ownership: "USER_OWNED",
        },
      }],
    };
    const built = await client.buildEditedCreatureModelPackage(
      await fetchBytes(librarySourceUrl),
      await fetchBytes(appearanceUrl),
      serializeCreatureAnimationAuthoringV2(authoring),
      serializeAnimationStudioDocumentV1(studio),
      JSON.stringify({
        schemaVersion: 1,
        projectId: "browser-library-project",
        projectName: "Browser library project",
        projectRevision: 1,
      }),
      undefined,
      "library-build",
    );
    expect(built).toMatchObject({ ok: true, type: "MODEL_PACKAGE_BUILT" });
    if (!built.ok || built.type !== "MODEL_PACKAGE_BUILT") {
      throw new Error("real Worker did not build the library-derived package");
    }
    const canonical = projectCanonicalResult(
      built.reportJson,
      built.summaryJson,
      built.manifestJson,
      built.artifacts,
    );
    expect(canonical.animationStudioEvidence).toMatchObject({
      readbackStatus: "MATCH",
      sourceGlbUnchanged: true,
      authoredClipIds: [clip.id],
      authoredClipOutputNames: [clip.name],
      animationStudioReadback: { status: "MATCH", diagnostics: [] },
    });
    const readback = projectCanonicalReadback(built.readbackJson);
    const rightCross = readback.animations.find(({ name }) => name === "lib_rightcross");
    expect(rightCross).toBeDefined();
    const position = flattenReadbackNodes(rightCross?.nodeTree.roots ?? [])
      .flatMap(({ controllers }) => controllers)
      .find(({ controllerName }) => controllerName === "position");
    const expected = payload.tracks.find(({ targetBoneName, path }) => (
      targetBoneName === "Hips" && path === "TRANSLATION"
    ));
    expect(position).toBeDefined();
    expect(expected).toBeDefined();
    expect(position?.times).toHaveLength(expected?.keyframes.length ?? 0);
    expect(position?.values).toHaveLength(expected?.keyframes.length ?? 0);
    expected?.keyframes.forEach((keyframe, index) => {
      expect(position?.times[index]).toBeCloseTo(keyframe.timeSeconds, 5);
      keyframe.value.forEach((value, axis) => {
        expect(position?.values[index]?.[axis]).toBeCloseTo(value, 5);
      });
    });
  }, 120_000);

  it("returns stable schema diagnostics and supersedes an older preview", async () => {
    const client = new StudioWorkerClient();
    clients.push(client);
    await client.request({ requestId: "studio-errors-init", type: "INITIALIZE" });

    const malformed = await client.validateAnimationStudioDocument(
      await fetchBytes(sourceUrl),
      "{",
      "studio-malformed",
    );
    expect(malformed).toMatchObject({
      ok: true,
      type: "ANIMATION_STUDIO_DOCUMENT_VALIDATED",
    });
    if (!malformed.ok || malformed.type !== "ANIMATION_STUDIO_DOCUMENT_VALIDATED") {
      throw new Error("real Worker did not return schema diagnostics");
    }
    expect(JSON.parse(malformed.validationJson)).toMatchObject({
      schemaVersion: 1,
      status: "BLOCKED",
      diagnostics: [{
        code: "M2A-ANIMATION-EDIT-SCHEMA",
        path: "animationStudioDocumentJson",
      }],
    });

    const first = client.previewAuthoredAnimationClip(
      "{}",
      "older",
      "studio-preview-older",
    );
    const newer = client.previewAuthoredAnimationClip(
      "{}",
      "newer",
      "studio-preview-newer",
    );
    await expect(first).rejects.toMatchObject({ name: "AbortError" });
    await expect(newer).rejects.toThrow();
  }, 30_000);
});

function flattenReadbackNodes<T extends { children: T[] }>(nodes: T[]): T[] {
  return nodes.flatMap((node) => [node, ...flattenReadbackNodes(node.children)]);
}
