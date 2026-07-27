# M0 v12 native-geometry gate diagnostic — 2026-07-20

**Status:** AUR-S07 runtime admission remains `missing`.  This is not evidence
that the Meshy model failed to load.

## Observation performed

The documented public, read-only command
`bootstrap-geometry-observe` was run against the saved v12 bootstrap
declaration.  It wrote
[`native-geometry-blocker.json`](../../proof-output/m0-v12-toolset-runtime-20260720/native-geometry-blocker.json)
with:

```text
failures:
  - area_geometry_invalid
  - entry_surface_native_observation_unavailable
```

No Toolset configuration, HAK, appearance, creature position, model resource,
camera or NWN process was changed.

## Facts that rule out a model-placement failure

Immediately after that observation, native readback still found one saved GIT
creature:

| Field | Value |
| --- | --- |
| Module SHA-256 | `5c7bd3de9f76c04d89688275815a525913cfa9d230619e0daade64fa52a42483` |
| Area | `m2a_m0a12` |
| Template | `nw_dwarfmerc001` |
| Appearance Type | `848` (`M2A_M0_MESHY_RIGID`) |
| Position | `[5.0642, 14.9358, 0]` |
| HAK | `m2a_m0v10`, SHA-256 `04afbb1dc1e2f5cbb601e5d005f81396a044fc7b16597022b104367c5ee74966` |
| Toolset state | one responding owned PID `20188`, clean `m2a_m0v12.mod` frame, no top-level modal |

The fresh validated `TScrollBox` observation recorded in
[`m0-v12-toolset-viewport-binding-2026-07-20.md`](m0-v12-toolset-viewport-binding-2026-07-20.md)
also shows the rendered fixture.  Thus neither a missing creature nor a
missing Meshy resource explains this runtime-admission result.

## Why `area_geometry_invalid` is not a geometry diagnosis here

The saved manifest records a successful native Save with a clean module title:

```text
titleBefore: ... m2a_m0v12.mod*
titleAfter:  ... m2a_m0v12.mod
hashDeferredUntilClose: true
```

The live Toolset holds an exclusive lock, so the save atom could not compute
its immediate file hash and persisted `before.sha256 = null` and
`after.sha256 = null`.  The current geometry observer defines
`nativeSave.areaWasSaved` as a clean title **and** a non-null
`manifest.stages.save.after.sha256`.  Consequently it emitted
`area_geometry_invalid` even though it also read the current saved MOD/GIT
identity, saw no invalid-location dialog, saw no inaccessible-object dialog,
and found the expected one fixture.

This is therefore a **gate-evidence deficiency caused by deferred hash
availability**, not a verified claim that the Area or creature geometry is
invalid.  It must not trigger another placement, Adjust Location retry, HAK
retry, MDL edit, or INI change.

## Genuine remaining AUR-S07 prerequisite

The independent failure
`entry_surface_native_observation_unavailable` is real: the central vertical
slice has no verified native observer that proves the IFO entry coordinate
`[10, 10, 0]` is an accessible painted Toolset surface.  Serialized IFO/GIT
coordinates and the existing Area screenshot cannot be promoted to that proof.

The documented fallback is a nonce-bound, user-owned entry-surface handoff.
Until a central native observer or that validated handoff exists, a geometry
gate cannot truthfully be `verified`; therefore no `aur-s07-runtime-profile/v1`
can be emitted and NWN must not be launched by a local substitute.

## Next safe work

Keep the healthy owned Toolset session open.  Any future central correction
for the deferred-hash condition must first satisfy the shared operating
standard: reproduce the narrow generic condition offline and add both an
executable regression and a negative contract test.  It would still not
replace the separate entry-surface proof requirement.
