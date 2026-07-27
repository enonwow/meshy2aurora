# TLC Meshy P20K Placeables V1 — `ready_for_owner_proof`

1. Exact test-module `.mod` filename: `m2a_tlcm3_mod.mod`
2. Module name shown in Toolset: `Meshy2Aurora TLC Meshy P20K Trio`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Data zamrożenia: `2026-07-25`

Status: `ready_for_owner_proof`

## 1. Wynik

Pakiet zawiera trzy różne statyczne placeable wygenerowane przez Meshy 6:

| Placeable | Meshy preview task | Meshy refine task | Źródło Meshy | Finalny GLB | MDL readback |
|---|---|---|---:|---:|---:|
| Civic Reliquary | `019f9b19-50ba-7ec7-bcce-4452b1dd4e78` | `019f9b1a-a953-77a2-8d0a-825d6fb8fa6a` | 21 550 tris | 20 000 tris | 20 000 faces / 60 000 indices |
| Aether Street Lamp | `019f9b1b-4ed3-7f64-98c4-3cdccc268092` | `019f9b1c-1e20-7813-8711-d6cfc8eeab44` | 21 456 tris | 20 000 tris | 20 000 faces / 60 000 indices |
| Sewer Ward Barricade | `019f9b1b-a06b-77df-a4bc-476e112c4dc4` | `019f9b1c-ec55-7849-9a31-c51986251f05` | 22 037 tris | 20 000 tris | 20 000 faces / 60 000 indices |

Kierunek wizualny był opisany wyłącznie na wysokim poziomie:
industrial-gothic dark fantasy, osmolony kamień, czarne żelazo, oksydowany
mosiądz i miejskie pieczęcie magiczne. Nie kopiowano modelu, tekstury, UV ani
innego payloadu z `the_last_city - codex.mod`.

Źródłowy lineage Meshy zużył 90 kredytów. Dodatkowe 10 kredytów zużył
redundantny refine powstały po timeoutcie lokalnego monitora; task
`019f9b1b-19f8-77c3-a010-f44f5d37bb2b` nie należy do zamrożonego kandydata.

## 2. Finalne źródła Meshy

| Plik | Bytes | SHA-256 | Wierzchołki | Wysokość |
|---|---:|---|---:|---:|
| `tlc-civic-reliquary-20000.glb` | 11 308 892 | `865fe8eaab2e2996354faf19491b7ccf341a3239a30762c78cadad3b3a0dc994` | 25 721 | 3,6 m |
| `tlc-aether-lamp-20000.glb` | 12 722 972 | `310a6597f92bfcd391f44d293585417fa53abe3f1a576d93a7e0dc2c586b1ac7` | 26 086 | 3,2 m |
| `tlc-sewer-ward-20000.glb` | 11 682 532 | `4192345b22327997708bad441fc70ce35be8db45331aa01dd928eeb432cc28c1` | 24 990 | 2,0 m |

Deterministyczny reduktor zachowuje materiały i osadzone obrazy Meshy,
usuwa wierzchołki nieużywane po uproszczeniu, odrzuca trójkąty niespełniające
progu bezpieczeństwa Aurory i domyka dokładny target przez podział największych
bezpiecznych trójkątów. Ponowna generacja wszystkich trzech GLB dała identyczne
SHA-256.

## 3. Wspólny pipeline

Każdy model przeszedł tę samą ścieżkę:

`Meshy GLB -> GLB ingest -> Profile A -> AuroraModelIrV1 -> binary MDL writer`

Dopiero potem adapter placeable dodaje:

`placeables.2da -> UTP/GIT/GIC/ITP -> PWK -> HAK/MOD`

Creature zachowuje domyślny guardrail `5 000 / 10 000`. Statyczny placeable
używa osobnego, skompilowanego profilu ostrzeżenie/blokada
`10 000 / 21 845`, zgodnego z limitem jednego mesha `65 535` indeksów.
Nie powstał drugi writer ani równoległy format modelu.

## 4. Exact candidate identity

| Pole | Wartość |
|---|---|
| root lineage | `proof-output/tlc-meshy-p20k-placeables-v1-20260725` |
| lane | `TLC_MESHY_P20K_PLACEABLE_TRIO_V1` |
| Area resref | `m2a_tlcm3_ar` |
| HAK filename | `m2a_tlcm3_hak.hak` |
| HAK resref | `m2a_tlcm3_hak` |
| MOD filename | `m2a_tlcm3_mod.mod` |
| appearance rows | `16500`, `16501`, `16502` |
| model resrefs | `m2a_tlcm_rel`, `m2a_tlcm_lamp`, `m2a_tlcm_ward` |
| placement X/Y | `6.5/14.5`, `10.0/14.5`, `13.5/14.5` |
| materialization count | `1` |

Machine-readable handoff:
`proof-output/tlc-meshy-p20k-placeables-v1-20260725/ready-for-owner-proof.json`.

## 5. Render i kolizja

| Model | MDL SHA-256 | PWK SHA-256 | PWK readback |
|---|---|---|---|
| `m2a_tlcm_rel` | `c12b8e4b11f3923375402a49dbecf1307bda228258c553b23a4e4371ed6d030f` | `d846257d1cc5f9d623d771b842d390b3ea9d1ab48d8039a22d14a41e69511835` | passed |
| `m2a_tlcm_lamp` | `9765663ec2347de7a94b5fd4e4ddea3fe9e6fba86fa365507b0a1be2348c063a` | `cae94d2155b4ee81fb81fa2e65df631af4d66fdc5bedc61f18c952290e975daf` | passed |
| `m2a_tlcm_ward` | `5be4664df4d86b1b9aad287a7387262dbcac9d7fb991651b6d5cdb65299aaedc` | `4c0f6860bba13ca4ad89b9c2825e603c3ca05876e49f87e35c247fc7a75d6e5f` | passed |

Każdy PWK:

- jest tekstowym `nwn1-ascii-pwk`;
- ma ten sam resref co odpowiadający MDL;
- zawiera konserwatywny prostokąt obrysu world-XY;
- ma `4` wierzchołki, `2` faces i surface ID `7`;
- przeszedł własny parser/readback offline.

Runtime collision pozostaje `not_tested` do testu właściciela w NWN.

## 6. MOD/HAK i instalacja

| Artefakt | Bytes | SHA-256 | Native destination |
|---|---:|---|---|
| `m2a_tlcm3_mod.mod` | 19 652 | `60b943b3090487aecb48034667ebc6e41a9383a78577c4860da0b20b895dcdbb` | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_tlcm3_mod.mod` |
| `m2a_tlcm3_hak.hak` | 45 818 454 | `0ce9f1097941a928861ee8ee06e44fe174420054c009fd05a16546e5282c55cb` | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_tlcm3_hak.hak` |

Oba cele były nieobecne przed kopiowaniem. Kopie wykonano w trybie
no-clobber i po instalacji potwierdzono byte-identical SHA-256. Nie uruchamiano
ani nie kontrolowano Aurora Toolset ani NWN.

## 7. Weryfikacja

- [x] Kanoniczny workspace potwierdzony.
- [x] Trzy różne modele wygenerowane przez Meshy 6.
- [x] Zamrożone task ID preview/refine i hashe źródeł.
- [x] Każdy finalny GLB ma dokładnie `20 000` bezpiecznych trójkątów.
- [x] Deterministyczna regeneracja GLB daje identyczne hashe.
- [x] Profile A placeable przyjmuje limit `21 845`; creature pozostaje bez zmian.
- [x] Wszystkie modele przechodzą wspólny `AuroraModelIrV1` i binary MDL writer.
- [x] Binary MDL readback potwierdza po jednym meshu i `60 000` indeksów.
- [x] `placeables.2da`, trzy UTP, trzy instancje GIT/GIC i trzy wpisy palety są spójne.
- [x] Trzy PWK zostały zapisane i odczytane własnym parserem.
- [x] MOD przechodzi `inspect_module`.
- [x] 176 testów istotnych dla ingest/profile/model/placeable/archive przechodzi; 1 test owner-input jest pominięty.
- [x] `cargo check -p m2a-core --examples` przechodzi.
- [x] Exact MOD/HAK zainstalowane natychmiast po zamrożeniu.
- [x] Native MOD/HAK są byte-identical z kanonicznymi artefaktami.
- [ ] Właściciel potwierdza trzy modele w Aurora Toolset.
- [ ] Właściciel potwierdza te same trzy modele w NWN.
- [ ] Właściciel potwierdza kolizję każdego modelu w NWN.

## 8. Handoff dla właściciela

1. Otworzyć `m2a_tlcm3_mod.mod`.
2. Potwierdzić nazwę `Meshy2Aurora TLC Meshy P20K Trio`.
3. Otworzyć Area `Meshy2Aurora M0 binary vertical-slice area`.
4. Sprawdzić trzy obiekty ustawione w jednym rzędzie.
5. Uruchomić Test Module bez rebuildowania lub podmieniania HAK.
6. Podejść do każdego obiektu z kilku kierunków i potwierdzić blokowanie ruchu.

Do czasu raportu właściciela:

- `modelVisibility=not_tested`;
- `proofCompleteness=missing`;
- `collisionRuntimeVerdict=not_tested`.
