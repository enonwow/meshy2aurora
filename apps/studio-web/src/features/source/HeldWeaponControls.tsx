import type { CreatureHeldWeaponModeV1 } from "./heldWeapon";

interface Props {
  readonly value: CreatureHeldWeaponModeV1;
  readonly onChange: (value: CreatureHeldWeaponModeV1) => void;
  readonly compact?: boolean;
}

export function HeldWeaponControls({ value, onChange, compact = false }: Props) {
  return (
    <div className="held-weapon-controls">
      <label>
        Item in Creature hand
        <select
          aria-label="Item in Creature hand"
          value={value}
          onChange={(event) => onChange(event.currentTarget.value as CreatureHeldWeaponModeV1)}
        >
          <option value="NONE">None</option>
          <option value="RIGHT_HAND">V10 bastard sword — right hand</option>
          <option value="LEFT_HAND">V10 bastard sword — left hand</option>
          <option value="BOTH_HANDS">V10 bastard sword — both hands</option>
        </select>
      </label>
      {!compact ? (
        <p role="note">
          Embeds the complete V10 item recipe in the selected GIT/UTC hand slot. This setting
          does not change animations.
        </p>
      ) : null}
    </div>
  );
}
