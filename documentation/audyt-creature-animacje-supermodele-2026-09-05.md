**Audyt Creature, animacji i supermodeli — 5 września 2026**

**Ocena: warto rozwijać ten PoC. Rdzeń konwersji jest użyteczny, lecz proces przygotowania animowanego Creature wymaga domknięcia.** Największe braki dotyczą zapisu projektu, świadomego wyboru animacji, edycji eventów, naprawy dopasowania siatki oraz sprawdzenia końcowego eksportu wraz z dziedziczeniem. Tworzenie własnych supermodeli wymaga osobnej funkcji produktu; aktualne zastosowanie istniejącego supermodelu jej nie zastępuje.

To audyt bieżącego drzewa `C:\Projects\meshy2aurora`, łącznie ze zmianami niezatwierdzonymi w Git, przy HEAD `7289f2b0c8d385b2058912fb4fa6c0eb7e1990ba`. Rozwija [audyt ogólny](C:/Projects/meshy2aurora/documentation/audyt-projektu-2026-09-05.md). Nie zmieniono kodu produktu. Uruchomiono wybrane testy oraz reprodukcje syntetyczne offline; nie tworzono kandydatów MOD/HAK ani nowego proofu Aurora/NWN.

Ustalenia z kodu są **wnioskami implementacyjnymi**; reprodukcje opisano osobno. Historyczne wyniki gry są **faktami z zapisanych dowodów właściciela**, związanymi z konkretnymi artefaktami. P1 oznacza blokadę wydania dotkniętej funkcji; P2 — błąd lub ograniczenie do zaplanowanej naprawy. Brak funkcji produktu nie oznacza automatycznie błędu formatu MDL.

**Co faktycznie już istnieje**

| Obszar | Stan i granica |
| --- | --- |
| Humanoid z własnym szkieletem i animacjami | Import GLB, ścisły profil 42 stanów, zachowanie rozpoznanych klipów źródłowych, proceduralne uzupełnianie braków, zapis pochodzenia animacji. Ograniczona selekcja i interpretacja klipów w Studio. |
| Generowanie animacji humanoida | Wdrożone uzupełnianie stanów, death/recovery i kontrola ciągłości pozy, automatyczne eventy. Brak kompletnego interfejsu autorskiej korekty tych wyników. |
| Nowa siatka używająca istniejącego supermodelu | Lokalna biblioteka, dokładny łańcuch zależności i hashe, niezmienna hierarchia i bind pose, nadawanie wag, dziedziczenie animacji, kontrola deformacji przed eksportem. Ograniczenia importu i dwa potwierdzone błędy opisane niżej. |
| Naprawianie rigu | Można numerycznie poprawiać wagi, przypisania komponentów i ograniczenia regionów. Brakuje wygodnego zaznaczania powierzchni i wizualnego authoringu anatomii. |
| Podgląd animacji | Jest odtwarzacz: wybór klipu, play/pause, seek, prędkość, pętla, klucze, poza spoczynkowa. Brakuje edytora kluczy/eventów i scenariuszy przejść; finalny Review ma defekt dziedziczenia. |
| Własny supermodel | Brak pełnej trasy Studio do utworzenia i wersjonowania własnego wspólnego szkieletu, biblioteki animacji oraz modelu pochodnego. |
| Inne anatomie | Można stosować istniejące referencje, ale kompletność i jakość są zależne od pary siatka–referencja. Intake MotionPack rozpoznaje quadrupeda; pełny humanoidalny emitter nie staje się przez to autorskim exporterem quadrupedów. |
| Produkt Creature | Core obejmuje appearance, wyposażenie, grip, blueprint i runtime envelope. Część parametrów, m.in. `motionPack`, `runtimeEnvelope`, `performancePreset`, nie ma ustawień w Studio. |

Własny binary MDL writer/readback, formaty pakietów, pochodzenie zasobów, dokładne zależności i odmowa eksportu złego dopasowania są warte zachowania. Nie ma podstaw do rekomendowania przepisania całego rdzenia. Potrzebna jest konsolidacja trzech procesów: własne animacje, proceduralne uzupełnianie i dziedziczenie z supermodelu.

**Potwierdzone błędy i ograniczenia techniczne**

**1. [P1] Finalny Review nie odtwarza odziedziczonych animacji eksportowanego Creature**

Biblioteka scala animacje z całego łańcucha w [SupermodelPreviewViewport.tsx:15](C:/Projects/meshy2aurora/apps/studio-web/src/features/supermodels/SupermodelPreviewViewport.tsx:15). Finalny Review otrzymuje tylko readback pojedynczego MDL i artefakty: [App.tsx:2371](C:/Projects/meshy2aurora/apps/studio-web/src/App.tsx:2371), [AuroraReadbackViewport.tsx:282](C:/Projects/meshy2aurora/apps/studio-web/src/features/preview/AuroraReadbackViewport.tsx:282). Model reference celowo ma zero lokalnych klipów i 42 odziedziczone; potwierdza to asercja [lib.rs:2394](C:/Projects/meshy2aurora/crates/m2a-wasm/src/lib.rs:2394).

**Skutek:** użytkownik może obejrzeć ruch w bibliotece, ale po eksporcie nie sprawdzi tych samych ruchów na odczytanym wyniku. Komunikat playera o braku klipów w GLB dodatkowo myli format i przyczynę. Dowód: prześledzenie przekazywanych danych, bez uruchamiania scenariusza w przeglądarce.

**Naprawa i akceptacja:** Review odczytanego eksportu korzysta z tego samego łańcucha zweryfikowanego hashami, pokazuje dostawcę każdego klipu i odtwarza animację na finalnej siatce. Po utracie dostępu do zależności wymaga ponownego połączenia konkretnego źródła. Dane referencyjne pozostają lokalnym odczytem; nie trzeba kopiować ich payloadów do projektu.

**2. [P1] Wydzielony render mesh blokuje generyczne zastosowanie supermodelu**

[reference_supermodel_generic.rs:2040](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_generic.rs:2040) wymaga równości liczby wszystkich węzłów referencji i carrierów kontraktu. Tymczasem [reference_supermodel_motion.rs:1181](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_motion.rs:1181) celowo wyklucza render-only węzły bez odziedziczonych kontrolerów transformacji.

**Reprodukcja:** własny syntetyczny binary MDL z writera ma 3 węzły: 2 carriery i 1 oddzielny rigid segment. Reader odczytuje 3 węzły; kontrakt prawidłowo zachowuje 2 carriery. Publiczny immutable ingress kończy się `M2A-REFERENCE-SUPERMODEL-REFERENCE-NODE-COUNT-MISMATCH`, zanim zacznie skinning. Wynik zapisano w [logu reprodukcji](C:/Projects/meshy2aurora/target/audit-supermodel-20260905/repro-output.txt).

**Naprawa i akceptacja:** oddzielić inwentarz renderowania potrzebny do obwiedni od dokładnego inwentarza carrierów. Test root + animowany joint + osobny mesh ma przejść bez zmiany bind pose; analogiczny wariant ze skin meshem wymaga osobnej kontroli. Prawdziwa niezgodność carrierów nadal ma blokować.

**3. [P1 dla zastosowania ASCII] Parser ASCII gubi geometrię i eventy**

[reference_supermodel_generic.rs:1245](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_generic.rs:1245) zawsze ustawia `mesh: None`, `skin: None` i flagi dummy; [linia 879](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_generic.rs:879) tworzy pustą listę eventów.

**Reprodukcja:** własne ASCII zawierające `node trimesh`, `render 1`, `verts 3`, `faces 1` daje `mesh_present=false`, `flags=1`, `unsupported=[]`. Osobny `event 0.02 hit` daje zero eventów. Immutable ingress wymaga obwiedni renderowanej geometrii, więc sama analiza hierarchii nie wystarcza do zastosowania takiej referencji.

To pogłębia diagnozę z audytu ogólnego: niepowodzenie testu WASM ASCII nie jest wyłącznie sprawą starej fixture z trzema dummy. Nowa fixture zawierająca rzeczywistą geometrię również ujawnia utratę informacji. Nie oznacza to problemu z binarnym wyjściowym formatem produktu.

**Naprawa i akceptacja:** albo uzupełnić parser o dane potrzebne do zastosowania referencji i porównać własne fixture ASCII/binary, albo jawnie pokazać w katalogu ograniczenie „analiza struktury”, oddzielając je od „można zastosować”. Eventy muszą być zachowane lub zgłoszone jako nieobsługiwane. Dowody: [repro-output.txt](C:/Projects/meshy2aurora/target/audit-supermodel-20260905/repro-output.txt), [repro-samples-output.txt](C:/Projects/meshy2aurora/target/audit-supermodel-20260905/repro-samples-output.txt).

**4. [P1] Studio wybiera procedurę z liczby i nazw klipów; dodatkowy klip może uniemożliwić eksport**

[App.tsx:1412](C:/Projects/meshy2aurora/apps/studio-web/src/App.tsx:1412) i [directCreatureAnimationProfile.ts:51](C:/Projects/meshy2aurora/apps/studio-web/src/features/source/directCreatureAnimationProfile.ts:51) wymagają dokładnie 42 nazw. Komplet 42 plus `custom_extra` trafia do procedury humanoida. Nieznany klip otrzymuje `m2a_h1_N` w [model_pipeline.rs:2065](C:/Projects/meshy2aurora/crates/m2a-core/src/model_pipeline.rs:2065), potem zostaje odrzucony przez `M6-PROCEDURAL-HUMANOID-CLIP-NAME` w [direct_creature_animation.rs:198](C:/Projects/meshy2aurora/crates/m2a-core/src/direct_creature_animation.rs:198). Istniejący test profilu utrwala odrzucenie dodatkowego klipu.

Przy jednym klipie proceduralny mapping przypisuje go do `cpause1` bez rozpoznania jego znaczenia: [model_pipeline.rs:2059](C:/Projects/meshy2aurora/crates/m2a-core/src/model_pipeline.rs:2059). Klip nazwany `Walk` może więc zostać potraktowany jako idle. Przy wielu dowolnie nazwanych klipach brakuje dostępnego w Studio mapowania MotionPack. Rozpoznane natywne klipy są zachowywane; problemem nie jest automatyczna utrata wszystkich 42 animacji.

**Naprawa i akceptacja:** użytkownik przypisuje plik/klip do stanu NWN, wyłącza niepotrzebne klipy i widzi politykę uzupełnień. Zestawy `Idle/Walk/Run/Attack/Death` oraz 42 + extra przechodzą po świadomym mapowaniu; pojedynczy klip nie dostaje znaczenia bez potwierdzonej reguły lub decyzji autora.

**5. [P2] MotionPack odrzuca wspólny slot animacji dla rozłącznych rodzin broni**

[creature_product.rs:924](C:/Projects/meshy2aurora/crates/m2a-core/src/creature_product.rs:924) wymaga globalnie unikalnego `outputClipName`, ignorując `weaponFamily` przy wykrywaniu duplikatu.

**Reprodukcja offline przez WASM:** syntetyczny GLB i MotionPack `SWORD → ca1slashl`, `AXE_MACE → ca1slashr` przechodzą intake z dwoma klipami. Zmiana tylko drugiego slotu na `ca1slashl` powoduje `CREATURE-MOTION-OUTPUT-DUPLICATE`, mimo rozłącznych rodzin.

Dodatkowo filtr rodziny w [model_pipeline.rs:2120](C:/Projects/meshy2aurora/crates/m2a-core/src/model_pipeline.rs:2120) pomija aktualizację mapowania niewybranej rodziny, ale nie usuwa jej wcześniej utworzonego mapowania. [profile_a.rs:2592](C:/Projects/meshy2aurora/crates/m2a-core/src/profile_a.rs:2592) wymaga mapowania wszystkich klipów źródła. Ten drugi problem potwierdzono statycznie; reprodukcja WASM powyżej dotyczy intake i konfliktu slotów, nie pełnego eksportu wariantów.

**Naprawa i akceptacja:** rozwiązywać rodzinę i selekcję klipów przed kontrolą unikalności finalnych slotów. Jeden zestaw z wariantami miecza, topora i włóczni ma eksportować wybraną rodzinę bez konfliktów i bez obcych `m2a_h1_N`.

**6. [P1] Pełny zestaw klipów może wyjść bez eventów, a proceduralny nie pozwala ich swobodnie poprawić**

Importer tworzy puste eventy w [profile_a.rs:3177](C:/Projects/meshy2aurora/crates/m2a-core/src/profile_a.rs:3177). Automat eventów działa dla profilu proceduralnego w [model_pipeline.rs:3805](C:/Projects/meshy2aurora/crates/m2a-core/src/model_pipeline.rs:3805). Pełne 42 klipy bez JSON-a wybierają normalną trasę `H1_SKINNED_FULL_42` w [App.tsx:1442](C:/Projects/meshy2aurora/apps/studio-web/src/App.tsx:1442), a kontrola eventów jest warunkowa: [model_pipeline.rs:4117](C:/Projects/meshy2aurora/crates/m2a-core/src/model_pipeline.rs:4117).

W drugą stronę: JSON eventów dla źródła mającego kilka klipów zostaje zablokowany przez [App.tsx:1420](C:/Projects/meshy2aurora/apps/studio-web/src/App.tsx:1420). Autor nie może więc poprawić czasu automatycznego uderzenia lub castu po proceduralnym uzupełnieniu. Polityka automatu obejmuje 23 pary klip/event, a czas szacuje z interwałów ruchu wskazanych kości; to punkt startowy, nie ręczna synchronizacja akcji.

**Naprawa i akceptacja:** wspólna warstwa eventów po wyborze i uzupełnieniu klipów. Autor przesuwa `hit`, ustawia kroki i cast, a readback zachowuje dokładne czasy. Kompletność nazw animacji i kompletność wymaganych eventów powinny mieć osobne statusy. To wniosek z kodu; audyt nie przypisuje konkretnemu Creature nowej awarii walki w NWN.

**7. [P2] Ponowne zastosowanie tego samego supermodelu resetuje poprawki rigu**

Przycisk „Analizuj i zastosuj” pozostaje dostępny po edycji. [SupermodelLibrary.tsx:261](C:/Projects/meshy2aurora/apps/studio-web/src/features/supermodels/SupermodelLibrary.tsx:261) wysyła świeże `BUILD_REFERENCE_SUPERMODEL_APPLIED_PREVIEW` bez authoring JSON i bezwarunkowo zastępuje receptę rigu. Osobna operacja [applyAuthoredRig:309](C:/Projects/meshy2aurora/apps/studio-web/src/features/supermodels/SupermodelLibrary.tsx:309) potrafi ją zachować, ale nie jest używana przez ten przycisk. Dowód statyczny.

**Naprawa i akceptacja:** ponowna analiza tej samej referencji zachowuje edycje; reset jest osobną, odwracalną operacją. Test: apply → poprawa wag → ponowna analiza → poprawka nadal występuje w buildzie.

**8. [P2 — luka walidacji] Dziewięć próbek nie pokrywa wszystkich ekstremów ruchu**

[reference_supermodel_motion.rs:10122](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_motion.rs:10122) ogranicza próbkowanie klipu do dziewięciu punktów. Redukcja wybiera je według rozmieszczenia czasu, nie wielkości zmiany transformacji.

**Reprodukcja dokładnego helpera z bieżącego źródła:** klucze `[0, .01, .02, .03, 1]` z krótkim ekstremum przy `.02` dają próbki `[0, .14, .25, .375, .5, .625, .75, .875, 1]`. Cały impuls wypada z kontroli. To dowód luki pokrycia czasowego, **nie** odtworzony pełny false PASS produktu ani wynik w grze.

**Naprawa i akceptacja:** uwzględniać istotne klucze i ekstrema transformacji, adaptacyjnie zagęszczać podejrzane odcinki oraz jawnie raportować pokrycie. Fixture z krótkim obrotem powodującym kolaps między standardowymi fazami musi zostać wykryta. „Sprawdzono wszystkie klipy” nie może sugerować sprawdzenia wszystkich ich ekstremów.

**9. [P2 — luka walidacji] Lokomocja nie ma wspólnej kontroli kontaktu i prędkości**

Kontrola kinematyki proceduralnego humanoida działa tylko na tej trasie: [model_pipeline.rs:4101](C:/Projects/meshy2aurora/crates/m2a-core/src/model_pipeline.rs:4101). Dla dryfu `Hips` porównuje pierwszy i ostatni klucz pozycji: [direct_creature_animation.rs:1988](C:/Projects/meshy2aurora/crates/m2a-core/src/direct_creature_animation.rs:1988). Sam taki pomiar nie rozróżnia poprawnego in-place od `0 → 2 m → 0` wewnątrz klipu.

Brakuje wspólnego dla źródłowych i generowanych animacji pomiaru kontaktu stóp, ich ślizgania oraz zgodności cyklu z `WALKDIST/RUNDIST`. `fourPointContactReady` w intake MotionPack quadrupeda sprawdza obecność semantycznych nazw łap, nie jakość ich kontaktu w ruchu. To ograniczenie dowodu; nie oznacza, że każdy aktualny model ślizga stopy.

**Naprawa i akceptacja:** osobno mierzyć trajektorię root, kontakt, ślizg i prędkość cyklu dla wszystkich obsługiwanych tras. Wyniki pokazywać przy konkretnych klipach, a zgodność zachowania finalnego eksportu zamykać dowodem właściciela w NWN.

**10. [P2] Wybór dawcy appearance zależy od fizycznej kolejności wierszy 2DA**

Studio przekazuje wybrany model i jego dzieci: [App.tsx:1311](C:/Projects/meshy2aurora/apps/studio-web/src/App.tsx:1311). [reference_supermodel_product.rs:342](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_product.rs:342) zamienia kandydatów w zbiór i wybiera pierwszy pasujący wiersz tabeli; później kopiuje parametry poza LABEL/RACE.

**Reprodukcja publicznego buildera:** kandydaci `[c_selected, c_child]`; wcześniejszy wiersz dziecka ma `WALKDIST=0.4`, późniejszy wybranego modelu `WALKDIST=2.0`. Wybrany zostaje `c_child`, row 0. Faktem jest zależność od kolejności tabeli; pierwszeństwo wybranego modelu jest rekomendowanym kontraktem produktu, obecnie niewymuszanym.

**Naprawa i akceptacja:** jawny wybór i zapis dawcy parametrów albo określony deterministyczny priorytet. Przestawienie niezależnych wierszy 2DA nie powinno samo zmieniać parametrów rozmiaru i lokomocji Creature.

**Funkcje, które trzeba dopisać, aby wygodnie tworzyć Creature**

| Funkcja | Luka w bieżącym produkcie | Minimalny użyteczny zakres |
| --- | --- | --- |
| **Zapis i wznowienie projektu Creature** | Stan źródła, ustawienia i rig są w pamięci App; [artifactStore.ts:8](C:/Projects/meshy2aurora/apps/studio-web/src/features/downloads/artifactStore.ts:8) zapisuje ostatnie artefakty, nie projekt. Odświeżenie traci receptę pracy. | Wersjonowany dokument: hashe źródeł, materiały, ustawienia, mapping klipów, eventy, rig overrides, exact chain. Ponowne połączenie brakujących lokalnych plików; odtworzenie tego samego wyniku. |
| **Przygotowanie animacji** | MotionPack i część parametrów są w core/Worker, lecz bez interfejsu autora. Odtwarzacz nie edytuje klipów. | Mapowanie/wykluczanie klipów, wybór idle, jawne uzupełnienia, import własnego motion packa, edytor eventów. Retarget animacji z osobnego szkieletu jako wyraźnie opisana i walidowana funkcja. |
| **Wizualna naprawa siatki i wag** | [RigJointEditor.tsx:183](C:/Projects/meshy2aurora/apps/studio-web/src/features/supermodels/RigJointEditor.tsx:183) wymaga indeksów vertexów i bone IDs; helper landmarków nie jest podłączony do UI. | Zaznaczanie powierzchni/regionów, wizualizacja wpływów, korekta landmarków, edycja wag, undo. Referencyjna hierarchia i bind pose pozostają niezmienne. |
| **Wieloczęściowe Creature i kontrola dopasowania** | Generyczne zastosowanie przyjmuje jeden primitive, bez skina i animacji: [reference_supermodel_generic.rs:1994](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_generic.rs:1994). Rejestracja opiera się na wysokości i dolnym środku. | Obsługa części/primitive i materiałów z zachowaniem tożsamości; kontrola ułożenia oraz korekt mesha względem szkieletu, z ponowną walidacją. Bez cichego odrzucania istniejącego rigu. |
| **Scenariusze zachowania** | Player uruchamia pojedynczą akcję; readback preview nie wykorzystuje pełnej semantyki przejść i eventów. | Idle → walk/run → attack → damage → death, oś eventów, widoczna trajektoria root i kontakty. Później świadome warianty broni i warstwy animacji. |
| **Własne supermodele** | Brak kompletnej trasy authoringu i eksportu własnej wspólnej biblioteki. | Import własnego rigu i animacji, walidacja hierarchii/nazw, własny resref i wersja, lista dostarczanych stanów, tworzenie dziecka oraz test nadpisywania/dziedziczenia. Osobny proces od stosowania cudzej referencji. |

Pierwsza wersja takiej aplikacji nie musi zawierać pełnego edytora rzeźbienia, szkieletu i keyframes. Może przyjmować własne rigowane GLB i animacje przygotowane poza Studio. Musi jednak pozwalać świadomie je wybrać, przypisać, skorygować potrzebne dane, zapisać projekt oraz sprawdzić wynik. Jeśli tworzenie własnych szkieletów ma odbywać się bezpośrednio w Studio, jest to dodatkowy zakres wymagający odrębnego projektu interfejsu i walidacji.

Starszy retarget do animowanego dawcy istnieje w core (`animated_donor.rs`, V1–V5), ale używa osobnej ścieżki z fallbackiem siedmiu stanów, WASM wystawia V1, a Studio nie ma wyboru `animatedDonor`. Nie jest to jeszcze gotowy uniwersalny import animacji. Podobnie [plan nowych animacji broni V2](C:/Projects/meshy2aurora/documentation/plan-implementacji-animacji-nowych-broni-v2-2026-08-27.md) pozostaje planem wobec bieżącego kanonicznego Studio; kod historycznych eksperymentów z innych worktree nie został zaliczony do obecnej funkcjonalności.

**Co zostało dowiedzione na prawdziwych Creature**

| Przypadek i źródło | Co można stwierdzić | Czego nie należy z niego wyprowadzać |
| --- | --- | --- |
| [Powrotnik V4, 28 lipca](C:/Projects/meshy2aurora/documentation/evidence/tlc-powrotnik-death-family-v4-ready-for-owner-proof-2026-07-28.md) | Właściciel potwierdził NWN na dokładnym `tlcpowdemo4.mod` / `tlcpowhak4.hak`: model widoczny, death sequence i wejście do death poprawne. | Nie jest to dowód wszystkich animacji wszystkich modeli. Toolset w tym wpisie pozostaje `not_tested/missing`. |
| [Borzoi + c_wolf, naprawa 2 września](C:/Projects/meshy2aurora/documentation/evidence/borzoi-v10-immutable-supermodel-repair-offline-pass-2026-09-02.md) | Immutable bind 30/30, zero lokalnych i 42 odziedziczone animacje, admission i motion quality PASS offline. | Wpis ma status `offline_product_pass_native_hak_collision`; brak zamkniętego nowego owner proof dla naprawionych bajtów. Kolizja HAK to status zapisany 2 września, nie nowa inspekcja instalacji. |
| [Borzoi + c_direwolf, 2 września](C:/Projects/meshy2aurora/documentation/evidence/borzoi-v10-c-direwolf-application-offline-result-2026-09-02.md) | To samo źródło, pełny bind i 42 inherited; kontrola deformacji blokuje wynik. Admission działa mimo eksperymentalnego obejścia jednego limitu naprawy. | Sam wybór referencji i poprawne pokrycie jointów nie dowodzą dobrego ruchu. Ta referencja pochodzi z opisanej lokalnej kolekcji, nie należy utożsamiać jej z dowolnym retail modelem o podobnej nazwie. |

Historyczne [porównanie Borzoi z 26 sierpnia](C:/Projects/meshy2aurora/documentation/evidence/borzoi-v10-vs-retail-wolf-runtime-video-comparison-2026-08-26.md) pokazało działające dziedziczenie i widoczne problemy deformacji. Nie przenoszę tego werdyktu na nowe bajty naprawy z 2 września. Zestaw wyników pokazuje postęp i jednocześnie brak podstaw do deklaracji „dowolne Creature na dowolnym supermodelu”.

**Rekomendowana kolejność rozwoju**

1. **Domknąć jeden projekt Creature od importu do ponownego otwarcia.** Zapis recepty, brak resetów rigu i materiałów, spójna tożsamość eksportu. Naprawić finalny Review łańcucha oraz oba błędy wejścia supermodeli. Zachować hashe i rozdział gotowości offline od wyniku właściciela.
2. **Dać autorowi kontrolę nad animacjami.** Mapowanie i selekcja klipów, reguły uzupełnień, poprawiony MotionPack, wspólne eventy z edytorem. Warunek ukończenia: zwykłe nazwy źródłowe, niepełny zestaw i 42 + extra dają świadomie zdefiniowany wynik.
3. **Umożliwić naprawę deformacji.** Wizualne regiony/wagi/landmarki, wieloczęściowe wejście i kontrola dopasowania siatki. Uzupełnić próbkowanie ekstremów, root motion, kontakt i scenariusze przejść. Nie podnosić progów wyłącznie po to, żeby pojedynczy przypadek dostał PASS.
4. **Rozszerzyć korpus i dopiero wtedy zakres automatu.** Kilka źródeł na rodzinę: różne proporcje humanoida, quadruped, ogon/skrzydła, części rigid i skinned, dodatkowe materiały. Dla każdego: bind, deformacja, eventy, przejścia, appearance i finalny readback; runtime zgodnie z dowodem właściciela na dokładnych artefaktach.
5. **Dodać autorskie supermodele i zaawansowane warstwy broni.** Osobny dokument projektu, własna biblioteka szkieletów/animacji, stabilne dziedziczenie i jawny wybór pochodzenia każdego stanu. Pełny własny exporter innych anatomii rozwijać jako świadomą funkcję, niezależnie od korzystania z istniejących referencji.

Problemy materiałów pomijanych przez reference route i identyfikatora nieuwzględniającego supermodelu/chain/rigu z audytu ogólnego nadal są zależnościami punktu 1; nie zostały naprawione przez ten audyt.

**Weryfikacja wykonana w tym audycie**

| Sprawdzenie | Wynik |
| --- | --- |
| Studio: `directCreatureAnimationProfile`, `animationPlayback`, `AuroraReadbackViewport`, `exactChain`, `appliedPreview`, `rigAuthoring`, `SupermodelPreviewViewport`, `projectReferenceSupermodelResult` | **8 plików, 32 PASS**. Ostrzeżenia React `act` w środowisku testowym; exit 0. [Log](C:/Projects/meshy2aurora/.codex-tmp/audit-creature-20260905-studio.log). |
| `cargo test -p m2a-core --lib direct_creature_animation -- --nocapture` | **11 PASS**: źródła, proceduralne uzupełnienia, death/recovery, skala i timing eventów. |
| Wybrane suites `reference_supermodel_authoring`, `reference_supermodel_generic`, `reference_supermodel_product` | **21 PASS**: odpowiednio 6, 12 i 3. |
| `cargo test -p m2a-wasm --lib ascii_supermodel_completes_analysis_preview_and_product_offline -- --nocapture` | **1 FAIL** w [lib.rs:2646](C:/Projects/meshy2aurora/crates/m2a-wasm/src/lib.rs:2646): `REFERENCE-RENDER-ENVELOPE-MISSING`. |
| Syntetyczne reprodukcje | Utrata ASCII mesh i eventu; niezgodność render nodes/carriers; pominięcie impulsu przez sampler; kolejność dawcy appearance; konflikt MotionPack między rodzinami broni — potwierdzone w zakresie opisanym wyżej. |
| Nowy proof Aurora/NWN | Nie wykonywano; nie zmieniano statusów historycznych artefaktów. |

Pełne testy i build wykonane wcześniej tego dnia są opisane w audycie ogólnym. Nie powtarzano ich tu ani nie przedstawiano wybiórczych PASS jako zielonej bramki całego repozytorium. Testy nie obejmują jeszcze szeregu powyższych scenariuszy końcowych — ich przechodzenie nie zamyka tych ustaleń.

**Decyzja wynikająca z audytu:** kontynuować ten rdzeń, a najbliższy etap poświęcić spójnemu warsztatowi Creature: zapis projektu → wybór/mapowanie animacji lub supermodelu → korekta dopasowania i eventów → finalny podgląd → eksport. To daje drogę do solidnej aplikacji i mierzalne kryteria ukończenia kolejnych funkcji.
