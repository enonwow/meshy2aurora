# TLC Guard Longsword V4 - ready for owner proof

Test module: `m2atgls4.mod`

Toolset module name: `Meshy2Aurora TLC Guard Longsword item v4`

Area: `TLC Guard Longsword Item Properties V4` (`m2atgla4`)

Status: `ready_for_owner_proof`

## Exact installed lineage

- MOD: `m2atgls4.mod`
  - canonical SHA-256: `c0d92732ea9523920c80d440eb081902a456b679891811b6cce00d4f0e58f171`
  - installed SHA-256: `c0d92732ea9523920c80d440eb081902a456b679891811b6cce00d4f0e58f171`
- ordered HAK: `m2atglh4.hak`
  - canonical SHA-256: `b73c7e324cb0975e70ace13171a42979dfe59748477d1d0414b98ea0d1cd14a1`
  - installed SHA-256: `b73c7e324cb0975e70ace13171a42979dfe59748477d1d0414b98ea0d1cd14a1`
- UTI: `m2atglu4`
  - SHA-256: `d0fb7f39b18efd4d997a42066acabd10968f7ce4a035f4bf10c9a7eb0817da89`
  - `BaseItem=1`, `Identified=1`, Model/Color selectors `23/63/23`

Canonical packet:
`C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-properties-v4-20260803\ready-for-owner-proof.json`

## Minimal corrective delta from V3

The same exact Meshy Bottom/Middle/Top sources, proportions, selector matrix and bronze colorway are retained. Every part is transformed from the Meshy `+Z` longitudinal axis into the retail WSwLs `+Y` item-model frame.

- fit algorithm: `ORDERED_SLOT_AXIAL_ENVELOPE_SEAM_GATE_V2_AURORA_Y`
- fit solution SHA-256: `3e5aa566baf9e629632c9fc58ca3af5d11459362a0943bc717ef19bcc0ce2da3`
- target lengths: Bottom `0.22`, Middle `0.08`, Top `0.90`
- source triangles: `16,356`
- emitted triangles: `16,356`

The module uses `ITEM_ONLY_GROUND_ITEM_V1`: one real identified Item, zero creatures and no equipped-render witness. Area placement rotation is not used to mask the Item Properties defect.

## Owner proof criterion

Open the exact module and the item `Last City Bronze Guard Longsword`, then inspect `Item Properties`. The assembled sword should stand upright like the retail WSwLs reference, with Bottom/Middle/Top joined in order. This handoff does not claim visual success before the owner's check.

Current axes:

- Toolset `modelVisibility=not_tested`
- Toolset `proofCompleteness=missing`
- Toolset `visualAcceptance=not_tested`
- NWN `modelVisibility=not_tested`
- NWN `proofCompleteness=missing`
- NWN `visualAcceptance=not_tested`

The agent did not start or control Aurora Toolset or NWN.
