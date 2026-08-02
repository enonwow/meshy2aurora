# `m2a_tlcs1_mod.mod` - ready for owner proof

Toolset module name: `The Last City - Ship Under Construction Demo`

Area name: `The Last City Shipyard Construction Demo`

Status: `ready_for_owner_proof`

## Goal and result

The owner selected four consistent concept views for a new The Last City ship
under construction and directly authorized one Meshy Multi-Image generation at
approximately 150,000 polygons, followed by the complete Meshy2Aurora static
Placeable pipeline. This is a distinct asset and does not mutate or replace the
existing wooden-sailboat proof lineage.

One paid Meshy task succeeded. The resulting GLB was downloaded through the
local Bridge, hash-verified, registered under the canonical `sample-3d` root,
and passed the product's normal 300,000-triangle admission without an oversized
exception.

## Meshy generation identity

- asset: `tlc-ship-under-construction-s1-p150k-v1`;
- source mode: `MULTI_IMAGE` with four owner-selected JPEG inputs;
- profile: `S1-static-prop/v1`;
- model: `meshy-6`;
- local Bridge run: `51a32abd-9fe9-46de-ba15-1c8bddd0c689`;
- Meshy task: `019fb9aa-379a-7024-9e10-8418206ac9b7`;
- requested target: `150000` triangles;
- measured source: `152574` triangles;
- source bytes: `11498076`;
- source SHA-256: `61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`;
- balance before/after: `586 / 556`;
- credits spent: `30`;
- second model generated: `false`;
- manual mesh or texture edits: `false`.

The request used triangle remesh, 2K texturing, PBR disabled, image enhancement
disabled, baked-light removal enabled, moderation enabled and GLB-only output.
The persisted provenance contains no API key, session token or signed URL.

## Exact input images

| Role | File | SHA-256 |
|---|---|---|
| top | `concept-top.jpg` | `0530e60430361ecccf935fb2cbbc8000ee56dbdab8bfba907f48c0515c20c441` |
| stern | `concept-stern.jpg` | `f907e64bf290291a3f808625ddf0b657229bc4899ca16446d6e35541de8e3fa4` |
| side three-quarter | `concept-side-three-quarter.jpg` | `5ac81677a92eea70c83e870227761f89941295d8b9c78912ad43e334aba7ec48` |
| bow | `concept-bow.jpg` | `a03ed01fa916f51f7abc927c5d918a919a72c9bf097f246bfab0a247f92640ae` |

## Geometry, scale and material

- source triangles: `152574`;
- Aurora output triangles: `152574`;
- truly invalid triangles removed: `0`;
- aggressive cleanup enabled: `false`;
- binary-MDL render streams: `7`;
- largest stream: `21845` triangles;
- uniform scale: `8.0`;
- dimensions: `15.173519 x 8.754520 x 6.288216 m`;
- grounded minimum Z: `0.0`;
- source materials/textures: `1 / 1`;
- generated diffuse texture: `2048 x 2048` RGB TGA;
- source/TGA luminance percentiles P05/P50/P95: `48/81/109` in both files.

The identical source and TGA luminance distribution confirms that the pipeline
did not darken the new texture. Compared with the earlier sailboat source's
median luminance of `55`, the new median of `81` is materially more readable,
while the final visual judgement remains owner-owned in NWN.

## Exact candidate identity

- proof packet: `C:\Projects\meshy2aurora\proof-output\tlc-ship-under-construction-placeable-v1-20260731`;
- MOD: `m2a_tlcs1_mod.mod`;
- MOD bytes: `13423`;
- MOD SHA-256: `e02f71e5ba8b5f92ccf76486450ab4bdc78e8b019893fd364d1c109959eb9e83`;
- HAK: `m2a_tlcs1_hak.hak`;
- HAK bytes: `27750143`;
- HAK SHA-256: `80ebb8bca45a671754bc39a6db690da0251963b128efbb25efbbf07cc2f8ff12`;
- model resref: `m2a_tlcs1_mdl`;
- model bytes: `12146684`;
- model SHA-256: `a8f471e050fd02d8cdb54a15c6ede4d1bc13641fb022b4e669de8dc2087b99d1`;
- texture resref: `m2a_tlcs1_tex`;
- texture SHA-256: `92297351fa02b6731a7a340039ab41b3744424acfb7d195454d7843d74f209b8`;
- blueprint resref: `m2a_tlcs1_utp`;
- object tag: `m2a_tlcs1_building_ship`;
- Appearance row: `16500`;
- ordered HAK list: `m2a_tlcs1_hak.hak`;
- placement: `(10.0, 14.5, 0.0)`, bearing `0.0`.

## Verification

| Check | Result |
|---|---|
| canonical workspace guard | PASS |
| Meshy asset-layout guard | PASS |
| Bridge contract tests | PASS - 21 passed |
| Studio tests | PASS - 236 passed |
| Studio TypeScript typecheck | PASS |
| exact GLB hash and header verification | PASS |
| static Placeable authoring inspection | PASS - no warnings or fatals |
| exact source materialization and MDL/HAK/MOD readback | PASS |
| `m2a-core --lib` | PASS - 89 passed, 2 ignored |
| Placeable integration | PASS - 13 passed, 1 ignored |
| materializer clippy with `-D warnings` | PASS |

## Native installation

Both native destinations were absent before installation. The exact generated
files were copied without overwrite and verified byte-identical afterward:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_tlcs1_mod.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_tlcs1_hak.hak`.

No Aurora Toolset or NWN process was started, adopted, controlled or captured.

## Proof state

- Toolset `modelVisibility`: `not_tested`;
- Toolset `proofCompleteness`: `missing`;
- NWN `modelVisibility`: `not_tested`;
- NWN `proofCompleteness`: `missing`.

The owner performs the final visual and performance proof using the exact
module, Area and object identity above.
