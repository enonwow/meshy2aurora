# M0 r30 Toolset Gate B live proof - 2026-07-21

## Verdict

Toolset Gate B is `modelVisibility=visible` and
`proofCompleteness=verified` for exact r30. This handoff stops before NWN.

## Exact identity

- MOD `m2a_m0r30`: SHA-256 `8c87317956053cecdf40d939c9061a174943b982a8db2c94d35aa2fba44043df`.
- HAK `m2a_m0r30`: SHA-256 `6694c0fb7f16665025d312dfd35c4bcbb293fccdb0760c110492a5eb2d576d09`.
- Area `m2a_m0a30`; fixture tree text `Meshy M0 binary vertical-slice fixture`, one match, occurrence `0`, depth `3`.
- Tree selection handle and independent caret handle: `131152968`; caret sibling index `0`.
- Bound fixture identity: `m0_fixture`, template `nw_dwarfmerc001`, appearance row `15100`, position `[10,14.5,0]`, model `m2a_m0p01`.

## Live route

The central start recorded `unexpected_module_window_loaded` because the MRU
loaded `m2a_m0r29.mod`. In the same clean proof-owned session, File/Open was
validated as menu position `0/1`, command `44`. The exact `TdlgModuleSelect /
Open` modal was moved from the primary monitor to the frame's `DISPLAY1` using
the verified no-global-input atom, then the verified opener loaded the exact
installed r30 MOD. The resulting clean frame was
`BioWare Aurora Neverwinter Nights Toolset v89.8193.37-17 - m2a_m0r30.mod` in
PID `7764`; the modal closed and `nwmain=0`.

Area `m2a_m0a30` opened through the native Area tree. Objects selection mode
was read back checked. No Properties dialog was needed because the immutable
binary profile, unique native node, independent caret readback, and hash-bound
fixture/appearance resource chain already establish exact identity.

## Capture and evidence

- Fresh PNG: `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\live\toolset\r30-after-selection.png`
  - SHA-256 `aeb49f6cb4335f1e91a1f33aa06675b94ec829e006cbb2d905fa0cf19b0fc01b`
  - `TScrollBox`, 1275x839, `DISPLAY1`, `textured_scene`.
- Capture metadata: `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\live\toolset\r30-after-selection.json`
  - SHA-256 `bf2b252fb65f367a05f3ba2acf6722a36b91e34b0820f2f7a7a175b3b340df5f`.
- Recovery continuation: `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\live\toolset\r30-clean-session-switch-continuation.json`
  - SHA-256 `7c87ddac8669d89bdaf193f2871f0a61783f4a751ca56dc675ee9abced65700b`.
- Gate B packet: `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\live\toolset\r30-gate-b-packet.json`
  - SHA-256 `c2ebbf98c5cc7c8b431bbab58c868c2d2956c55befb6d385816268fade3506b5`.

Visual inspection of the original PNG finds a small textured 3D model inside
the green selected-object outline, distinct from the terrain grid. This meets
the owner acceptance rule of exact identity plus a fresh validated `TScrollBox`
image; no Focus or camera adjustment was performed.

## Safety

No Save, Build, repack, HAK copy, geometry-observe, geometry-finalize, global
input, Focus, camera movement, or NWN launch occurred. Toolset remains open,
responsive, clean, and proof-owned at exact r30. The MOD is intentionally
locked while loaded; its installed SHA was verified before open and its source
SHA was verified again after capture.
