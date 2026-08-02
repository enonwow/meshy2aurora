import { useEffect, useState } from "react";

export function BoneTransformInspector({
  selectedBoneName,
  path,
  selectedKeyCount,
  currentValue,
  currentValueSource,
  playheadSeconds,
  onPathChange,
  onInsert,
  onDeleteSelectedKeys,
}: {
  selectedBoneName: string | null;
  path: "ROTATION" | "TRANSLATION";
  selectedKeyCount: number;
  currentValue?: readonly number[];
  currentValueSource: "SELECTED_KEY" | "PLAYHEAD" | "DEFAULT";
  playheadSeconds: number;
  onPathChange: (path: "ROTATION" | "TRANSLATION") => void;
  onInsert: (value: number[]) => void;
  onDeleteSelectedKeys: () => void;
}) {
  const projectedCurrent = path === "ROTATION"
    ? quaternionToEulerDegrees(currentValue ?? [0, 0, 0, 1])
    : asTranslation(currentValue);
  const currentSignature = [
    path,
    selectedBoneName ?? "",
    currentValueSource,
    ...projectedCurrent,
  ].join(":");
  const [components, setComponents] =
    useState<[number, number, number]>(projectedCurrent);
  useEffect(() => {
    setComponents(projectedCurrent);
  // The signature intentionally tracks scalar values, not an ephemeral array.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentSignature]);
  const value = path === "ROTATION"
    ? eulerDegreesToQuaternion(components)
    : components;
  return (
    <section className="bone-transform-inspector" aria-labelledby="bone-transform-title">
      <h3 id="bone-transform-title">Transform</h3>
      <p>Selected bone: <strong>{selectedBoneName ?? "None"}</strong></p>
      <div role="tablist" aria-label="Transform path">
        {(["ROTATION", "TRANSLATION"] as const).map((candidate) => (
          <button
            type="button"
            role="tab"
            aria-selected={path === candidate}
            key={candidate}
            onClick={() => onPathChange(candidate)}
          >
            {candidate === "ROTATION" ? "Rotation" : "Translation"}
          </button>
        ))}
      </div>
      <p className="animation-inspector-current" role="status">
        {currentValueSource === "SELECTED_KEY"
          ? "Editing the selected keyframe value."
          : currentValueSource === "PLAYHEAD"
            ? `Sampled at ${playheadSeconds.toFixed(3)} s.`
            : "No track yet; using the neutral transform."}
      </p>
      <fieldset disabled={!selectedBoneName}>
        <legend>
          {path === "ROTATION" ? "Euler presentation (degrees)" : "Local translation"}
        </legend>
        {(["X", "Y", "Z"] as const).map((axis, index) => (
          <label key={axis}>
            {axis}
            <input
              type="number"
              step={path === "ROTATION" ? 1 : 0.01}
              value={components[index]}
              onChange={(event) => {
                const next = [...components] as [number, number, number];
                next[index] = event.currentTarget.valueAsNumber || 0;
                setComponents(next);
              }}
            />
          </label>
        ))}
      </fieldset>
      <p className="animation-inspector-canonical">
        {path === "ROTATION"
          ? `Keyframe quaternion to write: ${value.map((item) => item.toFixed(4)).join(", ")}`
          : `Keyframe local translation to write: ${value.map((item) => item.toFixed(4)).join(", ")}`}
      </p>
      <div className="bone-transform-inspector__actions">
        <button
          type="button"
          disabled={!selectedBoneName}
          onClick={() => onInsert([...value])}
        >
          + Add keyframe
        </button>
        <button
          type="button"
          aria-label="Delete selected keyframes"
          disabled={selectedKeyCount === 0}
          onClick={onDeleteSelectedKeys}
        >
          Delete keyframe
        </button>
      </div>
    </section>
  );
}

function asTranslation(value?: readonly number[]): [number, number, number] {
  return [
    value?.[0] ?? 0,
    value?.[1] ?? 0,
    value?.[2] ?? 0,
  ];
}

function quaternionToEulerDegrees(
  value: readonly number[],
): [number, number, number] {
  const length = Math.hypot(
    value[0] ?? 0,
    value[1] ?? 0,
    value[2] ?? 0,
    value[3] ?? 1,
  ) || 1;
  const [x, y, z, w] = [
    (value[0] ?? 0) / length,
    (value[1] ?? 0) / length,
    (value[2] ?? 0) / length,
    (value[3] ?? 1) / length,
  ];
  const roll = Math.atan2(
    2 * (w * x + y * z),
    1 - 2 * (x * x + y * y),
  );
  const pitchInput = 2 * (w * y - z * x);
  const pitch = Math.abs(pitchInput) >= 1
    ? Math.sign(pitchInput) * Math.PI / 2
    : Math.asin(pitchInput);
  const yaw = Math.atan2(
    2 * (w * z + x * y),
    1 - 2 * (y * y + z * z),
  );
  return [roll, pitch, yaw].map((component) => (
    component * 180 / Math.PI
  )) as [number, number, number];
}

function eulerDegreesToQuaternion(
  [xDegrees, yDegrees, zDegrees]: readonly number[],
): [number, number, number, number] {
  const [x, y, z] = [xDegrees, yDegrees, zDegrees]
    .map((value) => value! * Math.PI / 180);
  const [cx, sx] = [Math.cos(x / 2), Math.sin(x / 2)];
  const [cy, sy] = [Math.cos(y / 2), Math.sin(y / 2)];
  const [cz, sz] = [Math.cos(z / 2), Math.sin(z / 2)];
  const quaternion: [number, number, number, number] = [
    sx * cy * cz - cx * sy * sz,
    cx * sy * cz + sx * cy * sz,
    cx * cy * sz - sx * sy * cz,
    cx * cy * cz + sx * sy * sz,
  ];
  const length = Math.hypot(...quaternion);
  return quaternion.map((value) => value / length) as [
    number,
    number,
    number,
    number,
  ];
}
