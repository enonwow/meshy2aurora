import type { CreatureWeaponEulerOffsetV1, CreatureWeaponGripOptionsV1 } from "../source/weaponGrip";
import { defaultCreatureWeaponGripOptionsV1 } from "../source/weaponGrip";

interface Props {
  readonly value: CreatureWeaponGripOptionsV1;
  readonly onChange: (value: CreatureWeaponGripOptionsV1) => void;
  readonly compact?: boolean;
}

const axes = [
  ["rollDegrees", "Roll (blade +Y)"],
  ["pitchDegrees", "Pitch (local +X)"],
  ["yawDegrees", "Yaw (local +Z)"],
] as const;

export function WeaponGripControls({ value, onChange, compact = false }: Props) {
  const updateHand = (
    hand: "rightHand" | "leftHand",
    axis: keyof CreatureWeaponEulerOffsetV1,
    angle: number,
  ) => onChange({
    ...value,
    [hand]: {
      ...value[hand],
      [axis]: Number.isFinite(angle) ? Math.max(-180, Math.min(180, angle)) : 0,
    },
  });
  const manual = value.mode === "AUTO_PLUS_OFFSETS";
  return (
    <section className={`weapon-grip-controls${compact ? " weapon-grip-controls--compact" : ""}`} aria-label="Creature weapon rotation">
      <div className="weapon-grip-controls__mode">
        <label>
          Item family
          <select
            aria-label="Item grip family"
            value={value.itemFamily ?? "SWORD"}
            onChange={(event) => onChange({
              ...value,
              itemFamily: event.target.value as NonNullable<CreatureWeaponGripOptionsV1["itemFamily"]>,
            })}
          >
            <option value="SWORD">Sword</option>
            <option value="AXE_MACE">Axe / mace</option>
            <option value="SPEAR_POLEARM">Spear / polearm</option>
            <option value="BOW_CROSSBOW">Bow / crossbow</option>
            <option value="SHIELD">Shield</option>
          </select>
        </label>
        <label>
          Weapon rotation
          <select
            aria-label="Weapon rotation mode"
            value={value.mode}
            onChange={(event) => onChange(event.target.value === "AUTO"
              ? { ...defaultCreatureWeaponGripOptionsV1(), itemFamily: value.itemFamily ?? "SWORD" }
              : { ...value, mode: "AUTO_PLUS_OFFSETS" })}
          >
            <option value="AUTO">Auto</option>
            <option value="AUTO_PLUS_OFFSETS">Auto + manual offsets</option>
          </select>
        </label>
        <button type="button" className="button button--quiet" onClick={() => onChange(defaultCreatureWeaponGripOptionsV1())}>
          Reset to Auto
        </button>
      </div>
      <div className="weapon-grip-controls__hands">
        {(["rightHand", "leftHand"] as const).map((hand) => (
          <fieldset key={hand} disabled={!manual}>
            <legend>{hand === "rightHand" ? "Right hand" : "Left hand"}</legend>
            {axes.map(([axis, label]) => (
              <label key={axis}>
                {label}
                <input
                  aria-label={`${hand === "rightHand" ? "Right" : "Left"} ${label}`}
                  type="number"
                  min={-180}
                  max={180}
                  step={0.1}
                  value={value[hand][axis]}
                  onChange={(event) => updateHand(hand, axis, Number(event.target.value))}
                />
                <span>deg</span>
              </label>
            ))}
          </fieldset>
        ))}
      </div>
    </section>
  );
}
