# `m2assmpal221.mod` — superseded palette-only module

Toolset module name: `Meshy2Aurora Sandstorm Mask Palette 221`

Exact Area name: `Sandstorm Mask Item Palette 221`

## Superseded result

The owner-provided Toolset capture showed that the Appearance combo ended at
the installed retail helmet models and did not expose `helm_221`. The model
HAK was attached, but this module omitted the exact effective `lc_2da` HAK;
therefore Toolset used a `baseitems.2da` helmet range that did not reach
variant `221`.

Do not use this module for the next check. It is superseded by
`m2assmpal221r2.mod`, documented in
`tlc-sandstorm-mask-helmet-palette-only-221-r2-ready-2026-09-04.md`.

- Evidence: `tlc-sandstorm-mask-helmet-221-toolset-selector-absent-2026-09-04.png`
- Evidence SHA-256:
  `ec45d1453237717f66d34ab213b08668fee2f414e3fa25ff6c81a6a1c147e9d0`
- Result: `selectorAvailability=absent`
- Model axes: `modelVisibility=not_tested`, `proofCompleteness=failed`

## Outcome

This module implements the owner's revised requirement: it contains no
creature blueprint, no placed creature and no UTC resource. It exposes the
helmet as an Item blueprint in the module-local custom palette:

`Items -> Custom -> Armor -> Helmets`

- Item display name: `Brązowa Maska Burzy Piaskowej`
- Item UTI ResRef: `m2assmaskpal221`
- Native palette ID: `9` (`Helmets`)
- Custom palette resource: `itempalcus.itp`, type `2030`
- Helmet model ResRef: `helm_221`
- Helmet variant: `221`

## Exact lineage

The model and HAK are unchanged from the frozen 300,000-triangle candidate.
Only the module wrapper, UTI identity/palette metadata and custom palette were
authored for the revised no-creature workflow.

- Module: `m2assmpal221.mod`
- Module SHA-256:
  `8413f88aebd61507e8206096ffcdd6d3e571a0f66d766346f95f63111d45245a`
- Installed module:
  `C:/Users/enonw/Documents/Neverwinter Nights/modules/m2assmpal221.mod`
- HAK: `m2assmh221.hak`
- HAK SHA-256:
  `d6f34a55f1d845aa86fe35cdec4df3cd7c0e9494daf26bb8b028307abfbd84ed`
- Installed HAK:
  `C:/Users/enonw/Documents/Neverwinter Nights/hak/m2assmh221.hak`
- Model SHA-256:
  `578c5fb940469ff31cf4aae9a50c02e497e7e92dca7c4d33328fb0c7e7f535be`
- UTI SHA-256:
  `e36b803fa9b98a78cf29ef229b64b44116ebbe690653c058e8f10b3d98a46207`
- Custom palette SHA-256:
  `19d363b2b0e3ebda0b533457890aaa915d41f14357d60a9d557de9a53fdbac51`

## Offline readback

The generated MOD parses as MOD V1.0 and contains exactly seven resources:
IFO, FAC, ARE, GIT, GIC, UTI and `itempalcus` ITP.

- UTC resource count: `0`
- GIT `Creature List` count: `0`
- GIC `Creature List` count: `0`
- UTI `PaletteID`: `9`
- UTI `ModelPart1`: `221`
- ITP route: `Armor -> Helmets`
- ITP matching NAME/RESREF leaves: exactly `1`
- Ordered HAK list: exactly `m2assmh221`
- Archive semantic readback: pass
- Native MOD post-copy source/destination hash equality: pass
- Existing native HAK exact-hash reuse: pass

The Item palette and palette-only package tests pass. All m2a-core examples and
the m2a-wasm crate compile.

## Owner check

Close the access-violation message and do not reopen `m2assmmod221`. Open
`m2assmpal221` instead. In the right palette select the Items icon, then
`Custom -> Armor -> Helmets`. The entry should be
`Brązowa Maska Burzy Piaskowej`.

Current proof axes remain:

- `modelVisibility=not_tested`
- `proofCompleteness=missing`

The agent did not control, close or restart Toolset or NWN.
