# Initial audit: local Neverwinter Vault corpus (2026-07-18)

## Scope and evidence boundary

This audit reads the owner-authorized, Git-ignored Neverwinter Vault corpus
registered in `neverwintervault-local-reference-corpus-2026-07-18.md`.
No external model was copied into a fixture or product HAK/MOD, installed in
NWN, or used as a replacement for a Meshy2Aurora runtime proof.

The purpose is to extract independent structural contracts and compare them
with the previously generated H1 binary artifacts. A result from an external
model is a format observation, not proof that the same model renders in the
owner's current game session.

## Corpus inventory

- 50 unpacked MDL files: 49 ASCII and one native binary MDL.
- The binary file is `c_direwolf.mdl` from the NWN2 creature conversion
  package. It is the first additional external binary reference that the own
  reader has successfully inspected.
- The 36 `Skinmesh Animals` models are ASCII, `classification Character`,
  `setsupermodel <model> NULL`, and have one or two skin nodes. They contain
  39--43 local animations, not a uniform fixed count of 44.
- The Geralt and Harpy HAKs also contain ASCII MDL resources. The Harpy HAK
  uses `c_harpy` as an animation supermodel; its four LOD variants inherit it.
- The Alvin model is ASCII, has `a_ba` as supermodel, six skin nodes and 40
  local animation definitions.

## appearance.2da observations

`Skinmesh Animals` supplies text rows, including:

- squirrel: row 1277, `NAME=Squirrel`, model `c_squirrel`, `MODELTYPE=S`;
- turtle: row 1278, model `c_turtle`, `MODELTYPE=S`;
- dire turtle: row 1279, model `c_direturtle`, `MODELTYPE=S`.

Alvin provides a full `appearance.2da`; its row 881 selects
`tw1_cr_kid_m01` with `MODELTYPE=F`. This is normal character-model variety,
not a prerequisite for the simple creature rows above.

These rows reinforce the earlier owner-observed working tortoise control: an
ordinary appearance row and `MODELTYPE=S` are sufficient for a valid creature
package. They do not identify a defect in the H1 appearance row.

## Animation contract observations

### Self-contained and supermodeled pairs

- `c_turtle` and `c_direturtle` each have exactly 42 local animation names.
  The set exactly matches native binary `c_Direwolf`, including `cpause1`,
  `cwalk`, `crun`, `cappear`, combat, damage, knockdown and disappear clips.
- `c_squirrel` has 43 local animations. Compared to `c_Direwolf`, it replaces
  `cspasm`, `cgetmidlp` and `cdisappearlp` with `ca2slashl`, `ca2slashr`,
  `ca2stab` and `cparrys`. Therefore 42 is an observed common set, not a
  parser-enforced universal cardinality.
- The NWN2 gargoyle provides a particularly useful A/B pair with identical
  static geometry (48 nodes, five skins): the supermodeled version has
  `setsupermodel c_gargoyle c_Orcus` and zero local animations; the "Non Super
  model" version has `supermodel NULL` and 42 local animations. This is a
  concrete reference design in which zero local clips are delegated to an
  animation supermodel; its runtime behavior was not independently re-proved
  in this audit.

### Comparison with own H1 artifacts

The own reader accepts all three binary artifacts below with zero diagnostics.

| Artifact | Base structure | Local animations | Skin report |
| --- | --- | --- | --- |
| `c_Direwolf` external binary | 30 nodes, 24 renderable meshes, `NULL` supermodel | 42 | none |
| H1 `rigid-isolation-v20` | 25 nodes, one renderable mesh, `NULL` supermodel | 7 | none |
| H1 `animation-gate-v19` | same one-mesh family, `NULL` supermodel | 7 | one `extended64` skin, 25 node-to-bone-map slots |

The H1 seven are `cpause1`, `cappear`, `cwalk`, `crun`, `ca1slashl`,
`cdamagel` and `cdead`; all are a subset of the 42 in `c_Direwolf`. The
remaining 35 names are absent. This is a high-value runtime hypothesis because
H1 has no supermodel from which to resolve the missing clips, but it is not yet
a proven visibility failure.

The `v19` skin has 1,334 vertex weight/reference records and an `extended64`
layout with 25 slots in the node-to-bone map. In contrast, inspected ASCII
skin controls have no unresolved bone names, at most four influences per
vertex, and 9--16 distinct bones per individual skin:

- squirrel: two skins, 9 and 10 bones;
- turtle/dire turtle: one skin, 12 and 14 bones;
- gargoyle: five skins, maximum 16 bones;
- Alvin: six skins, maximum 12 bones.

This makes the 25-slot H1 profile a separately testable risk. It does not
establish that the number alone is invalid: a slot count is not automatically
identical to the renderer's active-bone count, and Aurora evidence below has
not yet produced a renderer-side numeric rejection.

## Native binary reader evidence

The existing env-gated test
`direct_file_smoke_skips_cleanly_without_environment_variable` was run with
`M2A_REFERENCE_MDL_FILE` set to the corpus `c_direwolf.mdl`; it passed.

The own-reader summary reports:

- exact binary header ranges: 374,808 B total, 348,780 B core and 26,016 B
  appended MDX;
- `name=c_Direwolf`, `geometryType=2`, `classification=4`, `fog=1`,
  `animationScale=1`, `supermodelName=NULL`;
- 30 declared/reachable base nodes, maximum depth 6, 42 animation records;
- 24 meshes with `render=1`, `shadow=1`, texture `c_direwolf`; no skin;
- zero unsupported families and zero diagnostics.

The compact inspection adapter used for this report is local and ignored with
the corpus; it invokes the existing `m2a_core::inspect_binary_mdl` API and is
not product code.

## Reader limitation discovered

The separately materialized binary `incaxje.mdl` generated by Neverblender is
rejected by the own reader with:

`M2A-MDL-SKIN-VARIANT-AMBIGUOUS at 5196: skin node-to-bone pointer 0x00000000 matches neither explicit skin profile`.

This is a reader-support gap, not a finding that the Neverblender file is
invalid. Its runtime status was not established in this audit, so it must not
be promoted to a valid external binary control. The input is nevertheless a
useful regression candidate for an explicit zero-pointer skin-profile
diagnostic or support decision.

## Aurora First check

The local Aurora decompilation was used before drawing a runtime conclusion.

- `FUN_00a569c0` recognizes the model-level ASCII commands, including
  `newanim`, `setsupermodel`, `setanimationscale`, `classification` and
  `newmodel`; `FUN_00a56b54` registers their handlers.
- `FUN_00a5d758` parses animation `length`, `transtime` and `animroot` into
  the animation structure. The observed parser path dynamically appends each
  `newanim`; it contains no explicit test requiring 42 or 44 animations.
- `FUN_00a5cc98` parses skin `weights`, inverse bone rotations/translations
  and bone constants; the observed ASCII importer path dynamically grows these
  lists. It contains no explicit renderer-side numerical bone-limit check.

Consequently neither candidate is confirmed by the inspected decompilation
path: the importer accepts a variable animation count and variable weight
lists. Runtime selection/rendering can still impose a later constraint, which
requires a controlled game proof or a renderer-path location in the
decompilation.

## Current conclusion and next test

The corpus rules out several shallow explanations: `MODELTYPE=S`, a simple
appearance row, ASCII versus binary alone, and the mere presence of 44
animations are not sufficient diagnoses.

The most economical controlled next test is an own binary A/B pair, keeping
the known H1-v20 rigid geometry, texture, hierarchy and module fixed:

1. baseline: existing seven local animations, `supermodel=NULL`;
2. candidate: same model with the 42 observed creature animation names and
   the same existing root, using minimal own clips where the writer supports them.

The Toolset proof and the NWN proof must then be repeated under the existing
second-display standard. A separate later A/B should vary only the skin
profile: no skin versus legacy-sized map versus the current 25-slot map. No
external animation, skeleton or model data may be copied into either generated
candidate.
