# M0 r33 animated-donor correction plan — 2026-07-24

## Admission

This record satisfies the diagnosis and minimal-delta conditions that remain
after the accepted immutable r32 proof.

Exact accepted r32 result:

- accepted proof:
  `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-2/final-model-proof.accepted-at-543958529728.json`;
- accepted-proof SHA-256:
  `1c787faf0780a44637264051ab4e049b02266d512db15145789113cabad53345`;
- Aurora Toolset: `modelVisibility=visible`,
  `proofCompleteness=verified`;
- NWN: `modelVisibility=not_visible`,
  `proofCompleteness=verified`.

The exact r32 candidate is preserved. This investigation did not overwrite its
MOD, HAK, MDL, texture, 2DA row, proof profile or accepted evidence.

## Diagnosed failure boundary

The prior offline audit and the accepted NWN image eliminate the creature
container, ordered HAK, 2DA resolution, texture presence, bounds, `render`
flag, triangle-list inputs and source geometry as sufficient explanations.
The same exact M0 is drawn by Toolset and absent in a judgeable NWN frame.

The remaining candidate-specific failure boundary is therefore the r32 runtime
model representation:

- one rigid mesh;
- a four-node synthetic hierarchy;
- seven controller-free type-5 state projections;
- no deforming skin surface, bone map or active lifecycle motion.

`SkinMesh` is not a universal requirement for all NWN creatures. It is the
selected correction here because the project-owned H1 positive runtime witness
proves a different writer-produced model family is drawable by NWN: an
animated hierarchy with a weighted skin path and seven resolved lifecycle
clips. The correction removes the unproven r32 runtime representation instead
of changing already-verified resolver, container or geometry inputs.

NeverBlender's exporter independently confirms that a usable skin path requires
real bone identities and vertex weights; it does not invent them for a static
mesh. The four strongest influences are retained and normalized. Relevant
sources:

- `https://github.com/gyoerkaa/mdltools/blob/master/neverblender/nvb_node.py`;
- `https://github.com/gyoerkaa/mdltools/blob/master/neverblender/nvb_anim.py`;
- `https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp`;
- `https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/animation.cpp`.

## Exact inputs selected for the correction

- static Meshy M0 source:
  `proof-output/m0-r30-retail-runtime-conformance-20260721/generated/source.glb`;
  - bytes: `8,581,684`;
  - SHA-256:
    `aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1`;
- project-owned Meshy H1 animated donor:
  `sample-3d/h1-humanoid-1500/source.glb`;
  - path amendment 2026-07-27: canonical relocation only; source bytes and
    SHA-256 used by this evidence did not change;
  - bytes: `7,944,380`;
  - SHA-256:
    `3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`.

The donor supplies hierarchy, bind transforms, reference skin surface, weights
and animation channels. M0 supplies the output geometry and material/texture.
No stock or third-party model payload is copied.

## Minimal intended functional delta

The one intended r33 functional change is the MDL runtime representation:

| Property | r32 | Intended r33 |
| --- | --- | --- |
| source M0 geometry | exact Meshy M0 | unchanged input geometry |
| source M0 texture | exact `m2a_m0t01` payload | unchanged payload |
| deformation | rigid | weighted `SkinMesh` |
| hierarchy | four synthetic nodes | 24 donor-derived owned nodes |
| active bones | 0 | 22 |
| lifecycle animations | seven controller-free states | donor-motion `cpause1` plus six owned aliases |
| scale/alignment | rigid Profile A | donor-bind Profile A, computed scale `0.8980634` |

The required direct-creature clip inventory is:
`cappear`, `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `cdead`.
Missing donor states are aliases of the user-provided `cpause1` clip, matching
the already tested H1 package path.

Fresh r33 MOD, Area, HAK and model resrefs are only immutable delivery
identities. The runtime-complete creature envelope, 2x2 fixture scene,
Appearance semantics and source texture bytes remain the validated r32 family.

## Implementation defect found before materialization

The first exact-input retarget test failed closed with:

`M3A-LIMIT-EXCEEDED at distanceEvaluations`

The converter was performing a complete reference-surface distance scan to
choose a segment even when the donor rig had exactly one segment. That scan
could not change the assignment and consumed the bounded evaluation budget
before weight projection.

The correction in `profile_a`:

- assigns directly to the sole segment;
- retains exhaustive distance evaluation for multi-segment rigs;
- does not raise the compiled safety maximum;
- validates every reference triangle once in target world so the optimization
  cannot hide a transformed degenerate surface.

Verification after the fix:

- `cargo test -p m2a-core --test profile_a` — 40 passed;
- exact real-input retarget — passed;
- synthetic animated-donor retarget — passed after requiring the complete
  seven-clip runtime inventory.

Exact real-input offline result, before any r33 package materialization:

- rig profile SHA-256:
  `c01297d44204a96c77a97628190a67da837856db909ecf2498b73bcd517ffc72`;
- nodes: `24`;
- skin segments: `1`;
- active bones: `22`;
- local clips: `7`;
- uniform scale: `0.8980634`;
- deterministic candidate MDL SHA-256:
  `b82b6d7b9260a05cebb7bb93ed75f2938a210a01bd01bd397f1b63fc60fde25e`.

The final candidate keeps the already proven M0 texture resref
`m2a_m0t01`; the earlier pre-package diagnostic used a temporary texture
resref and is superseded by the hash above. The TGA payload itself is
byte-identical to r32.

## Execution boundary

One r33 package may now be materialized from the exact inputs and delta above.
It must pass deterministic replay, MDL semantic readback, weighted-skin
readback, HAK resource readback, complete Appearance-row readback and
runtime-complete MOD readback before installation.

No Aurora/NWN proof is warranted until all offline contracts pass. After that,
perform one candidate-bound Toolset-to-NWN proof through the public
`aurora-model-proof-120s` skill. A proof-lane failure must resume the same r33;
only a fresh visual `not_visible` result can admit another model iteration.

## Accepted r33 result and r34 admission

The immutable r33 candidate was subsequently exercised through the mandatory
120-second proof route. Its accepted packet is:

- accepted proof:
  `proof-output/m0-r33-animated-donor-20260724/live/model-proof-120s-run-1/final-model-proof.accepted-at-584746800812.json`;
- accepted-proof SHA-256:
  `dd0f74ab0cb7494f0a95ca63aa01f20eb88f067cc79ca9bbd079de16afdfea82`;
- Aurora Toolset: `modelVisibility=visible`,
  `proofCompleteness=verified`;
- Toolset capture SHA-256:
  `626ddebab1171852a5929342f01ced666add75fefb3ee8745f0367dca26d2113`;
- NWN: `modelVisibility=not_visible`,
  `proofCompleteness=verified`;
- NWN capture SHA-256:
  `8cb70100e4076a908ba0467e458bec43959d6653b9b50ad79e10b6280ee737c5`.

This fresh candidate-bound NWN visual failure admits exactly one new model
iteration under the project gate. r33 and its accepted evidence remain
immutable.

## r33 post-failure diagnosis

Offline world-space readback rejected an axis or scale correction:

- exact r33 base bounds in world space:
  `min=[-0.488232175,-0.286940173,-0.00000017988]`,
  `max=[0.488232226,0.286940215,1.700000010]`;
- the model is Z-up, grounded and approximately 1.7 m tall;
- sampled linear-blend-skinning evaluation across `cpause1` remains near the
  origin, grounded and approximately 1.83–1.85 m tall;
- bone-slot maps, extended64 skin layout, WXYZ inverse binds, animation type 5
  and generic state dummies all have independent native witnesses.

The remaining exact lineage difference is the local-animation state topology:

- r33 base tree: 24 donor rig nodes plus one `SkinMesh`;
- every r33 state: the same 24 rig nodes plus a generated generic `0x01`
  `m2a_seg_1` SkinMesh identity dummy;
- the project-owned H1 runtime-visible v20 model: 25 base nodes, but only the
  24 root/joint identities in `cpause1`;
- audited native families such as `c_kocrachn`/`c_Horror` and `c_squirrel`
  likewise omit base skin leaves from their local state trees.

The corpus contains other legal state families, so the existing full-base
dummy profile remains unchanged. The correction is a new explicit,
candidate-selected rig-only profile, not a global writer rewrite.

## Minimal intended r34 functional delta

r34 preserves the exact r33 source M0, donor H1, texture bytes, weighted
SkinMesh, inverse binds, active-bone map, controller tracks, clip inventory,
scale, Appearance semantics and runtime-complete module fixture.

The only intended MDL behavior change is:

| Property | r33 | Intended r34 |
| --- | --- | --- |
| base tree | 24 rig nodes + 1 SkinMesh | unchanged |
| every type-5 state | 24 rig nodes + `m2a_seg_1` generic dummy | 24 rig nodes only |
| state rig identities/controllers | donor-derived | unchanged |
| renderable payload in states | none | none |

The new profile is
`RETAIL_DIRECT_CREATURE_TYPE5_RIG_ONLY_V1`. Its offline conformance requires
at least one renderable base leaf, exact ordered projection of every
non-renderable rig identity, generic `0x01` state nodes, animation type 5, no
state mesh/skin payload and no state entry for a base mesh/skin leaf.

Focused verification before r34 materialization:

- writer contract: 33/33 tests passed;
- synthetic animated-donor retarget: passed;
- exact M0/H1 retarget with the rig-only profile: passed;
- exact r34 pre-package model SHA-256:
  `2fe4ad1ae4354335008916cbff3e0f724fedf0119f30f07aa8d5341c3d5b4af5`;
- base nodes: 25; every local-animation state: 24;
- rig nodes: 24; SkinMesh nodes: 1; active bones: 22; clips: 7.

No further Aurora or NWN run is permitted until the versioned r34 package,
deterministic replay, readback and proof profiles are complete. Then one
candidate-bound Toolset-to-NWN proof is performed through the shared skills.

## Accepted r34 result and r35 admission

The immutable r34 candidate completed one public
`aurora-model-proof-120s` run:

- accepted proof:
  `proof-output/m0-r34-rig-only-state-20260724/live/model-proof-120s-run-1/final-model-proof.accepted-at-610953639291.json`;
- accepted-proof SHA-256:
  `862cc0cfa2869aafdd382d548db57bebf2330d9a8db80a7885e3ceb023755098`;
- publication time: `66 347 ms`, inside the 120-second live budget;
- Aurora Toolset: `modelVisibility=visible`,
  `proofCompleteness=verified`;
- Toolset capture SHA-256:
  `78d9dc65c9fe808dd0937c302c4ba654a691dd9d4c189cb2864f75b5eaf37f73`;
- NWN: `modelVisibility=not_visible`,
  `proofCompleteness=verified`;
- NWN capture SHA-256:
  `fefde601eef38f926f4c917193af49da2a5904d1e39d6c334cd66a8dcb960e1c`.

The NWN frame is candidate-bound and judgeable: the player and unobstructed
forward fixture location are visible, but the expected r34 model is absent.
This admits one new iteration. The separate coordinator field
`standardCertification=unverified_requires_two_live_runs` does not alter the
accepted candidate verdict and must not be reported as a global two-run
certification.

## r34 post-failure SkinMesh diagnosis

The project-owned H1 v20 positive control does not prove SkinMesh rendering.
Its exact runtime-visible model is a rigid `0x21` TriMesh. r33 and r34 use the
writer's still-unproven `0x61` SkinMesh path. Therefore the positive H1 witness
proves the common binary writer and custom-resource route, but not the failing
skin palette serialization.

Exact read-only comparison found one writer decision that remained explicitly
open since M4:

- r34 has 22 active inline palette slots followed by 42 unused `-1` entries;
- compiled CEP `c_squirrel`, SHA-256
  `fce071fcebc04d0ffce33b6ef48891af615e70dd402120554dcc963cca36f78b`,
  has dense active slots followed by a zero-filled unused tail in both
  SkinMeshes;
- compiled CEP `c_kocrachn`, SHA-256
  `f16426310f826ae2ab15034ac979c65f812ee8bda0d13ee459bf2b293d7db270`,
  confirms the active forward/reverse round trip; only its used prefix is
  semantically relevant;
- Aurora `FUN_00a932cc` iterates the 64 inline entries, skips negative entries
  and terminates after the repeated zero sentinel. The writer's `-1` tail was
  structurally readable but had no positive runtime witness.

The correction is isolated in the new writer profile
`M4_DIRECT_CREATURE_EXTENDED64_ZERO_TERMINATED_V2`. The frozen V1 profile still
reproduces r34 byte-for-byte. With the same `m2a_m0p34` resref, an exact test
proves that V1 and V2 differ only in the 84 bytes representing the 42 unused
`i16` inline entries:

- V1 tail: `ff ff` repeated 42 times;
- V2 tail: `00 00` repeated 42 times;
- source, donor rig, weights, refs, q/t inverse binds, mesh, materials,
  controllers, clips and rig-only type-5 state trees are unchanged.

For the fresh r35 resref the exact pre-package model SHA-256 is
`779d93fa762980ef17448762ba97d8f8775b03335b1d9561b8c2b6483772b3e0`.
The focused writer suite passes `34/34`, the exact same-resref delta test
passes, and the deterministic r35 package/replay contract passes. No live
Aurora or NWN action is justified until r35 materialization, immutable hashes
and proof profiles are complete; after that only one candidate-bound proof is
performed.

## Accepted r35 result and r36 admission

The immutable r35 candidate completed one public
`aurora-model-proof-120s` run:

- accepted proof:
  `proof-output/m0-r35-zero-terminated-skin-20260724/live/model-proof-120s-run-1/final-model-proof.accepted-at-631673200993.json`;
- accepted-proof SHA-256:
  `7479bbe57093e7e74f34aee5474ef6bc2d8a44a2f7fe843101c2322c57ea3d7c`;
- publication time: `54 889 ms`, inside the 120-second live budget;
- Aurora Toolset: `modelVisibility=visible`,
  `proofCompleteness=verified`;
- Toolset capture SHA-256:
  `bff57d75f84aeb95d98ab75e20f0f275271b5028ae1079b3298ec22642a6943d`;
- NWN: `modelVisibility=not_visible`,
  `proofCompleteness=verified`;
- NWN capture SHA-256:
  `9188c5fc5c772c80d5a6e21c58e964bd7adfd0732e5fb98e475fd393febae440`;
- cleanup packet SHA-256:
  `b7ca835c08ab6250bc084c043a96f7b1cf16a4bc1c9995fa2b3675902ea22124`,
  with zero remaining Toolset and NWN processes.

This is a fresh, candidate-bound and judgeable NWN visual failure. It
disproves the unused `-1` SkinMesh palette tail as the sole runtime cause and
admits one new model iteration. The separate coordinator field
`standardCertification=unverified_requires_two_live_runs` does not alter the
accepted candidate result.

## r35 post-failure dedicated-root diagnosis

### Facts from generated and native binary readback

- r35 has 24 rig nodes plus one SkinMesh; its only skeleton root is the donor
  joint `Hips`, which V1 renamed to the model resref;
- that same root joint is heavily weighted: 933 active weight lanes, total
  weight approximately `297.84`, maximum weight approximately `0.948`;
- r35 therefore places a weighted joint at tree ordinal `0` and includes
  ordinal `0` in active SkinMesh slot `0`;
- native `c_squirrel` has a dedicated model-named root followed by
  `Bdger_rootdummy`; its active SkinMesh ordinals exclude tree ordinal `0`;
- native `c_kocrachn` independently excludes tree ordinal `0` from its active
  SkinMesh palette;
- the NeverBlender authoring contract requires one model/file-named Aurora
  Root, while the exact Meshy H1 GLB has no `skin.skeleton`: `Hips` is its only
  joint root and `Armature` is an external transform container.

Aurora `FUN_00a932cc` does not prove that a single active zero ordinal is
illegal: it accepts the first zero and terminates on a later zero. Therefore
the correction is not justified as a copied sentinel rule. The diagnosed
structural defect is that the conversion collapsed two roles into one node:
the model-named Aurora Root and the weighted donor skeleton root.

### Rejected alternatives

- changing SkinMesh constants remains unsupported: the direct-palette trace
  does not read them and the local corpus contains thousands of valid zero
  entries;
- copying native routine fields remains forbidden and unjustified: they are
  opaque runtime values, while the project-owned rigid positive control is
  visible with the writer's zero routine fields;
- axis, scale, bounds, q/t inverse-bind order, WXYZ storage, normalized
  weights, raw bone references and rig-only state topology remain unchanged
  because their exact offline invariants already pass;
- another palette-tail change is rejected by the accepted r35 failure.

## Minimal intended r36 functional delta

The new public retarget route
`retarget_static_mesh_to_animated_donor_v2` inserts one identity, unweighted
Aurora Root named after the model resref. It preserves the original `Hips`
name, id, bind-local transform, child hierarchy, weights and all animation
track targets, and reparents only `Hips` below the new root. V1 remains frozen.

Exact same-input offline comparison gives:

| Property | r35 / V1 | r36 / V2 |
| --- | --- | --- |
| rig nodes | 24 | 25 |
| base nodes including SkinMesh | 25 | 26 |
| nodes in each rig-only state | 24 | 25 |
| root | renamed weighted `Hips` | identity unweighted `m2a_m0p36` |
| skeleton root | collapsed into model root | preserved child `Hips` |
| active SkinMesh ordinals | `0..21` | `1..22` |
| forward slot for ordinal 0 | `0` | `-1` |
| active bones / weighted vertices | 22 / 2380 | unchanged |
| unused inline tail | 42 zero entries | unchanged |

The exact pre-package r36 MDL SHA-256 is
`459b9954d377c1daab9b12c73a2bf9a64507b5f3cf6d2a6a2ea7d751f680963a`.
The exact r34 and r35 hashes still pass after the V2 addition.

## r36 offline candidate result

The deterministic candidate was materialized once at
`proof-output/m0-r36-dedicated-aurora-root-20260724` after exact-input replay
and readback:

- MDL SHA-256:
  `459b9954d377c1daab9b12c73a2bf9a64507b5f3cf6d2a6a2ea7d751f680963a`;
- TGA SHA-256:
  `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`;
- `appearance.2da` SHA-256:
  `cad23b4624c15ac4ede16dce354ff25b1c25a891f786eadf849968bf34957f09`;
- HAK SHA-256:
  `af4b3d486cf0e91895ebec4bb066a98aa298dbd8eed3ffba5935582f7a6cea5f`;
- MOD SHA-256:
  `dfebd329713ad60217d7364a3811df499cc1b7a264579eaf82dea53fd81135d4`;
- lineage-contract SHA-256:
  `8b7ec622903399957858dd5cda2079b8dc8eac1798c7e72895844f608d75660a`.

Focused verification:

- synthetic V2 retarget test: passed;
- three exact M0/H1 retarget tests, including frozen r34/r35 hashes: passed;
- exact r36 candidate deterministic replay, HAK resource readback,
  Appearance-row readback and runtime-complete MOD readback: passed;
- materializer argument/identity contract and post-write byte readback:
  passed.

The dedicated-root change is still a tested implementation hypothesis until
NWN draws the exact immutable r36 candidate. No further model iteration is
admitted. The next live action, after proof profiles and immutable install
preflight are complete, is one candidate-bound Toolset-to-NWN proof through
the shared `aurora-model-proof-120s` skill.

## Accepted r36 result and r37 admission

The immutable r36 candidate completed exactly one public
`aurora-model-proof-120s` run after all offline correction work:

- model-proof profile SHA-256:
  `26163b870bb8563556612f599f3fa40907910e36cfdf9e4bbecaba9bf04c11e6`;
- accepted proof:
  `proof-output/m0-r36-dedicated-aurora-root-20260724/live/model-proof-120s-run-1/final-model-proof.accepted-at-648924905534.json`;
- accepted-proof SHA-256:
  `d91bd0f724903ee8ffdd915a86c12f3d3e64282f9f45a208ca330ae3ed857775`;
- publication time: `58 501 ms`;
- Toolset: `modelVisibility=visible`,
  `proofCompleteness=verified`, capture SHA-256
  `d733bb2d3099b10e60ede55970fcb16bc98f6b5baa7dc9e5c4d0e4c93a537f1a`;
- NWN: `modelVisibility=not_visible`,
  `proofCompleteness=verified`, capture SHA-256
  `837e6b2b8602b24791b328a58310494b650331a94fd2c1242ebe6f7cec78bf6f`;
- cleanup SHA-256:
  `120abdcaff0729defa009b57edb0fbe8ef8daa332b3647a47203efb64ae755f8`,
  with zero Toolset and NWN processes.

The clear runtime frame shows the player and unobstructed forward fixture
location but no r36 model. Therefore a dedicated unweighted Aurora Root and
exclusion of ordinal zero from the palette are not sufficient. This fresh
candidate-bound visual failure admits one r37 iteration.

## r36 post-failure SkinMesh-parent diagnosis

Fresh own-reader topology comparison found a stronger invariant:

- r36 SkinMesh `m2a_seg_1` is a child of the weighted `Hips` skeleton joint;
- binary `c_squirrel`: both `sqrlfront` and `sqrlback` are direct children of
  the model root `c_squirrel`;
- binary `c_kocrachn`: all three SkinMeshes are direct children of the model
  root `c_kocrachn`;
- the local read-only ASCII skinmesh-animals corpus contains 69 SkinMesh nodes
  across 36 models; all 69 have `parent` equal to their `newmodel` root.

Thus the current route differs from 74/74 observed SkinMesh nodes across
38 models. This is a topology fact, not yet proof that the parent alone is the
runtime rejection predicate.

r36 mesh positions are local to `Hips`. A parent-only field edit would rotate
and translate the bind pose and would be invalid. The minimal correction must
change coordinate domains coherently:

1. keep the dedicated identity Aurora Root and the preserved child `Hips`;
2. bake the old `Hips` bind-local rigid transform into SkinMesh positions,
   normals and tangents;
3. reparent only the SkinMesh segment to the Aurora Root;
4. preserve joint ids, hierarchy, weights, refs, UVs, indices, animation
   tracks, texture, clips and state topology;
5. let the writer recompute q/t inverse binds relative to the new SkinMesh
   parent.

The closing invariant is equality of every bind-pose world vertex and normal
between r36 and the corrected route, not equality of local mesh bytes. The
new V3 synthetic test uses a translated and rotated skeleton root and proves
that the baked/reparented mesh has the same world geometry, is a direct child
of the Aurora Root, preserves weights and animation tracks, and passes
semantic readback.

No r37 package or proof is permitted until the exact M0/H1 V3 test locks the
new model hash, direct-root parent, unchanged bind-pose world bounds, 22 active
bones, 2380 normalized weight rows, seven 25-node rig-only states and
deterministic replay.

## r37 offline candidate result

The V3 gate passed before packaging. For the exact M0/H1 inputs it proves that
every bind-pose world vertex and normal remains equal to r36 within `1e-6`,
while the SkinMesh becomes a direct child of the dedicated Aurora Root.
Weights, indices, UVs, joint ids, skeleton hierarchy and all animation tracks
remain unchanged. Exact readback reports 25 rig nodes, 26 base nodes, seven
25-node rig-only states, 22 active bones, 2380 weighted vertices, active
ordinals `1..22`, ordinal-zero forward slot `-1`, and a zero-terminated unused
palette tail.

The deterministic candidate was materialized once at
`proof-output/m0-r37-direct-root-skinmesh-20260724`:

- MDL SHA-256:
  `48746e6e0b19bedbdcc8a364ff96cd583848dfa38e06971706bfb69b0341f676`;
- TGA SHA-256:
  `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`;
- `appearance.2da` SHA-256:
  `d04d5d6a185aca56a8fef0bbc81b0a49bfa4a1aa23ded163a568a9d7664a2ccb`;
- HAK SHA-256:
  `d70343b64a993883729c1459fd2334e5d7dedbda30a19b47a8490abefe9abcac`;
- MOD SHA-256:
  `f7b8ea2ea2401a6931f90cefb82b7a6b9887bce527fae8a5367604c2fb0c5d6f`;
- lineage-contract SHA-256:
  `25cc98f41cc10a177d131802a7ea08614b03fccecfd6b9fbb197eb04e4f3715d`.

Focused verification passed:

- 34 binary MDL writer/readback tests;
- four synthetic retarget tests for V1/V2/V3;
- four exact M0/H1 regression tests, including frozen r34-r36 hashes;
- exact r36 deterministic replay;
- exact r37 deterministic replay, HAK resource readback, Appearance-row
  readback, runtime-complete MOD readback and negative identity mutations;
- materializer CLI contract and post-write byte readback.

r37 is now immutable and ready for one candidate-bound Toolset-to-NWN proof.
No repeated live probe is authorized: the next live run occurs only after the
versioned profiles, exact absent-only native install and public READY
transitions pass.

## Accepted r37 result

The immutable direct-root SkinMesh candidate completed exactly one public
`aurora-model-proof-120s` run:

- model-proof profile SHA-256:
  `22898dea23b1659b2b7ae6da0e93cd2bf486f134e8ba0128d251c0f237d8a2a7`;
- accepted proof:
  `proof-output/m0-r37-direct-root-skinmesh-20260724/live/model-proof-120s-run-1/final-model-proof.accepted-at-664099664627.json`;
- accepted-proof SHA-256:
  `bf739f2f1d040dd408d06dd6ee0de4f4a70463c359e5e0754737dcb484ab88fd`;
- publication time: `50 822 ms`;
- Toolset: `modelVisibility=visible`,
  `proofCompleteness=verified`, capture SHA-256
  `d7e1ce0822a4afe2a14e597883ae2b1f81eeb160d614c38262dc2234e3563ecc`;
- NWN: `modelVisibility=not_visible`,
  `proofCompleteness=verified`, capture SHA-256
  `83499efb61ced0f2f392356096b0be07d4df4563e776a4989ae40b351600b157`;
- cleanup SHA-256:
  `0f172902722a1c3e11488140440876e69d9c55444db1d721a2a0a31beae6461a`,
  with zero Toolset and NWN processes.

The judgeable NWN frame shows the player and unobstructed forward fixture
location but no Meshy creature. Therefore direct-root SkinMesh parenting is a
real native topology invariant but is not sufficient to make this candidate
draw in runtime. This fresh exact-candidate failure admits diagnosis for one
new iteration; no r38 artifact may be created until a new concrete cause and
minimal intended delta are recorded.

## r37 post-failure animation-scale diagnosis

The exact r37 readback contains seven non-unit animation-scale controllers:
one on `Hips` in every materialized local clip (`cpause1`, `cappear`, `cwalk`,
`crun`, `ca1slashl`, `cdamagel` and `cdead`). Each controller has two rows and
is constant at approximately `1.1764704`. The source H1 GLB confirms that this
is not writer corruption: its one scale channel targets `Hips` and contains
two positive, uniform rows around `1.1764705`. The static `Hips` bind scale is
unit; the external `Armature` container carries the separate `0.01` scale
already handled by the H1 mapping.

Three independent local witnesses establish that this controller is outside
the observed Aurora SkinMesh animation contract:

- binary `c_squirrel.exact.mdl` has 43 local animations and 22 scale
  controllers in its base tree, all unit; none of its animation trees has a
  scale controller;
- binary `c_kocrachn.exact.mdl` has 28 base-tree scale controllers, all unit,
  and no local animations;
- the read-only ASCII SkinMesh-animal corpus contains 69 SkinMesh nodes in 36
  models and 761 `scale` rows; every row is exactly `1.0`, with zero
  `scalekey` blocks.

The engine trace supplies the mechanism rather than only a corpus
correlation. The SkinMesh palette routine `FUN_00a932cc` combines each active
bone's position with inverse translation and its quaternion with inverse
rotation. Its caller `FUN_00a936a4` likewise propagates skeleton transforms
through translation and quaternion inputs. No scale vector is supplied to or
read by this palette path. Aurora's generic controller parser recognizes
controller type `0x24` (`scale`/`scalekey`), but that does not make animated
scale representable by the runtime SkinMesh deformation path.

Therefore r37 emits an animation component which its selected runtime
deformation path cannot consume. This is a concrete contract mismatch. It is
not yet claimed to be the only runtime rejection condition; the next
candidate remains an implementation hypothesis until NWN draws it.

## Minimal intended r38 functional delta

The new versioned retarget route will keep the complete r37 model, mesh,
direct-root SkinMesh topology, bind pose, inverse binds, weights, UVs,
indices, materials, translations, rotations, clips and rig-only states
unchanged. Its only functional delta is animation-scale normalization:

1. require every emitted scale track to target the preserved weighted
   skeleton root;
2. require its scalar rows to be finite, positive and constant within a tight
   tolerance;
3. require the same constant value in every materialized runtime clip;
4. remove those root-scale tracks before the binary writer;
5. fail closed on a scale track for any other node, a varying value, mixed
   values between clips, malformed row width or a clip missing the expected
   track.

No geometry or skeleton translation is scaled again. The static source mesh
has already been normalized into the target rig's bind-space bounds, and the
H1 container scale has already been folded into target bind translations by
the existing validated mapping. Baking the animation-only `1.1764704` again
would introduce a second geometry transform rather than remove the
unsupported palette component.

V1 through V3 and the frozen r34-r37 payload hashes must remain unchanged.
Before packaging, focused tests must prove exact removal of all scale
controllers, byte-for-byte preservation of every non-scale animation track,
unchanged creature IR and bind-world geometry relative to r37, semantic
writer/readback success and deterministic exact-input replay. Only after
those gates pass may one immutable r38 candidate be materialized. No live
proof is scheduled during this correction phase.

## r38 offline candidate result

The versioned V4 retarget route now enforces the scale-normalization contract
before writing. Synthetic negative tests reject a varying root scale, a scale
track on a non-root joint and a missing scale track. The exact M0/H1 test
proves:

- creature IR, bind geometry, direct-root SkinMesh topology, inverse-bind
  inputs, weights, indices and UVs are equal to r37;
- every non-scale animation track is byte-for-byte equal to the corresponding
  r37 track;
- r37 has exactly seven animation-scale controllers and r38 has zero;
- r34, r35, r36 and r37 retain their frozen model hashes;
- exact r38 replay is deterministic and semantic binary readback has no diff.

The deterministic candidate was materialized once at
`proof-output/m0-r38-scale-normalized-skinmesh-20260724`:

- MDL SHA-256:
  `039d07cd937430d83006c7d0176aa7659440265417fb7fe53bf73b405563c248`;
- TGA SHA-256:
  `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`;
- `appearance.2da` SHA-256:
  `1dbc4a5af38e4ef688c05cfb93939f047d8231724d7c86f0caa185c03358d3c6`;
- HAK SHA-256:
  `6e842d4b883867903e2cc29ce4efe9a8f5143d56691b7500aec4dc92855d6278`;
- MOD SHA-256:
  `f22625b291f2e43a0dfa738f514efd3adf58b38832f914158d38f6eafa4440dd`;
- lineage-contract SHA-256:
  `7f3dc228027b27e1bb35c43a93f9ed4ff30afefd2ee7adc483c748888117e8e2`.

The readback locks 26 base nodes, 25 rig nodes in each of seven type-5 states,
one direct-root SkinMesh, 2380 normalized weighted vertices, 22 active bones,
active ordinals `1..22`, a zero-terminated 42-slot tail and zero animation
scale controllers. The exact MOD and HAK were installed through absent-only
copies and their destination hashes are byte-identical to these sources.

Proof-profile hashes are:

- binary bootstrap:
  `c07ca8cc3804974c90f502e60ba32db914e7afaea2d9999ec64eb643cac3ff9b`;
- Toolset:
  `78bb4aaf15e2f48b586b3b8297a8b8a8e75cffa277c93e13163a608533adecab`;
- runtime:
  `45ccd4da9838d6b173695f1f3d92b7bb26294a427c141b6136a515e2c825ef38`;
- model-proof profile:
  `7bb6610711f5b5429e40f0621a46eb3180a0bbcfc3c2e1fba7554eb10abd0b3b`.

No Toolset or NWN proof has been run for r38 yet. The next live action is the
single public skill preparation/READY/timed proof sequence for this immutable
candidate; there is no repeated probe loop.

## Accepted r38 result

The immutable scale-normalized candidate completed one public
`aurora-model-proof-120s` run after the full offline correction phase:

- accepted proof:
  `proof-output/m0-r38-scale-normalized-skinmesh-20260724/live/model-proof-120s-run-1/final-model-proof.accepted-at-682918803228.json`;
- accepted-proof SHA-256:
  `59e8e461013b109c13611ab22d2112e5997ef044b3f43a26bcbc26dafb552db6`;
- publication time: `51 780 ms`;
- Toolset: `modelVisibility=visible`,
  `proofCompleteness=verified`, capture SHA-256
  `4d1602c53d6ed784c83f893809d8c1c54b0115354d481e7b99ad4516e01a7ab8`;
- NWN: `modelVisibility=not_visible`,
  `proofCompleteness=verified`, capture SHA-256
  `332c0e52b93f0074f5d818d62f5555724dd5d7176c789e4f8b55c5617ecd964e`;
- cleanup SHA-256:
  `f2608ed2d5c9cff5cdada9bda794f9fc486c9b5d5b7013972eaf2ef549fcdbb8`,
  with zero Toolset and NWN processes.

The exact r38 runtime frame is judgeable and shows the player with an
unobstructed forward fixture location but no Meshy creature. Therefore
animation-scale normalization repaired an observed contract mismatch but was
not the runtime draw blocker. This fresh exact-candidate failure admits
diagnosis for one new iteration; no r39 artifact may be created until another
concrete cause and minimal delta are recorded.

## r38 post-failure controllerless-root diagnosis

Exact base-tree readback exposes a new structural difference:

- r38's dedicated identity Aurora Root has two controllers: one constant
  `position [0,0,0]` and one constant identity `orientation [0,0,0,1]`;
- binary `c_squirrel.exact.mdl` and `c_kocrachn.exact.mdl` both have zero
  controllers on their model root;
- all 36 models in the read-only ASCII SkinMesh-animal corpus have zero
  `position`, `positionkey`, `orientation`, `orientationkey`, `scale` or
  `scalekey` entries on their `newmodel` root.

Thus 38/38 native/corpus witnesses use a controllerless model root, while
r38 emits redundant bind controllers on that node. This is isolated to the
base tree: all 43 `c_squirrel` animation roots and all seven r38 animation
roots are already controllerless.

The difference was introduced when V2 added the dedicated Aurora Root. The
writer currently allocates and emits two bind controllers for every rig node,
including an identity, model-named root. Removing controllers from a
non-identity node would lose its bind transform and is forbidden. Removing
them from the dedicated identity root preserves its implicit identity
transform and matches every observed creature root.

## Minimal intended r39 functional delta

A new versioned binary format profile will extend the r38 zero-terminated
SkinMesh profile by omitting the controller-key and controller-data arrays
only for the single model-named, parentless rig node, and only when its bind
matrix is exactly identity within the writer's existing tolerance. It will:

1. require exactly one parentless rig root named after the model resref;
2. require that root to be identity and unweighted;
3. emit zero base controller keys/data for that root;
4. leave every other base-node controller, every animation tree and track,
   all mesh/skin data, hierarchy, bounds, states, resources and fixture bytes
   unchanged apart from layout-pointer shifts caused by removing 60 bytes;
5. fail closed if the new profile is used with a non-identity root.

The existing format profiles and frozen r34-r38 hashes remain unchanged.
Tests must compare r38 and the new profile semantically, prove exactly
`baseRootControllers: 2 -> 0`, preserve zero controllers on every animation
root, preserve reconstructed bind-world geometry and pass deterministic
writer/readback before any r39 package is admitted. No new live proof is
scheduled during this implementation phase.

## r39 offline candidate result

The new
`M4_DIRECT_CREATURE_EXTENDED64_ZERO_TERMINATED_CONTROLLERLESS_ROOT_V3`
writer profile is implemented and fail-closed. It accepts only the one
parentless model-named identity root when that node is absent from every skin
weight row. The semantic readback now distinguishes an intentional
controllerless identity rig node from a missing bind-controller payload.
Negative tests reject a non-identity root, a wrong root name and a weighted
root with stable `M4-CONTROLLERLESS-ROOT-INVALID` errors.

The exact M0/H1 differential proves:

- creature IR and every mapped animation track are unchanged from r38;
- the MDL raw region, SkinMesh maps, inverse rotations/translations, weights
  and bone references are unchanged;
- only the base model root changes from two bind controllers to zero;
- core length decreases by exactly 60 bytes;
- all seven animation roots remain controllerless and all scale-controller
  counts remain zero;
- semantic readback is empty and repeated writes are byte-identical.

The deterministic candidate was materialized once at
`proof-output/m0-r39-controllerless-identity-root-20260724`:

- MDL SHA-256:
  `fab5ab98e9225c1553947f17994441273ae4c9bbd5a1d14034721ee3be2d86db`;
- TGA SHA-256:
  `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`;
- `appearance.2da` SHA-256:
  `51a70b5bfe4070bf0136a52d36fcb5b075fd9990690d782f1124cfd94ffc6d0f`;
- HAK SHA-256:
  `241e93853150ac4ca2923ff3ec05aefdd936eeb7f132ae1dcc7a24345a978c5c`;
- MOD SHA-256:
  `d751b6a66c8887bae6f86402508e9b8385dbb99da110c40272bd435bf254fed1`;
- lineage-contract SHA-256:
  `3790962c8aab016b1f27fe78b459d91a764423666739e529269855924763b710`.

The exact MOD and HAK were installed through absent-only copies. Source and
native destination lengths and SHA-256 values are identical. The full
non-ignored `m2a-core` library/test suite passed, followed by exact
deterministic replays of every frozen r34, r35, r36, r37, r38 and new r39
candidate.

Proof-profile hashes are:

- binary bootstrap:
  `1b07ca363e16fdb30cbfbe0643030fb90afc5a783b3ab2738a61e3ecee7a1361`;
- Toolset:
  `ca8ef2207dd1a2ae08a238aebfa8c1ec34f58e3f30f9579f6f1fdcbf5effc4f8`;
- runtime:
  `f16b48fd2cf3f0e7731a82ad0c57e5efe11ad5fc2616a17c8ded02212008e150`;
- model-proof profile:
  `88e30bd7eeefcdbd9d5522805f220461a0f348cc5b7a65250b8f03e9f64a811b`.

## Accepted r39 Toolset result

The immutable r39 candidate later completed the Toolset part of the public
proof route:

- accepted proof:
  `proof-output/m0-r39-controllerless-identity-root-20260724/live/model-proof-120s-run-2/final-model-proof.accepted-at-736964523279.json`;
- accepted-proof SHA-256:
  `f023957be74395baaa8228ac7b7683118aaa127849cf385e4f5badd4b871694a`;
- Toolset capture SHA-256:
  `5820876b146e48b0be51057029796dda2892ca9ce514b21e0c9819f94ddb0f97`;
- recorded Toolset result: `modelVisibility=not_visible`,
  `proofCompleteness=verified`;
- NWN: `modelVisibility=not_tested`, because the public route correctly stops
  before runtime when the Toolset verdict is not visible;
- cleanup packet SHA-256:
  `eb1a470bd09e0073c9d237c8f8840950e3959296eb8a0c4eee584510fff10cc5`,
  with zero remaining Toolset and NWN processes.

The recorded candidate-bound `not_visible` result admits one new iteration
under the project gate. Its visual interpretation remains weaker than the
accepted r38 runtime absence: the r39 Toolset image is visually similar to the
r38 image that was classified `visible`. Therefore r39 is not evidence that
removing root controllers broke the model, and it is not a sufficient basis
for another one-field SkinMesh-header guess.

## Rejected r40 layout-order hypothesis

The initial r40 investigation considered moving the base model tree before all
local-animation data because r39 stores its base root at core offset
`0x6e280`. Read-only native binary comparison rejects that as a creature
invariant:

- `c_kocrachn`, `c_nulltail`, `c_vampire_f` and `c_eye`, which have no local
  animations in the inspected payloads, place the base root at `0xe8`;
- native animated `c_phod_horror_b` and `c_phod_horror_p` both place their
  42-entry animation pointer array at `0xe8`, first animation header at
  `0x190`, first animation root at `0x278`, and base root only later at
  `0xb5b74`;
- r39 follows the same animation-region-before-base-tree family.

Changing the writer's block order would therefore replace one native layout
with another, not repair a demonstrated ABI mismatch. No such change is
implemented.

## r39 post-result route diagnosis

The unresolved common boundary across r33 through r39 is the writer's
runtime-unproven `0x61` SkinMesh route. The iterations corrected real
structural differences—state topology, palette tail, dedicated Aurora Root,
SkinMesh parenting, unsupported animation scale and root controllers—but none
established a positive NWN draw for a writer-emitted SkinMesh.

The only project-owned, writer-produced and runtime-visible H1 witness is a
rigid `0x21` TriMesh. Native Aurora creatures also support rigid mesh parts
parented to animated nodes. Consequently the next candidate does not guess
another opaque SkinMesh field. It projects the same Meshy surface onto the
already mapped donor animation hierarchy as deterministic rigid triangle
groups, using the writer path with a positive owned runtime witness.

This does not claim that SkinMesh is invalid or permanently abandoned. It
separates two goals:

1. obtain one visible and animated Meshy creature through the proven rigid
   renderer family;
2. continue smooth SkinMesh ABI work only after the model/resource/fixture path
   has a positive candidate.

## Minimal intended r40 functional delta

r40 starts from the exact r39 source M0, H1 donor hierarchy, scale-free seven
clip set, controllerless dedicated Aurora Root, material, texture, Appearance
semantics and runtime-complete fixture. Only the deformation representation
changes:

1. for every source triangle, sum its three vertices' normalized weights by
   donor bone;
2. assign the complete triangle to the bone with the largest summed weight,
   breaking ties by stable binary-tree ordinal;
3. group triangles by assigned bone in deterministic ordinal order;
4. duplicate triangle vertices inside each group and transform positions,
   normals and tangents from Aurora-Root bind space into that bone's local bind
   space;
5. emit each group as one rigid `0x21` TriMesh parented to its assigned donor
   node;
6. emit zero SkinMesh nodes, skin palettes, inverse-bind arrays and vertex
   weight streams, while preserving the exact triangle count, UVs, texture and
   bind-pose world surface;
7. keep the 25-node rig-only type-5 animation trees and every non-scale
   translation/orientation track unchanged.

The expected visual quality is a coarse rigidly segmented first creature:
triangles do not blend between bones and seams may open during motion. That is
an explicit limitation, not a hidden success criterion. Candidate admission
requires deterministic replay, exact bind-pose world-geometry equality,
controllerless-root readback, zero SkinMesh nodes, nonempty rigid groups,
complete triangle preservation, seven scale-free clips and full MOD/HAK/2DA
readback. The owner performs the final Toolset/NWN proof after the candidate is
reported `ready_for_owner_proof`.

## r40 offline candidate result

The rigid triangle-group route is implemented and frozen as an exact candidate.
For every source triangle the converter aggregates its three normalized weight
rows, assigns the complete triangle to the strongest donor bone with
rig-tree-order tie breaking, duplicates the three corners and reconstructs
them in that bone's bind-local coordinates. Reapplying the parent bone's bind
world matrix reproduces the r39 bind-world surface within `0.0001` Aurora
units, and the exact UV triangle multiset is byte-identical.

Exact model readback proves:

- 25 rig nodes plus 20 rigid mesh nodes, for 45 base nodes total;
- 20 distinct animated parent bones;
- 20 `0x21` rigid TriMesh nodes and zero SkinMesh nodes;
- 1,569 source triangles and 4,707 duplicated triangle-corner vertices;
- the exact seven clip names, 25 rig-only type-5 nodes per state and zero
  animation scale controllers;
- a controllerless identity model root named `m2a_m0p40`;
- empty semantic writer/readback diff and deterministic repeated bytes.

The candidate was materialized exactly once at
`proof-output/m0-r40-rigid-triangle-groups-20260724`:

- MDL `m2a_m0p40.mdl`, 678,532 bytes, SHA-256
  `1809b05370e77f2c2559ec3e6fb354518bb5695ed48600954033175377fc0a40`;
- TGA `m2a_m0t01.tga`, 12,582,956 bytes, SHA-256
  `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`;
- `appearance.2da`, 6,901,359 bytes, SHA-256
  `07ca01286326a0fcdf206a376d332eca3bbc124f94b78eef0de7cbdee3291506`;
- HAK `m2a_m0r40.hak`, 20,163,103 bytes, SHA-256
  `c4ccf0b83f73106268186e6d72326ac5049792cf946613f42bce2fae420aed8a`;
- MOD `m2a_m0r40.mod`, 14,723 bytes, SHA-256
  `2a78d14a592e0eae92f08a9d27b7951f138d1232acdd4ce089c09d7ee8ce6df5`;
- lineage contract, 5,652 bytes, SHA-256
  `cf5a9267128a5b43055c14649209927fef01ada98553edabc75564d9661e31c2`.

The full non-ignored `m2a-core` library/test suite passed. Exact-input,
deterministic, runtime-complete replays then passed independently for every
frozen candidate from r34 through r39 and for r40. No Aurora Toolset, NWN,
native NWN directory, proof runner or proof profile was touched during the
r40 implementation. The candidate is `ready_for_owner_proof`; Toolset and NWN
visibility remain `not_tested` until the owner performs that proof.

## r40 owner-authorized native installation

After the candidate handoff, the owner explicitly requested installation into
the native NWN user directories. Both targets were confirmed absent before
copying, created without overwrite semantics, and hashed again afterwards:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0r40.mod`,
  14,723 bytes, SHA-256
  `2a78d14a592e0eae92f08a9d27b7951f138d1232acdd4ce089c09d7ee8ce6df5`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r40.hak`,
  20,163,103 bytes, SHA-256
  `c4ccf0b83f73106268186e6d72326ac5049792cf946613f42bce2fae420aed8a`.

Both native files are byte-for-byte identical to the frozen project outputs.
The MDL, TGA and appended `appearance.2da` are contained in the installed HAK;
they were not copied separately to `override`. No Toolset or NWN process was
started, adopted or controlled.

## r40 owner visual result and r41 iteration admission

The owner tested the exact installed r40 lineage. The immutable result is
recorded in
`documentation/evidence/m0-r40-owner-visual-result-2026-07-24.json`,
SHA-256
`1a043cb45bcf74f34c69dfc5352c996c297604f82a2114181220eec135b6f10a`.

The two proof axes are:

- Toolset: `modelVisibility=visible`, `proofCompleteness=verified`. The selected
  `Meshy M0 rigid-donor fixture` renders as the expected coarse collection of
  rigid pieces. The owner capture has SHA-256
  `82ae0e2635800702cb908f914153192c7458626222c46b082169b72cea7a1f42`;
- NWN: `modelVisibility=not_visible`, `proofCompleteness=verified`, from the
  owner's visual result for the exact runtime scene. Both native logs bind the
  observation to `Loading Module: m2a_m0r40` at `20:15:12`; the engine log has
  SHA-256
  `ca7403978ef35cea8133158c30534d969f9d52e9c687589775a05b9294867c98`
  and the client log has SHA-256
  `594e482dc936773644ffa2f086aaf39aafdc28fdb58ed27444a6602a635e95aa`.

This fresh candidate-bound NWN absence admits exactly r41 under the model
iteration gate.

## r40 Toolset/NWN divergence diagnosis

The following classifications keep facts separate from the implementation
inference.

### Aurora decompilation fact

`C:\Projects\New Folder\export\decompiled_all.c` shows that
`FUN_00a3b874` creates a separate runtime object for each local animation and
calls `FUN_00a3b994` for its root state node. `FUN_00a3b994` dispatches on the
serialized node content flags, and the node constructors recursively
materialize their serialized child arrays. The animation state tree is
therefore a real independent runtime structure, not unused file decoration.
The decompilation does not by itself choose one legal mesh-state family.

### Retail/resource facts

- r40 has 45 base identities: 25 rig nodes and 20 rigid `0x21` TriMeshes;
- each of its seven type-5 local-animation trees has only the 25 rig
  identities and omits all 20 renderable mesh identities;
- exact retail `c_Direwolf` has 30 base identities and all 42 type-5 states
  mirror all 30 name/number/parent identities as payload-free `0x01` nodes;
- exact retail `c_horror` has 27 base identities and all 42 type-5 states
  mirror all 27 identities in the same retail dummy family;
- the separately audited CEP rigid-placeholder family is also legal, but it
  uses empty `0x21` state placeholders and requires a distinct provenance
  contract. r41 does not mix that family into retail output;
- the class and empty-field warnings in the r40 NWN log predate this candidate
  family and are not model-specific fatal diagnostics. Exact project readback
  already proves the serialized creature and module fields used by this
  fixture.

### Implementation inference

The narrowest explanation consistent with the Toolset-visible base tree and
NWN absence is that r40 enters a type-5 state whose independent tree lacks all
20 mesh identities. r41 therefore restores the already implemented and
native-audited retail full-topology state family. This is a candidate-specific
runtime hypothesis; only the owner's NWN proof can establish its visual
effect.

## Minimal intended r41 functional delta

r41 preserves the exact r40 source and donor bytes, 25-node rig, seven clip
tracks, controllerless identity root, 20 bone-local rigid meshes, 1,569
triangles, 4,707 duplicated corners, texture, material, scale, placement,
Appearance row and HAK/MOD structure. The only model-behavior change is:

1. select `RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1` instead of the r40 rig-only
   projection;
2. project all 45 base name/number/parent identities into every type-5 state;
3. represent the 20 mesh identities as header-only `0x01` nodes without mesh,
   skin, geometry, raw MDX or controllers;
4. retain all 25 rig nodes and their existing translation/orientation tracks;
5. fail readback if any state has other than 45 identities or other than 20
   `m2a_seg_*` identities.

No base mesh, weighting/grouping, bind-pose transform or texture algorithm was
changed.

## r41 offline candidate result

The correction is implemented test-first. The exact differential test proves
that r40 and r41 have identical rigid mesh names, parent numbers, vertices,
normals, UVs and face vertex indices after readback. The full-topology oracle
then proves for all seven r41 states:

- binary local-animation type `5`;
- 45 identities per state;
- 20 matching `m2a_seg_*` identities per state;
- exact base/state name, number and parent identity;
- content flags `0x01` for every state node;
- no state mesh, skin, geometry or scale-controller payload.

The candidate was materialized exactly once at
`proof-output/m0-r41-full-state-rigid-20260724`:

- MDL `m2a_m0p41.mdl`, 694,772 bytes, SHA-256
  `c6a341ac8fc432d3d3f5862f67e1fcbcad72bb1ab3d2d241ce6421a976261f6b`;
- TGA `m2a_m0t01.tga`, 12,582,956 bytes, SHA-256
  `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`;
- `appearance.2da`, 6,901,359 bytes, SHA-256
  `0c40478729e416b46acd11115bd07c77f4d631217fe545b71fb9f11784068cf5`;
- HAK `m2a_m0r41.hak`, 20,179,343 bytes, SHA-256
  `4f14731659bc785313949acb98b772231ad45f56133d7c41c8fd59bf2b01459e`;
- MOD `m2a_m0r41.mod`, 14,733 bytes, SHA-256
  `8fd39e088605785864089f2ae48cfccbccb021c04f0a8f5f268c2f3c03837bc6`;
- lineage contract, 5,892 bytes, SHA-256
  `2a252f421077cacbe574fd2d8c779e8da7407ccfb89bf17ac590cec5449df434`.

Verification passed:

- full non-ignored `m2a-core` suite;
- exact deterministic r40 replay, proving the previous lineage is unchanged;
- exact deterministic r41 replay;
- exact V5 surface-preservation test;
- exact in-place oracle over the owned H1 witness plus the three audited native
  state-projection families;
- materializer argument/no-override contract and byte-for-byte output
  readback.

r41 is `ready_for_owner_proof`. No Toolset or NWN process was started or
controlled, no proof workflow was invoked, and r41 was not installed into the
native NWN directories. Visual status remains
`modelVisibility=not_tested`, `proofCompleteness=missing` independently for
Toolset and NWN until the owner performs the proof.

## r41 owner-authorized complete native installation

The owner clarified that each ready candidate must be delivered immediately as
a complete native test module, not only as project-local MOD/HAK artifacts.
Both exact r41 destinations were absent before installation. The files were
created with no-overwrite semantics and their destination lengths and SHA-256
values were verified against the frozen project sources:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0r41.mod`,
  14,733 bytes, SHA-256
  `8fd39e088605785864089f2ae48cfccbccb021c04f0a8f5f268c2f3c03837bc6`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r41.hak`,
  20,179,343 bytes, SHA-256
  `4f14731659bc785313949acb98b772231ad45f56133d7c41c8fd59bf2b01459e`.

Both native files are byte-for-byte identical to the frozen r41 sources. The
MOD already contains the complete Area and creature fixture and declares the
single ordered HAK `m2a_m0r41`; the HAK contains `m2a_m0p41.mdl`,
`m2a_m0t01.tga` and the full appended `appearance.2da`. No separate override
resources are required. No Toolset or NWN process was started or controlled.
