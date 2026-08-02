# Audyt motion retargetingu, plan implementacji i kryteria ukończenia

Data audytu: 2026-08-01

Workspace: `C:\Projects\meshy2aurora`

Zakres: wielokrotne używanie animacji wygenerowanych dla jednego humanoidalnego
modelu Meshy na innych humanoidalnych Creature, bez ponownego zamawiania tych
samych ruchów dla każdego modelu.

## 1. Werdykt

Motion retargeting w rozumieniu użytkownika **nie jest obecnie kompletną
funkcjonalnością produktu**.

Projekt potrafi dziś:

1. pobrać od 1 do 10 animacji Meshy wygenerowanych dla **tego samego** modelu;
2. rygorystycznie sprawdzić identyczność rigu, skinu i siatki tych plików;
3. scalić ich klipy do jednego GLB;
4. przeliczyć animacje tego modelu do jego własnego rigu Aurora;
5. zachować jawne klipy źródłowe i uzupełnić pełną przestrzeń 42 stanów Creature;
6. zapisać animacje w binary MDL oraz przeprowadzić semantic readback.

Projekt nie potrafi jeszcze bezpiecznie:

1. wyodrębnić z jednego modelu sam ruch jako wersjonowany pakiet biblioteczny;
2. sprawdzić zgodności semantycznej rigu dawcy i rigu odbiorcy;
3. przenieść rest-pose-relative motion z dawcy na inny, już oskórowany model;
4. przeskalować ruch bioder do innych proporcji ciała;
5. zachować kontakt stóp z podłożem i skorygować rozkrok/ślizganie;
6. pokazać tę operację jako zwykłą, audytowalną funkcję w Studio;
7. dowieść jej na korpusie różnych sylwetek w Aurora Toolset i NWN.

Wniosek: pomysł jednej biblioteki ruchów Meshy jest technicznie prawidłowy i
powinien zmniejszyć liczbę płatnych generacji. Nie wolno jednak kopiować
surowych quaternionów i translacji pomiędzy modelami ani użyć istniejącej
ścieżki `animated_donor` jako substytutu. Potrzebny jest osobny, motion-only
pipeline.

## 2. Zakres faktów i granice audytu

### 2.1 Potwierdzone w kodzie

- `tools/meshy-local-bridge/merge-animation-glbs.mjs` przyjmuje od 1 do 10
  nazwanych GLB i przed scaleniem wymaga identycznych nodów, skinów, jointów,
  inverse bind matrices, meshy, materiałów, atrybutów i indeksów. Jest to
  poprawny **same-model animation merge**, nie cross-model retargeting.
- `apps/studio-web/src/features/meshy/bridge.ts` i Local Bridge mają limit 10
  akcji na jeden run. Domyślnie mapują Meshy `actionId: 0` na `cpause1`.
- `crates/m2a-core/src/profile_a.rs::emit_animation_set_v1` stosuje korektę
  rest pose oraz zmianę bazy współrzędnych podczas mapowania animacji źródła
  na rig wynikowy tej samej konwersji.
- `crates/m2a-core/src/direct_creature_animation.rs` wymaga dokładnie jednego
  `cpause1`, zachowuje jawne klipy, normalizuje rodzinę zgonu i uzupełnia pełne
  42 nazwy stanów. Proceduralne klipy są budowane od pierwszej klatki idle.
- `crates/m2a-core/src/animated_donor.rs` implementuje inną operację:
  **statyczna siatka -> rig, wagi i animacje osobnego dawcy**. V1-V4 przenoszą
  skinning dawcy, a V5 zamienia go na sztywne grupy trójkątów. Ta ścieżka nie
  zachowuje skinningu gotowego modelu docelowego i nie jest właściwą podstawą
  biblioteki ruchów.
- WASM wystawia historyczne
  `retargetStaticMeshToAnimatedDonorV1`, lecz Studio nie ma kompletnego
  workflow wyboru biblioteki ruchów i przeniesienia ich na rig gotowego
  Creature.
- Obecne raporty Creature mierzą kompletność 42 stanów, ciągłość wymaganych
  przejść, rodzinę zgonu i lineage klipów. Nie ma bramki neutralnego idle,
  dystansu stóp, foot contact ani foot sliding.

### 2.2 Potwierdzone na lokalnym korpusie Meshy

Co najmniej pięć aktualnych manifestów humanoidalnych Creature (`Fogbound`,
`Powrotnik P20K`, `Stoneback`, `Veiled`, `Void Crystal Knight`) deklaruje
T-pose oraz Meshy `action_id: 0` jako NWN `cpause1`. Dlatego styl pozy z akcji
0 jest obecnie propagowany do stanu spoczynkowego, a przez bazę proceduralną
również do części klipów uzupełnianych przez aplikację.

Porównanie wcześniejszych wejściowych GLB z plikami po merge wykazało, że merge
nie dodaje rozkroku: zachowuje klucze, wartości, czasy i interpolację animacji.
Oznacza to, że retargeter musi mieć jawny wybór neutralnego dawcy oraz bramkę
kinematyczną; samo ponowne scalenie klipów nie naprawi pozy.

### 2.3 Niepotwierdzone i wymagające implementacji/testu

- Nie jest jeszcze potwierdzone, czy wszystkie przyszłe Meshy H1 zachowają
  identyczny zestaw nazw kości, hierarchię i osie lokalne.
- Nie jest jeszcze wyznaczony właścicielsko zaakceptowany próg neutralnego idle
  oraz kontaktu stóp dla Creature w NWN.
- Nie ma owner proof, że ruch przeniesiony pomiędzy różnymi sylwetkami wygląda
  poprawnie w Aurora Toolset i NWN.

Te trzy punkty są bramkami, nie założeniami do zgadywania.

## 3. Rozdzielenie trzech różnych operacji

| Operacja | Stan | Co zachowuje | Do czego służy |
|---|---|---|---|
| Same-model merge | jest | dokładny rig, skin i mesh bazowego GLB | zebranie wielu akcji Meshy tego samego modelu |
| Static mesh -> animated donor rig | jest jako V1-V5 | geometrię statycznego źródła, ale nie jego istniejący skinning | historyczne ratowanie nieoskórowanej siatki |
| Motion donor -> rig gotowego Creature | brak | **musi zachować geometrię, UV, materiał, jointy i wagi celu** | biblioteka ruchów używana na wielu modelach |

Nowa funkcja musi być trzecią operacją. Nie może być kolejną wersją
`retarget_static_mesh_to_animated_donor`.

## 4. Pełna lista braków

### P0 — konieczne do bezpiecznego użycia

1. **Brak wersjonowanego `MotionPackV1`.** Nie istnieje motion-only schema z
   rest pose, semantyką kości, klipami, polityką root motion, eventami i pełnym
   provenance Meshy.
2. **Brak skeleton compatibility gate.** Nie ma raportu mapowania dawca -> cel,
   wykrywania brakującej/zdublowanej kości, złej hierarchii, odbicia osi,
   niejednorodnej skali i niezgodnych attachment nodes.
3. **Brak cross-model rest-pose correction.** Obecna korekta działa wewnątrz
   pojedynczej konwersji, nie pomiędzy dwoma niezależnymi rigami.
4. **Brak motion-only invariant.** Nic nie gwarantuje, że operacja nie zmieni
   POSITION, indeksów, UV, materiałów, JOINTS, WEIGHTS, inverse bind matrices,
   stabilizacji accessories albo liczby trójkątów celu.
5. **Brak polityki translacji.** Surowa translacja Hips/Root nie jest skalowana
   do wzrostu i długości nóg odbiorcy; translacje innych kości nie mają jawnej
   polityki preserve/reject.
6. **Brak foot contact i stance gate.** Nie wykrywamy interwałów podparcia,
   penetracji podłoża, unoszenia, ślizgania ani przesadnego rozkroku.
7. **Brak korekcji kończyn.** Dla różnych proporcji potrzebny jest ograniczony
   solver nóg, a dla ruchów z bronią co najmniej audyt dłoni i punktów chwytu.
8. **Brak integracji z pełnymi 42 stanami.** Retarget musi wejść przed
   materializacją stanów proceduralnych, aby poprawny `cpause1` był ich bazą,
   i nie może zepsuć istniejących reguł zgonu oraz przejść.
9. **Brak API Native/WASM/Worker.** Nie ma kontraktu żądania zawierającego
   pakiet ruchu, model celu, opcje i deterministic result identity.
10. **Brak Studio workflow.** Użytkownik nie może utworzyć/importować pakietu,
    wybrać go dla Creature, zobaczyć compatibility report ani zaakceptować
    polityki root/feet.
11. **Brak korpusu regresyjnego różnych proporcji.** Obecne testy
    `animated_donor_retarget` dowodzą innej operacji.
12. **Brak owner proof cross-model.** Nie ma exact MOD/HAK pokazującego ten sam
    pakiet ruchów na co najmniej trzech różnych Creature.

### P1 — wymagane przed nazwaniem funkcji produkcyjną

1. Biblioteka nie ma batch assembly ponad limit 10 akcji jednego runu Meshy.
2. Nie ma semantycznych klas klipów: idle, locomotion, attack, reaction,
   knockdown, recovery, terminal death i emote.
3. Nie ma per-clip polityki loop, in-place/preserve root motion, transition ani
   terminal hold.
4. Nie ma wersjonowania algorytmu i automatycznego unieważniania starych
   wyników po zmianie retargetera lub profilu jakości.
5. Nie ma deduplikacji pakietów po hashach ani raportu, że dany ruch został już
   pobrany i nie wymaga kolejnych kredytów Meshy.
6. Nie ma porównania podglądu Source/Retargeted w Studio ani diagnostycznej
   wizualizacji ścieżek stóp, bioder i root motion.
7. Nie ma jawnego rozdzielenia problemu chwytu przedmiotu od retargetingu.
   Motion może ustawić dłoń, ale sam punkt zaczepienia broni i semantyka
   wyposażenia Creature nadal muszą przejść osobny gate.

### P2 — rozwój biblioteki po działającym V1

1. Wiele profili szkieletu zamiast jednego H1.
2. Korekcja dłoni przez IK dla broni dwuręcznej.
3. Automatyczna segmentacja i blend klipów z dłuższych nagrań.
4. Właścicielskie tagi stylu, warianty ruchu i ranking jakości.
5. Rosnący katalog zatwierdzonych pakietów zamiast próby pobrania „wszystkich
   animacji Meshy” bez oceny ich przydatności dla NWN.

## 5. Docelowy kontrakt

### 5.1 `MotionPackV1`

Logiczny pakiet powinien zawierać co najmniej:

- `schemaVersion`, `packId`, `retargetProfileVersion`;
- SHA-256 każdego wejściowego GLB, task ID, action ID, nazwę Meshy, docelową
  nazwę NWN, rozmiar i provenance/rights;
- opis bazy: handedness, up, forward, units, height i rest-pose digest;
- wersjonowany semantic skeleton profile oraz mapowanie kości;
- klipy z czasem, interpolacją, trackami, klasą ruchu, loop, transition,
  root-motion policy i eventami;
- hash logicznego payloadu i deterministyczną tożsamość wyniku;
- raport, że pakiet nie zawiera siatki, tekstur, materiałów ani wag dawcy.

Źródłowe modele i GLB Meshy pozostają wyłącznie w
`sample-3d/<asset-id>/manifest.yaml`. Pakiet ruchu jest artefaktem pochodnym,
nie drugim źródłowym katalogiem modeli. Fizyczny envelope pakietu trzeba
zamknąć w fazie kontraktu; schema logiczna nie może zależeć od wyboru ZIP/GLB.

### 5.2 `HumanoidMotionCompatibilityV1`

Raport przed retargetingiem musi zawierać:

- jednoznaczną mapę semantic bone dawca -> cel;
- brakujące, dodatkowe i zdublowane kości;
- zgodność parent chain;
- rest local/global transforms i ich determinanty;
- skalę całej postaci oraz długości segmentów kończyn;
- obecność root, hips, stóp, dłoni i wymaganych attachment nodes;
- werdykt `compatible | compatible_with_warnings | blocked`;
- diagnostykę z konkretną ścieżką zamiast cichego fallbacku.

### 5.3 `retarget_motion_pack_to_creature_v1`

Operacja powinna działać w kanonicznej przestrzeni animacji core, po
rozpoznaniu rigu celu, lecz przed rozwinięciem do pełnych 42 stanów.

Zasady:

1. rotacja celu jest obliczana przez rest-space correction dawca -> cel;
2. quaterniony są normalizowane i utrzymywane w jednej półsferze;
3. translacje zwykłych kości są domyślnie rest-relative albo blokowane;
4. Hips/Root mają jawną politykę i skalę wynikającą z proporcji, nie z całego
   rozmiaru bounding boxu;
5. skale animacyjne są odrzucane, chyba że przechodzą istniejący kontrakt
   bezpiecznego usunięcia stałej skali;
6. target mesh, skin, materiały i accessories są tylko do odczytu;
7. po retargetingu działa contact/stance correction oraz pełna walidacja;
8. raport pokazuje metryki przed/po dla każdego klipu.

### 5.4 Minimalny zatwierdzony pakiet V1

Nie należy zaczynać od całego katalogu Meshy. Pierwszy pakiet powinien mieć
dziesięć sprawdzonych ruchów odpowiadających obecnemu limitowi pojedynczego
runu:

1. neutral idle;
2. walk;
3. run/sprint;
4. attack left;
5. attack right;
6. hit reaction;
7. knockdown;
8. arise;
9. death;
10. alert/taunt.

Pakiet docelowy może mieć więcej klipów. Jego assembler musi łączyć kolejne
batche 1-10 tylko wtedy, gdy donor rig/rest digest jest identyczny. Nie należy
traktować limitu jednego wywołania jako limitu biblioteki.

## 6. Plan implementacji

### Etap 0 — kontrakt i baseline

1. Dodać wersjonowane typy `MotionPackV1`, `HumanoidMotionProfileV1`,
   `MotionRetargetOptionsV1`, raport i stabilne kody błędów.
2. Zamrozić jednego owner-approved dawcę H1 i trzy cele: smukły, standardowy,
   masywny. Wszystkie źródła przez canonical `sample-3d` i manifesty.
3. Wyznaczyć z zatwierdzonego korpusu progi neutralnego idle, kontaktu stóp,
   root drift i dozwolonych zakresów stawów; zapisać je jako wersjonowany
   `HumanoidMotionQualityProfileV1`.
4. Zapisać baseline hashy geometrii, skinningu, materiałów i aktualnych
   animacji każdego celu.

Warunek wyjścia: schema, profile jakości, corpus i oczekiwane błędy są
zatwierdzone przed algorytmem.

### Etap 1 — motion-pack extractor i batch assembler

1. Wyodrębnić skeleton rest pose i animacje z same-model merged GLB.
2. Usunąć mesh, materiały, tekstury, skin weights i obrazy z pakietu pochodnego.
3. Zachować pełny lineage task/action/hash.
4. Umożliwić łączenie wielu batchy 1-10 po exact donor rig/rest digest.
5. Dodać deterministyczny readback i walidator pakietu.

### Etap 2 — semantic skeleton compatibility

1. Zbudować mapowanie po semantyce, nazwach i hierarchii, nie po samym indeksie.
2. Obliczyć rest local/global transforms i proporcje kończyn.
3. Fail closed dla brakujących required bones, odbicia, niejednorodnej skali,
   cyklu albo niejednoznacznej mapy.
4. Wystawić raport Native i WASM.

### Etap 3 — motion-only retarget core

1. Uogólnić sprawdzoną korektę rest pose z `profile_a`, ale użyć osobnych
   rest transforms dawcy i celu.
2. Zaimplementować polityki rotacji, translacji, Hips/Root, czasu i skali.
3. Zagwarantować brak mutacji docelowej geometrii i skinningu.
4. Dodać lineage per clip i deterministic output identity.
5. Najpierw osiągnąć poprawny retarget bez IK na testach syntetycznych.

### Etap 4 — stance, contact i proporcje

1. Wykrywać interwały podparcia na podstawie wysokości i prędkości stopy.
2. Skorygować hips oraz nogi ograniczonym two-bone solverem z zachowaniem
   kierunku kolana i limitów stawu.
3. Blendować wejście/wyjście korekty, aby nie tworzyć skoków.
4. Dodać neutral-idle gate; Meshy action 0 nie może automatycznie zostać
   `cpause1`, jeżeli nie przejdzie profilu jakości.
5. Raportować foot drift, penetration, floating, stance i joint-limit hits.

### Etap 5 — integracja Creature i 42 stanów

1. Wpiąć retargetowane klipy przed `materialize_direct_creature_runtime_clips`.
2. Generować brakujące stany od poprawionego neutralnego `cpause1`.
3. Zachować reguły zgonu, terminal holds i wymagane ciągłości przejść.
4. Zachować root-motion/event policy i istniejącą stabilizację accessories.
5. Dodać motion-pack hash i wersję retargetera do raportu/model identity.

### Etap 6 — WASM, Worker i Studio

1. Dodać identyczny kontrakt core/WASM oraz parity tests JSON/bytes.
2. Całe ciężkie obliczenie wykonywać w Workerze.
3. W Studio dodać:
   - `Use source animations`;
   - `Retarget from motion library`;
   - import/eksport i wybór pakietu;
   - compatibility report;
   - mapowanie klipów i politykę Root/Feet;
   - podgląd Source/Retargeted oraz ostrzeżenia jakości.
4. Nie pozwalać rozpocząć Build przy werdykcie `blocked`.

### Etap 7 — testy regresyjne i realny corpus

1. Test identity: pakiet wyciągnięty z celu i ponownie nałożony na ten sam rig.
2. Test trzech proporcji: slim/standard/brute.
3. Testy negatywne: missing bone, duplicate bone, wrong parent, mirrored basis,
   non-uniform scale, varying scale track i niezgodny rest digest batcha.
4. Testy kinematyczne: idle, walk, run, attacks, hit, knockdown/arise i death.
5. Testy braku regresji: geometria, UV, materiał, wagi, accessories, 300 000
   trójkątów, segmentacja writera, 42 stany i semantic readback.
6. Browser test pełnego Studio -> Worker -> WASM -> core bez blokady UI.

### Etap 8 — demonstrator i human-owned proof

Po zielonych bramkach offline przygotować jeden exact MOD/HAK z tym samym
pakietem ruchów na trzech różniących się Creature. Artefakty zainstalować i
zweryfikować hashami zgodnie z `AGENTS.md`. Agent zatrzymuje się na
`ready_for_owner_proof`; Aurora Toolset i NWN sprawdza właściciel.

## 7. Kryteria ukończenia

Funkcjonalność można uznać za gotową dopiero, gdy wszystkie poniższe kryteria
są spełnione.

### 7.1 Kontrakt i provenance

- Istnieje wersjonowany `MotionPackV1` i fail-closed validator.
- Każdy klip ma task ID, action ID, nazwę źródłową, nazwę NWN, SHA-256 i rights
  attestation.
- Pakiet motion-only nie zawiera geometrii, obrazów, materiałów ani wag dawcy.
- Dwa uruchomienia z tym samym celem, pakietem i opcjami dają identyczny raport
  i identyczne artefakty binarne.
- Zmiana pakietu, opcji, profilu jakości albo wersji algorytmu zmienia result
  identity.

### 7.2 Zgodność szkieletu

- Każda required semantic bone ma dokładnie jedno mapowanie dawca -> cel.
- Hierarchia, handedness, determinanty i scale policy przechodzą raport.
- Każdy przypadek niezgodny jest blokowany przed mutacją modelu i wskazuje
  konkretną kość/ścieżkę.
- Same-rig identity test przechodzi dla całego zatwierdzonego pakietu.

### 7.3 Nienaruszalność modelu docelowego

- Liczba trójkątów przed/po jest identyczna, również dla modelu 300 000.
- Semantyczne hashe POSITION, indices, NORMAL, TANGENT, UV, material bindings,
  JOINTS, WEIGHTS i inverse bind matrices celu są identyczne z baseline.
- Zestaw i wynik stabilizacji odłączonych accessories nie zmienia się.
- Retargeter nie kopiuje mesha, materiałów ani wag dawcy.

### 7.4 Poprawność tracków

- Czasy są skończone, monotoniczne i znormalizowane do startu 0.
- Quaterniony są skończone, jednostkowe i bez skoków znaku półsfery.
- Nie powstają niedozwolone scale tracks.
- Root/Hips translacje odpowiadają jawnej polityce `in-place` albo `preserve` i
  są przeskalowane według wersjonowanej metryki proporcji.
- Wszystkie tracki wskazują istniejące nody celu.

### 7.5 Jakość kinematyczna

- `HumanoidMotionQualityProfileV1` ma trwałe, wyznaczone z zatwierdzonego
  korpusu progi; nie są one liczbami dobranymi ad hoc pod jeden model.
- Wybrany neutral idle przechodzi stance gate, a nieakceptowalny Meshy action 0
  z szerokim rozkrokiem jest odrzucany jako `cpause1`.
- Dla wykrytych interwałów podparcia foot drift, penetration i floating są
  poniżej zamrożonych progów profilu.
- Solver nie odwraca kolan, nie przekracza limitów i nie tworzy skoków na
  wejściu/wyjściu kontaktu.
- Walk i run pozostają rozróżnialne; wybrana polityka root motion jest zgodna z
  rzeczywistym przemieszczeniem.

### 7.6 Kontrakt Creature/NWN

- Wynik ma dokładnie 42 wymagane nazwy stanów i dokładnie jeden `cpause1`.
- Raport per clip rozróżnia `retargeted source`, `preserved target source`,
  `derived terminal hold` i `procedural`.
- Wszystkie istniejące required transition boundaries są ciągłe.
- Jednorazowy zgon i nieruchomy terminalny `cdead` przechodzą obecne bramki.
- Binary MDL writer, parser i semantic readback są zgodne.

### 7.7 Native/WASM/Studio

- Core i WASM zwracają identyczny raport oraz byte-identical wynik.
- Worker wykonuje retargeting bez ciężkiej pracy na głównym wątku UI.
- Studio pozwala utworzyć/importować, wybrać i usunąć lokalny pakiet, pokazuje
  jego exact identity i compatibility report.
- Build jest blokowany dla niezgodnego rigu i pokazuje czytelny błąd.
- Użytkownik może porównać source/retargeted oraz wybrać politykę Root/Feet.
- Biblioteka może połączyć wiele batchy 1-10; jej całkowita liczba klipów nie
  jest ograniczona limitem pojedynczego runu Meshy.

### 7.8 Testy i proof

- Zielone są testy syntetyczne, real-corpus, native, WASM, writer/readback i
  browser integration.
- Ten sam zatwierdzony pakiet przechodzi offline na co najmniej trzech różnych
  proporcjach Creature bez zmiany ich geometrii/skinningu.
- Exact demonstrator ma hash-verified MOD/HAK i status
  `ready_for_owner_proof`.
- Właściciel potwierdza w Aurora Toolset i NWN: widoczność, poprawny idle,
  walk/run, oba ataki, hit, knockdown/arise, pojedynczy zgon, brak ślizgania
  stóp, brak deformacji geometrii i brak ponownego odłączenia accessories.

## 8. Definition of Done w jednym zdaniu

Motion retargeting jest gotowy wtedy, gdy jeden wersjonowany i audytowalny
pakiet ruchów Meshy może zostać zastosowany przez Studio/Worker/WASM/core do
różnych gotowych humanoidalnych Creature, zachowując byte/semantic identity ich
geometrii i skinningu, przechodząc pełne bramki 42 stanów i kinematyki, a exact
demo zostaje pozytywnie potwierdzone przez właściciela w Aurora Toolset i NWN.

## 9. Rekomendowana kolejność decyzji

1. Zatwierdzić, że P0 V1 dotyczy wyłącznie Meshy H1 humanoids.
2. Wybrać neutralnego dawcę i dziesięć ruchów V1.
3. Zamrozić corpus/progi jakości.
4. Dopiero potem implementować extractor, compatibility i core retarget.

Nie należy teraz wydawać kredytów na wszystkie dostępne ruchy Meshy. Najpierw
trzeba dowieść, że jeden curated pack działa na trzech różnych sylwetkach.
