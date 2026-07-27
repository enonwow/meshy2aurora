# Wyciąg grafik NUI: runy i ulepszanie przedmiotów

Data: 2026-07-26

## Korekta zakresu

Właściwym przedmiotem audytu są grafiki interfejsu NUI, nie tekstury modeli
placeable. Poprawny wyciąg znajduje się w:

`C:\Projects\meshy2aurora\proof-output\reference-extracts\tlc-nui-rune-upgrade-graphics-20260726`

Wyciąg zawiera 12 dokładnych zasobów PNG typu `2080` z
`lc_system_icons.hak` oraz trzy plansze podglądowe.

## Dowód z kodu NUI modułu

Źródłem odwołań jest dokładny moduł:

| Źródło | SHA-256 |
| --- | --- |
| `the_last_city - codex.mod` | `b774037f160a9be3fcc0e985dde880a0ae97e10af2bf868c38a3f6573cbb021e` |
| `lib_loot_rune.nss` / typ 2009 | `ac880dda921f3e236ab2b1ae40eff781f9dbc9f4a427b67e357103c3cb62d105` |
| `lib_loot_pc.nss` / typ 2009 | `2370f0580add5e5318c48b584560f66688e1364fb7a99d721223bd6b5f90d200` |
| `lib_nui.nss` / typ 2009 | `e026ace025470895cafbae8c5cd5ccf05eae8beca21982e6bc9937ebdd692ed5` |

### Okno run

`lib_loot_rune` wskazuje:

- `rune_background` jako tło `NuiDrawList`;
- `rune_btn_background` jako ramkę każdego slotu;
- `rune_empty` jako grafikę pustego slotu;
- wartość `LOOT_RUNE_IN_SLOT` jako dynamiczną grafikę zajętego slotu;
- `custom_close_nui` jako przycisk zamknięcia;
- `information64` przez wspólną funkcję `NuiAddInfoImage`.

Instancje przedmiotów testowych zapisane w `enon_test_loot.git` i
`wnd_testing.git` potwierdzają wartości `LOOT_RUNE_IN_SLOT`:

- `rune_attack`;
- `rune_enh`.

### Okno ulepszania przedmiotów

`lib_loot_pc` w `LootPCUpgradeWindowCreate` i wierszach właściwości wskazuje:

- `loot_work_2` — tło okna;
- `custom_pick_nui` — wybór przedmiotu;
- `custom_close_nui` — zamknięcie;
- `information64` — pomoc/informacje;
- `add_prop_item` — dodanie właściwości;
- `upd_prop_item` — ulepszenie właściwości;
- `re_prop_item` — zamiana właściwości.

## Resolver HAK

Wszystkie znalezione grafiki pochodzą z jednego podpiętego HAK-a:

| HAK | SHA-256 |
| --- | --- |
| `lc_system_icons.hak` | `c07830669c9d476e54094fa68541b3925996a4a3dc27d395ef43941090e37be3` |

`lc_icons.hak` i `lc_crafting.hak` nie zawierają dokładnych kluczy tej listy.

Wszystkie zasoby mają typ `2080`, czyli natywny PNG używany przez NUI.

## Mapa zasobów

| Funkcja | Odwołanie NUI | Dokładny klucz HAK | Rozmiar |
| --- | --- | --- | ---: |
| runy | `rune_background` | `rune_background` | 1024×1024 RGB |
| runy | `rune_btn_background` | `rune_btn_backgro` | 1024×1024 RGBA |
| runy | `rune_empty` | `rune_empty` | 1024×1024 RGBA |
| runy | `rune_attack` | `rune_attack` | 430×490 RGBA |
| runy | `rune_enh` | `rune_enh` | 485×542 RGBA |
| wspólne | `custom_close_nui` | `custom_close_nui` | 1024×1024 RGBA |
| wspólne | `information64` | `information64` | 64×64 RGBA |
| ulepszanie | `loot_work_2` | `loot_work_2` | 1024×1024 RGB |
| ulepszanie | `add_prop_item` | `add_prop_item` | 1024×1024 RGBA |
| ulepszanie | `upd_prop_item` | `upd_prop_item` | 1024×1024 RGBA |
| ulepszanie | `re_prop_item` | `re_prop_item` | 1024×1024 RGBA |
| ulepszanie | `custom_pick_nui` | `custom_pick_nui` | 1024×1024 RGBA |

### Szczególny przypadek 16 znaków

Kod NWScript używa `rune_btn_background` (19 znaków), ale ERF/HAK przechowuje
klucz `rune_btn_backgro` (16 znaków). Wyciąg zachowuje dokładną tożsamość
archiwum i dlatego plik nazywa się `rune_btn_backgro.png`. Dokumentacja
pokazuje oba identyfikatory zamiast cicho zmieniać nazwę.

## Integralność

Manifest:

`proof-output/reference-extracts/tlc-nui-rune-upgrade-graphics-20260726/nui-images/extraction-manifest.json`

zawiera dla każdego pliku:

- dokładny resref i resource type;
- długość payloadu;
- SHA-256 pliku;
- SHA-256 źródłowego HAK-a.

Wybrane skróty:

| Plik | SHA-256 |
| --- | --- |
| `rune_background.png` | `6f3d2fd128ce0787f6bc679e6b789055ccdbe5d74f38e6e8c74fd2878fa3d453` |
| `rune_btn_backgro.png` | `17e92eb74136d0e076350c41b28f37bf819163c0d2f78cb365b6bab12155a0ea` |
| `rune_attack.png` | `88bd02a63be0d5b03c7b8afcd39b246ef548e088104e5b60d8c26fcd7b56392b` |
| `rune_enh.png` | `09b5326e3dc3081e663f0d3f7a1364c7ed86a7a9ddf54614d9128dde1974de3f` |
| `loot_work_2.png` | `061579f4a6dd012cfa2055052718d38bd490ea59571d0909827cd4018b3b30df` |
| `add_prop_item.png` | `538eef14a017a033ad90fca8342184716b5b223e20b26f0a3c85812919d3b5ef` |
| `upd_prop_item.png` | `98bf1b54f08516483a30e7ade27a3c659f1042bd487709119458ec4df78b186a` |
| `re_prop_item.png` | `e201d8f74616cbf0f0808f1b4eb586aec230c1fb78de1ad005e88da3ee24649f` |

## Status

- [x] odnalezione dokładne funkcje tworzące oba okna NUI;
- [x] odzyskane wszystkie stałe i dynamiczne resrefy grafik;
- [x] potwierdzone dynamiczne grafiki run z instancji GIT;
- [x] rozwiązany HAK dostarczający payloady;
- [x] wyciągnięte dokładne PNG typu 2080;
- [x] zweryfikowane rozmiary, kanał alfa i SHA-256;
- [x] wygenerowane osobne plansze run, ulepszania i kompletna;
- [ ] opcjonalnie: odtworzenie pełnego układu NUI w podglądzie aplikacji.

Wyciąg jest lokalnym materiałem referencyjnym. Przed redystrybucją należy
zweryfikować licencję właściwej paczki contentu.
