// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import {
  createInitialStudioSession,
  studioSessionReducer,
  type StudioSessionState,
} from "./studioSession";

const source = () => new File(["glb"], "hero.glb", { type: "model/gltf-binary" });
const appearance = () => new File(["2DA V2.0"], "appearance.2da", { type: "text/plain" });

function readySourceState() {
  let state = createInitialStudioSession();
  state = studioSessionReducer(state, { type: "SOURCE_SELECTED", file: source() });
  return studioSessionReducer(state, { type: "APPEARANCE_SELECTED", file: appearance() });
}

describe("Studio session reducer", () => {
  it("starts at Source with only Source unlocked", () => {
    expect(createInitialStudioSession()).toMatchObject({
      revision: 0,
      currentStep: "SOURCE",
      lastAvailableStep: "SOURCE",
      source: null,
      appearance: null,
      sourceInspection: null,
      build: { kind: "IDLE" },
      result: null,
      download: { kind: "LOCKED" },
    });
  });

  it("stores both input kinds and increments revision for every selection", () => {
    const state = readySourceState();

    expect(state.revision).toBe(2);
    expect(state.source).toMatchObject({ name: "hero.glb", size: 3, sha256: null, parse: { kind: "NOT_STARTED" } });
    expect(state.appearance).toMatchObject({ name: "appearance.2da", sha256: null, parse: { kind: "NOT_STARTED" } });
    expect(state.lastAvailableStep).toBe("SOURCE");
  });

  it("continues to Inspect only when both inputs are selected", () => {
    const initial = createInitialStudioSession();
    const partial = studioSessionReducer(initial, { type: "SOURCE_SELECTED", file: source() });

    expect(studioSessionReducer(initial, { type: "CONTINUE_TO_INSPECT" })).toBe(initial);
    expect(studioSessionReducer(partial, { type: "CONTINUE_TO_INSPECT" })).toBe(partial);

    const ready = studioSessionReducer(partial, { type: "APPEARANCE_SELECTED", file: appearance() });
    expect(studioSessionReducer(ready, { type: "CONTINUE_TO_INSPECT" })).toMatchObject({
      currentStep: "INSPECT",
      lastAvailableStep: "INSPECT",
    });
  });

  it("ignores navigation to a locked step and permits navigation to an unlocked step", () => {
    const ready = readySourceState();
    const inspect = studioSessionReducer(ready, { type: "CONTINUE_TO_INSPECT" });

    expect(studioSessionReducer(inspect, { type: "NAVIGATE", step: "BUILD" })).toBe(inspect);
    expect(studioSessionReducer(inspect, { type: "NAVIGATE", step: "SOURCE" })).toMatchObject({
      currentStep: "SOURCE",
      lastAvailableStep: "INSPECT",
    });
  });

  it("does not relock previously unlocked steps when continuing after back navigation", () => {
    const ready = readySourceState();
    const visited: StudioSessionState = {
      ...ready,
      currentStep: "SOURCE",
      lastAvailableStep: "DOWNLOAD",
    };

    expect(studioSessionReducer(visited, { type: "CONTINUE_TO_INSPECT" })).toMatchObject({
      currentStep: "INSPECT",
      lastAvailableStep: "DOWNLOAD",
    });
  });

  it("invalidates every downstream value when an input is replaced", () => {
    const base = readySourceState();
    const populated: StudioSessionState<{ nodes: number }, { artifactIds: string[] }> = {
      ...base,
      currentStep: "DOWNLOAD",
      lastAvailableStep: "DOWNLOAD",
      sourceInspection: { revision: base.revision, value: { nodes: 4 } },
      build: {
        kind: "SUCCEEDED",
        requestId: "request-1",
        revision: base.revision,
        result: { revision: base.revision, value: { artifactIds: ["model-mdl"] } },
      },
      result: { revision: base.revision, value: { artifactIds: ["model-mdl"] } },
      download: { kind: "READY", revision: base.revision },
    };

    const next = studioSessionReducer(populated, {
      type: "SOURCE_SELECTED",
      file: new File(["new"], "replacement.glb"),
    });

    expect(next).toMatchObject({
      revision: base.revision + 1,
      currentStep: "SOURCE",
      lastAvailableStep: "SOURCE",
      sourceInspection: null,
      build: { kind: "IDLE" },
      result: null,
      download: { kind: "LOCKED" },
    });
    expect(next.source?.name).toBe("replacement.glb");
    expect(next.appearance?.name).toBe("appearance.2da");
  });

  it("invalidates downstream and increments revision when either input is removed", () => {
    const ready = readySourceState();
    const inspect = studioSessionReducer(ready, { type: "CONTINUE_TO_INSPECT" });
    const withoutAppearance = studioSessionReducer(inspect, { type: "APPEARANCE_REMOVED" });

    expect(withoutAppearance).toMatchObject({
      revision: ready.revision + 1,
      currentStep: "SOURCE",
      lastAvailableStep: "SOURCE",
      appearance: null,
    });

    const withoutSource = studioSessionReducer(withoutAppearance, { type: "SOURCE_REMOVED" });
    expect(withoutSource.revision).toBe(ready.revision + 2);
    expect(withoutSource.source).toBeNull();
  });

  it("ignores stale input metadata and keeps selection distinct from validation", () => {
    const selected = studioSessionReducer(createInitialStudioSession(), {
      type: "SOURCE_SELECTED",
      file: source(),
    });

    const stale = studioSessionReducer(selected, {
      type: "INPUT_METADATA_UPDATED",
      input: "SOURCE",
      revision: selected.revision - 1,
      sha256: "a".repeat(64),
      parse: { kind: "VALID" },
    });
    expect(stale).toBe(selected);

    const current = studioSessionReducer(selected, {
      type: "INPUT_METADATA_UPDATED",
      input: "SOURCE",
      revision: selected.revision,
      sha256: "a".repeat(64),
      parse: { kind: "VALID" },
    });
    expect(current.source).toMatchObject({ sha256: "a".repeat(64), parse: { kind: "VALID" } });
  });

  it("starts a clean conversion with a newer revision", () => {
    const ready = readySourceState();
    const next = studioSessionReducer(ready, { type: "START_NEW_CONVERSION" });

    expect(next).toMatchObject({
      revision: ready.revision + 1,
      currentStep: "SOURCE",
      lastAvailableStep: "SOURCE",
      source: null,
      appearance: null,
    });
  });
});
