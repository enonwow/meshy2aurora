# `m2a_h2r42.mod` — ready for owner proof

Toolset module name: `m2a_h2r42`

Exact Area: `m2a_h2a42`

Status: `ready_for_owner_proof`

## Result carried forward

The exact r41 lineage has a fresh owner-bound NWN result
`modelVisibility=not_visible`, `proofCompleteness=verified`. Its durable result
record is
`documentation/evidence/m0-r41-owner-nwn-visual-result-2026-07-24.json`,
SHA-256
`38ca22bf01bb264461a5c1419071cf18fff2b6762569a313004d1b8ee906aa6b`.
This admitted one new iteration.

## New Meshy source

H2 is a newly generated Meshy humanoid, not an M0 reskin:

- source:
  `sample-3d/h2-clockwork-sentinel-1500/source.glb`;
- path amendment 2026-07-27: canonical relocation only; source bytes and
  SHA-256 used by this evidence did not change;
- visual identity: turquoise enamel and burnished copper clockwork sentinel
  with an amber chest core;
- SHA-256:
  `f8cf0af21c8143a62b64c490a81dd2855ad3c3f9865922e3854f84b714dec3a3`;
- `8,234,708` bytes, `1,543` triangles;
- one skinned mesh, one 24-joint skin and one Meshy Idle animation in the
  source GLB.

The exact Meshy task lineage and source readback are recorded in
`documentation/evidence/h2-meshy-clockwork-sentinel-generation-2026-07-24.json`.

## Diagnosed correction

Facts:

1. r41 proved that the 20-piece, bone-parented rigid M0 surface remains absent
   in NWN even after projecting the full 45-node topology into every type-5
   state.
2. The project-owned H1 v20 witness proved that NWN can draw a model emitted by
   this writer when the surface is one rigid mesh and local states use the
   historical type-0 rig-only family.
3. The H1 v20 surface was attached to the animated skeleton root, so its entire
   rigid body could be displaced or deformed as one object.
4. The exact H2 source contains harmless exporter scale noise up to about
   `1.1e-5` on mapped joints. The mapper previously rejected values above
   `1e-5`, while its own rigid target check already uses `1e-4`.

Implemented delta:

- the mapper now accepts only near-unit source scale within `1e-4`; a real
  non-unit scale such as `2.0` remains a fatal error;
- H2 uses its own 24-joint hierarchy and mapped Idle motion;
- one new controllerless identity Aurora Root is inserted;
- the old skeleton-root bind transform is baked into the H2 surface;
- the complete surface is emitted as one rigid TriMesh attached directly to
  the new Aurora Root, so bone animation cannot tear or move the surface;
- seven required direct-creature state names are emitted using the exact
  project-owned runtime-positive type-0 rig-only family;
- the module includes a stock Hook Horror control alongside H2 so a single
  owner frame can distinguish model/resource failure from fixture/module
  failure.

This is intentionally a visibility candidate. The H2 source retains its rig
and animation provenance, but the r42 surface itself is rigid and therefore is
not yet a visual skin-deformation proof.

## Frozen lineage

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `m2a_h2p42.mdl` | 600,712 | `20fe29854f01880b0b9d5b4f0a5e703c76f858eeebb01a00ea3a29a4913969d4` |
| `m2a_h2t42.tga` | 12,582,956 | `03169b1493ed4b2f5269ed6fa611aef787e219cddd08661c9135a7f82191c167` |
| `appearance.2da` | 6,901,367 | `134f3e8a33c1103716b0b2b6f0e68f4686aa7144c1da2bb7447785a7627fff65` |
| `m2a_h2r42.hak` | 20,085,291 | `6d50bd1f784a8d1b10b36b09e32565781674def6b4f4367d803ac7acced6da69` |
| `m2a_h2r42.mod` | 20,284 | `c22741a72ab19a4b6e43620902ec9530e73d251c67de2beb23a2fef60e2b3c10` |
| lineage contract | — | `7f06b7d1f33e134963ec110ca20242e65f37a8c959e0817b0fa62c997a0d2782` |

Canonical output:

`C:\Projects\meshy2aurora\proof-output\h2-r42-root-rigid-type0-20260724`

The MOD and HAK were installed with absent-target, create-new semantics and
post-copy byte-identical verification:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_h2r42.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_h2r42.hak`.

## Exact scene

- ordered HAK list: `m2a_h2r42`;
- H2 Appearance row: `15100`, `MODELTYPE=S`, `RACE=m2a_h2p42`;
- H2 fixture: `h2_fixture`, display name
  `Meshy H2 turquoise clockwork sentinel`, position `(10.0, 14.5, 0.0)`;
- stock control: `stock_control`, display name `Stock Hook Horror control`,
  Appearance row `102`, position `(7.0, 14.5, 0.0)`;
- player entry: `(10.0, 10.0, 0.0)`, facing north.

Expected visual distinction: H2 is the turquoise/copper sentinel in the
centre; the stock Hook Horror is offset to the player's left.

## Offline verification

- full nonignored `cargo test -p m2a-core`: passed;
- exact H2 r42 deterministic build/replay test: passed;
- model own-readback: one rigid mesh, zero SkinMesh nodes, 1,543 triangles,
  25 rig nodes including the dedicated Aurora Root;
- seven local states: type `0`, mesh omitted, rig identity projected;
- HAK exact resource readback: passed;
- complete two-fixture MOD exact GFF/ERF readback: passed;
- native installed MOD and HAK hashes: byte-identical to canonical outputs.

No Aurora Toolset or NWN session was started or controlled. Visual axes remain:

- Toolset: `modelVisibility=not_tested`,
  `proofCompleteness=missing`;
- NWN: `modelVisibility=not_tested`,
  `proofCompleteness=missing`.
