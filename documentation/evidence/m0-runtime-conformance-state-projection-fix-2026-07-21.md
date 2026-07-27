# M0 runtime conformance/state-projection fix — offline evidence (2026-07-21)

## Outcome and boundary

The product writer now has a reusable, explicit state-tree projection policy.
The implementation does **not** allocate or materialize another `rNN` lineage,
MOD, HAK, proof profile, installation, Toolset session, or NWN run.  It is an
offline code fix intended to support one later proof candidate after review.

The general binary MDL reader remains corpus-tolerant.  Family restrictions are
enforced only when a caller selects a writer/runtime conformance profile.

## Adversarial admission review closure

The first implementation was blocked by three valid review findings:

1. the M0 runtime fixture contract hashed the MDL but did not bind or recompute
   its state-projection family;
2. the CEP conformance API accepted CEP-shaped trees without carrying the exact
   audited provenance required by the writer;
3. an ordinary test invocation could return green after silently skipping a
   missing external witness set.

The implementation below closes all three.  The negative contracts include a
self-consistent historical `0x21` candidate with recomputed resource identities
and structural digest, missing/wrong CEP provenance, and both branches of the
absent-witness policy.  No live action or candidate materialization was used to
close the review.

## Independent witnesses

No protected model payload was copied into source or test fixtures.  The
differential test reads the following exact resources in place and retains only
metadata/structural summaries:

| Family | Exact witness | Payload-free structural fact used by the oracle |
| --- | --- | --- |
| Retail direct creature | `c_horror`, SHA-256 `2faf553a0665da200b232bd52d03c0e1d79b88959cabdbe840f35f16e5878c8e`, exact in-memory BIF range | 27 base nodes, 21 trimeshes, depth 7; 42 type-5 states; every state mirrors all 27 base name/part/parent identities as `0x01` dummies with no mesh/skin payload. |
| Retail direct creature | `c_Direwolf`, SHA-256 `121b63cd51ff46c3633c740951752110bebdeab7db139719ff6ff7db077f0fd9` | 30 base nodes, 24 trimeshes; 42 type-5 states; complete all-`0x01` topology projection with no state mesh payload. |
| Owned positive runtime witness | H1 rigid v20 `m2a_m6p01`, SHA-256 `6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd` | NWN drew the owned rigid mesh. It has 25 base nodes, while each of seven historical type-0 states contains the 24 root/joint dummies and omits the mesh leaf. This supports state/base identity as a relevant axis, not direct serializer equivalence. |
| CEP placeholder family | `c_phod_horror_b` in `cep3_core1`, SHA-256 `62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a` | 27 base/state nodes and 42 type-5 states; 21 rigid base-mesh identities use empty `0x21` state placeholders and six nodes use `0x01`. This is a separate legal family. |

The two retail witnesses and H1 agree that `0x21` is not a universal
requirement.  CEP proves the converse: globally deleting `0x21` would also be
wrong.  The code therefore rejects family mixing instead of turning either
corpus observation into a global rule.

## Implemented architecture

### State projection

`MdlStateProjectionProfileV1` has two explicit policies:

- `RetailDirectCreatureType5DummyV1` is the M0 product policy.  The exact base
  topology remains root -> `m2a_seg_1`; each type-5 state mirrors both
  name/part/parent identities as `0x01` nodes.  Animation trees contain no
  mesh, skin, vertex, face, texture, or raw-MDX payload.
- `CepRigidPlaceholderV1` preserves empty `0x21` rigid placeholders, but only
  with explicit, well-formed caller provenance that runtime conformance
  compares against an independently supplied expected binding. It rejects
  missing/mismatched provenance and skin segments. It is not the M0 default.

The selected profile and optional provenance are recorded in the writer report.
Retail verification rejects any CEP provenance. CEP admission uses
`verify_direct_creature_state_projection_with_expected_provenance_v1` and
requires the actual report binding to equal a separately supplied expected
binding. The compatibility verifier has no expected CEP trust root and therefore
rejects CEP-shaped input. Exact witness identities live only in the ignored,
forced tests; production code embeds no witness name/hash list. The
profile-bound oracle then verifies type, node count, name, part number, parent
identity, content flags, and absence/shape of state payloads.

The historical implementation at this checkpoint used
`M0RuntimeFixtureContractV1`. It has since been superseded for runtime
admission by the explicit caller-owned `M0RuntimeFixtureContractV2` documented
in `m0-direct-creature-v2-trust-boundary-and-log-window-2026-07-21.md`; V1
construction is now legacy/offline and inadmissible. The original contract
persisted
`stateProjectionProfile=RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1`, absent CEP
provenance, the complete payload-free `DirectCreatureStructuralSummaryV1`, and
the SHA-256 of its canonical JSON.  Both the builder and verifier inspect the
exact MDL bytes and call the Retail oracle; the verifier recomputes and compares
the summary and digest instead of trusting packet fields.

### Full runtime appearance binding

`M0AppearanceTableBindingV1` now records scope, input/output physical row
counts, appended physical row, input/output byte lengths and SHA-256 values,
and the prefix-preservation fact.  `FullRuntimeAppendV1` requires:

1. the complete caller-supplied `appearance.2da` as the exact output prefix;
2. exactly one appended physical row;
3. the fixture `Appearance_Type` to equal that appended physical row;
4. byte length/hash and parsed row-count agreement.

The runtime fixture validator recomputes these conditions from HAK bytes.  An
`IsolatedToolsetVerticalSlice`, truncated table, changed prefix, wrong output
count, or wrong fixture row is rejected.  Isolated packages may still exist as
Toolset controls, but they cannot pass the general runtime-profile validator.

## Differential and negative tests

The test suite locks all of the following without generating a candidate:

- exact M0 base semantic digest unchanged relative to r29;
- exact raw MDX bytes unchanged relative to r29;
- exact TGA bytes, material texture resref, geometry, and base hierarchy
  unchanged;
- retail output accepted by the retail oracle and rejected by the CEP oracle;
- provenance-bound CEP output accepted only by the CEP oracle;
- missing state leaf; wrong name, part, or parent; wrong `0x01`/`0x21` family;
  mesh payload in a retail dummy; wrong animation type; and mixed profiles;
- missing/wrong CEP provenance and unproven CEP skin use;
- changed M0 profile, changed structural digest, and a historical r29 `0x21`
  tree whose MOD/HAK/MDL/mesh/summary identities and digest were all recomputed;
  the latter still fails the Retail oracle with
  `M2A-MDL-CONFORMANCE-PROFILE-MIXED`;
- isolated/truncated appearance table, changed prefix, wrong physical row, and
  wrong row count.

The witness test also distinguishes the unsupported reference paths:

- the ASCII `c_squirrel.mdl` is correctly rejected by the binary reader;
- the preserved NeverBlender -> CleanModels binary remains a reference-only,
  runtime-inconclusive compiler output and is not silently promoted to the
  product oracle.

Both external-witness tests are explicitly ignored in an ordinary test run.
The normal suite reports them as `ignored` and runs a separate negative policy
test proving that an absent optional witness produces an explicit skip state,
whereas a required witness fails closed.  Corpus validation can be claimed only
from this exact forced command:

```powershell
$env:M2A_REQUIRE_RUNTIME_WITNESSES='1'
cargo test -p m2a-core --test runtime_witness_conformance -- --ignored
```

H1 is not copied as a wrapper: its 23-joint hierarchy and motion originate in a
different user source, its visible artifact used historical type-0 headers, and
its exact writer revision was not cryptographically bound.  Its structural
summary is a positive runtime indicator only.  The reusable product mechanism
is projection of the caller-owned base topology, not transplantation of H1 or
stock nodes/animations.

## Code and verification entry points

- `crates/m2a-core/src/mdl/writer_types.rs`
- `crates/m2a-core/src/mdl/write_binary_mdl.rs`
- `crates/m2a-core/src/mdl/runtime_conformance.rs`
- `crates/m2a-core/src/model_pipeline.rs`
- `crates/m2a-core/tests/mdl_writer.rs`
- `crates/m2a-core/tests/model_pipeline.rs`
- `crates/m2a-core/tests/runtime_witness_conformance.rs`

Targeted offline command:

```powershell
cargo test -p m2a-core --test mdl_writer --test model_pipeline --test runtime_witness_conformance
```

Result: `32 + 16 + 1 = 49` tests passed and the two external-witness cases were
reported as `ignored`.  This ordinary result is not witness validation.  The
forced command above separately passed `2/2` and read all four exact families
in memory.  No Aurora/Toolset/NWN process or live artifact was created.

The broad `cargo test -p m2a-core` run reached unrelated pre-existing GFF test
failures (`swapped_field_indices_are_in_bounds_but_violate_canonical_encounter_order`
and `phase_nine_field_indices_layout_precedes_phase_ten_oversized_locstring_limit`).
The M0/MDL/conformance suites themselves passed; this change does not modify the
GFF lane.

Final offline verification also passed:

- `$env:M2A_REQUIRE_RUNTIME_WITNESSES='1'; cargo test -p m2a-core --test runtime_witness_conformance -- --ignored`
  (`2/2`, so no witness was silently skipped);
- all remaining `m2a-core` tests when only the two named, unrelated GFF cases
  were excluded;
- `cargo test -p m2a-wasm` (`20/20`) and `cargo check --workspace`;
- the immutable r29 lineage and runtime-profile contract scripts;
- `cargo fmt --all -- --check` and `git diff --check`.

No candidate was materialized or installed during any of these checks.
