# Item Studio V1 — implementacja pipeline’u

Status: `IMPLEMENTED_OFFLINE_PIPELINE_READY_FOR_OWNER_PROOF`

Data: 2026-07-30

Aktualizacja po ponownym audycie: skorygowany kontrakt Item V2 jest opisany w
[implementacji ustaleń audytu Item](item-audit-implementation-2026-07-30.md).
Zamrożony kandydat `r01` zachowuje dokładny kod i artefakty V1; nowa ścieżka
V2 nie regeneruje go i nie jest nowym kandydatem proof.

Branch/worktree:

- branch: `items`;
- worktree: `C:\Projects\meshy2aurora\.worktrees\items`.

## Wynik

`Item` jest osobnym przypadkiem domenowym i osobnym workflow Studio. Nie jest
wariantem Creature ani Placeable i nie dzieli przedmiotów na ręczne kategorie
`Weapon`, `Shield`, `Potion` itd. Kontrakt zawsze zaczyna się od:

```text
UTI.BaseItem
  -> drukowany wiersz baseitems.2da
  -> ModelType + ItemClass + wyjątki retail
  -> dokładne pola numeryczne UTI
  -> zasoby własne albo retailowe resolvery
```

## Macierz wspieranych kompozytorów

| `ModelType` / wyjątek | Pola UTI | Meshy GLB | Geometria i ikona |
|---|---:|---:|---|
| `0` | `ModelPart1` | 1 | jeden własny MDL; standardowa ikona źródłowa |
| `1` | `ModelPart1` + 6 kolorów | 1 | jeden własny MDL; tekstura PLT lub direct-color |
| `2` | `ModelPart1/2/3` | 3 | osobne MDL Bottom/Middle/Top |
| `3` | 19 × `ArmorPart_*` + 6 kolorów | 0 | `CAPART` mapuje slot, właściwa tabela `parts_*` ogranicza wariant, `PARTS_ROBE` dodaje maski |
| cloak, BaseItem `80` | `ModelPart1` + 6 kolorów | 0 | `CloakModel` wybiera retailowe `MODEL`, `TEXTURE` i `ICON`; prefix postaci pochodzi z `appearance.2da` |
| spell scroll, BaseItem `54/75` | `ModelPart1` + `PropertiesList` | 1 | ikona z `IPRP_SPELLS.Icon`, wybrana przez property `15` |

ModelType `3` nie oznacza dziewiętnastu wejściowych modeli. To dziewiętnaście
numerycznych selektorów kontekstu postaci. Studio pokazuje więc `0 Meshy GLBs`
i wymaga `CAPART.2da` oraz dwunastu tabel `PARTS_FOOT/SHIN/LEGS/PELVIS/CHEST/
BELT/NECK/FOREARM/BICEP/SHOULDER/HAND/ROBE`. Build sprawdza 19 dokładnych
mapowań `ArmorPart_* -> CAPART.MDLNAME/NODENAME -> parts_*`, zakres każdego
selektora i 19 binarnych masek robe. Nie emituje dla nich niezależnych MDL ani
wymyślonych warstw ikon.

## Retail `baseitems.2da`

Dokładne źródło użyte w testach:

- SHA-256:
  `3fbdcd012b55f0b869c6324cc7d4ece44ed63a78f00e7889e7acefac28c8fdf4`;
- `113` fizycznych wierszy;
- `17` strukturalnie nieaktywnych wierszy, w których `ItemClass` lub
  `ModelType` jest nullem/`****`;
- `96` rozwiązywalnych wierszy.

Etykieta `DELETED` nie jest sama w sobie kryterium nieaktywności. Część takich
wierszy ma nadal kompletny kontrakt techniczny; BaseItem `54` jest potrzebny
przez retailową ścieżkę scrolla i `IPRP_SPELLS`. Parser odrzuca wiersze
strukturalnie puste, a nie zgaduje statusu z tekstu etykiety.

`DefaultModel` i `DefaultIcon` są opcjonalnymi resrefami, nie liczbami.
Parser obsługuje także retailowe kolumny `%AnimSlash*`, drukowane numery
wierszy i końcowe spacje nagłówka. Ogólny parser 2DA pozostaje rygorystyczny;
jedynie czytnik tabel referencyjnych Item odcina końcowe puste rekordy
(występujące w retailowym `iprp_spells.2da`) i raportuje osobno SHA źródła,
znormalizowany rozmiar oraz liczbę odciętych rekordów.

## UTI i `PropertiesList`

Writer emituje natywny `UTI V3.2` i sprawdza własny semantic readback.
Potwierdzone typy retailowe:

- `BaseItem`: `INT`;
- `ModelPart*`, `ArmorPart_*`, kolory, `Charges`, `PaletteID`: `BYTE`;
- `StackSize`: `WORD`;
- `Cost`, `AddCost`: `DWORD`;
- `PropertiesList`: lista struktur z
  `PropertyName WORD`, `Subtype WORD`, `CostTable BYTE`, `CostValue WORD`,
  `Param1 BYTE`, `Param1Value BYTE`, `ChanceAppear BYTE`.

Dla ikony spell scrolla musi istnieć dokładnie jedno property
`PropertyName=15`; jego `Subtype` wybiera wiersz `IPRP_SPELLS`.

## Meshy → part Item

Dla własnego partu pipeline:

1. pobiera kanoniczny GLB z `sample-3d`;
2. opcjonalnie wybiera dokładny, jednoznaczny root `sourceNode` domyślnej sceny;
3. konwertuje GLB do wspólnego IR i profilu `ITEM_PART_STATIC_RIGID`;
4. stosuje pivot i jednolitą skalę, a translację/obrót zapisuje w kontrolerze
   korzenia MDL;
5. dzieli geometrię przekraczającą granicę 65 535 indeksów jednego strumienia
   bez usuwania trójkątów;
6. zapisuje niezależny binarny MDL i wykonuje własny readback;
7. zapisuje teksturę jako direct-color TGA albo natywny `PLT V1`;
8. dla profili `STANDARD/LAYERED` ścieżka V2 rasteryzuje faktyczne trójkąty
   modelu w deterministycznej projekcji izometrycznej, próbkuje UV z tekstury
   i pozostawia przezroczystość poza sylwetką;
9. wszystkie party ikony korzystają ze wspólnych granic projekcji, więc ich
   warstwy mają ten sam kadr, skalę i położenie.

PLT ma dokładny 24-bajtowy header, warstwy Metal/Cloth/Leather i semantic
readback. Nieprzezroczyste źródło jest wymagane, ponieważ ten writer nie udaje
obsługi niezweryfikowanej semantyki alfa PLT.

## Podgląd i składanie

Studio ładuje rzeczywiste GLB przez Three.js. Widoki `Composed` i `Exploded`
nie są diagramem zastępczym. Podgląd:

- mapuje współrzędne źródłowe GLB Y-up do semantyki wyjściowej Aurora Z-up;
- stosuje ten sam per-part transform i pivot co writer;
- wiąże cache z tożsamością obiektu `File`, a nie tylko nazwą, rozmiarem i datą;
- wymaga globalnie jednoznacznego `sourceNode` i potwierdza, że jest rootem
  domyślnej sceny — dokładnie tak samo jak converter;
- przelicza AABB i klasyfikuje każdą sąsiednią parę jako
  `TOUCHING`, `GAP` albo `OVERLAP`;
- reaguje na transform bez ponownego parsowania ciężkich GLB.

AABB jest tylko podpowiedzią preview i nie blokuje Build. Worker sam przygotowuje
dokładną geometrię obu partów i wykonuje autorytatywny pomiar powierzchni
trójkątów przez BVH i testy zawierania brył. Raport wiąże wynik z hashami
źródeł, transformów/sourceNode, liczbami trójkątów i hashem pomiaru. Każdy
autorytatywny `GAP` powyżej tolerancji lub `OVERLAP` blokuje paczkę. Tryb
`Icon` pokazuje geometrię w kierunku projekcji autora ikony; Review dekoduje
dokładne wyemitowane TGA, pokazuje każdą warstwę i kompozyt.

Nie powstaje jeden „złożony MDL”. UTI zachowuje numery wariantów, a Aurora
ładuje niezależne zasoby według recepty `ModelType`.

## Namespace i paczka

Deterministyczny allocator sprawdza zajęte klucze `resourceType:resref` dla:

- modeli, ikon, tekstur i UTI;
- HAK i MOD;
- wspólnego resrefu Area dla ARE/GIT/GIC.
- osobnego UTC typu `2027` dla wyposażonego fixture’u CAPART/Cloak.

Przy kolizji zwykłego partu wybiera następny wolny wariant bez nadpisywania
zasobu. Jawne resrefy kontekstowe są fail-closed.

Worker emituje:

- niezależne MDL/tekstury/ikony tylko wtedy, gdy profil naprawdę je posiada;
- natywny UTI;
- HAK z własnym ERF readbackiem;
- candidate-bound testowy MOD zawierający IFO, Area, GIC/GIT/FAC i dokładnie
  jeden umieszczony UTI dla zwykłych profili;
- dla CAPART/Cloak osobny MOD z jednym stworzeniem oraz UTC, z UTI wyposażonym
  odpowiednio w native slot `2` albo `8192`; GIT i UTC zachowują ten sam
  jawny kontekst `Appearance_Type`, `Race`, `Gender`, `Phenotype`, prefix
  wyprowadzony z wiersza `appearance.2da` oraz SHA-256 tej tabeli;
- raport z hashami, budżetem, rozdziałem `meshyPartCount` /
  `referenceSelectorCount` oraz granicą proof.

## Kandydat właściciela

Zamrożony kandydat `item-composed-r01-20260730` ma `22 955` trójkątów, trzy
niezależne MDL wariantu `251`, UTI/HAK/MOD z readbackiem PASS oraz byte-identical
instalację MOD/HAK do natywnych katalogów NWN. Dokładny handoff:

[item-composed-r01 — ready for owner proof](evidence/item-composed-r01-ready-for-owner-proof-2026-07-30.md).

## Granica proof

Agent nie uruchamiał, nie adoptował i nie sterował Aurora Toolset ani NWN.
Status kandydata:

- `ready_for_owner_proof`;
- `modelVisibility=not_tested`;
- `proofCompleteness=missing`.

`ready_for_owner_proof` oznacza wyłącznie ukończony, zahashowany i
zainstalowany handoff. O wyniku wizualnym decyduje właściciel.

## Weryfikacja

Pokryte testami i audytem:

- wszystkie cztery `ModelType`, wyjątki CAPART/Cloak/IPRP i retailowe tabele;
- dokładne typy oraz round-trip korpusu retailowych UTI;
- TGA, PLT, globalnie jednoznaczny `sourceNode`, ikonę z geometrii, wspólny
  kadr warstw, transformacje wieloosiowe/hierarchiczne, seam gate, segmentację
  i budżet 300 000;
- allocator kolizji, UTI, HAK, MOD i semantic readback;
- unit/integration testy Rust, UI, Worker/WASM, typecheck i build produkcyjny;
- rzeczywisty browser flow ModelType `2`: 3 Meshy MDL, 3 ikony, `22 955`
  trójkątów;
- rzeczywisty browser flow ModelType `3`: 0 Meshy MDL, 19 selektorów,
  0 własnych ikon i poprawny CAPART UTI/MOD/HAK.

Końcowy wynik automatyczny:

- `m2a-core --lib`: 83 PASS, 2 jawnie ignorowane;
- Item core: 25 PASS;
- dokładny retail conformance: 4 PASS;
- Item Worker/WASM w Chromium: 2 PASS;
- UI: 228 PASS, 1 jawnie pominięty;
- TypeScript typecheck, `cargo clippy` dla obu bibliotek oraz produkcyjny
  build WASM/Vite: PASS.

Pełny, niezawężony runner repozytorium nadal ma dwa niezależne testy Creature,
które wymagają nieobecnego lokalnego pliku
`sample-3d/h2-clockwork-sentinel-1500/source.glb`. Nie jest on zastępowany ani
kopiowany przez pipeline Item; jego brak nie blokuje odizolowanych testów Item.

Powiązany audyt:
[audyt dekompilacji Aurora Item](audyt-dekompilacji-aurora-item-model-parts-2026-07-29.md).
