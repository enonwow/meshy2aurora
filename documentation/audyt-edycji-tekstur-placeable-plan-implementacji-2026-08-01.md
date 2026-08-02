# Audyt edycji tekstur Placeable oraz plan implementacji

Data: 2026-08-01

Status: `POINTS_1_2_IMPLEMENTED_OFFLINE / OWNER_PROOF_PENDING`

Zakres: aplikacja webowa local-first, statyczny Placeable, Core Rust, WASM,
Worker, viewport, TGA/HAK i owner-run proof w Aurora Toolset/NWN.

## Wynik implementacji 2026-08-01

Produkcyjny vertical slice punktów 1 i 2 został wdrożony. Źródłowy GLB
pozostaje read-only, a edycja jest wersjonowaną recipe przypisaną do SHA-256
źródła, materiału i obrazu. Każdy używany material slot może niezależnie użyć
źródła albo override'u PNG/JPEG. Core waliduje descriptor i binarny payload,
dekoduje obraz z limitami, stosuje politykę `OPAQUE_ONLY`, tworzy
deterministyczny TGA, deduplikuje exact bajty i wiąże wynik z MDL oraz HAK.

Zamknięty zakres obejmuje:

- strict-schema `PlaceableTextureAuthoringDocumentV1` oraz osobne payloady
  binarne z length/hash/MIME validation;
- Material/Texture Inspector, wybór slotu, podświetlenie geometrii,
  `Replace`, `Use source`, `Reset`, `Undo/Redo` i podgląd `Source/Edited`;
- wspólny resolver Core używany przed buildem oraz build V5 z tym samym
  authoringiem i payloadami;
- deterministyczne resrefy, loose TGA, raport slot -> resref/hash, readback HAK
  oraz `placeable-authoring-v3.json`;
- nazwaną diagnostykę stale recipe, niespójnych payloadów, limitów, alpha oraz
  source cutout (`MASK`/`BLEND`) wymagającego przyszłego kontraktu TXI/MTR;
- raportowanie source normal/roughness/metalness maps jako świadomie
  nieuwzględnionych w profilu V1 final diffuse.

Punkty 3 (projekcja/re-UV) i 4 (texture painting) nie zostały ukrycie dodane do
MVP: pozostają odpowiednio `DESIGN_REQUIRED` i `OUT_OF_MVP`.

Walidacja offline obejmuje Core, natywne API WASM, realny Worker/WASM, reducer
UI, TypeScript i build produkcyjny. Kryteria wymagające oceny wyglądu w Aurora
Toolset/NWN pozostają świadomie otwarte do owner proofu; implementacja ani ta
aktualizacja dokumentu nie uruchamiały Toolsetu lub NWN.

Zamknięte bramki offline:

- `cargo test -p m2a-core --quiet` — zielony pełny pakiet Core;
- `cargo test -p m2a-wasm --quiet` — `36 passed`, `0 failed`;
- `cargo test -p m2a-core --test placeable_pipeline placeable_texture` —
  `4 passed`, w tym exact HAK/TGA, niezależny slot, alpha, malformed PNG,
  stale binding, authoring geometrii i cutout bez TXI;
- `npm run typecheck` oraz `npm test -- --run` — `41` plików i `247 passed`;
- `npm run test:worker-integration` — realny WASM Worker, `9 passed`,
  `2 skipped` zgodnie z konfiguracją fixture;
- `npm run build` — produkcyjny build Vite zakończony powodzeniem;
- `git diff --check` — brak błędów whitespace.

## 1. Werdykt

Nakładanie nowych tekstur na Placeable jest wykonalne. Aktualny produkt ma już
większość fundamentów formatu docelowego:

- GLB IR zachowuje materiały, tekstury, obrazy, `TEXCOORD_0` i powiązania
  `primitive -> material -> texture -> image`;
- Profile A nadaje stabilne sloty materiałowe i zachowuje `uv0`;
- Placeable potrafi przypisać różne tekstury do różnych slotów materiałowych;
- Core przyjmuje jawne payloady tekstur TGA lub DDS i pakuje je do HAK-a;
- istnieje deterministyczny writer TGA z readbackiem i SHA-256;
- Studio ma viewport Three.js, Worker/WASM oraz mechanizm wersjonowanego
  authoringu Placeable.

Brakującą częścią nie jest więc podstawowa obsługa tekstury przez Aurorę, lecz
wersjonowany kontrakt override'u, przekazanie zewnętrznych bajtów obrazu przez
Worker/WASM, edytor materiałów, zgodny podgląd oraz związanie hashy override'u z
wynikowym MDL/TGA/HAK.

Rekomendowana kolejność:

1. zrealizować razem punkt 1 i punkt 2 jako produkcyjny vertical slice;
2. po ich owner proofie rozważyć ograniczoną projekcję/nowe UV z punktu 3;
3. punkt 4 pozostawić poza MVP i traktować jako osobny produkt albo round-trip
   przez Blender.

## 2. Audytowane funkcje

| Punkt | Funkcja | Werdykt | Złożoność | Rekomendacja |
|---|---|---|---|---|
| 1 | Podmiana istniejącej tekstury z zachowaniem UV | wykonalna teraz | średnia | implementować |
| 2 | Osobna tekstura dla każdego materiału | fundament Core istnieje | średnia | implementować razem z 1 |
| 3 | Dowolny obraz bez zgodności z UV, projekcja lub re-UV | brak aktywnego pipeline'u | wysoka | osobny etap eksperymentalny |
| 4 | Malowanie bezpośrednio na modelu 3D | brak infrastruktury edytora rastrowego | bardzo wysoka | poza MVP |

Orientacyjny effort jednej osoby po zamknięciu kontraktu i przy zachowaniu TDD:

- punkty 1+2: około 8–15 dni roboczych łącznie;
- punkt 3: około 3–6 tygodni dla ograniczonego, produkcyjnego profilu;
- punkt 4: co najmniej 6–12 tygodni i dalszy koszt utrzymania.

Są to szacunki względnej wielkości, nie zobowiązanie terminowe. Owner proof
Aurora/NWN i ewentualne odkrycia Aurora First mogą zmienić zakres.

## 3. Klasyfikacja dowodów

### 3.1. Fakty projektowe z aktualnego kodu

`PROJECT_FACT`

- `crates/m2a-core/src/glb/mod.rs` przechowuje `IrMaterial`,
  `IrTextureBinding`, `IrTexture`, `IrImageRef` i `IrPrimitive.uv0`.
- Decoder obrazów obsługuje osadzone PNG/JPEG i produkuje `TgaImageV1` z
  limitami wymiaru, liczby pikseli oraz 64 MiB wynikowego TGA.
- `crates/m2a-core/src/model_pipeline.rs` rozwiązuje każdy użyty slot materiału
  do dokładnego obrazu base color bez założenia, że indeksy materiału, tekstury
  i obrazu są takie same.
- `crates/m2a-core/src/placeable.rs` automatycznie wyciąga base color z GLB,
  emituje TGA, deduplikuje identyczne obrazy i wiąże wynik z material slotami.
- Niskopoziomowy `StaticPlaceableBuildRequestV1` już przyjmuje
  `material_textures` i `textures`; wspiera TGA type `3` i DDS type `2033`.
- Testy Placeable potwierdzają dwa różne obrazy dla dwóch material slotów oraz
  deduplikację jednego wspólnego obrazu.
- `PlaceableAuthoringDocumentV2` opisuje elementy i PWK, ale nie ma recipes ani
  payloadów tekstur.
- `BUILD_PLACEABLE_PACKAGE` przyjmuje GLB, 2DA, identity, placement i authoring,
  lecz nie przyjmuje zewnętrznych obrazów ani descriptorów override'u.
- Studio pokazuje tylko liczbę materiałów/tekstur. Projekcja source inspection
  nie zachowuje szczegółowej listy materiałów, obrazów i bindingów potrzebnej
  edytorowi.
- Placeable używa obecnie stałych resrefów/nazw demo. Zmiana tekstury nie jest
  jeszcze częścią deterministycznej tożsamości artefaktu.

### 3.2. Fakty Aurora/retail zapisane w projekcie

`DECOMP_FACT / RETAIL_FACT / BIOWARE_SPEC`

- Placeable MDL odwołuje się do tekstury przez resref.
- HAK rozróżnia TGA jako Resource Type `3` i DDS jako Resource Type `2033`.
- Pierwszy profil produktu może używać klasycznego TGA/DDS; TXI/MTR należy
  dodawać dopiero dla jawnie zdefiniowanego materiału i po osobnym proofie.
- Brak albo konflikt tekstury jest awarią warstwy materiałowej i nie jest
  równoważny z brakiem geometrii.

Źródło utrwalone:
`documentation/audyt-placeable-widocznosc-aurora-nwn-plan-implementacji-2026-07-25.md`,
sekcje 11, 12 i 17.

### 3.3. Wcześniej zatwierdzony kierunek

`PRIOR_APPROVED_DIRECTION`

`documentation/viewport-walidacja-animacje-plan-codex.md`, sekcja 9.1, już
ustala niedestrukcyjny kontrakt:

- źródłowy GLB pozostaje read-only;
- edycja jest deklaratywną recipe;
- reset przywraca źródło;
- wynikowy TGA jest zawsze wyliczany ponownie z bieżącej recipe;
- PBR musi zostać jawnie zbakowany albo oznaczony jako odrzucony/unsupported;
- manualne pixel painting jest poza MVP.

Kod nie implementuje jeszcze planowanego `m2a.project.json`. Nowy vertical
slice nie może twierdzić, że ten format istnieje. Najpierw powinien dodać
wersjonowany kontrakt Placeable i sidecar, który później może stać się częścią
formatu projektu.

## 4. Stan obecnego przepływu

```text
source GLB
  -> GLB ingest: materials/textures/images/UV0
  -> Profile A: stable material slots + preserved UV0
  -> automatic embedded baseColor extraction
  -> PNG/JPEG decode
  -> deterministic TGA
  -> MDL material-slot binding
  -> HAK: MDL + TGA(s) + 2DA + PWK
```

Brakujący przepływ:

```text
user image
  -> validation + hash + decode
  -> versioned material override recipe
  -> exact material-slot resolution
  -> edited preview
  -> deterministic target texture
  -> MDL/TGA/HAK identity bound to override hashes
```

## 5. Punkt 1 — podmiana tekstury z zachowaniem UV

### 5.1. Co już działa

- geometria ma `uv0` i jest eksportowana z tym samym kontraktem Profile A;
- Core potrafi zapisać RGB/RGBA jako TGA;
- HAK i MDL potrafią użyć jawnie przekazanego payloadu i resrefu;
- viewport potrafi wyświetlić mapę Three.js na istniejącym materiale;
- PNG/JPEG decoder istnieje w Core, choć jest obecnie związany z obrazem
  osadzonym w GLB.

### 5.2. Braki

- brak uploadu obrazu dla konkretnego material slotu;
- brak publicznego, ogólnego decode API dla obrazu spoza GLB;
- brak recipe i hasha override'u;
- brak podglądu source/edited oraz resetu tekstury;
- brak ścieżki override przez Worker/WASM do buildera Placeable;
- brak włączenia override'u do tożsamości resrefów i artefaktów;
- brak jawnej polityki alpha oraz PBR dla ręcznie dostarczonego obrazu.

### 5.3. Decyzja implementacyjna

`IMPLEMENTATION_DECISION`

Pierwsza wersja obsługuje PNG/JPEG jako wejście i emituje klasyczny TGA.
Podmiana oznacza **finalny diffuse dla Aurory**, a nie nowy materiał PBR.
Źródłowe UV pozostają bez zmian.

Alpha nie może być interpretowana po cichu. V1 ma wybrać jedno z dwóch
bezpiecznych rozwiązań:

- profil `OPAQUE`, który blokuje nieprzezroczystość inną niż 1; albo
- jawne `FLATTEN_ALPHA` z wybranym kolorem tła.

Cutout/transparency wymagające TXI/MTR pozostają poza V1 do czasu osobnego
Aurora First contractu i proofu.

### 5.4. Ryzyka

- obraz nieprzygotowany pod istniejący atlas UV będzie rozciągnięty albo
  pokaże szwy; to błąd dopasowania assetu, nie pipeline'u;
- niejawne użycie source `baseColorFactor` może dać różnicę między Three.js i
  TGA; override V1 powinien być traktowany jako finalny diffuse bez ponownego
  mnożenia przez source factor;
- 4K RGBA zbliża się do aktywnej granicy 64 MiB TGA; limit ma działać przed
  dużą alokacją i zgłaszać nazwany błąd;
- stały texture resref może powodować stale cache lub kolizję HAK. Resref i
  manifest muszą zależeć od recipe/hashu wejścia.

## 6. Punkt 2 — tekstura per materiał

### 6.1. Co już działa

- Profile A ma stabilne `material_source_bindings`;
- model dzieli segmenty według `material_slot`;
- placeable builder potrafi emitować wiele TGA i wiele bindingów;
- istnieje raport `texture_resources` z material slotami i SHA-256;
- wspólny obraz używany przez kilka slotów jest deduplikowany.

To sprawia, że punkt 2 nie wymaga przebudowy formatu MDL. Jest rozszerzeniem
kontraktu authoringu i UI.

### 6.2. Braki

- source inspection używane przez UI redukuje dane do samych liczników;
- brak listy: slot, source material id/name, primitive/node usage, source image
  id/hash, thumbnail i UV coverage;
- brak edytora przypisania `material slot -> source/override`;
- brak rozstrzygnięcia, czy override jednego z dwóch slotów współdzielących
  source image ma rozdzielić zasoby — powinien rozdzielić je deterministycznie;
- brak walidacji stale recipe po zmianie GLB/material order.

### 6.3. Decyzja implementacyjna

`IMPLEMENTATION_DECISION`

Recipe nie może identyfikować materiału wyłącznie numerem slotu. Każde
przypisanie powinno zawierać:

- `materialSlot`;
- `sourceMaterialId` i opcjonalną nazwę informacyjną;
- `sourceImageSha256`;
- tryb `SOURCE` albo `OVERRIDE`;
- dla override'u: `inputSha256`, MIME, byte length, output resref i alpha
  policy.

Resolver fail-closed odrzuca recipe, gdy slot nadal istnieje, ale material id
lub source image hash nie zgadza się z aktualnym GLB.

### 6.4. Ryzyka

- Meshy często tworzy wiele materiałów o mało czytelnych nazwach; UI musi
  podświetlać geometrię używającą wybranego slotu;
- duża liczba unikatowych TGA szybko zwiększa HAK;
- scalanie materiałów lub zmiana authoringu geometrii może zmienić użycie
  slotów; resolver musi pracować po dokładnie tej samej projekcji renderowej,
  która trafia do MDL;
- deduplikacja może być wyłącznie po exact wynikowych bajtach/hashu, nie po
  nazwie pliku.

## 7. Punkt 3 — dowolny obraz, projekcja lub nowe UV

### 7.1. Co już działa

- Core przechowuje pozycje, indeksy i UV0;
- authoring Placeable ma transformacje elementów;
- Three.js daje raycast, kamery i render targety;
- Meshy Lab ma osobny ReTexture dla zweryfikowanego `inputTaskId`, domyślnie z
  `enableOriginalUv=true`, i zwraca nowy GLB.

### 7.2. Braki

- brak generatora UV, atlas packera i seam policy;
- brak planar/box/cylindrical/triplanar projection recipe;
- brak bake'u obrazu z projekcji do finalnego atlasu;
- brak overlap/texel-density/stretch diagnostics;
- placeable authoring celowo edytuje obiekty/komponenty, nie pojedyncze
  wierzchołki ani UV;
- brak kontraktu określającego, czy UV liczy się przed czy po transformacjach
  elementów;
- brak readbacku potwierdzającego dokładny target UV użyty przez MDL.

### 7.3. Decyzja implementacyjna

`IMPLEMENTATION_DECISION`

Punkt 3 nie może być automatycznym fallbackiem przy błędzie punktu 1. Powinien
być osobnym, eksperymentalnym profilem z ograniczonym zestawem projekcji:

1. `PLANAR_AXIS` dla płaskich elementów;
2. `BOX_PROJECT` dla prostych brył;
3. opcjonalnie `CYLINDRICAL_AXIS` dla masztów/rur.

Automatyczny unwrap i packing złożonego statku nie powinien być deklarowany
bez osobnej biblioteki/algorytmu, fixture i pomiarów jakości. Dla assetów
Meshy preferowaną alternatywą jest Meshy ReTexture zachowujący oryginalne UV i
ponowny import nowego GLB. Dla lokalnych assetów o złożonej geometrii bez
zgodnego UV preferowany jest round-trip przez Blender.

### 7.4. Hipotezy wymagające testu

`HYPOTHESIS`

- prosty box projection może być wystarczający dla części architektury i
  prostych propsów;
- dla statku z wieloma cienkimi deskami automatyczny projection bez selekcji
  wysp prawdopodobnie da zbyt dużo nakładania i szwów;
- target preview można oprzeć na output UV IR, ale musi to zostać porównane z
  own binary MDL readback oraz owner proofem.

## 8. Punkt 4 — malowanie bezpośrednio na modelu

### 8.1. Co już działa

- viewport i raycast Three.js;
- obsługa pointer events;
- materiał z mapą tekstury;
- undo/redo dla authoringu elementów może być wzorcem architektonicznym.

### 8.2. Braki

- UV-space framebuffer/canvas i mapowanie trafienia raycast na barycentryczne
  UV;
- pędzle, rozmiar, hardness, opacity, kolor i blending;
- obsługa szwów i malowanie na sąsiednich wyspach;
- undo/redo dla dużych obrazów bez kopiowania pełnych 64 MiB przy każdym
  ruchu;
- warstwy, maski, flood fill, clone/smudge i eksport spłaszczonego obrazu;
- seam dilation/bleed zapobiegający czarnym liniom przy mipmappingu;
- deterministyczny raster Core zgodny z podglądem GPU;
- testy cross-browser/WebGL i polityka utraty kontekstu.

### 8.3. Decyzja implementacyjna

`IMPLEMENTATION_DECISION`

Punkt 4 pozostaje poza MVP Meshy2Aurora. Jego realizacja ma sens dopiero po
stabilnym punkcie 1+2 i ewentualnym punkcie 3. Jeśli zostanie rozpoczęty,
powinien być osobnym modułem `Texture Paint`, a nie dodatkowym przyciskiem w
panelu materiałów.

Pierwszy proof-of-concept może obsługiwać wyłącznie jeden materiał, jedno UV0,
okrągły pędzel koloru i kafelkowe undo. Nie może zostać nazwany produkcyjnym
edytorem, dopóki nie ma seam dilation, limitów pamięci i deterministycznego
eksportu.

## 9. Proponowany kontrakt danych

Pierwszy etap powinien dodać osobny, strict-schema kontrakt, bez wciskania
binarnych obrazów do JSON:

```json
{
  "schemaVersion": 1,
  "sourceSha256": "<glb-sha256>",
  "bindings": [
    {
      "materialSlot": 0,
      "sourceMaterialId": 3,
      "sourceMaterialName": "wood",
      "sourceImageSha256": "<source-image-sha256>",
      "mode": "OVERRIDE",
      "overrideAssetId": "texture-override-0",
      "overrideSha256": "<input-image-sha256>",
      "overrideMimeType": "image/png",
      "overrideByteLength": 123456,
      "alphaPolicy": "OPAQUE",
      "outputResref": "<derived-resref>"
    }
  ]
}
```

Bajty obrazów są przekazywane jako descriptor list + payload buffers w Worker
request. Core/WASM sprawdza descriptor, długość i SHA-256 przed dekodowaniem.
JSON bez dokładnego payloadu albo payload bez descriptoru jest błędem.

Po integracji dokument może zostać włączony do
`PlaceableAuthoringDocumentV3`; sidecar eksportu powinien mieć nazwę
`placeable-authoring-v3.json`. Historyczne V1/V2 pozostają odczytywalne i
oznaczają wszystkie materiały w trybie `SOURCE`.

## 10. Docelowy przepływ

```text
GLB + PlaceableAuthoringV3 + override image buffers
  -> strict source/hash/material validation
  -> render projection authoring
  -> stable material-slot resolver
  -> source or override image decode
  -> deterministic alpha/color policy
  -> deterministic TGA per resolved texture
  -> exact-byte deduplication
  -> MDL material bindings
  -> HAK + loose TGA artifacts + report + authoring sidecar
```

Podgląd Studio ma korzystać z tego samego **resolved texture report**, co
build. UI może używać GPU do wyświetlania, ale nie może samodzielnie zgadywać
slotów, resrefów, deduplikacji albo wynikowych hashy.

## 11. Plan implementacji

### Etap T0 — kontrakt i testy blokujące

1. Dodać syntetyczne GLB fixture:
   - jeden materiał/jeden obraz;
   - dwa materiały/dwa obrazy;
   - dwa materiały/jeden wspólny obraz;
   - materiał bez UV0;
   - materiał z base color factor innym niż biały;
   - PNG z alpha.
2. Zapisać strict schema recipe i descriptorów payloadów.
3. Zapisać testy stale source/material/image identity.
4. Zamknąć V1 alpha policy oraz jawne odrzucenie PBR/cutout.

### Etap T1 — Material/Texture Inspector

1. Rozszerzyć publiczny raport Core o:
   - material slot;
   - source material id/name;
   - używane primitive/node ids;
   - source texture/image id i SHA-256;
   - rozmiar, format, alpha presence;
   - obecność UV0 i source PBR maps.
2. Rozszerzyć projekcję Worker/UI zamiast redukowania danych do liczników.
3. Dodać panel `Materials` i izolowanie/podświetlenie slotu w viewportcie.

### Etap T2 — resolver override'u w Core

1. Wydzielić ogólny bounded PNG/JPEG decoder z obecnego decode obrazu GLB.
2. Dodać `resolve_placeable_textures_v1` zwracający canonical resolved report.
3. Zweryfikować SHA-256, MIME, długość, limity, material identity i UV0.
4. Emitować finalne `TgaImageV1`, TGA SHA-256, output resref i slot list.
5. Zachować legacy `SOURCE` byte-for-byte względem obecnego buildera.

### Etap T3 — WASM i Worker

1. Dodać `RESOLVE_PLACEABLE_TEXTURES` z request ID i revision binding.
2. Przekazywać obrazy jako transferable `ArrayBuffer` z descriptorami.
3. Odrzucać odpowiedzi dla starej rewizji źródła/recipe.
4. Rozszerzyć build Placeable o exact tę samą recipe i payloady.
5. Nie klonować wielokrotnie dużych obrazów między UI i Workerem.

### Etap T4 — UI punktów 1 i 2

1. Panel każdego material slotu:
   - nazwa i użycie;
   - miniatura source;
   - `Use source`, `Replace texture`, `Reset`;
   - nazwa i hash override'u;
   - diagnostyka UV/alpha/rozmiaru.
2. Podgląd `Source` / `Edited`.
3. Undo/redo recipe bez kopiowania binarnych payloadów.
4. Zmiana jednego slotu nie może zmieniać innych slotów.
5. Build jest zablokowany, dopóki resolved report nie odpowiada bieżącej
   source revision i recipe hash.

### Etap T5 — packaging, report i tożsamość

1. Związać placeable identity z:
   - source GLB SHA-256;
   - authoring SHA-256;
   - uporządkowaną listą override SHA-256;
   - aktywną polityką tekstur.
2. Emitować każdy wynikowy TGA jako osobny downloadable artifact.
3. Zapisać w raporcie sloty, resrefy, input/output hash i byte length.
4. Readback HAK musi znaleźć każdy exact `(resref, resource type)` i ten sam
   SHA-256.
5. Eksportować `placeable-authoring-v3.json`.

### Etap T6 — regresja i owner proof punktów 1+2

1. Pełne unit/integration/WASM/Worker/UI tests.
2. Exact synthetic fixture z widocznym wzorem UV orientation.
3. Jeden realny owner-selected Placeable z jednym materiałem.
4. Jeden realny owner-selected Placeable z co najmniej dwoma materiałami.
5. Zamrozić tylko jeden exact candidate na scenariusz, przygotować handoff i
   pozostawić finalny Toolset/NWN proof właścicielowi.

### Etap T7 — punkt 3, osobny eksperyment

1. Zatwierdzić profile `PLANAR_AXIS` i `BOX_PROJECT`.
2. Zdefiniować coordinate space i moment aplikacji względem transformacji
   elementów.
3. Dodać CPU resolver UV i canonical hash output UV.
4. Dodać overlap/stretch/texel-density diagnostics.
5. Dodać bake do target atlasu i porównać preview z output IR/readbackiem.
6. Dopiero po owner proofie zdecydować, czy profil jest produkcyjny.

### Etap T8 — punkt 4, osobny projekt

1. Opracować oddzielny ADR i memory model.
2. Zbudować single-material prototype: raycast -> UV -> brush stamp.
3. Dodać tiled undo, seam dilation i deterministic flatten.
4. Dopiero potem ocenić warstwy, maski i wielomateriałowość.

## 12. Kryteria ukończenia

### 12.1. Kryteria globalne

- [x] Źródłowy GLB nie jest modyfikowany ani nadpisywany.
- [x] Każda recipe jest strict-schema, source-hash-bound i fail-closed.
- [x] Te same wejścia dają byte-identical TGA, MDL i HAK.
- [x] Podgląd i build używają tego samego authoringu i resolved reportu Core.
- [x] Build nie przyjmuje stale response ani brakującego payloadu.
- [x] Każdy wynikowy resref ma maksymalnie 16 znaków i jest deterministyczny.
- [x] Raport zawiera source, recipe, input image, output TGA, MDL i HAK SHA-256.
- [x] HAK readback potwierdza exact tekstury przypisane do exact slotów.
- [x] Legacy authoring V1/V2 i tryb `SOURCE` zachowują obecny output
  byte-for-byte.
- [x] Brak regresji geometrii, UV0, PWK, shadow flags i limitu 300 000
  trójkątów.
- [x] OOM/oversized/malformed images kończą się nazwanym błędem bez panic.
- [ ] Finalny wizualny werdykt jest opisany osobno dla Toolsetu i NWN i jest
  dostarczony przez właściciela.

### 12.2. Punkt 1 — done

- [x] Użytkownik może zastąpić teksturę jednego materiału PNG/JPEG.
- [x] Tekstura jest pokazana na istniejącym UV bez zmiany geometrii i UV0.
- [x] `Source`/`Edited` oraz `Reset` działają bez ponownego importu GLB.
- [x] Alpha ma jawną politykę; unsupported cutout/PBR jest raportowany, nie
  pomijany po cichu.
- [x] Loose TGA i TGA wewnątrz HAK mają ten sam SHA-256.
- [ ] Wzór orientacji UV jest zgodny offline i zaakceptowany w owner proofie.

### 12.3. Punkt 2 — done

- [x] UI pokazuje wszystkie użyte material sloty i geometrię każdego slotu.
- [x] Każdy slot można niezależnie ustawić na `SOURCE` albo `OVERRIDE`.
- [x] Override jednego współdzielonego source image nie zmienia drugiego slotu.
- [x] Exact identyczne wynikowe obrazy mogą zostać bezpiecznie deduplikowane.
- [x] Recipe po zmianie GLB/material identity jest odrzucana jako stale.
- [x] MDL readback i raport HAK potwierdzają slot -> texture resref dla każdego
  slotu.
- [ ] Owner proof pokazuje co najmniej dwa wizualnie różne materiały na tym
  samym Placeable.

### 12.4. Punkt 3 — done

- [ ] Każdy wspierany projection profile ma wersjonowany contract i fixture.
- [ ] Output UV jest deterministyczny i ma canonical SHA-256.
- [ ] Preview renderuje dokładnie output UV, nie source UV.
- [ ] Raport pokazuje overlap, stretch i texel density lub jawnie blokuje asset,
  którego nie potrafi ocenić.
- [ ] Bake nie ma niepomalowanych pikseli przy krawędziach wysp w zakresie
  zdefiniowanego dilation policy.
- [ ] Box/planar fixture przechodzi own readback i owner Toolset/NWN proof.
- [ ] Złożony asset, którego profil nie obsługuje, kończy się `unsupported`, a
  nie pozornie udanym eksportem.

### 12.5. Punkt 4 — done

- [ ] Raycast wybiera poprawny trójkąt i barycentryczne UV.
- [ ] Stroke w viewportcie i wynikowy raster są zgodne.
- [ ] Malowanie przez szew nie zostawia przerwy według jawnej seam policy.
- [ ] Undo/redo jest kafelkowe/delta-based i mieści się w ustalonym limicie
  pamięci.
- [ ] Utrata kontekstu WebGL nie traci zatwierdzonej recipe/raster state.
- [ ] Flatten jest deterministyczny i daje byte-identical TGA dla tych samych
  stroke'ów.
- [ ] Jeden i wiele material slotów mają osobne testy.
- [ ] Owner proof potwierdza finalny raster w Toolset i NWN.

## 13. Wymagana macierz testów

| Warstwa | Minimalne pokrycie |
|---|---|
| Core unit | decode limits, hash, alpha policy, material resolver, dedup, resref |
| Core integration | GLB + recipe + images -> MDL/TGA/HAK + own readback |
| WASM parity | exact resolved JSON i bytes względem native Core |
| Worker | transferable buffers, stale revision, brak/extra descriptor, download artifacts |
| UI reducer | replace/reset/undo/redo, niezależne sloty, stale source |
| Browser integration | real WASM Worker, preview source/edited, build gate |
| Negative security/limits | malformed PNG/JPEG, decompression bomb limits, 64 MiB boundary |
| Owner proof | exact candidate w Toolset i ten sam lineage w NWN |

## 14. Poza zakresem pierwszego wdrożenia

- edycja geometrii albo UV vertex-by-vertex;
- automatyczny quality unwrap złożonego statku;
- Substance Painter-like warstwy i pędzle;
- normal/specular/metallic/roughness runtime bez osobnego kontraktu;
- TXI/MTR/cutout bez Aurora First proofu;
- modyfikacja źródłowego GLB;
- automatyczne uznanie preview Three.js za proof Aurory.

## 15. Bramka rozpoczęcia implementacji

Punkty 1+2 są `READY_FOR_IMPLEMENTATION_PLAN_EXECUTION`, gdy właściciel
zaakceptuje trzy decyzje zakresowe:

1. V1 wejścia: PNG/JPEG, wynik: TGA;
2. V1 materiału: finalny diffuse, bez PBR i bez TXI/MTR;
3. alpha: `OPAQUE_ONLY` albo jawne `FLATTEN_ALPHA`.

Punkt 3 pozostaje `DESIGN_REQUIRED`. Punkt 4 pozostaje `OUT_OF_MVP`.
