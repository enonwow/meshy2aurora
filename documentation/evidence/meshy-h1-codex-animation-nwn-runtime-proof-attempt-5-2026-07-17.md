# Meshy H1 Codex animation native NWN runtime proof — attempt 5

Status: `FAILED`.

Window: `2026-07-17T16:07:35.2528110+02:00` to
`2026-07-17T16:12:03.6408888+02:00`. Complete native packet: `false`.

## Preflight

The existing, installed canonical files were used without installation or
overwrite:

- HAK `m2a_codex_aproof.hak` —
  `da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756`.
- MOD `m2a_codex_aproof.mod` —
  `dee6585745a6a57ce518af1443d9845333d1c7f1225b438ad9295e44b5a29d2d`.

Fresh Toolset PID `20304` used its visible `TfrmFrame` on physical screen 2:
`\\.\DISPLAY1`, `primary=false`, `1920x1032`. No NWN process existed at
start.

## Actual Toolset route and failure gate

The current live menu exposed `&File`; its position 1 command was `44` with
enabled state `256`. It opened the native `TdlgModuleSelect` titled `Open`.
The dialog contained 77 items and exactly one `m2a_codex_aproof`, index 51.
The adapter selected that exact item and clicked the dialog's own `Open`
button without global input.

The dialog then closed, but the Toolset title remained the bare application
title and no MDI area viewer/resource context appeared. There was no error
modal. This fails the mandatory Toolset module-context readback, so no
`Build → Test Module`, NWN launch, log claim, PNG or MP4 was attempted.

Supporting state capture:
`proof-output/meshy-h1-native-runtime-2026-07-17-attempt-5/toolset-no-module-context.png`.

## Mandatory aurora-web post-mortem (read-only)

The exact checked precedent is
`C:\Projects\aurora-web\docs\aurora-toolset-command-map-2026-06-18.md`
CM-01/CM-02 and
`C:\Projects\aurora-web\docs\aurora-toolset-basic-ops-reproduced-2026-06-17.md`
section 2. It specifies `SendMessageTimeout(WM_COMMAND, 44)` to the current
`TfrmFrame`, then exact `TdlgModuleSelect` selection and button click. Its
required readback is a frame title containing the selected module filename,
then the Areas tree/area context.

What was wrong: this attempt used asynchronous `PostMessage` for command 44.
Although the dialog appeared, no module context was verified after it closed.
The next attempt changes only that dispatch to the documented synchronous
`SendMessageTimeout` and will fail immediately again unless the module title
changes. The Toolset was closed after this audit; NWN was never started.
