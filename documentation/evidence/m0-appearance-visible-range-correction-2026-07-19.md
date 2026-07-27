# M0 Appearance visible-range correction — 2026-07-19

Status: `OFFLINE VERIFIED / FRESH TOOLSET BOOTSTRAP READY / NWN RUNTIME MISSING`.

## Symptom and live diagnosis

The active Toolset session opened `Creature Properties` for the placed Dwarf
Mercenary by a targeted native double-click in the validated Area viewport.
`Appearance` was available and initially rendered `Dwarf, Male`; therefore
the failure was not an inability to open or edit creature properties.

The owner-drawn Appearance combo exposed exactly 848 entries, indexed `0..847`.
Its final entry was `Zombie, Warrior 2`.  The generated
`M2A_M0_MESHY_RIGID` row was absent.

The previous generated `appearance.2da` had 15,105 physical data rows and
appended M0 after that inaccessible tail.  Toolset did not expose those rows.
This is a live Toolset fact; the inference is that an M0 runtime HAK must put
its own row inside the visible prefix.

## Corrective implementation

`retain_two_da_row_prefix_v1` now retains a byte-identical, syntactically
validated 2DA prefix.  The M0 packaging lane retains at most 848 appearance
rows before appending `M2A_M0_MESHY_RIGID`.  It does not modify the input
appearance file.

The new regression test constructs a valid 15,105-row table with a tail of
`OS_RESERVED` rows and requires all of the following:

- output `appearance.2da` has 849 rows;
- M0 has `Appearance_Type = 848` in the generated proof module;
- the inaccessible tail is absent from the runtime HAK;
- the result remains deterministically readable through the project parser.

The Studio worker now reports the generated isolated M0 MOD as
`m2a_bm0p1.mod`, matching the core package writer.

## Materialized packet

Packet: `proof-output/m0-meshy-golem-20260719-v6`.

| Artifact | SHA-256 | Fact |
| --- | --- | --- |
| source GLB | `aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1` | same owner-provided Meshy source |
| `appearance.2da` | `04f7933cd6cb67ae4d065a18c6bed9e18b8688bb97bd4cae5f9072e1d2792acb` | 849 rows, M0 at 848 |
| `m2a_m0_proof.hak` | `673f51a82792c7e28eb26f0595ffe96b7018cb29eef8cbc3807790a63dead82d` | new runtime HAK |
| `m2a_bm0p1.mod` | `ffe97b8f0b6a121af9e3d29be3240bf9af3ec10febafeaab6413fae7efcef958` | proof MOD bound to row 848 |

`m2a_m0_v6.hak` was copied as a new file into the user NWN HAK directory and
its installed hash was read back as the value above.  No INI, MRU, Toolset
preference, game installation file, or existing HAK was changed.

## Live attachment result

The central owner-drawn Module Properties atom read the following exact state
in `m2a_m0nwn1.mod`:

- available HAK source: `m2a_m0_v6` at combo index 82;
- attached HAK list: only `m2a_m0_proof`;
- Add button: disabled after the atom's `CB_SETCURSEL`.

This atom stopped before `Add`, dialog commit, rebuild, or save.  The central
selection-notification atom cannot recover this state because its published
contract rejects a non-empty attached HAK list.  The dialog was cancelled by
its exact native `Cancel` button; a readback then showed the same single,
responsive Toolset frame and no modal.

Thus the live HAK-list delta, M0 Appearance selection, saved module readback,
and NWN runtime proof have **not** occurred.  They must not be inferred from
the successful offline packet or the installed side-by-side HAK.

## Fresh bootstrap preparation

The canonical vertical-slice bootstrap declaration is
`proof-output/m0-v6-central-bootstrap/bootstrap-declaration.json`.  Its
offline dry-run passed without starting Toolset or NWN and proves that the
destination module is absent, the only attached HAK will be `m2a_m0_v6`, and
the placed Dwarf Mercenary will be changed to exact Appearance label
`M2A_M0_MESHY_RIGID` at row `848`.

During this preparation, the shared central Appearance selector was found to
declare a PowerShell function named `R`.  `R` resolves to the built-in
`Invoke-History` alias at invocation time, so the original selector failed
before enumerating the combo boxes.  The selector now uses `Get-R`; its
contract test and compile-only check pass.  This was a central operator-tool
repair only: it does not change INI, MRU, module data, HAK content, or the
current Toolset session.

## Verification executed

```text
cargo test --locked -p m2a-core --test model_pipeline              PASS (12)
cargo test --locked -p m2a-core --test binary_m0_vertical_slice_module  PASS (1)
npm --prefix apps/studio-web run typecheck                         PASS
npm --prefix apps/studio-web test -- --run                         PASS (172)
git diff --check                                                   PASS
node C:\Projects\aurora-web\backend\scripts\test-select-aurora-toolset-dialog-combo-item-timeout-validated-contract.mjs  PASS
node C:\Projects\aurora-web\backend\scripts\aurora-toolset-vertical-slice.mjs bootstrap-dry-run --declaration proof-output\m0-v6-central-bootstrap\bootstrap-declaration.json  PASS
```

The full `cargo test --locked -p m2a-core` audit is `FAILED` in two existing
GFF tests: `swapped_field_indices_are_in_bounds_but_violate_canonical_encounter_order`
and `phase_nine_field_indices_layout_precedes_phase_ten_oversized_locstring_limit`.
Both `crates/m2a-core/src/gff.rs` and `crates/m2a-core/tests/gff.rs` are clean
relative to `HEAD`; this is not hidden by the targeted M0 passes and was not
changed as part of the 2DA correction.

`cargo test --locked -p m2a-wasm` is `FAILED` independently of this M0
range gate: 19/20 pass and `ready_owned_boundary_is_exact_native_batch_oracle_and_immutable`
has a stale M7 golden SHA-256 (`expected edafe...`, actual `ba485...`).  The
golden was not rewritten without a separate semantic audit.

## Resume condition

The owner authorized continuation and the central close route posted the
targeted close to PID `5856`.  Aurora then showed exactly one visible native
`#32770 / Confirmation` on `\\.\DISPLAY1` with text `Would you like to save
your changes to the module?` and enabled buttons `&Tak`, `&Nie`, and `Anuluj`.
The dirty target is the old working module `m2a_m0nwn1.mod*`.

This confirmation is a meaningful save-or-discard choice.  It is not covered
by the earlier close authorization, so no button has been posted and the
Toolset process has been preserved.  The owner must now expressly choose save,
discard, or cancel.  After a confirmed close, run the already validated
bootstrap declaration and require fresh Toolset readback showing
`M2A_M0_MESHY_RIGID` at index 848, a rendered model, saved module readback,
and the central runtime gate before claiming NWN compatibility.
