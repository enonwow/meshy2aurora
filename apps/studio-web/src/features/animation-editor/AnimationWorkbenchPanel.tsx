import { useEffect, useMemo, useState } from "react";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import { createHumanoidSwordSlashFromSourceClipV1, sampleAnimationTrackLinearV1 } from "./editing";
import {
  rigWireV1,
  type AnimationPoseSnapshotV1,
  type AnimationWorkbenchProjectArtifactV1,
  type AnimationWorkbenchProjectStateV1,
  type AnimationSequenceBakeV1,
  type AnimationSequenceSegmentRecipeV1,
  type AnimationWorkbenchRequestV1,
  type AnimationWorkbenchResultV1,
  type CombatAttackPhaseTimelineV1,
  type CombatQualityReportV1,
  type MotionVisualizationV1,
} from "./animationWorkbench";

interface Props {
  readonly sourceRevision: string;
  readonly clip: AuthoredAnimationClipV1;
  readonly clips: readonly AuthoredAnimationClipV1[];
  readonly rig: readonly AnimationRigNodeV1[];
  readonly playheadSeconds: number;
  readonly onOperation: (request: AnimationWorkbenchRequestV1) => Promise<AnimationWorkbenchResultV1>;
  readonly onReplaceClip: (clip: AuthoredAnimationClipV1) => void;
  readonly onAddClip: (clip: AuthoredAnimationClipV1) => void;
  readonly onPreviewClip: (clip: AuthoredAnimationClipV1 | null) => void;
  readonly onVisualization: (value: MotionVisualizationV1 | null) => void;
  readonly projectState: AnimationWorkbenchProjectStateV1;
  readonly onProjectStateChange: (value: AnimationWorkbenchProjectStateV1) => void;
}

const PHASES = ["READY", "WIND_UP", "STRIKE", "IMPACT", "RECOVERY"] as const;
const ATTACK_PHASE_PRESETS = {
  CUSTOM: [0, 0.2, 0.45, 0.6, 1],
  PUNCH: [0, 0.28, 0.5, 0.62, 1],
  SLASH: [0, 0.3, 0.52, 0.66, 1],
  OVERHEAD: [0, 0.38, 0.58, 0.72, 1],
  THRUST: [0, 0.24, 0.48, 0.6, 1],
} as const;
const MOTION_VISUALIZATION_CACHE_V1 = new Map<string, MotionVisualizationV1>();

export function AnimationWorkbenchPanel({
  sourceRevision,
  clip,
  clips,
  rig,
  playheadSeconds,
  onOperation,
  onReplaceClip,
  onAddClip,
  onPreviewClip,
  onVisualization,
  projectState,
  onProjectStateChange,
}: Props) {
  const rigWire = useMemo(
    () => rigWireV1(sourceRevision, clip.animationRoot, rig),
    [clip.animationRoot, rig, sourceRevision],
  );
  const [selected, setSelected] = useState<readonly number[]>([]);
  const [pose, setPose] = useState<AnimationPoseSnapshotV1 | null>(null);
  const [phaseTimes, setPhaseTimes] = useState<readonly number[]>(() => phaseDefaults(clip.lengthSeconds));
  const [timeline, setTimeline] = useState<CombatAttackPhaseTimelineV1 | null>(null);
  const [quality, setQuality] = useState<CombatQualityReportV1 | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [speed, setSpeed] = useState(1);
  const [amplitude, setAmplitude] = useState(1);
  const [mirrorVariant, setMirrorVariant] = useState(false);
  const [variantTags, setVariantTags] = useState("combat, custom");
  const [segments, setSegments] = useState<readonly AnimationSequenceSegmentRecipeV1[]>(() => initialSegments(clips));
  const [sequenceBake, setSequenceBake] = useState<AnimationSequenceBakeV1 | null>(null);
  const [motionOverlaysEnabled, setMotionOverlaysEnabled] = useState(true);
  const [reducedMotion, setReducedMotion] = useState(() => globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false);

  useEffect(() => {
    setPhaseTimes(phaseDefaults(clip.lengthSeconds));
    setTimeline(null);
    setQuality(null);
  }, [clip.id, clip.lengthSeconds, clip.revision]);
  useEffect(() => {
    setSegments((current) => current.length > 0 ? current : initialSegments(clips));
  }, [clips]);

  const execute = async (label: string, request: AnimationWorkbenchRequestV1) => {
    setBusy(label);
    setError(null);
    try {
      return await onOperation(request);
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
      return null;
    } finally {
      setBusy(null);
    }
  };
  const persist = (
    key: "posePresets" | "phaseTimelines" | "sequences" | "variantRecipes",
    artifact: AnimationWorkbenchProjectArtifactV1,
  ) => onProjectStateChange({
    ...projectState,
    sourceRevision,
    [key]: [...projectState[key].filter(({ id }) => id !== artifact.id), artifact],
  });

  const capture = async () => {
    const result = await execute("pose", {
      operation: "CAPTURE_POSE", clip, rig: rigWire,
      timeSeconds: playheadSeconds, selectedNodeIds: selected,
    });
    if (result?.result === "POSE_CAPTURED") setPose(result.value);
  };
  const applyPose = async (candidate: AnimationPoseSnapshotV1, mode: "WHOLE_BODY" | "SELECTED_BONES") => {
    const result = await execute("pose", {
      operation: "APPLY_POSE", clip, rig: rigWire, pose: candidate,
      timeSeconds: playheadSeconds, mode, selectedNodeIds: selected,
    });
    if (result?.result === "POSE_APPLIED") onReplaceClip(result.value.clip);
  };
  const mirror = async () => {
    if (!pose) return;
    const { pairs, centers } = explicitMirrorPairs(rig);
    const mapResult = await execute("mirror", {
      operation: "BUILD_MIRROR_MAP", rig: rigWire, axis: "X", pairs,
      centerNodeIds: centers,
    });
    if (mapResult?.result !== "MIRROR_MAP_BUILT") return;
    const mirrored = await execute("mirror", {
      operation: "MIRROR_POSE", pose, rig: rigWire, mapping: mapResult.value,
    });
    if (mirrored?.result === "POSE_MIRRORED") {
      setPose(mirrored.value);
      await applyPose(mirrored.value, selected.length === 0 ? "WHOLE_BODY" : "SELECTED_BONES");
    }
  };
  const reset = async (nodeIds = selected) => {
    if (nodeIds.length === 0) return;
    const result = await execute("pose", {
      operation: "RESET_POSE", clip, rig: rigWire,
      timeSeconds: playheadSeconds, selectedNodeIds: nodeIds,
    });
    if (result?.result === "POSE_RESET") onReplaceClip(result.value.clip);
  };
  const createPhases = async () => {
    const result = await execute("phases", {
      operation: "CREATE_COMBAT_PHASES", clip,
      phaseTimes: phaseTimes as [number, number, number, number, number],
    });
    if (result?.result === "COMBAT_PHASES_CREATED") {
      setTimeline(result.value);
      persist("phaseTimelines", {
        id: `phases:${clip.id}`,
        name: `${clip.name} combat phases`,
        kind: "PHASE_TIMELINE",
        sourceRevision,
        fingerprintSha256: result.value.fingerprintSha256,
        payload: result.value as unknown as Record<string, unknown>,
      });
    }
  };
  const createSlashFromCurrentPose = () => {
    try {
      setError(null);
      const attack = createHumanoidSwordSlashFromSourceClipV1(clip, rig, playheadSeconds);
      onAddClip({
        ...attack,
        id: uniqueClipId(`${clip.id}-slash`, clips),
        name: `${clip.name} slash`,
        revision: 1,
      });
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  };
  const previewPhase = (index: number) => {
    const timeSeconds = phaseTimes[index] ?? 0;
    const previewLength = Math.min(0.6, Math.max(0.1, clip.lengthSeconds));
    onPreviewClip({
      ...clip,
      id: `${clip.id}-phase-${PHASES[index]?.toLowerCase() ?? index}`,
      name: `${clip.name} ${PHASES[index]?.toLowerCase() ?? "phase"} pose`,
      kind: "STATIC_POSE",
      lengthSeconds: previewLength,
      events: [],
      tracks: clip.tracks.map((track) => {
        const value = sampleAnimationTrackLinearV1(track, timeSeconds);
        return {
          ...track,
          keyframes: [
            { id: `${track.id}-phase-start`, timeSeconds: 0, value },
            { id: `${track.id}-phase-end`, timeSeconds: previewLength, value },
          ],
        };
      }),
      revision: 1,
    });
  };
  const analyze = async () => {
    let current = timeline;
    if (!current) {
      const created = await execute("quality", {
        operation: "CREATE_COMBAT_PHASES", clip,
        phaseTimes: phaseTimes as [number, number, number, number, number],
      });
      if (created?.result !== "COMBAT_PHASES_CREATED") return;
      current = created.value;
      setTimeline(current);
    }
    const result = await execute("quality", {
      operation: "ANALYZE_COMBAT_QUALITY", clip, rig: rigWire, timeline: current,
      policy: {
        schemaVersion: 1, sampleRateHz: 60, minimumPhaseSeconds: 0.02,
        minimumAnticipationPerHeight: 0.015, maximumImpactPeakOffsetSeconds: 0.12,
        maximumDirectionReversals: 2, maximumRecoveryErrorPerHeight: 0.08,
        maximumRootDisplacementPerHeight: 0.2, maximumGripErrorPerHeight: 0.04,
      },
      context: {
        schemaVersion: 1,
        attackNodeId: selected[0] ?? rig.at(-1)?.id ?? 0,
        rootNodeId: rig.find((node) => node.parentId === null)?.id ?? rig[0]?.id ?? 0,
        gripMaxError: null,
        contactNodeIds: rig.filter((node) => /foot|ankle/i.test(node.name)).map((node) => node.id),
        contributionNodeIds: rig.filter((node) => /hip|pelvis|spine|torso|shoulder|arm/i.test(node.name)).slice(0, 12).map((node) => node.id),
      },
    });
    if (result?.result === "COMBAT_QUALITY_ANALYZED") setQuality(result.value);
  };
  const visualize = async () => {
    if (!motionOverlaysEnabled) {
      onVisualization(null);
      return;
    }
    const nodeIds = selected.length > 0 ? selected : [rig.at(-1)?.id ?? 0];
    const sampleCount = reducedMotion ? 45 : 90;
    const onionCount = reducedMotion ? 0 : 2;
    const cacheKey = JSON.stringify([sourceRevision, clip.id, clip.revision, nodeIds, sampleCount, onionCount, Number(playheadSeconds.toFixed(4))]);
    const cached = MOTION_VISUALIZATION_CACHE_V1.get(cacheKey);
    if (cached) {
      onVisualization(cached);
      return;
    }
    const result = await execute("visualization", {
      operation: "BUILD_MOTION_VISUALIZATION", clip, rig: rigWire,
      request: {
        schemaVersion: 1, sourceRevision, clipId: clip.id, clipRevision: clip.revision,
        nodeIds, startSeconds: 0, endSeconds: clip.lengthSeconds, sampleCount,
        onionBefore: onionCount, onionAfter: onionCount, currentTimeSeconds: playheadSeconds,
      },
    });
    if (result?.result === "MOTION_VISUALIZATION_BUILT") {
      MOTION_VISUALIZATION_CACHE_V1.set(cacheKey, result.value);
      if (MOTION_VISUALIZATION_CACHE_V1.size > 24) {
        const oldest = MOTION_VISUALIZATION_CACHE_V1.keys().next().value;
        if (oldest !== undefined) MOTION_VISUALIZATION_CACHE_V1.delete(oldest);
      }
      onVisualization(result.value);
    }
  };
  const createVariant = async () => {
    const fingerprint = await execute("variant", { operation: "FINGERPRINT_CLIP", clip });
    if (fingerprint?.result !== "CLIP_FINGERPRINTED") return;
    const sourceClipFingerprintSha256 = fingerprint.value;
    const operations: Record<string, unknown>[] = [];
    if (speed !== 1) operations.push({ kind: "SPEED", factor: speed });
    if (mirrorVariant) {
      const { pairs, centers } = explicitMirrorPairs(rig);
      const mapResult = await execute("variant mirror", {
        operation: "BUILD_MIRROR_MAP", rig: rigWire, axis: "X", pairs,
        centerNodeIds: centers,
      });
      if (mapResult?.result !== "MIRROR_MAP_BUILT") return;
      operations.push({ kind: "MIRROR", mapping: mapResult.value });
    }
    if (amplitude !== 1) operations.push({ kind: "AMPLITUDE", factor: amplitude, nodeIds: selected });
    if (operations.length === 0) operations.push({ kind: "SPEED", factor: 1 });
    const recipe = {
      schemaVersion: 1, sourceRevision, sourceClipId: clip.id,
      sourceClipRevision: clip.revision, sourceClipFingerprintSha256,
      outputClipId: uniqueClipId(`${clip.id}-variant`, clips),
      outputName: `${clip.name} variant`, inheritedTags: ["custom"],
      addedTags: variantTags.split(",").map((tag) => tag.trim()).filter(Boolean), operations,
    };
    const result = await execute("variant", {
      operation: "CREATE_VARIANT", source: clip, rig: rigWire,
      recipe,
    });
    if (result?.result === "VARIANT_CREATED") {
      persist("variantRecipes", {
        id: `variant:${result.value.clip.id}`,
        name: `${result.value.clip.name} recipe`, kind: "VARIANT_RECIPE",
        sourceRevision, fingerprintSha256: result.value.fingerprintSha256,
        payload: recipe,
      });
      onAddClip(result.value.clip);
    }
  };
  const bakeSequence = async () => {
    const result = await execute("sequence", {
      operation: "BAKE_SEQUENCE",
      document: {
        schemaVersion: 1, sourceRevision,
        sequenceId: uniqueClipId("sequence", clips), outputName: "Custom sequence",
        segments, revision: 1,
      },
      availableClips: clips,
    });
    if (result?.result === "SEQUENCE_BAKED") {
      setSequenceBake(result.value);
      persist("sequences", {
        id: `sequence:${result.value.customClip.id}`,
        name: result.value.customClip.name, kind: "SEQUENCE",
        sourceRevision, fingerprintSha256: result.value.fingerprintSha256,
        payload: {
          schemaVersion: 1, sourceRevision,
          sequenceId: result.value.previewClip.id,
          outputName: result.value.previewClip.name,
          segments,
          revision: result.value.documentRevision,
        },
      });
      onPreviewClip(result.value.previewClip);
    }
  };

  return (
    <section className="animation-workbench-tools" aria-labelledby="advanced-authoring-title" aria-busy={busy !== null}>
      <header>
        <div>
          <h3 id="advanced-authoring-title">Advanced animation authoring</h3>
          <small>Core-backed pose, combat, motion, sequence and variant tools</small>
        </div>
        {busy ? <span role="status">Working: {busy}â€¦</span> : null}
      </header>

      <details open>
        <summary>Pose tools</summary>
        <div className="animation-workbench-tools__bone-grid" aria-label="Pose bone multi-select">
          {rig.map((node) => (
            <label key={node.id}>
              <input type="checkbox" checked={selected.includes(node.id)} onChange={() => setSelected(toggleId(selected, node.id))} />
              {node.name}
            </label>
          ))}
        </div>
        <div className="animation-workbench-tools__actions">
          <button type="button" onClick={() => setSelected([])}>Whole body mode</button>
          <button type="button" onClick={() => setSelected(namedBoneGroup(rig, "ARMS"))}>Select arms</button>
          <button type="button" onClick={() => setSelected(namedBoneGroup(rig, "LEGS"))}>Select legs</button>
          <button type="button" onClick={() => setSelected(namedBoneGroup(rig, "SPINE"))}>Select spine</button>
          <button type="button" disabled={selected.length !== 1} onClick={() => setSelected(descendantChain(rig, selected[0]!))}>Select child chain</button>
          <button type="button" onClick={() => void capture()}>Copy pose</button>
          <button type="button" disabled={!pose} onClick={() => pose && persist("posePresets", { id: `pose:${clip.id}:${pose.timeSeconds.toFixed(4)}`, name: `${clip.name} pose @ ${pose.timeSeconds.toFixed(2)} s`, kind: "POSE", sourceRevision, fingerprintSha256: pose.fingerprintSha256, payload: pose as unknown as Record<string, unknown> })}>Save pose preset</button>
          <button type="button" disabled={!pose} onClick={() => pose && void applyPose(pose, selected.length === 0 ? "WHOLE_BODY" : "SELECTED_BONES")}>Paste pose</button>
          <button type="button" disabled={!pose} onClick={() => void mirror()}>Mirror X</button>
          <button type="button" disabled={selected.length === 0} onClick={() => void reset()}>Reset selected</button>
          <button type="button" onClick={() => void reset(rig.map(({ id }) => id))}>Reset whole body</button>
        </div>
        {pose ? <small>Copied {pose.bones.length} bones Â· {pose.fingerprintSha256.slice(0, 12)}â€¦</small> : null}
      </details>

      <details open>
        <summary>Attack phases &amp; combat quality</summary>
        <div className="animation-workbench-tools__actions">
          <label>Phase template <select defaultValue="CUSTOM" onChange={(event) => {
            const factors = ATTACK_PHASE_PRESETS[event.currentTarget.value as keyof typeof ATTACK_PHASE_PRESETS];
            setPhaseTimes(factors.map((factor) => Number((factor * clip.lengthSeconds).toFixed(4))));
          }}>{Object.keys(ATTACK_PHASE_PRESETS).map((preset) => <option key={preset} value={preset}>{preset.toLowerCase()}</option>)}</select></label>
          <button type="button" onClick={createSlashFromCurrentPose}>Create slash from current pose</button>
        </div>
        <div className="animation-workbench-tools__phase-grid">
          {PHASES.map((phase, index) => (
            <label key={phase}>{phase.replace("_", " ")}
              <input type="number" min={0} max={clip.lengthSeconds} step={0.01} value={phaseTimes[index] ?? 0}
                onChange={(event) => setPhaseTimes(replaceNumber(phaseTimes, index, event.currentTarget.valueAsNumber))} />
              <button type="button" onClick={() => previewPhase(index)}>Preview {phase.replace("_", " ")}</button>
            </label>
          ))}
        </div>
        <div className="animation-workbench-tools__actions">
          <button type="button" onClick={() => void createPhases()}>Save phase timeline</button>
          <button type="button" onClick={() => onPreviewClip(clip)}>Preview full one-shot</button>
          <button type="button" onClick={() => void analyze()}>Analyze attack</button>
          <label><input type="checkbox" checked={motionOverlaysEnabled} onChange={(event) => { setMotionOverlaysEnabled(event.currentTarget.checked); if (!event.currentTarget.checked) onVisualization(null); }} /> Motion overlays</label>
          <label><input type="checkbox" checked={reducedMotion} disabled={!motionOverlaysEnabled} onChange={(event) => setReducedMotion(event.currentTarget.checked)} /> Reduced motion</label>
          <button type="button" disabled={!motionOverlaysEnabled} onClick={() => void visualize()}>Show trails + onion skin</button>
          <button type="button" onClick={() => onVisualization(null)}>Hide overlays</button>
        </div>
        {motionOverlaysEnabled ? <small className="animation-motion-legend"><span>Blue: previous pose</span> Â· <span>current: model</span> Â· <span>pink: next pose</span></small> : null}
        {timeline ? <small>Five phases saved Â· {timeline.fingerprintSha256.slice(0, 12)}â€¦</small> : null}
        {quality ? <QualitySummary report={quality} /> : null}
      </details>

      <details>
        <summary>Procedural variant</summary>
        <label>Speed <input type="number" min={0.05} max={20} step={0.05} value={speed} onChange={(event) => setSpeed(event.currentTarget.valueAsNumber)} /></label>
        <label><input type="checkbox" checked={mirrorVariant} onChange={(event) => setMirrorVariant(event.currentTarget.checked)} /> Mirror through explicit humanoid map</label>
        <label>Amplitude <input type="number" min={0} max={4} step={0.05} value={amplitude} onChange={(event) => setAmplitude(event.currentTarget.valueAsNumber)} /></label>
        <label>Tags <input value={variantTags} onChange={(event) => setVariantTags(event.currentTarget.value)} /></label>
        <button type="button" onClick={() => void createVariant()}>Create immutable Custom variant</button>
      </details>

      <details open>
        <summary>Sequence composer</summary>
        <div className="animation-sequence-composer">
          {segments.map((segment, index) => (
            <article key={segment.segmentId}>
              <strong>#{index + 1}</strong>
              <select value={segment.clipId} onChange={(event) => setSegments(updateSegment(segments, index, clips, { clipId: event.currentTarget.value }))}>
                {clips.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}
              </select>
              <label>In <input type="number" min={0} step={0.01} value={segment.sourceInSeconds} onChange={(event) => setSegments(updateSegment(segments, index, clips, { sourceInSeconds: event.currentTarget.valueAsNumber }))} /></label>
              <label>Out <input type="number" min={0.01} step={0.01} value={segment.sourceOutSeconds} onChange={(event) => setSegments(updateSegment(segments, index, clips, { sourceOutSeconds: event.currentTarget.valueAsNumber }))} /></label>
              <label>Repeat <input type="number" min={1} max={64} step={1} value={segment.repeatCount} onChange={(event) => setSegments(updateSegment(segments, index, clips, { repeatCount: event.currentTarget.valueAsNumber }))} /></label>
              <label>Blend <input type="number" min={0} step={0.01} disabled={index === 0} value={segment.transitionFromPrevious.durationSeconds} onChange={(event) => setSegments(updateSegment(segments, index, clips, { blend: event.currentTarget.valueAsNumber }))} /></label>
              <div>
                <button type="button" disabled={index === 0} onClick={() => setSegments(move(segments, index, index - 1))}>â†‘</button>
                <button type="button" disabled={index === segments.length - 1} onClick={() => setSegments(move(segments, index, index + 1))}>â†“</button>
                <button type="button" disabled={segments.length === 1} onClick={() => setSegments(segments.filter((_, itemIndex) => itemIndex !== index))}>Remove</button>
              </div>
            </article>
          ))}
        </div>
        <div className="animation-workbench-tools__actions">
          <button type="button" disabled={clips.length === 0 || segments.length >= 64} onClick={() => setSegments([...segments, makeSegment(segments.length, clips[0]!)])}>Add segment</button>
          <button type="button" disabled={segments.length === 0} onClick={() => void bakeSequence()}>Bake + preview</button>
          <button type="button" disabled={!sequenceBake} onClick={() => sequenceBake && onAddClip(sequenceBake.customClip)}>Save as Custom</button>
          <button type="button" onClick={() => onPreviewClip(null)}>Stop preview</button>
        </div>
        {sequenceBake ? <small>{sequenceBake.placedSegments.length} segments Â· {sequenceBake.customClip.lengthSeconds.toFixed(2)} s Â· {sequenceBake.fingerprintSha256.slice(0, 12)}â€¦</small> : null}
      </details>
      {error ? <p role="alert">{error}</p> : null}
    </section>
  );
}

function QualitySummary({ report }: { readonly report: CombatQualityReportV1 }) {
  return <div className="combat-quality-summary" role="status">
    <strong>Arc {report.attackArcLength.toFixed(3)} Â· peak {report.peakSpeed.toFixed(3)} Â· reversals {report.directionReversals} Â· contact {report.impactContactDisplacement.toFixed(3)}</strong>
    {report.bodyContributions.length > 0 ? <small>Body share: {report.bodyContributions.map((entry) => `bone ${entry.nodeId} ${(entry.normalizedShare * 100).toFixed(0)}%`).join(" Â· ")}</small> : null}
    {report.issues.length === 0 ? <p>No combat warnings.</p> : <ul>{report.issues.map((issue, index) => <li key={`${issue.kind}-${index}`}><b>{issue.severity}</b> {issue.message}</li>)}</ul>}
  </div>;
}

function phaseDefaults(length: number) { return [0, 0.2, 0.45, 0.6, 1].map((factor) => Number((factor * length).toFixed(4))); }
function toggleId(values: readonly number[], id: number) { return values.includes(id) ? values.filter((value) => value !== id) : [...values, id].sort((a, b) => a - b); }
function replaceNumber(values: readonly number[], index: number, value: number) { const output = [...values]; output[index] = value; return output; }
function uniqueClipId(base: string, clips: readonly AuthoredAnimationClipV1[]) { const ids = new Set(clips.map(({ id }) => id)); let id = base; let suffix = 2; while (ids.has(id)) id = `${base}-${suffix++}`; return id; }
function initialSegments(clips: readonly AuthoredAnimationClipV1[]) { return clips.length === 0 ? [] : Array.from({ length: 4 }, (_, index) => makeSegment(index, clips[index % clips.length]!)); }
function makeSegment(index: number, clip: AuthoredAnimationClipV1): AnimationSequenceSegmentRecipeV1 { return { segmentId: `segment-${crypto.randomUUID()}-${index}`, clipId: clip.id, sourceInSeconds: 0, sourceOutSeconds: clip.lengthSeconds, repeatCount: 1, transitionFromPrevious: { kind: index === 0 ? "CUT" : "CROSS_FADE", durationSeconds: index === 0 ? 0 : Math.min(0.1, clip.lengthSeconds / 4) }, phaseMarkers: [] }; }
function updateSegment(segments: readonly AnimationSequenceSegmentRecipeV1[], index: number, clips: readonly AuthoredAnimationClipV1[], patch: { clipId?: string; sourceInSeconds?: number; sourceOutSeconds?: number; repeatCount?: number; blend?: number }): readonly AnimationSequenceSegmentRecipeV1[] { return segments.map((segment, itemIndex): AnimationSequenceSegmentRecipeV1 => { if (itemIndex !== index) return segment; const selectedClip = clips.find(({ id }) => id === patch.clipId); const blend = patch.blend ?? segment.transitionFromPrevious.durationSeconds; const kind: "CUT" | "CROSS_FADE" = index === 0 || blend === 0 ? "CUT" : "CROSS_FADE"; return { ...segment, ...(patch.clipId ? { clipId: patch.clipId, sourceInSeconds: 0, sourceOutSeconds: selectedClip?.lengthSeconds ?? segment.sourceOutSeconds } : {}), ...(patch.sourceInSeconds !== undefined ? { sourceInSeconds: patch.sourceInSeconds } : {}), ...(patch.sourceOutSeconds !== undefined ? { sourceOutSeconds: patch.sourceOutSeconds } : {}), ...(patch.repeatCount !== undefined ? { repeatCount: patch.repeatCount } : {}), transitionFromPrevious: { kind, durationSeconds: index === 0 ? 0 : blend } }; }); }
function move<T>(values: readonly T[], from: number, to: number) { const output = [...values]; const [value] = output.splice(from, 1); if (value !== undefined) output.splice(to, 0, value); return output; }
function explicitMirrorPairs(rig: readonly AnimationRigNodeV1[]) { const groups = new Map<string, { left?: number; right?: number }>(); const centers: number[] = []; for (const node of rig) { const parsed = sideKey(node.name); if (!parsed) { centers.push(node.id); continue; } const group = groups.get(parsed.key) ?? {}; if (group[parsed.side] !== undefined) { centers.push(node.id); continue; } group[parsed.side] = node.id; groups.set(parsed.key, group); } const pairs: { leftNodeId: number; rightNodeId: number }[] = []; for (const group of groups.values()) { if (group.left !== undefined && group.right !== undefined) pairs.push({ leftNodeId: group.left, rightNodeId: group.right }); else centers.push(group.left ?? group.right!); } return { pairs, centers: [...new Set(centers)].sort((a, b) => a - b) }; }
function sideKey(name: string): { key: string; side: "left" | "right" } | null { const normalized = name.toLowerCase().replaceAll(/[^a-z0-9]/g, ""); if (normalized.includes("left")) return { key: normalized.replace("left", "side"), side: "left" }; if (normalized.includes("right")) return { key: normalized.replace("right", "side"), side: "right" }; const match = normalized.match(/^(.*?)(l|r)$/); return match ? { key: `${match[1]}side`, side: match[2] === "l" ? "left" : "right" } : null; }
function namedBoneGroup(rig: readonly AnimationRigNodeV1[], group: "ARMS" | "LEGS" | "SPINE") { const pattern = group === "ARMS" ? /arm|hand|shoulder|clavicle/i : group === "LEGS" ? /leg|foot|thigh|calf|ankle/i : /spine|torso|chest|neck|hip|pelvis/i; return rig.filter(({ name }) => pattern.test(name)).map(({ id }) => id).sort((a, b) => a - b); }
function descendantChain(rig: readonly AnimationRigNodeV1[], rootId: number) { const selected = new Set([rootId]); let changed = true; while (changed) { changed = false; for (const node of rig) if (node.parentId !== null && selected.has(node.parentId) && !selected.has(node.id)) { selected.add(node.id); changed = true; } } return [...selected].sort((a, b) => a - b); }
