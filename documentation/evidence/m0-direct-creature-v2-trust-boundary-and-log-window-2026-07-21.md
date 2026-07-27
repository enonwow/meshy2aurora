# M0 direct-creature V2 trust boundary and LogWindowV1 — offline evidence (2026-07-21)

## Outcome and hard boundary

This change admits a production M0 runtime fixture only through an explicit,
caller-owned `DirectCreatureRuntimeProfileV2`. The builder and verifier each
reparse the exact caller-supplied GLB bytes. The verifier additionally parses
the MOD, selects its exact single ordered HAK, extracts the MDL resource from
that HAK, and independently replays source-to-writer conversion. A report,
loose generated MDL, or profile copied out of the contract is not a trust root.

No r31 artifact, MOD, HAK, proof profile, installation, Toolset action, NWN
action, or live process was created or started. The exact r30 files remain a
project-owned negative regression: their structural result is
`STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED`, while their immutable runtime result
remains `modelVisibility=not_visible` and `proofCompleteness=failed`.

## Post-audit P1 closure map

- **P1-1 legacy runtime path:** both public canonical V1 builders and the
  canonical V1 writer return `M0-LEGACY-RUNTIME-ADMISSION-FORBIDDEN` before
  construction/output. The old CLI flag is parse-time rejected. Integration
  and example tests prove no canonical MOD/HAK packet is emitted.
- **P1-2 registry-pinned sidecar:** same-caller packet/capture input can create
  only `DiagnosticRuntimeObservationClaimV1`; that type is accepted by no
  admission API. The generic sidecar builder is private and the generic
  verifier was removed. The sole public runtime-observation admission function
  is the exact-r30 negative verifier backed by a private registry pin for full
  candidate digest, packet/capture identities and `not_visible/failed` axes. No
  signature or issuer is fabricated for future observations.
- **P1-3 envelope unknowns:** `unknown0=-1` and `startMdx=0` semantics are
  explicit. Actual-byte marker, alias/overlap and trailing attacks rebuilt
  into HAKs fail engine-envelope admission before replay.
- **P1-4 frozen r30 audit:** the forced test traverses MOD -> ordered HAK ->
  actual MDL, reproduces exact artifacts from the pinned source/full table,
  independently pins topology/envelope digests and verifies only the
  registry-pinned `not_visible/failed` runtime observation. Frozen bytes are
  never rewritten.

## A — SourceTopologyBindingV1

`DirectCreatureRuntimeProfileV2` binds schema/profile versions, source SHA-256
and byte length, a `sha256:<digest>` canonical identity, ownership origin,
conversion/writer/topology/envelope profile versions, and final model resref.
The source path is informational only.

`SourceTopologyBindingV1` records the default scene, every scene and ordered
root, every node in source order (nullable name, ordered parents/children,
mesh/skin and default-scene traversal), every mesh/primitive/skin in source
order, primitive attributes and multiplicity, the exact selected
scene/node/mesh/primitive tuple, source-to-output node mapping, mesh attachment,
final MDL SHA/length, and canonical versioned summary/digests. Its geometry
mapping digest covers exact source positions, normals, UVs, indices and node
transform, plus the final MDL geometry digest.

The current M0 profile is deliberately narrow: one scene, one root, one
reachable unskinned mesh, one primitive, zero ignored nodes/meshes, no source
joints, and no invented joints. A future rigged profile must be a new explicit
profile; it cannot weaken this one. A root resref normalization is the only
permitted output-name exception.

The named test
`v2_a_source_topology_trust_root_rejects_self_consistent_regenerated_source_and_mapping_drift`
passes and covers a rehashed mapping mutation, a fully regenerated and
internally self-consistent different GLB/profile lineage, and unreachable
source topology. None can cross the original caller-owned trust root.
`v2_a_table_driven_scene_root_name_child_mesh_primitive_skin_attribute_and_transform_attacks_fail`
adds table-driven changes to scenes, root multiplicity/order, nullable and
duplicate names, child topology, mesh/primitive multiplicity, skin/joints,
attributes and transforms. Representable changes still fail against the
independent original profile; inadmissible M0 shapes fail even with a newly
recomputed profile.

## B — DirectCreatureEngineEnvelopeV1

The envelope is recomputed from the MDL bytes extracted from the exact HAK
selected by the parsed MOD. It binds the binary header and complete core/raw
MDX ranges, SHA-256 values, geometry/classification/supermodel, bounds/radius/
scale, ordered base and state nodes, parent/part/name/flags/kind, controllers,
animation headers and correspondence, mesh material/texture/runtime fields,
face/index semantics, vertex-colour presence, raw MDX pointers/ranges/strides/
digests, and parser diagnostics. Diagnostics, unsupported families, trailing
bytes, incomplete ranges, invalid indices, and overlapping raw ranges are
fail-closed. `unassessedFields` must be empty for structural admission.
`unknown0` is explicitly constrained to the absent `-1` sentinel. `startMdx`
is explicitly constrained to the zero raw-origin marker for a non-empty M0
mesh; that marker may alias the first real raw stream, while real streams may
not overlap or alias each other.

The only positive structural enum is intentionally named
`STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED`. It cannot promote, reset, or infer a
visual result. Exact source-to-writer replay runs before accepting contract
resource hashes, so an attacker cannot replace the writer result by jointly
rehashing a foreign or modified MDL/HAK and its JSON reports.

The named test
`v2_b_engine_envelope_replay_rejects_self_consistent_rehashed_mdl_hak_and_swapped_envelope`
passes. It creates a real modified MDL, rebuilds a real HAK, recomputes the
model/HAK/source-mapping/envelope identities, and still fails deterministic
replay. It separately proves that a mutated and rehashed envelope cannot
replace exact HAK readback.

`v2_b_actual_ordered_hak_bytes_reject_unknown0_start_mdx_alias_and_trailing_before_replay`
mutates actual MDL bytes, rebuilds the HAK and resource hashes, and proves that
`unknown0`, `startMdx`, real-stream alias and trailing-byte attacks fail as
engine-envelope errors before and independently of deterministic replay.

The ignored/forced test
`a_b_c_exact_r30_mod_hak_source_replay_and_registry_pinned_runtime_negative`
verifies exact source, MOD, HAK, MDL, runtime packet, PNG and log lengths/SHA
values; parses the frozen MOD to ordered `[m2a_m0r30]`; extracts the actual MDL;
replays the source/profile to byte-identical MOD/HAK/MDL; independently checks
source topology and engine envelope; pins topology digest
`efb39972365d58f093c8d77a23a90595eaf956b8a2777d3d46c1949ba084df21`
and envelope digest
`2b99838587f1a7ec60cafc3edd01402941620710c1a4102a46e047e955e57c17`;
then admits only the registry-pinned r30 `not_visible/failed` observation. Its
full candidate digest is
`616b2064be4fde23ed1103ebbc5743ff499f1556c69b258a17e29679ff88b317`.

## C — diagnostic-only LogWindowV1

`LogWindowV1` is a pure project-side parser/diagnostic. `from_bytes` hashes the
provided bytes but does not attest caller-supplied file identity, timestamps,
PID or process start time; every such report therefore has `osAttested=false`
and can never establish runtime admission. It still binds the supplied pre/post
identities, lengths, hashes, modification/read timestamps, new suffix offsets
and hashes, PID/process start time, observation time, and the full exact V2
candidate: module, Area, entry/fixture including orientation, ordered HAK,
model, TGA, full appearance resource/table/physical row and scale. The
normalized token set must equal the candidate module/HAK/model resrefs. Only
complete token-bound resrefs in the new contiguous UTF-8 suffix are considered.

Rotation, truncation, file-identity change, PID/start-time mismatch, invalid or
out-of-order timestamps, invalid encoding, old-window tokens, suffix
substrings, bare rows, unrelated warnings, and absent tokens produce
`resource_load_not_observable`. Absence is never a loaded PASS. Only a defined
fatal diagnostic that names a bound candidate resource produces
`resource_load_failed`; complete positive observations produce
`resource_load_observed`. Every verdict remains diagnostic-only.

The following named C tests pass:

- `c_log_window_requires_new_contiguous_token_bound_positive_observations`
- `c_no_log_token_old_window_suffix_substring_bare_row_and_warning_are_not_loaded`
- `c_only_defined_fatal_diagnostic_naming_bound_resource_is_failed`
- `c_regenerated_rotation_pid_encoding_and_noncontiguous_attacks_are_not_observable`
- `c_candidate_token_and_timestamp_binding_fail_closed_without_promoting_load`
- `c_r30_structural_pass_is_permanently_separate_from_runtime_failure`
- `c_generic_caller_minted_observation_has_no_public_admission_api`

`inspect_diagnostic_runtime_observation_claim_v1` may parse internally
consistent same-caller packet/capture bytes, including `visible/verified`, but
its output is explicitly `diagnosticOnly=true` and carries no candidate or
runtime admission. `ProjectRuntimeEvidenceSidecarV2` can be produced by the
private builder only after the production MOD/ordered-HAK/source verifier and
the private registry pin both pass. No public generic build/verify function or
caller-supplied expected-observation type exists. Sidecar JSON and its public
digest are data integrity only, never admission. The structural axis remains
only `STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED` and cannot alter runtime axes.

## Admission APIs and legacy boundary

Only these product entry points can create or admit the V2 runtime contract:

- `build_meshy_m0_static_rigid_package_with_profile_v2`
- `build_meshy_m0_canonical_runtime_package_with_profile_v2`
- `build_meshy_m0_canonical_runtime_package_with_identity_and_profile_v2`
- `verify_m0_binary_runtime_fixture_contract_v2`
- `write_m0_canonical_runtime_proof_packet_with_profile_v2`

The V2 writer requires the expected profile and exact source bytes separately;
it never uses `contract.runtimeProfile` as its expected trust root. The
standalone `materialize_m0_runtime_candidate` example declares the profile
from explicit source bytes and passes both values through build, verify and
write.

The public canonical V1 builders, including the identity overload, now return
`M0-LEGACY-RUNTIME-ADMISSION-FORBIDDEN` before construction. The canonical V1
proof writer returns the same error before creating an output directory, and
the old `materialize_m6 --meshy-m0-static-source` CLI route is rejected during
argument parsing. This removes canonical V1 **runtime admission**, not every V1
byte-emission path. The isolated V1 builder and its writer may still emit an
offline MOD/HAK packet; that packet has no V2 contract, cannot pass the
canonical runtime writer and is rejected by the V2 writer. The named
integration test
`legacy_m0_construction_paths_cannot_obtain_v2_runtime_admission` covers every
old public runtime API construction/writer path; the example unit test
`legacy_m0_canonical_cli_route_is_rejected_before_any_emission` covers the old
CLI route. The existing WASM M0 V1 export may likewise emit an offline artifact,
but it has no caller-owned V2 profile/contract and is not an admission route.

## Read-only witness discipline

Production code does not open stock/BIF/CEP/local-reference assets and embeds
no protected node tree or default array. Reference witnesses exist only in the
ignored test target. Ordinary green reports them as ignored; witness validation
requires the explicit environment gate and fails closed when any exact witness
is absent.

CEP-shaped conformance also cannot self-authorize: the compatibility verifier
has no expected CEP trust root and rejects it, while the explicit verifier
requires actual provenance to equal an independently supplied expected
provenance. Production validates only the generic provenance schema; exact
witness names and hashes remain in the forced tests.

Forced command used for this evidence:

```powershell
cargo --config "env.M2A_REQUIRE_RUNTIME_WITNESSES.value='1'" test -p m2a-core --test runtime_witness_conformance -- --ignored --nocapture
```

Result: 2 passed, 0 failed, with the owned H1 witness and the three independent
native families inspected read-only/in memory.

## Central integration handoff

No code was changed in `C:\Projects\aurora-web`, and no project-local UI runner
was introduced. A future central evidence emitter may provide the following
caller-owned values to the pure project contract after its existing lifecycle:

1. exact pre/post snapshots of `nwclientLog1` and `nwengineLog` with OS-derived
   file ID, bytes, length, mtime and read time;
2. exact OS-derived adopted/started NWN PID and process StartTime before and
   after, tied to the same collection process;
3. exact module, ordered HAK and model resrefs plus SHA-256 identities;
4. observation timestamp, runtime packet identity and capture identity.

Only such a future central issuer can attest those OS facts and authenticate an
expected observation. That issuer integration is deliberately unimplemented:
there are no project-invented signatures, keys or caller-mintable seals.
Arbitrary project-side
`from_bytes` values remain pure parser inputs. The central runner must not turn
token absence into success. If it cannot provide one continuous exact window,
the project-side result is
`resource_load_not_observable`; the existing visual packet remains the owner of
`modelVisibility` and `proofCompleteness`.

## Verification and remaining integration gaps

Completed offline checks:

- `cargo check -p m2a-core`: PASS
- `cargo test -p m2a-core --test model_pipeline`: 22 passed
- focused A/B trust-root/envelope tests: 4 passed
- LogWindow/diagnostic-claim/r30 ordinary tests: 7 passed, exact-file test explicitly ignored
- exact r30 forced negative test: 1 passed
- forced runtime witness suite: 2 passed
- `cargo check --workspace`: PASS for `m2a-core` and `m2a-wasm`
- `cargo test -p m2a-wasm`: 20 passed
- `cargo test -p m2a-core -- --skip
  swapped_field_indices_are_in_bounds_but_violate_canonical_encounter_order
  --skip
  phase_nine_field_indices_layout_precedes_phase_ten_oversized_locstring_limit`:
  PASS across all remaining core/integration/doc tests; only the deliberately
  ignored exact-r30 and external-witness tests were skipped

The broad `cargo test -p m2a-core` run reached two pre-existing, unrelated GFF
expectation failures in `tests/gff.rs`:
`swapped_field_indices_are_in_bounds_but_violate_canonical_encounter_order` and
`phase_nine_field_indices_layout_precedes_phase_ten_oversized_locstring_limit`.
This change does not modify the GFF lane; a skip-filtered broad run is recorded
above.

Remaining gaps are explicit rather than silently defaulted:

- the central runner does not yet emit authenticated project observation
  inputs; this document is the integration handoff, not a local issuer,
  signature or transport implementation;
- the WASM M0 V1 route remains legacy/offline; a future browser runtime route
  must expose an explicit caller-owned V2 profile rather than auto-declare one;
- structural conformance still does not explain or reverse r30's runtime
  invisibility. Only one future candidate-bound live proof can establish that;
- no runtime-visible admission is claimed and no r31 lineage exists.
