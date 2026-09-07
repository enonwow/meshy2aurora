# `m2assmmod221.mod` — superseded creature-fixture module

Toolset module name: `Meshy2Aurora Sandstorm Mask Helmet 221`

Exact Area name: `Sandstorm Mask Helmet Test 221`

## Status

The owner-supplied Meshy model was materialized as one native NWN
`MODEL_TYPE1` helmet candidate. Offline construction and semantic readback
passed, but this module included an equipped Human creature fixture. The owner
subsequently required a palette-only module with no creature, so this fixture
module is superseded by `m2assmpal221.mod`.

- Candidate status: `superseded_fixture_module`
- `modelVisibility`: `not_tested`
- `proofCompleteness`: `failed`
- Native NWN installation: `completed_byte_identical`

## Owner-observed Toolset lane failure

On 2026-09-04 the owner opened this exact installed module. Toolset displayed
Area `Sandstorm Mask Helmet Test 221` and then raised an access violation in
`nwtoolset.exe` while the creature fixture was present. This is a lane failure,
not evidence that the helmet model is invisible.

- Evidence:
  `documentation/evidence/tlc-sandstorm-mask-helmet-221-toolset-access-violation-2026-09-04.png`
- Evidence SHA-256:
  `c3689b6dece132dea7838d8944dda9c1522cb78ae72dfafe24974c1a3d7d2da3`
- Owner requirement: no creature; expose only the helmet blueprint in the Item
  custom palette
- Replacement handoff:
  `documentation/evidence/tlc-sandstorm-mask-helmet-palette-only-221-v1-ready-2026-09-04.md`

## Exact candidate lineage

- Canonical source: `sample-3d/tlc-sandstorm-mask-meshy-0904135727-v1/source-p300k.glb`
- Source SHA-256: `f9c694eea90d8b0dedca7434c732e9cd4d49ac8be517c0d0f1e48c3ea5b49577`
- Output root: `proof-output/tlc-sandstorm-mask-helmet-221-v1-20260904`
- HAK: `m2assmh221.hak`
- HAK SHA-256: `d6f34a55f1d845aa86fe35cdec4df3cd7c0e9494daf26bb8b028307abfbd84ed`
- Module: `m2assmmod221.mod`
- Module SHA-256: `4dcf3dd2150a959e58f9a8094008bb604eab5cd095d2753d57c0705128f3be0c`
- Model: `helm_221.mdl`
- Model SHA-256: `578c5fb940469ff31cf4aae9a50c02e497e7e92dca7c4d33328fb0c7e7f535be`
- Diffuse texture: `helm_221.tga`
- Diffuse SHA-256: `06c2f89a53869e06c19b273c7e93edde12b3babaf74099a01bb434e5509b138d`
- Inventory icon: `ihelm_221.tga`
- Icon SHA-256: `a63e1bdd58641e55387d0f76905e97b3fa9e93d526df492a156fc3060694fba4`
- Item blueprint: `m2assmask221.uti`
- UTI SHA-256: `962c1ed61e7e624c8128ec65c6284a1a7e54e04bc7c7f9b21fc1f9148d47bca7`

## Native Item binding

- Effective `baseitems.2da`: row `17`, `ModelType=1`, `ItemClass=helm`,
  `GenderSpecific=0`, equipment slot `1`
- Effective 2DA source SHA-256:
  `b2ba08aa55c185642c8973859b9d0dbf486bc8411a630a0a10a9f77e84d701c6`
- Helmet variant: `221`
- Model classification: `4` (`CHARACTER`, retail helmet parity)
- Fixture creature: `m2assmcre221`, human appearance row `6`
- Fixture placement: `x=10.0`, `y=14.5`, `z=0.0`
- Ordered module HAK list: `m2assmh221`

## Geometry and fit checks

- Triangles: exactly `300000` of the shared `300000` product budget
- Deterministic render segments: `15`
- Largest segment: `65523` index entries, below the `65535` binary boundary
- MDL inspection: `17` declared/parsed nodes, no unsupported families and no
  diagnostics
- Uniform scale: `0.14936666190624237`
- Translation: `[0.0, 0.0033032000064849854, -0.06474350392818451]`
- Rotation quaternion: `[0.0, 0.0, 0.0, 1.0]`
- Independent-axis scaling: not applied
- Fitted bounds min:
  `[-0.09702057391405106, -0.10513239353895187, -0.06474350392818451]`
- Fitted bounds max:
  `[0.09702057391405106, 0.11173879355192184, 0.21952541172504425]`
- Provisional cowl boundary check: pass

## Independent archive readback

The generated HAK parses as HAK V1.0 and contains exactly three candidate
resources: `helm_221` TGA, `helm_221` MDL and `ihelm_221` TGA. The generated
module parses as MOD V1.0 and contains the expected seven ARE/GIT/GIC/UTI/UTC/
IFO/FAC resources. Every payload hash matches `manifest.json`. The module IFO
reads back the one ordered HAK resref `m2assmh221`.

The conservative namespace scan covered `350554` relevant resource keys from
the retail key, current override and all local HAKs. Variant `221` and all
candidate resrefs were absent before materialization.

## Native installation

After the owner showed that Toolset could not find `m2assmmod221`, the exact
frozen artifacts were installed on 2026-09-04. Both destination files were
absent before the copy; nothing was overwritten or replaced. Post-copy size
and SHA-256 checks proved byte-identical equality with the canonical sources.

- Module destination:
  `C:/Users/enonw/Documents/Neverwinter Nights/modules/m2assmmod221.mod`
- Module SHA-256:
  `4dcf3dd2150a959e58f9a8094008bb604eab5cd095d2753d57c0705128f3be0c`
- HAK destination:
  `C:/Users/enonw/Documents/Neverwinter Nights/hak/m2assmh221.hak`
- HAK SHA-256:
  `d6f34a55f1d845aa86fe35cdec4df3cd7c0e9494daf26bb8b028307abfbd84ed`
- Agent Toolset/NWN control: not performed

## Verification run

- `cargo test -p m2a-core --test two_da --test item --test item_part --test item_icon --test item_uti --test item_package --test mdl_writer`: `88 passed`, `0 failed`
- `cargo check -p m2a-wasm`: pass
- Canonical workspace guard: pass
- Canonical Meshy asset-layout guard: pass

## Owner proof handoff

Install the exact HAK and MOD above, open `m2assmmod221.mod`, load Area
`Sandstorm Mask Helmet Test 221`, and inspect the equipped mask on
`m2assmcre221` in Toolset and NWN. Report each lane independently as visible or
not visible. Do not allocate a new model, variant, HAK or module from a tooling,
capture or setup failure; complete proof for this exact hash lineage.
