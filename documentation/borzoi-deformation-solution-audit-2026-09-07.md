# Borzoj / c_wolf — audyt pod naprawę deformacji, 2026-09-07

Status: AUDYT ZAKOŃCZONY; model pozostaje DIAGNOSTIC_ONLY. W tym audycie nie zmieniono modelu, authoringu, progów produktu ani kodu aplikacji. Przygotowano plan implementacyjny i dane do odtworzenia błędów.

## Werdykt

Problem obejmuje zarówno przypisanie powierzchni do kości, jak i kontrolę, która nie obejmuje całego potrzebnego zakresu. Zachowanie dokładnego szkieletu c_wolf jest poprawne, ale nie dowodzi dopasowania psa do niego. Aktualna ścieżka nadaje polu jointFit pewność 1 i residual 0 bez pomiaru lokalnego dopasowania siatki. Rzadkie próbkowanie ruchu oraz budżet liczony na dużym komponencie pozwalają przejść niektórym silnym zapadnięciom.

Poprzedni werdykt „7/42 FAIL” jest prawdziwy dla standardowego raportu, lecz nie opisuje pełnego zakresu problemów. Audyt gęstszych danych znajduje 12 dodatkowych klipów z krawędzią krótszą niż 6,25% długości bazowej, mimo PASS komponentu w standardowym raporcie. W cwalk są 142 próbki krawędź×czas poza zakresem 0,25–4 dla całej powierzchni; wcześniejsze 2 dotyczyły tylko dolnych łap. W crun jest 23 takich próbek poza dolnymi łapami. Te liczniki nie oznaczają liczby klatek, unikalnych wad ani automatycznego werdyktu wizualnego.

## Tożsamość i zakres dowodów

- Źródło: sample-3d/borzoi-tripo-dc22ecb0/source-cwolf-paw-local-final.glb; SHA-256 cc8e663457706c4f105d4af9dd82996d43ad1edb8e0d91b6609e1405c908bfd4.
- Supermodel: c_wolf; SHA-256 a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726.
- Authoring: da58a34f9e0e05a1ceae5858f349fef96729962226a6eb59d86992961bcf32df.
- Własny wynik MDL: 0d02419491fe1d93e91285b7955cf4ef5ffda1e2a3e0eb8601a01a71bc169c50.
- Aplikacja http://127.0.0.1:5186/?creatureAgent=1: WebMCP potwierdził revision 20, powyższe źródło i MDL, canExport=false, DIAGNOSTIC_ONLY. Binding 23/23 jest zgodnością ścieżek animacji.
- Zweryfikowano wszystkie 9 plików z manifestu artifacts/creatures/borzoi-cwolf-paw-repair-20260906/manifest.json: rozmiary i SHA-256 zgodne; źródło także zgodne.
- Ponownie przeanalizowano istniejące raporty z 2026-09-06: standardowy oraz wszystkie unikalne krawędzie przy około 30 Hz dla 42 klipów. Nie uruchamiano ponownie całego zestawu animacji ani buildów. Wykonano świeży odczyt aplikacji i oględziny zatrzymanej pozy ckdbckdie około 0,47 s.
- Dane maszynowe: [audit-evidence.json](../artifacts/diagnostics/borzoi-solution-audit-20260907/audit-evidence.json). Zawierają czasy, pozycje, MDL vertex IDs, odpowiadające im źródłowe vertex IDs (wraz z duplikatami na szwach), wagi i hashe sprawdzonego kodu. Wszystkie zapisane punkty udało się przypisać do źródła z tolerancją 1e-6.

## Ustalenia według priorytetu

### P0. jointFit myli integralność szkieletu z dopasowaniem powierzchni

FAKT Z KODU: crates/m2a-core/src/reference_supermodel_structure.rs:585–706, build_immutable_reference_supermodel_joint_report_v1. Pole _target_bounds jest nieużywane. Pozycje target/reference są tymi samymi punktami szkieletu; residual_fraction=0.0, confidence=1.0 i constraint_verdict=PASS są nadawane konstrukcyjnie. OWNED_MESH_REGISTRATION zależy od anatomy.report.status == READY. Nie ma tu pomiaru przekrojów łapy względem osi kości ani obwiedni tkanki wokół stawu.

FAKT Z RAPORTU: aktualny jointFit ma trzy ograniczenia PASS: inventory, immutable bind i anatomy READY. Wszystkie 30 jointów ma residual 0 i confidence 1. To potwierdza zachowanie riga, nie 100% poprawności anatomicznej. Dodatkowo reference_supermodel_bind_pose.rs:158–168 dla immutable bind nadaje IMMUTABLE_GROUND_CONTACT_SEMANTICS=true, odkładając kontrolę umieszczenia powierzchni na mesh registration.

WNIOSEK: rozdzielić referenceIntegrity od surfaceFit. Zachować niezmienne transformacje. Dodać rzeczywisty pomiar lokalnych przekrojów, środka segmentu, otoczenia przegubu i terminalnej powierzchni. Brak danych ma oznaczać NOT_EVALUATED/NEEDS_AUTHORING. Nie wracać do przesuwania kości c_wolf, żeby pasowały do siatki.

### P0. Standardowy test pomija uszkodzone momenty i część geometrii

FAKT Z KODU: reference_supermodel_motion.rs:10123–10207 redukuje controller keys i midpointy przez rozłożenie w czasie do budżetu 9 próbek. Tylko fazy bazowe i eventy są obowiązkowe; extrema deformacji nie są wyliczane. Linie 10706–10713 wybierają część trójkątów każdego surowego komponentu, a 11039 używa tego podzbioru.

FAKT Z RAPORTU: w tym modelu standard bada 16 817 z 20 534 trójkątów na chwilę, czyli około 81,9%. cwalk ma 9 czasów, audyt regionalny 29. Audyt regionalny bada wszystkie unikalne krawędzie, ale nie jest pełnym audytem pól wszystkich trójkątów.

Dodatkowe klipy przekraczające istniejącą awaryjną granicę edge ratio <0,0625, choć standardowy komponent przechodzi:

| Klipy | Minimum standardowe | Minimum gęstszego audytu | Czas świadka |
|---|---:|---:|---:|
| ca1slashl, ca1stab, creach | 0,08976 | 0,04273 | 0,73333 s |
| cparryl, cparryr, cdodgelr, cdodges | 0,09485 | 0,05935 | 0,13333 s |
| cguptokdb | 0,06300 | 0,01839 | 0,63333 s |
| cgustandb | 0,07154 | 0,02277 | 0,42130 s |
| ccloseh | 0,06535 | 0,05017 | 0,53333 s |
| cclosel | 0,10362 | 0,05559 | 0,90000 s |
| cspasm | 0,06365 | 0,05610 | 0,23333 s |

Każdy z powyższych świadków leży poza standardową listą czasów tego klipu. To potwierdza lukę czasową; oddzielny udział podpróbkowania trójkątów wymaga testu ablacyjnego. Nie należy go zgadywać.

WNIOSEK: szybki podgląd może używać budżetu, lecz końcowe dopuszczenie potrzebuje pełnej geometrii oraz wspólnego zestawu controller/event keys, regularnej siatki czasu i zagęszczenia wokół lokalnych minimów. Wynik niepełnego audytu nie może mieć statusu pełnego PASS. Zachować kontrolę obciążenia przez porcjowanie w workerze, a nie pomijanie pomiarów i nadanie zielonego statusu.

### P0. Błąd lokalny ginie w dużym komponencie

FAKT Z KODU: reference_supermodel_motion.rs:10014–10035 liczy budżet dla całego komponentu. Awaryjny próg długości to 0,25² = 0,0625, a pola trójkąta 0,05² = 0,0025. Przekroczenia zwykłego hard ratio 0,25–4 mogą stanowić do 1% próbek; zapadnięcia pola do 0,05%.

FAKT Z RAPORTU: komponent 0 w cwalk przechodzi z minimum długości 0,08279, minimum pola 0,01511, 54 złymi próbkami krawędzi spośród 402 786 i 9 zapadnięciami trójkątów spośród 134 262. Powierzchnia może więc lokalnie stracić ponad 90% długości albo 98% pola, a zbiorczy wynik pozostaje zielony.

WNIOSEK: dodać stabilne patche powierzchni wokół stawów, podbrzusza, szyi i nasady kończyn; mierzyć najgorszą deformację, rozmiar obszaru i czas utrzymywania się błędu. Nie opierać decyzji wyłącznie na procencie względem całego psa. Nie zamieniać automatycznie każdej bardzo małej fałdki sierści w blocker; kontrola wymaga kalibracji na poprawnych zasobach i rozróżnienia powierzchni nośnej od dodatków.

### P1. Wagi łączą tkanki o różnych zadaniach

FAKTY Z WŁASNEGO MDL I JEGO DEFORMACJI:

| Obszar / klip | Dokładny świadek | Co robią obecne wagi |
|---|---|---|
| Podbrzusze między przednimi nogami / cwalk | MDL edge 11092–11004, 0,23333 s, ratio 0,08279 | Jeden koniec ma ok. 15% wpływu prawej przedniej nogi, drugi ok. 5,4% lewej, mimo położenia przy środku brzucha. Pozostałe wpływy to ribcage/pelvis. |
| Dolna szyja i kryza / cconjure1, ctaunt | 14384–14385, 0,13333 s, ratio 0,04184 | Szyja, głowa i klatka jednocześnie ciągną ten pas powierzchni; head zmienia się z ok. 25,8% do 34,8%. |
| Połączenie prawej przedniej nogi i kryzy / ckdbckdie | triangle 14094/14095/14096, 0,90 s, area ratio 0,001581 | ok. 56–61% Rfrontupperleg i 35–39% neck. Ten sam trójkąt ma ratio 0,003848 w cdead. |
| Lewa tylna stopa / cwalk | dokładne punkty i źródłowe IDs w JSON, chwila 0 s | Pozostałe 2 silne próbki dolnych łap dotyczą stopy; wcześniejsza korekta stawu skokowego nie zamyka tego problemu. |

WNIOSEK: obecne przypisania są lokalne w grafie kości, ale to nie gwarantuje zgodności z anatomią. local_segment_weights w reference_supermodel_skinning.rs:2240–2289 używa odległości do segmentu i bliskości w hierarchii; późniejsze wygładzanie oraz obecny authoring nie usuwają wszystkich błędnych zależności. Nie ustalono, który historyczny etap stworzył każdą konkretną wagę — wymagałoby to porównania etapów. Potwierdzony jest stan końcowy i deformacja.

Rozwiązanie: zaznaczyć semantycznie powierzchnię tułowia, kryzy, rzeczywistych kończyn i stóp, potem wyznaczać dopuszczalne wpływy oraz kontrolowane strefy przejścia po powierzchni. Maski mają być przypisane do źródła i topologii, nie wynikać tylko z tych samych wag, które oceniamy. Dotychczasowe regiony x/y/z są skrzynkami diagnostycznymi: np. right_front_upper obejmuje również środek kryzy. Nie można użyć ich wprost jako masek automatycznej naprawy.

HIPOTEZA DO TESTU: korekta semantycznego przypisania i przebudowa przejść szyja/klatka oraz brzuch/kończyny naprawi większość pozostałych wad. Nie ma jeszcze dowodu, że wszystkie znikną samą edycją wag. Jeśli zgodne anatomicznie wagi nadal powodują zapadanie lokalnej powierzchni, potrzebna będzie miejscowa korekta geometrii/topologii przy stawie, z kontrolą wyglądu i tekstury.

### P1. Kontrola kontaktu łap nie obejmuje pełnego chodu

FAKT Z KODU: reference_supermodel_motion.rs:9395 ustawia paw_gate_applied na podstawie obecności ról łap. inspect_anchor_motion_sample_v2, linie 9018–9036, bada wysokość łap tylko dla is_idle_clip. W lokomocji linie 8993–9016 sprawdzają głównie sprzeczny znak bocznego ruchu próbki względem kontrolera, nie pełny błąd pozycji i kształtu stopy.

FAKT Z RAPORTU: cwalk ma pawGateApplied=true oraz contact=0, side=0, heightError=0. To nie jest pomiar „stopa trzyma podłoże w całym chodzie”. Brak też w tym teście dowodu braku ślizgania podeszwy, skręcenia całej stopy czy penetracji powierzchni.

WNIOSEK: raportować osobno pokrycie testu kontaktu, trajektorii, strony i deformacji. Zbudować wzorzec ruchu stopy ze wskazanej referencji w tej samej przestrzeni i uwzględnić root motion. Fazy podporu, wysokość i przesunięcie oceniać względem referencji; nie wymuszać bezwzględnej nieruchomości łapy w miejscu w animacjach, które zakładają ruch postaci w silniku. W nieodpowiednich klipach N/A z przyczyną, a nie 0 udające wykonany test.

### P1. Narzędzia agenta nie zamykają pętli diagnoza → miejscowa poprawka

FAKT Z APLIKACJI: WebMCP ma source, select_supermodel, prepare, authoring, preview, report i build. Umożliwia hash-bound import pełnego authoringu. Brakuje opublikowanych operacji: ustaw dokładny czas animacji, wybierz świadka wady na powierzchni, pokaż heatmapę lokalnej deformacji, zastosuj małą poprawkę wybranego regionu i porównaj ją z bieżącą wersją. Dopasowanie geometryczne w poprzedniej pracy odbywało się przez narzędzia Node poza interfejsem aplikacji i powrót plików do pipeline.

WNIOSEK: standard WebMCP już działa; potrzeba rozszerzyć konkretne operacje autora i diagnostyki w aplikacji. Sam kolejny interfejs do generowania pełnego pliku wag nie rozwiąże deformacji. Raporty i lokalne operacje muszą używać jednej tożsamości source/chain/authoring/MDL/revision.

## Co wykluczono, a co pozostaje nieudowodnione

- Nie oglądamy przypadkowo starego wariantu: bieżący MDL w aplikacji zgadza się z wybranym artefaktem i raportami.
- W zapisanym audycie wszystkich 42 klipów długości segmentów przednich i tylnych nóg pozostają w zakresie 0,999998927–1,000000834 długości bazowej. To wyklucza rozciąganie tych kości jako przyczynę opisanego problemu. Ruch rootdummy jest odrębnym zagadnieniem; nie należy traktować jego translacji jako rozciągania nogi.
- Zachowane UV, tekstury i poprawny bind nie dowodzą jakości deformacji. Udana próba obrotu w Blenderze także jej nie dowodzi.
- Nie udowodniono potrzeby zastąpienia całego modelu, rezygnacji z psa na rzecz wilka ani ponownego generowania concept artu.
- Audyt regionalny używa tego samego projektowego ewaluatora deformacji co pipeline. Jest niezależnym sposobem próbkowania i agregacji, nie niezależną implementacją silnika NWN. Ewentualna zgodność ewaluatora z silnikiem pozostaje osobną granicą dowodową.
- Obecność sygnału edge ratio poza 0,25–4 w 39/42 klipach nie oznacza, że wszystkie 39 są jednakowo zepsute wizualnie. 7 standardowych FAIL i 12 dodatkowych przekroczeń awaryjnego progu wystarczają, aby utrzymać blokadę produktu.

## Plan implementacyjny

| Etap | Zmiana w aplikacji/pipeline | Test i kryterium odbioru |
|---|---|---|
| 1. Wiarygodna kontrola | Rozdzielić referenceIntegrity/surfaceFit; dodać FINAL_AUDIT obejmujący całą powierzchnię i gęste/adaptacyjne czasy; raportować niepełne pokrycie. Pliki structure, bind_pose, motion, admission oraz worker. | Test negatywny: niezmienny rig + przesunięta łapa nie dostaje surfaceFit PASS. Odzyskać wszystkich 12 zapisanych świadków pominiętych przez standard. Żaden brak pomiaru nie przechodzi jako pełny PASS. Szybki preview pozostaje dostępny jako diagnostyka. |
| 2. Diagnostyka lokalna i operacje WebMCP | Regiony powierzchni, heatmapa, lista błędów z clip/time/vertex/triangle ID, dokładny seek oraz focus na świadku. Stable IDs i mapowanie szwów między GLB a MDL. | Ten sam witness ID i hash otwiera tę samą powierzchnię i czas w UI. Odrzucenie starej rewizji/source/hash. Testy mapowania duplikatów UV bez arbitralnego wyboru jednej kopii. |
| 3. Naprawa semantyki wag | W edytorze zaznaczyć brzuch, kryzę, kończyny i stopy; dopuszczalne wpływy i przejścia po powierzchni. Propozycja zmiany + blokada reszty + cofnięcie, wszystko w authoringu źródła. | Najpierw brzuch/przednie kończyny: cwalk 0,23333 s i ccwalkf/ccwalkb; następnie szyja/klatka: cconjure1, ctaunt, ckdbckdie, cdead; potem tylna stopa i pozycje wstawania. Patch nie zmienia zablokowanych wag ani riga. Brak nowych potwierdzonych wad w 42 klipach. |
| 4. Geometria tylko tam, gdzie potrzebna | Rzeczywiste przekroje i osie na podstawie etapu 1; lokalna korekta otoczenia stawu, a jeśli konieczne siatki. Operacja w authoringu z ograniczeniem zmiany sylwetki. | Porównać wariant „same wagi” i „wagi + lokalna geometria” na tym samym zestawie póz. Nie zmieniać głowy/ogona/tułowia poza zakresem poprawki. Przy zmianie topologii jawne mapowanie UV i tekstury; przy braku jej zmiany dokładna kontrola zachowania. |
| 5. Chód i regresja | Osobne pomiary pozycji, orientacji/kształtu stopy i kontaktu w fazie podporu; root-motion-aware; pełny audyt wszystkich klipów. | Cwalk, crun i 4 kierunki chodu bojowego: przebieg stóp zgodny z referencją, brak potwierdzonych zapadnięć/przebić. Wszystkie awaryjne naruszenia z packetu muszą zniknąć; pozostałe sygnały wymagają oceny obszaru i obrazu. Wzorcowy poprawny zasób nie może zostać odrzucony przez nowy test z powodu spodziewanego ruchu. |
| 6. Dostarczenie modelu | Po zamknięciu kontroli: dokładny GLB/BLEND, binarny produkt Creature, spójny manifest i przygotowanie jednej linii do testu właściciela zgodnie z root AGENTS. | Zgodne hashe authoringu, podglądu i eksportu; zero niesprawdzonych etapów ukrytych pod PASS. Ostateczny test Toolset/NWN wykonuje właściciel. |

Kolejność pracy: najpierw testy negatywne i diagnoza, potem minimalna implementacja. Silnik dopasowania i regiony mają wynikać z wybranego riga i powierzchni, bez numerów kości c_wolf ani progów XYZ borzoja zaszytych w produkcie. Niniejszy packet psa jest przypadkiem regresyjnym, nie nową architekturą tylko dla wilków.

Plan nie obiecuje matematycznej gwarancji 100% wyglądu w każdej chwili. Daje odtwarzalne kryteria, pełniejsze pokrycie, kontrolę regresji i osobny odbiór wizualny. Podnoszenie progów, zmiana nazwy PASS, ukrycie sierści lub podmiana psa nie są naprawą.

## Najbliższy konkretny krok

Zaimplementować etap 1 i minimalną część etapu 2: rozdzielenie surfaceFit od integralności, pełny audyt, raport świadków oraz dokładne ustawienie clip/time. Dopiero na tym wyniku wykonać pierwszy miejscowy patch brzucha/przednich kończyn, który można ocenić na dokładnie tych samych klatkach. Pozwala to przerwać cykl kolejnych poprawek opartych na zielonym, niepełnym teście.
