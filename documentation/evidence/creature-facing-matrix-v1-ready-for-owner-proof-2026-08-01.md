# Creature source-forward matrix v1 — ready for owner proof (2026-08-01)

1. Exact test module file: `m2afacedemo1.mod`
2. Module name shown in Toolset: `Meshy2Aurora creature runtime-profile comparison`
3. Exact Area name: `Meshy2Aurora creature comparison area`

Status: `ready_for_owner_proof`.

The final visual verdict remains human-owned. This packet does not start,
adopt, control or capture Aurora Toolset or Neverwinter Nights. Until the owner
reports the result, both lanes remain:

- Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`
- NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`

## Purpose

This is a four-way source-forward selection matrix for the same canonical
Meshy creature. It proves the new caller-selected Creature facing path with an
owner-visible comparison instead of relying on an assumed source convention.

All four fixtures use the same source GLB, texture, geometry, animation set,
runtime profile and placement orientation. Only the pipeline's
`sourceForward` input and its resulting proper-rotation basis differ. Every
reported transform has determinant `+1`; no mirror transform is admitted.

From left to right in the Area:

| Creature display name | Source forward | Appearance row | Model resref | Position |
|---|---:|---:|---|---:|
| `Source +Z to Aurora -Y` | `POSITIVE_Z` | 15100 | `m2afacepz1` | x=4.50, y=14.50 |
| `Source -Z to Aurora -Y` | `NEGATIVE_Z` | 15101 | `m2afacenz1` | x=8.25, y=14.50 |
| `Source +X to Aurora -Y` | `POSITIVE_X` | 15102 | `m2afacepx1` | x=11.75, y=14.50 |
| `Source -X to Aurora -Y` | `NEGATIVE_X` | 15103 | `m2afacenx1` | x=15.50, y=14.50 |

Every fixture orientation is exactly `(0, -1)`. Therefore a visual difference
between the four models comes from the generated model basis, not from a
different GIT placement orientation.

## Immutable lineage

- Canonical source:
  `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`
- Source SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`
- Triangle count in every generated model: `297190`
- HAK: `m2afacehak1.hak`
- HAK SHA-256:
  `27399b1853c197df169cd1e945cabe4616c0e337e755285a8917e55c622d557a`
- MOD SHA-256:
  `728f4ce5b397b5b993b72488ef0888ca28a6f915a5eff5dc0993803715c3c43c`
- HAK entry count: `6` (four MDLs, one shared TGA, one complete
  `appearance.2da`)
- Canonical packet:
  `artifacts/creature-facing-matrix-v1/handoff.json`

The exact artifacts were installed for the owner test with no overwrite:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2afacedemo1.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2afacehak1.hak`

Post-copy SHA-256 verification matched the canonical MOD and HAK hashes above.

The aggregate HAK and MOD were parsed after generation. HAK resource readback
was byte-identical for all six entries. Module readback binds the ordered HAK,
Area, four UTC fixtures, rows 15100–15103, coordinates and orientations shown
above.

## Owner check

Open `m2afacedemo1.mod`, load `Meshy2Aurora creature comparison area`, and
compare the four named creatures in both Toolset and NWN. Report which label
faces the player correctly. That result selects the source-forward setting for
this asset family; it does not require another generated model just to inspect
the four supported cardinal conventions.
