# M7-V5 static-mesh retarget core — 2026-07-23

## Scope

This record covers offline implementation only. It does not materialize a new
M0/r33 model, HAK, MOD, `appearance.2da` row or proof resref, and it does not
claim Aurora Toolset or NWN runtime visibility.

## Implemented routes

### Static mesh plus user-provided animated donor

`m2a_core::animated_donor::retarget_static_mesh_to_animated_donor_v1` accepts:

1. one static, unskinned source GLB;
2. one separate Meshy H1-style skinned and animated donor GLB;
3. explicit binary MDL writer options.

The donor is ingested through the existing H1 clean-room route. Its
user-provided hierarchy, bind transforms, reference skin surface, weights and
mapped animation channels form the target rig. Profile A independently scales
and aligns the static geometry to that target, assigns it to target surfaces,
transfers and normalizes at most four bone influences per output vertex, and
emits SkinMesh geometry. The donor animation tracks are renamed to the output
model root and written as local Aurora animations.

The result contains the binary MDL, own-reader inspection, converted creature
IR, mapped animation set and a deterministic report binding the source hash,
donor hash, rig-profile hash, scale, active bones, clip names and output-model
hash.

The browser/WASM boundary
`retargetStaticMeshToAnimatedDonorV1` exposes the same operation without an
alternate conversion path. Model bytes are transferred once; report,
readback, animation and conversion JSON remain available for audit.

### Static mesh plus compatible Aurora supermodel

`m2a_core::reference_supermodel::retarget_static_mesh_to_reference_supermodel_v1`
implements the separate inheritance route.

The reference-side contract contains compatibility facts only: supermodel
resref, exact inspected source hash, ordered part/name/parent topology, and
read-only/no-payload-copy attestations. Exportable hierarchy transforms, skin
surface and weights must be supplied independently by an `Owned`, `Synthetic`
or `UserProvided` `CreatureRigProfileV1`. A `ReferenceOnly` rig is rejected.

The converter requires an exact topology match, transfers the static source to
weighted SkinMesh geometry, emits zero local animations and writes the selected
supermodel resref into the binary model header. Own semantic readback now
checks that supermodel value rather than assuming the legacy `NULL` sentinel.

## Files

- `crates/m2a-core/src/animated_donor.rs`
- `crates/m2a-core/src/reference_supermodel.rs`
- `crates/m2a-core/src/mdl/write_binary_mdl.rs`
- `crates/m2a-core/src/mdl/semantic_readback.rs`
- `crates/m2a-wasm/src/lib.rs`
- `crates/m2a-core/tests/animated_donor_retarget.rs`
- `crates/m2a-core/tests/reference_supermodel_retarget.rs`

## Verification

- `cargo test -p m2a-core` — exit 0; all executed unit, integration and
  documentation tests passed. Tests requiring exact external runtime witnesses
  remained explicitly ignored by their existing environment gates.
- `cargo test -p m2a-wasm --lib` — 21 passed.
- `cargo fmt --all -- --check` — exit 0.
- Library Clippy for `m2a-core` and `m2a-wasm` passed after allowing known
  pre-existing lint classes in unrelated current-worktree code. The unrestricted
  all-target `-D warnings` run remains blocked by unrelated existing lints in
  runtime evidence, examples and older tests.

The focused tests prove deterministic output, input immutability, weighted
SkinMesh emission, nonempty donor motion, local animation serialization,
supermodel-header serialization, own-reader semantic parity, strict donor
inventory rejection, topology mismatch rejection, contract-hash rejection and
reference-only provenance rejection.

## Remaining proof and product work

1. Select an owner-approved real animated donor whose proportions and motion
   suit the Meshy creature, then run an offline conversion without assigning a
   new proof iteration identity.
2. Review deformation quality. Automatic surface-based weight transfer can
   require artist-authored weight cleanup even when the structural conversion
   is valid.
3. Add the two-file donor picker and result download flow to Studio if this
   route is to be user-facing.
4. Add texture extraction and general HAK/2DA packaging only after choosing
   caller-owned output identities and satisfying the current model-iteration
   gate.
5. Preserve the exact r32 candidate until it receives the required fresh,
   candidate-bound Toolset or NWN visibility verdict. Only a qualifying
   `modelVisibility=not_visible` result permits materializing r33 or an
   equivalent new model/HAK/2DA lineage.

Therefore the converter core is implemented, but a real creature conversion
and runtime proof are intentionally not claimed.
