# Meshy2Aurora Tileset Studio — UI mockups

> Superseded concept. The corrected Tileset Builder workflow is documented in
> [../tileset-builder-v2/README.md](../tileset-builder-v2/README.md). It treats
> Aurora Toolset as the Area-authoring consumer rather than reproducing its Area
> editor inside Meshy2Aurora.

This set presents the proposed tileset-authoring workflow as nine distinct
application states. The images are product-design mockups, not screenshots of
the current implementation and not Aurora Toolset proof.

## Workflow

1. [Source Models](01-source-models.png) — import and classify source meshes.
2. [Tile Library](02-tile-library.png) — define tile identity, geometry and
   validation metadata.
3. [Terrains](03-terrains.png) — create the terrain entries exposed by the SET.
4. [Crossers](04-crossers.png) — author roads, rivers and other edge-spanning
   palette elements.
5. [Features](05-features.png) — define paintable single-tile and multi-tile
   decorative features.
6. [Rules](06-rules.png) — map terrain neighbourhoods to deterministic tile and
   rotation choices.
7. [Groups](07-groups.png) — compose reusable multi-cell palette stamps.
8. [Area Preview](08-area-preview.png) — exercise the generated Aurora-style
   Terrain, Features and Groups palette against a test Area.
9. [Export](09-export.png) — validate and package SET, MDL, WOK, textures, HAK
   and an owner-test MOD.

## Panel behaviour

The left workflow rail and top project context remain persistent. Selecting a
workflow step swaps the central editor, its context inspector and the bottom
validation/readiness strip. Selection within a screen updates the inspector
without changing the workflow step.

The Area Preview palette is the direct counterpart of the highlighted Aurora
Toolset panel: its tree is generated from Terrain, Features and Groups authored
in the preceding screens. Rules resolve the concrete tile and rotation after a
palette brush is applied.

## Visual-generation notes

- Mode: built-in image generation.
- Shared reference: the Tile Library screen was used as the visual anchor for
  all other states.
- Prompt set: one focused prompt per workflow state, preserving the same
  application shell while specifying each panel's data, controls, validation
  states and handoff purpose.
