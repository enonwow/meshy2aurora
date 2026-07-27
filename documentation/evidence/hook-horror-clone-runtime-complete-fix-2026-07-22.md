# Hook Horror vanilla-control — runtime-complete creature fix

Date: 2026-07-22  
Status: offline fix verified; fresh Toolset/NWN proof for corrected MOD pending

## Symptom

The first generated control module `m2a_van01.mod` selected the appended
Hook Horror Appearance row correctly in Aurora, but did not establish a
working NWN runtime result. A creature created manually in Toolset and assigned
the same Appearance row rendered as Hook Horror, showing that the appended
`appearance.2da` row and stock `c_horror` resolution were not sufficient to
explain the generated instance failure.

## Confirmed cause in the first generator

The first implementation called the legacy M0 vertical-slice module builder.
The independent runtime-complete module parser rejected that generated module
at the creature envelope (`MaxHitPoints=13` was absent; the module-local UTC
also lacked the required 28-entry `SkillList`). This was a generator defect,
not evidence that the cloned 2DA row or the stock Hook Horror model failed.

The corrected implementation in
`crates/m2a-core/src/hook_horror_clone_diagnostic.rs` calls
`build_binary_creature_multi_fixture_module_v1` with one owned fixture. The
test now invokes `inspect_binary_creature_multi_fixture_module_v1`, so the
specific sparse-envelope regression fails before materialization.

## Corrected immutable artifact

- source MOD:
  `proof-output/hook-horror-label-clone-runtime-complete-20260722/generated/m2a_van02.mod`
- installed MOD:
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_van02.mod`
- MOD SHA-256:
  `fd7b047062bd7262958c67bc3d0988d7e38a7ca23098abdd50f95f3806930a91`
- source/installed HAK SHA-256:
  `d2f968decdd3d0ad4cddef6f58d6309743e4943569a876aa97172472aef1d677`
- module / Area / HAK:
  `m2a_van02` / `m2a_vana02` / `[m2a_vanh01]`
- fixture:
  `m2a_vhook`, Appearance row `15219`, position `[10,14.5,0]`
- displayed fixture name:
  `M2A Hook Horror clone control`

The HAK still contains only `appearance.2da`; it does not copy stock MDL or
texture payload. The appended physical row remains the exact 35-cell clone of
Last City row 102 with only `LABEL` changed. In particular,
`NAME=Hook_Horror`, `RACE=c_horror`, `MODELTYPE=S`, and `STRING_REF=****`
remain unchanged.

Independent readback reports one 70-field GIT creature and one 67-field
module-local UTC. Both have `MaxHitPoints=13`, a 28-entry `SkillList`, and
identical common field values. Their differences are limited to the expected
GIT placement/orientation fields versus UTC `PaletteID`/`Comment` fields.

## Verification performed

Passed:

```text
cargo test -p m2a-core --test hook_horror_clone_diagnostic -- --nocapture
node backend/scripts/validate-aurora-toolset-binary-module-bootstrap.mjs --mode dry-run --profile <corrected-binary-profile>
node backend/scripts/validate-aurora-toolset-binary-module-bootstrap.mjs --mode preflight --profile <corrected-binary-profile>
node backend/scripts/aur-s07-runtime-execution.mjs dry-run --profile <corrected-runtime-profile> --outDir <corrected-runtime-out>
git diff --check
```

The source and installed MOD/HAK pairs are byte-identical. Central structural
preflight read back exact module/Area/entry/fixture/ordered-HAK bindings.

## Remaining live gate

No runtime success is claimed yet. The corrected candidate still requires one
fresh visual Toolset/NWN run.

The public 120-second proof coordinator currently resolves MDL/TGA identities
only from the ordered HAK list. This control intentionally uses the base-game
`c_horror` resources and keeps only `appearance.2da` in its HAK. Copying stock
payload into the HAK would invalidate the diagnostic and violate provenance.
The narrow central dependency is therefore a versioned `provider=base-game`
resource resolver that reads and hash-binds installed game resources while
preserving the existing HAK-only path for project-owned candidates.

Until that resolver is admitted and the fresh images exist, report:

- corrected Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- corrected NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`.

The first `m2a_van01` evidence remains a failed historical run and must not be
rewritten as proof for `m2a_van02`.
