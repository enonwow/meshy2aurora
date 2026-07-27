# Incaxje: NeverBlender runtime baseline — 2026-07-18

Status: `FAILED / MODEL-NOT-VISIBLE / NOT-PRODUCT-WRITER-PROOF`.

## Cel i granica

Wlasciciel zlecil osobny test odpowiedzi na pytanie, czy ten sam Meshy H1 po
konwersji przez Blender/NeverBlender bedzie widoczny w Aurora Toolset lub w
NWN. Wariant nazywa sie **`incaxje`** i dostal wlasny, drugi wpis
`appearance.2da`.

To jest izolowany **baseline zewnetrznego eksportera/kompilatora**, nie proof
writera Meshy2Aurora i nie nowa zaleznosc produktu. NeverBlender i CleanModels
nie sa importowane ani uruchamiane przez kod produktu; Meshy2Aurora zbudowal
wlasnym writerem tylko `2DA`, `HAK` i `MOD` dla tego eksperymentu.

## Fakty wejsciowe i konwersja

| Pole | Fakt |
| --- | --- |
| GLB | `sample-3d/h1-humanoid-1500/source.glb`, SHA-256 `3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`; path amended 2026-07-27 after byte-identical canonical relocation |
| Eksporter | Blender `4.0.2`, NeverBlender `4.1.0`, izolowany profil uzytkownika |
| Eksport ASCII | `proof-output/incaxje-neverblender-proof/incaxje.ascii.mdl`; SHA-256 `d6060a9a873ad894a35c571974f8cdfbcfb1fa970753f5ee540004bbaba9b507` |
| Semantyka ASCII | `newmodel incaxje`, `bitmap incaxjet`, jeden `skin`, 1334 wierzcholki, 1556 trojkatow, 24 grupy wag |
| Kompilacja referencyjna | lokalny CleanModels WASM `v4.0.0-rc7`; bez ostrzezen |
| Binarny MDL | `proof-output/incaxje-neverblender-proof/incaxje.mdl`; 129864 B; SHA-256 `a30425cd49fe9cbe7cefc1fcdce255256ecf3a2361e779366a04038043acaeae` |

Pierwsza proba z lokalnym `nwnmdlcomp.exe` nie byla uzyta: binarny Windows
odrzucil `-i`, ktorego kod zrodlowy ma tylko w nie-Windowsowej galezi. Nie
zmieniano rejestru, konfiguracji NWN ani `nwtoolset.ini`.

## Pakiet Meshy2Aurora

Wlasny proof-harness `crates/m2a-core/examples/materialize_incaxje_reference_proof.rs`
zachowuje poprzedni wpis `15100` dla `m2a_m6p01` i appenduje fizyczny wiersz
`15101`:

```text
LABEL=INCAXJE_NEVERBLENDER
MODELTYPE=S
RACE=incaxje
```

HAK zawiera zarowno dotychczasowy `m2a_m6p01/m2a_m6t01`, jak i
`incaxje/incaxjet`; moduł ma testowego creature z `Appearance_Type=15101`.

| Artefakt | SHA-256 | Readback |
| --- | --- | --- |
| `m2a_codex_aproof.hak` | `c6e1fd4214a55a8686489e61394ec0512c546a7acf37252d41f2a254184d9a15` | `PASS` — wszystkie piec zasobow sa w ERF |
| `m2a_codex_aproof.mod` | `9af06d6a3155a1043bd0343a23164d7ba88a9e30fef3e0685b780ae666fbb1cd` | `PASS` — creature wskazuje 15101 |
| `appearance.2da` | `204cf62e3bd093c6230499fc09056fa5ce228ee41ee7ef2b295682dbb0ee9573` | `PASS` |

Takie same hashe HAK i MOD zostaly potwierdzone w katalogach NWN wlasciciela
przed uruchomieniem proofu.

## Istotny rozjazd own readera

`inspect_binary_mdl` odrzuca referencyjny plik binarny:

```text
M2A-MDL-SKIN-VARIANT-AMBIGUOUS at 5196:
skin node-to-bone pointer 0x00000000 matches neither explicit skin profile
```

To jest fakt o obecnych profilach readbacku Meshy2Aurora w porownaniu z
wariantem CleanModels. Nie jest dowodem ani poprawnosci, ani niepoprawnosci
pliku dla engine, bo ponizszy proof wizualny nie pokazal modelu. W manifestcie
jest dlatego jawny status `UNSUPPORTED_BY_OWN_READER`, a nie falszywy `PASS`.

## Live proof

1. Uzyto istniejacego adaptera Toolsetu; jedyna sesja otworzyla dokladnie
   `m2a_codex_aproof.mod` i obszar `m2a_caproof_area` na `\\.\DISPLAY1`.
   Nie bylo modalu Access Violation. Capture: `toolset-incaxje.png`, SHA-256
   `2f70d1965982d3275cd3f315556fb6a99bd5d78920f84a3e9099621f7cf07c6e`.
2. Aktualnie odczytane menu `Build` mialo `Test Module` pod pozycja `2`, ID
   `121`, `enabled=true`; dispatcher uzyty dopiero po tym readbacku.
3. Log klienta o `03:24:43` zawiera `Loading Module: m2a_codex_aproof`.
4. Capture poczatkowy gry (`nwn-incaxje-initial.png`, SHA-256
   `6e034a4b36a29b6cac57d4b5fd4b49a0e4c5dc0ede127b117a7d2dea1f1de0e8`)
   pokazuje start gracza bez fixture w kadrze. Po standardowym ruchu w kierunku
   pozycji fixture capture `nwn-incaxje-move-east.png`, SHA-256
   `fc789ca69a9e9846cbf6091c2dc1c4173349d0c20b69eeda788665de116de6e7`,
   nie pokazuje `incaxje`. Duza czarno-brazowa forma przy kamerze jest
   geometria obszaru/kadrowaniem, nie identyfikowalnym modelem testowym.

## Wynik i wniosek

> **Wycofane.** Ponizsza wstepna interpretacja zostala skorygowana po review
> screenshotow i nie jest wynikiem tego packetu. Nadrzedny wynik znajduje sie
> w sekcji `Korekta po wspolnym review screenshotow` oraz w A/B ASCII ponizej.

- **NWN:** tak, `incaxje` jest renderowany przez klienta gry. Nie jest jednak
  poprawnym humanoidem — rozmiar, transformacja lub skinning sa zle.
- **Aurora:** modul i obszar laduja bez bledu; widok z gory nie daje
  wystarczajaco czytelnej sylwetki do oznaczenia Toolsetu jako wizualnego PASS.
- **Nie wolno wyciagnac wniosku**, ze problem znika po konwersji Blendera.
  Zewnetrzny baseline dochodzi do tego samego rodzaju runtimeowej deformacji,
  wiec sama niewidocznosc poprzedniego binarnego writera nie jest jedynym
  problemem. Ten test nie izoluje jeszcze, czy przyczyna lezy w skonstruowanej
  hierarchii referencyjnej, mapie osi/rest pose, czy roznicy skin profilu
  CleanModels.

Brakuje MP4 ruchu i niezaleznej analizy roznicy wymaganych przez pelny runbook;
ten packet nie ma statusu `VERIFIED`. Jego zamknieta odpowiedz na zlecone
pytanie brzmi: **widoczny, ale niepoprawnie zdeformowany**.

## Korekta po wspolnym review screenshotow

Poprzednie zdanie koncowe bylo bledne i jest wycofane. Forma wskazana jako
model byla geometria obszaru; na zadnym z trzech screenshotow nie ma
jednoznacznie widocznego `incaxje`.

Aktualny, nadrzedny wynik packetu:

- Aurora: modul i obszar laduja, ale model nie przeszedl bramki wizualnej.
- NWN: modul laduje, ale model nie jest widoczny w proofie.
- Zewnetrzna konwersja Blender -> NeverBlender -> CleanModels nie rozwiazala
  problemu widocznosci.

## A/B: ASCII NeverBlender bez CleanModels (2026-07-18)

Ten wariant ma rozdzielic eksport NeverBlendera od binarnego payloadu
CleanModels. Wlasny harness dostal testy dla opisu provenance i byl uruchomiony
z dokladnie tym samym `incaxje`, wierszem `15101`, tekstura, MOD-em i
fixturem, ale zasob type `2002` w HAK-u zawiera bezposrednio
`incaxje.ascii.mdl`. Jest to test referencyjny parsera przy ladowaniu, nie
sciezka produktu: produktowy proof nadal wymaga native binary MDL.

| Artefakt | Wynik |
| --- | --- |
| HAK ASCII A/B | `package-ascii/m2a_codex_aproof.hak`, SHA-256 `79f19df7391810efb2cec8ff5d2d058c3882436badcfc3107f6b4059d4ee029c` |
| MOD | SHA-256 `9af06d6a3155a1043bd0343a23164d7ba88a9e30fef3e0685b780ae666fbb1cd` |
| HAK/MOD readback | `PASS`; w katalogach NWN potwierdzony identyczny hash po kopii |
| Toolset | dokladnie `m2a_codex_aproof.mod` i `m2a_caproof_area` na `\\.\DISPLAY1`; Area viewer otwarty przez walidowane `View Area` (ID `4`) |
| Kadr Area | `toolset-incaxje-ascii-area.png`, SHA-256 `c674ebe50d772e427cbd69f4cbf28bd84e4c0c2d82cce14d97a1d29bfabef6c8` |

Kadr Area nadal nie pokazuje jednoznacznej sylwetki `incaxje`; w miejscu
instancji widoczny jest co najwyzej nieidentyfikowalny drobny znak. Standardowy
adapter kamery wykonal kontrolowane zblizenie i zapisal klatki przed/po, ale
widok zatrzymal sie na kafelkach przy punkcie poczatku. Przed zapisaniem
koncowego JSON-a proces Toolsetu przestal byc obecny. Nie ma dowodu, czy byl to
crash, reczne zamkniecie, czy inna przyczyna, wiec przyczyny **nie przypisano**
modelowi ani adapterowi.

Zgodnie z gate'em runbooka **NWN nie zostal uruchomiony** dla wariantu ASCII.
Wynik A/B to `FAILED / MODEL-NOT-VISIBLE-IN-TOOLSET / INCONCLUSIVE-FOR-PARSER`:
nie udowadnia odrzucenia ASCII przez engine, lecz rowniez nie dostarcza
widocznosci modelu i nie rozstrzyga roznicy CleanModels vs. parser Aurory.

Klasyczny lokalny `nwnmdlcomp.exe` zostal ponownie sprawdzony zgodnie z jego
wlasnym usage (`-c`, nie `-i`). Bez zmian rejestru ani konfiguracji zwraca
`Unable to locate or open Neverwinter Night` i nie tworzy pliku wyjsciowego;
nie jest wiec dostepnym, poprawnie skonfigurowanym trzecim kompilatorem dla
tego testu.
