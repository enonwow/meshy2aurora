# Meshy2Aurora Tileset Builder — corrected UI concept

> Superseded UX direction. The owner clarified that Tileset should be a target
> analogous to the existing Creature and Placeable workflows, not a separate
> application-level builder. See
> [../tileset-analog-workflow-v3/README.md](../tileset-analog-workflow-v3/README.md).

These mockups describe an authoring feature that produces a complete tileset
for Aurora Toolset and Neverwinter Nights. Meshy2Aurora owns the tileset
definition, conversion, validation and packaging stages. Area painting remains
in Aurora Toolset.

The images are product-design mockups, not screenshots of the current
implementation and not live Aurora/NWN proof.

## Workflow

1. [Tileset Setup](01-tileset-setup.png) — define `[GENERAL]`, the shared
   10 × 10 metre tile contract and output identity.
2. [Tile Catalog](02-tile-catalog.png) — bind stable Tile IDs to binary MDL,
   ASCII WOK, AABB and minimap resources.
3. [Tile Topology](03-tile-topology.png) — assign the exact `[TILEn]` corner
   terrains, corner heights, edge crossers and orientations.
4. [Terrain Types](04-terrain-types.png) — author `[TERRAIN TYPES]` and verify
   the available tile variants for each terrain.
5. [Crosser Types](05-crosser-types.png) — author `[CROSSER TYPES]` for roads,
   rivers, walls and their connection signatures.
6. [Rules](06-rules.png) — author deterministic `[PRIMARY RULES]` and
   `[SECONDARY RULES]` that resolve topology to Tile IDs and orientations.
7. [Groups](07-groups.png) — compose multi-cell `[GROUPn]` palette entries.
8. [Palette & Coverage](08-palette-and-coverage.png) — preview the palette tree
   that the SET should expose in Aurora and identify missing topology coverage.
9. [Build Package](09-build-package.png) — emit and verify SET, MDL/WOK,
   textures, HAK and an owner-test MOD.

## Entry from a Meshy result

The workflow is also reachable directly from a completed Meshy API job inside
Meshy2Aurora:

1. [Meshy result](meshy-entry/01-meshy-result-add-to-tileset.png) — select
   **Add to Tileset** beside a completed GLB result.
2. [Target and role](meshy-entry/02-target-and-role-dialog.png) — choose the
   target tileset and whether the source becomes a new tile, a variant, a group
   component or a reusable source asset.
3. [Review and add](meshy-entry/03-review-and-add-dialog.png) — review generated
   TileStaticV1 MDL, AABB, ASCII WOK and topology before an atomic SET update.

The action belongs to Meshy2Aurora's API-connected Meshy result screen; it does
not require or imply modifying the external Meshy website.

## Product boundary

The palette on screen 8 is a read-only projection. It lets the author verify
which SET record produces every expected Aurora palette entry. It deliberately
contains no Area grid or painting tools:

- Meshy2Aurora authors and validates tileset data and resources.
- Aurora Toolset consumes the generated HAK/SET and is used to paint Areas.
- NWN consumes the packaged resources at runtime.

## Visual-generation notes

- Mode: built-in image generation.
- Shared reference: screen 1 established the corrected Tileset Builder shell;
  subsequent screens preserved that shell and changed only the workflow state.
- Prompt set: nine focused `ui-mockup` prompts grounded in the audited SET
  layers and the shared TileStaticV1, MDL/AABB and ASCII WOK pipeline.
