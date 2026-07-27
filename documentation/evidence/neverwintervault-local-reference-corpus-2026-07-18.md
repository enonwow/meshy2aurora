# Local Neverwinter Vault reference corpus (2026-07-18)

## Owner authorization and boundary

On 2026-07-18 the owner explicitly authorized moving the downloaded Neverwinter
Vault archives into an ignored local corpus inside the canonical repository, so
they can be used as additional models and 2DA inputs for analysis and
env-gated tests.

Storage root (ignored by the pre-existing `/local-reference-assets/` rule):

`C:\Projects\meshy2aurora\local-reference-assets\neverwintervault-2026-07-18`

This is an owner-authorized local reference corpus exception, not source
content. Its files must remain read-only inputs: they are never copied into
`src`, fixtures, test-output baselines, product HAK/MOD, a release archive or
Git. Existing provenance and licensing rules continue to apply.

## Imported sources and archive integrity

| Source project | Local archive(s) | SHA-256 |
| --- | --- | --- |
| [Skinmesh Animals](https://neverwintervault.org/project/nwn1/model/skinmesh-animals) | `animals_0.7z` | `1bee6094cdfa58bc11b563a5e54fe7373d34f9eb00c44d95b8688eeec9ed2a17` |
| [NWN2 Creature conversion for NWN1](https://neverwintervault.org/project/nwn1/model/nwn2-creature-conversion-nwn1) | `nwn2_models01.zip` | `c1b8f86cd68aadf446af7c59a0b03be5d06355a89eb78a5fc0e6cf40b13acfd` |
| same | `cat_original_texture.rar` | `bf2af8311d8a61e7843478fd270cbeb935d4c5f0123fba7f1802c515f5195b58` |
| same | `nwn2_models02.rar` | `cd96cb65724d4f96b8abbda046a17b3947819e45d0c6a682e04dbe8f4f6229cf` |
| [The Witcher 1 Geralt models 4--6](https://neverwintervault.org/project/nwn1/model/witcher-1-geralt-models-4-6-ravens-armor-nwn1-ee) | `geralt_mdl4-6_release1.zip` | `489cd0c345789d8b99238796876f12373131f50693f8e03106d33510f130ab98` |
| [The Witcher 1 Child (Alvin)](https://neverwintervault.org/project/nwn1/model/witcher-1-child-alvin-nwn1-ee) | `tw1_cr_kid_m01.zip` | `0465ff0244321b70dc998f9f322b567b2f2af1b71d4a5318d21207d0d521d42` |
| same, optional original textures | `alvin_originalnonenwn1-dds-texture.zip` | `d593832d6863a48287047561477f5d62120ffe9448d04aed758f25968604a69d` |
| [The Witcher 2 Harpies](https://neverwintervault.org/project/nwnee/model/witcher-2-harpies-nwn1-ee) | `tw2_harpies_ee.zip` | `fcf0112577bf8f868eb195a52ae001fdeae88b68a7e15ab1d607dbc96361bf8b` |

Every digest was calculated before moving the source archive out of Downloads
and re-calculated after the move; each pair matched.

## Layout and observed inventory

- Original archives: `<root>\archives\`.
- Separate unpacked working copies: `<root>\extracted\<project-slug>\`.
- Unpacked files: 50 `.mdl`, 1 `.2da`, 54 `.dds`, 12 `.tga`, 2 `.txi`, 3
  `.hak`, 2 `.mod`, 11 `.gmax`, 3 `.txt` and 1 `.png`.
- The corpus was not installed in NWN and no included module/HAK was executed.

## Intended analysis order

1. Start with `extracted\skinmesh-animals`, especially `c_squirrel.mdl` and
   `c_turtle.mdl`, as simple comparable creature controls.
2. Then inspect `extracted\nwn2-creature-conversion\models02`, where the
   published package specifically documents corrections to skin/bone and dummy
   node issues.
3. Use Geralt and Alvin for the supermodel, skin/shadow and Blender-rigging
   boundary cases; use the Harpies HAK/MOD only as an EE/LOD/bone-limit case.

## Verification result

The local corpus is present, unpacked, Git-ignored and hash-verified. It is
available for subsequent reader/writer investigations without re-downloading
or modifying the original upstream payloads.
