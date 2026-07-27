# Last City `c_squirrel` reference control v1 — 2026-07-18

Status: `PACKAGE-AND-INSTALL-VERIFIED / TOOLSET-VISIBILITY-VERIFIED / NWN-VISIBILITY-FAILED`.

## Purpose and boundary

This is an external-reference control requested by the owner. It isolates the
container and selection path:

```text
Last City source copies -> Meshy2Aurora 2DA/HAK/MOD writers -> Aurora -> NWN
```

It is **not** a proof of the Meshy2Aurora MDL writer: `c_squirrel.mdl` is an
unmodified ASCII resource from `lc_hd_animals.hak`. The source copies were
explicitly authorized only for this audit/proof and remain under
`proof-output`; they are not product inputs, fixtures, or runtime
dependencies.

## Materialized package

Harness:
`crates/m2a-core/examples/materialize_lc_squirrel_reference_proof.rs`.

The harness copies the complete Last City `appearance.2da` row `15216` except
for `LABEL`, preserving its creature semantics such as `MODELTYPE=S`,
`RACE=c_squirrel`, `MOVERATE=FAST`, size, movement, head, and targeting
fields. It appends physical row `15103` to the supplied M6 baseline table:

| Field | Value |
| --- | --- |
| `LABEL` | `M2A_LC_SQUIRREL_REF` |
| `RACE` | `c_squirrel` |
| `MODELTYPE` | `S` |
| `Appearance_Type` in generated UTC/GIT | `15103` |

The generated HAK includes every texture referenced by the model:

| Resource | Type | SHA-256 |
| --- | ---: | --- |
| `c_squirrel` | `2002` MDL | `e381bd13f4f10ab9c33a9751f409c061b6579e2a042d12170d74e23f5b669767` |
| `c_squirrel` | `2033` DDS | `c26e36494debc0d8eb3491d01eadfd28ef24cb6b36667caf28e1afe149d4f8c1` |
| `c_badger` | `3` TGA | `993467cf6632d918a355074df4b9170ed57aff6d0a74fdb713eae33839a2537b` |
| `c_badger` | `2033` DDS | `d6e1b01dc6a797e8e3683461bc243a2b6bbf509508f543bcc4c5ff62ab7ed2eb` |

Own HAK and MOD readback both returned `PASS`. Generated outputs:

| Output | SHA-256 |
| --- | --- |
| `package/m2a_codex_aproof.hak` | `0506eacab54c785d3f1b1dd93f346272c677b8f7b36e802cefcbc003d0da008f` |
| `package/m2a_codex_aproof.mod` | `7ad62134fc301044ae8341b8965a1d969432d6396492c61965cd3f800f383eab` |
| `package/appearance.2da` | `c494dd15557eadaec572248b36d1e900b8d51a3dd8056be6fbbc9345ce1e240a` |

The package manifest is
`proof-output/lc-c-squirrel-reference-control-v1/package/lc-squirrel-reference-proof-manifest.json`.

## Installation and readback

With no `nwtoolset`, `nwmain`, or `nwserver` process running, the package was
installed to the one canonical proof filenames:

```text
C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_codex_aproof.hak
C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_codex_aproof.mod
```

The target files read back with the generated hashes above. The prior
NeverBlender ASCII A/B package was copied before replacement to:

```text
proof-output/lc-c-squirrel-reference-control-v1/live/preinstall-m2a_codex_aproof.hak
proof-output/lc-c-squirrel-reference-control-v1/live/preinstall-m2a_codex_aproof.mod
```

The exact pre/post installation identities are in
`proof-output/lc-c-squirrel-reference-control-v1/live/install-readback.json`.

## Aurora-first proof and result

The approved read-only Aurora preflight passed for this exact module and Area:

| Gate | Result |
| --- | --- |
| Toolset/NWN/server processes | none |
| `temp0` recovery directory | absent |
| Proof display | `\\.\DISPLAY1`, non-primary |
| Exact module | `m2a_codex_aproof.mod` |
| Exact Area | `m2a_caproof_area` |

The owner explicitly authorized a narrowly scoped profile addition to the
existing standard session runner, `m2a-lc-squirrel-reference-control`. It
opened the installed, hash-pinned module in Aurora on `\\.\DISPLAY1`; no
direct launcher, global input, or altered Aurora/NWN configuration was used.

The TreeView identity readback and the physical screenshot agree on the
following placement:

```text
Areas
  Codex H1 animation proof area
    Creatures
      Codex Meshy H1 animation proof creature
```

The creature was selected from that exact TreeView identity. Native, read-only
camera readback (anchored to the decompiled `TfrmViewerArea` camera binding)
then confirmed a view centred at `(13.75, 10.00)`, adjacent to the GIT
placement `(14.00, 10.00)`, at `zoom=21.103367` and `pitch=69` degrees. The
camera was controlled only with the established, targeted Toolset toolbar
adapter; every captured viewport was on non-primary `\\.\DISPLAY1`.

The initial medium-distance Toolset frames showed only a small green selection
marker. That was not a sufficient visual threshold for this `animationScale`
`0.5` asset. The established camera adapter then kept the selected creature
centred and reduced its distance to `zoom=10.293026`, with `pitch=69` degrees.
At that measured camera pose the final accepted, textured viewport visibly
contains the small brown squirrel silhouette inside the exact green selection
marker. This was checked from both the complete Toolset frame and the
validated physical-pixel viewport crop:

```text
proof-output/lc-c-squirrel-reference-control-v1/toolset-session/viewport-gate/
  toolset-area-creature-centred-zoom8-pitchdown.png
  toolset-area-viewport-creature-centred-zoom8-pitchdown.png
  toolset-area-viewport-creature-centred-zoom8-pitchdown.json
```

The frame also records the expanded `Creatures` tree and selected creature;
the previous precise top-down frame is retained as
`toolset-area-creature-centred-zoom22.png`. A temporary uniform-white viewport
after a wheel experiment was rejected by the viewport validator and is **not**
used as evidence; a bounded native Toolset camera move restored a textured
scene before the accepted captures above.

Because the Toolset gate passed, the standard dynamic-menu readback verified
`Build` position `4`, `Test Module` position `2`, command ID `121`, enabled;
only then was that command posted. `nwmain` launched and its exact window was
captured on non-primary `\\.\DISPLAY1`:

```text
proof-output/lc-c-squirrel-reference-control-v1/live/
  nwn-test-module-initial.png
  nwn-test-module-approach-1.png
  nwn-test-module-approach-1-settled.png
```

The NWN captures show the loaded test location and player, but no visible
squirrel model. One bounded, validated click on clear floor was captured as a
secondary approach frame; it did not reveal the model. No result is inferred
from the former rejected white Toolset view.

**Result:** the Last City ASCII MDL, complete referenced textures, copied
appearance semantics, HAK, MOD, and creature placement are sufficient for
visible Toolset rendering, but the same reference creature is not visible in
the NWN runtime proof. This eliminates the package/2DA/Toolset selection lane
as the explanation for the original failure and strengthens the remaining
runtime-renderer investigation (model state/animation versus skin/mesh
runtime semantics).

## NWN Wiki follow-up: facts and next isolated test

The external NWN Wiki was consulted after the proof. Its information changes
one earlier hypothesis and supplies a narrower A/B experiment:

| Classification | Finding | Consequence for this control |
| --- | --- | --- |
| Fact | `appearance.2da` is loaded by client, server, and Toolset; its appearance ID is a `uint16` limited to `0..65535`. | Physical row `15103` is within the documented engine/network range; row width or this ID limit is not the explanation. |
| Fact | For `MODELTYPE=S`, `cpause1` is the standing idle; `cwalk` and `crun` are the movement animations. There is no documented `cstand` requirement. | The Last City source has these simple-creature states. A missing `cstand` is explicitly rejected as the explanation. |
| Fact | The Last City source is an ASCII MDL with skinmeshes. The client dynamically compiles ASCII models; the Wiki recommends `compileloadedasciimodels` only after the skinmeshed model is actually loaded and visible, writing output to `Documents\\Neverwinter Nights\\modelcompiler`. | ASCII-to-runtime compilation remains a distinct path from the binary Meshy writer and needs its own A/B. |

The authoritative next test is therefore **not** another `appearance.2da`
variant: after owner approval for the external output, load the reference
creature in NWN, use the game's `compileloadedasciimodels` command, package
only its resulting compiled `c_squirrel` into a fresh control HAK, and repeat
the exact Toolset/NWN proof. If only the compiled reference renders in NWN,
the remaining fault is in the runtime ASCII/skin compilation lane; if it still
does not render, the investigation returns to client resource resolution or
runtime visibility rather than the model writer.

Sources: [appearance.2da](https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da),
[Animations](https://nwn.wiki/spaces/NWN1/pages/38175170/Animations), and
[Models](https://nwn.wiki/spaces/NWN1/pages/38175602/Models).

## Verification performed

```text
cargo fmt --all --check
cargo test -p m2a-core --example materialize_lc_squirrel_reference_proof
```

Both checks passed. The harness test verifies that the complete 35-column
Last City squirrel row is represented and targets `RACE=c_squirrel`.
