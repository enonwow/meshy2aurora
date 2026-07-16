// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import {
  createInitialStudioSession,
  studioSessionReducer,
  type StudioSessionState,
} from "./studioSession";

const source = () => new File(["glb"], "hero.glb", { type: "model/gltf-binary" });
const appearance = () => new File(["2DA V2.0"], "appearance.2da", { type: "text/plain" });

function readySourceState<TResult = unknown>() {
  let state = createInitialStudioSession<unknown, TResult>();
  state = studioSessionReducer(state, { type: "SOURCE_SELECTED", file: source() });
  return studioSessionReducer(state, { type: "APPEARANCE_SELECTED", file: appearance() });
}

function readyBuildState<TResult = unknown>() {
  const selected = readySourceState<TResult>();
  const inspect = studioSessionReducer(selected, { type: "CONTINUE_TO_INSPECT" });
  const inspected: StudioSessionState<{ nodes: number }, TResult, { columns: number }> = {
    ...inspect,
    sourceInspection: { revision: inspect.revision, value: { nodes: 4 } },
    appearanceInspection: { revision: inspect.revision, value: { columns: 12 } },
  };
  return studioSessionReducer(inspected, { type: "CONTINUE_TO_BUILD" });
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
      appearanceInspection: null,
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

  it("continues to Build only with current source inspection evidence", () => {
    const ready = readySourceState();
    const inspect = studioSessionReducer(ready, { type: "CONTINUE_TO_INSPECT" });
    expect(studioSessionReducer(inspect, { type: "CONTINUE_TO_BUILD" })).toBe(inspect);

    const inspected: StudioSessionState<{ eligible: boolean }> = {
      ...inspect,
      sourceInspection: {
        revision: inspect.revision,
        value: { eligible: true },
      },
      appearanceInspection: {
        revision: inspect.revision,
        value: { columns: 12 },
      },
    };
    expect(studioSessionReducer(inspected, { type: "CONTINUE_TO_BUILD" })).toMatchObject({
      currentStep: "BUILD",
      lastAvailableStep: "BUILD",
    });
  });

  it("invalidates every downstream value when an input is replaced", () => {
    const base = readySourceState();
    const populated: StudioSessionState<{ nodes: number }, { artifactIds: string[] }> = {
      ...base,
      currentStep: "DOWNLOAD",
      lastAvailableStep: "DOWNLOAD",
      sourceInspection: { revision: base.revision, value: { nodes: 4 } },
      appearanceInspection: { revision: base.revision, value: { columns: 12 } },
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
      appearanceInspection: null,
      build: { kind: "IDLE" },
      result: null,
      download: { kind: "LOCKED" },
    });
    expect(next.source?.name).toBe("replacement.glb");
    expect(next.appearance?.name).toBe("appearance.2da");
  });

  it("starts Build only on the current revision with current inspection evidence", () => {
    const ready = readyBuildState<{ artifactIds: string[] }>();
    const stale = studioSessionReducer(ready, {
      type: "BUILD_STARTED",
      requestId: "build-stale",
      revision: ready.revision - 1,
    });
    expect(stale).toBe(ready);

    const running = studioSessionReducer(ready, {
      type: "BUILD_STARTED",
      requestId: "build-1",
      revision: ready.revision,
    });
    expect(running).toMatchObject({
      currentStep: "BUILD",
      lastAvailableStep: "BUILD",
      build: {
        kind: "RUNNING",
        requestId: "build-1",
        revision: ready.revision,
      },
      result: null,
      download: { kind: "LOCKED" },
    });

    expect(studioSessionReducer(running, {
      type: "BUILD_STARTED",
      requestId: "build-2",
      revision: ready.revision,
    })).toBe(running);
  });

  it("commits a matching Build success atomically and opens Review", () => {
    const ready = readyBuildState<{ artifactIds: string[] }>();
    const running = studioSessionReducer(ready, {
      type: "BUILD_STARTED",
      requestId: "build-1",
      revision: ready.revision,
    });
    const succeeded = studioSessionReducer(running, {
      type: "BUILD_SUCCEEDED",
      requestId: "build-1",
      revision: ready.revision,
      result: { artifactIds: ["model-mdl", "package-hak"] },
    });

    expect(succeeded).toMatchObject({
      currentStep: "REVIEW",
      lastAvailableStep: "REVIEW",
      build: {
        kind: "SUCCEEDED",
        requestId: "build-1",
        revision: ready.revision,
        result: {
          revision: ready.revision,
          value: { artifactIds: ["model-mdl", "package-hak"] },
        },
      },
      result: {
        revision: ready.revision,
        value: { artifactIds: ["model-mdl", "package-hak"] },
      },
      download: { kind: "LOCKED" },
    });
    expect(succeeded.build.kind).toBe("SUCCEEDED");
    if (succeeded.build.kind !== "SUCCEEDED") throw new Error("expected success");
    expect(succeeded.build.result).toBe(succeeded.result);
  });

  it("ignores stale Build success responses by request and revision", () => {
    const ready = readyBuildState<{ artifactIds: string[] }>();
    const running = studioSessionReducer(ready, {
      type: "BUILD_STARTED",
      requestId: "build-current",
      revision: ready.revision,
    });

    expect(studioSessionReducer(running, {
      type: "BUILD_SUCCEEDED",
      requestId: "build-old",
      revision: ready.revision,
      result: { artifactIds: ["stale"] },
    })).toBe(running);
    expect(studioSessionReducer(running, {
      type: "BUILD_SUCCEEDED",
      requestId: "build-current",
      revision: ready.revision - 1,
      result: { artifactIds: ["stale"] },
    })).toBe(running);
  });

  it("keeps a matching Build failure on Build and ignores stale failures", () => {
    const ready = readyBuildState();
    const running = studioSessionReducer(ready, {
      type: "BUILD_STARTED",
      requestId: "build-current",
      revision: ready.revision,
    });
    expect(studioSessionReducer(running, {
      type: "BUILD_FAILED",
      requestId: "build-old",
      revision: ready.revision,
      failure: { message: "stale" },
    })).toBe(running);

    const failed = studioSessionReducer(running, {
      type: "BUILD_FAILED",
      requestId: "build-current",
      revision: ready.revision,
      failure: { code: "M6-WRITER", message: "writer failed" },
    });
    expect(failed).toMatchObject({
      currentStep: "BUILD",
      lastAvailableStep: "BUILD",
      build: {
        kind: "FAILED",
        requestId: "build-current",
        revision: ready.revision,
        failure: { code: "M6-WRITER", message: "writer failed" },
      },
      result: null,
      download: { kind: "LOCKED" },
    });
  });

  it("cancels only the matching running Build", () => {
    const ready = readyBuildState();
    const running = studioSessionReducer(ready, {
      type: "BUILD_STARTED",
      requestId: "build-current",
      revision: ready.revision,
    });
    expect(studioSessionReducer(running, {
      type: "BUILD_CANCELLED",
      requestId: "build-old",
      revision: ready.revision,
    })).toBe(running);

    expect(studioSessionReducer(running, {
      type: "BUILD_CANCELLED",
      requestId: "build-current",
      revision: ready.revision,
    })).toMatchObject({
      currentStep: "BUILD",
      lastAvailableStep: "BUILD",
      build: { kind: "IDLE" },
      result: null,
      download: { kind: "LOCKED" },
    });
  });

  it("blocks all workflow navigation while Build is running", () => {
    const ready = readyBuildState();
    const running = studioSessionReducer(ready, {
      type: "BUILD_STARTED",
      requestId: "build-current",
      revision: ready.revision,
    });

    expect(studioSessionReducer(running, { type: "NAVIGATE", step: "SOURCE" })).toBe(running);
    expect(studioSessionReducer(running, { type: "NAVIGATE", step: "INSPECT" })).toBe(running);
    expect(studioSessionReducer(running, { type: "NAVIGATE", step: "BUILD" })).toBe(running);
  });

  it("invalidates a running Build and rejects its late response after input change", () => {
    const ready = readyBuildState<{ artifactIds: string[] }>();
    const running = studioSessionReducer(ready, {
      type: "BUILD_STARTED",
      requestId: "build-current",
      revision: ready.revision,
    });
    const invalidated = studioSessionReducer(running, {
      type: "APPEARANCE_SELECTED",
      file: new File(["replacement"], "appearance.2da"),
    });

    expect(invalidated).toMatchObject({
      revision: ready.revision + 1,
      currentStep: "SOURCE",
      lastAvailableStep: "SOURCE",
      sourceInspection: null,
      appearanceInspection: null,
      build: { kind: "IDLE" },
      result: null,
      download: { kind: "LOCKED" },
    });
    expect(studioSessionReducer(invalidated, {
      type: "BUILD_SUCCEEDED",
      requestId: "build-current",
      revision: ready.revision,
      result: { artifactIds: ["stale"] },
    })).toBe(invalidated);
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

  it("stores source inspection only for the current revision", () => {
    const selected = studioSessionReducer(createInitialStudioSession<{ nodes: number }>(), {
      type: "SOURCE_SELECTED",
      file: source(),
    });

    const stale = studioSessionReducer(selected, {
      type: "SOURCE_INSPECTION_SUCCEEDED",
      revision: selected.revision - 1,
      sha256: "a".repeat(64),
      inspection: { nodes: 4 },
    });
    expect(stale).toBe(selected);

    const current = studioSessionReducer(selected, {
      type: "SOURCE_INSPECTION_SUCCEEDED",
      revision: selected.revision,
      sha256: "b".repeat(64),
      inspection: { nodes: 4 },
    });
    expect(current.source).toMatchObject({
      sha256: "b".repeat(64),
      parse: { kind: "VALID" },
    });
    expect(current.sourceInspection).toEqual({
      revision: selected.revision,
      value: { nodes: 4 },
    });
  });

  it("stores appearance inspection only for the current revision", () => {
    const selected = studioSessionReducer(
      createInitialStudioSession<unknown, unknown, { columns: number }>(),
      { type: "APPEARANCE_SELECTED", file: appearance() },
    );

    const stale = studioSessionReducer(selected, {
      type: "APPEARANCE_INSPECTION_SUCCEEDED",
      revision: selected.revision - 1,
      sha256: "a".repeat(64),
      inspection: { columns: 12 },
    });
    expect(stale).toBe(selected);

    const current = studioSessionReducer(selected, {
      type: "APPEARANCE_INSPECTION_SUCCEEDED",
      revision: selected.revision,
      sha256: "b".repeat(64),
      inspection: { columns: 12 },
    });
    expect(current.appearance).toMatchObject({
      sha256: "b".repeat(64),
      parse: { kind: "VALID" },
    });
    expect(current.appearanceInspection).toEqual({
      revision: selected.revision,
      value: { columns: 12 },
    });
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
