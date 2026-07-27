# M0 r31 hierarchy-only integration candidate — 2026-07-22

## Result and boundary

Exactly one `r31` lineage was materialized after the frozen exact r30 NWN
observation had been registry-verified as `modelVisibility=not_visible` and
`proofCompleteness=failed`. No Toolset or NWN process was started, no module
was saved or rebuilt, no runtime profile was created, and no local Toolset/UI
adapter was added. The current r31 axes remain
`modelVisibility=not_tested`, `proofCompleteness=missing` for both Toolset and
NWN until a separate proof owner accepts fresh Gate B evidence.

Admission is bound to:

- r30 runtime packet SHA-256
  `a3ff0ae6708b8235beee1161b06ea7654e999578be39a174ee08c1389176af21`;
- r30 NWN capture SHA-256
  `a09fdaeea3f2e1c400e88cc332b3a6437ca535502d60ffed5be7be1f95aea9dc`;
- r30 engine log SHA-256
  `5063c46fcc90e577bb85234337c0ef03794271c6ecd561e72007599ca84ae9a8`;
- exact r30 MOD/HAK/MDL and the registry-pinned structural-pass/runtime-negative
  replay in `runtime_evidence`.

## Candidate-capable architecture

The non-admitting V3 experiment remains offline-only. R31 uses the new,
separate V4 production route:

- `MeshyHierarchyDerivedSourceBindingV4` deterministically transforms only
  source topology;
- `MeshyHierarchyCandidateProfileV4` is the caller-owned model trust root;
- `MeshyHierarchyCandidateModelContractV4` binds exact source/model replay;
- `MeshyHierarchyPackageProfileV4` and
  `MeshyHierarchyPackageContractV4` bind the MOD/HAK/full-appearance package;
- `materialize_m0_hierarchy_candidate` is the only production materializer
  for this exact lineage and hardcodes the fresh `r31` container identities.

The exact original Meshy source remains `8581684` bytes, SHA-256
`aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1`.
The derived source is `8581932` bytes, SHA-256
`88f1443aac1cf385222fc5c8940b6532b721de25e35b091492231ba88a4e2f16`.
Its complete BIN chunk is byte-identical to the original: `8579120` bytes,
SHA-256
`0d4ba1391c6d6fd9ef01d4869ef816af447269c03051b0d9464eb10f1c4f95b5`.
The selected positions/normals/UV/indices/identity-transform semantic digest is
unchanged on both sides:
`42cd14c19edecc88907942e1933ff804929b9d9cd46a63c4392cbedee09577cc`.

The declared, project-owned source topology is exactly:

1. `m2a_m0p01`, root identity dummy;
2. `m2a_hier_1`, identity dummy parented to part/node `0`;
3. `m2a_mesh_anchor`, identity transform parented to `1`, owning mesh `0`.

The output base adds only the generated Meshy mesh node `m2a_seg_1` as `0x21`
under `m2a_mesh_anchor`. All seven local type-5 state trees mirror the complete
four-node base identity as controller-free `0x01` dummy nodes. No state mesh,
skin, raw-MDX payload, controller, CEP profile, or CEP provenance is present.

Raw MDX remains byte-identical to r30: `95096` bytes, SHA-256
`731749c2e501305356a12939e519ee27a6bf6221bfbb925477d28e8f7fe58507`.
The protected non-topology writer-envelope digest is also equal on candidate
and r30:
`c4c7fe47431d27b1a6110429c738ac7590257bcef61c496626d9d74d18cd6531`.
Only topology/core pointers/layout/state topology are allowed to differ.

## Exact artifacts

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `proof-output/m0-r31-hierarchy-only-20260722/generated/m2a_m0r31.mod` | 13173 | `8575465ef683256a54c101a3f4445a4e22803194c9b899a0e43e3fd069fdeafd` |
| `proof-output/m0-r31-hierarchy-only-20260722/generated/m2a_m0r31.hak` | 19635900 | `ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12` |
| `proof-output/m0-r31-hierarchy-only-20260722/generated/m2a_m0p01.mdl` | 151328 | `fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6` |
| `proof-output/m0-r31-hierarchy-only-20260722/generated/m2a_m0t01.tga` | 12582956 | `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` |
| `proof-output/m0-r31-hierarchy-only-20260722/generated/appearance.2da` | 6901360 | `48d313b75761809e2231c99ad51e17d67632c1ce10e70b87fa8b1ccdfbc474a6` |
| production V4 contract | 40805 | `dace5e84a9b5a503010c1d1310d3f590d84836b88a04e772e528da26d160d0cf` |
| durable lineage contract |  — | `13047d6f758989f1af3063304f415085340e00d90397e9606b201e5517d7791d` |

The MOD readback is exact Area `m2a_m0a31`, entry `[10,10,0]` facing `+Y`,
ordered HAK list `[m2a_m0r31]`, and one `nw_dwarfmerc001` fixture at
`[10,14.5,0]`, appearance physical row `15100`, scale `1.0`. The full
`appearance.2da` input prefix is byte-identical, one row is appended, and the
output has `15101` physical rows.

## Absent-only native installation

Both destinations were verified absent before `FileMode.CreateNew`. Installed
hashes were then required to equal source hashes:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0r31.mod` —
  `8575465ef683256a54c101a3f4445a4e22803194c9b899a0e43e3fd069fdeafd`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r31.hak` —
  `ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12`.

No r30 or earlier artifact was overwritten, renamed, copied as a candidate, or
otherwise changed. `materializationCount=1`.

## Profiles and durable binding

- binary bootstrap:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r31-hierarchy-only-binary-bootstrap-v1.json`,
  SHA-256
  `7f62825f76ce3bd0022ba319e9015b61ca60b602d6f612afe1383d9bb7a523da`;
- no-Save Toolset proof:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r31-hierarchy-only-toolset-proof-v1.json`,
  SHA-256
  `e1b99937e310d4badf753e5655d6b8f0104b38ba1e2a079131fe82ca5abeb23e`;
- lineage contract:
  `C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\m0-r31-hierarchy-only-lineage-contract-v1.json`,
  SHA-256
  `13047d6f758989f1af3063304f415085340e00d90397e9606b201e5517d7791d`.

The runtime profile is intentionally absent and remains forbidden until an
accepted fresh Gate B packet exists for this exact MOD/HAK/model identity.

## Offline validation

PASS:

```powershell
cargo --config "env.M2A_REQUIRE_RUNTIME_WITNESSES.value='1'" test -p m2a-core --test hierarchy_candidate -- --ignored --nocapture
# 2 passed: exact derivation/package plus self-consistent attacks

cargo test -p m2a-core --test source_topology_rigid_experiment
# 7 passed

cargo test -p m2a-core --test model_pipeline
# 22 passed

cargo test -p m2a-core --test mdl_writer
# 32 passed

cargo --config "env.M2A_REQUIRE_RUNTIME_WITNESSES.value='1'" test -p m2a-core --test runtime_witness_conformance -- --ignored --nocapture
# 4 passed; forced witness mode, not an ordinary skip

cargo --config "env.M2A_REQUIRE_RUNTIME_WITNESSES.value='1'" test -p m2a-core runtime_evidence::tests::a_b_c_exact_r30_mod_hak_source_replay_and_registry_pinned_runtime_negative -- --ignored --exact --nocapture
# 1 passed

cargo test -p m2a-core --tests -- --skip swapped_field_indices_are_in_bounds_but_violate_canonical_encounter_order --skip phase_nine_field_indices_layout_precedes_phase_ten_oversized_locstring_limit
# PASS; only the two pre-existing unrelated GFF tests are excluded

cargo check --workspace
node tools/m0-r31-hierarchy-only-lineage.contract.test.mjs
# PASS, 10 negative cases
```

Central shared-tooling offline results:

- `binary_bootstrap_profile_schema_valid`;
- `binary_bootstrap_structural_readback_valid`;
- `binary_native_geometry_plan_valid`;
- every result states `startsToolset=false`, `startsNwn=false`,
  `usesGlobalInput=false`, and `noLocalToolsetAdapter=true`.

## Single proof handoff

The only next candidate action is fresh no-Save Gate B for exact r31 through
the shared central operator, using:

```powershell
$toolsetProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r31-hierarchy-only-toolset-proof-v1.json'
```

The proof owner must bind exact installed/hash-verified MOD `m2a_m0r31`, Area
`m2a_m0a31`, ordered HAK `[m2a_m0r31]`, fixture `m0_fixture`, native tree text
`Meshy M0 binary vertical-slice fixture`, occurrence `0`, exact row `15100`,
and a fresh validated `TScrollBox`. No other rNN iteration is admitted.
