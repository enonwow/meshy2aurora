# Audyt: dlaczego wygenerowanego creature nie widać w NWN

Data: `2026-07-25`

Status: `AUDIT_CONTINUED / R43_INSTALLED_HASH_VERIFIED_NOT_LOADED /
R42_TYPE0_ROOT_LAYOUT_ISOLATED_FOR_OWNER_A_B / NO_R44_ADMITTED`

## 0. Werdykt pewny po drugim przebiegu

Na poziomie aktualnego porównania „placeable widać, creature nie widać”
przyczyna jest rozstrzygnięta:

> NWN uruchomił aktualne moduły placeable, ale nie uruchomił najnowszego
> creature `r43`. W chwili tej kontroli exact `m2a_h2r43.mod` i
> `m2a_h2r43.hak` nie istniały w natywnych katalogach NWN; później zostały
> zainstalowane byte-for-byte, lecz nadal nie ma żadnego loadu r43.

Read-only kontrola wykonana `2026-07-25`:

| Obiekt | Stan natywny / runtime |
|---|---|
| `modules\m2a_h2r43.mod` | zainstalowany po pierwszym odczycie; 20 280 B; SHA-256 `ef2b48b80e4c01b455a04246c8e1da8f839171d9e9b4f9e835b846738f0a0c89`; jeszcze niezaładowany |
| `hak\m2a_h2r43.hak` | zainstalowany po pierwszym odczycie; 20 084 361 B; SHA-256 `d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1` |
| `modules\m2a_h2r42.mod` | obecny stary, Toolset-repacked r42; 40 865 B; SHA-256 `fe2408754ca10b71998aa64fe5cd8b1cd9347b335980db79f58b01472730da99` |
| `hak\m2a_h2r42.hak` | obecny stary r42; 20 085 291 B; SHA-256 `6d50bd1f784a8d1b10b36b09e32565781674def6b4f4367d803ac7acced6da69` |
| bieżący `nwclientLog1.txt` | tylko `m2a_s1_plc_mod`, `m2a_p20k_mod`, `m2a_s1_col_mod`, `m2a_s1_c2_mod` |
| bieżący `nwengineLog.txt` | te same cztery loady placeable; brak r43 |

Kanoniczny r43, obecnie również zainstalowany:

- MOD SHA-256:
  `ef2b48b80e4c01b455a04246c8e1da8f839171d9e9b4f9e835b846738f0a0c89`;
- HAK SHA-256:
  `d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1`.

To daje pewność, że obecna obserwacja nie jest wynikiem r43 i nie może
świadczyć o jego sukcesie ani porażce. W praktyce porównywany jest uruchomiony
S1 placeable ze starym r42 albo z brakiem jakiegokolwiek aktualnego testu
creature.

Osobna kwestia to przyczyna niewidoczności starego r42. Tego konkretnego
modelowego root cause nie da się jeszcze nazwać z taką samą pewnością:

- r42 miał dodatkowy controllerless root, którego nie ma runtime-draw witness
  H1;
- r43 usuwa ten root i odtwarza strukturę H1: 25 base node'ów, root z
  `position+orientation`, rigid mesh bezpośrednio pod rootem i 7 stanów type 0
  po 24 node'y z rootowymi `position+orientation+scale`;
- r43 nie został jednak uruchomiony, więc ta korekta pozostaje nieprzetestowanym
  A/B, a nie potwierdzonym fixem.

Wniosek o pewności:

1. **Pewne:** dotychczasowe porównanie nie testowało latest creature; r43 jest
   już zainstalowany, ale nie został załadowany.
2. **Pewne:** kolizja PWK placeable nie ma z tym związku.
3. **Nieudowodnione:** dokładny bajt/pole MDL odpowiedzialne za historyczną
   niewidoczność r42.
4. **Rozstrzygający gate:** instalacja exact r43 została wykonana; pozostał
   jeden owner-run tej samej sceny ze stock Hook Horrorem.

## 1. Odpowiedź wykonawcza

Nie ma dziś dowodu na jeden konkretny wadliwy bajt, który można uczciwie
nazwać przyczyną. Zebrane dowody pozwalają jednak mocno zawęzić problem:

> Wspólny parser i podstawowy writer binary MDL działają. Awaria leży po
> stronie ścieżki specyficznej dla creature: wygenerowanej instancji
> UTC/GIT albo runtime'owego użycia hierarchii, stanów animacji i transformacji
> custom MDL/MDX.

Sukces placeable odrzuca globalną awarię:

- importu GLB;
- podstawowej serializacji `trimesh`;
- nagłówka `binary MDL + raw MDX`;
- tekstury TGA;
- HAK-a i lookupu zasobu typu `2002`;
- samego renderera sztywnej geometrii.

Nie odrzuca natomiast błędu creature, ponieważ placeable nie przechodzi przez:

- `appearance.2da -> MODELTYPE -> RACE`;
- UTC i `GIT Creature List`;
- creature animation/state routing;
- hierarchię riga i runtime'owe kontrolery;
- creature-specific tworzenie instancji.

Kolizja placeable nie ma związku z tym werdyktem. Render MDL i kolizja PWK są
oddzielnymi zasobami oraz oddzielnymi torami.

Najważniejsze ograniczenie dowodowe: najnowszy dokładny creature `r43` jest już
zainstalowany, ale nie został uruchomiony. Nie istnieje więc świeży wynik
`not_visible` dla r43. Wcześniejszy `r42` nie izoluje stockowego control,
ponieważ raport nie ma runtime capture, Hook Horror stał około `33.7°` poza
osią startowego widoku, a trzeci blueprint ponownie używał customowego
`Appearance_Type=15100`.

## 2. Granice audytu

- Nie uruchamiano, nie przejmowano i nie kontrolowano Aurora Toolset ani NWN.
- W trzecim przebiegu zainstalowano wyłącznie exact r43 MOD/HAK przez
  create-new/no-clobber i potwierdzono byte-for-byte; niczego nie nadpisano.
- Nie utworzono nowego `rNN`, resrefu, HAK-a, MOD-a ani payloadu modelu.
- Wykonano wyłącznie inspekcję istniejących artefaktów, proofów, kodu,
  dekompilacji i read-only corpusu.
- Wynik nie deklaruje wizualnego sukcesu ani porażki r43.
- Obowiązuje human-owned final proof oraz bramka zakazująca `r44` przed
  świeżym, związanym z r43 wynikiem właściciela.

## 3. Klasy dowodów

| Oznaczenie | Znaczenie |
|---|---|
| `OWNER_PROOF` | Świeży wynik wizualny dostarczony przez właściciela i związany z dokładnym kandydatem. |
| `PROJECT_FACT` | Fakt z kodu, własnego parsera/readbacku albo wygenerowanego artefaktu. |
| `DECOMP_FACT` | Fakt z lokalnej dekompilacji Aurory. |
| `RETAIL_FACT` | Fakt z lokalnego, read-only zasobu retail lub dozwolonego corpusu. |
| `INFERENCE` | Wniosek wsparty wieloma faktami, lecz jeszcze bez rozstrzygającego proofu. |
| `HYPOTHESIS` | Możliwa przyczyna wymagająca dokładnie nazwanego testu. |

## 4. Co zostało rzeczywiście udowodnione

### 4.1. Widoczny placeable

`OWNER_PROOF`

Dokładny S1 placeable ma:

- Toolset: `modelVisibility=visible`, `proofCompleteness=verified`;
- NWN: `modelVisibility=visible`, `proofCompleteness=verified`.

Źródła:

- `documentation/evidence/s1-placeable-ritual-pedestal-p1-ready-for-owner-proof-2026-07-25.md`;
- `documentation/evidence/s1-placeable-ritual-pedestal-p1-owner-visual-result-2026-07-25.json`;
- `documentation/evidence/s1-placeable-ritual-pedestal-p1-owner-nwn-visible-2026-07-25.png`.

### 4.2. Potwierdzone porażki creature

`OWNER_PROOF`

| Kandydat | Toolset | NWN | Co zmieniał |
|---|---|---|---|
| r32 | `visible/verified` | `not_visible/verified` | pełna 70-polowa instancja GIT i 67-polowy UTC po naprawie sparse envelope |
| r40 | `visible/verified` | `not_visible/verified` | 20 sztywnych grup trójkątów, 7 stanów type 5 z samym rigiem |
| r41 | zachowany wcześniejszy pozytywny tor Toolset | `not_visible/verified` | te same 20 meshy oraz pełne 45-node'owe drzewa stanów type 5 |
| r42 | `visible/verified` | `not_visible/failed` | nowy H2, jeden rigid mesh, 7 stanów type 0, stock Hook Horror w tej samej scenie |

R40 i r41 są mocnymi wynikami custom-model `not_visible`. R42 jest słabszy
jako izolacja MDL: właściciel zgłosił brak obu modeli, a runtime nie ma
zaakceptowanego obrazu. Dodatkowo ostatni load użył przepakowanego przez
Toolset MOD-a, nie kanonicznego 20 KB źródła.

Źródła:

- `documentation/evidence/m0-r32-corrected-container-iteration-2026-07-22.md`;
- `documentation/evidence/m0-r40-owner-visual-result-2026-07-24.json`;
- `documentation/evidence/m0-r41-owner-nwn-visual-result-2026-07-24.json`;
- `documentation/evidence/h2-r42-owner-toolset-nwn-visual-result-2026-07-24.json`;
- `documentation/evidence/h2-r43-deep-model-visibility-audit-2026-07-24.md`.

### 4.3. R43 nie ma negatywnego wyniku

`PROJECT_FACT`

Dokładne źródła r43:

| Artefakt | Bajty | SHA-256 |
|---|---:|---|
| `m2a_h2r43.mod` | 20 280 | `ef2b48b80e4c01b455a04246c8e1da8f839171d9e9b4f9e835b846738f0a0c89` |
| `m2a_h2r43.hak` | 20 084 361 | `d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1` |
| `m2a_h2p43.mdl` | 599 784 | `5e169877c2f66d68f7bc8cae58e73087f346ff51b5652b37bc787e59c4c38d2a` |

Pierwsza read-only kontrola z `2026-07-25` potwierdziła brak:

- `...\modules\m2a_h2r43.mod`;
- `...\hak\m2a_h2r43.hak`.

Zatem aktualne osie r43 pozostają:

- Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`.

Według aktualnej decyzji właściciela status `ready_for_owner_proof` wymaga
wcześniejszej instalacji dokładnego MOD/HAK i potwierdzenia byte-for-byte.
Historyczny handoff r43 deklarujący `ready_for_owner_proof` był w tym momencie
niekompletny. Brak został później zamknięty instalacją i post-copy hash checkiem
opisanym w sekcji 12.

## 5. Aurora First: wspólny parser, różni konsumenci

`DECOMP_FACT`

Lokalna dekompilacja pokazuje wspólną ścieżkę:

- `FUN_00a546cc` tworzy instancję na podstawie wspólnego modelu;
- `FUN_00a5de94` obsługuje cache i ładowanie;
- `FUN_00a3b260 -> FUN_00a3b3ec -> FUN_00a3b4b4` parsuje binary MDL;
- `FUN_00a3b994` odtwarza wspólne drzewo node'ów.

W tych funkcjach nie ma argumentu `creature/placeable/tile`.

Różnica powstaje przed i po wspólnym parserze:

- creature rozwiązuje `appearance.2da`, `MODELTYPE`, `RACE`, części i
  runtime'owe stany;
- placeable rozwiązuje `placeables.2da.ModelName` i zwykle instancjuje jeden
  statyczny model.

Pełne mapowanie funkcji i offsetów:
`documentation/audyt-wspolnego-pipeline-parserow-mdl-creature-placeable-tile-2026-07-25.md`.

`INFERENCE`

Sukces placeable dowodzi, że parser binary MDL i podstawowy `TriMesh` są
akceptowane przez NWN. Nie dowodzi, że creature assembler i state machine
zaakceptują ten sam model w tej samej postaci.

## 6. Dokładne porównanie binary MDL

Własny `inspect_binary_mdl` został uruchomiony na czterech istniejących,
niezmienionych modelach:

| Cecha | S1 placeable, widoczny | H2 r43, nieprzetestowany | H1 v20, tylko corrupt-draw witness | `c_Direwolf`, retail |
|---|---:|---:|---:|---:|
| bajty | 137 244 | 599 784 | 557 268 | 374 808 |
| base nodes | 2 | 25 | 25 | 30 |
| głębokość | 1 | 7 | 7 | 6 |
| renderowalne meshe | 1 | 1 | 1 | 24 |
| wierzchołki | 2 224 | 2 678 | 1 334 | 644 |
| trójkąty | 1 477 | 1 543 | 1 556 | 472 |
| meshe z vertex colors | 1 | 0 | 0 | 24 |
| `render=0` | 0 | 0 | 0 | 0 |
| `geometryType` | 2 | 2 | 2 | 2 |
| classification | 4 | 4 | 4 | 4 |
| fog | 1 | 1 | 1 | 1 |
| supermodel | `NULL` | `NULL` | `NULL` | `NULL` |
| animacje lokalne | 0 | 7 | 7 | 42 |
| typy animacji | — | 7 × type 0 | 7 × type 0 | 42 × type 5 |

Wnioski:

1. Liczba trójkątów nie wyjaśnia braku H2. Widoczny placeable ma prawie tę
   samą złożoność co H2.
2. `geometryType`, classification, fog, `render` i supermodel są zgodne.
3. SkinMesh nie jest globalnym warunkiem draw: H1 v20 i retailowe sztywne
   direct-creature stanowią przeciwprzykłady.
4. Brak vertex colors nie jest samodzielnym warunkiem całkowitego braku draw,
   ponieważ H1 v20 bez kolorów wywołał draw szarego, zdeformowanego blobu.
5. Największa funkcjonalna różnica między widocznym placeable a H2 leży w
   hierarchii oraz siedmiu runtime'owych stanach creature.

### 6.1. Różnice w naszym writerze

`PROJECT_FACT`

Creature i placeable używają tego samego `AuroraModelIrV1`,
`inspect_binary_mdl` i `write_binary_mdl`.

Profil `PlaceableStaticRigidNativeV1` różni się od sztywnego profilu creature
głównie tym, że:

- emituje vertex colors;
- wylicza model bounds bezpośrednio z geometrii;
- w tym vertical slice nie emituje lokalnych animacji;
- dostaje prostą hierarchię root -> mesh.

Oba profile emitują `meshType=3`. R40/r41/r43 używają retailowego envelope
model bounds `[-5,-5,-1]..[5,5,10]`, radius `7`, czyli dokładnie takiego jak
badany `c_Direwolf`. Bounds nie są obecnie sensownym kandydatem root cause.

Źródło:
`crates/m2a-core/src/mdl/write_binary_mdl.rs`.

## 7. Co zostało odrzucone jako samodzielna przyczyna

### 7.1. Brak kolizji / PWK

`REJECTED`

Placeable był widoczny przed dodaniem PWK. PWK odpowiada za blokowanie ruchu i
use points, nie za draw MDL.

### 7.2. Ogólna awaria binary MDL, TriMesh, MDX lub TGA

`REJECTED`

S1 placeable przeszedł NWN na tym samym wspólnym parserze i writerze.

Nie odrzuca to bardziej szczegółowego błędu creature state/transform albo
konkretnej odmiany MDX pod hierarchią creature.

### 7.3. Zbyt duża liczba polygonów

`REJECTED`

S1: `1477` trójkątów, H2: `1543`. Różnica jest zbyt mała, by tłumaczyć
całkowite zniknięcie przy działającym wspólnym writerze.

### 7.4. Zły `MODELTYPE=S`, `RACE` albo brak modelu/tekstury w HAK

`REJECTED_AS_SOLE_CAUSE`

Łańcuchy r32-r43 przechodzą własny readback:

`GIT Appearance_Type -> appearance.2da row -> MODELTYPE=S -> RACE -> MDL -> TGA`.

Aurora pokazuje dokładny custom model, a nie stockowy fallback. R40/r41 używają
jednego HAK-a. Nie znaleziono kolidującego loose modelu lub `appearance.2da`.

### 7.5. Zła baza `appearance.2da`

`REJECTED_AS_SOLE_CAUSE`

Test z pełnym, zachowanym byte-for-byte prefixem Last City nadal zakończył się
`Toolset visible / NWN not_visible`.

Źródło:
`documentation/evidence/m0-last-city-appearance-diag02-runtime-result-2026-07-22.md`.

### 7.6. Sparse UTC/GIT

`FIXED_BUT_NOT_SUFFICIENT`

Pierwszy generator rzeczywiście nie emitował `MaxHitPoints=13` i pełnej
28-elementowej `SkillList`. R32 naprawił tę wadę, ale dokładny r32 nadal był
niewidoczny w NWN. Sparse envelope nie jest więc bieżącym pełnym wyjaśnieniem.

### 7.7. Sam typ animacji 0 lub 5

`REJECTED_AS_SOLE_CAUSE`

- r40/r41: type 5, niewidoczne;
- r42: type 0, wynik niewidoczny, lecz confounded;
- H1 v20: type 0, silnik wykonał corrupt draw;
- retail: type 5, działa.

Typ nagłówka nie wystarcza do rozstrzygnięcia.

### 7.8. Sam brak vertex colors

`REJECTED_AS_SOLE_CAUSE`

Retail i widoczny placeable je mają, ale H1 v20 bez nich został narysowany.
Kolory mogą wpływać na shading lub poprawność, lecz nie są samodzielnym
warunkiem wydania draw call.

### 7.9. Brak SkinMesh albo supermodelu

`REJECTED_AS_UNIVERSAL_REQUIREMENT`

Badane działające rodziny direct creature mają legalne sztywne meshe i
`supermodel=NULL`.

## 8. Ranking pozostałych przyczyn

### P0 — brak czystego A/B dla aktualnego kandydata

Pewność: wysoka.

To jest obecny blocker dowodowy, nie przyczyna techniczna.

R43 zawiera w jednej kanonicznej scenie:

- custom H2 na row `15100`, pozycja `(10,14.5,0)`;
- stock Hook Horror na row `102`, pozycja `(7,14.5,0)`;
- wejście gracza `(10,10,0)`, kierunek `+Y`.

R43 nie został uruchomiony. R42 został przepakowany przez Toolset, wejście
gracza przesunęło się z `(10,10,0)` do około `(10.0944,6.96065,0)`, a właściciel
zgłosił brak także stockowego controlu. Ten wynik nie izoluje MDL.

### P1 — creature state/hierarchy/transform path

Pewność: średnio wysoka, warunkowa.

`INFERENCE`

To najsilniejsza techniczna hipoteza po sukcesie placeable:

- Toolset potrafi pokazać base tree;
- NWN creature wybiera i stosuje lokalne stany;
- placeable nie ma tego toru;
- H1 v20 daje zdeformowany blob, co wskazuje na realny wpływ transformacji;
- r40/r41 zmieniały topology state tree, lecz nie wszystkie semantyki
  kontrolerów, routingu i runtime'owego przejścia stanu;
- r43 odtwarza strukturalną rodzinę H1, ale nie został przetestowany.

Ta hipoteza obejmuje nie jeden header, lecz wspólną granicę:

`base hierarchy + state hierarchy + controllers + raw MDX transforms`.

### P2 — wygenerowana instancja creature lub runtime GFF

Pewność: średnia.

`HYPOTHESIS`

R32 potwierdził kompletność obecnego envelope względem własnego parsera i
porównania z Toolset-authored strukturą. Nie ma jednak świeżego, czystego
runtime proofu, w którym po tej naprawie stockowy `c_horror` jest widoczny z
tej samej wygenerowanej rodziny UTC/GIT.

Istniejący, już zainstalowany diagnostyczny:

- MOD: `m2a_van02.mod`;
- Toolset module name: `Meshy2Aurora creature comparison`;
- Area display name: `Meshy2Aurora M0 binary vertical-slice area`;
- Area resref: `m2a_vana02`;
- stock model: `c_horror`;
- fixture: `M2A Hook Horror clone control`;
- placement: `(10,14.5,0)`;

izoluje właśnie tę granicę. Jego source/installed MOD i HAK są nadal
byte-identical:

- MOD SHA-256:
  `fd7b047062bd7262958c67bc3d0988d7e38a7ca23098abdd50f95f3806930a91`;
- HAK SHA-256:
  `d2f968decdd3d0ad4cddef6f58d6309743e4943569a876aa97172472aef1d677`.

### P3 — custom binary mesh/MDX jest akceptowany przez Toolset, lecz odrzucany
lub błędnie konsumowany przez runtime creature

Pewność: średnia, warunkowa.

Ten branch staje się główny dopiero wtedy, gdy w dokładnej scenie:

- stock Hook Horror jest widoczny;
- H2 jest nieobecny albo zdeformowany.

Wymuszony read-only test świadków runtime nie znalazł pojedynczego opaque
defaultu wspólnego H1, `c_Direwolf` i `c_horror`, a nieobecnego w historycznym
negatywnym r30. Różnice obejmują kilka rodzin równocześnie: face adjacency,
surface IDs, vertex colors, routine words, materiały, topologię i stany.
Wybór jednego z nich bez czystego A/B byłby zgadywaniem.

### P4 — brakująca liczba lub zestaw nazw animacji

Pewność: niska jako warunek samego draw.

Retail ma 42 stany, a nasze modele 7. H1 v20 również ma tylko 7 i wywołał draw,
więc sama liczba nie wyjaśnia pełnej niewidoczności. Może jednak wpływać na
konkretny runtime state lub poprawność zachowania po utworzeniu creature.

## 9. Rozstrzygający następny test

Nie tworzyć r44 i nie regenerować modelu.

### 9.1. Najpierw domknąć exact r43

Handoff:

1. Exact test-module file: `m2a_h2r43.mod`.
2. Toolset module name: `Meshy2Aurora creature comparison`.
3. Exact Area display name: `Meshy2Aurora M0 binary vertical-slice area`.
4. Area resref: `m2a_h2a43`.
5. Custom H2: środek, row `15100`.
6. Stock Hook Horror: lewa strona, row `102`.

Exact źródłowy MOD i HAK zostały zainstalowane według polityki
absent-target/no-clobber i potwierdzone byte-for-byte. Audyt nie uruchomił
Toolsetu ani NWN.

Interpretacja jednego czytelnego kadru:

| Stock row 102 | H2 row 15100 | Wniosek | Następna praca |
|---|---|---|---|
| brak | brak | nie odizolowano modelu; problem sceny, instancji albo runtime GFF | nie zmieniać MDL; testować istniejący `m2a_van02` i porównać exact runtime container |
| widoczny | brak | custom MDL/MDX creature odrzucony | różnicowy audyt H2 kontra czysty retail whole-model creature |
| widoczny | zdeformowany blob | draw zaakceptowany, błędna geometria/transform | audyt hierarchy/controller/raw-MDX transform |
| widoczny | widoczny poprawnie | visibility gate zamknięty | przejść do SkinMesh/deformacji, nie do r44 |

### 9.2. Jeśli oba r43 creature są nieobecne

Użyć istniejącego, już zainstalowanego `m2a_van02.mod` jako stock-only
diagnostic. Nie jest potrzebny nowy model, HAK ani resref.

- jeśli `c_horror` z wygenerowanego UTC/GIT jest widoczny, r43 miał problem
  sceny/repacku albo modelu;
- jeśli nadal jest nieobecny, głównym celem staje się runtime GFF/instantiation,
  nie writer MDL.

## 10. Wynik wymuszonej weryfikacji offline

Uruchomiono:

```text
M2A_REQUIRE_RUNTIME_WITNESSES=1
cargo test -p m2a-core --test runtime_witness_conformance -- --ignored --nocapture
```

Wynik: `4 passed`.

Test odczytał in-place:

- owned H1 v20;
- `c_Direwolf`;
- retail `c_horror` z dokładnego zakresu BIF;
- rodzinę CEP;
- historyczny negatywny r30.

Wynik differentialu jest jawnie `confounded`: brak jednego renderer defaultu,
który stanowiłby wspólną pozytywną regułę.

Dodatkowo:

```text
cargo test -p m2a-core \
  --test h2_r43_h1_root_layout_candidate \
  --test binary_creature_multi_fixture_module \
  --test placeable_pipeline --quiet
```

Wynik:

- `h2_r43_h1_root_layout_candidate`: `1 passed`;
- `binary_creature_multi_fixture_module`: `5 passed`;
- `placeable_pipeline`: `7 passed`, `1 ignored` jako jawny env-gated test.

## 10A. Drugi przebieg: wykluczenia pole po polu

### `appearance.2da`

Skąpy custom row nie jest samodzielnym root cause. Historyczny
`m0-last-city-appearance-diag02` zachował pełną tabelę Last City jako
byte-identical prefix, a M0 nadal był `not_visible` w NWN. Pełne sklonowanie
samej bazy 2DA nie naprawiło draw.

### GIT/UTC

Dokładny H1 v20, który NWN narysował jako zdeformowany blob, używa
wygenerowanego MOD-a:

- MOD SHA-256:
  `34838f87f65b89c4e05057acfe04531c4f39c6a3675902558de47958f2777fc1`;
- 70 pól GIT;
- `CurrentHitPoints=1`, `MaxHitPoints=1`;
- pusty `SkillList`;
- `FactionID=2`, `WalkRate=0`, `PerceptionRange=0`.

NWN mimo tego wydał draw dla exact H1. Zatem późniejsza normalizacja
`MaxHitPoints=13` i 28-elementowego `SkillList` była poprawą kompletności
kontenera, lecz te dwa pola nie są uniwersalnym wyjaśnieniem braku draw.

Lokalna dekompilacja w `FUN_00532174` wczytuje również `WalkRate`,
`PerceptionRange`, `Interruptable`, HP, `ClassList` i pozostałe pola. Dla
wartości używanych przez generator nie ma w tej funkcji gałęzi odrzucającej
creature ani wyniku błędu; wartości są zapisywane do stanu obiektu.

Ostatni zainstalowany r42 zawiera trzy 70-polowe creature:

| Template | Appearance | HP / MaxHP | Race | Faction | WalkRate |
|---|---:|---:|---:|---:|---:|
| `m2a_h2utc42` | 15100 | `1 / 13` | 0 | 2 | 0 |
| `m2a_h2ctrl42` | 102 | `1 / 13` | 0 | 2 | 0 |
| Toolset `x2_drider003` | 15100 | `80 / 128` | 7 | 1 | 7 |

Trzeci rekord jest pełnym blueprintem Toolsetu, a nie minimalnym rekordem
generatora. Końcowy load r42 nastąpił po zapisaniu tego 40 865-bajtowego MOD-a,
lecz raport właściciela mówi o braku wszystkich oczekiwanych modeli. To
potwierdza, że wynik r42 nie izoluje jednego pola generatora ani samego custom
MDL.

### Struktura MDL: H1 kontra r42/r43

Niezależny readback daje:

| Cecha | H1 draw witness | r42 historyczny | r43 nieprzetestowany |
|---|---:|---:|---:|
| base nodes | 25 | 26 | 25 |
| base controllers | 48 | 48 | 48 |
| root base controllers | position, orientation | brak | position, orientation |
| rigid mesh parent | root | dodatkowy outer root | root |
| nodes/state | 24 | 25 | 24 |
| controllers/state | 49 | 49 | 49 |
| root state controllers | position, orientation, scale | brak | position, orientation, scale |
| animation type | 0 | 0 | 0 |

R43 jest więc rzeczywistym minimalnym A/B struktury H1 względem r42. Sam
readback nie dowodzi jednak zachowania zamkniętego renderera NWN.

### Renderer defaults i vertex colors

Wymuszony differential exact r30-negative kontra H1-positive nie wskazał
jednego renderer defaultu. Co istotne, r30 miał vertex colors dla 2380
wierzchołków, a H1 nie miał ich wcale i mimo to został narysowany. Brak vertex
colors nie jest root cause pełnej niewidoczności.

## 11. Ostateczny werdykt audytu

Na pytanie „dlaczego teraz widzimy placeable, a nie latest creature” odpowiedź
jest pewna: testy z `2026-07-25` załadowały moduły placeable, natomiast exact
r43 nie został jeszcze ani razu załadowany. Obecnie jest już zainstalowany i
hash-verified, więc brakujący krok jest wyłącznie owner-run.

Na inne pytanie — „jaki dokładnie błąd modelowy powodował historyczną
niewidoczność r42” — bieżące dowody nie dają jeszcze pewnej odpowiedzi.
Najbardziej prawdopodobny techniczny obszar awarii to creature-specific
hierarchy/state/transform path, nie ogólny binary MDL ani liczba polygonów.
Nie można jednak jeszcze uczciwie wykluczyć runtime instancji UTC/GIT,
ponieważ po naprawie creature envelope brakuje czystego testu stock-control.

Dokładna odpowiedź brzmi więc:

> Placeable działa, ponieważ używa już potwierdzonego prostego toru
> root -> rigid TriMesh bez creature state machine. Creature w Toolsecie
> pokazuje poprawny base model, ale NWN przechodzi przez dodatkową instancję i
> runtime'owe stany. To w tej dodatkowej warstwie leży awaria lub obecny brak
> dowodu. R43 został przygotowany właśnie po to, aby rozdzielić te możliwości;
> jest już zainstalowany, ale nie został jeszcze przetestowany.

Do świeżego owner-bound wyniku r43:

- nie tworzyć r44;
- nie zmieniać MDL, animacji, vertex colors, 2DA ani GFF na podstawie zgadywania;
- nie traktować r42 jako dowodu „stock visible / custom absent”;
- zachować pozytywny placeable proof jako dowód wspólnego parsera i rigid
  render path.

## 12. Trzeci przebieg: korekta kontroli r42 i instalacja r43

Ta sekcja jest nowsza niż snapshot stanu natywnego w sekcjach 0, 2 i 11.
Stwierdzenia, że r43 nie jest zainstalowany, opisują stan sprzed instalacji
wykonanej później tego samego dnia.

### Co dokładnie pokazał ponowny odczyt r42

Natywny, przepakowany przez Toolset `m2a_h2r42.mod` nadal ma SHA-256
`fe2408754ca10b71998aa64fe5cd8b1cd9347b335980db79f58b01472730da99`
i zawiera trzy creature:

| Obiekt | Appearance | Pozycja | Kąt względem osi startowej kamery |
|---|---:|---:|---:|
| generated H2 | 15100 | `(10.0017, 14.7943)` | około `-0.68°` — praktycznie na osi |
| generated Hook Horror | 102 | `(7.0, 14.5)` | około `-22.32°` |
| Toolset `x2_drider003` | 15100 | `(13.7455, 13.7347)` | około `+28.32°` |

Kąty są policzone od rzeczywistego startu po repacku:
`(10.094443, 6.960651)`, kierunek `+Y`, a nie od kanonicznego `(10, 10)`.
Trzeci rekord nie był stockowym modelem kontrolnym: mimo pełnego blueprintu
Toolsetu ponownie używał customowego `Appearance_Type=15100`. Jedynym stockowym
modelem był Hook Horror, przesunięty o `22.32°` w lewo. Właściciel zgłosił brak
wszystkich oczekiwanych modeli, więc zachowuje on wynik `not_visible`; brak
runtime capture pozostawia jednak `proofCompleteness=failed` i nie pozwala
odtworzyć, które kontrolki były równocześnie czytelne w kadrze. R42 nie daje
zatem czystego rozdzielenia „stock działa / custom nie działa”.

### Co zostało odrzucone po ponownej analizie modelu

- Sam controllerless model root nie jest ogólnie nielegalny: retail
  `c_Direwolf` ma controllerless root i 42 działające stany type 5.
- Konstruktor runtime noda w dekompilacji (`FUN_00a4d318` /
  `FUN_00a4d61c`) inicjalizuje orientację i skalę do identity/`1.0` oraz bit
  aktywności do `1`; brak kontrolera nie zeruje więc automatycznie skali.
- H2 nie ma odwróconego windingu: wszystkie `1543/1543` face records mają
  dodatni iloczyn `cross(v1-v0, v2-v0) · faceNormal`; nie ma NaN ani
  zdegenerowanych trójkątów.
- Wszystkie `1543/1543` face normals zgadzają się z uśrednionymi vertex normals.
- Limit geometrii nie jest przekroczony: H2 ma 1543 trójkąty w jednym meshu,
  podczas gdy obecny limit NWN:EE wynosi 21845 trójkątów na mesh.
- W `cpause1` H2 zachowuje prawidłową wysokość: w każdej z 121 klatek
  najwyższy vertex pozostaje co najmniej `1.8839 m` nad podłożem, a
  `94.7–97.5%` vertexów jest nad `z=0`.

### Najwęższa pozostała różnica r42 → pozytywny H1

Retailowy counterexample odrzuca zdanie „każdy controllerless root jest
błędny”, ale nie odrzuca różnicy specyficznej dla tej pary:

- oba modele używają lokalnych stanów `type=0`;
- pozytywny H1 ma kontrolery `position+orientation` na model root i
  `position+orientation+scale` na root każdego stanu;
- r42 wstawia dodatkowy outer root bez tych kontrolerów, a animowany `Hips`
  staje się jego dzieckiem;
- rigid mesh r42 jest rodzeństwem `Hips`, bezpośrednio pod outer rootem, i nie
  występuje w drzewie stanu;
- r43 usuwa wyłącznie ten dodatkowy poziom i odtwarza dokładny układ
  root/state/mesh pozytywnego H1, zachowując geometrię oraz teksturę H2.

To izoluje błąd r42 do projekcji `type=0` przez dodatkowy outer root. Statyczny
readback nie może jednak zastąpić wykonania zamkniętego renderera; werdykt
przyczynowy staje się wizualnie potwierdzony dopiero, jeśli exact r43 zostanie
narysowany w NWN.

### Instalacja exact r43

`2026-07-25` zainstalowano przez create-new, bez nadpisania istniejących
plików, i zweryfikowano po kopii:

| Natywny plik | Bajty | SHA-256 |
|---|---:|---|
| `modules\m2a_h2r43.mod` | 20 280 | `ef2b48b80e4c01b455a04246c8e1da8f839171d9e9b4f9e835b846738f0a0c89` |
| `hak\m2a_h2r43.hak` | 20 084 361 | `d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1` |

Źródła i cele są byte-for-byte identyczne. Agent nie uruchamiał ani nie
kontrolował Toolsetu/NWN. Exact handoff dla właściciela:

- test-module: `m2a_h2r43.mod`;
- nazwa modułu w Toolset: `Meshy2Aurora creature comparison`;
- Area: `m2a_h2a43`;
- H2 stoi na osi gracza w `(10, 14.5, 0)`;
- wynik r43 pozostaje `modelVisibility=not_tested`,
  `proofCompleteness=missing` do świeżego owner-run.

## 13. Pełny rejestr problemów

Poniższa lista rozdziela problemy produktu, problemy dowodu, naprawione defekty
i odrzucone tropy. „Naprawiony” nie oznacza automatycznie „widoczność
naprawiona”; kilka rzeczy było prawdziwymi defektami, ale po ich korekcie
creature nadal nie było rysowane.

### 13.1. Potwierdzone i nadal otwarte

| ID | Problem | Stan i dowód |
|---|---|---|
| `OPEN-01` | Brak parytetu Toolset → NWN dla custom creature. | r32, r40 i r41: Toolset `visible`, NWN `not_visible/verified`; r42: Toolset `visible`, owner NWN `not_visible`, ale capture `failed`. |
| `OPEN-02` | Exact r43 nie ma żadnego live verdictu. | MOD/HAK są zainstalowane i hash-verified, ale Toolset i NWN pozostają `not_tested/missing`. |
| `OPEN-03` | Nie ma jeszcze poprawnie narysowanego i animowanego outputu creature. | H1 v20 potwierdza jedynie corrupt draw szarego, zdeformowanego blobu; r43 jest rigid visibility candidate, nie końcowym SkinMesh/deformation proofem. |
| `OPEN-04` | Brak czystego runtime rozdzielenia „generated UTC/GIT” kontra „custom MDL”. | W r42 stock `c_horror` używał generated envelope, a pełny blueprint Toolsetu ponownie dostał custom `Appearance_Type=15100`; nie powstał pełny cross-control. |
| `OPEN-05` | Generated stock-only creature po naprawie envelope nie ma owner proofu. | Istniejący i zainstalowany `m2a_van02.mod` izoluje `c_horror` w centrum, lecz nie został przetestowany przez właściciela. |
| `OPEN-06` | R42 ma niepotwierdzony jeszcze live root cause. | Najwęższa różnica to dodatkowy outer root w projekcji `type=0`; r43 usuwa tylko tę różnicę. Dopiero widoczność r43 potwierdzi przyczynowość. |
| `OPEN-07` | Zestaw stanów creature nie ma jeszcze runtime proofu pełnego zachowania. | Legacy V1 nadal ma jawny 7-state fallback dla frozen lineage. Full V2 ma exact 42-state namespace bez idle fallback, controller content/motion gate, rozróżnienie kluczowych semantyk i terminalny `ckdbckdie`; nadal brak owner proofu state routingu, eventów, pętli i jakości wszystkich ruchów. |
| `OPEN-08` | Końcowa deformacja/skin nie została osiągnięta w rendererze. | Full V2 ma już fail-closed CPU conformance dla wielojointowego, nierigid SkinMesh motion w pięciu kluczowych stanach. R40/r41 są rigid-group experiments, a r42/r43 jednym rigid meshem; dopiero hash-bound owner proof kolejnej dozwolonej linii może potwierdzić interpretację pól przez NWN. |
| `OPEN-09` | Nie wolno rozpocząć r44. | Iteration gate wymaga świeżego, candidate-bound wyniku r43; obecnie brak wyniku, nie brak artefaktu. |

### 13.2. Problemy dowodu i konstrukcji testów

| ID | Problem | Stan |
|---|---|---|
| `PROOF-01` | R42 nie ma runtime screenshotu. | Owner verdict `not_visible` jest zachowany, ale `proofCompleteness=failed`; nie można odtworzyć czytelności każdej kontrolki z jednego kadru. |
| `PROOF-02` | R42 nie był czystym stock/custom A/B. | H2 i `x2_drider003` używały row 15100; tylko Hook Horror używał stock row 102, ale z generated GFF. |
| `PROOF-03` | Toolset zmienił dokładny MOD r42. | Kanoniczny MOD miał 20 284 B / SHA `c22741...`; runtime użył repacku 40 865 B / SHA `fe2408...`. Wynik jest związany z repackiem, nie z pierwotnym MOD-em. |
| `PROOF-04` | Repack przesunął start gracza i obiekty. | Start zmienił się na `(10.094443, 6.960651)`; rzeczywiste kąty: H2 `-0.68°`, Hook Horror `-22.32°`, trzeci H2 `+28.32°`. |
| `PROOF-05` | Pierwotny handoff r43 błędnie deklarował gotowość bez instalacji. | Naprawione: exact MOD/HAK zainstalowane create-new i sprawdzone po hashach. |
| `PROOF-06` | Bieżące logi porównywały uruchomione placeable z nieuruchomionym r43. | Naprawione operacyjnie przez instalację, ale nadal nie ma loadu r43; obecna obserwacja nie jest wynikiem r43. |
| `PROOF-07` | Wcześniejszy audyt policzył kąty od starego startu `(10,10)`. | Naprawione w sekcji 12 przez odczyt faktycznego repackowanego IFO. |
| `PROOF-08` | Zamknięty renderer nie pozwala potwierdzić skutku samego readbackiem. | Offline można odrzucać pola i sprawdzać ABI, ale finalny draw/no-draw wymaga owner-run zgodnie z human-owned proof. |

### 13.3. Potwierdzone defekty już naprawione, ale niewystarczające

| ID | Defekt | Korekta i wynik |
|---|---|---|
| `FIXED-01` | Wczesny generated UTC/GIT był sparse: brakowało m.in. `MaxHitPoints=13` i pełnej 28-elementowej `SkillList`. | R32 uzupełnił envelope; creature nadal `not_visible/verified`. |
| `FIXED-02` | Wczesny writer profilu creature zapisywał `meshType=0` mimo trójkątów. | Writer zapisuje `meshType=3`; późniejsze kandydaty nadal bywały niewidoczne. |
| `FIXED-03` | Wczesne local-animation headers miały `type=0` zamiast native-family `type=5`. | Skorygowano dla odpowiedniej rodziny; r40/r41 z type 5 nadal nie były widoczne. H1 pokazuje też, że type 0 nie jest uniwersalnie błędny. |
| `FIXED-04` | R40 pomijał 20 rigid mesh nodes w drzewach stanów. | R41 dodał pełne 45-node'owe state trees; nadal `not_visible/verified`. |
| `FIXED-05` | R42 wprowadził dodatkowy controllerless outer root i oddzielił animowany `Hips` od rigid mesha. | R43 usuwa outer root i odtwarza układ pozytywnego H1; wynik live jeszcze nieznany. |
| `FIXED-06` | Exact r43 MOD/HAK nie były zainstalowane. | Zainstalowane create-new/no-clobber; source i destination są byte-for-byte identyczne. |

### 13.4. Otwarte gałęzie diagnostyczne po jednym owner-run r43

| Wynik | Problem potwierdzony |
|---|---|
| Hook Horror widoczny, H2 niewidoczny | Custom creature MDL/MDX lub jego state/hierarchy jest odrzucany; generated scene/instantiation działa. |
| Hook Horror widoczny, H2 jako blob | Draw działa, ale błędne są transformacje, attachment albo geometria w runtime animation path. |
| Hook Horror i H2 niewidoczne | Nie odizolowano custom modelu; główny branch to generated runtime GFF/instantiation albo scena. Następny jest istniejący `m2a_van02`, bez tworzenia r44. |
| Hook Horror i H2 widoczne | Visibility gate zamknięty; problem przechodzi na SkinMesh, weights i poprawną deformację. |
| H2 r43 widoczny | Przyczyną niewidoczności r42 był dodatkowy outer-root layout w projekcji `type=0`. |
| H2 r43 niewidoczny | Hipoteza outer-root jako wystarczającej przyczyny zostaje odrzucona; nie wolno mimo to zmieniać kilku pól naraz. |

### 13.5. Sprawdzone i odrzucone jako samodzielne przyczyny

- brak PWK lub kolizji placeable;
- globalna awaria parsera binary MDL;
- globalna awaria `TriMesh`, surowego MDX albo TGA;
- 1543 trójkąty lub 2678 wierzchołków jako przekroczenie limitu;
- odwrócony winding;
- odwrócone face/vertex normals;
- NaN albo zdegenerowane trójkąty;
- wyzerowana skala przez sam brak root controllera;
- położenie całego H2 pod podłożem;
- model bounds/radius jako samodzielna przyczyna;
- `render=0`, zły `geometryType`, classification albo fog;
- brak vertex colors;
- brak SkinMesh jako uniwersalny warunek draw;
- `supermodel=NULL`;
- sam animation header `type=0` albo sam `type=5`;
- sama liczba 7 animacji jako wyjaśnienie pełnego braku draw;
- `MODELTYPE=S`, `RACE` lub brak MDL/TGA w exact HAK;
- sparse custom row jako jedyna wada `appearance.2da`;
- same `MaxHitPoints`, `SkillList`, `WalkRate` albo `PerceptionRange`;
- ostrzeżenia `Invalid Class` jako candidate-bound przyczyna;
- sam controllerless model root — retail `c_Direwolf` jest przeciwprzykładem;
- alpha/pełna przezroczystość badanej tekstury;
- sama obecność lub nieobecność face adjacency/surface IDs;
- sama obecność routine words/defaultów z jednego retail modelu.

## 14. Pierwszy pakiet implementacyjny po audycie

Pakiet wdrożono bez zmiany bajtów zamrożonego r43 i bez utworzenia r44.
Nie uruchamiano ani nie kontrolowano Toolsetu/NWN.

### 14.1. Zaimplementowane zabezpieczenia

| ID | Implementacja | Efekt |
|---|---|---|
| `IMPL-01` | `verify_h2_r43_installed_lineage_v1` | Fail-closed sprawdza długości, SHA-256 i byte-for-byte equality exact source ↔ natywna instalacja MOD/HAK. |
| `IMPL-02` | `verify_h2_r43_owner_proof_scene_v1` | Pinuje dokładny moduł, Area, kolejność HAK, start/kierunek gracza oraz dokładnie dwa fixture: H2 row 15100 i stock row 102 z pozycjami/orientacją r43. |
| `IMPL-03` | `classify_h2_r43_owner_observation_v1` | Rozdziela wynik stock/H2 i dopuszcza nową iterację wyłącznie przy exact-candidate-bound, judgeable oraz stock `visible_correct` / H2 `not_visible`. `proofCompleteness=failed` nie wymazuje czytelnego owner verdictu. |
| `IMPL-04` | Gałąź both-absent | Nie otwiera r44; kieruje do istniejącego stock-only `m2a_van02`. |
| `IMPL-05` | Gałąź incomplete/unbound | `missing`, `not_tested`, nieczytelny kadr lub brak exact binding kończą się `ProofIncomplete` i wymagają dokończenia tego samego r43. |
| `IMPL-06` | Niepoprawny stock-control | Stock `visible_corrupt` nie izoluje custom MDL, nie otwiera r44 i kieruje do diagnostyki instancji stock-control. |
| `IMPL-07` | Test natywnej instalacji tylko do odczytu | Faktyczne `modules\m2a_h2r43.mod` i `hak\m2a_h2r43.hak` zostały ponownie odczytane i zaakceptowane przez nowy gate. |

Kod:

- `crates/m2a-core/src/creature_visibility_gate.rs`;
- `crates/m2a-core/tests/h2_r43_owner_gate.rs`;
- eksport modułu w `crates/m2a-core/src/lib.rs`.

### 14.2. Wyniki walidacji

- testy gate: `6 passed`, `0 failed`, jeden test natywnej instalacji domyślnie
  `ignored`, bo wymaga jawnych ścieżek środowiskowych;
- wymuszony test faktycznej natywnej instalacji: `1 passed`, `0 failed`;
- regresja `h2_r43_owner_gate`, `h2_r43_h1_root_layout_candidate` i
  `binary_creature_multi_fixture_module`: łącznie `12 passed`, `0 failed`,
  `1 ignored`;
- pełne testy jednostkowe `m2a-core --lib`: `58 passed`, `0 failed`,
  `1 ignored`;
- lint gate/testu: przechodzi po wyciszeniu trzech istniejących klas ostrzeżeń
  z innych modułów; pełne `-D warnings` nadal zatrzymuje sześć wcześniejszych
  ostrzeżeń poza zakresem tego pakietu;
- `cargo fmt --all -- --check`: bez różnic.

### 14.3. Czego ten pakiet celowo jeszcze nie rozwiązuje

- nie dostarcza live verdictu r43; to nadal wymaga owner-run;
- nie twierdzi, że outer root jest już potwierdzoną przyczyną;
- nie wdraża SkinMesh, weights ani deformacji;
- nie tworzy nowego modelu, 2DA, HAK, MOD, resrefu ani r44;
- nie zastępuje obrazu z zamkniętego renderera readbackiem statycznym.

W praktyce usunięto teraz możliwość błędnego przejścia z niepełnego lub
nieizolującego wyniku do kolejnej iteracji. Pozostały blocker widoczności jest
już jednoznaczny: świeży wynik właściciela dla exact zainstalowanego r43.

## 15. Drugi pakiet implementacyjny: SkinMesh conformance

Ta sekcja jest nowsza niż sekcja 14.3. Implementacja nadal nie tworzy r44 ani
nowego artefaktu proof, ale rozwija wspólny tor SkinMesh wymagany po zamknięciu
visibility gate.

### 15.1. Zaimplementowane

| ID | Implementacja | Efekt |
|---|---|---|
| `IMPL-08` | `boneconstantindices: Vec<u32>` | Parser i publiczny readback zachowują pełne 4-bajtowe słowo zamiast przedstawiać je jako dwie pozorne wartości `i16`. |
| `IMPL-09` | High-half mutation | Test zapisuje `0x89abcdef`; osobna mutacja wysokich 16 bitów daje dokładnie semantic diff `skin.constants`. |
| `IMPL-10` | `evaluate_skin_deformation_v1` | Odbudowuje bind/animated world matrices, translation, quaternion slerp, WXYZ inverse bind, sloty kości i cztery wagi bezpośrednio z własnego binary MDL readbacku. |
| `IMPL-11` | Hierarchy/weight oracle | Potwierdza ruch kości nadrzędnej, dziedziczenie przez potomka oraz mieszane wagi `1/2/4`. |
| `IMPL-12` | Owned M6 end-to-end sample | Istniejący generowany GLB przechodzi GLB → Profile A → MDL/MDX writer → parser → niezerowy SkinMesh motion sample. |
| `IMPL-13` | Fail-closed inverse bind | Zerowy, niefinitywny albo nieunitarny quaternion inverse-bind oraz niefinitywna translacja są odrzucane stabilnym kodem. |
| `IMPL-14` | Env-gated native state-skin invariant | Exact CEP HAK potwierdza wszystkie trzy wymagane profile w podzbiorze przyjmowanym przez strict reader i osobno ujawnia jego nieobsługiwany zakres. |

Writer nadal emituje cztery zerowe bajty constants, więc zmiana typu nie
zmienia payloadu. Ponowne testy frozen r43 potwierdziły jego dotychczasowe
hashe i niezmienioną scenę.

### 15.2. Weryfikacja

- parser MDL: `41 passed`, `0 failed`;
- writer MDL: `39 passed`, `0 failed`;
- `m2a-core --lib + profile_a + model_pipeline`: `120 passed`,
  `0 failed`, `1 ignored`;
- łączna regresja zmienionego zakresu wraz z `runtime_witness_conformance`:
  `201 passed`, `0 failed`, `6 ignored`;
- wymuszony exact CEP state-skin corpus: `1 passed`, `0 failed`;
- exact r43 candidate i owner gate: `7 passed`, `0 failed`, `1 ignored`;
- Clippy dla zmienionego zakresu przechodzi po wyciszeniu wyłącznie znanych,
  wcześniejszych klas ostrzeżeń z innych funkcji/testów;
- `cargo fmt --all -- --check`: PASS.

Pełny `cargo test --workspace` przeszedł cały `m2a-core`, ale kończy się kodem
1 na czterech istniejących frozen-hash testach `m2a-wasm`: trzech snapshotach
Profile A JSON oraz jednym M7 batch JSON. Profile A JSON powstaje przed writerem
MDL i nie zawiera `SkinReport`, więc tych hashy nie zmieniono bez osobnej
diagnozy. Nie są dowodem regresji nowego SkinMesh oracle, ale pozostają jawnym
problemem quality gate workspace.

### 15.3. Granica dowodu

CPU oracle dowodzi matematycznej spójności emitowanych transformacji, inverse
bindów, slotów i wag dla własnej fixture. Nie dowodzi, że zamknięty renderer
NWN interpretuje wszystkie pola identycznie. `OPEN-08` pozostaje otwarte do
hash-bound owner proof poprawnej wizualnej deformacji. Visibility branch nadal
czeka na exact owner verdict r43; bez niego nie wolno materializować r44.

### 15.4. Exact native state-skin corpus

Env-gated test
`cep_state_skin_profiles_are_preserved_and_reader_coverage_gap_is_explicit`
przeszedł na exact `cep3_core1.hak`:

- SHA-256:
  `6a8e6a64773a77fd46740cbcce19a708db6a70b4975732d0405978f3fbe8eb1a`;
- zasoby MDL: `3517`;
- payloady z sygnaturą binarną: `3430`;
- modele przyjęte przez strict reader: `2151`;
- nieobsługiwane modele binarne: `1279`;
- modele z base SkinMesh i animacjami: `72`;
- profil bez state SkinMesh: `66`;
- matching generic `0x01`: `6` modeli, `214` node'ów, wszystkie liście bez
  kontrolerów;
- matching state SkinMesh `0x61`: `6` modeli, `565` node'ów.

Test dowodzi obecności i rozpoznawania każdego z trzech profili w obsługiwanym
podzbiorze. Nie udaje pełnego wsparcia CEP: `1279` binarnych modeli pozostaje
jawnym długiem strict readera. Historyczne pełne liczby z tolerancyjnego sweepu
nie zostały przepisane jako wynik tego parsera.

## 16. Trzeci pakiet implementacyjny: pełny profil 42 stanów

Pakiet nie tworzy r44, nowego resrefu, HAK-a ani MOD-a. Frozen r43 zachowuje
dotychczasowe bajty i hashe.

### 16.1. Zaimplementowane

| ID | Implementacja | Efekt |
|---|---|---|
| `IMPL-15` | Exact 42-state namespace | Publiczna lista jest potwierdzana jako identyczny zbiór przez own reader na retail `c_Direwolf`, retail `c_horror` i CEP R3 `c_phod_horror_b`. |
| `IMPL-16` | `DirectCreatureAnimationProfileV1` | Rozdziela legacy 7-state idle-fallback od `FullNative42ExplicitV1`; pełny profil nie tworzy żadnego brakującego stanu przez alias idle. |
| `IMPL-17` | Core builders V2 | Explicit-profile builder przyjmuje caller-owned mapping 42 animacji; automatyczny H1 V2 uznaje wyłącznie dokładne source animation names i fail-closed raportuje braki. |
| `IMPL-18` | Completeness report | Pełny raport zapisuje `requiredClipCount=42`, `explicitClipCount=42`, `fallbackAliasCount=0`, `complete=true`; legacy JSON pozostaje bez nowego pola. |
| `IMPL-19` | WASM/Worker/Studio | WASM eksportuje H1 V2, Worker obsługuje `H1_SKINNED_FULL_42`, a Studio wybiera lane tylko dla dokładnego kompletnego zbioru bez braków, duplikatów i nadmiarowych nazw. |
| `IMPL-20` | Behavior oracle po own binary readbacku | Wymaga controller content wszystkich 42 stanów, ruchu w stanach aktywnych zgodnych między trzema rodzinami native, rozróżnia kluczowe idle/locomotion/attack/damage/death semantics i wymaga terminalnej pozy `ckdbckdie`. |
| `IMPL-21` | Korekta `cdead`/family-variable states | Retail i CEP wykazały, że `cdead`, `cgetmidlp` i `ccastoutlp` nie mają jednego globalnego motion/stillness contractu. Gate wymaga ich jawnego payloadu, ale nie wymusza arbitralnie ruchu ani bezruchu; death transition jest wiązany z `ckdbckdie`. |
| `IMPL-22` | Full V2 SkinMesh animation conformance | Próbkuje `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `ckdbckdie`, wymaga co najmniej dwóch aktywnych jointów i nierigid zmiany kształtu ważonej siatki. Root-only rigid motion jest odrzucany przez `M6-SKIN-ANIMATION-INELIGIBLE`. |

### 16.2. Weryfikacja

- Core V2: pełne 42 source animations → 42 binary animation headers type 5;
- negatywne: jeden idle albo brak `ccturnr` →
  `M6-ANIMATION-FULL-PROFILE-MISSING`; nadmiarowy stan →
  `M6-ANIMATION-FULL-PROFILE-UNKNOWN`;
- negatywne behavior: brak controller content, brak ruchu aktywnego stanu,
  identyczne `walk/run`, alias kluczowych semantyk lub brak terminalnej pozy →
  `M6-ANIMATION-BEHAVIOR-INELIGIBLE`;
- negatywne SkinMesh: 42 zmienne klipy przesuwające wyłącznie root →
  `M6-SKIN-ANIMATION-INELIGIBLE`;
- focused regresja Core (`lib`, `mdl_writer`, `model_pipeline`,
  `runtime_witness_conformance`): `124 passed`, `0 failed`, `6 ignored`;
- exact r43 candidate/owner gate: `7 passed`, `0 failed`, `1 ignored`;
- wymuszone env-gated local witnesses: `5 passed`, `0 failed`; ten sam behavior
  floor przechodzą retail `c_Direwolf`, retail `c_horror` i CEP R3;
- WASM V2 native adapter: `1 passed`, `0 failed`;
- Studio unit: `8 passed`, `0 failed`;
- real Chromium Worker/WASM integration: `6 passed`, `0 failed`;
- TypeScript typecheck, targeted Clippy i `cargo fmt --check`: PASS.

Szeroki Clippy wszystkich test targets nadal zatrzymuje się na dwóch
wcześniejszych ostrzeżeniach poza zmienionym zakresem:
`clippy::useless_format` w `m0_r33_animated_donor_candidate.rs` oraz
`clippy::single_element_loop` w `profile_a.rs`.

### 16.3. Granica dowodu

Pełny namespace, brak niejawnego fallbacku, offline behavior admission i
wielojointowa nierigid deformacja są zaimplementowane. Syntetyczna fixture 42
stanów używa niezależnych payloadów i ruchu child-bone; nie jest kopią
referencyjnych keyframes.

To nadal nie oznacza poprawnie animowanego finalnego creature w grze. Oracle
odrzuca klasy błędów możliwe do rozstrzygnięcia offline, ale nie dowodzi:

- artystycznej i gameplayowej poprawności source animations;
- runtime'owego wywołania callbacków `hit`/`cast`/sound; caller-owned timing,
  zapis i binary readback są zaimplementowane w sekcji 17;
- engine loop i state routingu;
- interpretacji emitowanego SkinMesh przez zamknięty renderer NWN;
- widoczności exact r43, która pozostaje human-owned blockerem przed r44.

Finalne zamknięcie nadal wymaga hash-bound owner proof deformacji i zachowania
w NWN po zamknięciu visibility branch exact r43.

## 17. Czwarty pakiet implementacyjny: caller-owned eventy

Ten pakiet takze nie tworzy r44, nowego resrefu, HAK-a ani MOD-a i nie zmienia
zamrozonego r43.

### 17.1. Zaimplementowane

| ID | Implementacja | Efekt |
|---|---|---|
| `IMPL-23` | Exact common-native event floor | Own reader wylicza i testuje dokladna wspolna czesc retail `c_Direwolf`, retail `c_horror` i CEP R3 `c_phod_horror_b`: 23 pary clip/event. |
| `IMPL-24` | `DirectCreatureEventAuthoringV1` | Strict, wersjonowany caller-owned sidecar; unikalne clip names, bezpieczne nazwy, skonczone czasy w granicach klipu, zachowanie nieznanych event names bez ich ukrywania. |
| `IMPL-25` | Core V3 i readback gate | Eventy sa nakladane po exact 42-state mappingu, zapisywane do MDL i sprawdzane ponownie z wlasnych bajtow. Brak jednej wymaganej pary daje `M6-ANIMATION-EVENTS-INELIGIBLE`. |
| `IMPL-26` | Provenance | Raport zachowuje `animationEventConformance` oraz kanoniczne `byteLength`/SHA-256 sparsowanego sidecara. |
| `IMPL-27` | WASM/Worker/Studio | `buildMeshyH1ModelPackageV3`, lane `H1_SKINNED_FULL_42_EVENTS` i opcjonalny input `animation-events.json` udostepniaja pelny tor w aplikacji. Studio odrzuca sidecar dla placeable i source bez exact 42 stanow. |
| `IMPL-28` | Widoczny wynik w Review | UI rozroznia event-complete `PASS 23/23` od skin package, ktory nie uzywal eventful lane; nie przedstawia braku eventow jako kompletnego creature. |

Timingi nie sa kopiowane z witnessow. Lista 23 par jest profilem pokrycia;
kazdy czas pochodzi z owned source/sidecara uzytkownika.

### 17.2. Weryfikacja

- focused Core (`lib`, `mdl_writer`, `model_pipeline`,
  `runtime_witness_conformance`): `125 passed`, `0 failed`, `6 ignored`;
- wymuszone exact native witnesses: `5 passed`, `0 failed`;
- exact r43 candidate/owner/install gates: `8 passed`, `0 failed`;
- WASM native: `25 passed`, `0 failed`;
- Studio unit: `183 passed`, `0 failed`;
- real Chromium Worker/WASM: `7 passed`, `0 failed`;
- TypeScript typecheck, targeted Clippy i `cargo fmt --check`: PASS.

Test pozytywny potwierdza `23/23`, canonical sidecar identity i
`ccastout:cast` przy dokladnie caller-owned czasie po binary readbacku. Testy
negatywne obejmuja malformed/unknown JSON, czas poza klipem, duplicate clip,
brak `ccastout:cast`, niepelny namespace i root-only rigid motion.

### 17.3. Pelna aktualna lista problemow

| ID | Stan | Co dokladnie pozostaje |
|---|---|---|
| `OPEN-01` | blokuje kolejna iteracje | Exact zainstalowany r43 nadal nie ma owner verdictu w Toolset ani NWN. Nie wolno utworzyc r44. |
| `OPEN-02` | zalezne od wyniku r43 | Przyczyna niewidocznosci r42 jest najsilniej zwiazana z dodatkowym outer-root layoutem, ale stanie sie przyczyna potwierdzona dopiero, gdy r43 bedzie widoczny. Jesli r43 nie bedzie widoczny, hipoteza nie wystarcza. |
| `OPEN-03` | runtime | Nie ma jeszcze poprawnie widocznego finalnego SkinMesh creature z hash-bound owner proofem deformacji w NWN. |
| `OPEN-04` | runtime | Exact 42-state engine routing, petle i przejscia nie maja owner proofu. Offline sa namespace, controller content, motion floor, rozne kluczowe semantyki i terminalny `ckdbckdie`. |
| `OPEN-05` | runtime | Event payload i `23/23` binary conformance sa gotowe, lecz rzeczywiste callbacki `hit`, `cast`, `snd_footstep`, `snd_hitground` nie sa jeszcze zaobserwowane w NWN. |
| `OPEN-06` | input/akceptacja | Artystyczna i gameplayowa jakosc 42 caller-owned animacji oraz timingow eventow wymaga oceny wlasciciela; oracle nie moze jej wywnioskowac. |
| `OPEN-07` | dowod kontrolny | Jesli oba creature r43 beda nieobecne, trzeba uzyc istniejacego stock-only `m2a_van02`; nie wolno wtedy diagnozowac custom MDL ani otwierac r44. |
| `OPEN-08` | reader debt | Strict reader jawnie nie obsluguje `1279` binarnych modeli CEP. Nie blokuje wspieranego full-42 profilu, ale nie wolno przedstawic go jako pelnego wsparcia calego CEP. |
| `OPEN-09` | final acceptance | Dopiero owner proof moze zamknac widocznosc, deformacje, routing i callback semantics. Agent-side granica pozostaje `ready_for_owner_proof`, nie deklaracja live success. |

Nieaktualny punkt z sekcji 16.3 o "braku event callbackow ani wymaganych
timingow" nalezy rozdzielic: authoring, zapis, zakres czasu, coverage i
readback sa zaimplementowane; otwarte pozostaje tylko zachowanie callbackow
oraz ocena owned timingow w zamknietym runtime.

## 18. Recheck blokera -- 2026-07-26

Ponowny read-only audit external state nie znalazl nowego owner resultu ani
capture zwiazanego z exact r43. Aktualne logi NWN zawieraja pozniejsze loady
`m2a_s1_c2_mod`, `m2a_s1_c3_mod` i `m2a_tlcm3_mod`, ale nie zawieraja
`m2a_h2r43`, `m2a_h2a43` ani nazwy `Meshy2Aurora creature comparison`.

Exact native installation gate zostal ponownie wymuszony:

- `m2a_h2r43.mod`: source i native destination sa byte-for-byte identyczne;
- `m2a_h2r43.hak`: source i native destination sa byte-for-byte identyczne;
- test `exact_native_installation_matches_the_frozen_candidate`: `1 passed`.

To jest ten sam zewnetrzny blocker, a nie nowa porazka modelu:

- `modelVisibility=not_tested`;
- `proofCompleteness=missing`;
- r44 pozostaje zabroniony;
- wszystkie bezpieczne, niezalezne od verdictu braki offline opisane w
  sekcjach 14-17 sa zaimplementowane i przetestowane.

Nastepny dopuszczalny input to swiezy owner verdict dla obu obiektow w exact
r43: stock Hook Horror po lewej i H2 w centrum. Dopiero klasyfikacja tego
wyniku moze wybrac minimalna delte albo zamknac visibility gate.

## 19. Wynik właściciela z dodanym natywnym blueprintem — 2026-07-26

Ta sekcja zastępuje nieaktualny stan z sekcji 18. Właściciel dodał w Toolsecie
trzecie creature z natywnej palety NWN i przekazał świeży wynik:

> `dodałem do testów model ktoy jest natywny z nwn bo twoje z meshy i tyen horror nie dzialaja!`

Trwały packet:

`documentation/evidence/h2-r43-owner-added-native-control-result-2026-07-26.json`.

### 19.1. Dokładna scena po zapisie Toolsetu

- plik modułu: `m2a_h2r43.mod`;
- frame Toolsetu: `m2a_h2r43.mod`;
- Area resref: `m2a_h2a43`;
- nazwa Area:
  `Meshy2Aurora M0 binary vertical-slice area`;
- exact HAK nadal ma `20 084 361` bajtów i SHA-256
  `d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1`;
- oryginalny frozen MOD miał `20 280` bajtów i SHA-256
  `ef2b48b80e4c01b455a04246c8e1da8f839171d9e9b4f9e835b846738f0a0c89`;
- MOD zapisany przez właściciela po dodaniu kontroli ma `39 090` bajtów,
  timestamp `2026-07-26T00:39:51.5276275+02:00`;
- ten zmodyfikowany MOD pozostaje zablokowany przez zewnętrzny proces, dlatego
  jego SHA-256 jest jeszcze niedostępny;
- odczytany in-place GIT ma `12 113` bajtów i SHA-256
  `40275d3d358f11bbe97b7947b7311e9b2e932f68e1244f44872bf4154f1ec71d`.

Screenshot właściciela ma `463 868` bajtów i SHA-256
`7ddad96134188551c73b69e804b5110918728058dd61f10c5691ab672e3446ce`.
Pokazuje trzy wpisy w drzewie Toolsetu i trzy renderowane obiekty, ale nie jest
walidowanym capture `TScrollBox` związanym z dokładnie zaznaczonym obiektem,
więc pozostaje dowodem obserwacyjnym Toolsetu, a nie kompletnym verdict packetem.

### 19.2. Zaskakujący, ale rozstrzygający readback GIT

Dokładny GIT zawiera:

| Obiekt | `TemplateResRef` | `Appearance_Type` | Profil runtime |
|---|---|---:|---|
| Meshy H2 | `m2a_h2utc43` | `15100` | race 0, faction 2, HP `1/1/13`, walk 0, perception 0, class `12:12`, bez default AI |
| wygenerowany Hook Horror | `m2a_h2ctrl43` | `102` | ten sam minimalny profil |
| dodany „Drider Chief” | `x2_drider003` | `15100` | race 7, faction 1, HP `80/80/128`, walk 7, perception 11, classes `11:6` + `9:10`, default AI |

Nazwa w drzewie i `TemplateResRef` trzeciego obiektu są natywne, lecz jego
zapisany `Appearance_Type` wynosi `15100`, nie retailowy row Dridera `407`.
Właściciel następnie wyjaśnił:

> `bo podmienilem do testow wyglad =)`

Override był więc celowy. Nie jest to błąd Toolsetu ani przypadkowa zmiana
appearance. Test łączy natywny, kompletny profil blueprintu `x2_drider003` z
naszym Appearance row `15100`.

Jeżeli właściciel potwierdzi, że nowa kontrola jest widoczna w NWN, wtedy będą
dwie możliwe interpretacje jej obrazu, które trzeba nadal rozdzielić:

1. jeżeli w NWN widać pająkowatego Dridera, runtime cofnął się do wyglądu
   blueprintu albo nie zastosował instancyjnego row `15100`;
2. jeżeli w NWN widać niebieski model clockwork, custom MDL `m2a_h2p43`
   działa przez natywnie kompletny profil, a awaria jest już całkowicie
   odizolowana od MDL/HAK/2DA.

Właściciel potwierdził intencję podmiany wyglądu, ale cytowane wypowiedzi nadal
nie podają jawnie wyniku tej instancji w NWN. Dlatego packet zapisuje dla tej
kontroli `modelVisibility=not_tested` i `proofCompleteness=missing`. Nie wolno
z samego dodania obiektu i celowego override'u wywnioskować, że zadziałał.

### 19.3. Fakt z dekompilacji Aurory

Ścieżka ładowania instancji creature nie wykonuje runtime merge z UTC według
samego `TemplateResRef`:

- `FUN_0053ab24` odczytuje pozycję i orientację z bieżącej struktury GIT;
- następnie wywołuje `FUN_00532174` bezpośrednio na tej samej strukturze;
- `FUN_00532174` odczytuje z niej `Appearance_Type`, `TemplateResRef`, rasę,
  frakcję, statystyki, skille, klasy, HP i skrypty.

`TemplateResRef` jest więc zachowaną tożsamością/metadanymi blueprintu w
ścieżce Toolsetu. Nie naprawia minimalnego GIT przez automatyczne dołączenie
właściwości retailowego UTC. Nowa kontrola ma stan natywnego blueprintu
skopiowany przez Toolset do GIT, lecz jej zachowanie w NWN pozostaje do
potwierdzenia.

### 19.4. Skorygowany błąd naszego gate

Poprzednia implementacja błędnie uznawała:

- `MaxHitPoints == 13`;
- dokładnie 28 skilli z rangą zero

za wymagania silnika. Retailowe, poprawne UTC obalają oba twierdzenia:

- `nw_horror`: `MaxHitPoints=37`, 23 skille;
- `x2_drider002`: `MaxHitPoints=58`, 27 skilli;
- `nw_dwarfmerc001`: `MaxHitPoints=14`, 20 skilli.

Walidator `validate_binary_creature_runtime_complete_envelope` wymaga teraz
poprawnego typu i dodatniego `MaxHitPoints`, a dla każdej obecnej pozycji
`SkillList` wymaga struktury 0 z jednym polem BYTE `Rank`. Długość listy i
wartości rang mogą być natywnie różne.

### 19.5. Implementacja V2 bez materializacji r44

Dodano wersjonowany generator:

- `BinaryCreatureRuntimeProfileV2::LegacyMinimal`;
- `BinaryCreatureRuntimeProfileV2::PassiveMonsterBaseline`;
- `BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline`;
- `build_binary_creature_profile_matrix_module_v2`;
- `inspect_binary_creature_profile_matrix_module_v2`.

Profil jest emitowany niezależnie do GIT oraz module-local UTC. Readback
klasyfikuje go ponownie z exact bajtów obu zasobów i wymaga ich zgodności.
Nieznana mieszanka pól, nawet pojedyncze `WalkRate=6`, jest odrzucana kodem
`M0-BINARY-PROFILE-MATRIX-READBACK-INVALID`.

Baseline nie kopiuje retailowego UTC ani jego payloadu. Jest własnym profilem
syntetycznym opartym na potwierdzonych typach/zakresach z dekompilacji i
obserwacjach kilku natywnych zasobów. Rozdział passive/active pozwala
odróżnić:

- odrzucenie lub degradację minimalnego gameplay profile;
- brak ruchu do kadru wynikający z neutralnej, nieruchomej i pozbawionej
  skryptów instancji;
- właściwą awarię modelu/appearance.

Nie utworzono nowego MOD-a, HAK-a, modelu, 2DA, resrefu ani r44.

Weryfikacja:

- nowy profile matrix: `2 passed`, `0 failed`;
- istniejący multi-fixture: `6 passed`, `0 failed`;
- exact r43 layout: `1 passed`, `0 failed`;
- owner gate: `6 passed`, `0 failed`, `1 ignored`;
- razem w focused regression: `15 passed`, `0 failed`, `1 ignored`;
- `cargo fmt --all`: wykonany.

### 19.6. Aktualna pełna lista problemów

| ID | Stan | Problem / wynik |
|---|---|---|
| `VIS-01` | wykluczony | To nie jest niewłaściwy moduł ani niewłaściwa Area: frame, GIT, HAK i właścicielski test są związane z `m2a_h2r43` / `m2a_h2a43`. |
| `VIS-02` | otwarty | Natywny blueprint został dodany i istnieje w GIT, ale właściciel nie podał jeszcze jego jawnego wyniku NWN. Ogólnego braku wszystkich creature nie wolno jeszcze wykluczyć na podstawie samego dodania kontroli. |
| `VIS-03` | potwierdzony błąd projektu | Stock Hook Horror nie był niezależną kontrolą kontenera. Miał native appearance row 102, ale ten sam nasz minimalny GIT/UTC runtime profile co Meshy H2. |
| `VIS-04` | potwierdzony błąd projektu | Gate `MaxHitPoints=13` / 28 zerowych skilli przedstawiał wartości jednej fixture jako invariant silnika. Jest naprawiony. |
| `VIS-05` | potwierdzony fakt | `TemplateResRef` nie dziedziczy runtime pełnego UTC do instancji GIT. Generator musi emitować poprawny pełny profil sam. |
| `VIS-06` | najsilniejszy boundary, scalar otwarty | Oba zgłoszone jako niewidoczne obiekty mają ten sam minimalny profil GIT/UTC. Nie wiadomo jeszcze, czy scalar to odrzucana semantyka gameplay, generated UTC/container, neutralna/scriptless/stationary scena, czy szerszy błąd runtime. V2 rozdziela część tych przypadków. |
| `VIS-07` | intencja potwierdzona, wynik otwarty | Dodany `x2_drider003` ma celowo ustawiony row `15100`. Trzeba już tylko rozstrzygnąć, czy NWN pokazał clockwork H2, fallback Dridera, czy nic. |
| `VIS-08` | blokuje r44 | Zmieniony MOD `39 090` bajtów jest nadal zablokowany i nie ma SHA-256. Bez tego exact current lineage nie jest zamknięty i nie wolno materializować kolejnej iteracji. |
| `VIS-09` | gotowe offline | Generator trzech profili i fail-closed readback GIT/UTC są zaimplementowane i przetestowane. |
| `VIS-10` | runtime | V2 nie ma jeszcze owner proofu; sam test offline nie dowodzi widoczności w NWN. |
| `VIS-11` | final runtime | Nadal brakuje owner proofu finalnego SkinMesh, deformacji, 42-state routingu i callbacków eventów w NWN. |

Minimalna deklarowana delta następnego kandydata, po zamknięciu hasha bieżącego
MOD-a, nie zmienia HAK/MDL/TGA/appearance r43. Zmienia wyłącznie scenę creature
na jawną macierz profili V2. Dzięki temu kolejny wynik nie będzie ponownie
mieszał błędu modelu z błędem naszego kontenera.

## 20. Korekta: r43 odtwarzał corrupted baseline, a model nie działa w NWN

Sekcja 20 zastępuje diagnozę i minimalną deltę z końca sekcji 19. Właściciel
przekazał jednoznaczną korektę:

> `rozumiesz ze tworzysz modul testowy z corrupted modelamai a dotego model nie dziala w nwn =(`

Wynik operacyjny custom modelu jest więc failed. Dla trzeciej instancji
`x2_drider003` z celowym `Appearance_Type=15100` wypowiedź nie rozróżnia
`visible_corrupt` od `not_visible`, dlatego nie wolno wymyślać dokładnej
morfologii. Nie zmienia to głównego werdyktu: custom appearance/MDL nie działa
poprawnie w NWN, a testowy moduł nie zawierał czystej niezależnej kontroli.

### 20.1. Potwierdzony błąd procesu

Kod i test r43 używały nazwy
`OwnedRuntimePositiveType0RigOnlyV1`. Była ona fałszywa:

- jedyny runtime obraz H1 v20 przedstawiał duży, silnie zdeformowany blob;
- H1 v20 dowodził wyłącznie wejścia bajtów na ścieżkę draw;
- nie dowodził poprawnej geometrii, transformacji, hierarchii, animacji ani
  creature;
- mimo tego r43 miał obowiązek odtworzyć jego jeden rigid mesh pod rootem,
  siedem stanów typu 0 oraz family z zerowymi routine words.

Implementacja zachowuje immutable serializację frozen r43, ale nazwa Rust i
bramka zostały skorygowane:

- `OwnedCorruptDrawType0RigOnlyV1`;
- H1 v20 klasyfikuje się jako `DrawPathOnly`;
- tylko exact, verified, visible i visually correct obserwacja może zostać
  `CorrectCreatureBaseline`;
- test r43 nazywa operację replayem corrupt-draw, nie zgodnością z poprawnym
  creature.

Exact frozen hash r43 pozostaje bez zmian; test replay przechodzi.

### 20.2. Różnica z poprawnymi modelami NWN

Reader odczytał in-place trzy niezależne poprawne rodziny z lokalnymi
animacjami oraz poprawny SkinMesh korzystający z supermodelu:

| Rodzina | Base | Mesh | Skin | Stany | Typ stanów | Vertex colors |
|---|---:|---:|---:|---:|---:|---:|
| retail `c_Direwolf` | 30 | 24 | 0 | 42 | 5 | 24/24 |
| retail `c_Horror` | 27 | 21 | 0 | 42 | 5 | 21/21 |
| CEP R3 `c_phod_horror_b` | 27 | 21 | 0 | 42 | 5 | 21/21 |
| CEP `c_kocrachn`, supermodel `c_Horror` | 38 | 31 | 3 | 0 lokalnych | n/d | 0/3 SkinMesh |
| H1 v20 corrupt draw | 25 | 1 | 0 | 7 | 0 | 0/1 |
| r43 failed | 25 | 1 | 0 | 7 | 0 | 0/1 |

Wcześniejsze odrzucenie pełnego 42-state namespace jako istotnej różnicy było
oparte wyłącznie na H1 v20. Po poprawnym sklasyfikowaniu H1 ten argument znika:
wszystkie poprawne rodziny samodzielne mają 42 stany typu 5. Poprawny
`c_kocrachn` ma zero stanów lokalnych, ale jawnie dziedziczy kompatybilny
`c_Horror`. Nasze r40/r41/r43 miały `supermodel=NULL` i tylko siedem stanów,
czyli nie spełniały żadnej z tych dwóch legalnych rodzin.

`c_kocrachn` potwierdza także docelowy shape SkinMesh: controllerless model
root, skeleton pod rootem, trzy extended64 SkinMesh bezpośrednio pod rootem i
legalny brak vertex colors. Nowy profil H2 odtwarza te invarianty bez
dziedziczenia niekompatybilnego obcego szkieletu.

Nie oznacza to, że pojedynczym scalarem jest sama liczba `42`. Poprawny profil
musi łącznie zapewnić natywny shape root/skeleton/surface, brak niedozwolonego
runtime scale oraz komplet semantycznie różnych stanów. Nie wolno wpisywać
adresów routine words z retailowego procesu; te pola pozostają różne między
legalnymi rodzinami i nie są kopiowane.

### 20.3. Zaimplementowany pełny profil animacji H2

Dodano opt-in
`DirectCreatureAnimationProfileV1::FullNative42ProceduralHumanoidV1`.
Wejściem pozostaje dokładny owned H2 z jednym klipem
`Armature|Idle|baselayer`. Implementacja:

1. wiąże dokładnie jedenaście semantycznych jointów H2:
   `Hips`, `Spine`, `Head`, obie ręce, przedramiona, uda i golenie;
2. zachowuje caller-owned `cpause1`;
3. samodzielnie authoruje 41 dalszych, odrębnych clipów clean-room;
4. emituje dokładny namespace 42 stanów typu 5;
5. rozróżnia ruch walk/run, ataki, cast, damage, knockdown, turn, appear,
   disappear i terminal death pose;
6. authoruje 23 gameplay callbacki z własnymi znormalizowanymi timingami:
   `hit`, `cast`, `snd_footstep`, `snd_hitground`;
7. usuwa wyłącznie finite, constant, uniform scale tracks; zmienny albo
   niejednorodny scale jest fail-closed;
8. dodaje controllerless identity Aurora root;
9. zachowuje weighted `Hips` skeleton pod rootem;
10. bake'uje stary bind root do geometrii i przyczepia SkinMesh bezpośrednio
    do Aurora root;
11. używa writer profile V3 z zero-terminated SkinMesh palette;
12. sprawdza exact binary readback, pełny behavior oracle i niejednorodną
    deformację ważonych vertexów.

Test na dokładnym pliku
`test-assets/meshy/incoming/h2-clockwork-sentinel-1500.glb` przechodzi:

- 42/42 nazw;
- wszystkie stany typu 5;
- controllerless model root;
- `Hips` i SkinMesh bezpośrednio pod rootem;
- wszystkie animation roots controllerless;
- 41 procedural clips + 1 source clip, bez idle aliases;
- 23/23 callbacki;
- distinct walk/run i essential states;
- terminal death transition;
- non-rigid SkinMesh deformation na wymaganych stanach.

To jest implementacja offline, nie wizualny proof NWN.

### 20.4. Skorygowana pełna lista problemów

| ID | Stan | Problem / wynik |
|---|---|---|
| `VIS-01` | wykluczony | Test dotyczy `m2a_h2r43` / `m2a_h2a43`; „zły moduł” nie jest diagnozą. |
| `VIS-02` | potwierdzony | Custom model nie działa poprawnie w NWN według świeżej korekty właściciela. |
| `VIS-03` | potwierdzony | H1 v20 był corrupted draw, nie poprawnym pozytywnym wzorcem. |
| `VIS-04` | naprawiony | Kod i testy nie nazywają już H1 correctness baseline; corrupt draw nie może kwalifikować modelu. |
| `VIS-05` | potwierdzony | r43 świadomie kopiował błędny profil H1: one rigid mesh, siedem type-0 states. |
| `VIS-06` | potwierdzony | Generated Hook Horror nie był niezależną kontrolą; używał naszego generated GIT/UTC harnessu. |
| `VIS-07` | potwierdzony | Dodany native blueprint miał celowo row `15100`; minimalny profil H2 nie może sam wyjaśnić całej awarii custom modelu. |
| `VIS-08` | naprawiony offline | Fałszywy gate `MaxHitPoints=13` / 28 zerowych skilli został usunięty. |
| `VIS-09` | naprawiony offline | V2 container profile matrix rozdziela legacy, passive i active monster baseline. |
| `VIS-10` | naprawiony offline | Dodano pełne 42 distinct procedural animations dla owned H2, zamiast siedmiu idle aliases. |
| `VIS-11` | naprawiony offline | Dodano 23 clean-room gameplay callbacks. |
| `VIS-12` | naprawiony offline | Procedural profile usuwa stały uniform runtime scale i odrzuca pozostałe scale tracks. |
| `VIS-13` | naprawiony offline | Procedural profile emituje controllerless Aurora root, skeleton child i direct-root SkinMesh. |
| `VIS-14` | otwarty runtime | Sam offline behavior/deformation oracle nie dowodzi jeszcze, że exact nowe bajty wyrenderują się poprawnie w NWN. |
| `VIS-15` | blokuje materializację | Owner-modified `m2a_h2r43.mod`, 39 090 bajtów, nadal jest zablokowany przez zewnętrzny proces i nie ma SHA-256. |
| `VIS-16` | hard stop | Do uzyskania tego hasha nie wolno tworzyć r44, nowego MOD/HAK/MDL/2DA/resrefu. |
| `VIS-17` | final runtime | Po odblokowaniu i materializacji wymagany jest owner proof tej samej exact lineage w Toolsecie i NWN. |

Werdykt wizualny custom modelu dopuszcza diagnozę kolejnej delty, lecz
materializacja pozostaje zatrzymana wyłącznie na brakującym hashu zmienionego
MOD-a. Deklarowana następna delta artefaktu to zastąpienie corrupt-draw r43 MDL
przez zaimplementowany controllerless SkinMesh + 42-state procedural profile i
użycie aktywnego profilu creature V2. H1 v20 nie będzie już kontrolą
poprawności.

## 21. Domknięcie produkcyjnego toru po korekcie corrupted module

Po sekcji 20 wykryto i naprawiono cztery dalsze braki, które powodowały, że
poprawiony Core nadal nie był pełną implementacją aplikacji.

### 21.1. Studio nadal wybierało stary corrupted-draw lane

Core miał już `FullNative42ProceduralHumanoidV1`, ale WASM, Worker i Studio go
nie udostępniały. Oskórowany GLB bez exact 42 własnych nazw nadal trafiał do
`H1_SKINNED`, czyli generatora siedmiu aliasów idle użytego przez błędny r43.

Naprawa:

- dodano publiczne
  `buildMeshyProceduralHumanoidModelPackageV1`;
- dodano Worker lane `SKINNED_PROCEDURAL_HUMANOID_42`;
- Studio wybiera ten lane dla oskórowanego źródła, które nie dostarcza exact
  42 caller-owned stanów;
- `H1_SKINNED` usunięto z produkcyjnego request union;
- realny test Worker wymaga odrzucenia historycznego `H1_SKINNED` bez
  zwrócenia artefaktów.

Legacy Core/WASM pozostają wyłącznie do odtwarzania historycznych frozen
kontraktów i diagnostyki. Aplikacja nie może już z nich zbudować modułu do
testu.

### 21.2. Ukryty scale controller pozostał w `cpause1`

Pierwsza wersja proceduralna usuwała scale track z 41 nowych stanów, lecz
zachowywała go w caller-owned `cpause1`. Wcześniejszy deformation gate badał
pięć klipów gameplay i dlatego nie widział asymetrii idle.

Nowy bind-pose test wymusił ocenę `cpause1` i otrzymał
`M2A-MDL-SKIN-DEFORMATION-SCALE-UNSUPPORTED`. Naprawa normalizuje source idle
przed utworzeniem całego zestawu:

- finite, constant, uniform scale jest usuwany ze wszystkich 42 stanów;
- animated, non-finite albo non-uniform scale jest fail-closed;
- każdy z 42 stanów przechodzi unit-scale SkinMesh deformation oracle.

Po naprawie exact H2 ma błąd rekonstrukcji bind pose poniżej `1e-4`.
Odczytane bounds siatki to około
`[-0.6094,-0.3487,0.0]..[0.6094,0.3487,1.7000]`. Test wymaga:

- modelu stojącego na `Z=0`;
- wysokości `1.6..1.8 m`;
- szerokości/głębokości poniżej dwóch metrów.

To blokuje powrót dużego corrupted blobu H1.

### 21.3. Czysty moduł ma jedną fixture, nie trzy mylące modele

Procedural package nie używa już starego minimalnego proof module. Emituje:

- dokładnie jednego creature;
- pozycję `[10,14.5,0]`, bezpośrednio przed entry `[10,10,0]`;
- ten sam `ActiveMonsterBaseline` w GIT i module-local UTC;
- osobny readback klasyfikujący oba zasoby z bajtów;
- dokładnie jeden UTC i brak wygenerowanych H1/Hook Horror controls.

HAK zawiera wyłącznie własny model, teksturę i `appearance.2da`. Poprawna
kontrola jakości nie jest już kolejnym naszym modelem z tej samej błędnej
rodziny.

### 21.4. Zero-terminated extended64 ujawnił błąd web preview

Offline browser readback początkowo odrzucił slot `22`. Writer i Rust reader
miały jednak wzajemnie zgodne mapy. Błąd był w adapterze Three.js: traktował
zera w nieużywanej części 64-elementowej inline mapy jako dalsze aktywne sloty.

Adapter wyznacza teraz aktywne sloty z forward `nodeToBoneMap`, wymaga ich
unikalności i ciągłości, a inline mapę sprawdza tylko w tym zakresie. Jest to
zgodne z obserwowanym native zero-terminated profilem extended64.

Po korekcie realny browser test na exact H2:

- buduje model przez zoptymalizowany WASM i prawdziwy Worker;
- odczytuje 42 stany;
- rekonstruuje SkinMesh oraz inverse binds;
- renderuje poprawną, proporcjonalną i artykułowaną sylwetkę w offline
  readback viewport;
- nie zgłasza błędu mapowania kości.

Ten widok jest dowodem spójności naszego binarnego readbacku, nie dowodem
widoczności w NWN.

### 21.5. Pełna lista problemów po implementacji

| ID | Stan | Problem / wynik |
|---|---|---|
| `VIS-01..VIS-17` | patrz sekcja 20.4 | Poprzednia lista pozostaje historycznie obowiązująca z korektami poniżej. |
| `VIS-18` | naprawiony | Studio/WASM/Worker wcześniej nie udostępniały proceduralnego profilu i nadal wybierały corrupted `H1_SKINNED`. |
| `VIS-19` | naprawiony | Worker miał niebezpieczny fallback do legacy H1 dla nieznanego lane; teraz switch jest fail-closed, a H1 lane jest odrzucany bez artefaktów. |
| `VIS-20` | naprawiony | `cpause1` zachowywał exporter scale mimo jego usunięcia z 41 nowych stanów. Wszystkie 42 stany są teraz unit-scale. |
| `VIS-21` | naprawiony | Poprzedni proof module używał starego minimalnego UTC/GIT. Nowy procedural module ma jedną fixture z readbackiem `ActiveMonsterBaseline` w GIT i UTC. |
| `VIS-22` | naprawiony | Web preview błędnie interpretował zero-filled unused extended64 inline slots jako aktywne kości. |
| `VIS-23` | zweryfikowany offline | Bind reconstruction, skala 1.7 m, ground contact, 42 type-5 states, 23 callbacki i pięć wymaganych deformacji przechodzą na exact owned H2. |
| `VIS-24` | zweryfikowany offline | Prawdziwy zoptymalizowany WASM/Worker i Three.js readback rekonstruują widoczną artykułowaną sylwetkę bez błędów skina. |
| `VIS-25` | blokuje artefakt | Owner-modified `m2a_h2r43.mod` nadal ma 39 090 bajtów, jest zablokowany i nie ma SHA-256. |
| `VIS-26` | hard stop | Nie utworzono r44 ani nowego exact candidate MOD/HAK/model/2DA/resrefu. |
| `VIS-27` | otwarty runtime | Tylko owner proof nowej, tej samej immutable lineage może potwierdzić widoczność i animacje w Toolsecie/NWN. |

Aktualny stan nie brzmi „model działa w NWN”. Brzmi:

`offline implementation complete for the procedural lane; exact runtime
candidate not materialized because the modified r43 lineage is still
unhashable; owner runtime proof pending`.

### 21.6. Pełna regresja workspace

Po naprawie toru proceduralnego wykonano pełne `cargo test --workspace`.
Pierwszy przebieg ujawnił jeden nieaktualny test parsera: przypadek
`additive_trailing_families` nadal traktował flagę AABB `0x200` jako
nieobsługiwaną, mimo że parser obsługuje już AABB i poprawnie wymaga niepustego
drzewa. Test zawężono do nadal odroczonych rodzin AnimMesh i DanglyMesh; nie
osłabiono walidacji AABB.

Końcowy pełny przebieg zakończył się kodem `0`. Razem z wcześniej wykonanymi
testami Core, WASM, Studio, prawdziwego zoptymalizowanego Workera/browsera,
typecheckiem i strict Clippy oznacza to brak znanej regresji offline. Nie zmienia
to `VIS-25..VIS-27`: zablokowany owner-modified `m2a_h2r43.mod` nadal nie ma
odczytywalnego SHA-256, nowa exact lineage nie została zmaterializowana, a
widoczność w NWN pozostaje do decyzji właściciela.

## 22. Wnioski końcowe i obowiązujący stan

### 22.1. Wniosek o wyniku r43

Korekta właściciela jest wiążąca: moduł r43 zawierał uszkodzone rodziny
wygenerowanych modeli, a custom creature nie działał w NWN. Tego wyniku nie
wolno ponownie tłumaczyć jako „zły moduł”, sukces renderowania albo wyłącznie
brak kompletnego proof packetu.

### 22.2. Wniosek o przyczynie

Głównym potwierdzonym błędem procesu i implementacji było uznanie H1 v20 za
wzorzec poprawnego creature. H1 v20 dowodził jedynie wejścia w ścieżkę
renderera i rysował silnie zdeformowany obiekt. r43 odziedziczył z niego
niepoprawną rodzinę:

- jeden rigid mesh zamiast prawidłowego SkinMesh;
- siedem stanów typu 0 zamiast kompletnego kontraktu animacji;
- `supermodel NULL`;
- brak poprawnego, niezależnego wzorca jakości w module.

Generated Hook Horror korzystał z tego samego wygenerowanego harnessu, dlatego
nie był niezależną kontrolą. Dodany przez właściciela natywny blueprint miał
celowo podmieniony Appearance row `15100`; potwierdził problem custom
appearance, ale nie naprawił błędnego modelu.

### 22.3. Wniosek o zaimplementowanej naprawie

Produkcyjny tor nie korzysta już z corrupted `H1_SKINNED`. Dla exact H2
zaimplementowano:

- controllerless Aurora root;
- SkinMesh bezpośrednio pod rootem i weighted skeleton poniżej;
- usunięcie dopuszczalnego stałego uniform scale ze wszystkich stanów;
- 42 odrębne stany typu 5;
- 23 callbacki gameplay;
- walidację bind pose, deformacji, mapowania kości i wymiarów sylwetki;
- czysty moduł z dokładnie jednym `ActiveMonsterBaseline` creature, bez H1 i
  wygenerowanego Hook Horrora.

Ścieżka przechodzi Core readback, WASM, Studio, prawdziwy Worker, zoptymalizowany
browser readback, typecheck, strict Clippy oraz pełne `cargo test --workspace`.
Jest to zakończona weryfikacja implementacji offline, nie proof NWN.

### 22.4. Wniosek o aktualnym blockerze

Nie istnieje jeszcze exact nowy kandydat runtime. Owner-modified
`m2a_h2r43.mod` ma 39 090 bajtów, pozostaje zablokowany przez zewnętrzny proces
i nie ma odczytywalnego SHA-256. Do chwili związania tego dokładnego pliku z
wynikiem porażki nie wolno tworzyć r44, nowego MOD/HAK/modelu/2DA/resrefu.

Stan projektu należy opisywać dokładnie tak:

`implementacja offline toru proceduralnego jest kompletna; kandydat runtime
nie został zmaterializowany; widoczność i animacje w NWN nie są jeszcze
potwierdzone`.

### 22.5. Jedyna dopuszczalna kolejność dalszych działań

1. Po naturalnym zwolnieniu blokady odczytać SHA-256 dokładnego
   39 090-bajtowego `m2a_h2r43.mod`.
2. Uzupełnić candidate-bound record o ten hash.
3. Zmaterializować dokładnie jedną czystą lineage z już zaimplementowaną
   minimalną deltą SkinMesh/42-state/active-monster.
4. Zainstalować exact MOD/HAK wyłącznie zgodnie z no-clobber i hash-verified
   procedurą, jeżeli operacja jest autoryzowana.
5. Przekazać właścicielowi nazwę `.mod`, nazwę modułu, Area, HAK, Appearance
   row, placement i wszystkie hashe.
6. Za wynik runtime uznać wyłącznie proof właściciela tej samej immutable
   lineage w Toolsecie i NWN.

Nie wolno wracać do H1 v20 ani generated Hook Horror jako kontroli poprawności
i nie wolno uznać buildu, offline readbacku lub web preview za dowód działania
w NWN.
