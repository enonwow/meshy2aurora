# Creature weapon roll calibration - preparation - 2026-08-04

Status: **confirmed preview-basis defect fixed; exact next-candidate recipe
prepared; no new MOD/HAK materialized**.

## Confirmed cause

The Studio proxy used the correct longitudinal weapon axis (`+Y`) but swapped
the two cross-section axes:

- old proxy blade: broad in local `X`, thin in local `Z`;
- audited NWN `WSwLs` item parts: thin in local `X`, broad in local `Z`.

The retail structural packet records, for example, the longsword top bounds as
approximately `X = +/-0.010` and `Z = +/-0.048`. The old proxy therefore hid a
quarter-turn roll error: it could look broad in Studio while the real item was
rendered edge-on in Toolset.

The proxy now uses the NWN basis convention for both blade and guard. This is a
basis-equivalent clean-room proxy; no retail geometry was copied.

## Exact model observation

Owner screenshot of V6:

`C:\Users\enonw\AppData\Local\Temp\codex-clipboard-fa109118-61f2-4992-b3e3-05dbed7c0c32.png`

SHA-256: `b10f10b9f92903ca784489a2c2dbb5baffebaabf08d5d5771cacb9f4edd8ad46`.

shows the weapon attached to the right hand with the blade rendered as a thin
edge. V7 retains the same Creature hook matrix and changes the equipment
resource contract only. The prepared correction for the next allowed build is:

```json
{
  "schemaVersion": 1,
  "mode": "AUTO_PLUS_OFFSETS",
  "rightHand": {
    "rollDegrees": 90.0,
    "pitchDegrees": 0.0,
    "yawDegrees": 0.0
  },
  "leftHand": {
    "rollDegrees": 0.0,
    "pitchDegrees": 0.0,
    "yawDegrees": 0.0
  }
}
```

The correction is a local roll around the blade axis. It preserves the exact
automatic palm position and longitudinal blade direction. `+90` and `-90`
produce the same blade plane for a symmetric section; `+90` is the canonical
deterministic choice for this recipe.

## Candidate gate

The current exact candidate remains V7:

- `m2aweapdemo7.mod`;
- module `Meshy2Aurora Creature Weapon Grip V7`;
- Area `Meshy2Aurora Creature Weapon Test V7`;
- HAK `m2aweaphak7.hak`.

Its recorded state is still `modelVisibility=not_tested` and
`proofCompleteness=missing`. The V6 screenshot is strong diagnostic evidence
because the hook transform is unchanged, but it is not an exact V7 visual
verdict. The model-iteration gate therefore forbids materializing V8 until the
owner reports the visual result for exact V7. No V7 resource, hash, generator
identity or installed file was changed by this preparation.
