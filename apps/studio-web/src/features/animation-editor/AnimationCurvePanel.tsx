import { useEffect, useMemo, useState, type PointerEvent as ReactPointerEvent } from "react";
import type { AuthoredAnimationTrackV1 } from "../animation-studio/types";
import {
  createAutoCurveFromTrackV1,
  type AnimationCurveResampleReportV1,
  type AnimationEditorCurveTrackV1,
} from "./animationCurves";

interface Props {
  readonly track: AuthoredAnimationTrackV1 | null;
  readonly busy: boolean;
  readonly error: string | null;
  readonly report: AnimationCurveResampleReportV1 | null;
  readonly onSmooth: (curve: AnimationEditorCurveTrackV1, tolerance: number) => void;
}

export function AnimationCurvePanel({ track, busy, error, report, onSmooth }: Props) {
  const [tolerance, setTolerance] = useState(0.005);
  const [channel, setChannel] = useState(0);
  const [curve, setCurve] = useState<AnimationEditorCurveTrackV1 | null>(
    () => track ? createAutoCurveFromTrackV1(track) : null,
  );
  const [selected, setSelected] = useState<readonly string[]>([]);
  const [tangentMode, setTangentMode] = useState<"AUTO" | "ALIGNED" | "BROKEN" | "FLAT">("AUTO");
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState(0.5);
  const [snapEnabled, setSnapEnabled] = useState(true);
  const [snapSeconds, setSnapSeconds] = useState(0.01);
  const [drag, setDrag] = useState<GraphDragV1 | null>(null);
  useEffect(() => {
    setCurve(track ? createAutoCurveFromTrackV1(track) : null);
    setSelected([]);
    setChannel(0);
    setZoom(1);
    setPan(0.5);
    setDrag(null);
  }, [track]);
  const view = useMemo(() => graphView(curve, channel, zoom, pan), [channel, curve, pan, zoom]);
  const points = useMemo(() => graphPoints(curve, channel, view), [channel, curve, view]);
  const active = curve?.keys.find(({ id }) => selected.includes(id)) ?? null;
  const updateActive = (patch: { timeSeconds?: number; value?: number; inTangent?: number; outTangent?: number }) => {
    if (!curve || selected.length === 0) return;
    setCurve({
      ...curve,
      keys: curve.keys.map((key) => {
        if (!selected.includes(key.id)) return key;
        const value = [...key.value];
        const inTangent = key.inTangent ? [...key.inTangent] : null;
        const outTangent = key.outTangent ? [...key.outTangent] : null;
        if (patch.value !== undefined) value[channel] = patch.value;
        if (patch.inTangent !== undefined && inTangent) inTangent[channel] = patch.inTangent;
        if (patch.outTangent !== undefined && outTangent) outTangent[channel] = patch.outTangent;
        const timeSeconds = patch.timeSeconds === undefined
          ? key.timeSeconds
          : clampTime(snapTime(key.timeSeconds + patch.timeSeconds - (active?.timeSeconds ?? patch.timeSeconds), snapEnabled, snapSeconds), curve);
        if (patch.outTangent !== undefined && tangentMode === "ALIGNED" && inTangent) inTangent[channel] = patch.outTangent;
        if (patch.inTangent !== undefined && tangentMode === "ALIGNED" && outTangent) outTangent[channel] = patch.inTangent;
        return { ...key, timeSeconds, value, inTangent, outTangent };
      }).sort((left, right) => left.timeSeconds - right.timeSeconds),
    });
  };
  const updateDraggedKey = (id: string, timeSeconds: number, valueAtPointer: number) => {
    if (!curve) return;
    setCurve({
      ...curve,
      keys: curve.keys.map((key) => {
        if (key.id !== id) return key;
        const value = [...key.value];
        value[channel] = valueAtPointer;
        return {
          ...key,
          timeSeconds: clampTime(snapTime(timeSeconds, snapEnabled, snapSeconds), curve),
          value,
        };
      }).sort((left, right) => left.timeSeconds - right.timeSeconds),
    });
  };
  const updateDraggedTangent = (kind: "IN_TANGENT" | "OUT_TANGENT", timeSeconds: number, valueAtPointer: number) => {
    if (!curve || !active || curve.path !== "TRANSLATION") return;
    const dt = timeSeconds - active.timeSeconds;
    if (Math.abs(dt) <= 1e-6) return;
    const tangent = (valueAtPointer - (active.value[channel] ?? 0)) / dt;
    updateActive(kind === "IN_TANGENT" ? { inTangent: tangent } : { outTangent: tangent });
  };
  const pointerData = (event: ReactPointerEvent<SVGSVGElement>) => {
    const rectangle = event.currentTarget.getBoundingClientRect();
    const x = (event.clientX - rectangle.left) / Math.max(1, rectangle.width) * 640;
    const y = (event.clientY - rectangle.top) / Math.max(1, rectangle.height) * 240;
    return { x, y, ...graphToData(view, x, y) };
  };
  const onGraphPointerDown = (event: ReactPointerEvent<SVGSVGElement>) => {
    if (event.button !== 0 || event.target !== event.currentTarget) return;
    const point = pointerData(event);
    event.currentTarget.setPointerCapture(event.pointerId);
    setDrag({ kind: "BOX", startX: point.x, startY: point.y, x: point.x, y: point.y });
    if (!event.shiftKey) setSelected([]);
  };
  const onGraphPointerMove = (event: ReactPointerEvent<SVGSVGElement>) => {
    if (!drag) return;
    const point = pointerData(event);
    if (drag.kind === "KEY") updateDraggedKey(drag.keyId, point.timeSeconds, point.value);
    else if (drag.kind === "IN_TANGENT" || drag.kind === "OUT_TANGENT") updateDraggedTangent(drag.kind, point.timeSeconds, point.value);
    else if (drag.kind === "BOX") setDrag({ kind: "BOX", startX: drag.startX, startY: drag.startY, x: point.x, y: point.y });
  };
  const onGraphPointerUp = (event: ReactPointerEvent<SVGSVGElement>) => {
    if (drag?.kind === "BOX") {
      const minX = Math.min(drag.startX, drag.x); const maxX = Math.max(drag.startX, drag.x);
      const minY = Math.min(drag.startY, drag.y); const maxY = Math.max(drag.startY, drag.y);
      const boxed = points.filter(({ x, y }) => x >= minX && x <= maxX && y >= minY && y <= maxY).map(({ id }) => id);
      setSelected((current) => [...new Set([...current, ...boxed])]);
    }
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
    setDrag(null);
  };
  const applyTangentMode = (mode: typeof tangentMode) => {
    setTangentMode(mode);
    if (!curve || curve.path === "ROTATION" || selected.length === 0) return;
    setCurve({
      ...curve,
      keys: curve.keys.map((key, index, keys) => {
        if (!selected.includes(key.id)) return key;
        const previous = keys[Math.max(0, index - 1)]!;
        const next = keys[Math.min(keys.length - 1, index + 1)]!;
        const dt = Math.max(1e-8, next.timeSeconds - previous.timeSeconds);
        const auto = key.value.map((_, axis) => ((next.value[axis] ?? 0) - (previous.value[axis] ?? 0)) / dt);
        if (mode === "FLAT") return { ...key, inTangent: key.value.map(() => 0), outTangent: key.value.map(() => 0) };
        if (mode === "AUTO" || mode === "ALIGNED") return { ...key, inTangent: auto, outTangent: auto };
        return key;
      }),
    });
  };
  return (
    <section className="animation-curve-panel" aria-labelledby="curve-panel-title">
      <header>
        <div><h3 id="curve-panel-title">Graph editor</h3><small>Visual translation curves; rotations keep shortest-arc quaternion interpolation.</small></div>
        <button type="button" disabled={!curve} onClick={() => curve && onSmooth(curve, tolerance)}>{busy ? "Bakingâ€¦" : "Bake LINEAR"}</button>
      </header>
      <div className="animation-curve-panel__toolbar">
        <label>Channel <select value={channel} onChange={(event) => setChannel(Number(event.currentTarget.value))}>
          {(track?.path === "ROTATION" ? ["Quaternion"] : ["X", "Y", "Z"]).map((label, index) => <option key={label} value={index}>{label}</option>)}
        </select></label>
        <label>Tolerance <input type="number" min={0.0001} max={1} step={0.001} value={tolerance} onChange={(event) => setTolerance(event.currentTarget.valueAsNumber)} /></label>
        <label>Zoom <input aria-label="Graph zoom" type="range" min={1} max={20} step={0.25} value={zoom} onChange={(event) => setZoom(event.currentTarget.valueAsNumber)} /></label>
        <label>Pan <input aria-label="Graph horizontal pan" type="range" min={0} max={1} step={0.01} value={pan} disabled={zoom === 1} onChange={(event) => setPan(event.currentTarget.valueAsNumber)} /></label>
        <button type="button" onClick={() => { setZoom(1); setPan(0.5); }}>Fit all</button>
        <label><input type="checkbox" checked={snapEnabled} onChange={(event) => setSnapEnabled(event.currentTarget.checked)} /> Snap</label>
        <label>Step <input aria-label="Graph snap seconds" type="number" min={0.0001} max={10} step={0.001} disabled={!snapEnabled} value={snapSeconds} onChange={(event) => setSnapSeconds(event.currentTarget.valueAsNumber)} /></label>
        {curve?.path === "TRANSLATION" ? <div role="group" aria-label="Tangent mode">{(["AUTO", "ALIGNED", "BROKEN", "FLAT"] as const).map((mode) => <button key={mode} type="button" aria-pressed={tangentMode === mode} onClick={() => applyTangentMode(mode)}>{mode}</button>)}</div> : null}
      </div>
      <svg className="animation-curve-panel__graph" viewBox="0 0 640 240" role="img" aria-label="Time value curve" onPointerDown={onGraphPointerDown} onPointerMove={onGraphPointerMove} onPointerUp={onGraphPointerUp} onPointerCancel={() => setDrag(null)}>
        <defs><clipPath id="animation-curve-plot"><rect x="32" y="16" width="592" height="208" /></clipPath></defs>
        <path d="M32 120H624M32 16V224" className="animation-curve-panel__axis" />
        <g clipPath="url(#animation-curve-plot)">
          {points.length > 1 ? <polyline points={points.map(({ x, y }) => `${x},${y}`).join(" ")} className="animation-curve-panel__line" /> : null}
          {curve?.path === "TRANSLATION" && active ? <TangentHandles active={active} channel={channel} view={view} onBegin={(kind, event) => { event.stopPropagation(); event.currentTarget.ownerSVGElement?.setPointerCapture(event.pointerId); setDrag({ kind, keyId: active.id }); }} /> : null}
          {points.map((point) => <circle key={point.id} cx={point.x} cy={point.y} r={selected.includes(point.id) ? 7 : 5} tabIndex={0} aria-label={`Key ${point.id}`} className={selected.includes(point.id) ? "is-selected" : ""} onPointerDown={(event) => { event.stopPropagation(); event.currentTarget.ownerSVGElement?.setPointerCapture(event.pointerId); setSelected(event.shiftKey ? toggle(selected, point.id) : [point.id]); setDrag({ kind: "KEY", keyId: point.id }); }} onKeyDown={(event) => { if (event.key === "Enter" || event.key === " ") setSelected(event.shiftKey ? toggle(selected, point.id) : [point.id]); }} />)}
          {drag?.kind === "BOX" ? <rect className="animation-curve-panel__selection" x={Math.min(drag.startX, drag.x)} y={Math.min(drag.startY, drag.y)} width={Math.abs(drag.x - drag.startX)} height={Math.abs(drag.y - drag.startY)} /> : null}
        </g>
      </svg>
      {active ? <fieldset className="animation-curve-panel__key-editor"><legend>{selected.length} selected key(s)</legend>
        <label>Time <input type="number" min={0} step={0.001} value={active.timeSeconds} onChange={(event) => updateActive({ timeSeconds: event.currentTarget.valueAsNumber })} /></label>
        <label>Value <input type="number" step={0.001} value={active.value[channel] ?? 0} onChange={(event) => updateActive({ value: event.currentTarget.valueAsNumber })} /></label>
        {curve?.path === "TRANSLATION" ? <><label>In tangent <input type="number" step={0.01} value={active.inTangent?.[channel] ?? 0} onChange={(event) => updateActive({ inTangent: event.currentTarget.valueAsNumber })} /></label><label>Out tangent <input type="number" step={0.01} value={active.outTangent?.[channel] ?? 0} onChange={(event) => updateActive({ outTangent: event.currentTarget.valueAsNumber })} /></label></> : null}
      </fieldset> : <p>Select keys; Shift-click adds to the selection.</p>}
      {report ? <p role="status">{report.editorKeyCount} editor keys â†’ {report.linearKeyCount} LINEAR keys Â· max error {Math.max(report.maxPositionError, report.maxAngularErrorRadians).toExponential(2)}</p> : null}
      {error ? <p role="alert">{error}</p> : null}
    </section>
  );
}

interface GraphViewV1 { readonly minTime: number; readonly maxTime: number; readonly minValue: number; readonly maxValue: number; }
type GraphDragV1 =
  | { readonly kind: "KEY" | "IN_TANGENT" | "OUT_TANGENT"; readonly keyId: string }
  | { readonly kind: "BOX"; readonly startX: number; readonly startY: number; readonly x: number; readonly y: number };

function graphView(curve: AnimationEditorCurveTrackV1 | null, channel: number, zoom: number, pan: number): GraphViewV1 {
  if (!curve || curve.keys.length === 0) return { minTime: 0, maxTime: 1, minValue: -1, maxValue: 1 };
  const times = curve.keys.map(({ timeSeconds }) => timeSeconds);
  const values = curve.keys.map(({ value }) => value[channel] ?? 0);
  const fullMinTime = Math.min(...times); const fullMaxTime = Math.max(...times);
  const fullSpan = Math.max(1e-6, fullMaxTime - fullMinTime);
  const visibleSpan = fullSpan / Math.max(1, zoom);
  const center = fullMinTime + visibleSpan / 2 + (fullSpan - visibleSpan) * Math.min(1, Math.max(0, pan));
  const rawMinValue = Math.min(...values); const rawMaxValue = Math.max(...values);
  const valuePadding = Math.max(1e-4, (rawMaxValue - rawMinValue) * 0.08);
  return { minTime: center - visibleSpan / 2, maxTime: center + visibleSpan / 2, minValue: rawMinValue - valuePadding, maxValue: rawMaxValue + valuePadding };
}
function graphPoints(curve: AnimationEditorCurveTrackV1 | null, channel: number, view: GraphViewV1) {
  if (!curve || curve.keys.length === 0) return [];
  return curve.keys.map((key) => ({ id: key.id, ...dataToGraph(view, key.timeSeconds, key.value[channel] ?? 0) }));
}
function toggle(values: readonly string[], id: string) { return values.includes(id) ? values.filter((value) => value !== id) : [...values, id]; }
function dataToGraph(view: GraphViewV1, timeSeconds: number, value: number) { return { x: 32 + ((timeSeconds - view.minTime) / Math.max(1e-8, view.maxTime - view.minTime)) * 592, y: 224 - ((value - view.minValue) / Math.max(1e-8, view.maxValue - view.minValue)) * 208 }; }
function graphToData(view: GraphViewV1, x: number, y: number) { return { timeSeconds: view.minTime + ((x - 32) / 592) * (view.maxTime - view.minTime), value: view.minValue + ((224 - y) / 208) * (view.maxValue - view.minValue) }; }
function snapTime(value: number, enabled: boolean, step: number) { return enabled && Number.isFinite(step) && step > 0 ? Math.round(value / step) * step : value; }
function clampTime(value: number, curve: AnimationEditorCurveTrackV1) { const maximum = Math.max(...curve.keys.map(({ timeSeconds }) => timeSeconds)); return Math.min(maximum, Math.max(0, value)); }

function TangentHandles({ active, channel, view, onBegin }: { readonly active: AnimationEditorCurveTrackV1["keys"][number]; readonly channel: number; readonly view: GraphViewV1; readonly onBegin: (kind: "IN_TANGENT" | "OUT_TANGENT", event: ReactPointerEvent<SVGCircleElement>) => void }) {
  const center = dataToGraph(view, active.timeSeconds, active.value[channel] ?? 0);
  const timeSpan = view.maxTime - view.minTime;
  const dt = Math.max(timeSpan * 0.08, 1e-5);
  const incoming = dataToGraph(view, active.timeSeconds - dt, (active.value[channel] ?? 0) - (active.inTangent?.[channel] ?? 0) * dt);
  const outgoing = dataToGraph(view, active.timeSeconds + dt, (active.value[channel] ?? 0) + (active.outTangent?.[channel] ?? 0) * dt);
  return <g className="animation-curve-panel__tangents">
    <path d={`M${incoming.x} ${incoming.y}L${center.x} ${center.y}L${outgoing.x} ${outgoing.y}`} />
    <circle cx={incoming.x} cy={incoming.y} r="4" aria-label="Incoming tangent handle" onPointerDown={(event) => onBegin("IN_TANGENT", event)} />
    <circle cx={outgoing.x} cy={outgoing.y} r="4" aria-label="Outgoing tangent handle" onPointerDown={(event) => onBegin("OUT_TANGENT", event)} />
  </g>;
}
