# Creature audit remediation — 2026-08-17

## Scope and boundary

This change closes the code-level findings from the Creature pipeline audit. It does not create a
new model iteration, freeze a MOD/HAK candidate, install proof artifacts, or operate Aurora Toolset
or NWN. Under the human-owned proof rule, a visual success can only be recorded after the owner
tests an exact installed candidate.

## Audit findings

1. The Studio exposed material-separation authoring for all Creature profiles, while the P100K and
   P300K experiment workers omitted the material payload. A user could therefore author materials
   that were silently ignored.
2. Studio inspection and the authoring editor emitted Face Mode V2 documents, but the Creature
   build boundary accepted only the component-only V1 document.
3. Material inspection, resolution, texture preparation and build were asynchronous without one
   monotonic authoring identity. A response produced for an older recipe could race a newer edit.
4. The workspace Clippy gate was red, obscuring regressions in the changed paths.

## Implemented decision

The supported capability matrix is explicit:

| Creature profile | Component Mode V1 | Face Mode V2 |
| --- | ---: | ---: |
| `PRODUCT_300K` | supported | supported |
| `EXPERIMENTAL_P100K` | rejected | rejected |
| `EXPERIMENTAL_P300K` | rejected | rejected |

Studio hides the editor and explains the product-profile requirement on experiment profiles. The
Worker also rejects any experiment request that contains a material payload with
`CREATURE-MATERIALS-PROFILE-UNSUPPORTED`, before attempting to read the model bytes. Core repeats
the fail-closed policy so a caller cannot bypass it through another frontend.

For `PRODUCT_300K`, the Worker/WASM parser now accepts strict schema V1 or V2 input. Core resolves a
V2 face document once, projects that exact resolution into Profile A render materials and texture
authoring, and uses the same report for the resulting package. Existing V1 component callers remain
compatible. Tile retains its independent component-only compatibility adapter.

Studio session state now has a monotonic `authoringRevision`, while a build epoch invalidates
asynchronous preflight/build work. Source, Appearance, material recipe, texture recipe, profile
switch and cancellation changes invalidate the in-flight build. Late success or failure responses
cannot reopen Review or publish an artifact for stale authoring.

The audit also removed seven Clippy failures. Narrow `too_many_arguments` allowances are retained
only on existing Placeable API boundaries where replacing the public signature was outside this
Creature remediation.

## Automated acceptance evidence

The following gates passed after implementation:

- `cargo fmt --all -- --check`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- `cargo test -p m2a-core --all-features`.
- `cargo test -p m2a-wasm --all-features`: 45 passed.
- Studio unit suite, including capability, Face V2 identity and stale-authoring regressions.
- Default browser Worker/WASM suite: 14 passed, 2 heavy replays skipped by their normal opt-in
  contract.
- Production Studio build (`wasm-pack`, TypeScript and Vite).
- Release canonical Creature geometry corpus: 6 passed, including P100K and both P300K routes,
  without triangle loss.
- Real browser Worker/WASM Face Mode V2 package build, including two material slots and textures.
- Real browser experiment-material request rejection before model-byte parsing.

The opt-in heavy browser replay correctly loaded the canonical P100K and P300K GLBs after its
environment flags were applied during fixture preparation. Its functional builds completed, but
two immutable expectations failed:

- P100K expected MOD SHA-256
  `91af75ead718d742045767c31617d560dad1bc3f05ae00644417149dea21215c`, received
  `032c1d5e31b52939bd940e7de1a60f147247105bb85a0f230f0581142d151d90`.
- P300K texture cleanup retained the expected algorithm, two passes, 4,177,936 inspected pixels and
  4,588 repaired color outliers, but the input/output pixel hashes differed from the frozen values.

Those golden values were deliberately not updated. Accepting the new bytes would create or bless a
new proof lineage without the candidate-bound visual failure and owner proof required by the model
iteration gate. The independent release geometry corpus passed 6/6, so this is recorded as an
immutable-artifact lineage blocker rather than a triangle-budget or face-loss failure.

## Definition of done

The code-level remediation is complete when all of the following remain true:

- [x] Material support is explicit and consistent in Studio, Worker and Core.
- [x] Experiment profiles cannot silently ignore a material payload.
- [x] Product Creature accepts component V1 and Face V2 material documents end to end.
- [x] V2 material resolution drives both MDL material slots and texture authoring from one resolved
  projection.
- [x] A changed material/texture recipe invalidates in-flight preflight and build results.
- [x] Shared `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000` behavior and lossless stream partitioning
  are preserved by the canonical corpus.
- [x] Formatting, Clippy, Core, WASM, Studio unit, default Worker/WASM and production-build gates
  pass.
- [x] No Toolset/NWN session was started and no proof candidate was generated or mutated.
- [ ] An exact future candidate's immutable hashes match its approved manifest and frozen golden
  contract.
- [ ] The owner reports the exact candidate's Toolset and NWN visual verdict.

The last two items are proof-stage closure criteria, not authorization to change a candidate or its
goldens. Until the owner admits a new iteration and completes visual proof, the agent-side status is
code remediation complete, proof lineage not closed.
