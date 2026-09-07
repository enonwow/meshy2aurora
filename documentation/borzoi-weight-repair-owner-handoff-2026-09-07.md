# Borzoi — naprawa wag i przekazanie do testu (2026-09-07)

**Plik modułu: tlc_brz_260906.mod. Nazwa w Toolset/NWN: TLC Borzoi c_wolf - test. Area: Borzoi - test animacji (tlc_brz_area).**

Status: **ready_for_owner_proof**. Dokładne MOD i HAK są już zainstalowane w katalogach użytkownika Neverwinter Nights. Odczyt po kopiowaniu potwierdził identyczność SHA-256. Nie uruchamiano Toolset ani NWN; test wizualny wykonuje właściciel zgodnie z AGENTS.md.

## Co faktycznie było wadliwe

Koncept określał wygląd, ale nie gwarantował poprawnych wag modelu wygenerowanego przez Tripo. Dopasowanie niezmienionych kości i brak szwów nie oznaczały, że powierzchnia poprawnie zgina się między tymi kośćmi. W poprzedniej wersji lokalne mieszanki wag powodowały zapadnięcia i rozciągnięcia, m.in. w obrębie klatki, przednich kończyn i sierści pod szyją. Kości referencji nie zmieniały długości. Poprzedni audyt: [diagnoza źródłowa](borzoi-deformation-solution-audit-2026-09-07.md).

Standardowy test wybierał ograniczoną liczbę chwil i część trójkątów. Przejście tej kontroli nie obejmowało wszystkich skrajnych póz między próbkami. Dodatkowo raporty zgodności niezmienionego szkieletu nie są pomiarem zgodności anatomii siatki.

## Wykonana naprawa

- Dodano diagnostyczny solver wag optymalizujący odkształcenia na rzeczywistych pozach wszystkich 42 animacji c_wolf. Sprawdzono zgodność obliczeń solvera z ewaluatorem MDL na 25 830 wierzchołkach (maks. błąd pierwszego etapu 2.43e-7).
- Dodano miejscową korektę współrzędnych wag: każda przyjęta zmiana zmniejsza funkcję błędu mierzoną we wszystkich pozach na sąsiednich krawędziach i trójkątach. Zachowano ograniczenie czterech wpływów, normalizację oraz istniejącą kontrolę gradientów wag.
- Odrzucono zbyt szerokie przypisanie szyi/brzucha, eksperyment minimax i prostą korektę par punktów, ponieważ tworzyły regresje. Nie trafiły do wyniku.
- Nie zmieniono w tej naprawie pozycji wierzchołków, indeksów trójkątów, transformacji kości ani tekstury. Zmieniono wagi wiążące powierzchnię z c_wolf.
- Poprawki przeszły oryginalny Creature pipeline, import WebMCP w Studio i eksport produktu. Progi oryginalnej walidacji nie zostały poluzowane.

Implementacja: crates/m2a-wasm/examples/creature_strain_solver/mod.rs oraz tryby strain-optimize, strain-local i region-audit --all-triangles w diagnose_creature_skinning.rs. Tryb strain-minimax pozostał eksperymentem diagnostycznym, odrzuconym dla tego modelu. Narzędzie materialize_creature_product_fixture.rs przygotowuje i odczytuje moduł dla już wyeksportowanego produktu, bez sterowania grą.

## Wyniki

| Pomiar | Przed | Wybrany wynik |
|---|---:|---:|
| Standardowe animacje bez blokera | 35/42 | 42/42 |
| Mocno odkształcone krawędzie × próbki czasu (poniżej 0.25 lub powyżej 4 razy długość bazowa) | 7050 | 382 |
| Skrajne zapadnięcia krawędzi w gęstym teście | występowały | 0 |
| Skrajne zapadnięcia/rozciągnięcia trójkątów w gęstym teście | brak pełnego pomiaru przed naprawą | 0 |

Gęsty test: 1113 póz w 42 klipach, około 30 próbek/s, wszystkie 20 534 trójkąty w każdej pozie, łącznie 22 854 342 pomiary trójkątów. Zakres długości krawędzi: 0.109576–3.647966 razy długość bazowa. Zakres pola trójkąta: 0.006615–30.315481 razy pole bazowe. Brak przekroczeń progów absolutnych: krawędzie 0.0625–16; pola 0.0025–400.

Pozostaje 439 próbek z polem poniżej 0.05 i 64 powyżej 20. To nie jest twierdzenie, że każdy fragment siatki jest idealny. Spadek 7050 → 382 dotyczy liczby zdarzeń pomiarowych, nie procentu „naprawionego modelu”. Gęste raporty podglądu i eksportowanego MDL są identyczne poza tożsamością pliku.

Blender odczytał 25 830 wierzchołków, 20 534 trójkąty i 30 węzłów. Błąd odtworzenia pozy bazowej: 2.39e-7. Osobne próby ruchu czterech łap, dwóch węzłów ogona i głowy poruszyły przypisane wierzchołki. Cztery testy obliczeń narzędzia przeszły. W aplikacji obejrzano wybrane klatki chodu, biegu oraz końcowe pozy cdead/ckdbckdie; nie stanowi to pełnego przeglądu wizualnego każdego klipu.

## Dokładny wynik

Katalog: C:/Projects/meshy2aurora/artifacts/creatures/borzoi-cwolf-tissue-repair-20260907.

- borzoi-cwolf-rigged.glb — edytowalny model ze szkieletem i wagami; SHA-256 fcc1edd5f3e705ad869e6ee343630b86b4c10074fd9701c3bc02503a35b755f1.
- borzoi-cwolf-rigged.blend — sprawdzona scena Blendera. GLB/BLEND nie zawierają skopiowanych oryginalnych animacji NWN; model gry dziedziczy je z c_wolf.
- nwn/c_tlcborzoi.mdl — SHA-256 2b24d9e325b0e7ab8b502901b624de4d488a47ab2f579f7362f006cbebeac2bf.
- nwn/tlc_brz_260906.hak — SHA-256 30657c5d3be9689f8782424044365daa6721a4f74a0b80bbd4cda000448d7923.
- nwn/tlc_brz_260906.mod — SHA-256 1a313df6cdd60ed82fe14fee1d4f7a5e4f38f6dae38c468592be7130b474b136.
- manifest.json, dense-motion-report.json, nwn/installation.json i nwn/module-readback.json — tożsamość, pomiary, instalacja oraz niezależny odczyt modułu.

Źródło: sample-3d/borzoi-tripo-dc22ecb0/source-cwolf-paw-local-final.glb, SHA-256 cc8e663457706c4f105d4af9dd82996d43ad1edb8e0d91b6609e1405c908bfd4. Referencja c_wolf: a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726.

## Test właściciela i granice potwierdzenia

Otworzyć **tlc_brz_260906.mod**, moduł **TLC Borzoi c_wolf - test**, Area **Borzoi - test animacji**. HAK: tlc_brz_260906. Obiekt: **Borzoi - poprawione wagi**, template tlc_brz_dog, Appearance 15100 (TLC_BORZOI_TRIPO), pozycja (10, 14.5, 0). Gracz zaczyna w (10, 10, 0), skierowany na psa. Obiekt ma istniejący profil active_monster_baseline z aktywnym AI; test nie jest automatycznym pokazem wszystkich 42 animacji.

Do oceny pozostają rzeczywisty wygląd i zachowanie w NWN, kontakt łap z podłożem oraz naturalność chodu/biegu, obrotów i upadków. Pomiar odkształceń nie wykrywa wszystkich kolizji ani problemów sylwetki. Standardowe zielone dopuszczenie Studio nadal używa pierwotnego rzadszego testu; gęstszy walidator w tym zadaniu jest narzędziem diagnostycznym CLI i nie został podłączony jako obowiązkowa bramka każdej ścieżki aplikacji.

Toolset: modelVisibility=not_tested, proofCompleteness=missing. NWN: modelVisibility=not_tested, proofCompleteness=missing. Zachować tę samą zainstalowaną linię MOD/HAK/MDL do wyniku testu właściciela; nie regenerować jej pod tymi nazwami.
