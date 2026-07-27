# M0 r32: offline root-cause materials

Date: 2026-07-23

## Scope and gate

This is a read-only continuation of the model-visibility audit. It did not:

- start or operate Aurora Toolset or NWN;
- modify an installed module, HAK, override, development or user configuration;
- create another model/module/HAK/2DA iteration;
- change the immutable r31 model payload reused by r32.

The exact active candidate remains:

- module:
  `proof-output/m0-r32-corrected-container-20260722/generated/m2a_m0r32.mod`;
- module SHA-256:
  `a952e05746710c8461e74b38dbf93e6cc5c86314771330d68724579c11693573`;
- ordered HAK:
  `m2a_m0r31`;
- model:
  `m2a_m0p01.mdl`;
- model SHA-256:
  `fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6`;
- Toolset:
  `modelVisibility=visible`, `proofCompleteness=verified`;
- NWN:
  `modelVisibility=not_tested`, `proofCompleteness=missing`.

The NWN axis is not a negative model result. The exact r32 runtime launched,
but the central capture failed because another window obscured `nwmain`.

## Executive result

The newly collected evidence closes or strongly lowers the following
hypotheses:

1. missing `SkinMesh` prevents any creature draw;
2. the binary writer omitted the runtime node draw byte;
3. M0 lacks the native mesh inputs needed for triangle compilation;
4. M0 part numbers or parent topology are inconsistent;
5. controllerless type-5 state nodes necessarily hide base geometry;
6. model or mesh bounds exclude the actual M0 vertices;
7. r32 uses a malformed or still-sparse GIT creature envelope;
8. r32 `MODULE.ifo` contains an empty field label;
9. an obvious override/development collision replaces `m2a_m0p01` or its
   texture.

No offline source establishes a remaining MDL root cause. The highest-value
next observation is still the same exact r32 in NWN, not another converter or
model iteration.

## 1. Native renderer gates

The locally decompiled NWN client code at:

`C:\Projects\New Folder\export\decompiled_all.c`

contains the following relevant paths.

### Runtime draw eligibility

`FUN_00a5e508` rejects a runtime mesh node only when one of these conditions is
true:

- runtime node byte `+0x7a` is zero;
- the runtime node has no file-mesh pointer;
- file mesh `render` at `+0xdc` is zero;
- both vertex count at `+0x230` and MDX start at `+0x228` are zero.

The exact M0 has:

- `render=1`;
- `vertexCount=2380`;
- non-empty raw MDX;
- one texture;
- 1,569 faces.

### The `+0x7a` uncertainty is closed

`FUN_00a4d61c` initializes generic runtime node `+0x7a` to `1`.
`FUN_00a4cf44`, the mesh runtime constructor, then sets it from the file mesh:

`runtimeNode[0x7a] = fileMesh[0xdc] != 0`

Therefore exact M0 produces `+0x7a=1`. This value is runtime state derived from
the serialized `render` flag; it is not a missing independent writer field.

### Native triangle compilation inputs

`FUN_00a62b0c` requires the triangle-list path to have:

- position offset `+0x22c != -1`;
- UV0 offset `+0x234 != -1`;
- normal offset `+0x244 != -1`;
- mesh type `+0x224 == 3`;
- non-zero vertex and index counts.

Exact M0 satisfies all of these conditions. `FUN_00a62954` constructs the raw
16-bit triangle indices and assigns mesh type `3`.

Result: the currently known native pre-draw and triangle-compilation gates do
not explain an M0 no-draw.

## 2. Bounds and culling

The exact r31/r32 model readback reports:

- model bounds: `[-5,-5,-1] .. [5,5,10]`;
- model radius: `7`;
- mesh bounds:
  `[-0.54365003,-0.31950998,0] ..
   [0.54365003,0.31950998,1.892962]`;
- mesh radius: `1.8960401`;
- mesh average:
  `[-0.000943059,0.00050554797,0.93184364]`.

The model envelope safely contains the entire mesh with substantial margin.
All values are finite and the mesh has positive extent on every axis. A
simple out-of-bounds/frustum-culling explanation is therefore low
probability.

Source:

`proof-output/m0-r31-hierarchy-only-20260722/reports/r31-hierarchy-candidate-contract-v4.json`

## 3. Exact M0 compared with a runtime-visible owned model

The owned H1 v20 model written by the same Meshy2Aurora binary writer was
previously drawn by NWN, although its skin deformation was wrong.

H1 SHA-256:

`6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd`

Both M0 and H1 share:

- binary geometry type `2`;
- classification `CHARACTER` (`4`);
- fog `1`;
- `setsupermodel ... NULL`;
- fixed model bounds/radius family;
- mesh `render=1`;
- triangle-list mesh type `3`;
- the same writer implementation.

Important differences:

| Property | Exact M0 | Runtime-visible owned H1 |
|---|---:|---:|
| Base nodes | 4 | 25 |
| Max base depth | 3 | 7 |
| Mesh attachment | rigid to dummy anchor | rigid to root |
| Vertices | 2,380 | 1,334 |
| Animation type byte | 5 | 0 |
| Animation length | 1.0 s | 4.033 s |
| Nodes per animation | 4 | 24 |
| Controllers per animation | 0 | 49 |

This proves that the product writer and rigid creature mesh path can produce
geometry that NWN draws. It also proves that `SkinMesh` is not a universal
precondition for creature visibility.

H1 remains a positive draw witness, not a structural twin of M0. Its malformed
deformation does not validate the current skinning payload.

## 4. Part numbers and controllerless animation states

Exact M0 base topology is contiguous and internally consistent:

1. part `0`: `m2a_m0p01`, root dummy;
2. part `1`: `m2a_hier_1`, parent `0`;
3. part `2`: `m2a_mesh_anchor`, parent `1`;
4. part `3`: `m2a_seg_1`, parent `2`, renderable mesh.

Every one of the seven type-5 animation states mirrors the same four
name/part/parent identities. State nodes have no controllers.

Independent xoreos source reads the binary node number and uses it as the
animation-to-base-node lookup key. Its animation update changes position or
orientation only when the corresponding frame list is non-empty. Its NWN
model loader also retains base transforms when controllers are absent.

The project corpus independently contains retail `c_horror` and
`c_Direwolf` type-5 states that mirror the full base name/part/parent topology
as geometry-free dummy nodes.

Result:

- a part-number mismatch is not present in exact M0;
- a controllerless state is expected to leave base transforms unchanged;
- controllerless type-5 animation remains an implementation-risk area, but it
  is no longer a well-supported explanation for suppressing the base mesh.

Sources:

- `documentation/evidence/m0-runtime-conformance-state-projection-fix-2026-07-21.md`;
- <https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp>;
- <https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/animation.cpp>.

## 5. r32 creature container versus Toolset output

The installed diagnostic module inspected read-only was:

`C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_diag01.mod`

- byte length: `41,463`;
- SHA-256:
  `f8181522f8983fae8f42aebec3c233cb375e4775f5121c83c7a619ff2d92de22`;
- it had been saved by Toolset and contains a manually created stock
  `X2_DRIDER002` creature known to be visible.

Its GIT contains four creatures:

| Index | TemplateResRef | Tag | Appearance | Top-level fields |
|---:|---|---|---:|---:|
| 0 | `m2a_d_stock` | `stock_control` | 102 | 70 |
| 1 | `m2a_d_m0` | `candidate_m0` | 15100 | 70 |
| 2 | `m2a_d_h1` | `custom_control_h1` | 15101 | 70 |
| 3 | `x2_drider002` | `X2_DRIDER002` | 102 | 70 |

The exact r32 `m0_fixture` GIT creature also has:

- structure ID `4`;
- exactly 70 top-level fields;
- the same ordered field labels and GFF value types as the manually created
  `X2_DRIDER002` record.

The full recursive flattening of r32 `m0_fixture` and Toolset-saved
`candidate_m0` produced 230 entries on each side. The only differing scalar
payloads were:

- `TemplateResRef`;
- `FirstName`;
- `Tag`.

The following were identical:

- `Appearance_Type=15100`;
- `MaxHitPoints=13`;
- all 28 `SkillList` entries and their struct IDs;
- `ClassList` with `Class=12`, `ClassLevel=12`;
- position and orientation;
- field ordering, labels, GFF types, list sizes and nested struct IDs;
- all other creature-instance fields.

The r32 module-local UTC is more complete than the earlier Toolset-saved
diagnostic UTC:

- r32 UTC: `MaxHitPoints=13`, 28-skill list;
- old diagnostic UTC: `MaxHitPoints=1`, empty skill list.

Result: r32 is the first exact unchanged M0 model lineage with a
runtime-complete creature container. Historical r29/r31 absences used the
older sparse module family and did not isolate creature instantiation from
model drawing.

Sources:

- `crates/m2a-core/src/hierarchy_candidate.rs`;
- `crates/m2a-core/src/proof_module.rs`;
- `crates/m2a-core/tests/m0_r32_corrected_container.rs`;
- `proof-output/m0-r32-corrected-container-20260722/m0-r32-corrected-container-lineage-contract-v1.json`;
- `documentation/evidence/hook-horror-clone-runtime-complete-fix-2026-07-22.md`.

## 6. `MODULE.ifo` and runtime logs

The source r32 `MODULE.ifo` has:

- 55 fields;
- no empty label anywhere in its GFF tree;
- the same ordered 55 labels and GFF types as the Toolset-saved diagnostic
  module.

The engine warning:

`Empty field label while reading: CURRENTGAME:m2a_m0r32/MODULE.ifo`

does not establish an empty serialized label. The currentgame archive is
byte-identical to the exact source r32 module:

- currentgame SHA-256:
  `a952e05746710c8461e74b38dbf93e6cc5c86314771330d68724579c11693573`;
- source SHA-256:
  `a952e05746710c8461e74b38dbf93e6cc5c86314771330d68724579c11693573`.

The same warning occurred in earlier runs, including runs unrelated to this
model. It is consistent with an engine call to `GetFieldByLabel` using an
empty query label, not with a blank field stored in r32.

The r32 logs confirm:

- `Loading Module: m2a_m0r32`;
- exact runtime handoff to Area `m2a_m0a32`;
- a responsive NWN PID;
- no logged error naming `m2a_m0p01`, `m2a_m0t01`, appearance row 15100 or a
  missing MDL/TGA.

Repeated `Invalid Class` messages are not bound to the fixture. The exact same
sequence exists in the saved r26 log, while the r32 fixture itself parses as
class 12/level 12. They remain non-diagnostic environmental noise.

## 7. Resource resolution

Read-only inspection found:

- no `m2a_m0p01` or `m2a_m0t01` collision in user override;
- no matching collision in user development;
- exact ordered HAK singleton `[m2a_m0r31]`;
- byte-identical source and installed HAK;
- the expected appearance row and model resref in that HAK.

Old installed HAKs containing the same model resref are not ordered HAKs of
r32. An active NWSync/cache anomaly cannot be completely disproved offline,
but there is no direct evidence for one.

## 8. What a real static-mesh-to-creature converter must do

NeverBlender source confirms:

- `Skinmesh` extends `Trimesh` with weights for every vertex;
- only Blender vertex groups whose names match scene objects qualify as bones;
- weights below `0.001` are discarded;
- the four strongest influences are retained and normalized;
- the exporter does not invent an armature, bone mapping or weights.

Its animation exporter also preserves the imported hierarchy order and emits
animation nodes recursively from the chosen model root.

Therefore the proposed converter is feasible, but it is not merely:

`choose animated model + scale Meshy mesh`.

It requires an explicit retargeting pipeline:

1. select a compatible target skeleton contract;
2. transform and scale the source mesh into the target bind/rest space;
3. map or generate vertex weights for target bones;
4. cap and normalize weights to four influences;
5. preserve the target node names, part numbers, parents and animation
   topology;
6. emit valid inverse-bind/bone maps for `SkinMesh`;
7. bind animation clips to that exact node identity;
8. validate geometry and deformation separately.

Scaling solves size and alignment only. It does not solve skinning,
correspondence or animation retargeting.

Sources:

- <https://github.com/gyoerkaa/mdltools/blob/master/neverblender/nvb_node.py>;
- <https://github.com/gyoerkaa/mdltools/blob/master/neverblender/nvb_anim.py>;
- <https://nwn.wiki/spaces/NWN1/pages/38175669/MDL>.

The NWN format limits relevant to the converter include:

- at most four bones per vertex;
- at most 64 bones referenced by one skinmesh node;
- approximately 10,922 triangles per mesh node due to 16-bit index limits.

## 9. Ranked remaining uncertainty

### 1. Exact r32 runtime visibility

This is the dominant missing fact. The runtime was launched, but no
candidate-bound central NWN frame was captured. The same candidate must be
completed.

### 2. Closed-client state routing not represented by independent loaders

The native client copies animation type byte `5` into runtime state.
Independent loaders and retail structure strongly reduce the hypothesis that
empty type-5 states hide the base mesh, but only exact r32 runtime settles the
closed-client behavior.

### 3. Runtime resource/cache anomaly

The static resolution stack is correct. A transient client cache or NWSync
interaction remains possible but unproven.

### 4. A deeper native draw branch after current eligibility checks

The known node, mesh and triangle-list gates pass. There may be a later
closed-client condition not yet identified in decompilation. This becomes
worth tracing only after a fresh exact-r32 `not_visible` observation.

## 10. Decision

Do not build a new skinned conversion or r33 to answer the present no-draw
question.

Complete the exact r32 NWN lane first:

- if visible, the root cause of historical no-draw was the sparse creature
  container or another historical lane confounder, not the unchanged M0 MDL;
- if clearly absent in a candidate-bound, judgeable runtime frame, record
  `modelVisibility=not_visible` and only then choose one minimal diagnostic
  delta from the still-open hypotheses.

