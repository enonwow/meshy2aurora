import { describe, expect, it } from "vitest";
import { createInitialStudioSession } from "../../app/studioSession";
import { FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 } from "../source/directCreatureAnimationProfile";
import type { BinaryMdlInspectionReport } from "../preview/types";
import { createCreatureAnimationAuthoringV1 } from "./state";
import {
  assertCreatureAnimationBuildInputCurrentV1,
  createCreatureAnimationBuildInputV1,
  reconcileAuthoredAndBuiltAnimationsV1,
} from "./buildIntegration";

function readySession() {
  const authoring = createCreatureAnimationAuthoringV1("a".repeat(64), "S");
  authoring.assignments = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((targetSlot) => ({
    targetSlot,
    sourceKind: "SOURCE_CLIP",
    sourceClipName: targetSlot,
    customAnimationId: null,
    provenance: {
      provider: "SOURCE_GLB",
      assetId: "source",
      ownership: "USER_OWNED",
    },
  }));
  return {
    ...createInitialStudioSession(2),
    source: {
      file: {} as File,
      name: "source.glb",
      size: 1,
      type: "model/gltf-binary",
      lastModified: 1,
      sha256: "a".repeat(64),
      parse: { kind: "VALID" as const },
    },
    animationMapping: { revision: 2, value: authoring },
    animationMappingValidation: {
      revision: 2,
      value: {
        authoringRevision: authoring.authoringRevision,
        authoringFingerprintSha256: "f".repeat(64),
        status: "READY" as const,
        diagnostics: [],
      },
    },
  };
}

describe("creature animation build integration", () => {
  it("freezes and rejects stale build input", () => {
    const session = readySession();
    const input = createCreatureAnimationBuildInputV1(session);
    expect(JSON.parse(input.animationAuthoringJson).assignments).toHaveLength(42);
    expect(() => assertCreatureAnimationBuildInputCurrentV1(session, input)).not.toThrow();
    expect(() => assertCreatureAnimationBuildInputCurrentV1({
      ...session,
      revision: 3,
    }, input)).toThrow(/required|stale/);
  });

  it("reconciles authored slot names against canonical readback", () => {
    const authoring = readySession().animationMapping.value;
    const readback: BinaryMdlInspectionReport = {
      schemaVersion: 1,
      format: "NWN1_BINARY_MDL",
      nodeTree: { roots: [] },
      animations: FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((name, index) => ({
        offset: index + 1,
        name,
        length: 1,
        transition: 0,
        animationRoot: "root",
        events: [],
        nodeTree: { roots: [] },
      })),
      diagnostics: [],
    };
    expect(reconcileAuthoredAndBuiltAnimationsV1(authoring, readback)).toMatchObject({
      matches: true,
      missingClipNames: [],
      unexpectedClipNames: [],
    });
    readback.animations[0].name = "unexpected";
    expect(reconcileAuthoredAndBuiltAnimationsV1(authoring, readback)).toMatchObject({
      matches: false,
      missingClipNames: ["ca1slashl"],
      unexpectedClipNames: ["unexpected"],
    });
  });
});
