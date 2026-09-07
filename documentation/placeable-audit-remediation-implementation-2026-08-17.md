# Placeable audit remediation implementation — 2026-08-17

## Scope

This change closes the repository findings from the 2026-08-17 Placeable
audit. It changes product code and offline tests only. It does not materialize
a new proof candidate, install MOD/HAK files, or operate Aurora Toolset/NWN.

## Implemented contract

1. Every Placeable product route uses the shared
   `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000` gate. The source-bound
   oversized Placeable exception, its Profile A bridge, and its artifact
   materialization example were removed. Historical immutable proof packets
   and evidence remain unchanged.
2. Normal Studio Placeable builds always send the Face Mode V2 separation
   document and selected Aurora material profile to the V9 Worker route. An
   invocation of V1–V8 now requires the explicit
   `compatibilityPipeline: "PLACEABLE_V1_V8"` marker.
3. Placeable artifact identity schema version 2 binds the selected material
   profile, the complete separation document, optional material texture
   authoring, optional UV projection, geometry authoring, legacy texture
   authoring, source hashes and cleanup policy. Generated resrefs use a
   14-character base32 SHA-256 prefix (70 bits) within Aurora's 16-character
   resref limit.
4. Material source-quality inspection selects a deterministic worst-case
   density representative for overrides sharing one source material and emits
   contrast diagnostics independently for every output material slot. A flat
   override blocks the package regardless of override ordering.
5. Material profile, UV projection and material recipe changes invalidate any
   running or completed Placeable result. Review and Download are relocked at
   the Build boundary.
6. `PLACEABLE_AUTHORING_GROUND_Y_V1 = 0` is the single Placeable authoring,
   grid, pointer-picking, collision-preview and PWK plane. Below-ground and
   floating measurements use the same policy and remain visible as authoring
   diagnostics. This closes `STUDIO-VIEWPORT-GROUND-001` without moving source
   geometry or changing the Aurora origin.

## Offline verification

- canonical workspace guard: PASS;
- focused Core library, Placeable and segmentation tests: PASS
  (`115 + 24 + 5`, with three environment-gated tests ignored);
- Studio typecheck: PASS;
- Studio unit/component tests: PASS (`49` files, `285` tests);
- complete `m2a-wasm` test crate: PASS (`44` tests);
- complete Rust workspace excluding the separately verified `m2a-wasm` crate:
  PASS;
- real Worker/WASM Placeable V9 and explicit V1–V8 compatibility scenarios:
  PASS (`2` tests);
- `git diff --check` for the changed Placeable files: PASS.

## Completion boundary

This is an offline implementation result, not visual proof. A future
`ready_for_owner_proof` handoff still requires one exact frozen candidate,
canonical hashes, absent-or-identical native MOD/HAK installation, and the
owner's Toolset/NWN test. Only the owner's report can change Placeable
`modelVisibility` from `not_tested`.
