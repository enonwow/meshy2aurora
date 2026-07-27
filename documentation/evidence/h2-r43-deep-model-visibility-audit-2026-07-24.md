# H2 r42/r43 — deep model-visibility audit

Date: 2026-07-24  
Status: diagnosis complete; `r43` remains `ready_for_owner_proof`; no `r44`
admitted

## Executive verdict

There is no second NWN failure for `r43`.

The exact `r43` MOD and HAK are absent from the native NWN user directories,
and the current engine/client logs contain only three loads of `m2a_h2r42`.
Therefore the only honest live state for `r43` is:

- Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`.

The owner's `r42` observation remains a real NWN failure, but it does not
isolate the H2 binary MDL. The same judgeable scene was expected to contain the
custom H2 and stock Hook Horror control, and the owner reported that none of the
models were visible. Changing the skeleton, mesh layout or 2DA again before
testing the frozen `r43` A/B scene would therefore be an unsupported guess.

The next operation is not implementation. It is one owner-run test of the exact
`r43` candidate:

- test MOD: `m2a_h2r43.mod`;
- Toolset module name: `Meshy2Aurora creature comparison`;
- Area: `m2a_h2a43`;
- custom H2: centre, Appearance row `15100`;
- stock Hook Horror control: player-left, Appearance row `102`.

## 1. Exact artifact and runtime binding

### Frozen canonical r43

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `m2a_h2r43.mod` | 20,280 | `ef2b48b80e4c01b455a04246c8e1da8f839171d9e9b4f9e835b846738f0a0c89` |
| `m2a_h2r43.hak` | 20,084,361 | `d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1` |
| `m2a_h2p43.mdl` | 599,784 | `5e169877c2f66d68f7bc8cae58e73087f346ff51b5652b37bc787e59c4c38d2a` |

Canonical directory:

`C:\Projects\meshy2aurora\proof-output\h2-r43-h1-v20-root-rigid-type0-20260724\generated`

Native read-only audit:

- `...\modules\m2a_h2r43.mod`: absent;
- `...\hak\m2a_h2r43.hak`: absent.

Current logs end at the previous candidate:

```text
23:26:02 Loading Module: m2a_h2r42
23:27:05 Loading Module: m2a_h2r42
23:27:20 Loading Module: m2a_h2r42
```

No `m2a_h2r43` load exists in either `nwengineLog.txt` or
`nwclientLog1.txt`. This disproves the premise that the latest model was
already tested and again invisible.

### r42 container mutation

The generated `r42` source MOD was:

- 20,284 bytes;
- SHA-256
  `c22741a72ab19a4b6e43620902ec9530e73d251c67de2beb23a2fef60e2b3c10`.

The last runtime load used a Toolset-repacked MOD:

- 40,865 bytes;
- recorded SHA-256
  `fe2408754ca10b71998aa64fe5cd8b1cd9347b335980db79f58b01472730da99`;
- last write `23:27:14`, before the final logged load at `23:27:20`.

The currently open Toolset process holds that final MOD, so the file cannot be
re-hashed without interacting with the human-owned session. The adjacent
Toolset backup is independently readable:

- `m2a_h2r42.BackupMod`;
- 39,094 bytes;
- SHA-256
  `b7b09995de69749df742eb4bdda9dbbb21a79bca4ca4fa995196326ef51e42d8`.

The backup confirms that Toolset added palette resources and changed the test
scene:

- module entry moved from `(10,10,0)` to approximately
  `(10.0944,6.96065,0)`;
- H2 moved from `(10,14.5,0)` to approximately
  `(10.0017,14.7943,0)`;
- the stock Hook Horror remained at `(7,14.5,0)`;
- a Toolset-created `X2_DRIDER003` creature was added at approximately
  `(13.7455,13.7347,0)` and retargeted to H2 Appearance row `15100`;
- ordered HAK remained `m2a_h2r42`.

The HAK/model lineage remained frozen, but the exact tested MOD/scene did not
remain the 20 KB canonical source. Future verdicts must bind the post-Toolset
MOD hash instead of silently carrying the source MOD hash forward.

## 2. Module, creature and 2DA audit

The canonical `r42` and `r43` modules pass the independent
runtime-complete multi-fixture reader:

- exact module and Area resrefs;
- one ordered HAK;
- two GIT creature structures;
- two module-local UTC blueprints;
- 70-field GIT creature envelope;
- 67-field UTC envelope;
- `MaxHitPoints=13`;
- 28-entry `SkillList`;
- one class entry, `Class=12`, `ClassLevel=12`;
- valid placement and orientation fields.

The field labels and GFF types match a Toolset-authored control creature.
This rejects the already-fixed sparse-creature defect from `m2a_van01` as the
current explanation.

The resource chain is also internally complete:

```text
GIT Appearance_Type=15100
  -> HAK appearance.2da row 15100
  -> MODELTYPE=S
  -> RACE=m2a_h2p43
  -> HAK m2a_h2p43.mdl
```

The control chain is independent of the custom model:

```text
GIT Appearance_Type=102
  -> base appearance.2da row 102
  -> MODELTYPE=S
  -> RACE=c_horror
  -> base-game c_horror.mdl
```

No loose `appearance.2da`, `c_horror` model or `r42`/`r43` model collision was
found in the active NWN user override/development paths.

The local Aurora decompilation independently reads `MODELTYPE` from the
Appearance row and takes the whole-model path for non-`P` types. It does not
show a rule requiring `MODELTYPE=S` creatures to be SkinMesh or to declare a
supermodel.

As an external implementation cross-check, xoreos loads a creature's
`Appearance_Type`, reads that row from `appearance.2da`, uses multipart loading
only for `MODELTYPE=P`, and otherwise loads the row's `RACE` model directly.
It also loads a UTC blueprint first and then overlays the GIT instance fields.
This agrees with the audited resolver chain; it is corroboration, not proof of
the proprietary NWN renderer.

Primary/reference sources:

- <https://raw.githubusercontent.com/xoreos/xoreos/master/src/engines/nwn/creature.cpp>
- <https://github.com/xoreos/xoreos-docs/tree/master/specs/bioware>
- <https://raw.githubusercontent.com/xoreos/xoreos-docs/master/templates/NWN1MDL.bt>

## 3. MDL audit: what r43 proves offline

The focused deterministic test for `r43` passes and confirms:

- one binary MDL;
- 25 base nodes: the mapped 24-joint H2 rig plus one rigid TriMesh;
- one root named exactly `m2a_h2p43`;
- root position and orientation bind controllers;
- the rigid mesh is a direct child of that root;
- 1,543 faces and 2,678 vertices;
- no SkinMesh nodes;
- seven local animation states;
- all seven states use the project-owned type-0, rig-only layout;
- exact HAK `appearance.2da`, MDL and TGA resource readback.

This is intentionally a rigid visibility candidate. It cannot prove correct
skinned deformation.

The explicit read-only witness suite also passes for:

- the owned H1 v20 rigid witness;
- retail `c_Direwolf`;
- retail `c_horror`;
- the CEP rigid-placeholder family;
- the exact historical r30 negative.

That differential has no single renderer-default value shared by all three
positive witnesses and absent from r30. In particular:

- vertex colors are present in retail models but absent from H1 v20;
- routine words and material defaults differ between witness families;
- face adjacency and surface-ID payloads differ by geometry;
- state topology differs between H1 type 0 and retail type 5.

Therefore the current corpus does not justify changing one opaque default,
adding vertex colors or replacing type 0 with type 5 as a claimed root-cause
fix.

## 4. Correction to the H1 v20 baseline

H1 v20 is not evidence of a correct creature model.

The historical runtime image shows a large white/grey, severely deformed blob
at the fixture position. It proves only that NWN issued a draw for that rigid
payload. It does not prove correct mesh geometry, transforms, material,
animation or creature quality.

Consequently, `r43` matching the H1 root/state topology means:

> r43 is a legitimate draw-path experiment.

It does **not** mean:

> r43 is known to be a correct NWN creature model.

If r43 draws as another distorted blob, the remaining problem is the common
binary mesh/MDX or transform path. If r43 is entirely absent while the stock
control is visible, the H1-compatible topology is insufficient and the next
audit must compare the custom binary payload against a clean retail whole-model
creature.

## 5. Ranked diagnosis

### P0 — current observation does not isolate the MDL

Confidence: high.

`r43` has not run. In `r42`, the stock control was also reported absent.
Therefore the current evidence cannot distinguish:

- wrong runtime view/scene;
- a creature-instantiation problem;
- a repacked-MOD mismatch;
- custom MDL rejection.

The first three must be eliminated with the frozen two-control r43 scene before
the writer changes again.

### P1 — H1 is only a corrupt draw witness

Confidence: high.

The root/state layout copied by r43 is not a clean positive visual oracle. A
future custom-only failure or corrupted draw should focus on the one-large-rigid
mesh runtime path: transforms, face/index interpretation and MDX stream
semantics. It should not be treated as evidence that Meshy must generate a new
source model.

### P2 — custom binary MDL/MDX remains possible, but only after a clean A/B

Confidence: medium, conditional.

This branch becomes active only if the exact r43 frame shows:

- stock Hook Horror visible;
- H2 absent or corrupted.

At that point the next minimal delta should be selected from a byte-level
comparison with a clean retail direct creature, not another blind
root/skeleton rewrite.

### Rejected as current root cause

- missing r42 HAK, 2DA row, MDL or TGA;
- wrong `MODELTYPE`/`RACE` mapping;
- sparse GIT/UTC runtime envelope;
- the recurring unrelated invalid-class log warnings;
- a global requirement for SkinMesh or a supermodel;
- missing vertex colors alone;
- another Meshy source generation;
- another model iteration before r43 has a fresh owner-bound result.

## 6. Decisive next gate

Do not create `r44`.

Test the exact frozen `m2a_h2r43.mod` and `m2a_h2r43.hak` together. If Toolset
re-saves or rebuilds the module, hash the resulting MOD and bind the verdict to
that new container hash; do not call it byte-identical to the canonical
20,280-byte source.

Interpret one clear owner frame as follows:

| Stock row 102 | H2 row 15100 | Meaning | Next work |
| --- | --- | --- | --- |
| absent | absent | scene/module/instance lane not isolated | diagnose placement, instance creation and tested MOD; do not edit MDL |
| visible | absent | custom MDL/MDX rejection isolated | byte-level retail-vs-H2 renderer differential |
| visible | corrupted blob | draw accepted, geometry/transform wrong | audit common rigid mesh/MDX transform path |
| visible | visible and correct | visibility gate closed | proceed to SkinMesh/deformation work |

This is the shortest evidence-backed route to a real fix. Any model change
before this gate would destroy the only useful A/B experiment currently
prepared.

## Verification performed during this audit

```text
cargo test -p m2a-core --test h2_r43_h1_root_layout_candidate
  1 passed

cargo test -p m2a-core --test binary_creature_multi_fixture_module
  5 passed

M2A_REQUIRE_RUNTIME_WITNESSES=1
cargo test -p m2a-core --test runtime_witness_conformance -- --ignored
  4 passed
```

No Toolset or NWN session was started, adopted, controlled, saved, closed or
captured. No native proof artifact was installed or modified.
