# Audyt dekompilacji Aurora: modele przedmiotów składane z partów

Data: 2026-07-29
Status: ZAMKNIĘTY AUDYT OFFLINE, BEZ LIVE TOOLSET/NWN
Zakres: rozróżnianie profili itemów, `ModelPart*`, nazwy zasobów, składanie
MDL, składanie ikon, miejsce definicji transformacji i kontrakt dla
Meshy2Aurora.

## 1. Werdykt

Aurora nie rozróżnia sposobu budowy modelu przez produktowe kategorie
`weapon`, `shield`, `potion` itd. Autorytatywnym dyskryminatorem jest
`ModelType` z rekordu `baseitems.2da`, wybranego przez `UTI.BaseItem`.

Dla interesującego nas profilu trzyczęściowego:

| Warstwa | Wartość |
|---|---|
| `baseitems.2da.ModelType` | `2` |
| pola UTI | `ModelPart1`, `ModelPart2`, `ModelPart3` |
| znaczenie pól | `Bottom`, `Middle`, `Top` |
| skróty nazw MDL | `b`, `m`, `t` |
| nazwy MDL | `<ItemClass>_b_<NNN>`, `<ItemClass>_m_<NNN>`, `<ItemClass>_t_<NNN>` |
| nazwy warstw ikon | `i<ItemClass>_b_<NNN>`, `i<ItemClass>_m_<NNN>`, `i<ItemClass>_t_<NNN>` |
| miejsce transformacji partu | kontrolery w jego MDL, nie UTI |

`B` znaczy **Bottom**, nie `Blade`. `M` znaczy **Middle**, a `T` znaczy
**Top**.

Najważniejsza korekta względem sekcji 15 starego
`C:\Projects\New Folder\AUDYT_ITEMY.md`:

- helper z prefiksem `i` tworzy nazwy warstw ikon TGA/DDS;
- helper bez `i` tworzy nazwy modeli MDL;
- `FUN_0064af98` jest kompozytorem obrazu, nie funkcją appendującą MDL;
- właściwa ścieżka modelu 3D w `TdlgItemEdit` używa helpera bez `i`, sprawdza
  zasób typu `2002` i appenduje Middle oraz Top do modelu Bottom.

Poprzednie sformułowanie o "`i` używanym dla ikon/modeli itemowych" i
appendowaniu przez `FUN_0064af98` zostaje niniejszym zastąpione.

## 2. Metoda i granice

Audyt był wykonany wyłącznie read-only:

1. ponowny pass po dekompilacji Toolsetu;
2. sprawdzenie UI stringów;
3. sprawdzenie retailowego `baseitems.2da` i `nwscript.nss`;
4. odczyt istniejącego retailowego UTI;
5. scan `nwn_base.key` i wskazanych zakresów BIF bez wypakowywania payloadów;
6. odczyt trzech dokładnych MDL własnym readerem `m2a-core`.

Nie uruchamiano ani nie kontrolowano Aurora Toolset lub NWN. Nie zmieniono
instalacji gry. Nie skopiowano retailowych UTI, MDL ani tekstur do repo.

### 2.1 Tożsamość głównych wejść

| Źródło logiczne | SHA-256 |
|---|---|
| `Aurora-decomp/export/decompiled_all.c` | `36bb8b1031afe2abf23f0e18180a5ad649401d9ea5e078e5170a31a96167c572` |
| `Aurora-decomp/export/strings.tsv` | `864ae0139451d5333307cb2bbccce725bbd2d2c6056ac2d5f80f9b5ea50f190f` |
| retail `baseitems.2da` | `1c1a56c8c3a6cce1728befa4638241174b373ce03618e909b1e4f990e9a1bf46` |
| retail `nwscript.nss` | `c14098d0181f921618622f379ff5cb682b8656d5293f218392531eef7a8478ad` |
| retail `nw_wswmls002.uti` | `b52257f4441e9edeb6018f57362f51ca018fd470129147a7fcfa626de55c9ac6` |
| retail `nwn_base.key` | `09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935` |

Dokładne hashe trzech odczytanych MDL oraz skrót raportów readera są w
[P-REF dla partów `WSwLs`](evidence/p-ref-item-wswls-parts-2026-07-29.md).

## 3. Łańcuch autorytetu

```text
UTI.BaseItem
  -> wiersz baseitems.2da
     -> ModelType określa schemat partów
     -> ItemClass określa rdzeń nazw zasobów
     -> GenderSpecific modyfikuje nazwy profili jednoczęściowych
  -> UTI.ModelPart* / ArmorPart_* wybierają numery wariantów
  -> resolver tworzy logiczne resrefy
  -> Resource Manager ładuje MDL albo warstwy ikon
```

Nie ma jednej dodatkowej definicji typu `item_assembly.json`, która zawiera
offsety partów. UTI wybiera numery. `baseitems.2da` wybiera receptę i
namespace. Geometria, pivot i transformacja są zapisane w samych MDL.

## 4. Jak Aurora rozróżnia profile składania

`FUN_00604d70` w `decompiled_all.c:299891-299913` mapuje `ModelType` na liczbę
wartości modelowych i liczbę kanałów kolorów:

| `ModelType` | Pola modelu | Kanały kolorów | Profil danych |
|---:|---:|---:|---|
| `0` | 1 | 0 | `ModelPart1` |
| `1` | 1 | 6 | `ModelPart1` + Leather/Cloth/Metal |
| `2` | 3 | 0 | `ModelPart1..3` = Bottom/Middle/Top |
| `3` | 19 | 6 | `ArmorPart_*` + Leather/Cloth/Metal |

Odczyt UTI w `decompiled_all.c:300168-300255` najpierw pobiera wiersz
`baseitems.2da`, ustawia profil, a potem czyta dokładnie odpowiedni zestaw pól.
Zapis robi symetryczną operację w `decompiled_all.c:300324-300403`.

### 4.1 Dlaczego `weapon / shield` jest złą osią UI

Próbki z retailowego `baseitems.2da`:

| Row | Label | `ModelType` | `ItemClass` |
|---:|---|---:|---|
| 1 | `longsword` | 2 | `WSwLs` |
| 14 | `smallshield` | 0 | `AShSw` |
| 49 | `potions` | 2 | `it_potion` |
| 56 | `largeshield` | 0 | `AShLw` |
| 57 | `towershield` | 0 | `AShTo` |
| 101 | `empty_potion` | 2 | `it_potion` |
| 104 | `crafted_potion` | 2 | `it_potion` |

Legacy NWScript nazywa typ operacji `ITEM_APPR_TYPE_WEAPON_MODEL`, ale
`baseitems.2da` stosuje trzyczęściowy `ModelType=2` również do mikstur. Z kolei
tarcze retailowe są jednoczęściowym `ModelType=0`.

Wniosek: produkt ma pytać o **receptę partów zgodną z Aurorą**, nie o
marketingową kategorię przedmiotu.

## 5. Dokładne mapowanie `ModelPart1..3`

Retailowy `nwscript.nss:611-617` definiuje:

```text
ITEM_APPR_WEAPON_MODEL_BOTTOM = 0
ITEM_APPR_WEAPON_MODEL_MIDDLE = 1
ITEM_APPR_WEAPON_MODEL_TOP    = 2
```

W dekompilacji ścieżka modelu 3D przekazuje odpowiednio:

| Indeks resolvera | Pole UTI | Slot | Litera |
|---:|---|---|---|
| 0 | `ModelPart1` | Bottom | `b` |
| 1 | `ModelPart2` | Middle | `m` |
| 2 | `ModelPart3` | Top | `t` |

Mapowanie nie jest hipotezą wynikającą z kształtu miecza. Potwierdzają je
jednocześnie:

- stałe publicznego API;
- nazwy kontrolek Toolsetu;
- kolejność pól UTI w dekompilacji;
- kolejność wywołań resolvera;
- istniejące retailowe zasoby MDL.

UI Toolsetu zawiera `Middle`, `Bottom`, `Model`, `Color` oraz handlery
`cbWeaponMiddleChange`, `cbWeaponBottomChange`, `cbWeaponTopChange` i trzy
odpowiedniki kolorów (`strings.tsv:26204-26215`).

## 6. Resolver nazw zasobów

### 6.1 Modele 3D

`FUN_00504004` (`decompiled_all.c:169157-169172`) tworzy resref bez prefiksu
`i`:

```text
%s_%c_%03d
%s_%03d
```

Dla `ModelType=2` segment `%c` jest zawsze obecny. `%s` pochodzi z
`baseitems.ItemClass`, a liczba jest dopełniana do trzech cyfr.

Przykład dla `BaseItem=1`, `ItemClass=WSwLs`, pól `23/63/23`:

```text
wswls_b_023
wswls_m_063
wswls_t_023
```

### 6.2 Warstwy ikon

`FUN_00503c74` (`decompiled_all.c:169023-169039`) ma analogiczne formaty z
prefiksem `i`:

```text
i%s_%c_%03d
i%s_%03d
```

Daje to:

```text
iwswls_b_023
iwswls_m_063
iwswls_t_023
```

`nwn_base.key` rozstrzyga typ zasobów:

| Resref | Type 3 (TGA) | Type 2033 (DDS) | Type 2002 (MDL) |
|---|---:|---:|---:|
| `iwswls_b_023` | tak | tak | nie |
| `iwswls_m_063` | tak | tak | nie |
| `iwswls_t_023` | tak | tak | nie |
| `wswls_b_023` | nie | nie | tak |
| `wswls_m_063` | nie | nie | tak |
| `wswls_t_023` | nie | nie | tak |

To jest dowód binarny, że prefiksu `i` nie wolno używać dla MDL.

## 7. Jak Toolset składa model 3D

Ścieżka `TdlgItemEdit` zaczynająca się w okolicy
`decompiled_all.c:128061`:

1. pobiera rekord `baseitems.2da` według `BaseItem`;
2. dla `ModelType=2` generuje Bottom przez `FUN_00504004`;
3. sprawdza istnienie zasobu typu `0x07d2`, czyli `2002`/MDL
   (`decompiled_all.c:128110-128115`);
4. ładuje Bottom jako model bazowy;
5. generuje Middle z indeksem 1 i appenduje go wywołaniem vtable `+0xb0`
   (`decompiled_all.c:128180-128207`);
6. generuje Top z indeksem 2 i appenduje go tym samym przejściem
   (`decompiled_all.c:128208-128234`).

Jeżeli Bottom nie istnieje, Toolset bierze `DefaultModel` z rekordu base itemu
(`decompiled_all.c:128115-128151`). W tej gałęzi nie buduje normalnego zestawu
Bottom/Middle/Top.

Nie znaleziono odczytu żadnych pól UTI typu:

- `PartOffset`;
- `PartRotation`;
- `PartScale`;
- `Socket`;
- `Seam`;
- `Pivot`.

Append jest zasobowy: ładuje kolejne MDL-e. Dopasowanie przestrzenne musi być
więc gotowe w modelach.

## 8. Gdzie naprawdę są pozycje partów

Retailowe `nw_wswmls002.uti` ma:

```text
BaseItem=1
ModelPart1=23
ModelPart2=63
ModelPart3=23
```

Dokładne MDL-e wskazane przez tę receptę zostały odczytane in-place własnym
readerem:

| Slot | MDL | Pozycja child Trimesh | Orientacja |
|---|---|---|---|
| Bottom | `wswls_b_023` | `[0.000183583, -0.14571, 0.000191967]` | `[0, 0, 0, 1]` |
| Middle | `wswls_m_063` | `[0.000394173, -0.0206093, 0.000168152]` | `[0, 0, 0, 1]` |
| Top | `wswls_t_023` | `[0.0000303633, 0.262925, 0.0018867]` | `[0, 0, 0, 1]` |

Każdy plik ma root i jeden child Trimesh. Wartości `position` są kontrolerami
node'a child mesh. Po dodaniu lokalnego zakresu Y mesha do jego pozycji
otrzymujemy:

| Slot | Lokalny Y | Y po kontrolerze |
|---|---|---|
| Bottom | `[-0.0568282, 0.229862]` | `[-0.2025382, 0.084152]` |
| Middle | `[0, 0.17765]` | `[-0.0206093, 0.1570407]` |
| Top | `[-0.134606, 0.668461]` | `[0.128319, 0.931386]` |

Retailowy zestaw ma kontrolowane zakładki zakresów Bottom–Middle i
Middle–Top. To nie jest automatyczny snap Toolsetu. To wynik transformacji
zapisanych w MDL.

Wniosek dla edytora: użytkownik może ustawiać transformacje partów w naszym
Studio, ale build musi je **wypalić do kontrolerów node'ów odpowiednich MDL**.
UTI nadal zapisuje tylko numery wariantów.

## 9. Jak Toolset składa ikonę

Ścieżka w `decompiled_all.c:339398-339440`:

1. pobiera `InvSlotWidth` i `InvSlotHeight`;
2. mnoży oba wymiary przez 32 (`<< 5`);
3. kolejno tworzy nazwy warstw Bottom, Middle i Top helperem z `i`;
4. trzykrotnie wywołuje `FUN_0064af98` na tym samym canvasie.

`FUN_0064af98` (`decompiled_all.c:338985-339094`) ładuje obraz i przekazuje go
do kompozycji. Jego loader `FUN_0064b12c` żąda resource type `3`
(`decompiled_all.c:339208-339250`), czyli TGA. Retail KEY ma dla tych samych
resrefów również skompresowane odpowiedniki DDS typu `2033`.

Kontrakty 3D i 2D muszą pozostać rozdzielone:

- brak `i` + type 2002 = model 3D;
- `i` + type 3/2033 = warstwa ikony.

Ikony nie są screenshotem z viewportu 3D.

## 10. Pozostałe profile

### 10.1 Jedna część, bez kolorów (`ModelType=0`)

UTI zapisuje `ModelPart1`. Resolver tworzy jeden model, normalnie bez segmentu
Bottom/Middle/Top. Retailowe tarcze są właśnie w tej klasie.

### 10.2 Jedna część i sześć kolorów (`ModelType=1`)

UTI zapisuje `ModelPart1` oraz:

- `Leather1Color`;
- `Leather2Color`;
- `Cloth1Color`;
- `Cloth2Color`;
- `Metal1Color`;
- `Metal2Color`.

`GenderSpecific` może dodać segment znaku do nazwy. Helmet i cloak nie powinny
być traktowane jako wariant trzyczęściowy tylko dlatego, że są equipowalne.

### 10.3 Armor (`ModelType=3`)

UTI zachowuje 19 pól w kolejności z
`ITEM_APPR_ARMOR_MODEL_RFOOT..ROBE` (`nwscript.nss:590-609`) oraz sześć
kanałów kolorów. Robe uruchamia maskowanie wybranych części ciała, a preview
składa sekcje armorowego/capart namespace. To inny composer niż prosty
Bottom/Middle/Top.

Ten audyt potwierdza rozróżnienie profilu i serializację armora, ale nie
wykonuje nowego P-REF kompletu 19 armor parts.

## 11. Kontrakt dla Meshy2Aurora

### 11.1 Model domenowy

Minimalna recepta nie powinna mieć enumu `Weapon | Shield`. Powinna wyglądać
semantycznie tak:

```yaml
itemAppearanceRecipeV1:
  baseItem: 1
  modelType: 2
  itemClass: WSwLs
  parts:
    - { slot: bottom, variant: 23, sourceNode: Part_A }
    - { slot: middle, variant: 63, sourceNode: Part_B }
    - { slot: top, variant: 23, sourceNode: Part_C }
  iconLayers:
    mode: separate_aurora_layers
```

`sourceNode` jest metadanym naszego authoringu. Nie trafia do UTI.

### 11.2 Workflow authoringu

1. Użytkownik wybiera istniejący `BaseItem` albo jawnie dostarcza nowy rekord
   `baseitems.2da`.
2. Studio odczytuje `ModelType` i pokazuje wynikający z niego zestaw slotów.
3. Użytkownik przypisuje fragmenty Meshy/GLB do slotów.
4. Studio pozwala ustawić lokalne translation/rotation/scale i pokazuje
   złożenie.
5. Build wypala transformację do node'a MDL każdego partu.
6. Build generuje resref zgodny z `ItemClass`, slotem i numerem wariantu.
7. UTI zapisuje wyłącznie numery w odpowiednich polach.
8. Opcjonalny icon composer tworzy osobne warstwy `i...` TGA/DDS.

### 11.3 Bramy builda

- `BaseItem` musi istnieć w efektywnej tabeli `baseitems.2da`.
- `ModelType` musi zgadzać się z liczbą i rodzajem pól UTI.
- Każdy logiczny resref musi mieć maksymalnie 16 znaków i być unikalny w
  efektywnym namespace HAK/modułu/retail.
- Dla profilu trzyczęściowego Bottom jest wymaganym rootem; brak Middle lub
  Top nie może być ukryty fallbackiem `DefaultModel`.
- Transformacje muszą być wypalone i potwierdzone own readbackiem MDL.
- Preview musi sprawdzić seam/gap/overlap na złożonym modelu, nie tylko każdy
  part osobno.
- Budżet produktu `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000` dotyczy sumy
  wszystkich renderowalnych partów jednego itemu.
- Niezależna granica pojedynczego triangle-list mesh stream pozostaje
  `65 535` indeksów / `21 845` trójkątów; większy part wymaga segmentacji na
  wiele mesh nodes, nie podwyższenia tej granicy.
- Warstwy ikon są osobnym artefaktem i osobnym readbackiem.

## 12. Konsekwencje dla obecnego mocka

Mock wymaga następujących korekt:

1. usunąć główny podział `Shield / Weapon`;
2. zastąpić go wyborem `Base item` i wykrytym `Aurora composition profile`;
3. zmienić opis `B = Blade` na `Bottom`;
4. pokazywać `Bottom / Middle / Top` jako sloty recepty, nie klasy itemów;
5. pokazać, że transformacje są authoring metadata wypalanym do MDL;
6. rozdzielić panel `3D parts` od `Inventory icon layers`;
7. raportować łączny budżet trójkątów całego złożenia;
8. pokazać końcowe resrefy MDL bez `i` i resrefy ikon z `i`.

## 13. Klasyfikacja twierdzeń

| Twierdzenie | Klasa | Stan |
|---|---|---|
| `ModelType` wybiera profil 1/1+colors/3/19+colors | fakt z dekompilacji | potwierdzone |
| `ModelPart1/2/3` = Bottom/Middle/Top | dekompilacja + NWScript + UI + retail | potwierdzone |
| MDL nie mają prefiksu `i` | retail KEY/BIF | potwierdzone |
| zasoby `i...` są TGA/DDS | dekompilacja + retail KEY | potwierdzone |
| transformacje partów są w MDL | retail own-reader + brak pól UTI | potwierdzone dla badanego zestawu |
| append zachowuje kontrolery partów przy składaniu | wniosek implementacyjny z loadera i witnessu | wysoka pewność |
| dokładny blend/alpha order wszystkich warstw ikon | hipoteza poza zakresem | otwarte |
| reguły seamów są identyczne dla każdej rodziny itemu | hipoteza | wymaga szerszego corpusu |
| armor 19-part visual parity | proof gap | niewykonane w tym audycie |

## 14. Evidence pointers

### Dekompilacja

- `decompiled_all.c:169023-169039` — resolver z `i`;
- `decompiled_all.c:169157-169172` — resolver bez `i`;
- `decompiled_all.c:128061-128234` — model 3D Bottom + append Middle/Top;
- `decompiled_all.c:299891-299913` — `ModelType` -> liczba partów/kolorów;
- `decompiled_all.c:300168-300255` — odczyt pól wyglądu UTI;
- `decompiled_all.c:300324-300403` — zapis pól wyglądu UTI;
- `decompiled_all.c:338985-339094` — kompozytor warstwy obrazu;
- `decompiled_all.c:339208-339250` — ładowanie resource type 3;
- `decompiled_all.c:339398-339440` — kompozycja ikon Bottom/Middle/Top;
- `strings.tsv:26204-26215` — UI Appearance i handlery trzech slotów.

### Retail

- `baseitems.2da` — `ModelType`, `ItemClass`, `GenderSpecific`,
  `DefaultModel`, `DefaultIcon`, `InvSlotWidth`, `InvSlotHeight`;
- `nwscript.nss:575-617` — appearance type, 19 armor indices i trzy indeksy
  Bottom/Middle/Top;
- `nwscript.nss:10944-10968` — `CopyItemAndModify` i `GetItemAppearance`;
- `nw_wswmls002.uti` — `BaseItem=1`, `ModelPart1/2/3=23/63/23`;
- `nwn_base.key` — rozdzielne typy i locatory MDL/TGA/DDS;
- [P-REF `WSwLs` parts](evidence/p-ref-item-wswls-parts-2026-07-29.md)
  — hashe, own-reader i invarianty dokładnych trzech MDL.

## 15. Odpowiedź praktyczna

Tak, po ponownym audycie wiadomo, jak to odróżniać i gdzie jest definiowane:

- rodzaj recepty: `baseitems.2da.ModelType`;
- namespace nazw: `baseitems.2da.ItemClass`;
- wybór wariantów: pola wyglądu w UTI;
- nazwy i kolejność slotów: resolver Toolsetu + publiczne stałe NWScript;
- pozycja, obrót, skala i seam każdego partu: node controllers w MDL;
- składanie 3D: Bottom jako root, potem append Middle i Top;
- składanie ikony: osobne warstwy `i...` TGA/DDS.

To jest właściwy fundament nowej opcji w pipeline. Nie `shield` kontra
`weapon`, tylko `BaseItem -> ModelType -> sloty partów -> wypalone MDL-e`.
