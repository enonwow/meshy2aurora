# Meshy H1 Codex animation native NWN runtime proof — 2026-07-17

Status: `MISSING`.

This is a fail-closed native-runtime attempt. It does **not** establish that
`cpause1` plays in NWN, and it does not claim an Aurora Toolset proof.

## Canonical packet and installation

Only the canonical pair was used. Historical `m2a_h1proof.mod` and
`m2a_m6p01.hak` were not installed or referenced.

| Target | SHA-256 | Result |
|---|---|---|
| `m2a_codex_aproof.hak` | `da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756` | copied to the user `hak` directory; destination hash verified |
| `m2a_codex_aproof.mod` | `dee6585745a6a57ce518af1443d9845333d1c7f1225b438ad9295e44b5a29d2d` | copied to the user `modules` directory; destination hash verified |

The read-only installer plan found both destinations absent before the copy; its
`Install` route refused overwrites. The generated HAK has `appearance.2da`
(type `2017`), `m2a_m6p01` (binary MDL, type `2002`), and `m2a_m6t01` (TGA,
type `3`). The generated MOD has one `Mod_HakList` entry:
`m2a_codex_aproof`. Its H1 UTC uses `Appearance_Type=15100` in area
`m2a_caproof_area`.

## Aurora-first runtime route

The local Aurora decompilation contains the Toolset serialisation/read path for
`Mod_HakList` and separately writes each `Mod_Hak` resref. It also supplies the
local test launch template `-userdirectory "%s" +TestNewModule "%s"` for
`nwmain.exe`. This supports the generated HAK being resolved through the MOD's
HAK list; it does not support an assumption that Toolset rebuilds a HAK.

No Toolset was launched: the direct NWN route reached the native module browser
and showed `m2a_codex_aproof` selected. Therefore there was no observed local
requirement to build/repack the module first.

## Native evidence and monitor state

NWN EE `v89.8193.37-17` ran as PID `52200`. The first native frame visibly
shows `m2a_codex_aproof` selected in NWN's Local module list:

- raw PNG: `proof-output/meshy-h1-native-runtime-2026-07-17/nwn-module-selected-before-play.png`

The window was then moved through the window API to the required physical
screen 2, reported by Windows as `\\.\DISPLAY1` with `primary=false`. Native
capture dimensions were `1920x1032`. No global cursor was used.

The custom NWN UI did not accept targeted `WM_LBUTTON` input on Play. A
targeted Enter led to a transient loading view and then returned to the module
browser. `nwengineLog.txt` has no `m2a_codex_aproof` load or HAK-resolution
error; it only has unrelated OpenAL/GOG messages. The still-running NWN session
was deliberately not closed or restarted.

Consequently the packet has no in-area H1 frame, no before/after pose pair, and
no MP4. The browser screenshot is installation/preflight evidence only, **not**
runtime animation evidence.

## Independent readback and exact gap

`proof-output/meshy-h1-codex-animation-proof-v1/reports/summary.json` records
an own binary-MDL readback with one output animation: `cpause1`, duration
`4.0333333` seconds, `hasMotion=true`. That is supporting evidence only.

The remaining blocking condition is named in the companion
`proof-output/meshy-h1-native-runtime-2026-07-17/summary.json` as
`NWN_NATIVE_SCENE_NOT_ENTERED_WITHOUT_GLOBAL_INPUT`. A future run must enter
`m2a_caproof_area` in the existing or an explicitly reopened NWN session and
produce native NWN before/after PNGs plus an MP4 showing the visible `cpause1`
change. Only then may the status become `VERIFIED` or `FAILED`.

H1 remains an Idle-only packet. `cwalk`, `crun`, attack, damage, and death are
not present and are not covered by this attempt.
