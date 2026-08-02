# Creature weapon anchor V1 — ready for owner proof — 2026-08-01

> **Status 2026-08-02: OWNER PROOF FAILED — superseded by V2.** Model Creature
> był widoczny, ale broń nie została wyposażona ani wyrenderowana w dłoni;
> główny slot w Toolsecie pozostał pusty. Nie zmienia to wyniku widoczności
> modelu: `modelVisibility=visible`. Funkcjonalny proof wyposażenia ma
> `proofCompleteness=failed`. Ten immutable V1 pozostaje dowodem negatywnym i
> nie wolno go przedstawiać jako działającego demonstratora broni.
>
> Evidence: `codex-clipboard-d517cded-4d00-467f-8ca7-ab8e05c74f1d.png`,
> SHA-256 `0edc6563f30fcfe672170cbfcb4e71d1fd877b1d5af7b518a4b0d463f0d782a0`.
> Potwierdzone przyczyny: `MODELTYPE=S`, błędne hooki `rhand_g/lhand_g` oraz
> syntetyczny UTI, którego Toolset nie rozwiązał do wyposażenia. Poprawka V2
> używa `MODELTYPE=L`, hooków `rhand/lhand` i stockowego `nw_wswss001`.

## Handoff

1. Exact test module: `m2aweapdemo1.mod`
2. Module name in Toolset: `Meshy2Aurora Creature Weapon Anchors V1`
3. Exact Area: `Meshy2Aurora Creature Weapon Test`
4. Exact HAK: `m2aweaphak1.hak`

Agent nie uruchamiał ani nie sterował Aurora Toolset/NWN. Zgodnie z
`AGENTS.md` końcowy proof wizualny należy do właściciela.

## Exact lineage

- source GLB:
  `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`
- source SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`
- source triangles: `297190`
- output model: `m2aweapcre1.mdl`
- model SHA-256:
  `6c54cfc85edbcb71d03b8ec362b71ce0f9d89c92a383b4018b782803f09879a4`
- appearance row: `15104`
- MOD SHA-256:
  `8a2ab67b0f21230062803b2cac40f2a5848d8b837624d3659db2ebed7ab5d9c1`
- HAK SHA-256:
  `2c42980ad500c005c7c32722ba5d8ef5ed0ae70731bf55cde043bc43c276b903`

Canonical artifacts:

- `proof-output/creature-weapon-anchor-v1/m2aweapdemo1.mod`
- `proof-output/creature-weapon-anchor-v1/m2aweaphak1.hak`
- `proof-output/creature-weapon-anchor-v1/handoff.json`

## Offline semantic proof

- `rhand_g.parent = RightHand`;
- `lhand_g.parent = LeftHand`;
- oba anchory są dummy nodes i nie należą do skin weights;
- animation anchor coverage: `42/42`;
- right-hand control: UTC `m2awrhand1`, native slot struct `16`;
- left-hand control: UTC `m2awlhand1`, native slot struct `32`;
- oba kontrolki wskazują owned UTI `m2aweapitem1`;
- typed UTI readback: `UTI `, `BaseItem=1`, `ModelPart1/2/3=11`;
- GIT oraz odpowiadający UTC mają identyczny `Equip_ItemList`;
- MOD zawiera dokładnie jeden owned UTI i dwa owned UTC;
- model zachował `297190` trójkątów i `42` klipy.

## Offline gates

- `cargo test -p m2a-core --lib --no-fail-fast`:
  `93 passed, 0 failed, 2 ignored`;
- `cargo test -p m2a-core --test gff`:
  `19 passed, 0 failed`;
- `cargo test -p m2a-core --test model_pipeline --no-fail-fast`:
  `29 passed, 0 failed, 1 ignored`;
- targeted WASM full-native V4 parity: pass;
- targeted WASM Studio procedural V3 route: pass;
- weapon MOD writer/readback regression: pass;
- `git diff --check` for changed implementation files: pass.

Pełny `m2a-wasm --lib` ma jeden niezwiązany, istniejący w brudnym worktree
blocker frozen hash snapshot w `native_profile_a_adapter_is_exact_core_json...`.
Trasy WASM dotknięte implementacją attachmentów przeszły osobne testy.

## Native installation

Po sprawdzeniu, że oba cele nie istniały, exact pliki zostały skopiowane i
zweryfikowane byte-for-byte:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aweapdemo1.mod`
  — SHA-256 zgodny z canonical MOD;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aweaphak1.hak`
  — SHA-256 zgodny z canonical HAK.

## Owner proof checklist

W Area stoją dwie postacie przed punktem startowym:

- po lewej sceny (`x=7.5`): `RIGHT HAND - rhand_g`;
- po prawej sceny (`x=12.5`): `LEFT HAND - lhand_g`.

Właściciel powinien potwierdzić osobno w Toolsecie i NWN:

1. miecz jest widoczny przy właściwej dłoni obu kontrolek;
2. rękojeść znajduje się w dłoni, bez przesunięcia do root/floor/origin;
3. orientacja miecza nie jest lustrzana ani obrócona bokiem;
4. miecz dziedziczy ruch dłoni w idle, walk/run, attack, damage i death;
5. nie odłącza się, nie skaluje i nie rozciąga.

Do czasu wyniku właściciela:

- Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`.
