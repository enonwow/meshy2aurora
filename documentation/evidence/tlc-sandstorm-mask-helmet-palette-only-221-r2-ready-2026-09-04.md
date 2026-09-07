# `m2assmpal221r2.mod` — corrected helmet-selector module

Toolset module name: `Meshy2Aurora Sandstorm Mask Palette 221 R2`

Exact Area name: `Sandstorm Mask Item Palette 221 R2`

## Outcome

This correction keeps the owner's exact requirement: there is no creature
blueprint and no placed creature. The Item blueprint remains under:

`Items -> Custom -> Armor -> Helmets`

The earlier module attached `m2assmh221` but omitted the effective table HAK.
Aurora builds the helmet Appearance combo by scanning the `MinRange..MaxRange`
from `baseitems.2da` and retaining model resources that exist. The corrected
module attaches both required HAKs in this exact order:

1. `m2assmh221` — contains `helm_221.mdl`, its texture and inventory icon.
2. `lc_2da` — contains the exact effective `baseitems.2da` whose helmet row 17
   has `MinRange=0` and `MaxRange=255`.

Variant `221` now passes the offline selector-range gate.

## Exact lineage

- Module SHA-256:
  `389405717b3735acbdae9b29b8b51819526bc35f17795937df177dd7e563d6bd`
- Installed module:
  `C:/Users/enonw/Documents/Neverwinter Nights/modules/m2assmpal221r2.mod`
- Model HAK SHA-256:
  `d6f34a55f1d845aa86fe35cdec4df3cd7c0e9494daf26bb8b028307abfbd84ed`
- Table HAK SHA-256:
  `c4869ae516a8598f0db5e31b473abba9b15b3b3f04880eb02f4a3a37191a9e8f`
- Effective `baseitems.2da` SHA-256:
  `b2ba08aa55c185642c8973859b9d0dbf486bc8411a630a0a10a9f77e84d701c6`
- Model ResRef: `helm_221`
- Model SHA-256:
  `578c5fb940469ff31cf4aae9a50c02e497e7e92dca7c4d33328fb0c7e7f535be`
- Item UTI ResRef: `m2assmaskpal221`

## Offline readback

- MOD V1.0 resource count: `7`
- UTC resource count: `0`
- GIT placed Creature List count: `0`
- GIC Creature List count: `0`
- Ordered HAK list: exactly `[m2assmh221, lc_2da]`
- UTI `ModelPart1`: `221`
- Helmet selector range: `0..255`
- Selector range gate for `221`: pass
- Native MOD installation: absent-target copy followed by byte-identical
  SHA-256 verification
- Existing HAK dependencies: reused only after exact SHA-256 verification

## Owner check

Close the old module without saving it, restart Toolset, and open
`m2assmpal221r2`. Open the helmet Item properties and choose the Appearance
tab. The combo should contain `helm_221`; selecting it should show the new
mask model. The palette blueprint remains `Brązowa Maska Burzy Piaskowej`.

Current proof axes remain `modelVisibility=not_tested` and
`proofCompleteness=missing` until the owner reports the corrected Toolset
result. The agent did not control or close Toolset or NWN.
