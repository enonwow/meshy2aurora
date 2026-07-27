# M0 source-topology rigid hierarchy experiment — offline evidence (2026-07-22)

## Decision boundary

This record covers a code-only controlled experiment. It did not create or
install an `r31` MOD, HAK, proof profile, runtime sidecar, or candidate asset,
and it did not start Aurora Toolset or NWN. The result is deliberately
`OFFLINE_STRUCTURAL_EXPERIMENT_ONLY`; it is not runtime-visible evidence and
cannot alter the immutable r30 `not_visible/failed` observation.

The preceding payload-free differential found no exact positive consensus for
an isolated model/mesh/face/vertex-colour default change between the owned H1
and two retail-positive families versus frozen r30. Therefore this patch does
not toggle vertex colours, material values, model/header defaults, face
semantics, OPEN_M6 tails, or state animation type.

## Explicit non-default architecture

Product code now exposes the caller-owned
`SourceTopologyRigidExperimentProfileV3` and the non-admitting
`SourceTopologyRigidExperimentContractV3`. The binary writer has the separate,
non-default `SourceTopologyPreservingRigidExperimentV1` format profile. No
legacy M0/V2 builder selects it implicitly, and it is not connected to package,
MOD, HAK, fixture, appearance, or proof writers.

The repository-owned synthetic GLB contains exactly one scene/root, three
ordered identity transforms (including two nested transform dummies), one
unskinned indexed mesh primitive, and zero source skins or animations. The
mapping is deterministic:

- every reachable source transform becomes one base `0x01` dummy in exact
  source/traversal order;
- the root name alone is resref-normalized;
- a nullable non-root source name becomes `m2a_src_<source-order>`;
- the sole base `0x21` mesh attaches to the dummy for the source node that owns
  that mesh;
- all seven type-5 local states mirror the complete base name/part/parent
  identity as controller-free `0x01` nodes, with no mesh/skin/raw-MDX payload.

V3 intentionally restricts this first controlled witness to single-parent
identity transforms. Non-identity transform projection remains unresolved and
is not silently approximated. Historical flat M0/V2 remains unchanged and
rejects the nested source.

## Topology-only oracle

The experiment emits an in-memory flat control and hierarchy result through the
same new writer profile. The contract independently binds exact source
canonical SHA identity/length, full selected positions/normals/UV/indices plus
transform digest, source topology and transform digests, selected mesh/primitive,
source-to-output mapping, mesh attachment, final MDL identity, engine envelope,
state projection, output topology, and exact raw-MDX SHA.

The protected-fields digest holds constant all non-topology fields in scope:
model geometry/classification/supermodel/bounds/defaults; mesh type, material,
texture, render/shadow/default-sensitive fields; vertex-colour presence; face
surface/adjacency semantics; geometry/raw-pointer signatures; animation header
family; and exact raw MDX bytes. The synthetic experiment passes only when the
flat and hierarchy protected-field digests and raw-MDX SHA are identical.

The verifier reparses exact source and candidate bytes, validates topology and
state shape, regenerates both controls, and compares exact deterministic replay.
Caller-recomputed source/profile/contract changes cannot replace the separately
supplied expected profile trust root.

## Named adversarial coverage

`crates/m2a-core/tests/source_topology_rigid_experiment.rs` contains:

- `hierarchy_experiment_maps_owned_source_and_preserves_raw_mdx`;
- `hierarchy_experiment_rejects_source_inventory_order_names_and_transforms`;
- `hierarchy_experiment_nullable_names_are_order_derived_without_invented_joints`;
- `hierarchy_experiment_rejects_self_consistent_source_drift_and_profile_mixing`;
- `hierarchy_experiment_rejects_actual_topology_state_controller_and_payload_corruption`;
- `hierarchy_experiment_raw_partition_rejects_gap_and_unassessed_tail_before_replay`;
- `hierarchy_experiment_production_module_has_no_witness_locator_or_runtime_materializer`.

Those cases cover multiple scenes/roots, unreachable nodes, source-order/cycle
drift, duplicate names, transform drift, multiple mesh attachments/primitives,
skin, source animation, missing UVs, nullable names, caller-owned source
replacement, parent/child drift, CEP profile mixing, omitted/reordered/wrong
parts, wrong mesh-parent topology, wrong animation type, `0x21` state mixing,
valid-controller injection into a state dummy, protected render-field mutation,
and raw-MDX byte mutation. The raw-partition case inserts a real zero-byte gap,
rebases the concrete stream pointers and file/raw lengths, and separately
mutates canonical alignment tails; inspection rejects these before deterministic
replay with `M2A-MDL-ENGINE-ENVELOPE-RAW-GAP`,
`M2A-MDL-ENGINE-ENVELOPE-RAW-TAIL-NONZERO`, or
`M2A-MDL-ENGINE-ENVELOPE-RAW-TAIL-LENGTH`. MDL negatives mutate actual binary
bytes rather than only editing JSON claims.

The forced ignored suite additionally pins frozen r30 source SHA-256
`aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1` and proves
that the one-node r30 source cannot enter the new hierarchy experiment profile.
The existing exact r30 source/MOD/HAK/model replay and registry-pinned
`not_visible/failed` test remains the runtime-negative authority.

## Commands and results

```powershell
cargo test -p m2a-core --test source_topology_rigid_experiment -- --nocapture
# PASS: 7 passed, 0 failed

cargo test -p m2a-core --test model_pipeline v2_b_actual_ordered_hak_bytes_reject_unknown0_start_mdx_alias_and_trailing_before_replay -- --exact
# PASS: 1 passed, 0 failed

cargo --config "env.M2A_REQUIRE_RUNTIME_WITNESSES.value='1'" test -p m2a-core --test runtime_witness_conformance frozen_r30_negative_cannot_enter_the_hierarchy_experiment_profile -- --ignored --exact
# PASS: 1 passed, 0 failed

cargo --config "env.M2A_REQUIRE_RUNTIME_WITNESSES.value='1'" test -p m2a-core --test runtime_witness_conformance renderer_differential_against_exact_r30_is_payload_free_and_confounded -- --ignored --exact
# PASS: 1 passed, 0 failed

cargo check -p m2a-core
# PASS

cargo test -p m2a-core --test mdl_writer
# PASS: 32 passed, 0 failed

cargo test -p m2a-core --test profile_a
# PASS: 40 passed, 0 failed

cargo test -p m2a-core --test model_pipeline
# PASS: 22 passed, 0 failed

cargo --config "env.M2A_REQUIRE_RUNTIME_WITNESSES.value='1'" test -p m2a-core --test runtime_witness_conformance -- --ignored --nocapture
# PASS: 4 passed, 0 failed

cargo --config "env.M2A_REQUIRE_RUNTIME_WITNESSES.value='1'" test -p m2a-core runtime_evidence::tests::a_b_c_exact_r30_mod_hak_source_replay_and_registry_pinned_runtime_negative -- --ignored --exact
# PASS: 1 passed, 0 failed

cargo check --workspace
# PASS
```

The broad `cargo test -p m2a-core` run reached two unrelated failures in the
pre-existing GFF integration suite:
`swapped_field_indices_are_in_bounds_but_violate_canonical_encounter_order` and
`phase_nine_field_indices_layout_precedes_phase_ten_oversized_locstring_limit`.
This hierarchy patch does not change GFF code or tests; all unit tests before
that integration binary and every directly affected regression listed above
passed.

## Smallest future controlled delta

If an independent audit admits a single future live candidate, the justified
delta is only the base/state hierarchy policy: replace the collapsed flat
root-plus-mesh wrapper with the explicitly caller-bound source transform dummy
chain and attach the byte-identical rigid mesh payload to its source-owning
dummy. All protected writer fields, raw MDX, texture, appearance, fixture, and
module semantics stay fixed. This record does not itself authorize or
materialize that candidate.

Unresolved: runtime visibility remains unwitnessed; non-identity source
transforms are rejected by this controlled V3; and no claim is made that H1 or
stock binary defaults are serializer-equivalent. Neither protected witness
payload nor ASCII/H1 wrapper payload is copied or used by production code.
