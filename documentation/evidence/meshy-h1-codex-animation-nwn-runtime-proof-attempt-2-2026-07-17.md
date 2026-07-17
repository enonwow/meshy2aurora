# Meshy H1 Codex animation native NWN runtime proof — attempt 2

Status: `MISSING`.

This attempt continues, but does not overwrite,
`proof-output/meshy-h1-native-runtime-2026-07-17/summary.json`. It did not
produce a native NWN animation proof.

## Scope held constant

The only intended packet was `m2a_codex_aproof.hak` (SHA-256
`da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756`) plus
`m2a_codex_aproof.mod` (SHA-256
`dee6585745a6a57ce518af1443d9845333d1c7f1225b438ad9295e44b5a29d2d`). No
additional HAK or MOD was made, copied, or attached. The expected module
contract remains one `Mod_HakList` entry `m2a_codex_aproof`, area
`m2a_caproof_area`, UTC `m2a_caproof_h1`, and `Appearance_Type=15100`.

## Toolset failure boundary

A single fresh Aurora Toolset process ran as PID `50340` on the required
physical screen 2: `\\.\DISPLAY1`, `primary=false`, `1920x1032`. Its visible
initial frame was class `TfrmFrame` with title `BioWare Aurora Neverwinter
Nights Toolset v89.8193.37-17`.

The one Open Module route was the previously validated, non-global native
route: `TfrmFrame` receives `WM_COMMAND 44`, which must expose
`TdlgModuleSelect` titled `Open`; only then may its own controls select the
canonical MOD. In this session the expected dialog was not observed within
15 seconds. Readback then found no visible Toolset top-level window. The
pre-write process check found that PID `50340` no longer existed.

This is the exact blocker:
`TOOLSET_PROCESS_EXITED_AFTER_CANONICAL_OPEN_COMMAND`.

No second Open command was sent; no Toolset build/test action was guessed; no
direct `nwmain` argument was used; `nwtoolset.ini`/MRU were untouched; no
global cursor was used; and the process was neither closed nor restarted by
this worker. There is no fresh `nwengineLog` module-load entry, no NWN process
from this attempt, no H1 in-area image, and no MP4. Thus no native claim is
made.

The required next evidence remains: a successfully opened canonical MOD,
fresh NWN log evidence of the canonical module loading (or a precise engine
error), then native NWN before/after PNGs and an MP4 with visibly changing
`cpause1`. H1 remains Idle-only; this attempt does not cover movement, attack,
damage, or death.
