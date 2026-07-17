# Meshy H1 Codex animation native NWN runtime proof — attempt 6

Status: `FAILED`.

Window: `2026-07-17T16:13:30.3938932+02:00` to
`2026-07-17T16:15:22.5934601+02:00`. Complete native packet: `false`.

## Result

This was a fresh Toolset process, PID `16524`, on physical screen 2
`\\.\DISPLAY1` (`primary=false`). It used the already-installed canonical
pair, without copying or overwriting:

- HAK `da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756`;
- MOD `dee6585745a6a57ce518af1443d9845333d1c7f1225b438ad9295e44b5a29d2d`.

The current visible `TfrmFrame` menu exposed File position 1 / command `44`.
This time it was dispatched through synchronous `SendMessageTimeout`, then the
native `TdlgModuleSelect` (`Open`) was read. Of 77 entries, only index 51 was
`m2a_codex_aproof`; that exact entry was selected and the dialog's own `Open`
button was invoked.

The dialog closed, but no module filename entered the Toolset title, no Area
tree/viewer appeared, and no error modal was present. The support capture is
`proof-output/meshy-h1-native-runtime-2026-07-17-attempt-6/toolset-no-module-context.png`.
The required Toolset readback therefore failed. `Build → Test Module`, NWN,
the engine-log gate, native PNGs and MP4 were correctly not attempted.

## Mandatory aurora-web post-mortem (read-only)

`C:\Projects\aurora-web\docs\aurora-toolset-command-map-2026-06-18.md`
CM-01/CM-02 and
`C:\Projects\aurora-web\docs\aurora-toolset-basic-ops-reproduced-2026-06-17.md`
section 2 remain the latest checked precedent. Their route is exactly the
current `TfrmFrame` `WM_COMMAND 44`/`TdlgModuleSelect`/exact list selection/
`BM_CLICK Open`; their readback is a frame-title change to the module filename,
then an area context.

Attempt 5 used asynchronous dispatch; attempt 6 changed precisely that to the
documented synchronous route and still produced the same absence of module
context. This remains an attempt-level `FAILED`, not native animation success
and not a reason to launch NWN by a prohibited route. The next audit found a
specific adapter difference: the checked Aurora-web route sets the visible
`TdlgModuleSelect` `TEdit` with `WM_SETTEXT`, posts `BM_CLICK` to `Open`, and
then polls the frame title; it does not use list selection as its authoritative
input. The Toolset was closed after this attempt; NWN was never started.
