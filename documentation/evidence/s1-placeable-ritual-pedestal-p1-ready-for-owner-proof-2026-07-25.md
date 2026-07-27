# S1 placeable — handoff `ready_for_owner_proof`

1. Exact test-module `.mod` filename: `m2a_s1_plc_mod.mod`
2. Module name shown in Toolset: `Meshy2Aurora S1 Placeable Proof`
3. Exact Area name: `Meshy2Aurora S1 Ritual Pedestal`

Data zamrożenia: `2026-07-25`

Status: `ready_for_owner_proof`

To jest pierwszy i jedyny zmaterializowany kandydat statycznego placeable S1.
Agent nie uruchamiał ani nie przejmował Aurora Toolset/NWN. Po decyzji
właściciela z `2026-07-25` exact MOD i HAK zostały zainstalowane w natywnych
katalogach NWN oraz potwierdzone jako byte-identical. Widoczność pozostaje
decyzją właściciela.

## 1. Exact candidate identity

| Pole | Wartość |
|---|---|
| root lineage | `proof-output/s1-placeable-ritual-pedestal-p1-20260725` |
| Area resref | `m2a_s1_plc_ar` |
| HAK filename | `m2a_s1_plc_hak.hak` |
| HAK resref w `Mod_HakList` | `m2a_s1_plc_hak` |
| model resref | `m2a_s1_plc_ped` |
| texture resref | `m2a_s1_plc_tex` |
| UTP/archive resref | `m2a_s1_plc_utp` |
| object Tag | `m2a_s1_ritual_pedestal` |
| physical `placeables.2da` row | `16500` |
| placement | `X=10.0, Y=14.5, Z=0.0, Bearing=0.0` |
| profile | `STATIC_PLACEABLE` |
| materialization count | `1` |

Machine-readable handoff:
`proof-output/s1-placeable-ritual-pedestal-p1-20260725/ready-for-owner-proof.json`.

## 2. Immutable inputs

| Input | Bytes | SHA-256 |
|---|---:|---|
| `sample-3d/s1-placeable-ritual-pedestal-1500/source.glb` | 10011256 | `dad22a5c3490242458cb7a81e50c53e265abf75886f57f6bd8a770938c2f7372` |
| read-only base `placeables.2da` | 3019695 | `b772eafec5e6b380ad41e163e2a52585f2ddcec1c5bd7acea230b7e1a618df90` |

Path amendment 2026-07-27: the GLB was relocated to the canonical `sample-3d`
root without changing its bytes or SHA-256.

GLB readback: GLB 2.0, jeden node, jeden mesh, jeden primitive, jeden
materiał, cztery obrazy, `1477` trójkątów, bez skinów i animacji.

## 3. Exact output inventory

| Zasób | Resource type | Bytes | SHA-256 |
|---|---:|---:|---|
| `m2a_s1_plc_hak.hak` | HAK | 15740242 | `b6553710c6190346fcddaadec0040ed830fe4796cb448e7f455c5f06d727fceb` |
| `m2a_s1_plc_mod.mod` | MOD | 13400 | `ba0e978dc9d01d9ad4f69bd9e7857c687ba41ab782c8bc93d73370f52e0afa9b` |
| `m2a_s1_plc_ped.mdl` | `2002` | 137244 | `8a857aefbab79c8599fbe2db0168f010508caad0e5be01f14c995112620762b0` |
| `m2a_s1_plc_tex.tga` | `3` | 12582956 | `96a45ce0ac3b3eba8e54a56af1aeec425e2c63c441b8a87498f045bb365239dc` |
| `placeables.2da` | `2017` | 3019786 | `e8bce48f354c9e76597ccb95cd643a8b5c7a08f6ce211ec22a29d5507fab32a4` |
| `m2a_s1_plc_utp.utp` | `2044` | 1901 | `c86c04e3ab237d66b8545ab916277fea688f2f37ad20d5fa098c7620d8d69465` |
| `placeablepalcus.itp` | `2030` | 1464 | `60e0fc37d0ba62eaac586dc3d40093dde7fe279260c1adf8125cc9badc667b6c` |
| `m2a_s1_plc_ar.are` | `2012` | 2487 | `74a88db6c81e170e4bf592f94e0db3afb3803064b5aac9862e50808a116752fa` |
| `m2a_s1_plc_ar.git` | `2023` | 2581 | `e2057d52ae033228bfec7a344532fcb6cfaea4453afb8733bb156e3eb7591221` |
| `m2a_s1_plc_ar.gic` | `2046` | 500 | `485938dbb804a2c95dc839cac5a288cc014f47d0318fe1c8720a32ce94d8bfec` |
| `module.ifo` | `2014` | 2194 | `ee1bbd37116424bf03aa2578de2e47295089bd4929cfd5658739e9ca7ecf63d2` |
| `repute.fac` | `2038` | 1889 | `558f119cc9d39c5ddd5c93a3e8d88954ac83417fd7539d25278e9f8cf7b2017d` |
| `placeable-report.json` | report | 4265 | `fc4d5132b62ab4d4bb425ee930aa66803fbf507919adac81fde4664ab9fa4800` |
| `ready-for-owner-proof.json` | handoff | 2984 | `e267170646987de95800d233d1a4eebe804daebf472e6ed5f87d34ecbc123cd6` |

HAK ma dokładnie trzy zasoby: pełne `placeables.2da`, MDL oraz TGA. MOD ma
dokładnie siedem zasobów: IFO, FAC, ARE, GIT, GIC, UTP i ITP. Każda para
`(resref, resource type)` jest unikalna, a własny readback obu archiwów
potwierdził dokładny payload i resolver chain:

`GIT/UTP Appearance=16500 -> placeables.2da[16500].ModelName=m2a_s1_plc_ped -> m2a_s1_plc_ped:2002 -> m2a_s1_plc_tex:3`.

### 3.1. Zweryfikowana instalacja natywna

| Rola | Exact destination | SHA-256 |
|---|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_s1_plc_mod.mod` | `ba0e978dc9d01d9ad4f69bd9e7857c687ba41ab782c8bc93d73370f52e0afa9b` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_s1_plc_hak.hak` | `b6553710c6190346fcddaadec0040ed830fe4796cb448e7f455c5f06d727fceb` |

Oba cele były nieobecne przed kopiowaniem. Kopia użyła trybu no-clobber, a
hash każdego celu po instalacji jest identyczny z hashem źródła w kanonicznym
workspace. Instalacja nie uruchomiła i nie sterowała Toolset/NWN.

## 4. Wspólny pipeline modeli

Creature i placeable używają jednego `AuroraModelIrV1`, jednego ingestu GLB,
jednego parsera/readbacku MDL i jednego `write_binary_mdl`. Nazwa domenowa
`AuroraPlaceableIrV1` jest aliasem tego samego IR, a nie osobną kopią.
Rozdzielone są wyłącznie profile domenowe oraz resolvery:

- creature: appearance/UTC/GIT/GIC;
- placeable: `placeables.2da`/UTP/GIT/GIC/ITP;
- przyszły tile ma wejść do tego samego model IR oraz writera MDL.

## 5. Audyt clean-room manifestów

Read-only manifest UTP/GIT został zamrożony z lokalnego retail MOD:

- `Contest Of Champions 0492.mod`;
- module SHA-256:
  `26044acb596a15849cb93f583e76dd8b45eb66fe37a37fd02f0f844d8cc528b2`;
- blueprint `flamingbrazier`, UTP type `2044`, SHA-256:
  `b599c6062f85d017115e89a92ba9401aee4d6669dca6fc9117e43b8ede2af059`;
- Area/GIT resref `itemrestrictions`, GIT type `2023`, SHA-256:
  `484bb931a5e23a2de344d6c2d00944f05a5999f42480de3b4dc7007115355076`;
- odpowiadająca instancja ma StructID `9`.

Manifest ITP został porównany read-only z
`C:\Projects\Claude\rollnw\tests\test_data\user\scratch\placeablepalcus.itp`,
1462 bajty, SHA-256
`dad03a0f9ffe38eee4a2ec2686430e980d835990112dd63abb7795cfe242328c`.

Żaden retail payload nie został skopiowany do produktu lub fixture. Test
proweniencji zamraża hashe i manifesty. Wybrany retail GIC ma
niekanoniczny fizyczny układ `FieldIndices`, który celowo ścisły reader
odrzuca kodem `M6-GFF-LAYOUT-INVALID`. Jest to jawnie udokumentowane
ograniczenie analizy tego payloadu, nie nierozstrzygnięty kontrakt generacji:
wygenerowany minimalny GIC StructID `9` przechodzi canonical write/readback i
jest wyrównany indeksowo z GIT.

## 6. Verification ledger

Zielone kontrole:

- `cargo fmt --all -- --check`;
- pełne `cargo test -p m2a-core`, w tym istniejące regresje creature;
- placeable core: `7 passed`, jeden jawnie env-gated GLB audit;
- mutacyjne readbacki UTP/GIT: `2 passed` — template, appearance, state oraz
  brakujące/nieznane pola;
- wspólny IR/writer: `2 passed`;
- real GLB audit: `1 passed`;
- retail UTP/GIT manifest audit: `1 passed`;
- real ITP manifest audit: `1 passed`;
- placeable WASM boundary: `2 passed`;
- Studio: `29` plików, `176 passed`;
- Studio TypeScript typecheck;
- real Worker/WASM integration: `2` pliki, `6 passed`;
- `wasm-pack` build.

Pełny `cargo test -p m2a-wasm` ma `19 passed / 4 failed` i nie jest
raportowany jako zielony: cztery
wcześniejsze testy zamrożonych hashy Profile A/M7 pozostają czerwone po
istniejących w worktree zmianach algorytmu Profile A. W każdym z nich adapter
najpierw zgadza się z bezpośrednim core, a potem nie zgadza się ze starym
zamrożonym hashem/długością. Nie aktualizowano tych hashy i nie przypisano tego
do implementacji placeable. Z tego powodu checkbox pełnego workspace test
pozostaje otwarty.

Exact test names:

- `profile_a_native_tests::native_profile_a_json_errors_are_stable_and_strict`;
- `profile_a_native_tests::native_profile_a_adapter_is_exact_core_json_and_records_byte_proof`;
- `profile_a_native_tests::native_animated_profile_a_adapter_matches_core_and_rejects_invalid_mapping_json`;
- `m7_native_tests::ready_owned_boundary_is_exact_native_batch_oracle_and_immutable`.

## 7. Rozdzielone statusy proof i rozszerzeń

| Oś | `modelVisibility` | `proofCompleteness` |
|---|---|---|
| Aurora Toolset | `visible` | `verified` |
| NWN runtime | `visible` | `verified` |

- owner result: `owner_visual_proof_passed`;
- palette: `custom_itp_emitted`; obecność w palecie nadal niepotwierdzona;
- collision: `pwk_not_implemented`;
- interaction/animation/inventory: poza statycznym MVP;
- brak PWK nie jest traktowany jako błąd renderingu;
- `missing` nie jest wartością `modelVisibility`.

Pozytywny dowód jest zachowywany monotonicznie. Nie dopuszczono nowej
iteracji modelu: oba verdicts mają `modelVisibility=visible`.

## 8. Wynik proof właściciela

Właściciel zgłosił `pięknie zadziałało <3` i dostarczył dwa świeże obrazy:

- Toolset:
  `s1-placeable-ritual-pedestal-p1-owner-toolset-visible-2026-07-25.png`,
  SHA-256
  `b09c0ec912c58b8127006e2d19722748df93dc9822a9b757e69666a2e9dfe0b8`;
- NWN:
  `s1-placeable-ritual-pedestal-p1-owner-nwn-visible-2026-07-25.png`,
  SHA-256
  `2de846b35756faa010c4f68aa194e0268a664b223c13b77e1b6b4cfee010bef5`.

Toolset pokazuje exact filename `m2a_s1_plc_mod.mod`, Area resref
`m2a_s1_plc_ar`, zaznaczony obiekt `Meshy Ritual Pedestal` i czytelnie
wyrenderowany model. NWN pokazuje ten sam model w czytelnej scenie przed
graczem. Machine-readable verdict:

`s1-placeable-ritual-pedestal-p1-owner-visual-result-2026-07-25.json`.

Oddzielne ustalenia nie cofają pozytywnego verdictu:

- Toolset pokazuje dirty marker `*`;
- display name Area w drzewie to
  `Meshy2Aurora M0 binary vertical-slice area`, a nie nazwa z handoffu;
- zainstalowany MOD był byte-identical przed otwarciem, ale jego hash po
  proof nie mógł zostać ponownie odczytany, gdy Toolset trzymał blokadę pliku;
- HAK po proof nadal ma exact hash
  `b6553710c6190346fcddaadec0040ed830fe4796cb448e7f455c5f06d727fceb`;
- paleta i PWK pozostają osobnymi, niepotwierdzonymi osiami.
