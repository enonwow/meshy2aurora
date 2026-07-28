// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import {
  createInitialStudioSession,
  studioSessionReducer,
} from "./studioSession";
import {
  canContinueFromAnimationMapping,
  canContinueToAnimationMapping,
} from "./studioSelectors";
import { createCreatureAnimationAuthoringV1 } from "../features/animation-mapping/state";

const source = () => new File(["glb"], "hero.glb", { type: "model/gltf-binary" });
const appearance = () => new File(["2DA V2.0"], "appearance.2da", { type: "text/plain" });

function inspectedCreatureSession() {
  let state = createInitialStudioSession();
  state = studioSessionReducer(state, { type: "SOURCE_SELECTED", file: source() });
  state = studioSessionReducer(state, { type: "APPEARANCE_SELECTED", file: appearance() });
  state = studioSessionReducer(state, { type: "CONTINUE_TO_INSPECT" });
  state = studioSessionReducer(state, {
    type: "SOURCE_INSPECTION_SUCCEEDED",
    revision: state.revision,
    sha256: "a".repeat(64),
    inspection: { clips: 3 },
  });
  state = studioSessionReducer(state, {
    type: "APPEARANCE_INSPECTION_SUCCEEDED",
    revision: state.revision,
    sha256: "b".repeat(64),
    inspection: { modelType: "S" },
  });
  return state;
}

describe("Studio creature animation mapping session", () => {
  it("initializes only from current creature inspections and source hash", () => {
    const inspected = inspectedCreatureSession();
    expect(canContinueToAnimationMapping(inspected)).toBe(true);
    const authoring = createCreatureAnimationAuthoringV1("a".repeat(64), "S");
    const initialized = studioSessionReducer(inspected, {
      type: "ANIMATION_MAPPING_INITIALIZED",
      revision: inspected.revision,
      authoring,
    });
    expect(initialized.animationMapping).toEqual({
      revision: inspected.revision,
      value: authoring,
    });

    expect(studioSessionReducer(inspected, {
      type: "ANIMATION_MAPPING_INITIALIZED",
      revision: inspected.revision - 1,
      authoring,
    })).toBe(inspected);
  });

  it("invalidates the mapping after a source change", () => {
    const inspected = inspectedCreatureSession();
    const initialized = studioSessionReducer(inspected, {
      type: "ANIMATION_MAPPING_INITIALIZED",
      revision: inspected.revision,
      authoring: createCreatureAnimationAuthoringV1("a".repeat(64), "S"),
    });
    const changed = studioSessionReducer(initialized, {
      type: "SOURCE_SELECTED",
      file: new File(["new"], "replacement.glb"),
    });
    expect(changed.animationMapping).toBeNull();
    expect(changed.animationMappingValidation).toBeNull();
  });

  it("applies authoring events, invalidates build evidence and accepts only current validation", () => {
    const inspected = inspectedCreatureSession();
    let state = studioSessionReducer(inspected, {
      type: "ANIMATION_MAPPING_INITIALIZED",
      revision: inspected.revision,
      authoring: createCreatureAnimationAuthoringV1("a".repeat(64), "S"),
    });
    state = {
      ...state,
      result: { revision: state.revision, value: { artifact: "old" } },
      build: {
        kind: "SUCCEEDED",
        requestId: "old",
        revision: state.revision,
        result: { revision: state.revision, value: { artifact: "old" } },
      },
    };
    state = studioSessionReducer(state, {
      type: "ANIMATION_SOURCE_ASSIGNED",
      assignment: {
        targetSlot: "cwalk",
        sourceKind: "SOURCE_CLIP",
        sourceClipName: "Walk",
        customAnimationId: null,
        provenance: {
          provider: "SOURCE_GLB",
          assetId: "a".repeat(64),
          ownership: "USER_OWNED",
        },
      },
    });
    expect(state.animationMapping?.value.authoringRevision).toBe(2);
    expect(state.result).toBeNull();
    expect(state.build).toEqual({ kind: "IDLE" });

    const stale = studioSessionReducer(state, {
      type: "ANIMATION_MAPPING_VALIDATED",
      revision: state.revision,
      authoringRevision: 1,
      status: "READY",
      diagnostics: [],
    });
    expect(stale).toBe(state);

    const validated = studioSessionReducer(state, {
      type: "ANIMATION_MAPPING_VALIDATED",
      revision: state.revision,
      authoringRevision: 2,
      authoringFingerprintSha256: "f".repeat(64),
      status: "READY",
      diagnostics: [],
    });
    expect(canContinueFromAnimationMapping(validated)).toBe(true);

    const withoutCanonicalFingerprint = studioSessionReducer(state, {
      type: "ANIMATION_MAPPING_VALIDATED",
      revision: state.revision,
      authoringRevision: 2,
      status: "READY",
      diagnostics: [],
    });
    expect(canContinueFromAnimationMapping(withoutCanonicalFingerprint)).toBe(false);
  });
});
