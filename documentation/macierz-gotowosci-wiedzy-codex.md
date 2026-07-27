# Macierz gotowosci wiedzy

Data: 2026-07-10 | Aktualizacja: 2026-07-11 | Status: AKTYWNY GATE WIEDZY PRZED IMPLEMENTACJA

## 1. Cel

Ten dokument oddziela trzy stany, ktore wczesniej byly mieszane:

- `DIRECTION_LOCKED` - wiemy, jaka sciezka implementacyjna ma byc uzyta;
- `EVIDENCE_PARTIAL` - kierunek jest ustalony, ale konkretny wariant binarny wymaga readbacku albo proofu;
- `RUNTIME_PROVED` - wygenerowany przez nas asset przeszedl Toolset i gre.

`P-REF` jest osobnym dowodem kompatybilnosci readera/preview na wskazanym modelu NWN: zawiera hash, raport naszej implementacji, invariants i - gdy istnieje odpowiedni preview - widoczny artefakt. Nie zastepuje `RUNTIME_PROVED` dla wygenerowanego outputu.

Brak etapu M4/M5 w biezacym sprincie nie oznacza zgody na brak wiedzy. Przed M1A kierunek dla calego natywnego pipeline ma byc zapisany, a otwarte kwestie musza miec jednoznaczny test zamykajacy.

## 2. Stan wiedzy

| Obszar | Stan wiedzy | Kierunek | Co pozostaje do proofu |
|---|---|---|---|
| Produkt i granice repo | `DIRECTION_LOCKED` | Web local-first, Rust/WASM, `meshy2aurora` standalone | Brak; decyzja produktowa zamknieta |
| Binary MDL reader | `DIRECTION_LOCKED` | Checked little-endian reader, core pointers wzgledem bajtu 12, raw pointers wzgledem bloku MDX | M1A/M1B readback na syntetycznych fixture i opcjonalnych lokalnych referencjach |
| Binary MDL writer | `EVIDENCE_PARTIAL` | Writer sklada `12-byte header + core + volatile`, tylko profil A i jawnie wspierane typy nodow; M1B rozstrzygnal dla readera warianty skin `legacy17`/`extended64`, a writer emituje jawnie tylko `extended64`. P0 `meshType=0 -> 3` jest zamkniete jako source/readback dla M4 i M0; P0 local-animation `type=5` jest takze zamkniete jako source/readback, lecz hash-bound r21 nadal nosi starsze `type=0`. Globalny ASCII case-fold gate nazw nodow jest zamkniety jako source/readback: writer odrzuca kolizje rig--rig i rig--generowany mesh przez `M4-NODE-NAME-DUPLICATE`. `SkinMeshDummy` (state leaf `0x01` o nazwie base skina) ma liczne native witnessy i nie jest bledem do usuniecia; matching state-skin `0x61` jest osobnym, opcjonalnym profilem. Brak M4 vertex colors (`+0x248=-1`) jest legalnym sentinelem, a binarny controller `type=20` to quaternion `XYZW`, nie NeverBlenderowy ASCII axis-angle. Raw binary witness potwierdza formule skin q/t `inverse(boneWorld) * skinWorld` oraz WXYZ, ale nie deformacje runtime. `boneconstantindices` sa 4-bajtowymi slowami; parser i publiczny readback uzywaja lossless `u32`, a mutacja wysokiej polowki daje nazwany semantic diff. Env-gated invariant na exact CEP HAK potwierdza trzy profile state-skin w podzbiorze obslugiwanym przez strict reader i jawnie raportuje pozostala luke readera. | Wlasna M6 skin+motion fixture ma teraz CPU oracle translacji, hierarchii i quaternion slerp; nadal wymagany jest hash-bound Toolset/NWN proof emitera. Wizualna deformacja skinu, niezmieniony legacy17 output i wsparcie pozostalych `1279` binarnych modeli CEP pozostaja otwarte |
| Reference corpus MDL | `DIRECTION_LOCKED` | Synthetic CI + env-gated in-place corpus wielomodelowy + `P-REF` packets + own generated runtime proof | M1B inventory wybiera i hashuje R4-R6 bez payloadow w Git |
| MDX | `DIRECTION_LOCKED` dla profilu A | Jeden zasob HAK typu 2002; MDX jest doklejonym blokiem volatile, bez osobnego typu 2003 | Wlasny write/readback i NWN EE proof |
| Animacje | `DIRECTION_LOCKED`, semantyka gameplay `EVIDENCE_PARTIAL` | Produktowy proof ma byc self-contained; own-readback obejmuje binarne naglowki, `animroot` i state trees. V2 wymaga exact 42-state namespace bez idle fallback, controller content wszystkich stanow, zgodnego native motion floor, roznych semantyk kluczowych stanow, terminalnej pozy `ckdbckdie` oraz nierigid SkinMesh deformation dla pieciu kluczowych klipow. Trzy niezalezne local binary families potwierdzaja namespace i behavior floor. Legacy V1 zachowuje jawny 7-state gameplay-floor dla zamrozonych lineage. | Runtime mapowanie eventow i petli/state routing, artystyczna poprawnosc ruchu oraz hash-bound owner proof wizualnego zachowania i deformacji |
| `appearance.2da` | `EVIDENCE_PARTIAL` | Uzytkownik dostarcza jawny base table lub HAK; produkt musi jawnie rozroznic pelny runtime table od celowo skroconego Toolset vertical-slice. W obu wariantach zachowuje kolumny i nie zmienia zachowanych wierszy. | Proof, ze wybrany `MODELTYPE=S`, `RACE` i `Appearance_Type` sa rozwiazywane przez gre, oraz test zakresu: skrocony zasob nie moze zostac wydany jako ogolny runtime HAK. |
| HAK/ERF | `DIRECTION_LOCKED` | Wlasny HAK V1.0 writer w Rust; deterministyczne key/resource lists; zasoby bez kompresji | Readback i NWN EE proof |
| GFF/UTC/IFO | `DIRECTION_LOCKED` strukturalnie | Wlasny GFF V3.2 writer; wygenerowany UTC i minimalny modul; `Mod_HakList` zachowuje kolejnosc | Minimalny zestaw pol UTC/GIT/IFO i runtime proof |
| Priorytet wielu HAK | `EVIDENCE_PARTIAL` | Pierwszy proof uzywa jednego wygenerowanego HAK, wiec nie zalezy od niepotwierdzonej kolejnosci override | Osobny test konfliktu dwoch HAK, jezeli produkt zacznie je laczyc |
| Licencje i provenance | `DIRECTION_LOCKED` procesowo | Zrodla GPL i zasoby gry sa reference-only; nie kopiujemy kodu ani payloadow; proof fixtures sa generowane | Wlasciciel wybiera licencje repo przed publicznym wydaniem |
| Finalna akceptacja | `NOT_PROVED` | Toolset i gra sa jedynym finalnym proofem natywnego outputu | M6 |

### Aktualizacja P0 animation header -- 2026-07-21

`Binary MDL writer` ma drugi potwierdzony P0 obok M4 `meshType=0`:
`emit_animations()` pozostawia `GeometryHeader+0x6c` jako zero. Aurora kopiuje
ten pojedynczy bajt przy materializacji state, a bounded, read-only corpus
`cep3_core1` ma `type=5` we wszystkich `24 707` binary local-animation headers
(`1182` modeli z lokalnymi animacjami; dalsze `87` zasobow type `2002` to ASCII
MDL, nie binarne naglowki). Test-first ma zmienic readback z `runtime_6c == 0`
na `== 5`, wykryc mutacje jedynie tego bajtu, zachowac zerowy padding
`+0x6d..+0x6f`, a dopiero potem zapisac `u32 5` pod `+0x6c`.

To jest korekta binary ABI, nie runtime proof: po implementacji wymagany jest
kontrolowany, hash-bound proof NWN na tym samym M0. Pelny lancuch zrodel,
zakres i odrzucone hipotezy sa w
`nwn-model-visibility-research-synthesis-2026-07-21.md`, sekcja 44.

### Stan implementacji P0 -- 2026-07-21 (offline)

Aktualny snapshot drzewa roboczego zapisuje `u32 5` pod
`animation.header + 0x6c`, czyli ABI `animation_type: u8 == 5` oraz trzy
zerowe bajty paddingu. Semantic readback i test M0 rozdzielaja te pola.
Negatywny test semanticzny juz zapisuje zero w zakresie `+0x6c..+0x70` i
wymaga dokladnie diffa `animations[0].header`; poniewaz padding baselineu
jest zerowy, zmienia to materialnie tylko `type: 5 -> 0`. Osobne mutacje
parsera potwierdzaja niezalezne odczyty type i paddingu. Dwa waskie testy
`model_pipeline` przeszly, ale hash-bound artefakt r21 pozostaje starszym MDL
z `type=0`. P0 jest zatem zamkniety jedynie jako source/readback; przed
jakimkolwiek claimem runtime trzeba odswiezyc ten sam packet logiczny i
zachowac cala reszte bindingu M0 bez zmian.

### Aktualizacja P0 M4 `meshType` -- 2026-07-21 (offline)

Starszy wiersz tabeli opisuje stan sprzed poprawki `M4 meshType=0 -> 3`.
Aktualny `M4DirectCreatureExtended64V1` wybiera juz `3`, zapisuje je pod
`mesh+0x224`, a own readback i testy M4 rigid oraz extended64 skin to
egzekwuja. Negatywna mutacja tylko tego pola do `0` daje semantic diff
`nodes[3].profileDefaults`. Werdykt: P0 jest **zamkniete jako source/readback**;
runtime M4/H1 pozostaje osobnym, nieprzeprowadzonym proofem aktualnego
pelnego packetu i nie wolno przypisywac mu skutku historycznej wartosci `0`.
Podstawa: Aurora `FUN_00a62954` wpisuje `3` pod `mesh+0x224` przy budowie
trojek indeksow, xoreos czyta `3` jako Triangle, a NeverBlender eksportuje
Trimesh jako triangulowane faces. Szczegoly w syntezie, sekcja 45.

### Korekta P1 state-skin -- 2026-07-21

Zastepuje wiersz `Binary MDL writer` w czesci dotyczacej postulatu usuniecia
`SkinMeshDummy`. Wczesniejszy witness `c_squirrel` nadal jest prawdziwy, ale
jest tylko jednym z profili native: ma zero state-skinow. Szerszy, read-only
sweep tego samego HAK-a (`3430` binary MDL, SHA-256 HAK
`6a8e6a64773a77fd46740cbcce19a708db6a70b4975732d0405978f3fbe8eb1a`) wykazal:

- `137` modeli ma jednoczesnie base skin i local animations; `123` z nich nie
  ma skin node'a w state tree, a `14` ma matching-name state skin `0x61`
  (`1211` node'ow);
- niezaleznie wystepuje `3029` state node'ow `0x01` o nazwie jednego z
  base-skinow, w `43` modelach; `3015` z nich jest bezdzieciowe i nie ma
  kontrolerow -- czyli ma ten sam minimalny layout, ktory obecnie emituje
  `SkinMeshDummy`.

Wniosek: `0x01` nie jest sam w sobie bledem exportu, a usuniecie
`SkinMeshDummy` nie jest juz poprawka P1. `0x61` w stanie jest drugim,
opcjonalnym native profilem, ktorego nie wolno emitowac bez osobnego kontraktu.
Aktualny M0 jest rigid i nie ma skina, wiec oba profile sa poza jego zakresem.
Najmocniejszym pozostajacym offline kandydatem M0 jest `type=5` w local-
animation headerze opisany powyzej. Szczegoly, negatywny wynik i przyszly test
sa w sekcji 41b syntezy badawczej.

### Korekta P1 `boneconstantindices` -- 2026-07-21

Wiersz `Binary MDL writer` wymaga doprecyzowania: aktualny parser nie traci
bitow czterobajtowego wpisu `boneconstantindices`. Rezerwuje 4 B i odczytuje
dwa kolejne `i16`; para zachowuje dokladnie te same bajty, choc nie jest
wlasciwa, semantyczna nazwa nieznanego pola. Obecny writer swiadomie emituje
`0u32` jako dwa zerowe halfwordy, co daje identyczne bajty.

Aktualizacja 2026-07-25: P1 zostal zamkniety w source/readback. `SkinReport`
i semantic projection uzywaja `Vec<u32>`, parser czyta cale slowo little-endian,
a test zapisuje `0x89abcdef` i wymaga lossless readbacku. Osobna mutacja
**wysokiej** polowki slowa wymaga dokladnie diffa `skin.constants`. Writer nadal
emituje `0u32`, czyli identyczne cztery bajty jak przed retypowaniem; frozen
r43 zachowal dokladne hashe. Jest to poprawa API/readbacku, nie udowodniona
semantyka constants ani poprawka renderera.

### Aktualizacja offline deformation oracle -- 2026-07-25

Dodano `evaluate_skin_deformation_v1`, ktory pracuje wyłącznie na wyniku
wlasnego binary MDL readera. Oracle odbudowuje bind i animowane world matrices,
interpoluje translation oraz quaternion `XYZW`, rozwiazuje sloty przez
`node_to_bone_map`, stosuje WXYZ inverse bind, bone refs i cztery wagi, a
nastepnie raportuje bind/sample world position kazdego vertexa.

Pokrycie:

- ruch translacyjny jednej kosci i dziedziczenie ruchu przez jej dziecko;
- mieszane wagi `1/2/4` i poprawne sumowanie potomnych wplywow;
- quaternion slerp w polowie klipu;
- fail-closed dla modelu bez SkinMesh;
- fail-closed dla zerowego/niefinitywnego/nieunitarnego inverse-bind quaternionu;
- pelny owned M6 GLB -> Profile A -> binary MDL/MDX -> own readback -> skin
  motion sample z niezerowym ruchem.

To zamyka brak offline composition oracle, ale nie promuje stanu do
`RUNTIME_PROVED`: zamkniety renderer NWN i finalna wizualna deformacja nadal
wymagaja owner proof.

### Aktualizacja env-gated state-skin corpus -- 2026-07-25

Test `cep_state_skin_profiles_are_preserved_and_reader_coverage_gap_is_explicit`
przeszedl na exact `cep3_core1.hak` o SHA-256
`6a8e6a64773a77fd46740cbcce19a708db6a70b4975732d0405978f3fbe8eb1a`.
HAK zawiera `3517` zasobow MDL i `3430` payloadow z sygnatura binarna. Strict
reader przyjmuje obecnie `2151`, a `1279` odrzuca; test raportuje te liczby
oddzielnie, wiec ograniczona obsluga formatu nie moze zostac pomylona z pelnym
pokryciem korpusu.

W przyjetym podzbiorze potwierdzono trzy wymagane profile:

- `66` z `72` modeli z base SkinMesh i animacjami nie ma state SkinMesh;
- `6` modeli ma matching-name generic `0x01`: `214` node'ow, wszystkie
  bezdzietne i bez kontrolerow;
- `6` modeli ma matching-name state SkinMesh `0x61`: `565` node'ow.

To zamyka brak automatycznego witnessa kazdego profilu dla wspieranego
podzbioru. Nie zastepuje historycznego, tolerancyjnego sweepu calego korpusu
ani nie zamyka luki strict readera obejmujacej `1279` modeli.

### Aktualizacja full 42-state animation profile -- 2026-07-25

Dodano wersjonowany `DirectCreatureAnimationProfileV1`:

- `GameplayFloor7IdleFallbackV1` zachowuje dotychczasowy, jawny profil V1 i
  jego clean-room idle fallback dla zamrozonych lineage;
- `FullNative42ExplicitV1` wymaga wszystkich 42 jawnie zmapowanych source
  animations i nigdy nie tworzy ataku, ruchu, obrazen ani smierci przez zmiane
  nazwy `cpause1`.

Exact lista 42 nazw jest env-gated sprawdzana jako ten sam zbior w trzech
niezaleznych rodzinach binarnych: retail `c_Direwolf`, retail `c_horror` oraz
CEP R3 `c_phod_horror_b`. Nie sa kopiowane keyframes, szkielety, eventy ani
payloady referencyjne.

Core udostepnia explicit-profile builder i automatyczny H1 V2. Ten drugi
kanonizuje tylko source animation names zgodne ASCII-case-insensitive z exact
namespace; nieznana nazwa pozostaje poza profilem i powoduje named missing-
state error. Raport V2 zapisuje `required=42`, `explicit=42`, `fallback=0`.
WASM eksportuje `buildMeshyH1ModelPackageV2`, Worker ma lane
`H1_SKINNED_FULL_42`, a Studio wybiera go wyłącznie dla dokladnie kompletnego
zbioru bez brakow, duplikatow i nadmiarowych klipow.

Aktualizacja tego samego dnia dodaje behavior i SkinMesh gates. Behavior oracle
pracuje na exact output own readera i wymaga:

- controller content dla wszystkich 42 stanow;
- ruchu w kazdym stanie aktywnym, dla ktorego retail `c_Direwolf`, retail
  `c_horror` i CEP R3 sa zgodne;
- roznych semantyk idle, walk, run, glownego ataku, damage, death transition
  oraz dead state;
- terminalnej pozy w `ckdbckdie`.

To koryguje wczesniejsze zalozenie o `cdead`: trzy rodziny pokazuja, ze
`cdead`, `cgetmidlp` i `ccastoutlp` moga byc statyczne albo ruchome zależnie od
rodziny. Wymagany jest ich jawny controller content, lecz nie globalny
motion/stillness rule.

Drugi gate probkuje po own binary readbacku `cwalk`, `crun`, `ca1slashl`,
`cdamagel` i `ckdbckdie`, wymaga co najmniej dwoch aktywnych jointow i
rzeczywistej zmiany odleglosci miedzy wazonymi vertexami. Pakiet, ktory ma 42
zmienne kontrolery, ale wykonuje tylko rigid transform root, jest odrzucany
przez `M6-SKIN-ANIMATION-INELIGIBLE`.

To zamyka brak offline behavior/deformation admission dla full V2, lecz nie
jest dowodem artystycznej poprawnosci, event callbackow, engine loop/state
routingu ani widocznosci w NWN. Te elementy nadal wymagaja hash-bound owner
proof po zamknieciu exact r43 visibility branch.

### Aktualizacja caller-owned event authoring -- 2026-07-25

Event payload nie jest juz luka implementacyjna. Exact wspolna czesc eventow
retail `c_Direwolf`, retail `c_horror` i CEP R3 zawiera 23 pary clip/event.
`DirectCreatureAnimationEventProfileV1::CommonNativeGameplayHooksExplicitV1`
uzywa tej listy jako opt-in coverage floor. Czasy pozostaja caller-owned i nie
sa kopiowane ani inferowane z witnessow.

Core V3 przyjmuje strict `DirectCreatureEventAuthoringV1`, zapisuje eventy do
binarnego MDL i wymaga `23/23` po own readbacku. Raport zachowuje conformance
oraz kanoniczny hash sidecara. WASM V3, Worker
`H1_SKINNED_FULL_42_EVENTS` i Studio obsluguja ten sam jawny tor. Legacy V1/V2
pozostaja bez zmian.

Aktualne pole "co pozostaje do proofu" dla animacji nalezy czytac jako:
runtime callback semantics, petle/state routing, artystyczna poprawnosc oraz
hash-bound owner proof zachowania i deformacji. Samo mapowanie i zapis event
payloadu sa zamkniete offline.

## 3. Kanoniczne dokumenty wiedzy

```yaml
knowledge_contracts:
  mdl: "documentation/mdl-binary-crosswalk-codex.md"
  mdx: "documentation/mdx-polityka-codex.md"
  animation_profile_a: "documentation/animacje-kontrakt-profil-a-codex.md"
  hak_2da_gff: "documentation/hak-2da-gff-crosswalk-codex.md"
  mdl_reference_corpus: "documentation/korpus-referencyjny-mdl-codex.md"
  live_state: "documentation/orchestrator-state.yaml"
```

Starsze dokumenty nadal zachowuja kontekst, ale powyzsze kontrakty maja pierwszenstwo, gdy starszy tekst mowi `NIE WIEM`, wybiera wolny wiersz 2DA albo opiera produkcyjny proof na `aurora-web`.

Aktualizacja faktow 2026-07-11: checklist entry w `decyzje-i-zadania-cloud.md` o wyciagnieciu retail `appearance.2da` jest juz merytorycznie wykonany przez read-only KEY/BIF inspection (`nwn_base.key` -> `data/base_2da.bif`, type 2017). Pliku `*-cloud.md` nie zmieniamy zgodnie z regulami wspolpracy; aktualne fakty sa w `aurora-2da-creature-codex.md` i `hak-2da-gff-crosswalk-codex.md`.

## 4. Warunek startu implementacji

M1A nie wymaga runtime proofu M4-M6, ale wymaga spelnienia obu warunkow:

1. kierunek dla kazdego obszaru jest zapisany bez zgadywania;
2. kazda nierozstrzygnieta roznica ma nazwany test zamykajacy i nie przecieka jako stale zalozenie do kodu.

Aktualnie warunek wiedzy jest spelniony. Operacyjny start M1A nadal wymaga toolchainu Rust/WASM i jawnej zgody wlasciciela zgodnie z `audyt-gotowosci-startowej-2026-07-10-codex.md`.
