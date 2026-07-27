# M0 v12 Toolset viewport binding — 2026-07-20

**Status:** `verified` for Aurora Toolset visual presence; NWN runtime remains
`missing`.

## Scope

This packet binds the fresh Area viewport observation to the saved v12 module
and its one Meshy fixture.  It records a completed native transaction that
continued after an outer shell deadline; it does not infer success from that
deadline.

No `nwtoolset.ini`, MRU entry, stored window placement, or other Toolset user
setting was read for mutation or changed.  The capture itself was read-only.

## Saved identity chain

| Element | Fresh bound value |
| --- | --- |
| Module | `C:\\Users\\enonw\\Documents\\Neverwinter Nights\\modules\\m2a_m0v12.mod` |
| Module SHA-256 | `5c7bd3de9f76c04d89688275815a525913cfa9d230619e0daade64fa52a42483` |
| Area | `m2a_m0a12` |
| Ordered HAK list | `[m2a_m0v10]` |
| HAK SHA-256 | `04afbb1dc1e2f5cbb601e5d005f81396a044fc7b16597022b104367c5ee74966` |
| Fixture | `meshy-m0-v10`; template `nw_dwarfmerc001` |
| Saved Appearance Type | `848` |
| Appearance row | `M2A_M0_MESHY_RIGID` |
| Resolved model resource | `m2a_m0p01`, binary MDL type `2002`, SHA-256 `4cfa0ff8ef2ce54493256cc675de9236fb3045cc98d0f21dafaff22a0b0e4776` |
| Saved fixture position | `[5.0642, 14.9358, 0]` |
| Saved entry point | `[10, 10, 0]` in `m2a_m0a12` |

The native GIT readback contains exactly that one fixture and its declared
integer appearance row.  The completed bootstrap manifest records the native
dirty-to-clean Save and the same GIT/module identities:
[`bootstrap-manifest.json`](../../proof-output/m0-v12-toolset-runtime-20260720/bootstrap-manifest.json).

The resource relation `848 -> M2A_M0_MESHY_RIGID -> m2a_m0p01` is the current
v10 package mapping recorded and cross-checked in
[`m0-v10-binary-runtime-preflight-2026-07-20.md`](m0-v10-binary-runtime-preflight-2026-07-20.md).

## Fresh independent viewport observation

| Element | Value |
| --- | --- |
| PNG | [`m0-v12-tscrollbox-observation-20260720.png`](../../proof-output/m0-v12-toolset-runtime-20260720/m0-v12-tscrollbox-observation-20260720.png) |
| PNG SHA-256 | `db76394c277071bdcfa1b8267a094e6dcb28abbf0cbc6b749092f84f5ff21945` |
| Capture result | [`m0-v12-tscrollbox-observation-20260720.json`](../../proof-output/m0-v12-toolset-runtime-20260720/m0-v12-tscrollbox-observation-20260720.json) |
| Result SHA-256 | `77f70a3329bb05f0b33a81aa467337769dffddff7be0dbc6417bdd0c1a4f7f00` |
| Target | exact visible `TScrollBox` inside Area viewer `m2a_m0a12`, `1275 x 839` |
| Monitor | `\\.\DISPLAY1`, `primary=false` |
| Capture method | physical-pixel capture of the validated `TScrollBox` rectangle after `WindowFromPoint` ownership verification |
| Input safety | global cursor/keyboard/mouse all `false` |

Visual inspection of that exact PNG shows the sole creature as a rendered
grey-green silhouette on the upper-left tile of the 2x2 Area, at the fixture's
saved location.  It is not a blank/missing-model or rainbow-placeholder
result.  The PNG is necessarily wide because it captures the whole validated
viewport; its small on-screen scale is not a changed position or model state.

**Aurora Toolset visual presence of the current M0 fixture: `verified`.**

The earlier `Creature Properties` image is retained only as field-selection
readback; it is not used as the viewport proof.

## Remaining boundary

This packet is not an AUR-S07 runtime profile and is not an NWN proof.  The
central vertical-slice geometry observer still requires an independently
validated entry-surface observation before it can emit an immutable runtime
profile.  Until that exact profile binds module, ordered HAK, Area, entry, and
fixture to a fresh NWN capture and engine log, **NWN runtime remains
`missing`**.
