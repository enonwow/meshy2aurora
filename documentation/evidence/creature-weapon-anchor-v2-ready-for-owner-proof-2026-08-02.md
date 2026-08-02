# Creature weapon attachment V2 — ready for owner proof — 2026-08-02

## Handoff

1. Exact test module: `m2aweapdemo2.mod`
2. Module name in Toolset: `Meshy2Aurora Creature Weapon Attachment V2`
3. Exact Area: `Meshy2Aurora Creature Weapon Test V2`
4. Exact HAK: `m2aweaphak2.hak`

Agent nie uruchamiał ani nie kontrolował Aurora Toolset lub NWN. Końcowy proof
wizualny należy do właściciela.

## Cel i delta względem odrzuconego V1

V1 pokazał model, ale nie wyposażył broni. V2 zachowuje oryginalne źródło,
geometrię, teksturę i 42 animacje. Zmienia wyłącznie kontrakt wyposażenia:

- `MODELTYPE=L` zamiast wyłączającego broń `S`;
- `RightHand -> rhand` i `LeftHand -> lhand` zamiast błędnych
  `rhand_g/lhand_g`;
- prawdziwy bazowy przedmiot NWN `nw_wswss001` zamiast lokalnie
  syntetyzowanego UTI;
- jedna jednoznaczna postać kontrolna z bronią w prawym slocie;
- kanoniczny forward źródła `+Z`, bez wariantu facing-matrix.

## Exact lineage

- source GLB:
  `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`
- source SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`
- triangles: `297190`
- animations: `42`
- model: `m2aweapcre2.mdl`
- model SHA-256:
  `b587ee2419f875b8081023bea4615930b2c585468698534a2177cdec454747cd`
- appearance row: `15104`
- appearance MODELTYPE: `L`
- texture SHA-256:
  `7e24d51726355bf3a4ee2f1429ea85f43a604bc2ee3e079c9fd5a306d4b1426a`
- MOD SHA-256:
  `76f0e1c75b9ec83b1bcff4ac2839af9d321054007f69bcca201f4a8fe7206e07`
- HAK SHA-256:
  `1388c43c29756858a1d648a845065a1295d9ac11188bd82f438fdabf77985c94`

Canonical artifacts:

- `proof-output/creature-weapon-anchor-v2/m2aweapdemo2.mod`
- `proof-output/creature-weapon-anchor-v2/m2aweaphak2.hak`
- `proof-output/creature-weapon-anchor-v2/handoff.json`

## Offline readback

- anchor `rhand`, parent `RightHand`, unweighted dummy node;
- anchor `lhand`, parent `LeftHand`, unweighted dummy node;
- anchor coverage: `42/42` clips;
- GIT and UTC contain equal `Equip_ItemList`;
- native hand slot struct id: `16` (right hand);
- equipped resref: `nw_wswss001`;
- resource type: `2025` (`UTI`), scope: `NWN_BASE_GAME`;
- MOD contains no embedded UTI;
- output retains exactly `297190` triangles.

Targeted gates:

- core V2 MOD/UTC/GIT stock-weapon regression: pass;
- anchor authoring/idempotency/invalid calibration: `3 passed`;
- full procedural humanoid `MODELTYPE=L` regression: pass;
- separated product/demo regression with `MODELTYPE=L`: pass;
- release materialization plus binary readback: pass.

## Native installation

Both destinations were absent before copying. Exact canonical files were then
installed and verified byte-for-byte:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aweapdemo2.mod`
  — SHA-256
  `76f0e1c75b9ec83b1bcff4ac2839af9d321054007f69bcca201f4a8fe7206e07`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aweaphak2.hak`
  — SHA-256
  `1388c43c29756858a1d648a845065a1295d9ac11188bd82f438fdabf77985c94`.

## Owner proof checklist

- otwórz `m2aweapdemo2.mod`;
- przejdź do Area `Meshy2Aurora Creature Weapon Test V2`;
- wybierz jedyną postać `RIGHT HAND - native stock sword - rhand`;
- w Inventory główny slot broni powinien zawierać stockowy krótki miecz;
- w viewport i NWN miecz powinien podążać za prawą dłonią w idle, walk/run i
  atakach;
- postać ma zachować oryginalną orientację źródła, bez testowego obrotu.

Do czasu wyniku właściciela:

- Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`.
