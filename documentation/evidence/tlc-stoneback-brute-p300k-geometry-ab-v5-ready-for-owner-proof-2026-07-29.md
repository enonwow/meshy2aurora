# TLC Stoneback Brute P300K Geometry A/B V5 — owner verified

1. Test-module filename: `m2p3jd0eeb135c.mod`
2. Module name shown in Toolset: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Status: `owner_runtime_verified`

## Purpose and scope

This is a comparison candidate for the bright point artifacts visible on the
Stoneback Brute in NWN. It changes only the P300K experimental geometry
degeneracy policy. Product, P100K, texture cleanup, material authoring,
animation authoring, Appearance mapping and the stable V4 artifacts are not
modified.

The exact source is:

`sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb`

- bytes: `20,256,180`;
- SHA-256:
  `0eeb135c1077f2207b1f362222829a2493c003c64276db98633cf2af5c8385b3`.

## Why degenerate-triangle removal exists

The stage is a safety boundary, not an optimization. A truly zero-area face
has no usable plane normal. Keeping it may cause a divide by zero, NaN or
unstable face-plane data, invalid collision/render calculations, or malformed
binary MDL readback. Repeated-index and exactly collinear triangles therefore
must still be rejected.

The previous rule used the fixed absolute test:

`length(cross(b - a, c - a)) <= 1e-5`

or its squared equivalent `<= 1e-10`.

The cross-product magnitude is twice the triangle area and changes with the
square of model scale. The fixed epsilon consequently did not mean “zero”.
For this dense Meshy mesh it classified valid, finite microtriangles as
malformed and punched small holes in the rendered surface.

## Safer V5 rule

For the P300K comparison route only:

1. promote the input `f32` coordinates to `f64`;
2. calculate the cross product in `f64`;
3. reject a face when the result is non-finite or its squared length is
   exactly zero;
4. retain every finite non-collinear face regardless of absolute area;
5. apply the same policy in source sanitation, derived-rig surface validation,
   runtime sanitation, writer planning, emission and semantic readback.

This preserves protection against truly invalid face planes without applying
a scale-dependent “small triangle” policy. A future general product policy
should use either this exact rule or a separately specified scale-relative
machine-precision tolerance; it must not restore a fixed world-unit area
threshold.

## Offline A/B result

| Property | A: cleanup V4 | B: geometry V5 |
|---|---:|---:|
| Source GLB SHA-256 | `0eeb135c…b3` | `0eeb135c…b3` |
| Source triangles before sanitation | 296,276 | 296,276 |
| Profile conversion triangles | 290,319 | 296,276 |
| Written/read-back MDL triangles | 290,318 | 296,276 |
| Triangles missing from final MDL | 5,958 | 0 |
| Mesh streams | 14 | 14 |
| Rig nodes | 25 | 25 |
| Required NWN Creature states | 42 | 42 |
| Texture TGA SHA-256 | `ac6aceb3…5fbc` | `ac6aceb3…5fbc` |

The old source sanitizer removed `5,957` faces and the later runtime/writer
boundary removed one more. V5 preserves all `296,276` source faces; therefore
this exact source contains no truly zero-area or exactly collinear triangle.

The following reports are semantically identical between A and B:

- animation completeness and all 42 routed states;
- animation behavior;
- animation kinematics;
- animation events and event timing;
- texture artifact cleanup statistics and output pixels.

The material still comes from the same source slot `Material_1`, uses the same
authoring policy and binds the byte-identical repaired base-color TGA. Only the
fresh resource names change from the V4 `h` lineage to the V5 `j` lineage.

This result proves the intended topology delta offline. It does not yet prove
whether NWN's bright points disappear; that visual A/B verdict belongs to the
owner.

## Exact application route

The candidate passed the real application pipeline:

`Studio Worker -> m2a-wasm -> m2a-core -> owned MDL/TGA/2DA/HAK/MOD writers -> canonical result projector`

The headless Worker/WASM integration completed successfully. Core regression:

- `m2a-core --lib`: `79 passed`, `1 ignored`, `0 failed`;
- `mdl_writer`: `43 passed`, `0 failed`;
- `model_pipeline`: `28 passed`, `1 ignored`, `0 failed`;
- Studio TypeScript typecheck: passed;
- exact P300K Worker/WASM integration: `1 passed`, `0 failed`.

The ignored tests require separate external runtime/reference fixtures and are
unrelated to this comparison.

## Identity and readback

- model: `m2p3jm0eeb135c`;
- texture: `m2p3jt0eeb135c`;
- HAK: `m2p3jh0eeb135c`;
- MOD: `m2p3jd0eeb135c`;
- Area resref: `m2p3ja0eeb135c`;
- Creature resref: `m2p3jc0eeb135c`;
- Appearance row: `15100`;
- semantic MOD readback: `PASS`;
- HAK resources: exact Appearance 2DA, MDL and TGA;
- model deformation: `SKIN`;
- animation completeness: `true`.

## Frozen Studio artifacts

Canonical packet:

`proof-output/tlc-stoneback-brute-p300k-geometry-ab-v5/studio-export`

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2p3jd0eeb135c.mod` | 15,193 | `da60bc35b45f7ce53753ed91d6c2c13ce09192b6d63143acf2fffca2c94cf1df` |
| `m2p3jh0eeb135c.hak` | 47,922,458 | `90d79114f0bc725e91f54657dc39f7491862ab4c0f54e032703ac4fddd6c9853` |
| `m2p3jm0eeb135c.mdl` | 24,243,620 | `c8b173a59a576911ff938b7287f823694474be838a81cb55c370c8c3258eb6e9` |
| `m2p3jt0eeb135c.tga` | 16,777,260 | `ac6aceb3f2809c5ffe1170d2525df672fb077855cf8bc1d78f24241596f05fbc` |
| `inspection.json` | 1,242,998 | `5e44075533cea632431d2ea7a354d4ab82af96c12c764c039a21497b5c3bf27a` |
| `conversion-manifest.json` | 3,197 | `6ff80ccc91efb8fc9bc366c140eac0be15b365aef6e676ddd2669228cabf4393` |
| `summary.json` | 1,735 | `fddf29abe3b189dfce6a745ac7c79ece8f9608b4df9e60026f5be060f1c3f501` |

## Native installation

Both exact destinations were absent before installation. They were copied
without overwrite and verified byte-identical afterward:

| Artifact | Native destination | Verified SHA-256 |
|---|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2p3jd0eeb135c.mod` | `da60bc35b45f7ce53753ed91d6c2c13ce09192b6d63143acf2fffca2c94cf1df` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2p3jh0eeb135c.hak` | `90d79114f0bc725e91f54657dc39f7491862ab4c0f54e032703ac4fddd6c9853` |

No Toolset or NWN process was started, adopted or controlled.

## Owner A/B test

Use current V4 observation as A. For B open exact
`m2p3jd0eeb135c.mod`, then Area
`Meshy2Aurora M0 binary vertical-slice area`.

The Creature template is `m2p3jc0eeb135c`, Appearance row is `15100`, and the
fixture is placed directly in front of the player at `[10.0, 14.5, 0.0]`.

Compare the head, upper back and shoulders, chest and arms at approximately the
same camera distance as the V4 capture. Record:

- `modelVisibility = visible | not_visible | not_tested`;
- `proofCompleteness = verified | failed | missing`;
- `geometryArtifactResult = fixed | improved | unchanged | worse | not_tested`;
- whether any former bright point became a visible hole or disappeared.

Recommended promotion rule: adopt the exact finite/non-collinear policy beyond
this P300K demo only if the owner reports `fixed` or `improved` without new
rendering defects. Otherwise retain the frozen evidence and continue diagnosis
without changing the stable routes.

## Owner result — 2026-07-29

The owner tested this exact immutable V5 lineage and confirmed:

| Lane | `modelVisibility` | `proofCompleteness` |
|---|---|---|
| Aurora Toolset | `visible` | `verified` |
| Neverwinter Nights | `visible` | `verified` |

The owner described the model as appearing correctly and looking very good in
both environments. The former bright points/holes are fixed:

`geometryArtifactResult = fixed`

This result is bound to the exact V5 output hashes listed above. It does not
promote another model, HAK, MOD or texture merely because it shares the same
source asset or route name.

The visual A/B result confirms the offline diagnosis for this Stoneback
lineage: the fixed absolute epsilon removed valid finite microtriangles. It
does not by itself prove the cause of every historical model defect, although
the corpus replay documented below shows that the same legacy rule also
removed valid faces from both canonical P100K sources.

## P0 policy promotion and corpus replay — 2026-07-29

After the positive owner result, the active Creature routes Product 20K, P100K
and P300K were unified on `ExactFiniteNonCollinear` at all geometry boundaries:

1. source sanitation;
2. derived-rig/reference-surface validation;
3. runtime face-plane sanitation;
4. binary MDL face-plane planning and writing.

Frozen legacy Product bundle adapters retain their historical epsilon behavior
to avoid silently changing old byte lineages.

The local canonical H1 Creature corpus replay found:

| Asset | Raw | Exact | Legacy | Valid microtriangles recovered |
|---|---:|---:|---:|---:|
| H1 humanoid | 1,556 | 1,556 | 1,556 | 0 |
| H2 sentinel | 1,543 | 1,543 | 1,543 | 0 |
| Powrotnik Product 20K | 19,892 | 19,892 | 19,892 | 0 |
| Powrotnik P100K | 103,290 | 103,290 | 97,633 | 5,657 |
| Veiled P100K | 102,335 | 102,335 | 99,812 | 2,523 |
| Stoneback P300K | 296,276 | 296,276 | 290,319 | 5,957 |

End-to-end in-memory replay also preserved the exact triangle count between
input inspection, runtime geometry and the written MDL:

- Product 20K: `19,892 -> 19,892 -> 19,892`;
- P100K target class: `102,335 -> 102,335 -> 102,335`;
- P300K V5: `296,276 -> 296,276 -> 296,276`.

`P100K` remains an experimental Meshy generation target, not a product budget.
The route accepts an explicit bounded source/output envelope through 110,000
triangles so a normal Meshy overshoot is never corrected by deleting valid
microgeometry. The shared product budget remains exactly 20,000 triangles.

## Superseding product-budget amendment — 2026-07-29

The final sentence above records the policy active when the immutable V5 packet
was built and proved. A later direct owner decision on the same date superseded
that product limit with `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000`. V5's
source, hashes and owner proof remain unchanged; only the admission policy for
new product conversions changed.
