# `m2a_tlcs2_mod.mod` — TLC shadow-adjacency V2 ready for owner proof

1. Exact test-module `.mod` filename: `m2a_tlcs2_mod.mod`.
2. Module name shown in Toolset: `Meshy2Aurora TLC Shadow Adjacency V2`.
3. Exact Area name: `Meshy2Aurora TLC Shadow Adjacency Test`.

Status: `ready_for_owner_proof`.

- `modelVisibility = not_tested`
- `proofCompleteness = missing`
- shadow runtime verdict: `not_tested`
- collision runtime verdict: `not_tested`
- Toolset/NWN were not started or controlled by the agent.

## Scope and owner authorization

This is one new immutable candidate admitted by the owner's direct
`do dziela` instruction after the agent stated that a new numbered TLC
iteration was required to test the shadow fix and would be installed into the
native NWN directories.

The candidate changes only the generated model lineage and its identities. It
reuses the exact three Meshy GLB sources from V1. The intended semantic delta
is:

`position-welded face adjacency across UV and hard-normal render seams`.

The frozen V1 MOD, HAK, models, textures and native copies were not modified.

## Exact candidate identity

| Component | Identity |
|---|---|
| MOD | `m2a_tlcs2_mod.mod` |
| Module resref | `m2a_tlcs2_mod` |
| HAK | `m2a_tlcs2_hak.hak` |
| HAK resref | `m2a_tlcs2_hak` |
| Area | `Meshy2Aurora TLC Shadow Adjacency Test` |
| Area resref | `m2a_tlcs2_ar` |
| Ordered HAK list | `m2a_tlcs2_hak.hak` |
| Candidate directory | `C:\Projects\meshy2aurora\proof-output\tlc-meshy-p20k-placeables-shadow-v2-20260726` |
| Machine-readable handoff | `ready-for-owner-proof.json` |

### Package hashes

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2a_tlcs2_mod.mod` | 19,677 | `d3dbbb2d8114063a5dfd5410c900c48b02ae4ebeb448cf6f13da1e948c9fbe82` |
| `m2a_tlcs2_hak.hak` | 45,818,460 | `c7fb06293dc5d34962d26a9e53ede4bda8dffebcc80446b793d2cb1b011a64fc` |
| `placeables.2da` | 3,019,965 | `4fb989a7df0906c3e0568e51715d880f6d5a8ccfbe25fc903f883759f479d217` |
| Area GIT | 4,588 | `ccfd9a3292ba0e5a80ce7b7ce56638a11ffeb4b6ab6715afda5dfb61fb96c6a1` |
| Area GIC | 687 | `ce22bfb98dd2394f0be810f58d5252f0c39c53633edfa3b741e908b0552b3f9b` |
| custom palette ITP | 1,661 | `170fdcba7a7e7ea146af7a93001c5d954c6c82cb0283cce985b71241fb31edb9` |

## Exact placeables and adjacency result

Every model contains exactly one render mesh, 20,000 triangles and 60,000
indices. The shared MDL writer preserved the final render vertices and joined
face adjacency by exact geometric positions instead of final UV/normal-split
render indices.

| Appearance row | Object | Model resref | Placement | MDL SHA-256 | V1 false/open edge occurrences | V2 open edge occurrences | V2 linked edge occurrences |
|---:|---|---|---|---|---:|---:|---:|
| 16500 | TLC Shadow V2 Civic Reliquary 20K | `m2a_tlcs2_rel` | `(6.5, 14.5, 0.0)` | `9487328613bd22fc2fd24a7812828f522786e5310ea4f13f0c2b5c1e16e89892` | 24,036 | 146 | 59,854 |
| 16501 | TLC Shadow V2 Aether Lamp 20K | `m2a_tlcs2_lmp` | `(10.0, 14.5, 0.0)` | `c479b3e0e205546c3efb6edf050c644ad6ae48db6cd91367b34f95d37355d60e` | 24,500 | 104 | 59,896 |
| 16502 | TLC Shadow V2 Sewer Ward 20K | `m2a_tlcs2_wrd` | `(13.5, 14.5, 0.0)` | `431f7780618f8a5fd52668a3db76d677263e9a11d2fd1f978fffe629d3325be6` | 23,218 | 156 | 59,844 |

The remaining open occurrences are deliberate: true geometric boundaries and
all occurrences of non-manifold edge groups remain open. This fails closed
instead of inventing an ambiguous neighbour. The dense V1 stripe defect was
caused by tens of thousands of UV/hard-normal render seams being incorrectly
treated as open silhouette edges.

## Source lineage

| Source | SHA-256 |
|---|---|
| `tlc-civic-reliquary-20000.glb` | `865fe8eaab2e2996354faf19491b7ccf341a3239a30762c78cadad3b3a0dc994` |
| `tlc-aether-lamp-20000.glb` | `310a6597f92bfcd391f44d293585417fa53abe3f1a576d93a7e0dc2c586b1ac7` |
| `tlc-sewer-ward-20000.glb` | `4192345b22327997708bad441fc70ce35be8db45331aa01dd928eeb432cc28c1` |
| base `placeables.2da` | `b772eafec5e6b380ad41e163e2a52585f2ddcec1c5bd7acea230b7e1a618df90` |

No Meshy generation or additional credits were consumed for V2.

## Offline verification

Passed before materialization:

- canonical workspace assertion;
- targeted writer regression for UV/normal seam adjacency;
- targeted writer regression for non-manifold edges;
- full `mdl_writer`: 42 passed, 0 failed;
- `placeable_pipeline`: 8 passed, 0 failed, 1 ignored owner-source gate;
- example compile gate for `materialize_tlc_meshy_p20k_placeables`.

Passed during the single V2 materialization:

- exact input hashes;
- exact 20,000-triangle geometry per model;
- binary MDL readback;
- shadow adjacency readback and recorded edge counts;
- ASCII PWK readback;
- `placeables.2da`, UTP, GIT, GIC and custom-palette identity;
- combined HAK/MOD resource readback;
- create-new and byte-for-byte output readback.

After materialization, a repeated test command was temporarily blocked by the
concurrently active tile implementation: its in-progress
`MdlStateProjectionProfileV1` edit briefly omitted
`OwnedRuntimePositiveType0RigOnlyV1` while 14 existing call sites still
referenced it. The parallel task restored that existing variant without any
change to the frozen candidate. The final repeated gates then passed:

- `mdl_writer`: 42 passed, 0 failed;
- `placeable_pipeline`: 8 passed, 0 failed, 1 ignored owner-source gate;
- `cargo check` for the exact TLC materializer;
- targeted `git diff --check`;
- canonical workspace assertion.

The already materialized candidate was not regenerated.

## Native installation

Both destinations were confirmed absent immediately before the copy. The
source was hashed, copied once, and the destination was hashed again.

| Kind | Native destination | SHA-256 result |
|---|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_tlcs2_mod.mod` | byte-identical: `d3dbbb2d8114063a5dfd5410c900c48b02ae4ebeb448cf6f13da1e948c9fbe82` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_tlcs2_hak.hak` | byte-identical: `c7fb06293dc5d34962d26a9e53ede4bda8dffebcc80446b793d2cb1b011a64fc` |

## Owner proof checklist

- [ ] Open `m2a_tlcs2_mod.mod`.
- [ ] Confirm Toolset shows `Meshy2Aurora TLC Shadow Adjacency V2`.
- [ ] Open Area `Meshy2Aurora TLC Shadow Adjacency Test`.
- [ ] Confirm all three exact V2 placeables are visible at the placements above.
- [ ] Enable shadows and inspect each long projected shadow at the lighting angle
      that exposed the V1 stripe defect.
- [ ] Confirm that the dense striped false silhouettes are absent.
- [ ] Record any localized residual artifact separately; true boundaries and
      non-manifold source edges are not silently paired.
- [ ] Run the same exact MOD/HAK lineage in NWN without rebuilding or replacing
      either file.
- [ ] Confirm visibility and shadows in NWN.
- [ ] Confirm that every PWK still blocks the player from several directions.
- [ ] Return screenshots or an owner verdict without editing the candidate.

Only the owner observation may change the runtime fields from `not_tested`.
