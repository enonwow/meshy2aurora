# Tileset target — workflow analogous to Creature and Placeable

This is the corrected UX direction for tileset authoring in Studio. `Tileset`
is a third conversion target beside `Creature` and `Placeable`; it uses the
same five workflow steps:

1. [Source](01-source.png)
2. [Inspect](02-inspect.png)
3. [Build](03-build.png)
4. [Review Output](04-review-output.png)
5. [Download](05-download.png)

The lane is independent from the Creature and Placeable lanes. It is not a
shared project composer and does not introduce a separate application-level
workflow.

## Domain analogy

| Existing lane | Tileset lane |
|---|---|
| `appearance.2da` or `placeables.2da` input and update | existing `.set` input or a newly authored SET |
| model row identity | `[TILEn]`, terrain, crosser, rule and group identity |
| binary MDL and textures | `TileStaticV1` binary MDL, AABB, ASCII WOK and textures |
| package HAK | package HAK containing SET and all tile resources |
| generated proof MOD | generated test MOD with an Area bound to the custom tileset |

The Source screen supports both `Create new SET` and `Extend existing SET`.
Multiple Meshy GLB tile sources can be queued in one tileset conversion.

The Review screen includes a read-only Aurora palette projection. Area painting
still happens in Aurora Toolset.

These images are product-design mockups, not screenshots of the current
implementation and not live Aurora/NWN proof.

## Visual-generation notes

- Mode: built-in image generation.
- Prompt set: five `ui-mockup` prompts matching the existing
  `Source → Inspect → Build → Review Output → Download` Studio workflow.
- Shared visual anchor: the Source state established the target-specific lane;
  later states preserved its exact identity and shell.
