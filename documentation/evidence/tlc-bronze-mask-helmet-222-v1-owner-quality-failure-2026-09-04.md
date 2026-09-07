# `helm_222` V1 — owner quality failure and V2 admission

Date: 2026-09-04 (Europe/Warsaw)

## Exact failed candidate

- MOD: `m2bronpal222.mod`
- module name: `Meshy2Aurora Bronze Mask Palette 222`
- Area: `Bronze Mask Item Palette 222`
- ordered HAKs: `m2bronh222v1`, `lc_2da`
- model: `helm_222`
- icon: `ihelm_222`
- UTI: `m2bronmask222`
- MOD SHA-256: `0126dfbae0b3459c975b53daccd070d616e27ca974ffa261ef35758167611f4c`
- HAK SHA-256: `129d399e61ffcf0abac5bb17c58b82ea1e5f2edbba073cf4f88950b0f781b381`
- MDL SHA-256: `d6c722d089ae7a1e514b2755ff991c4468d4624e3b2f6ae9c49d112d0b4784d7`
- model PLT SHA-256: `3d17f7fd181f9d681293c2b586758d51e7d969a2a4d70c78ac1acbd92a83b5e1`
- icon PLT SHA-256: `131461a30a41aa2e45363471aa2b4658210af333cf0db105a779b0da23b86852`

## Owner result

- Toolset `modelVisibility`: `visible`
- Toolset `proofCompleteness`: `verified` for the two reported quality defects
- NWN model visibility for this corrective decision: not required; the owner accepted the shape and explicitly rejected the exact candidate on Toolset-observable material and placement defects.
- Defect 1: selecting `Cloth 1` leaves silver Metal 1 fragments on the cowl.
- Defect 2: the placed helmet intersects the terrain because fitted minimum Z is `-0.1233827621 m`.

Evidence:

- `tlc-bronze-mask-helmet-222-v1-owner-cloth-metal-bleed-2026-09-04.png`
  - SHA-256: `68fe5f5452ed480991d9654b9ac79c05935c82ad26a5b143d7ffe24234c3f668`
- `tlc-bronze-mask-helmet-222-v1-owner-ground-sink-2026-09-04.png`
  - SHA-256: `34e4077c87c1285c88702b8536eb16f70389ce9e7b6d1d9199f3be88ebf6db04`

## Diagnosed causes

1. V1 classified each source texel independently as bronze/warm (`Metal 1`) or not (`Cloth 1`). Bronze-coloured highlights and compression noise on the cloth therefore became isolated Metal 1 pixels.
2. V1 aligned the top of the source to the retail `helm_035` top. With the preserved uniform scale this produced fitted bounds `Z=-0.1233827621..0.2195254117 m`; Toolset places the item origin at terrain level, so the negative portion is buried.

## Minimal admitted delta

The new corrective lineage is `helm_223` / `m2bronh223v2.hak` / `m2bronpal223.mod`.

- Preserve the exact cleaned source GLB and all 36,365 triangles.
- Preserve the audited ModelType-1 helmet route, item palette, native PLT model texture and native PLT icon.
- Replace per-texel material classification with one hash-locked geometry-space face-shell mask: front shell is `Metal 1`; cowl is `Cloth 1`.
- Preserve the donor-derived uniform width scale, but set fitted minimum Z to a `0.002 m` ground clearance.
- Do not claim Toolset or NWN visual success. The owner performs the final visual proof, including the equipped vertical position because Aurora exposes only one variant transform for both contexts.
