# Plan implementacji i kryteria ukończenia retargetingu animacji między modelami

Data: 2026-07-31  
Status: `IMPLEMENTED / OFFLINE VERIFIED / RUNTIME NOT TESTED`  
Zakres MVP: `SAME_HIERARCHY_RETARGET_V1`  
Przypadek referencyjny: Fogbound Claw Guard → Void Crystal Knight

## 0. Stan realizacji 2026-07-31

Przepływ `Core → WASM → Worker → Studio → Custom` jest zaimplementowany i
zweryfikowany offline. Core klasyfikuje dokładną parę jako
`RETARGETABLE_SAME_HIERARCHY`, przelicza `ca1slashl` na rig Void Knighta,
wiąże wynik z dokumentem Animation Studio V3 i pełną proweniencją, a Studio
wymaga jawnego preview przed mutacją projektu.

Dokładny wynik oraz komendy bramek zapisano w
[`evidence/fogbound-to-void-knight-retarget-v1-offline-2026-07-31.md`](evidence/fogbound-to-void-knight-retarget-v1-offline-2026-07-31.md).
Nie utworzono nowego MOD/HAK. Binary MDL runtime proof i ocena wizualna
Toolset/NWN pozostają osobnym etapem właścicielskim.

## 1. Cel

Umożliwić bezpieczne skopiowanie animacji z jednego modelu humanoidalnego do
drugiego, gdy oba szkielety mają tę samą semantyczną hierarchię kości, ale
różnią się rest pose, proporcjami albo numerycznymi node ID.

Operacja ma działać przez właściwy produktowy przepływ:

`Studio → Worker → WASM → Core → Custom → Base 42 → binary MDL → readback`.

Retargeting nie może zmieniać źródłowego ani docelowego GLB, geometrii, UV,
materiałów, tekstur, wag skinningu ani inverse bind matrices. Wynikiem jest
wyłącznie nowy, edytowalny klip `Custom` związany z dokładnym modelem
docelowym i pełną proweniencją modelu-dawcy.

## 2. Potwierdzony stan wejściowy

### 2.1 Dokładne modele

Model docelowy:

- asset: `void-crystal-knight-h1-v1`;
- plik: `sample-3d/void-crystal-knight-h1-v1/source.glb`;
- SHA-256:
  `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`;
- 24 kości;
- 3 klipy źródłowe;
- 19 704 trójkąty.

Model-dawca:

- asset: `tlc-fogbound-claw-guard-h1-p300k-v1`;
- kanoniczny plik:
  `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`;
- SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`;
- 24 kości;
- 10 klipów źródłowych;
- 297 190 trójkątów.

Plik `source.glb` w katalogu Fogbounda jest zachowanym wariantem odrzuconym
przez gate ciągłości śmierci. Nie jest wejściem dla finalnego testu
retargetingu.

### 2.2 Wynik realnego testu aplikacji

Obecne Studio poprawnie klasyfikuje parę jako `Different rig` i blokuje
`Copy to Custom`:

- liczba kości: zgodna, 24 → 24;
- ID, nazwy i relacje parent: zgodne;
- różne rest translations: 24/24 kości;
- różne rest rotations: 23/24 kości;
- diagnostyka: `M2A-ANIMATION-IMPORT-RIG-MISMATCH`;
- pierwszy komunikat UI: `Rest translation differs for Hips.`

Referencyjny klip `ca1slashl` ma:

- długość 2,50 s;
- 48 tracków;
- 24 tracki rotacji i 24 tracki translacji;
- rzeczywiście zmienną translację tylko na `Hips`; translacje pozostałych
  kości są stałym rest pose dawcy.

Wniosek: nie wystarczy przeskalować całej postaci. Potrzebny jest
kontrolowany transfer ruchu względem rest pose.

## 3. Granice MVP

### W zakresie

- identyczny, jednoznaczny zbiór nazw kości;
- identyczna hierarchia `bone name → parent bone name`;
- ten sam semantyczny animation root;
- różne numery node ID;
- różne lokalne translacje i rotacje rest pose;
- różne proporcje kości;
- transfer rotacji, translacji, root motion, czasów i eventów;
- jawny podgląd na modelu docelowym przed dodaniem do `Custom`;
- pełna proweniencja i deterministyczny readback.

### Poza zakresem

- zgadywanie mapowania kości o różnych nazwach;
- brakujące lub dodatkowe kości;
- różne relacje parent;
- retarget humanoid ↔ quadruped albo humanoid ↔ creature custom rig;
- scale tracks, shear i nieodwracalne rest transforms;
- IK dłoni/stóp, foot locking, weapon attachment correction i motion warping;
- automatyczne nadpisanie istniejącego klipu albo presetu bibliotecznego;
- ukryty retargeting podczas zwykłego `STRICT_RIG_V1`;
- tworzenie nowej iteracji MOD/HAK bez osobnej zgody i spełnienia
  model-iteration gate.

## 4. Decyzje kontraktowe

### 4.1 Klasy zgodności

Core jest jedynym źródłem prawdy i zwraca dokładnie jeden status:

1. `EXACT_COPY`
   - zgodna sygnatura rigu i rest pose;
   - obecna ścieżka kopiowania pozostaje bez zmian.
2. `RETARGETABLE_SAME_HIERARCHY`
   - unikalne nazwy, root i parent graph są zgodne;
   - rest pose albo node ID mogą się różnić;
   - UI może zaoferować jawny `Retarget to current model`.
3. `INCOMPATIBLE`
   - brak/duplikat kości, różna hierarchia, niejednoznaczny root,
     nieobsługiwany scale/shear albo niepoprawne dane;
   - brak mutacji projektu.

Dotychczasowa niezależna funkcja TS `compareAnimationImportRigsV1()` zostaje
usunięta albo ograniczona do projekcji wyniku Core. Nie może utrzymywać drugiej
logiki zgodności.

### 4.2 Tryby operacji

- `EXACT_RIG_COPY_V1` — dotychczasowa kopia bez przeliczania ruchu;
- `SAME_HIERARCHY_RETARGET_V1` — nowy, jawnie wybrany tryb;
- brak wartości `AUTO`, która mogłaby po cichu zmienić zachowanie klipu.

### 4.3 Normatywny algorytm V1

Core wykonuje operację deterministycznie:

1. Inspekcja obu dokładnych GLB i zapis ich SHA-256.
2. Mapowanie kości po unikalnej nazwie i parent-name, nigdy wyłącznie po ID.
3. Rekonstrukcja lokalnych i globalnych macierzy rest pose w porządku
   topologicznym.
4. Dla każdej klatki wyliczenie ruchu dawcy jako delty względem jego rest pose.
5. Przeniesienie delty z układu rest dawcy do odpowiadającego układu rest
   modelu docelowego.
6. Złożenie target global pose i ponowne wyliczenie target-local transform
   względem już przeliczonego parenta.
7. Rotacje są normalizowane i kanonizowane znakowo jako unity quaternion.
8. Stała translacja kości nierootowej dawcy zostaje zastąpiona dokładną
   translacją rest kości docelowej.
9. Zmienna translacja nierootowa jest przenoszona jako delta i skalowana
   ilorazem długości odpowiadających kości; zerowa lub niejednoznaczna długość
   blokuje operację zamiast wymuszać dzielenie.
10. Root motion jest przenoszony jako delta od pozycji root rest i skalowany
    jednym raportowanym współczynnikiem wyliczonym z kanonicznej wysokości
    szkieletu, nie z geometrii modelu.
11. Czasy klatek, długość klipu, interpolation i eventy są zachowane. Core nie
    dodaje arbitralnego resamplingu; przy obliczeniu pozycji parenta używa
    deterministycznej interpolacji na dokładnym czasie bieżącej klatki.
12. Quaterniony, liczby, limity i wynikowy ruch przechodzą zwykły walidator
    Animation Studio przed zwróceniem klipu.

Kolejność mnożenia macierzy i quaternionów musi być zamrożona testami golden
dla przyjętej konwencji column-vector używanej w Core. Sam opis matematyczny
bez testu nie zamyka tej fazy.

### 4.4 Proweniencja i wersjonowanie

Retargetowany klip otrzymuje nowy source kind:

`RETARGETED_MODEL_COPY`.

Dokument Animation Studio jest migrowany z V1/V2 do V3 przed dodaniem klipu.
Nie dopisujemy nowego wariantu do już opublikowanego V2 pod tym samym numerem.

Minimalna `AnimationRetargetProvenanceV1` zawiera:

- `mode=SAME_HIERARCHY_RETARGET_V1`;
- donor source SHA-256;
- target source SHA-256;
- donor clip name i source clip fingerprint;
- donor rig signature SHA-256;
- target rig signature SHA-256;
- compatibility fingerprint SHA-256;
- root-motion scale;
- wynikowy motion fingerprint SHA-256;
- wersję algorytmu oraz limity użyte podczas materializacji.

Walidator V3 wymaga tego bloku wyłącznie dla
`RETARGETED_MODEL_COPY` i zabrania go dla innych source kinds. Zmiana modelu
docelowego, dawcy, klipu, rigu albo trybu oznacza stale provenance i blokuje
build.

## 5. Funkcje do zaimplementowania

### Core/Rust

- [x] `inspect_animation_transfer_compatibility_v1(target, donor)`;
- [x] `build_semantic_rig_mapping_v1(target_rig, donor_rig)`;
- [x] `animation_retarget_compatibility_fingerprint_v1(...)`;
- [x] `retarget_animation_clip_same_hierarchy_v1(...)`;
- [x] `copy_animation_clip_between_models_v1(target_glb, donor_glb,
  clip_name, options)` jako bezpieczna granica wysokiego poziomu;
- [x] walidacja `AnimationRetargetProvenanceV1` w walidatorze dokumentu V3;
- [x] migracja `AnimationStudioDocument` V1/V2 → V3;
- [x] target-bound walidacja klipu po retargetingu;
- [x] deterministyczny motion fingerprint wyniku;
- [x] diagnostyka z kodem, path, przyczyną i akcją naprawczą.

Planowane kody diagnostyczne:

- `M2A-ANIMATION-RETARGET-BONE-SET`;
- `M2A-ANIMATION-RETARGET-HIERARCHY`;
- `M2A-ANIMATION-RETARGET-ROOT`;
- `M2A-ANIMATION-RETARGET-REST-TRANSFORM`;
- `M2A-ANIMATION-RETARGET-BONE-LENGTH`;
- `M2A-ANIMATION-RETARGET-TRACK`;
- `M2A-ANIMATION-RETARGET-LIMIT`;
- `M2A-ANIMATION-RETARGET-PROVENANCE`;
- `M2A-ANIMATION-RETARGET-STALE`;
- `M2A-ANIMATION-RETARGET-READBACK`.

### WASM

- [x] `inspectAnimationTransferCompatibilityV1`;
- [x] `retargetAnimationClipBetweenModelsV1`;
- [x] strict options JSON i unknown-field rejection;
- [x] Node boundary porównany byte-for-byte z natywnym Core;
- [x] brak niezależnej implementacji matematyki w JS/WASM adapterze.

### Worker

- [x] `INSPECT_ANIMATION_TRANSFER_COMPATIBILITY`;
- [x] `RETARGET_ANIMATION_MODEL_CLIP`;
- [x] dwa exhaustywne response types;
- [x] transfer dwóch ArrayBufferów bez utraty kanonicznych File objects;
- [x] cancel/race guard dla project ID, target SHA, donor SHA, clip name,
  authoring revision i request generation;
- [x] stale response nigdy nie dodaje klipu ani nie zapisuje projektu.

### Studio/UI

- [x] statusy `Exact match`, `Retargetable` i `Incompatible`;
- [x] pełna lista różnic zamiast tylko pierwszej;
- [x] jawny wybór `Retarget to current model`;
- [x] podgląd przeliczonego klipu na aktualnym modelu przed zatwierdzeniem;
- [x] source/retargeted comparison i płynne odtwarzanie;
- [x] pokazanie donor/target SHA, liczby kości, root scale i trybu;
- [x] przycisk `Copy retargeted to Custom` dopiero po poprawnym preview result;
- [x] focus management, klawiatura, screen reader i stany busy/error/cancel;
- [x] brak zmiany layoutu całej strony i brak zajmowania ekranu osobnym oknem;
- [x] zapis V3 przez istniejący persistence dokumentu i project backup.

## 6. Plan implementacji

### F0 — kontrakt i failing tests

- [x] zamrozić trzy statusy zgodności i dwa jawne tryby;
- [x] dodać syntetyczne rig fixtures: exact, same hierarchy/different rest,
  different node IDs, missing bone, duplicate name, different parent, scale;
- [x] dodać golden pose dla rotacji i translacji w łańcuchu kości;
- [x] dodać test root motion i różnej wysokości szkieletów;
- [x] zapisać dokładne SHA pary Fogbound/Void Knight jako lokalny env-gated
  corpus test;
- [ ] potwierdzić, że testy są czerwone przed implementacją.

Rezultat: kontrakt matematyczny i granice błędu są mierzalne przed kodem.

### F1 — klasyfikacja zgodności w Core

- [x] przenieść źródło prawdy z TS do Core;
- [x] mapować po nazwach i parent-name;
- [x] pozwolić na różne node ID i rest pose tylko przy identycznej hierarchii;
- [x] policzyć obie rig signatures i compatibility fingerprint;
- [x] fail-closed dla niejednoznaczności, zmiennych scale tracks i limitów;
- [x] zachować istniejący `EXACT_RIG_COPY_V1` bez regresji.

Rezultat: para Fogbound/Void Knight ma
`RETARGETABLE_SAME_HIERARCHY`, nie `EXACT_COPY` ani `INCOMPATIBLE`.

### F2 — retarget matematyczny w Core

- [x] zaimplementować rest-space delta transfer;
- [x] zaimplementować canonical skeleton-height i root-motion scale;
- [x] zachować target rest translations dla stałych nierootowych tracków;
- [x] obsłużyć bounded variable child translation;
- [x] kanonizować quaterniony i liczby zmiennoprzecinkowe;
- [x] zachować czasy, interpolation, duration i eventy;
- [x] uruchomić zwykły walidator klipu na target rig;
- [x] udowodnić deterministyczność dwóch przebiegów.

Rezultat: Core zwraca target-bound `DRAFT`, który nie deformuje rest pose i ma
rzeczywisty ruch.

### F3 — dokument V3 i persistence

- [x] dodać `RETARGETED_MODEL_COPY` i provenance V1;
- [x] dodać deterministyczną migrację V1/V2 → V3;
- [x] zablokować nowy source kind w dokumentach V1/V2;
- [x] round-trip JSON i istniejący persistence projektu;
- [x] stale target/authoring oraz zmiana donor/clip/mode przed commit blokują wynik;
- [x] odczyt istniejących V1/V2 pozostaje bezstratny.

Rezultat: po restarcie aplikacji klip zachowuje dokładną linię dawcy i modelu
docelowego.

### F4 — WASM i Worker

- [x] wystawić dwie publiczne funkcje WASM;
- [x] rozszerzyć typowany kontrakt Workera;
- [x] dodać race/cancel guards;
- [x] Node native/WASM parity;
- [x] real browser Worker test na syntetycznych modelach;
- [x] test odrzucenia wyniku po zmianie targetu, dawcy, klipu lub revision.

Rezultat: Studio nie liczy retargetingu na main thread i nie może przyjąć
starej odpowiedzi.

### F5 — UX i preview

- [x] rozbudować istniejący dialog kopiowania, bez osobnego kroku workflow;
- [x] pokazać klasyfikację, różnice i dokładne identity;
- [x] wybrać klip i jawnie uruchomić retarget preview;
- [x] odtworzyć wynik płynnie na aktualnym modelu;
- [x] dopiero potem pozwolić dodać kopię do `Custom`;
- [x] dodać stany loading/error/cancel/offline;
- [x] testy dostępności i focus trap.

Rezultat: użytkownik rozumie, że ruch został przeliczony, a nie skopiowany
bitowo, i widzi rezultat przed zapisaniem.

### F6 — Custom, Base 42 i build

- [x] zapisać wynik jako edytowalny klip `Custom`;
- [x] pozwolić na dalszą edycję bez zmiany provenance wejścia;
- [x] przypisać klip do jednego albo wielu slotów Base 42;
- [x] materializacja dokumentu przez wspólny Core;
- [ ] binary MDL readback `MATCH`;
- [x] sprawdzić czasy, wartości, eventy i target node IDs;
- [x] potwierdzić byte-identical oba wejściowe GLB.

Rezultat: retargetowany klip jest pełnoprawnym źródłem animacji produktu.

### F7 — exact Fogbound → Void Knight

- [x] wczytać kanoniczny Fogbound
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`;
- [x] wczytać Void Knight
  `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`;
- [x] uzyskać `RETARGETABLE_SAME_HIERARCHY`;
- [x] przeliczyć `ca1slashl` do target-bound Draft;
- [x] obejrzeć pełny klip w viewportcie aplikacji bez przeskoku do rest pose;
- [x] sprawdzić w pamięci zapis do `Custom` i przypisanie do Base 42;
- [ ] zbudować 42/42 z binary MDL readback `MATCH`;
- [x] zapisać raport, fingerprinty i wynikowy klip;
- [x] nie tworzyć MOD/HAK bez osobnej zgody oraz spełnienia model-iteration
  gate.

Rezultat: exact lokalny corpus potwierdza działanie na dwóch realnych modelach,
nie tylko na fixture.

### F8 — regresja, dokumentacja i owner proof

- [x] pełne Core/WASM/Worker/Studio tests;
- [x] production build i bundle budget;
- [x] dokumentacja użytkownika i kontraktu V3;
- [x] immutable evidence JSON exact pary;
- [ ] po osobnej zgodzie przygotować jeden dokładny MOD/HAK i handoff;
- [ ] owner-reported Toolset/NWN result dla tego samego lineage;
- [ ] wynik ownera nie może być zastąpiony screenshotem viewportu Studio.

Rezultat: feature ma pełny offline proof, a claim runtime jest oddzielnym,
uczciwym owner gate.

## 7. Kryteria ukończenia

Każde kryterium jest binarne. Build, screenshot albo pojedyncza poprawna poza
nie zamykają funkcji.

### K0 — dokładny zakres i lineage

- [x] target i donor są związane pełnym SHA-256;
- [x] finalny test używa `source-death-continuous.glb`, nie odrzuconego
  `source.glb` Fogbounda;
- [x] wejściowe GLB pozostają byte-identical;
- [x] nie powstaje nieautoryzowana iteracja MOD/HAK.

### K1 — jedno źródło prawdy

- [x] klasyfikacja i matematyka istnieją wyłącznie w Core;
- [x] WASM i Worker są cienkimi adapterami;
- [x] TS nie ma niezależnego epsilon/mapping/retarget algorytmu;
- [x] native Core i WASM Node mają byte-identical parity, a browser Worker
  przechodzi ten sam kontrakt.

### K2 — poprawna klasyfikacja rigu

- [x] exact rig → `EXACT_COPY`;
- [x] ta sama hierarchia i inny rest → `RETARGETABLE_SAME_HIERARCHY`;
- [x] inne node ID przy tych samych nazwach/parentach nadal są retargetable;
- [x] brak/duplikat kości, inny parent albo root → `INCOMPATIBLE`;
- [x] przypadek incompatible nie mutuje projektu.

### K3 — matematyka ruchu

- [x] pozycja w `t=0` odpowiada docelowemu rest pose, jeśli źródłowa delta
  wynosi zero;
- [x] rotacja zagnieżdżonych kości zgadza się z golden;
- [x] nie ma stałego offsetu dawcy na kończynach Void Knighta;
- [x] długości kości targetu nie dryfują dla stałych translation tracks;
- [x] root motion zachowuje kierunek i jest skalowany raportowanym ratio;
- [x] quaterniony są jednostkowe i znakowo deterministyczne;
- [x] dwa przebiegi dają identyczny JSON i motion fingerprint.

### K4 — integralność klipu

- [x] duration, transition, keyframe times, interpolation i events są
  zachowane;
- [x] target node IDs pochodzą wyłącznie z target rigu;
- [x] wynik ma rzeczywisty ruch i przechodzi walidator Animation Studio;
- [x] limity tracków/klatek/eventów są sprawdzane przed alokacją wyniku;
- [ ] binary MDL readback ma `MATCH` dla każdej wynikowej ścieżki.

### K5 — provenance i persistence

- [x] dokument jest jawnie V3;
- [x] provenance zawiera donor/target/clip/rig/mode/root-scale/output hash;
- [x] zmiana targetu, authoring revision albo identity operacji przed commit
  odrzuca wynik;
- [x] JSON i istniejący persistence projektu round-tripują wszystkie pola;
- [x] istniejące dokumenty V1/V2 migrują bez utraty danych.

### K6 — UX

- [x] UI rozróżnia exact, retargetable i incompatible;
- [x] użytkownik jawnie wybiera retargeting;
- [x] wynik można odtworzyć na modelu docelowym przed zapisem;
- [x] przycisk zapisu jest wyłączony przed poprawnym preview result;
- [x] błędy pokazują przyczynę i działanie naprawcze;
- [x] klawiatura, focus, screen reader i cancel przechodzą testy;
- [x] operacja nie otwiera osobnego natywnego okna na ekranie użytkownika.

### K7 — race safety i brak mutacji

- [x] zmiana targetu podczas pracy odrzuca wynik;
- [x] zmiana dawcy albo klipu podczas pracy odrzuca wynik;
- [x] zmiana authoring revision podczas pracy odrzuca wynik;
- [x] cancel nie dodaje klipu;
- [x] failure nie zmienia dokumentu, `Custom`, mapowania ani autosave;
- [x] donor geometry, target geometry, weights i materiały nie są kopiowane ani
  modyfikowane.

### K8 — wydajność i offline

- [x] oba GLB są analizowane poza main thread;
- [ ] dokładna para 10 MB + 22 MB kończy inspection/retarget w 10 s na
  referencyjnej maszynie po załadowaniu WASM;
- [x] browser performance test nie wykrywa long tasku UI >100 ms podczas
  obliczeń Workera;
- [x] operacja działa offline;
- [x] payload dawcy nie trafia do IndexedDB, backupu ani bundle aplikacji;
- [x] build pozostaje w aktualnych budżetach JS/CSS/WASM.

### K9 — exact Fogbound → Void Knight

- [x] 24 kości i 0 różnic strukturalnych są potwierdzone przez Core;
- [x] 24 różnice translacji i 23 rotacji rest pose są raportowane;
- [x] `ca1slashl`, 2,50 s i 48 tracków przechodzi retarget;
- [x] pozostałe osiem klipów Fogbounda przechodzi ten sam high-level pipeline,
  walidację V3, zapis `Custom` i wspólną materializację Base 42;
- [x] jedyna zmienna translacja źródłowa `Hips` jest poprawnie przeniesiona;
- [x] viewport aplikacji pokazuje cały ruch na Void Knighcie bez stałego
  offsetu rest pose; zapisano 9,63 s proof MP4;
- [ ] wynik trafia do `Custom`, Base 42 i binary MDL z readbackiem `MATCH`;
- [x] źródłowy Void Knight pozostaje niezmieniony i zachowuje 19 704 trójkąty.

### K10 — regresja i dowody

- [x] `cargo fmt --all -- --check`;
- [x] `cargo clippy --workspace --all-targets -- -D warnings`;
- [x] `cargo test --workspace`;
- [x] WASM Node tests i generated Node boundary;
- [x] Studio typecheck i wszystkie testy;
- [x] real browser Worker/WASM i persistence;
- [x] production build i bundle budget;
- [x] `git diff --check`;
- [x] exact offline evidence zapisuje komendy, SHA, fingerprints i wyniki;
- [x] status runtime pozostaje `not_tested/missing`, dopóki właściciel nie
  przekaże wyniku dokładnego MOD/HAK w Toolset/NWN.

## 8. Macierz testów minimalnych

| Przypadek | Oczekiwany wynik |
|---|---|
| identyczny rig, różne GLB SHA | `EXACT_COPY`, jeśli rig signatures są zgodne |
| te same nazwy/parent, inne node ID | `RETARGETABLE_SAME_HIERARCHY` albo `EXACT_COPY` zależnie od rest pose |
| te same nazwy/parent, inny rest pose | `RETARGETABLE_SAME_HIERARCHY` |
| jedna brakująca kość | `INCOMPATIBLE`, zero mutacji |
| dodatkowa albo zduplikowana nazwa | `INCOMPATIBLE`, zero mutacji |
| inny parent jednej kości | `INCOMPATIBLE`, zero mutacji |
| scale/shear/nonfinite rest | `INCOMPATIBLE`, stabilny diagnostic |
| stałe child translations | dokładny target rest, bez zmiany długości kości |
| zmienny root translation | kierunek zachowany, jawny height ratio |
| q oraz -q | identyczny canonical output |
| stale Worker response | odrzucony, zero autosave |
| Fogbound `ca1slashl` → Void Knight | Custom + Base 42 + MDL readback `MATCH` |

## 9. Definition of Done

MVP jest ukończone, gdy realna aplikacja klasyfikuje kanoniczną parę
Fogbound/Void Knight jako `RETARGETABLE_SAME_HIERARCHY`, pokazuje przeliczony
`ca1slashl` płynnie na Void Knighcie, zapisuje go jako target-bound `Custom` z
pełną proweniencją V3 i buduje binary MDL z readbackiem `MATCH`, zachowując oba
GLB i całą geometrię bez zmian.

Offline Definition of Done nie oznacza automatycznie sukcesu w NWN. Nowy
MOD/HAK, instalacja i właścicielski proof są osobnym końcowym gate zgodnym z
regułami iteracji modelu.

## 10. Zalecana kolejność rozpoczęcia

1. Failing synthetic tests klasyfikacji i macierzy rest delta.
2. Core compatibility i semantic bone mapping.
3. Core retarget math oraz golden poses.
4. Dokument V3 i provenance.
5. WASM/Worker parity i race guards.
6. Preview UX w istniejącym dialogu.
7. Custom → Base 42 → binary MDL na fixture.
8. Exact Fogbound → Void Knight offline.
9. Pełna regresja, dokumentacja i dopiero potem osobna decyzja o demo runtime.
