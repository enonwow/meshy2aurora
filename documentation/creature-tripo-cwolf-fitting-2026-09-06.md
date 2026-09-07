# Tripo borzoi: c_wolf fitting work, 2026-09-06

Status: **attempt remains blocked; no admitted game model or proof package**.

The solver experiments described below were not admitted. Their code is archived in the diagnostic directory; it is not the retained application implementation. The retained changes address source registration, reference-surface guidance, label preservation and renderer-row grouping. The original bounded queue budget is restored.

## Exact input

- Canonical asset: `sample-3d/borzoi-tripo-dc22ecb0/source.glb`.
- Original SHA-256: `dc22ecb0561fc10773e1a156b0d126225a15cdfb67cd7c0999a965a4f1d59689`.
- 25,830 renderer vertices, 20,534 triangles, 6,687,432 bytes.
- Reference: `c_wolf`, SHA-256 `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`, read in memory from the owner's retail KEY/BIF installation.
- Authored registration variant: `source-cwolf-bind.glb`, SHA-256 `a2d74adb6dd36c8577c59a331ba98d0b2f41c4c544d1782a6968997b96a876ed`.
- The variant has an explicit affine registration recipe in `fit-cwolf.json`; it is an unrigged source, not an exported NWN model.
- Independent byte comparisons verified that index, UV and embedded image payloads are unchanged. Positions and normals are intentionally transformed. Original source bytes remain intact.

## Reproduced causes

1. Height-only registration gave incorrect longitudinal placement for this source. An explicit positive-axis affine source fit avoids repeating height normalization at preparation.
2. Incoming parent-to-joint segments were unsuitable as the sole reference for assigning surface regions. New guides use distances to the inspected reference's rendered surfaces, in memory.
3. Provisional smoothing changed the dominant row before label-boundary repair, discarding the supplied labels.
4. UV synchronization reassigned even already-consistent groups using an unrelated geometric score.
5. Middle-region thickening erased terminal paw and tail regions. The new fallback preserves original semantic labels and initializes continuous weights separately.
6. Weight-based seam grouping accumulated triangle supports transitively in traversal order, rather than from immutable direct vertex supports. It also allowed unequal rows to represent one collapsed renderer vertex. Both caused misleading downstream checks.

The first branch-repair gate now passes on this exact fitted source without the experimental bypass. All 23 currently required regions survive the initializer. This is intermediate evidence, not final motion coverage.

## Remaining problem and experiments

The independent gradient stage still blocks preparation. The initial strict run exceeded 2,036,608 queue updates. Wider initialization, correct row grouping, local branch support, per-vertex influence limits and current transactional domains were investigated. A primal-dual projection was tested with the local queue but did not produce an admitted result. It and the extended queue budget were removed from the retained implementation.

The four-influence writer constraint belongs to each vertex row. A triangle's union of influences can legitimately contain more than four carriers. Graph-distance restrictions, row limits, semantic coverage and inherited-motion validation remain independent concerns.

The latest long run before hybrid checkpointing reduced the recorded violating edges from 7,962 at 250,000 updates to 438 at 18,250,000 updates. It timed out; it did **not** pass. The diagnostic wrapper originally omitted the termination signal and must not interpret its zero-looking code as success. A report file and explicit `PREPARED` result are required.

The experimental regression run passed 34 of 36 skinning tests; two failures concerned accounting for the experimental solver. After removing that solver, **all 36 skinning tests pass**, including preservation of distal regions, consistent UV labels and distinct renderer rows.

The complete retained core test run is **223 passed, 2 failed, 3 ignored**. Both failures are in the unchanged `reference_supermodel_surface_anatomy` module and concern expected policy/provenance strings (`project_to_primary_surface` versus `project_to_semantically_compatible_authoritative_surface`, and `authoritative_component_low_surface`). They were not silently rewritten to make the suite green. The new source-frame and reference-surface tests pass. The complete suite is not claimed to pass.

## Execution and evidence

- Diagnostics and snapshots: `artifacts/diagnostics/borzoi-tripo-cwolf-fit-20260906/`.
- Archived unsuccessful solver: `skinning-experiment-hybrid-not-admitted.rs` in that directory.
- Retained-code test result: `final-core-tests.json` in that directory.
- Final exact-source rerun: `retained-fixes-final/` in that directory.
- Native runner: `crates/m2a-wasm/examples/diagnose_creature_skinning.rs`, `pipeline` mode. This calls the actual Creature preparation boundary with the branch-repair bypass disabled.
- Source registration tool: `tools/fit-creature-source.mjs`.
- New core modules: `reference_surface_guides.rs`, `reference_source_frame.rs`.
- Browser workbench: `http://127.0.0.1:5186/?creatureAgent=1`.
- The workbench has the fitted source selected, but its WASM package has not yet been rebuilt for these core changes.
- Compilation and diagnostic processes run hidden with captured output. No Aurora Toolset or NWN session has been started or controlled.

## Final retained-code reproduction

The final exact-source call to `prepare_reference_supermodel_rig_v2_native` returned `M2A-REFERENCE-SUPERMODEL-SKIN-GRADIENT-NONCONVERGENT` after the restored limit of **2,004,480** updates. At the recorded 2,000,000-update checkpoint, 9,755 edges remained above the diagnostic gradient bound. The result is blocked, not prepared or motion-tested. Both canonical-workspace and asset-layout guards pass.

There is no generated MDL, HAK or MOD for this attempt. The browser WASM bundle remains the earlier version; the retained backend changes have not been presented as a deployed successful pipeline.

## Completion requirements

1. Resolve convergence while preserving required surface regions and valid four-influence rows.
2. Pass meaningful unit/regression checks and the exact-source preparation boundary.
3. Rebuild WASM, then use the Creature workbench to prepare and inspect inherited animations on the same source and reference identities.
4. Review rear legs, tail, paws, head and connected fur surfaces in motion.
5. Export through the Creature product boundary only after admission. Record exact output identities and prepare the owner proof handoff if a MOD/HAK lineage is produced.

No artifact in this work is currently a ready model, and no native visual success is claimed.
