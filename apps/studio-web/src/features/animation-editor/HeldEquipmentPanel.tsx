import { useState } from "react";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import type { AnimationWorkbenchProjectStateV1 } from "./animationWorkbench";

export interface HeldEquipmentPreviewV1 {
  readonly file: File;
  readonly targetBoneName: string;
  readonly translation: readonly [number, number, number];
  readonly rotationEulerDegrees: readonly [number, number, number];
  readonly scale: readonly [number, number, number];
}

export interface HeldWeaponSourceInspectionV1 {
  readonly schemaVersion: 1;
  readonly filename: string;
  readonly byteSize: number;
  readonly sha256: string;
  readonly provenance: string;
  readonly triangleCount: number;
  readonly materialCount: number;
  readonly textureCount: number;
}

export function HeldEquipmentPanel({
  rig,
  value,
  onChange,
  onInspect,
  persisted,
  onPersistedChange,
}: {
  rig: readonly AnimationRigNodeV1[];
  value: HeldEquipmentPreviewV1 | null;
  onChange: (value: HeldEquipmentPreviewV1 | null) => void;
  onInspect?: (file: File) => Promise<HeldWeaponSourceInspectionV1>;
  persisted?: AnimationWorkbenchProjectStateV1["heldWeapon"];
  onPersistedChange?: (value: AnimationWorkbenchProjectStateV1["heldWeapon"]) => void;
}) {
  const [inspection, setInspection] = useState<HeldWeaponSourceInspectionV1 | null>(null);
  const [inspectionError, setInspectionError] = useState<string | null>(null);
  const [inspectionBusy, setInspectionBusy] = useState(false);
  const defaultBone = findDefaultRightHandV1(rig) ?? rig[0];
  const update = (patch: Partial<HeldEquipmentPreviewV1>) => {
    if (!value) return;
    const next = { ...value, ...patch };
    onChange(next);
    if (persisted && onPersistedChange) {
      onPersistedChange({
        ...persisted,
        attachment: {
          targetBoneName: next.targetBoneName,
          translation: next.translation,
          rotationEulerDegrees: next.rotationEulerDegrees,
          scale: next.scale,
        },
      });
    }
  };
  return (
    <section className="held-equipment" aria-labelledby="held-equipment-title">
      <header>
        <div>
          <h3 id="held-equipment-title">Held equipment</h3>
          <small>Direct creature attachment preview</small>
        </div>
        {value ? (
          <button type="button" onClick={() => { onChange(null); onPersistedChange?.(null); }}>Remove</button>
        ) : null}
      </header>
      <label>
        Weapon GLB
        <input
          type="file"
          accept=".glb,model/gltf-binary"
          onChange={(event) => {
            const file = event.currentTarget.files?.[0];
            if (!file || !defaultBone) return;
            setInspection(null);
            setInspectionError(null);
            const preview: HeldEquipmentPreviewV1 = {
              file,
              targetBoneName: persisted?.attachment.targetBoneName ?? defaultBone.name,
              translation: persisted?.attachment.translation ?? [0, 0, 0],
              rotationEulerDegrees: persisted?.attachment.rotationEulerDegrees ?? [0, 0, 0],
              scale: persisted?.attachment.scale ?? [1, 1, 1],
            };
            if (onInspect) {
              setInspectionBusy(true);
              void onInspect(file).then((result) => {
                if (
                  persisted
                  && (
                    result.sha256 !== persisted.source.sha256
                    || result.byteSize !== persisted.source.byteLength
                  )
                ) {
                  throw new Error(`Reconnect rejected: expected exact weapon sha256:${persisted.source.sha256}.`);
                }
                setInspection(result);
                onChange(preview);
                onPersistedChange?.({
                  source: {
                    name: file.name,
                    byteLength: file.size,
                    lastModified: file.lastModified,
                    sha256: result.sha256,
                  },
                  attachment: {
                    targetBoneName: preview.targetBoneName,
                    translation: preview.translation,
                    rotationEulerDegrees: preview.rotationEulerDegrees,
                    scale: preview.scale,
                  },
                });
                setInspectionBusy(false);
              }).catch((error: unknown) => {
                setInspectionError(error instanceof Error ? error.message : String(error));
                setInspectionBusy(false);
                onChange(null);
              });
            } else {
              onChange(preview);
            }
          }}
        />
      </label>
      {inspectionBusy ? <p role="status">Inspecting exact rigid weapon GLB…</p> : null}
      {inspectionError ? <p role="alert">{inspectionError}</p> : null}
      {!value && persisted ? (
        <p role="status">
          Weapon attachment saved. Reconnect <strong>{persisted.source.name}</strong>
          {" "}with exact sha256:{persisted.source.sha256.slice(0, 12)}â€¦ before preview or build.
        </p>
      ) : null}
      {value ? (
        <>
          <p className="held-equipment__identity">
            {value.file.name} · {(value.file.size / 1024).toFixed(1)} KiB
            {inspection
              ? ` · ${inspection.triangleCount.toLocaleString("en-US")} triangles · sha256:${inspection.sha256.slice(0, 12)}…`
              : ""}
          </p>
          <label>
            Primary hand bone
            <select
              value={value.targetBoneName}
              onChange={(event) => update({ targetBoneName: event.currentTarget.value })}
            >
              {rig.map((node) => <option key={node.id}>{node.name}</option>)}
            </select>
          </label>
          <VectorEditor
            label="Grip offset"
            value={value.translation}
            step={0.01}
            onChange={(translation) => update({ translation })}
          />
          <VectorEditor
            label="Grip rotation (°)"
            value={value.rotationEulerDegrees}
            step={1}
            onChange={(rotationEulerDegrees) => update({ rotationEulerDegrees })}
          />
          <VectorEditor
            label="Scale"
            value={value.scale}
            step={0.01}
            min={0.01}
            onChange={(scale) => update({ scale })}
          />
          <p className="held-equipment__warning">
            The preview is rigid-parented to this bone. A baked direct-creature
            weapon is part of the model and is not an interchangeable Aurora
            <code> Equip_ItemList</code> weapon.
          </p>
        </>
      ) : (
        <p>Select one local rigid GLB to preview it through the whole clip.</p>
      )}
    </section>
  );
}

export function parseHeldWeaponSourceInspectionV1(
  value: unknown,
): HeldWeaponSourceInspectionV1 {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Held weapon inspection must be an object.");
  }
  const record = value as Record<string, unknown>;
  if (
    record.schemaVersion !== 1
    || typeof record.filename !== "string"
    || !Number.isSafeInteger(record.byteSize)
    || typeof record.sha256 !== "string"
    || !/^[0-9a-f]{64}$/.test(record.sha256)
    || typeof record.provenance !== "string"
    || !Number.isSafeInteger(record.triangleCount)
    || !Number.isSafeInteger(record.materialCount)
    || !Number.isSafeInteger(record.textureCount)
  ) {
    throw new Error("Held weapon inspection does not match V1.");
  }
  return value as HeldWeaponSourceInspectionV1;
}

function VectorEditor({
  label,
  value,
  step,
  min,
  onChange,
}: {
  label: string;
  value: readonly [number, number, number];
  step: number;
  min?: number;
  onChange: (value: readonly [number, number, number]) => void;
}) {
  return (
    <fieldset className="held-equipment__vector">
      <legend>{label}</legend>
      {(["X", "Y", "Z"] as const).map((axis, index) => (
        <label key={axis}>
          {axis}
          <input
            type="number"
            step={step}
            min={min}
            value={value[index]}
            onChange={(event) => {
              const next = [...value] as [number, number, number];
              next[index] = event.currentTarget.valueAsNumber;
              onChange(next);
            }}
          />
        </label>
      ))}
    </fieldset>
  );
}

function findDefaultRightHandV1(rig: readonly AnimationRigNodeV1[]) {
  const aliases = new Set([
    "rhand",
    "handr",
    "righthand",
    "handright",
    "bip01rhand",
    "mixamorigrighthand",
  ]);
  return rig.find(({ name }) => aliases.has(
    name.toLocaleLowerCase("en-US").replaceAll(/[^a-z0-9]/g, ""),
  ));
}
