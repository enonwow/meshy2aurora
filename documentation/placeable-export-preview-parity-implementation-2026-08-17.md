# Placeable export-preview parity implementation — 2026-08-17

## Scope and admission

This offline change implements the required Studio remediation recorded for
the exact `m2a_tlcdm4_mod.mod` candidate. The owner proved that candidate
visible in Toolset and rejected its visual match. The durable diagnosis found
that the Material Separation viewport was a Material ID overlay, not a preview
of final binary-MDL UVs and exported texture payloads.

No MOD, HAK, model, texture, resref, fixture or `rNN` candidate was generated
or changed by this work. Aurora Toolset and NWN were not started or operated.

## Implemented contract

1. Every Placeable Worker build emits
   `placeable-model-readback.json` as the dedicated
   `placeable-model-readback-json` artifact.
2. The Placeable result projector requires that artifact and requires its
   exact UTF-8 bytes to equal the readback returned for the same build.
3. `AuroraExportViewport` verifies exactly one MODEL artifact and recomputes
   its SHA-256 before rendering. For a Placeable it also verifies the readback
   artifact, parses it through the canonical readback projector and requires
   semantic equality with the displayed report.
4. Final geometry and UV coordinates come only from binary-MDL readback.
   Diffuse, normal and specular maps come only from hash-verified exported
   resources. Missing resources remain fail-closed.
5. Duplicate case-insensitive texture resrefs are rejected instead of choosing
   one payload according to array order.
6. Review exposes the bound readback SHA-256 under `Export preview lineage`.
   The authoring viewport remains explicitly labelled `Material ID Colors`;
   it is not presented as final Aurora appearance.

## Completion criteria

- exact MODEL bytes are hash-verified before export preview: PASS;
- exact Placeable readback is a downloadable, hash-verified artifact: PASS;
- mismatched readback/report lineage is blocked: PASS;
- duplicate texture resrefs are blocked: PASS;
- final exported TGA payloads and binary-readback UV0 drive the Review
  viewport: PASS;
- the diagnostic Material ID overlay is visibly distinguished from the final
  export preview: PASS;
- Studio typecheck: PASS;
- Studio tests: PASS (`49` files, `288` tests);
- real Worker/WASM static Placeable and Placeable V9 scenarios: PASS (`2`);
- canonical workspace and Meshy asset-layout guards: PASS.

## Proof boundary

This closes the offline preview/export parity defect. It does not change the
frozen candidate result:

- Toolset `modelVisibility=visible`, `proofCompleteness=verified`, visual
  acceptance rejected for `m2a_tlcdm4_mod.mod`;
- NWN `modelVisibility=not_tested`, `proofCompleteness=missing`.

A new artifact iteration still requires admission under the model-iteration
gate. Only the owner performs final Toolset/NWN visual proof.
