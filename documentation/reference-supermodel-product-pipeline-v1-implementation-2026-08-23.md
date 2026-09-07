# Reference-supermodel product pipeline v1 — implementation report

Date: 2026-08-23

## Outcome

The reference-supermodel result is now part of the canonical Creature conversion
contract. An applied reference supermodel no longer ends as a React-only preview.
When the quality and provenance gates pass, the worker returns the normal Review
snapshot and downloadable MDL, TGA, MTR, `appearance.2da`, HAK, report, manifest,
and summary artifacts.

Diagnostic preview remains a separate typed result. It cannot enter the product
export lane when `motionCompatible=false`, runtime readiness is diagnostic, the
exact chain is unverified, or the repaired surface has no semantic delta from the
rejected baseline.

## Common flow

1. The UI resolves the selected family through a reference-supermodel rig profile.
2. The local retail MDL is read only, SHA-256 bound, and validated against the
   exact resolved supermodel chain. Retail payloads are never copied to artifacts.
3. The generic preview API produces diagnostic readback and quality evidence.
4. Creature Build re-reads and re-hashes the selected exact reference MDL instead
   of trusting the earlier React state.
5. The product API runs bind, carrier, skin, motion, surface-continuity, trajectory,
   semantic-delta, chain, contract, and provenance admission gates.
6. Only an admitted result is packaged into the normal Creature Review/download
   route and final MDL/TGA/MTR/2DA/HAK product.

Family-specific rig knowledge is isolated in adapters. The shared product module
does not hard-code `c_wolf`. An unsupported family fails explicitly with
`BLOCKED_PROFILE_MISSING`.

## c_wolf status

`c_wolf` is registered as profile `nwn-c-wolf-reference-rig-v1`. Its exact retail
resource and chain are SHA-256 bound and its tail profile maps the visible
`tail_base` and `tail_tip` surface clusters to `Wolf_tail` and `Wolf_tailend`.

The exact local API run resolves 42 inherited animations and passes bind/carrier/
skin compatibility. The new oracle samples actual visible-surface motion for
`cpause1`, `cwalk`, and `crun`; all six base/tip trajectory checks execute. The
current source remains blocked because strict visible-surface continuity reports
961,926 seam violations. The old allowance would have admitted 519,265 such
violations, while the V8 owner failure recorded 820,131. The new profile therefore
uses a fail-on-any-visible-seam gate.

The rejected V8 semantic signature is pinned. Renaming a resource alone cannot
satisfy the repair requirement: both the model bytes and the visible tail surface
signature must change.

Result: the `c_wolf` integration path is implemented, but the current candidate is
not exportable. This is an intentional quality block, not an NWN success claim.
No Toolset or NWN session was started and no new MOD/HAK/model iteration was made.

## Other families

The shared profile, admission, appearance-row, and packaging contract is covered
by synthetic `c_stag` and `c_serpent` families. A real family that has no registered
rig adapter is reported as `BLOCKED_PROFILE_MISSING`; it is neither silently
treated as `c_wolf` nor routed through a fake successful export.

## Verification

- Core product/profile tests: 5 passed.
- Core motion oracle tests: 15 passed.
- Core c_wolf rig tests: 3 passed.
- Core retarget tests: 7 passed.
- Core catalog tests: 7 passed.
- WASM boundary tests: 6 passed in the non-environment suite.
- Exact environment-gated rejected-V8 surface-signature oracle: executed and
  passed with signature
  `ff75e44d8c903e68fdded68061c594013374cd4255b9b31b356a9c77e64ab15a`.
- Exact environment-gated c_wolf preview/product gate: executed in release mode;
  preview returned `APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY` and the product API
  correctly rejected packaging.
- Exact Node/WASM API proof: executed and passed; 42 animations, exact SHA binding,
  `motionCompatible=false`, `motionQualityStatus=BLOCKED`, and product rejection.
- Studio test suite: 59 files and 328 tests passed.
- Studio typecheck: passed.
- Production WASM/web build: passed (existing bundle-size warning only).
- Local production-like UI on `http://127.0.0.1:4173/`: loaded without console
  errors.

## Remaining blockers

1. The c_wolf rig/deformation implementation must remove all visible seam/split
   defects and produce a real semantic change in the pinned repair area.
2. After that implementation passes the offline export gate, the exact new frozen
   candidate must be handed to the owner for the human-owned Toolset/NWN proof.
3. Every additional retail family needs its own verified rig/profile adapter before
   it becomes exportable.

The owner-reported V8 runtime quality failure remains authoritative. This work
repairs the pipeline and its admission oracle; it does not overwrite that verdict.
