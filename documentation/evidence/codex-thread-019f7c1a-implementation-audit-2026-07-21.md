# Audyt tasku `019f7c1a-98e4-75d3-88b1-e579dcd64e77`

Data: 2026-07-21  
Status: `IMPLEMENTED / OFFLINE GREEN / R27 SAVED_VERIFIED / LIVE VISUAL PROOF MISSING`  
Zakres: ustalenie następnej minimalnej implementacji dla statycznego M0,
bez sterowania Aurora Toolset/NWN i bez modyfikowania ich konfiguracji.

## Zamknięcie implementacyjne

Po audycie wdrożono wszystkie wskazane poprawki offline:

- każdy local-animation header dostaje `type=5` pod `+0x6c` oraz zerowy
  padding `+0x6d..+0x6f`;
- readback rozdziela jawne `animation_type: u8` i
  `animation_type_padding: [u8; 3]`; `+0x68` pozostaje osobnym opaque `u32`;
- oba własne profile niepustej direct-creature geometry emitują
  `meshType=3`;
- writer odrzuca globalne kolizje nazw `creature.nodes` i generowanych
  `m2a_seg_<segment_id>` po ASCII case-fold błędem
  `M4-NODE-NAME-DUPLICATE`;
- `SkinMeshDummy` pozostał bez zmian zgodnie z korektą corpusową.

Zielone testy celowane: `mdl_writer` 31/31, `model_pipeline` 14/14 oraz
`binary_m0_vertical_slice_module` 2/2. Następny krok pozostaje live proofem
jednego nowego kandydata M0 w kolejności Aurora -> NWN.

## Stan kandydata r27 po implementacji

Kandydat `r27` zostal zmaterializowany w
`proof-output/m0-r27-animation-type5-20260721/` z zachowaniem dokladnie tego
samego GLB, tekstury, `appearance.2da`, geometrii, kontrolerow i resrefu modelu
co r26.

- MDL r27: `bf7864f0ed541ed93c4075d167f4a2392eeb6ef37b4cdf444cbb74efd5731578`,
  153032 bajty;
- HAK r27: `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd`,
  13124438 bajtow;
- binary diff r26 -> r27: dokladnie siedem bajtow `00 -> 05`, po jednym pod
  `GeometryHeader + 0x6c` dla `cappear`, `cpause1`, `cwalk`, `crun`,
  `ca1slashl`, `cdamagel` i `cdead`; pozostale 153025 bajtow sa identyczne;
- semantic readback: wszystkie siedem clipow ma `animationType=5`,
  `animationTypePadding=[0,0,0]` i `runtime68=0`.

HAK zostal dolaczony do tego samego `m2a_m0r25.mod`. Natywny build zakonczyl
sie pojedynczym wynikiem `No errors found`, przy wylaczonym `Build` i aktywnym
`Done`. Po `Done` i jednym native Save dwa niezalezne readbacki potwierdzily
ordered HAK list:

```text
m2a_m0r27
m2a_m0r26
m2a_m0r21
```

Po autoryzowanym, targetowanym zamknieciu wlasnego PID modul ma 27898 bajtow,
SHA-256 `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257`,
a offline IFO readback potwierdza te sama ordered HAK list. To zamyka stan
`saved_verified` bez dotykania `nwtoolset.ini`.

Swiezy visual proof r27 pozostaje `proofCompleteness=missing`, a nie
`modelVisibility=not_visible`. Publiczny manager Toolsetu nie ma profilu
wiazacego dokladnie `m2a_m0r25.mod`; jego preflight odrzucil modul kodem
`UNAPPROVED_MODULE_PATH`. Nie wykonano recznego startu, profilu innego modulu
ani nowego adaptera. Do czasu dodania zatwierdzonego profilu nie wolno
przypisywac r27 werdyktu widocznosci w Aurora lub NWN.

Runtime zostal przygotowany bez lokalnego launchera: exact-hash profil
`proof-output/m0-r27-animation-type5-20260721/aur-s07-runtime-profile.json`
oraz jego `native-geometry-gate.json` przechodza centralny AUR-S07 `dry-run`.
Profil wiaze SHA modulu, Area `m2a_m0a25`, entry `(10,10,0)`, fixture
`m2a_m0p01` / Appearance 848 przy `(10,14.5,0)` i trzy HAK-i z hashami.
Centralny kontrakt nadal wymaga jednak user-owned `Test Module` action i sam
nie uruchamia NWN; request nie zostal uzbrojony bez gotowej sesji Toolsetu.

Pelne `cargo test --workspace` ujawnia dwa istniejace, niezalezne od MDL
niepowodzenia GFF (`phase_nine_field_indices_layout_precedes_...` i
`swapped_field_indices_are_in_bounds_...`). Celowane suite'y MDL/M0, wasm,
`cargo check --workspace` oraz webowy typecheck sa zielone.

## Werdykt

Następną implementacją dla bieżącego M0 powinien być wyłącznie brakujący
`local-animation type=5` w każdym binarnym nagłówku lokalnej animacji pod
`GeometryHeader + 0x6c`.

Aktualny writer nie zapisuje tego pola, więc wyzerowany bufor pozostawia
`type=0` we wszystkich siedmiu stanach M0. Jest to mocny kandydat na różnicę
Aurora/NWN: r26 jest widoczny w Aurora, ale nie jest widoczny w świeżym teście
NWN, a jego jedyna istotna eksperymentalna zmiana — identity controllers w
`cpause1` — nie naprawiła runtime.

Nie należy teraz usuwać `SkinMeshDummy`, dodawać kolejnych identity
controllers ani zmieniać geometrii, Appearance, pozycji fixture'a, kamery,
rootów, tekstur lub konfiguracji NWN.

## Zweryfikowany łańcuch dowodowy

1. Task wykonał bounded read-only sweep `cep3_core1.hak`: wszystkie `24 707`
   osiągalne binary local-animation headers w `1 182` modelach mają bajt
   `type=5` pod `+0x6c`.
2. Dekompilacja Aurory kopiuje dokładnie jeden bajt spod `+0x6c` do stanu
   runtime. Nie traktuje pola `+0x68` i wysokich trzech bajtów jako tej samej
   wartości.
3. xoreos niezależnie czyta `uint8 type`, pomija trzy bajty paddingu, a potem
   czyta `length` i `transition`.
4. Kod produkcyjny `emit_animations()` zapisuje name, root, node count,
   length, transition i animroot, ale nie zapisuje `animation.header + 0x6c`.
5. Własny semantic readback i test writera obecnie oczekują
   `runtime_6c == 0`; jest to samopotwierdzający się błąd kontraktu.
6. Świeże artefakty r26 rozdzielają wynik wizualny:
   - Aurora: `modelVisibility=visible`, capture SHA-256
     `01941f4fde610354e3e6c94fbcc91b40f507a83d4c09ab4d6cd8510882886a05`;
   - NWN: `modelVisibility=not_visible`, capture SHA-256
     `07254e07cfbba1310acd7635565bdf39e54618438cc006883f7da9e1a2ac986a`;
   - log runtime potwierdza `Loading Module: m2a_m0r25`, SHA-256
     `54ddfa3b5baa95bd79b825c37acf08ef9c7e69b6dd2da1d213b2baaaefe241768`.
7. MDL r26 z identity controllers ma SHA-256
   `b177f450f68a2e762b8c15886be17b4aaf7ccfe71da14d5ad0db9df8c5d01b77`,
   a HAK r26 ma SHA-256
   `6f805873420b5279b45dadc26d913fa7692374df9e0582c23b42c8a4b42d6bb0`.
   Wynik NWN odrzuca identity-controller-only jako wystarczającą poprawkę.
8. Ostatni pass eliminacyjny tasku potwierdził, że M0 ma classification `4`
   jak native `c_squirrel`, każdy `animroot` odpowiada nazwie modelu, wszystkie
   `4 194 304` piksele tekstury mają alpha `255`, a mesh ma `render=1`,
   `transparency=0` i `render_hint=0`. Te pola nie są uzasadnionymi celami
   następnej zmiany.

`proofCompleteness` runtime pozostaje osobnym polem i nie jest pełne, ponieważ
aktywny Toolset blokuje czysty odczyt hasza MOD oraz nie ma zamkniętego packetu
AUR-S07. Nie zmienia to świeżego, kandydat-bound wyniku
`modelVisibility=not_visible` w NWN i nie wolno mieszać tych dwóch ocen.

## P0 — implementować teraz, test najpierw

### 1. Testy RED

W `crates/m2a-core/tests/mdl_writer.rs`:

- zmienić oczekiwanie dla lokalnego clipu z `(runtime_68, runtime_6c) == (0,0)`
  na `(0,5)`;
- dla M0 jawnie sprawdzić wszystkie siedem clipów: `cappear`, `cpause1`,
  `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `cdead`;
- sprawdzić surowe bajty `+0x6d..+0x6f == [0,0,0]`, aby padding pozostał
  deterministyczny;
- dodać negatywną mutację tylko `animation.header + 0x6c: 5 -> 0` i wymagać
  różnicy `animations[0].header`. Istniejący test pola `+0x68` musi pozostać
  niezależny.

W testach semantic readback/policy verifiera oczekiwany profil own-output ma
odrzucać `runtime_6c != 5`.

### 2. Minimalna zmiana produkcyjna GREEN

W `crates/m2a-core/src/mdl/write_binary_mdl.rs`, w `emit_animations()`, dodać
dokładnie:

```rust
write_u32(core, animation.header + 0x6c, 5)?;
```

Zapis `u32(5)` daje bytes `05 00 00 00`: poprawny `type=5` i trzy zerowe
bajty paddingu. Nie kopiować natywnego przykładu `05 7f 00 00`; Aurora nie
kopiuje paddingu, a własny output ma być deterministyczny.

W `crates/m2a-core/src/mdl/semantic_readback.rs` zmienić own-output invariant z
`actual_clip.runtime_6c != 0` na `actual_clip.runtime_6c != 5`.

Na tym etapie `runtime_6c: u32` może pozostać typem przejściowym. Rozdzielenie
go na `animation_type: u8` i trzy bajty paddingu jest późniejszym refaktorem,
nie warunkiem minimalnej poprawki.

### 3. Weryfikacja offline

Uruchomić co najmniej:

```text
cargo test -p m2a-core --test mdl_writer
cargo test -p m2a-core --test model_pipeline
cargo test -p m2a-core --test binary_m0_vertical_slice_module
```

Następnie semantic diff nowego kandydata względem r26 ma wykazać tylko zmianę
czterech bajtów pod `+0x6c` w siedmiu nagłówkach local-animation. Rozmiar,
offsety, geometria, kontrolery, tekstury i fixture nie powinny się zmienić.

### 4. Weryfikacja live po GREEN

Świeży wynik r26 w NWN dopuszcza jedną następną iterację. Nowy kandydat powinien
zachować ten sam moduł, Area, obiekt, `Appearance_Type=848`, ordered HAK
lineage, entry `(10,10,0)` i fixture `(10,14.5,0)`; zmienić wolno tylko MDL,
HAK/manifest i ich hashe wymagane przez nową rewizję.

Kolejność proofu:

1. clean readback artefaktów i mapowania zasobów;
2. exact object selection/readback w Aurora i świeży capture `TScrollBox`;
   `Focus on Object` oraz kadrowanie są opcjonalne;
3. jeżeli model jest widoczny w Aurora, test dokładnie tej samej linii w NWN;
4. świeży capture NWN i log z dokładną nazwą modułu;
5. osobno zapisać `modelVisibility` i `proofCompleteness`.

## Osobny P0 — M4/H1, nie mieszać z M0

Task potwierdził również, że każda z `77 552` niepustych geometrii i `1 437`
niepustych skinmeshy w badanym corpusie ma `meshType=3`. Wszystkie `218`
przypadków `meshType=0` to puste node'y.

Dlatego profil `M4DirectCreatureExtended64V1`, który obecnie wybiera `0`,
powinien dostać osobną zmianę test-first `0 -> 3`. M0 już emituje `3`, więc
łączenie tej zmiany z następnym testem M0 zaciemniłoby A/B i jest zabronione.

## P1 — po zamknięciu P0

- Dodać globalną, case-insensitive unikalność nazw node'ów jako gate własnego
  outputu. To hardening loader compatibility, nie obecna diagnoza M0.
- Później rozdzielić parserowe `runtime_6c: u32` na `animation_type: u8` i
  jawny padding, zachowując lossless readback.

## Nie implementować na podstawie tego tasku

- Nie usuwać `SkinMeshDummy`. Pełny sweep znalazł `3 015` natywnych,
  bezdzieciowych i bezkontrolerowych node'ów `0x01` o nazwie base-skina —
  dokładnie kształt obecnego dummy. Dodatkowo istnieje `1 211` natywnych
  state-skin node'ów `0x61`; to osobny profil, nie uniwersalny obowiązek.
- Nie dodawać dalszych identity controllers do `cpause1` ani pozostałych
  stanów; r26 przetestował tę hipotezę w runtime i model nadal nie był
  widoczny w NWN.
- Nie zmieniać `classification`, geometry root, `animroot`, node flags,
  alpha, tekstury, `Appearance_Type=848`, resrefów, tile'a, entry, pozycji
  fixture'a ani kamery w tej iteracji.
- Nie dotykać `nwtoolset.ini`, konfiguracji NWN ani instalacji gry.
- Nie łączyć `type=5`, M4 `meshType=3`, zmian skinów i nazw node'ów w jednym
  kandydacie.

## Dług dokumentacyjny ujawniony przez audyt

W `documentation/nwn-model-visibility-research-synthesis-2026-07-21.md`
sekcja 41 nadal opisuje wcześniejszą rekomendację usunięcia `SkinMeshDummy`,
ale późniejsza sekcja 41a ją poprawia. W
`documentation/macierz-gotowosci-wiedzy-codex.md` główny wiersz „Binary MDL
writer” również nadal zawiera starą rekomendację, mimo że niżej znajduje się
korekta corpusowa. Przed rozpoczęciem prac nad skinem trzeba ujednolicić te
podsumowania, aby stara hipoteza nie została przypadkiem wdrożona.

## Kolejność implementacyjna

1. P0 M0: `local-animation type=5`, osobny minimalny kandydat i proof
   Aurora -> NWN.
2. Jeżeli NWN nadal nie pokaże modelu: nowa diagnoza oparta na tym świeżym
   wyniku, bez automatycznego przechodzenia do kolejnego eksportu.
3. Osobno P0 M4/H1: `meshType 0 -> 3`.
4. P1: unikalność nazw node'ów i parserowy refactor type/padding.
5. Skin profiles dopiero po osobnym, model-specific kontrakcie; bez usuwania
   obecnego dummy na podstawie obalonej hipotezy.

## Źródła lokalne

- Task Codex: `codex://threads/019f7c1a-98e4-75d3-88b1-e579dcd64e77`.
- `documentation/nwn-model-visibility-research-synthesis-2026-07-21.md`,
  szczególnie sekcje 41a, 44 i 45.
- `documentation/macierz-gotowosci-wiedzy-codex.md`.
- `crates/m2a-core/src/mdl/write_binary_mdl.rs`.
- `crates/m2a-core/src/mdl/semantic_readback.rs`.
- `crates/m2a-core/tests/mdl_writer.rs`.
- `crates/m2a-core/tests/model_pipeline.rs`.
- `proof-output/m0-r26-hak-on-r25-aurora-20260721/`.
- `proof-output/m0-r26-hak-on-r25-nwn-20260721/`.
