import { useState } from "react";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import type {
  AnimationQualityReportV1,
  AnimationQualityIssueV1,
} from "../animation-authoring-v2/types";

export function AnimationQualityPanel({
  rig,
  report,
  loading,
  error,
  loopExpected,
  rootNodeId,
  contactNodeIds,
  onLoopExpectedChange,
  onRootNodeChange,
  onContactNodeToggle,
  onJumpToIssue,
  toolBusy = false,
  toolMessage = null,
  onBlendLoop,
  onRootMotion,
  onLockContacts,
  onReduceKeys,
}: {
  rig: readonly AnimationRigNodeV1[];
  report: AnimationQualityReportV1 | null;
  loading: boolean;
  error: string | null;
  loopExpected: boolean;
  rootNodeId: number;
  contactNodeIds: ReadonlySet<number>;
  onLoopExpectedChange: (value: boolean) => void;
  onRootNodeChange: (nodeId: number) => void;
  onContactNodeToggle: (nodeId: number) => void;
  onJumpToIssue: (issue: AnimationQualityIssueV1) => void;
  toolBusy?: boolean;
  toolMessage?: string | null;
  onBlendLoop?: (windowSeconds: number) => void;
  onRootMotion?: (policy: "IN_PLACE" | { scale: number }) => void;
  onLockContacts?: () => void;
  onReduceKeys?: (positionError: number, angularErrorRadians: number) => void;
}) {
  const [blendWindowSeconds, setBlendWindowSeconds] = useState(0.12);
  const [rootScale, setRootScale] = useState(1);
  const [positionError, setPositionError] = useState(0.0001);
  const [angularError, setAngularError] = useState(0.001);
  return (
    <section className="animation-quality" aria-labelledby="animation-quality-title">
      <header>
        <div>
          <h3 id="animation-quality-title">Animation quality</h3>
          <small>Core world-pose analysis</small>
        </div>
        {loading ? <span role="status">Analyzing…</span> : null}
      </header>
      <label>
        <input
          type="checkbox"
          checked={loopExpected}
          onChange={(event) => onLoopExpectedChange(event.currentTarget.checked)}
        />
        This clip should loop
      </label>
      <label>
        Root motion bone
        <select
          value={rootNodeId}
          onChange={(event) => onRootNodeChange(Number(event.currentTarget.value))}
        >
          {rig.map((node) => <option key={node.id} value={node.id}>{node.name}</option>)}
        </select>
      </label>
      <details>
        <summary>Ground contact bones ({contactNodeIds.size})</summary>
        <div className="animation-quality__contacts">
          {rig.map((node) => (
            <label key={node.id}>
              <input
                type="checkbox"
                checked={contactNodeIds.has(node.id)}
                onChange={() => onContactNodeToggle(node.id)}
              />
              {node.name}
            </label>
          ))}
        </div>
      </details>
      {error ? <p role="alert">{error}</p> : null}
      {report ? (
        <>
          <p className="animation-quality__summary">
            {report.issues.length === 0
              ? "No quality issues detected."
              : `${report.issues.length} issue${report.issues.length === 1 ? "" : "s"}`}
            {" · "}{report.sampleCount} samples
            {" · "}{report.distinctPoseCount} distinct poses
          </p>
          <ul className="animation-quality__issues">
            {report.issues.map((issue, index) => (
              <li key={`${issue.kind}:${issue.nodeId ?? "clip"}:${issue.startSeconds}:${index}`}>
                <button type="button" onClick={() => onJumpToIssue(issue)}>
                  <strong>{qualityLabel(issue.kind)}</strong>
                  <span>{issue.severity} · {issue.startSeconds.toFixed(2)} s</span>
                  <small>{issue.message}</small>
                </button>
              </li>
            ))}
          </ul>
        </>
      ) : null}
      {onBlendLoop || onRootMotion || onLockContacts || onReduceKeys ? (
        <details className="animation-quality__tools" open>
          <summary>Repair tools</summary>
          <div>
            {onBlendLoop ? (
              <label>
                Loop blend window (s)
                <input
                  type="number"
                  min={0.01}
                  step={0.01}
                  value={blendWindowSeconds}
                  onChange={(event) => setBlendWindowSeconds(event.currentTarget.valueAsNumber)}
                />
                <button
                  type="button"
                  disabled={toolBusy || !(blendWindowSeconds > 0)}
                  onClick={() => onBlendLoop(blendWindowSeconds)}
                >
                  Blend loop seam
                </button>
              </label>
            ) : null}
            {onRootMotion ? (
              <label>
                Root motion scale
                <input
                  type="number"
                  min={0}
                  step={0.1}
                  value={rootScale}
                  onChange={(event) => setRootScale(event.currentTarget.valueAsNumber)}
                />
                <span className="animation-quality__tool-row">
                  <button
                    type="button"
                    disabled={toolBusy}
                    onClick={() => onRootMotion("IN_PLACE")}
                  >
                    Make in-place
                  </button>
                  <button
                    type="button"
                    disabled={toolBusy || !(rootScale >= 0)}
                    onClick={() => onRootMotion({ scale: rootScale })}
                  >
                    Scale root
                  </button>
                </span>
              </label>
            ) : null}
            {onLockContacts ? (
              <button
                type="button"
                disabled={toolBusy || contactNodeIds.size === 0}
                onClick={onLockContacts}
              >
                Lock suggested contacts
              </button>
            ) : null}
            {onReduceKeys ? (
              <fieldset>
                <legend>Key reduction tolerance</legend>
                <label>
                  Position
                  <input
                    type="number"
                    min={0}
                    step={0.0001}
                    value={positionError}
                    onChange={(event) => setPositionError(event.currentTarget.valueAsNumber)}
                  />
                </label>
                <label>
                  Angle (rad)
                  <input
                    type="number"
                    min={0}
                    step={0.001}
                    value={angularError}
                    onChange={(event) => setAngularError(event.currentTarget.valueAsNumber)}
                  />
                </label>
                <button
                  type="button"
                  disabled={toolBusy || positionError < 0 || angularError < 0}
                  onClick={() => onReduceKeys(positionError, angularError)}
                >
                  Reduce linear keys
                </button>
              </fieldset>
            ) : null}
          </div>
          {toolBusy ? <p role="status">Applying exact Core tool…</p> : null}
          {toolMessage ? <p role="status">{toolMessage}</p> : null}
        </details>
      ) : null}
    </section>
  );
}

function qualityLabel(kind: AnimationQualityIssueV1["kind"]) {
  return ({
    STATIC_PLAYBACK: "Static playback",
    LOOP_DISCONTINUITY: "Loop seam",
    ROOT_DRIFT: "Root drift",
    GROUND_PENETRATION: "Ground penetration",
    FOOT_SLIDING: "Foot sliding",
    MOTION_SPIKE: "Motion spike",
  } as const)[kind];
}
