# Tworzenie przedmiotów: plan implementacji i kryteria zakończenia

Data: 2026-08-17\
Status: `IMPLEMENTACJA_W_TOKU / OFFLINE_CORE_WASM_WORKER_GOTOWE_CZESCIOWO`\
Zakres: produkcyjny target `ITEM` w `m2a-core`, WASM, Workerze i webowym
Studio, wraz z UTI, osobnymi modelami MDL partów, ikonami, HAK/MOD, readbackiem
i human-owned proofem Aurora Toolset/NWN.

## 1. Cel i werdykt startowy

Celem jest umożliwienie utworzenia natywnego przedmiotu Aurora/NWN z lokalnych
źródeł GLB i danych authoringu, bez zastępowania kontraktu Aurory własną
kategorią `weapon`, `shield` albo `potion`.

Autorytatywny przepływ pozostaje następujący:

```text
UTI.BaseItem
  -> rekord baseitems.2da
     -> ModelType: profil 0 / 1 / 2 / 3
     -> ItemClass: namespace modeli i ikon
     -> pozostałe pola: sloty, rozmiar ikony, gender/default policy
  -> recepta wymaganych partów
  -> osobny binary MDL każdego partu
  -> osobne warstwy ikony
  -> UTI wybierające numery wariantów
  -> HAK + testowy MOD + manifest/readback
```

Audytowany stan przed rozpoczęciem implementacji nie był produkcyjnym Item
pipeline'em:

- Studio i Worker nie mają targetu `ITEM` ani operacji `BUILD_ITEM_PACKAGE`;
- `BinaryCreatureWeaponItemV1` jest fixture'em testowym broni Creature, nie
  modelem domenowym Item;
- `model_part.rs` rozwiązuje materiały jednego przyszłego partu, ale nie składa
  przedmiotu;
- ogólny `package.rs` wymaga jednego MDL i `appearance.2da`, więc nie może być
  po cichu rozszerzony na pakiet Item;
- makieta Item jest dokumentacją UX, nie implementacją.

### 1.1 Checkpoint implementacji 2026-08-17

W tej samej linii implementacyjnej powstały:

- wersjonowany kontrakt `ItemAppearanceRecipeV1`, resolver dokładnego rekordu
  `baseitems.2da`, canonical JSON/SHA-256, profile `ModelType=0/1/2/3`, wymagane
  sloty oraz komplet stabilnych błędów;
- produkcyjny writer/reader UTI z profile-aware field setem, w tym wszystkimi
  19 polami `ArmorPart_*`, `Robe` i sześcioma kanałami kolorów;
- osobny profil binary MDL dla statycznego sztywnego partu Item: klasyfikacja
  `0`, natywne bounds, kontrolery translation/rotation/uniform scale,
  deterministyczna segmentacja i own semantic readback;
- writer warstw TGA związany z dokładnymi pikselami, SHA-256 i wymiarami
  `32 * InvSlotWidth` na `32 * InvSlotHeight` dla potwierdzonych profili;
- osobny compositor Item tworzący HAK, UTI i testowy MOD oraz typed readback
  IFO/ARE/GIT/GIC/UTC, powiązanie exact UTI ze slotem fixture'a i manifest;
- collision preflight `REQUIRE_ABSENT` oraz jeden wspólny limit 300 000
  trójkątów dla całego przedmiotu;
- granice WASM i operacje Workera dla inspekcji wejść, resolution recepty,
  kompilacji partu, ikon i pakietu, z jawnym artefaktem `ITEM_BLUEPRINT`;
- testy deterministyczności, stale identity, mutacji profilu/typu GFF,
  payload/report bindingu, namespace, limitu trójkątów oraz parity Core/WASM.

Implementacja nie osiągnęła jeszcze `ITEM_VS1_READY_FOR_OWNER_PROOF`. Otwarty
zakres obejmuje pełne Studio Item Authoring, browser Worker E2E, assembly
seam/gap/overlap, zamrożenie exact kandydata `ModelType=2`, instalację
MOD/HAK i handoff do human-owned proofu. Profil pancerza zapisuje i odczytuje
poprawny UTI; jego produkcyjny packaging pozostaje świadomie fail-closed do
czasu zamknięcia P-REF ikon i kompletnego armora.

| Etap | Stan checkpointu |
|---|---|
| `ITEM-0` | częściowo: kontrakty/testy gotowe, nie wszystkie P-REF zamknięte |
| `ITEM-1` | gotowy offline |
| `ITEM-2` | gotowy dla profili 0/1/2; armor packaging fail-closed |
| `ITEM-3` | gotowy od wspólnego `AuroraModelIrV1` do MDL/readbacku |
| `ITEM-4` | sloty i wspólny budżet gotowe; seam/gap/overlap otwarte |
| `ITEM-5` | gotowy offline dla profili 0/1/2/3 |
| `ITEM-6` | gotowy dla potwierdzonych profili 0/1/2; armor P-REF otwarty |
| `ITEM-7` | compositor i typed readback gotowe; exact kandydat niezamrożony |
| `ITEM-8` | API WASM/Worker gotowe; browser Worker E2E otwarte |
| `ITEM-9` | niezaimplementowany |
| `ITEM-10` | nieuruchomiony |
| `ITEM-11` | częściowo: core profili gotowy, packaging/proof armora otwarte |
| `ITEM-12` | częściowo: regresja targetowana zielona, pełne wydanie otwarte |

## 2. Źródła autorytetu i bramki wejścia

Implementacja musi być Aurora First i opierać się na:

1. `audyt-dekompilacji-aurora-item-model-parts-2026-07-29.md`;
2. `evidence/p-ref-item-wswls-parts-2026-07-29.md`;
3. własnych readerach GFF, 2DA, ERF/HAK i binary MDL;
4. read-only dekompilacji Aurory oraz retail/resource corpusie;
5. syntetycznych fixture'ach własnego projektu i env-gated P-REF bez kopiowania
   retailowych payloadów do repozytorium.

Lista brakującej wiedzy z audytu pozostaje bramką dla odpowiadających jej
ścieżek produktu; nie blokuje już ukończonych części resolvera i UTI:

- P-REF co najmniej jednego przedmiotu `ModelType=0`;
- P-REF co najmniej jednego przedmiotu `ModelType=1` wraz z sześcioma kolorami;
- P-REF co najmniej jednego pancerza `ModelType=3`, obejmujący 19 pól partów,
  kolory, naming i sposób rozwiązywania modeli;
- potwierdzenie naming/dimension/composition dla ikon profili 0/1/3;
- potwierdzenie dozwolonego zakresu numerów wariantów, w tym semantyki zera;
- decyzję, które pola zwykłego UTI są wymagane, opcjonalne albo dziedziczone
  per profil, zamiast kopiowania jednego retailowego blueprintu.

Checkpoint potwierdził osobno naming modeli pancerza w formacie
`p<gender><race><phenotype>_<capart-name><variant:03>`, przykładowo
`pmh0_chest001`. Nie rozstrzyga to semantyki jego ikon. Hipotezy z tego punktu
nie mogą być zaszyte jako stałe produkcyjne przed zamknięciem odpowiedniego
testu albo P-REF.

## 3. Zakres produktu i poziomy ukończenia

### 3.1 Vertical slice `ITEM-VS1`

Pierwszy pionowy przekrój ma celowo wąski zakres:

- istniejący rekord `baseitems.2da` o `ModelType=2`;
- trzy wymagane party `Bottom`, `Middle`, `Top`;
- statyczna sztywna geometria;
- `AURORA_CLASSIC_SAFE` i tekstury TGA;
- osobne dostarczone lub jawnie wygenerowane warstwy ikon TGA;
- production UTI, HAK modeli/tekstur/ikon i testowy MOD;
- pełna ścieżka Studio -> Worker -> WASM -> Core;
- exact handoff `ready_for_owner_proof` i wynik właściciela dla tego samego
  lineage.

`ITEM-VS1` nie pozwala ogłosić pełnego targetu Item jako ukończonego.

### 3.2 Pełny target `ITEM-V1`

Pełne ukończenie wymaga czterech formalnych profili:

| `ModelType` | Geometria | Kolory | Wymagany profil |
|---:|---|---:|---|
| 0 | `ModelPart1` | 0 | jeden part |
| 1 | `ModelPart1` | 6 | jeden part + Leather/Cloth/Metal |
| 2 | `ModelPart1..3` | 0 | Bottom/Middle/Top |
| 3 | 19 `ArmorPart_*` | 6 | pełny composer pancerza |

### 3.3 Poza zakresem `ITEM-V1`

- tworzenie nowych gameplayowych rekordów `baseitems.2da`;
- edycja właściwości magicznych i skryptów jako pełny Toolset replacement;
- kopiowanie modeli, ikon albo UTI z retail/CEP;
- DDS jako wymagany format wyjściowy; TGA pozostaje formatem podstawowym;
- automatyczne malowanie ikon bez jawnego i przetestowanego źródła obrazu;
- agent-run Toolset/NWN proof przy obowiązującej decyzji human-owned proof;
- bezwarunkowe nadpisywanie istniejących wariantów w retail, HAK lub module.

Te elementy mogą otrzymać późniejsze wersjonowane rozszerzenia, ale nie mogą
rozmywać Definition of Done `ITEM-V1`.

## 4. Docelowa architektura

### 4.1 Core

Rekomendowane nowe moduły odpowiedzialności:

```text
item_baseitems.rs
  schema-aware readback istniejącego baseitems.2da
  ItemBaseRecordV1 i ItemCompositionProfileV1

item_recipe.rs
  ItemAppearanceRecipeV1
  ItemPartRecipeV1, ItemColorRecipeV1, ItemIconRecipeV1
  wersjonowanie, canonical JSON/hash i walidacja

item_namespace.rs
  resolver resrefów MDL i ikon
  EffectiveResourceNamespaceV1
  collision report i jawna collision scope

item_part.rs
  adapter materiałów/tekstur
  transform bake do kontrolerów MDL
  writer i semantic readback każdego partu

item_assembly.rs
  wymagane sloty profilu
  composed preview/readback model
  seam/gap/overlap i wspólny triangle budget

item_uti.rs
  profilowy writer GFF UTI
  exact typed semantic readback

item_icon.rs
  naming, rozmiar, kolejność warstw i TGA readback

item_package.rs
  wiele MDL + tekstury + ikony w HAK
  UTI + fixture w MOD
  manifest, hashes, provenance i readback całego grafu zasobów
```

Nie należy osłabiać kontraktu `write_model_package_v1`, który poprawnie
obsługuje inny profil jednego modelu. Item otrzymuje osobny compositor.

### 4.2 Kontrakt danych

Minimalny kontrakt domenowy:

```yaml
itemAppearanceRecipeV1:
  schemaVersion: 1
  baseItemsSourceSha256: "..."
  baseItem: 1
  resolved:
    modelType: 2
    itemClass: WSwLs
    invSlotWidth: 1
    invSlotHeight: 3
  identity:
    utiResref: m2a_item_001
    tag: M2A_ITEM_001
    displayName: Meshy2Aurora Longsword
  parts:
    - slot: BOTTOM
      variant: 251
      sourcePartId: bottom
      transform: { translation: [0, 0, 0], rotation: [0, 0, 0, 1], scale: 1 }
    - slot: MIDDLE
      variant: 252
      sourcePartId: middle
      transform: { translation: [0, 0, 0], rotation: [0, 0, 0, 1], scale: 1 }
    - slot: TOP
      variant: 253
      sourcePartId: top
      transform: { translation: [0, 0, 0], rotation: [0, 0, 0, 1], scale: 1 }
  icon:
    mode: SEPARATE_AURORA_LAYERS
  collisionPolicy: REQUIRE_ABSENT
```

Wartości wariantów w przykładzie są demonstracyjne. Nie są rezerwacją i nie
mogą zostać użyte bez sprawdzenia namespace.

### 4.3 Granice odpowiedzialności

- `baseitems.2da` wybiera profil i namespace, ale nie zawiera transformacji;
- recepta Studio przechowuje źródła i authoring transforms;
- transformacje są wypalane do kontrolerów odpowiedniego MDL;
- UTI zawiera wyłącznie pola odpowiednie dla resolved profilu;
- złożony preview nie jest dodatkowym modelem runtime;
- ikony są osobnymi obrazami, nie screenshotem viewportu 3D;
- HAK zawiera wygenerowane modele, tekstury i warstwy ikon;
- testowy MOD zawiera exact UTI, fixture wyposażenia i Area;
- manifest wiąże source, recipe, baseitems, namespace, MDL, TGA, UTI, HAK i MOD.

## 5. Plan implementacji

Każdy etap stosuje TDD: najpierw kontrakt i failing test, potem minimalna
implementacja, następnie refactor i own readback.

### ITEM-0 — kontrakt, corpus i stabilne błędy

1. Zamrozić schematy `ItemAppearanceRecipeV1`, `ItemBuildReportV1`,
   `ItemPackageManifestV1` i `EffectiveResourceNamespaceV1`.
2. Zdefiniować enumy profili i slotów bez osi `Weapon | Shield`.
3. Dodać syntetyczne minimalne fixture'y dla profili 0/1/2/3.
4. Domknąć P-REF wskazane w sekcji 2.
5. Zdefiniować stabilne kody błędów i ścieżki diagnostyczne.

Warunek wyjścia: kontrakt odróżnia fakty Aurora First od hipotez, a testy
schematów, wersji i błędnych profili są zielone.

### ITEM-1 — schema-aware resolver `baseitems.2da`

1. Użyć wspólnego parsera 2DA bez tworzenia drugiego parsera tekstowego.
2. Odczytywać wybrany fizyczny wiersz i wymagane kolumny, między innymi
   `ItemClass`, `ModelType`, `GenderSpecific`, `DefaultModel`, `DefaultIcon`,
   `InvSlotWidth` i `InvSlotHeight` oraz potwierdzone sloty.
3. Fail closed dla brakującej kolumny, null wymaganej wartości, nieznanego
   `ModelType`, błędnej liczby, niejednoznacznej kolumny i stale source hash.
4. Zwracać canonical resolved record wraz z SHA-256 źródłowego 2DA.

Warunek wyjścia: każdy profil ma test pozytywny i pełną macierz negatywną;
resolver nie zgaduje wartości domyślnych.

### ITEM-2 — recepta, naming i collision preflight

1. Wyprowadzać wymagane sloty wyłącznie z resolved `ModelType`.
2. Generować resrefy MDL bez prefiksu `i` oraz resrefy ikon z prefiksem `i`.
3. Walidować limit 16 znaków po pełnym formatowaniu, nie tylko `ItemClass`.
4. Zbudować `EffectiveResourceNamespaceV1` z jawnie wskazanych indeksów
   retail/KEY, HAK, MOD i bieżącego outputu.
5. Blokować autoalokację, gdy collision scope jest niepełny.
6. V1 używa `REQUIRE_ABSENT`; replacement/override jest poza zakresem.

Warunek wyjścia: exact naming dla wszystkich profili przechodzi P-REF, a
kolizje wewnętrzne, case-insensitive i zewnętrzne są deterministycznie
blokowane przed generacją payloadów.

### ITEM-3 — kompilator pojedynczego partu

1. Rozszerzyć neutralny `ModelPart` o jawny `ItemPartCompileRequestV1` bez
   importowania reguł UTI do wspólnego resolvera materiałów.
2. Użyć istniejącego GLB -> IR, Material Separation, Texture Authoring i binary
   MDL writera.
3. Wypalać translation/rotation/uniform scale do kontrolerów node'a MDL.
4. Zachować geometrię, UV, normalne, tangenty, materiały i hierarchy.
5. Segmentować każdy mesh stream powyżej 21 845 trójkątów bez utraty danych.
6. Wykonać own semantic readback i związać go z part recipe hash.

Warunek wyjścia: source -> MDL -> own readback zachowuje wszystkie wymagane
invarianty, a transform po readbacku jest równy resolved transformowi.

### ITEM-4 — composer partów i wspólne bramki geometrii

1. Zweryfikować dokładnie wymagany zestaw slotów i brak duplikatów.
2. Złożyć offline readback partów w jednym wspólnym item space.
3. Obliczać sumę renderowalnych trójkątów wszystkich partów.
4. Akceptować dokładnie 300 000 i blokować 300 001.
5. Raportować seam/gap/overlap dla wymaganych połączeń, z progami
   zamrożonymi z corpusu, nie dobranymi pod jeden miecz.
6. Rozdzielić status geometrii od późniejszego visual proofu.

Warunek wyjścia: żaden part nie przechodzi do packagingu bez pełnego assembly
reportu, a budżet nie może zostać pomnożony przez liczbę partów.

### ITEM-5 — produkcyjny writer i reader UTI

1. Utworzyć `item_uti.rs`; nie promować proofowego buildera Creature do API
   produkcyjnego.
2. Emitować pola partów i kolorów wynikające z profilu.
3. Walidować zgodność `TemplateResRef`, `BaseItem`, nazwy, tagu, stosu,
   identyfikacji, kosztu i potwierdzonego minimalnego envelope UTI.
4. Odrzucać pola z innego profilu, brak pola wymaganego i błędny typ GFF.
5. Reader ma odtworzyć typed semantic recipe, a nie jedynie listę etykiet.
6. Dwa identyczne requesty muszą dawać byte-identical UTI.

Warunek wyjścia: writer/readback przechodzi profile 0/1/2/3, mutacje każdego
ważnego pola oraz testy zgodności resource key <-> `TemplateResRef`.

### ITEM-6 — warstwy ikon

1. Przyjmować jawne źródło obrazu dla każdej wymaganej warstwy.
2. Walidować wymiary wynikające z `InvSlotWidth/InvSlotHeight` i kontraktu
   32-pikselowego canvasu.
3. Generować oddzielne TGA pod dokładnymi resrefami profilu.
4. Zachować określoną kolejność warstw, alpha i orientation.
5. Udostępnić złożony preview tylko jako podgląd; nie emitować go jako zamiennik
   wymaganych warstw.
6. Reader sprawdza każdy TGA, jego hash, wymiary i binding do recepty.

Warunek wyjścia: golden pixel tests i P-REF potwierdzają naming, wymiary,
kolejność oraz alpha; brak warstwy blokuje build zamiast tworzyć pustą ikonę.

### ITEM-7 — compositor HAK/MOD i pełny readback

1. Dodać osobny `write_item_package_v1` obsługujący wiele modeli.
2. HAK ma zawierać dokładnie manifestowane MDL, TGA i ewentualne potwierdzone
   zasoby materiałowe; bez `appearance.2da` wymuszonego przez inny profil.
3. Wygenerować standalone UTI oraz testowy MOD z tym exact UTI.
4. Testowy UTC/GIT ma wskazywać ten sam UTI w potwierdzonym slocie.
5. Readback ma przejść cały graf: recipe -> nazwy -> HAK resources -> MDL/TGA
   -> UTI -> UTC/GIT -> MOD/HAK attachment.
6. Manifest ma zawierać długości, SHA-256, role, resource types, resrefy,
   source/recipe/baseitems/namespace hashes i generator identity.

Warunek wyjścia: mutacja dowolnego payloadu, raportu, bindingu lub hasha jest
wykrywana; pakiet powstaje w jednej deterministycznej linii bez repacku.

### ITEM-8 — WASM i Worker

1. Dodać publiczne API inspekcji `baseitems.2da`, inspekcji partów, resolution,
   builda i readbacku Item.
2. Zapewnić parity Core/WASM dla JSON, błędów i bajtów artefaktów.
3. Dodać requesty Workera `INSPECT_ITEM_INPUTS`, `RESOLVE_ITEM_RECIPE` i
   `BUILD_ITEM_PACKAGE`.
4. Rozszerzyć `WorkerArtifact` o jawny `ITEM_BLUEPRINT`/UTI albo równoważną
   jednoznaczną rolę, bez maskowania UTI jako modelu.
5. Wszystkie ciężkie operacje wykonywać poza głównym wątkiem UI.
6. Zachować revision/source/recipe identity i odrzucać stale odpowiedzi.

Warunek wyjścia: Node/WASM i real browser Worker tworzą byte-identical pakiet,
a anulowanie lub zmiana recepty nie może opublikować starego wyniku.

### ITEM-9 — Studio Item Authoring

1. Dodać `ITEM` do `StudioTarget` i osobny stan sesji.
2. Ekran Source przyjmuje `baseitems.2da`, wybór `BaseItem`, źródła partów,
   źródła ikon i namespace inventories.
3. UI pokazuje resolved `ItemClass`, `ModelType`, wymagane sloty, wymiary ikony
   i collision scope.
4. Prepare Item oferuje per-part transform, Material Separation, tekstury,
   widoki Composed/Exploded/Icon i diagnostykę seamów.
5. Review Output pokazuje readback każdego MDL/TGA, UTI, HAK, MOD i manifestu.
6. Build pozostaje zablokowany dla incomplete scope, kolizji, przekroczonego
   budżetu, stale recipe, braku ikony albo błędu readbacku.
7. Source GLB i wejściowe 2DA nigdy nie są nadpisywane.

Warunek wyjścia: dostępny jest pełny keyboard-accessible workflow, reset/zmiana
targetu usuwa stan Item, a test real Chrome przechodzi Source -> Build ->
Review -> Download bez błędów konsoli.

### ITEM-10 — vertical slice `ModelType=2`

1. Wykonać cały pipeline na jednym zatwierdzonym trzyczęściowym przedmiocie.
2. Użyć istniejącego `BaseItem` i trzech własnych źródeł partów/ikon.
3. Zamrozić exact UTI, HAK, MOD, wszystkie MDL/TGA, manifest i readback.
4. Zainstalować exact MOD/HAK do natywnych katalogów NWN zgodnie z regułą
   absent-target lub reuse tylko przy identycznym SHA-256; kolizja o innym
   hashu jest fail-closed.
5. Handoff musi zaczynać się od nazwy pliku MOD, potem nazwy modułu w Toolsecie
   i dokładnej nazwy Area; dalej podaje obiekt, UTI, BaseItem, HAK, party,
   warianty, placement i wszystkie hashe.
6. Agent zatrzymuje się na `ready_for_owner_proof`; nie uruchamia ani nie
   kontroluje Toolset/NWN.
7. Właściciel testuje ten sam lineage w Toolsecie i NWN.

Warunek wyjścia technicznego: `ITEM_VS1_READY_FOR_OWNER_PROOF`.\
Warunek zamknięcia vertical slice: właściciel potwierdza exact model, ikonę,
wyposażenie i widoczność w Toolsecie oraz NWN.

### ITEM-11 — profile 0, 1 i 3

1. Dodać profil jednopartowy bez kolorów.
2. Dodać profil jednopartowy z sześcioma kanałami kolorów.
3. Dodać 19-part armor composer z pełnym typed UTI/readbackiem.
4. Dodać gender/default policy wyłącznie według potwierdzonego kontraktu.
5. Dla każdego profilu wykonać osobny representative package i owner proof,
   zachowując bramkę iteracji modelu.

Warunek wyjścia: jeden sukces `ModelType=2` nie maskuje braków pozostałych
profili; każdy profil ma własny test, manifest i wynik właściciela.

### ITEM-12 — hardening i wydanie

1. Uruchomić pełne testy workspace, fmt, clippy, WASM, Studio i browser E2E.
2. Dodać testy limitów pamięci, rozmiaru wejścia i liczby partów/zasobów.
3. Potwierdzić deterministyczność na dwóch czystych buildach.
4. Przeprowadzić niezależny review bezpieczeństwa formatów i readbacku.
5. Zaktualizować runbook użytkownika, indeks dokumentacji i stan funkcji.
6. Nie oznaczać `ITEM-V1 COMPLETE`, dopóki wszystkie kryteria sekcji 6 nie są
   spełnione.

## 6. Kryteria zakończenia

Wszystkie kryteria są kumulatywne. Udany build, własny readback albo sam
preview webowy nie zamyka funkcji.

### 6.1 Kontrakt i zgodność z Aurorą

- `BaseItem` istnieje w exact zahashowanym `baseitems.2da`.
- `ModelType`, `ItemClass`, rozmiar ikony i pozostałe resolved pola pochodzą z
  tego samego rekordu.
- Profile 0/1/2/3 mają odrębne wersjonowane recepty slotów i pól UTI.
- Nie istnieje produktowy enum `Weapon | Shield` sterujący składaniem modelu.
- Każda reguła naming/field/layout ma wskazany fakt z dekompilacji, P-REF albo
  jawny wniosek implementacyjny z testem.
- Żadna niezamknięta hipoteza nie jest traktowana jako sukces.

### 6.2 Recepta i deterministyczność

- Recipe ma schema version, canonical JSON i SHA-256.
- Source GLB, 2DA, obrazy ikon i namespace inventories mają zapisane SHA-256.
- Identyczne wejścia, recepta, namespace i generator dają byte-identical MDL,
  TGA, UTI, HAK, MOD oraz identyczne raporty.
- Zmiana partu, transformu, materiału, ikony, `BaseItem`, wariantu albo wersji
  generatora zmienia result identity.
- Stale recipe/source identity jest blokowana przed buildem.

### 6.3 Geometria i transformacje

- Każdy wymagany slot występuje dokładnie raz; slot z innego profilu jest
  odrzucany.
- Każdy MDL przechodzi binary writer i semantic readback.
- Transform per part po readbacku jest zgodny z resolved recipe.
- Trójkąty, UV, normalne, tangenty i material bindings są zachowane.
- Jeden mesh stream respektuje 65 535 indeksów / 21 845 trójkątów; większy jest
  deterministycznie segmentowany bez kasowania geometrii.
- Suma całego przedmiotu akceptuje 300 000 i blokuje 300 001.
- Composed report nie wykazuje blokującego gapu, seam mismatch ani
  niezamierzonego overlapu według wersjonowanego profilu jakości.

### 6.4 Naming i namespace

- MDL używa nazwy bez prefiksu `i`; ikona używa dokładnego prefiksu `i`.
- Pełny resref ma 1..16 dozwolonych znaków po formatowaniu.
- Klucze są unikalne case-insensitive w HAK, MOD i całym dostarczonym
  `EffectiveResourceNamespaceV1`.
- Raport podaje dokładny collision scope; incomplete scope blokuje autoalokację.
- `ITEM-V1` nie nadpisuje kolizji i nie przydziela po cichu innego wariantu.

### 6.5 UTI

- GFF ma exact typ `UTI ` i root struct ID `0xffffffff`.
- `TemplateResRef` jest zgodny z archive/resource key.
- `BaseItem` i pola wyglądu odpowiadają resolved profilowi.
- Profile z kolorami zachowują wszystkie sześć kanałów.
- Armor zachowuje dokładnie potwierdzone 19 pól i ich semantykę.
- Reader sprawdza wartości oraz typy, nie tylko obecność etykiet.
- UTI jest tym samym zasobem, na który wskazuje testowy UTC/GIT.

### 6.6 Ikony

- Każda wymagana warstwa ma poprawny resref, TGA type, hash i wymiary.
- Wymiary odpowiadają `InvSlotWidth/InvSlotHeight` i potwierdzonemu canvasowi.
- Kolejność, orientacja i alpha przechodzą golden pixel testy.
- Preview złożony z warstw odpowiada temu samemu zestawowi, który trafia do HAK.
- Brak lub błędna warstwa blokuje build.

### 6.7 Pakiet i readback

- HAK zawiera tylko manifestowane i dozwolone zasoby Item.
- MOD wskazuje exact HAK i zawiera exact UTI oraz testowy fixture.
- Manifest wiąże wszystkie wejścia i wyniki przez długości, role, typy, resrefy
  i SHA-256.
- Full readback przechodzi graf recipe -> HAK/MOD -> MDL/TGA/UTI -> fixture.
- Mutacja każdego zasobu, bindingu, raportu lub hasha daje kontrolowany błąd.
- Nie ma drugiego repacku ani rozjazdu między raportem a pobieranymi bajtami.

### 6.8 WASM, Worker i Studio

- Core i WASM mają identyczne kody błędów, raporty i byte-identical artefakty.
- Worker nie wykonuje ciężkiej konwersji na głównym wątku.
- `ITEM` działa jako pełnoprawny target sesji i poprawnie resetuje stan.
- UI pokazuje resolved profil, party, kolizje, sumaryczny budżet i readback.
- Build jest fail-closed dla każdego kryterium blokującego.
- Real Chrome E2E przechodzi import wielu partów, authoring, build, review i
  download bez błędów oraz stale-response race.
- UI jest obsługiwalne klawiaturą i ma czytelne etykiety diagnostyczne.

### 6.9 Testy

Minimalna macierz obejmuje:

- profile 0/1/2/3 i nieznany `ModelType`;
- brak/null/duplikat kolumny `baseitems.2da`;
- brak, nadmiar i duplikat slotu;
- błędny wariant według potwierdzonej polityki;
- resref 16 znaków, 17 znaków i kolizje case-insensitive;
- kolizję z retail, HAK, MOD i bieżącym outputem;
- 300 000 i 300 001 trójkątów sumarycznie;
- granicę per-stream oraz segmentację;
- transformy identity, translation, rotation i uniform scale;
- brak/nonfinite/nonuniform transform;
- ikony o poprawnych i błędnych wymiarach, alpha oraz orientation;
- UTI field-type mutation dla każdego profilu;
- HAK/MOD/resource/hash mutations;
- native/WASM parity i browser Worker E2E;
- dwa czyste buildy potwierdzające deterministyczność.

Wymagane są testy syntetyczne w CI oraz env-gated P-REF wykonywane in-place,
bez kopiowania retailowych payloadów do repo.

### 6.10 Human-owned proof

- Dla każdego representative profilu istnieje jeden exact, zamrożony lineage.
- Exact MOD/HAK są zainstalowane z absent-target/hash-before/hash-after gate;
  kolizja o innym hashu zatrzymuje lane.
- Handoff zaczyna się od: plik `.mod`, nazwa modułu w Toolsecie, nazwa Area.
- Handoff podaje również obiekt, UTI, `BaseItem`, HAK, slot wyposażenia,
  placement, warianty i wszystkie hashe.
- Agent nie uruchamia ani nie kontroluje sesji Toolset/NWN i kończy na
  `ready_for_owner_proof`.
- Właściciel potwierdza osobno Toolset i NWN dla tego samego lineage:
  model, właściwe złożenie partów, skalę/orientację, wyposażenie, ikonę i brak
  brakujących elementów.
- Wynik zapisuje osobno `modelVisibility` oraz `proofCompleteness`.

## 7. Statusy ukończenia

### `ITEM_VS1_READY_FOR_OWNER_PROOF`

Wszystkie offline, Core/WASM/Worker/Studio, packaging i installation gates dla
jednego exact `ModelType=2` są zielone. Nie jest to jeszcze visual success.

### `ITEM_VS1_PROVEN`

Właściciel potwierdził ten sam `ModelType=2` lineage w Toolsecie i NWN,
łącznie z modelem, wyposażeniem i ikoną.

### `ITEM_V1_READY_FOR_OWNER_PROOF`

Profile 0/1/2/3 są zaimplementowane i mają exact offline-verified pakiety,
readback oraz zainstalowane handoffy. Agent nie deklaruje visual success.

### `ITEM_V1_COMPLETE`

Wszystkie kryteria sekcji 6 są zielone, a właściciel potwierdził representative
lineage każdego profilu w Aurora Toolset i NWN. Dopiero ten status oznacza
pełny produkcyjny target `ITEM`.

## 8. Priorytety

### P0 — wymagane dla pierwszego działającego przedmiotu

- ITEM-0 do ITEM-10;
- `ModelType=2`, trzy party, UTI, ikony, HAK/MOD;
- pełny readback, summed triangle gate, collision preflight;
- Studio/Worker/WASM i owner proof.

### P1 — wymagane dla pełnego `ITEM-V1`

- profile 0, 1 i 3;
- kolory, gender/default policy i armor composer;
- representative owner proof każdego profilu;
- hardening i dokumentacja użytkownika.

### P2 — późniejsze rozszerzenia

- tworzenie własnych rekordów `baseitems.2da`;
- kontrolowany tryb replacement/override;
- rozszerzone właściwości UTI i gameplay scripting;
- DDS lub dodatkowe profile materiałowe po osobnym Aurora First proofie;
- automatyczne generowanie artystycznych ikon.

## 9. Definition of Done w jednym zdaniu

Target `ITEM-V1` jest gotowy dopiero wtedy, gdy Studio przez Worker/WASM/Core
potrafi deterministycznie utworzyć z lokalnych źródeł wszystkie cztery profile
`baseitems.2da.ModelType`, wygenerować i samodzielnie odczytać zgodne MDL, TGA,
UTI, HAK i MOD z pełną kontrolą namespace oraz wspólnym limitem 300 000
trójkątów, a właściciel potwierdzi exact representative każdego profilu w
Aurora Toolset i NWN.

## 10. Następny bezpieczny krok implementacyjny

Następnym krokiem jest `ITEM-9` dla wąskiego `ITEM-VS1` (`ModelType=2`):

1. dodać osobny stan sesji `ITEM`, import trzech źródeł partów, exact
   `baseitems.2da`, warstwy ikon i kompletnego namespace;
2. połączyć istniejące operacje Workera bez duplikowania kontraktów w React;
3. dodać stale-response gates oraz real Chrome Source -> Build -> Review ->
   Download;
4. dopiero po zielonym offline E2E zamrozić jeden exact kandydat, zbudować i
   zainstalować jego MOD/HAK zgodnie z absent/hash gate oraz przekazać go
   właścicielowi;
5. równolegle nie odblokowywać packagingu `ModelType=3`, dopóki P-REF nie
   zamknie ikon i pełnego armora.

UI pozostaje konsumentem zweryfikowanego kontraktu Core/WASM, a nie miejscem,
w którym reguły Aurory są zgadywane.
