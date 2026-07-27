# M0 r27 — live Toolset/NWN proof — 2026-07-21

## Exact lineage

- MOD `m2a_m0r25.mod`, SHA-256 `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257`.
- ordered HAK: `m2a_m0r27`, `m2a_m0r26`, `m2a_m0r21`.
- r27 HAK SHA-256 `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd`.
- Area `m2a_m0a25`; entry `[10,10,0]`.
- fixture `m0_fixture`, live name `Meshy M0 binary vertical-slice fixture`, template/tag `nw_dwarfmerc001`, `Appearance_Type=848`, position `[10,14.5,0]`.
- binding `848 -> M2A_M0_MESHY_RIGID -> m2a_m0p01`; MDL SHA-256 `bf7864f0ed541ed93c4075d167f4a2392eeb6ef37b4cdf444cbb74efd5731578`.

No module Save, HAK repack, payload change, camera action, Focus action, or new iteration occurred in this run.

## Toolset

- adopted one responsive `nwtoolset` PID `36216`, start `2026-07-21T15:33:05.1215555Z`, exact frame `m2a_m0r25.mod` on `\\.\DISPLAY1`.
- exact selected Area-tree instance was the only node `Meshy M0 binary vertical-slice fixture`.
- read-only Properties PrintWindow showed First Name `Meshy M0 binary vertical-slice fixture`, Tag `nw_dwarfmerc001`, Appearance `M2A_M0_MESHY_RIGID`, and the bound model preview.
- Properties was closed only through targeted `Cancel`; follow-up enumeration showed zero `TdlgCreatureEdit` and the unchanged exact module frame.
- fresh validated `TScrollBox`: `proof-output/m0-r27-animation-type5-20260721/live/toolset/r27-after-selection.png`, SHA-256 `aeb49f6cb4335f1e91a1f33aa06675b94ec829e006cbb2d905fa0cf19b0fc01b`, 410891 bytes, `1275x839`, captured `2026-07-21T15:40:30.5963646Z`.
- Properties identity image: `proof-output/m0-r27-animation-type5-20260721/live/toolset/r27-creature-properties-readonly.png`, SHA-256 `7c138fa61167c88064f961c2c07389824b2a7cbc29e0e464d0a6ebf6d7e39142`.

Owner image acceptance verdict: `modelVisibility=visible`, `proofCompleteness=verified`.

## NWN

- AUR-S07 dry-run and arm passed for profile `m2a-m0-r27-on-r25-runtime`.
- the verified menu readback identified enabled `Build` position `4`, item `2`, command ID `121`; one targeted Test Module command started one `nwmain` PID `29092` at `2026-07-21T15:54:25.0390804Z`.
- engine log snapshot contains `[Tue Jul 21 17:54:34] Loading Module: m2a_m0r25`.
- AUR-S07 observation bound the one responsive process to the exact profile.
- fresh runtime PNG: `proof-output/m0-r27-animation-type5-20260721/live/runtime/r27-nwn-runtime.png`, SHA-256 `45540e33c08c35c768df896b95292b5c87cfee2a645382f391070132f2e95524`, 4846733 bytes, `1920x1080`, captured `2026-07-21T15:55:38.5818416Z` on `\\.\DISPLAY1`.

The PNG shows a settled textured runtime and the player character, but it does not bind the camera frustum to fixture position `[10,14.5,0]`. The profile records entry and fixture positions, not a runtime camera direction/frustum readback. Therefore absence of the fixture from this single image is not a candidate-bound visual failure.

NWN verdict: `modelVisibility=not_tested`, `proofCompleteness=missing`.

The current AUR-S07 validator returns `aur_s07_runtime_proof_failed` with its aggregate code `runtime_model_blank_or_placeholder` whenever `modelVisibility.status` is not `visible`. The saved packet explicitly records `blank=false`, `placeholder=false`, and `not_tested`; the project classification remains `missing`, not `not_visible`. This packet does not admit a new iteration.

## Live terminal state

- `nwtoolset` PID `36216`: responsive and left open.
- `nwmain` PID `29092`: responsive and left open.
- no Save, repack, relaunch, movement, or camera action was performed after the runtime capture.

## Superseding spatial classification — 2026-07-21

The earlier NWN `modelVisibility=not_tested`, `proofCompleteness=missing` result is retained above as the superseded pre-spatial classification. It was made before the recorded entry facing and exact fixture offset were evaluated together with the image.

The exact runtime constants in `crates/m2a-core/src/proof_module.rs` set entry `[10,10,0]`, entry direction `[0,+1]`, and the only fixture `m0_fixture` at `[10,14.5,0]`. The relative vector is therefore `[0,+4.5,0]`: zero lateral offset and 4.5 m directly ahead. `proof-output/m0-r27-animation-type5-20260721/reports/materialization-report.json` records scale `1.0` and model dimensions approximately `1.087 x 0.639 x 1.893 m`.

The fresh runtime PNG is a third-person view from behind the player and visibly exposes open, unobstructed terrain directly ahead. A scale-1 model about 1.893 m tall at the exact bound position 4.5 m forward would occupy that shown terrain, but no model appears there. This is a candidate-bound visual absence, not a framing or automation failure.

Superseding NWN verdict for exact r27: `modelVisibility=not_visible`, `proofCompleteness=failed`.

This visual failure satisfies the visibility axis only. No new iteration is authorized by this record alone: the diagnosed cause and minimal intended artifact delta still need to be recorded under the model-iteration gate. The Toolset and NWN sessions were not controlled while making this correction; there was no Save, movement, camera action, relaunch, repack, or close.

## Authorized close and rollback evidence — 2026-07-21

The owner explicitly authorized clean closure of this proof lane in the order NWN first, Toolset second, with no Save and no module change.

- Before close there was exactly one responsive `nwmain` PID `29092`, started `2026-07-21T15:54:25.0390804Z`, and exactly one responsive `nwtoolset` PID `36216`, started `2026-07-21T15:33:05.1215555Z`.
- Read-only Toolset enumeration still resolved the exact hidden `TfrmFrame` title `BioWare Aurora Neverwinter Nights Toolset v89.8193.37-17 - m2a_m0r25.mod`; it contained no dirty `*`. No visible Toolset modal was present.
- A targeted close request to the exact NWN main window produced the expected in-game `Quit Neverwinter Nights?` confirmation. The exact `Yes` point was invoked by messages addressed only to HWND `37028904`; no global cursor, mouse, keyboard, focus, force termination, movement, camera action, or relaunch was used. Final readback: `nwmain=0`.
- The verified central Toolset close atom then sent targeted `WM_CLOSE` to exact PID `36216` / `TApplication` HWND `3476048`. It returned `ok=true`, `blockerCode=null`, `afterProcesses=[]`, and `afterWindows=[]`. No Save prompt appeared and no Save response was sent. Final readback: `nwtoolset=0`.
- Independent zero-process and hash verification completed at `2026-07-21T16:17:43.0020918Z`.

Post-close exact lineage readback:

- MOD `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0r25.mod`: SHA-256 `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257`.
- ordered HAK 1 `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r27.hak`: SHA-256 `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd`.
- ordered HAK 2 `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r26.hak`: SHA-256 `6f805873420b5279b45dadc26d913fa7692374df9e0582c23b42c8a4b42d6bb0`.
- ordered HAK 3 `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r21.hak`: SHA-256 `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6`.

Close/rollback verdict: `verified`. Both owned processes are absent, the exact frozen MOD and ordered HAK lineage retain their approved hashes, and no Save, repack, or payload mutation occurred.
