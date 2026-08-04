# `m2atgls2.mod`

Module name shown in Toolset: `Meshy2Aurora TLC Guard Longsword equipped v2`

Exact Area name: `TLC Guard Longsword Equipped Model Proof`

## Owner proof handoff

The exact V2 candidate is `tlc-guard-longsword-equipped-v2-20260803`.
It changes only the test-module fixture: the byte-identical UTI `m2atglu1`
is no longer placed as a ground item. One human proof creature, `m2atgln2`,
carries it in native `Equip_ItemList` struct id `16` (right hand). This forces
Aurora/NWN to exercise the equipped `ModelType=2` renderer and assemble the
three actual MDLs selected by `ModelPart1`, `ModelPart2` and `ModelPart3`.

- MOD: `m2atgls2.mod`
  - SHA-256: `249a63d794f0d719f804e3fe05419e49ee5ac079e048eeb512e4c3c094c20cb4`
  - canonical: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-equipped-v2-20260803\m2atgls2.mod`
  - installed: `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2atgls2.mod`
- HAK: `m2atglh1.hak`
  - SHA-256: `9dba2282a123177df29acabc6c543ececf135d63aed38871e01d8e7dab9fae22`
  - canonical: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-equipped-v2-20260803\m2atglh1.hak`
  - installed: `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2atglh1.hak`
- UTI: `m2atglu1`, SHA-256
  `b018aca195a5bf7cb4f8d1993bf1957f089ae590c332b3d9aae112971c4d5087`
- BaseItem: retail row `1`, `longsword`, `ItemClass=WSwLs`, `ModelType=2`
- proof creature context: Appearance row `6`, `RACIALTYPE=11`, gender `0`,
  phenotype `0`, derived model prefix `pmh0`
- semantic readback: MOD `PASS`, HAK `PASS`, UTI `PASS`
- UTI contract: `partCount=3`, `colorFieldCount=0`, `identified=true`

The MOD and HAK installed in the native NWN directories were independently
hashed after installation and are byte-identical to the canonical files.
The V2 HAK, UTI, three MDLs, three textures, three icons and fit transforms are
byte-identical to V1; only the MOD fixture changed.

## Expected owner check

Open the exact module above and load the exact Area. The creature is placed in
front of the module entry and should visibly hold the assembled longsword in
its right hand. This stage requires the owner's Toolset/NWN visual verdict;
the agent did not start or control either application.

The previously reported hilt-to-blade proportion issue is deliberately not
changed in V2. First this candidate proves that the pipeline renders actual
part models instead of the ground-item color/icon presentation. Proportion
changes require the resulting candidate-bound owner verdict.

Machine-readable packet:
`C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-equipped-v2-20260803\ready-for-owner-proof.json`
