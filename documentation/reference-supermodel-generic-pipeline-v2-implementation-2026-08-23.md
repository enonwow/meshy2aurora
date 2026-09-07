# Generic reference-supermodel pipeline V2 — implementation record (2026-08-23)

## Outcome

The Creature pipeline now accepts the exact supermodel selected from the local
catalog without a species, family, or resref whitelist. `c_wolf` is test data,
not a product architecture boundary. A selected binary or ASCII MDL is analyzed
together with its exact resolved parent chain, and an admitted result enters the
canonical Review and MDL/HAK export path.

The diagnostic preview remains a separate typed result. It cannot be packaged
when motion compatibility or runtime readiness is diagnostic.

## Canonical flow

1. The user selects any resolved catalog supermodel.
2. Studio reloads the full selected-to-root chain and verifies each exact
   resref, source, format, byte length, SHA-256, and parent link.
3. The worker sends the chain blob and descriptors through the generic WASM V2
   boundary.
4. Core parses binary or structural ASCII MDL, including hierarchy, nodes,
   controllers, rest/bind state, and inherited animations.
5. Core derives a rig from the actual structure and source surface. It does not
   assume wolf anatomy. Structural semantic anchors, when present, are derived
   from controller/node structure rather than selected resref.
6. The motion oracle samples deformed visible surface. Tail-like semantic
   anchors are validated against real surface amplitude and trajectory; any
   surface seam violation blocks admission.
7. An admitted product preserves exact-chain identity/provenance and is
   projected into the normal Creature Review contract.
8. Export revalidates the exact chain and packages the generated MDL/HAK. Retail
   source bytes remain read-only reference input and are not copied to product
   artifacts.

## Evidence

### Real `c_wolf` API run

The real retail binary was exercised through
`buildReferenceSupermodelAppliedPreviewV2`:

- selected SHA-256:
  `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`
- source size: 340,812 bytes
- inherited animations: 42
- bind pose compatible: true
- skin bind compatible: true
- `cpause1`, `cwalk`, and `crun` sampled
- visible semantic anchor clusters: 2
- `cpause1` and `cwalk` tail trajectory violations: 2 total
- seam violations: 47,794 under fail-on-any policy
- result: `APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY`
- `motionCompatible=false`; packaging is rejected

This proves that the API processes `c_wolf`, while correctly refusing the
known-bad surface deformation instead of reporting a false export PASS. No V9
or replacement MOD/HAK/model iteration was generated.

### Real non-wolf end-to-end run

The real retail binary `c_horror` was exercised through selection, exact-chain
analysis, preview, product construction, canonical Review projection, and
offline MDL/HAK export admission:

- selected SHA-256:
  `2faf553a0665da200b232bd52d03c0e1d79b88959cabdbe840f35f16e5878c8e`
- source size: 393,692 bytes
- parsed nodes: 27
- inherited animations: 42
- result: offline end-to-end PASS

This is the non-`c_wolf` completion oracle. The same generic entrypoints and
types were used; there is no alternate species-specific API.

### ASCII and structurally different families

A structural ASCII MDL passed the full analysis-to-export path. Core tests also
cover a branched stag-like hierarchy, a long serpent chain, a flyer hierarchy,
and disconnected rigid surface components. These fixtures differ structurally;
they are not renamed copies of a wolf fixture.

## Verification completed

- m2a-core generic/reference motion/product test suites: PASS
- generic structure fixtures: 6/6 PASS
- reference-supermodel motion suite: 15/15 PASS
- m2a-wasm native suite: 54/54 covered and PASS; environment-gated real
  `c_wolf` and real `c_horror` tests were explicitly run with their assets
- Node/WASM boundary: PASS
- Studio unit/component suites: 59 files, 328 tests PASS
- browser-worker/WASM integration: 3 files, 15 PASS, 2 intentionally skipped
- Studio TypeScript typecheck: PASS
- Studio production build: PASS
- live local UI smoke at `http://127.0.0.1:4173/`: Studio and the supermodel
  library route load correctly
- canonical workspace guard: PASS

## Remaining blockers and proof boundary

- `c_wolf` is not export-ready. Its current generated result is blocked by real
  visible-surface trajectory and seam failures. A new model iteration requires
  the repository's model-iteration gate and is outside this pipeline repair.
- Offline admission is not a Toolset/NWN visual success. The final Toolset/NWN
  proof is human-owned under `AGENTS.md`; no Toolset or NWN process was started
  and no runtime-success claim is made here.
- An arbitrary selected supermodel may still fail with concrete structural or
  motion-quality errors. That is data-driven fail-closed validation, not a
  name/family whitelist.

## Legacy isolation

The former specialized `c_wolf` implementation remains available only behind
the non-default Rust feature `legacy-c-wolf-demo` for historical comparison.
It is not exported by the Studio worker/WASM product boundary and is not used
by the generic V2 flow.
