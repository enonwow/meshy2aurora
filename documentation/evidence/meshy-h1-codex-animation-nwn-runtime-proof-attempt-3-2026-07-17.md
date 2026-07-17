# Meshy H1 Codex animation native NWN runtime proof — attempt 3

Status: `MISSING`.

This final attempt follows the binding
`documentation/aurora-web-nwn-proof-runbook-codex.md` and preserves attempts 1
and 2 unchanged.

## Canonical packet verified

The existing user-directory packet remains byte-verified:

| File | SHA-256 |
|---|---|
| `m2a_codex_aproof.hak` | `da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756` |
| `m2a_codex_aproof.mod` | `dee6585745a6a57ce518af1443d9845333d1c7f1225b438ad9295e44b5a29d2d` |

No extra HAK/MOD was created or copied. The required static contract remains a
single `Mod_HakList` entry `m2a_codex_aproof`, area `m2a_caproof_area`, UTC
`m2a_caproof_h1`, and `Appearance_Type=15100`.

## Runbook blocker

PID-first recovery found live, responsive Toolset PID `50340`. Its only active
surface was `TApplication`, titled `BioWare Aurora Neverwinter Nights Toolset
v89.8193.37-17`, recovered to physical screen 2 `\\.\DISPLAY1`
(`primary=false`, `1920x1032`).

Read-only window inspection then found no `TfrmFrame`, no Toolset child window,
and no menu handle. Therefore the runbook's mandatory File/Open route could not
be read and validated; neither a `TdlgModuleSelect/Open` dialog nor a current
Build → Test Module command/state exists to invoke. This is the concrete
fail-closed condition:

`TOOLSET_APPLICATION_WITHOUT_FRAME_OR_FILE_MENU_AFTER_PID_RECOVERY`.

No unvalidated command was sent after that determination. In particular, there
was no direct `nwmain` route, no NWN process, no fresh engine-log module load,
no global input, no `nwtoolset.ini`/MRU change, no second Toolset session, and
no Toolset cleanup/restart. Consequently there is no native H1 scene, PNG
before/after pair, MP4, or visible `cpause1` motion to claim.

The exact recovery requirement before another native attempt is a real,
visible `TfrmFrame` exposing its current File/Open menu. Only then may the
runbook continue to Toolset readback, live Build/Test Module discovery, NWN
load-log gate, and native animation capture. H1 remains Idle/`cpause1` only;
movement, attack, damage, and death are not covered.
