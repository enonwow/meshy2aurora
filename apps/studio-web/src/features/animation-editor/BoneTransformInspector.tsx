import { useEffect, useState } from "react";

export function BoneTransformInspector({
  selectedBoneName,
  path,
  onPathChange,
  onInsert,
}: {
  selectedBoneName: string | null;
  path: "ROTATION" | "TRANSLATION";
  onPathChange: (path: "ROTATION" | "TRANSLATION") => void;
  onInsert: (value: number[]) => void;
}) {
  const [components, setComponents] = useState<[number, number, number]>([0, 0, 0]);
  useEffect(() => setComponents([0, 0, 0]), [path, selectedBoneName]);
  const value = path === "ROTATION"
    ? eulerDegreesToQuaternion(components)
    : components;
  return (
    <section className="bone-transform-inspector" aria-labelledby="bone-transform-title">
      <h3 id="bone-transform-title">Bone &amp; clip</h3>
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
          ? `Stored quaternion: ${value.map((item) => item.toFixed(4)).join(", ")}`
          : "Values are stored in output-rig local space."}
      </p>
      <button
        type="button"
        disabled={!selectedBoneName}
        onClick={() => onInsert([...value])}
      >
        + Add keyframe
      </button>
    </section>
  );
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
