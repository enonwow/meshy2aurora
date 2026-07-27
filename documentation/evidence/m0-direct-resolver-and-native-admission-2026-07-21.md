# M0: direct resolver and native-admission checkpoint — 2026-07-21

## Scope and current status

This record concerns the current static Meshy M0 candidate.  It distinguishes
offline/package facts from live Toolset and NWN proof.

- Offline M0 package and structural module binding: `verified`.
- Fresh current-candidate Toolset viewport (`TScrollBox`) proof: `missing`.
- Fresh current-candidate NWN runtime proof: `missing`.

No `nwtoolset.ini`, MRU, Toolset setting, or game configuration was changed in
this checkpoint.

## Candidate identity

Generated directory:
`C:\Projects\meshy2aurora\proof-output\m0-direct-resolver-gate-20260721\generated`

| Artifact | SHA-256 |
| --- | --- |
| `m2a_m0p01.mdl` | `cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5` |
| M0 TGA | `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` |
| `m2a_m0_proof.hak` | `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6` |
| `m2a_bm0p1.mod` | `37def5947b47a1c7c499c9e8c96a80f7effbf49e870af0ad395d09fb72cf10b3` |

The package's physical `appearance.2da` row is `848`, with
`LABEL=M2A_M0_MESHY_RIGID`, `MODELTYPE=S`, and `RACE=m2a_m0p01`.  The generated
module describes Area `m2a_bm0a1`, a complete 2 x 2 Area, entry `[10,10,0]`,
and the fixture intent `[10,14.5,0]`.

## Resolver gate

### Evidence and diagnosis

Aurora decompilation, xoreos source inspection, and public NWN model material
all support the same resource-resolution chain for this direct-creature path:

`GIT Appearance_Type -> physical appearance.2da row -> MODELTYPE/RACE -> MDL -> mesh texture resrefs`.

The former verifier could compare a package to values carried by that same
package without rejecting every self-consistent but semantically wrong resolver
mapping.

### Implemented, test-first gate

`crates/m2a-core/src/model_pipeline.rs` now requires, for M0:

1. `MODELTYPE` exactly `S`;
2. `RACE` exactly `m2a_m0p01`;
3. the direct M0 mesh texture-resref list exactly `[m2a_m0t01]`.

The requirement is checked both while materialising the packet and during its
semantic verification.  Negative tests construct self-consistent packages with
`MODELTYPE=P`, a foreign `RACE`, and a foreign texture resref, and require a
stable rejection for each.  This is an implementation decision based on the
documented resolver evidence, not live renderer proof.

## Offline verification run

Executed in the canonical workspace on 2026-07-21:

```text
cargo test -p m2a-core --test model_pipeline              # 14 passed
cargo test -p m2a-core --test mdl_writer                  # 31 passed
cargo test -p m2a-core --test binary_m0_vertical_slice_module  # 2 passed
git diff --check                                           # passed
```

The central binary bootstrap structural validator accepted the current profile,
and the central native-geometry route accepted its dry-run plan.  These results
verify serialization/binding only; neither result is a viewport or runtime
render claim.

## Native-admission observation

The only existing central binary-module native route starts Toolset cleanly and
must obtain ownership through its Welcome dialog.  Its read-only admission
preflight stopped because the current MRU points at a different historical
module (`m2a_m0v12.mod`) rather than the fresh `m2a_bm0p1.mod`.  The route
correctly failed closed; no MRU or INI write was attempted.

An existing user HAK and module already occupy the otherwise tempting names:

| Existing path | Existing SHA-256 | Current generated SHA-256 | Result |
| --- | --- | --- | --- |
| `...\\hak\\m2a_m0_proof.hak` | `7211c1a016c2b36320f7ce2a399d2833b8f2d7901d2896261d01a93161600200` | `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6` | different; do not overwrite |
| `...\\modules\\m2a_bm0p1.mod` | `215df808dc5617f48c7d10e1f07176b4264f3c615f0bb001f72ef55ea7992d76` | `37def5947b47a1c7c499c9e8c96a80f7effbf49e870af0ad395d09fb72cf10b3` | different; do not overwrite |

No `nwtoolset.exe` or `nwmain.exe` process was present at the checkpoint.

## Next bounded action

Before a live run, resolve the physical Appearance_Type row against the current
generator and Aurora evidence.  Then, if the result remains `848`, use the
central vertical-slice bootstrap only with a new absent HAK/module resref and
destination.  It must retain the ordered-HAK, native-save, GIT/IFO readback,
geometry, `TScrollBox`, and AUR-S07 gates.  A successful creation alone is not
Toolset proof or NWN proof.

## Live r21 continuation — 2026-07-21

### Facts observed through the canonical Toolset route

The bounded fresh target was created without overwriting an existing user
module or HAK:

| Field | Observed value |
| --- | --- |
| Module | `m2a_m0r21.mod` |
| Area | `m2a_m0a21` (MicroSet, Tiny / 2 x 2) |
| HAK, in saved native order | `m2a_m0r21` |
| HAK SHA-256 | `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6` |
| HAK persistence checkpoint | `saved_verified` |
| Module SHA-256 after native creature save | `d9286d469e049f032cf70e79b763ca4d314cae13ca414b8230b6b24bb81e49ec` |
| Saved entry | `[10, 10, 0]` |
| Saved fixture | `meshy-m0-r21`, template `nw_dwarfmerc001`, `Appearance_Type=848`, position `[5.0642, 14.9358, 0]` |

The build result was independently read as `No errors found` with `Build`
disabled and `Done` enabled.  Only the named `Done` continuation was used;
there was no second build and no HAK retry.  The subsequent exact native HAK
inspection and one native Save yielded the ordered-list and clean-module
readback above.

No `nwtoolset.ini`, MRU, Toolset configuration, or game configuration was
written.

### Fresh viewport observation

The following artifact is a fresh physical-pixel capture of the validated
`TScrollBox`, not a full-frame desktop fallback:

`proof-output/m0-r21-central-bootstrap-20260721/m0-r21-tscrollbox-observation.png`

Its companion JSON records the exact r21 module title, Area viewer
`m2a_m0a21`, a 1275 x 839 viewport on `\\.\DISPLAY1` with `primary=false`,
and capture after `WindowFromPoint` ownership validation.  Visual inspection
shows the textured 2 x 2 MicroSet Area and a small pale silhouette in the
upper-left tile at the saved fixture's approximate position.  This is a
**verified observation artifact**, but it is deliberately not claimed as a
verified M0-model proof: it does not yet bind a focused selected object and
the geometry proof gate below has not passed.

### Exact remaining proof gate

The central native geometry observer emitted
`proof-output/m0-r21-central-bootstrap-20260721/native-geometry-blocker.json`:

- status: `failed`;
- blocker: `area_geometry_invalid`;
- failures: `area_geometry_invalid` and
  `entry_surface_native_observation_unavailable`;
- there was no `invalid locations` or `inaccessible objects` dialog;
- the Toolset frame was clean after the native save.

The immediate native save had a title-clean and file-size/time transition, but
its SHA was deliberately deferred while the owned Toolset process retained the
file lock.  The central geometry observer therefore recorded
`nativeSave.areaWasSaved=false`.  Independently, the current central contract
requires a validated entry-surface observation before it will issue a geometry
profile.  No new Adjust Location transport, direct leaf atom, shared-tooling
patch, or guessed camera/fixture mutation was attempted: none is authorized
by the present public continuation for this non-error state.

Consequently the current proof status is:

- package/resource chain and saved GIT/IFO identity: `verified`;
- fresh Toolset `TScrollBox` observation: `verified` (observational only);
- final Toolset M0 visual proof: `missing`;
- NWN runtime proof: `missing`.

The next valid continuation is a canonical route that can establish the
native entry/fixture-surface geometry profile for this exact saved module;
only after that profile exists can the dedicated AUR-S07 runtime route accept
this module and Area.  Starting NWN directly or treating the current image as
runtime evidence would not satisfy that contract.

### Model-size and static-creature diagnosis

An independent read of the exact binary resource
`m2a_m0p01.mdl` reports one rendered mesh (`meshType=3`) with 2,380 vertices,
texture `m2a_m0t01`, and mesh bounds:

```text
min = [-0.54365003, -0.31950998, 0.0]
max = [ 0.54365003,  0.31950998, 1.892962]
radius = 1.8960401
```

Therefore the source model is approximately `1.09 x 0.64 x 1.89 m`.  On a
whole-Area capture whose four tiles occupy only about 400 pixels across, this
is expected to appear as a small figure rather than fill a tile.  The visible
pale object in the upper-left tile is consistent with the saved M0 fixture's
position and dimensions; it is not evidence of an absent mesh.  It remains an
observation rather than final proof until the selection/resource and geometry
gates are both closed.

`MODELTYPE=S` is also the correct *single-MDL simple-creature* family for this
resource, not an error to be changed to `P`: the current NWN appearance
reference describes `S` as one MDL using one TGA/DDS texture, whereas `P` is
part-based.  Sources consulted for this conclusion are the local Aurora
decompilation/readback, the xoreos NWN1 MDL documentation index
(`https://github.com/xoreos/xoreos-docs`), and the current NWN EE
appearance.2da reference (`https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da`).

### Focus-on-object capability check

Aurora decompilation contains the exact UI label `Focus on Object` at string
`00fcb5b2`.  The current shared standards, however, do not publish a verified
central route that binds that action to a stable command/control and the exact
r21 selected creature.  The existing AUR-S02-01 public standard proves object
selection, but its available implementation is a leaf atom rather than an
authorized r21 vertical-slice continuation.  Sending a guessed command merely
because the label exists would violate the no-new-transport rule and could not
be accepted as proof.  This has been recorded as a capability gap; no shared
tooling was changed.

### Runtime resource-conflict preflight

Read-only inspection of the installed r21 HAK found exactly three entries:

| Resref | NWN resource type |
| --- | ---: |
| `appearance` | 2017 (`2DA`) |
| `m2a_m0p01` | 2002 (`MDL`) |
| `m2a_m0t01` | 3 (`TGA`) |

The archive SHA-256 matches the saved module's ordered-HAK checkpoint. The
user override directory contained only unrelated `vdr_star_p` and
`vfx_asn_mk4` assets; no `appearance`, `m2a_m0p01`, or `m2a_m0t01` file was
present there. No `userpatch` or installation-level override directory was
present. This rules out the obvious local same-resref override for the r21
test; it does not prove NWN has loaded the module or invalidate the required
fresh runtime packet.

### Texture-alpha negative check

The exact r21 `m2a_m0t01.tga` was read as a 12,582,956-byte, uncompressed
type-2 TGA: 2048 x 2048, 24-bit pixels, with descriptor `0` and zero alpha
bits.  It therefore carries no alpha channel that could make the model
transparent in NWN.  Together with the MDL readback (`render=1`, no
transparency hint, opaque vertex colour), this rejects texture alpha as the
current explanation for a model that is visible in Toolset but lacks an NWN
runtime proof.  No exporter change follows from this negative result.

### Corrected binary M0 fixture contract

The fresh r21 Area was parsed from its saved native `ARE` rather than inferred
from a screenshot. Its renderable fixture is a `2 x 2` `tms01` (MicroSet)
Area with this exact serialized `Tile_List` order:

```text
[(Tile_ID=12, Tile_Orientation=2),
 (Tile_ID=12, Tile_Orientation=1),
 (Tile_ID=12, Tile_Orientation=3),
 (Tile_ID=12, Tile_Orientation=3)]
```

Every record has `Tile_AnimLoop1/2/3=1`. This is the first native-ARE source
bound both to a fresh Toolset `TScrollBox` capture and to observed M0
rendering. It supersedes the historical synthetic `tdc01` / tile `5` /
orientation `0` x4 fixture only for newly generated binary M0 vertical slices;
historical evidence remains unchanged.

The binary generator and its parser-backed validator now emit and require that
exact `tms01` sequence, with the standard entry `[10, 10, 0]` facing `[0, 1]`
and one fixture `[10, 14.5, 0]`. Targeted tests passed after the change:

```text
cargo test -p m2a-core --test binary_m0_vertical_slice_module  # 2 passed
cargo test -p m2a-core --test model_pipeline static_meshy_m0 --quiet  # 4 passed
```

This is a pipeline correction, not an NWN-success claim. A newly materialized,
uniquely named package still requires a fresh Toolset proof packet and a
separately bound NWN runtime capture.

## Iteration-control audit and superseding decision

The r21 evidence did not prove a visual model failure. It recorded the saved
module/HAK/resource chain, `Appearance_Type=848`, a saved fixture, and a fresh
`TScrollBox` containing a small visible silhouette. The owner also confirmed
that the model had remained visible in Aurora from r1 onward. Therefore r21
has `Toolset.modelVisibility=visible`; its geometry/focus packet was
incomplete, and `NWN.modelVisibility=not_tested`. Calling either visibility
result `missing` was a category error. They required continuation on r21, not
a replacement candidate.

The later r22 HAK is byte-identical to r21: both have SHA-256
`25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6`.
R23 failed before model evaluation because the palette placement point was
outside the rendered tile footprint, and r24 did not reach creature placement
or viewport capture. Consequently r22, r23, and r24 were three unjustified
container iterations: none was admitted by a fresh Aurora or NWN visual model
failure.

This incident is now governed by the HARD STOP in root `AGENTS.md`,
`documentation/PROJECT_RULES.md`, the M0 runtime fixture standard, and shared
Aurora skill suite `1.0.10`. The superseding continuation rule is:

1. finish exact-object selection/readback and fresh `TScrollBox` proof on the
   existing admitted candidate; `Focus on Object` and camera framing are
   optional aids under the owner decision of 2026-07-21;
2. if the model is visible, run NWN using the same exact artifact lineage;
3. admit another `rNN` only after a candidate-bound Toolset or NWN visual
   `failed` result plus a diagnosis and minimal intended artifact delta.

`proofCompleteness=missing`, geometry, placement, HAK/build/save, file-lock,
capture, timeout, or automation failures do not admit a new iteration. Failure
or omission of optional focus/framing does not invalidate an otherwise bound
capture. Only `modelVisibility=not_visible` in Toolset or NWN admits a new
iteration.

## Fresh r21 NWN result and admitted placement-only successor

The exact saved r21 module was launched once through the verified native
Toolset `Build -> Test Module` command (`Build` position `2`, command ID
`121`). The resulting single `nwmain` process loaded `m2a_m0r21`; the fresh
engine log contains `Loading Module: m2a_m0r21`. A fresh full NWN window
capture was saved as:

`proof-output/m0-r21-nwn-runtime-20260721/nwn-r21-runtime.png`

The frame shows the player at the declared entry point and a normal rendered
MicroSet Area, but it does not show the declared M0 fixture. This candidate-
bound result is `NWN.modelVisibility=not_visible`; it is not evidence that the
MDL payload is wrong. The saved r21 GIT places the fixture at
`[5.0642, 14.9358, 0]`, while the fixed runtime-fixture contract requires
`[10, 14.5, 0]`, directly in front of entry `[10, 10, 0]`.

One canonical AUR-S02-03 attempt read the requested `10.00 / 14.50` values
inside the Adjust Location atom, but the native dialog restored
`5.06 / 14.94` before and after `Apply`; the module remained unchanged and the
dialog was cancelled. No further Adjust Location transport is admitted for
this unchanged state.

The next module iteration is therefore admitted with one minimal delta only:

- materialize a fresh uniquely named MOD/Area from the existing generator;
- set the single fixture to the already documented `[10, 14.5, 0]` contract;
- reuse the byte-identical installed `m2a_m0r21` HAK and keep Appearance row
  `848`, MDL `m2a_m0p01`, TGA `m2a_m0t01`, and `appearance.2da` unchanged;
- repeat Toolset Focus Object proof and then NWN Test Module on that exact
  lineage.

No Toolset/NWN INI, MRU, or user setting is part of this delta.
