# Audyt i wyciąg grafik stanowisk runicznych / ulepszania broni

Data: 2026-07-26

## Wynik

Wyciąg jest gotowy w:

`C:\Projects\meshy2aurora\proof-output\reference-extracts\tlc-rune-upgrade-workstations-20260726`

Zawiera:

- 10 modeli MDL;
- 10 odpowiadających im kolizji PWK;
- 8 oryginalnych tekstur DDS;
- 8 podglądów PNG i jedną zbiorczą planszę;
- dwa UTP kowadeł pobrane bezpośrednio z modułu;
- aktywne `placeables.2da`;
- osobny manifest pochodzenia i SHA-256 dla każdego archiwum źródłowego.

## Źródło prawdy

| Artefakt | SHA-256 |
| --- | --- |
| `the_last_city - codex.mod` | `b774037f160a9be3fcc0e985dde880a0ae97e10af2bf868c38a3f6573cbb021e` |
| `lc_2da.hak` | `c4869ae516a8598f0db5e31b473abba9b15b3b3f04880eb02f4a3a37191a9e8f` |
| `witcher.hak` | `869b42d562895897f74f9d408ec9edd942420d34076d02c68cb6c2d439e7549a` |
| `cep3_core0.hak` | `b3f2f5197183e2124db8be821f375f48895eb579f5be64cd1eb2467d39c40edb` |
| `cep3_core2.hak` | `4824096d449456b3d0cd0352227ad5d7e2b8e02399753bec4bea0c2e8ebbd3dc` |

## Klasyfikacja obiektów

### 1. Bezpośredni kandydat: stół runiczny

Aktywny `placeables.2da`, wiersz `11483`, deklaruje:

```text
"Table: Rune* (PLUSH HYENA of DOOM)" ... phod_tablimpet
```

Łańcuch zasobów:

```text
placeables.2da:11483
  -> phod_tablimpet.mdl
  -> phod_tablimpet.dds
  -> phod_tablimpet.pwk
```

To jedyny znaleziony obiekt, którego nazwa w aktywnej tabeli Appearance mówi
wprost, że jest stołem runicznym. Komplet pochodzi z `cep3_core2.hak`.

### 2. Rodzina stołów warsztatowych

Wiersze `5516..5522` aktywnego `placeables.2da` wskazują siedem modeli
`nw2workbench1..7`. Każdy ma własny MDL i PWK; tekstury są współdzielone
zgodnie z tabelą:

| Wiersz | Model | Tekstura |
| ---: | --- | --- |
| 5516 | `nw2workbench1` | `nw2_mc_cbench01` |
| 5517 | `nw2workbench2` | `nw2_mc_tubs01` |
| 5518 | `nw2workbench3` | `nw2_mc_cbench03` |
| 5519 | `nw2workbench4` | `nw2_mc_cbench01` |
| 5520 | `nw2workbench5` | `nw2_mr_awbench` |
| 5521 | `nw2workbench6` | `nw2_mr_wwbench` |
| 5522 | `nw2workbench7` | `nw2_mr_maskswork` |

Ta grupa jest dobrym źródłem graficznym dla stołu ulepszania broni, ale sam
moduł nie wiąże mechaniki run z jednym z tych modeli.

### 3. Kowadła faktycznie użyte w lokacjach kowala

Moduł zawiera UTP:

| UTP | Nazwa | Appearance | Model z `placeables.2da` |
| --- | --- | ---: | --- |
| `witcher278` | `[W] Kowadło` | 6042 | `plc_wit42` |
| `witcher279` | `[W] Kowadło 2` | 6043 | `plc_wit43` |

Instancje występują między innymi w `wnd_blacksmith`, `wnd_armorer` i
`d_spiders`. UTP są jednak statyczne (`Static=1`) i nieużywalne
(`Useable=0`), więc potwierdzają scenografię kowala, nie punkt wejścia do UI
ulepszania.

## Ważne rozróżnienie funkcjonalne

Zasoby i skrypty modułu pokazują, że obsługa run/ulepszania jest realizowana
przez `lib_loot_rune`, `rune_enh`, `rune_slot`, `rune_attack` i testowy
`test_loot`. Nie znaleziono twardego odwołania tych skryptów do jednego
konkretnego UTP stołu. Dlatego:

- `phod_tablimpet` jest bezpośrednim dopasowaniem semantycznym;
- `nw2workbench1..7` są kandydatami wizualnymi;
- `plc_wit42/43` są potwierdzonymi elementami lokacji kowala.

Nie należy opisywać wszystkich tych modeli jako już interaktywnie podpiętych
do mechaniki run.

## Integralność wyciągu

Każdy katalog źródłowy ma `extraction-manifest.json`. Najważniejsze zasoby:

| Zasób | Typ | SHA-256 |
| --- | ---: | --- |
| `phod_tablimpet.mdl` | 2002 | `f23465786bead3b23a31b7f009b5ad8d01abfa96cad4ebc2137b89c93a27cb78` |
| `phod_tablimpet.dds` | 2033 | `f18ef8f55b898ad3af465643cfd2f0c7df4a25761a347c19e861a38dd80d1377` |
| `phod_tablimpet.pwk` | 2053 | `e2c210e07616695da2f4ea47a5317cf61f5bbdb8fdc49c2ca84cf282dd7ea54c` |
| `plc_wit42.mdl` | 2002 | `89964412b2c9f28071232e713440d66dc0e2db71e41198e0a0b3d4f111764c99` |
| `plc_wit43.mdl` | 2002 | `bd731bedfbd3bc8e7d1a1b67ac82af3cf040df5d93e99e70ee0fb45f76a5956c` |
| `ob_anvilsma01.dds` | 2033 | `e51275ca05ea5b4c4633155589a255f45598f4a4005026c8ff9640a89fd80a1f` |
| `previews/contact-sheet.png` | PNG | `ef55b0007ebbc826ebffa73202e0bfcdc1dbfe2cf5c643129574ef13f06f1e5c` |

## Uwaga parserowa

`cep3_core2.hak` zawiera niezwiązany z wyciąganym stołem historyczny resref
`tr's_ruin_pic_01`. Apostrof wykracza poza kanoniczny profil resrefów
Meshy2Aurora, dlatego ścisły parser ERF zatrzymuje cały indeks.

Do wyciągu dodano jawny tryb
`extract_erf_resources --lenient-unrelated-resrefs`. Tryb:

- nadal sprawdza sygnaturę i wersję ERF/HAK/MOD;
- limituje liczbę wpisów;
- sprawdza granice tabel i payloadów;
- wybiera wyłącznie dokładne pary `(resref, resourceType)`;
- nie obniża zasad produkcyjnego parsera.

Wyszukiwarka archiwów dostała także `--resref-only`, aby przeglądanie dużych
HAK-ów nie skanowało niepotrzebnie każdego payloadu.

## Status

- [x] potwierdzony dokładny moduł i jego hash;
- [x] odczytana uporządkowana lista HAK;
- [x] znaleziony bezpośredni model stołu runicznego;
- [x] znalezione stoły warsztatowe i kowadła;
- [x] powiązane `placeables.2da -> MDL -> DDS -> PWK`;
- [x] wyciągnięte oryginalne zasoby;
- [x] wygenerowane PNG do szybkiego przeglądania;
- [x] zapisane manifesty pochodzenia i SHA-256;
- [ ] decyzja projektowa: który model ma zostać bazą nowego placeable;
- [ ] ewentualne podpięcie wybranego placeable do skryptu/NUI ulepszania.

Wyciąg jest lokalnym materiałem referencyjnym. Przed dystrybucją należy osobno
zweryfikować licencje właściwych paczek contentu.
