# Borzoi: próba zastosowania `c_wolf` i braki pipeline'u Creature

Data: 2026-09-06. Status wykonania: **BLOCKED_BEFORE_MODEL_EXPORT**.

**Korekta wyboru źródła, 2026-09-06:** właściciel wskazał następnie `Downloads/wolf+3d+model.glb`, SHA-256 `dc22ecb0561fc10773e1a156b0d126225a15cdfb67cd7c0999a965a4f1d59689`. Ten plik ma **20 534 trójkąty i 6 687 432 B**, nie prawie 2 mln. Poniższe historyczne próby zostały rzeczywiście wykonane, ale dotyczyły niewłaściwie wybranego starszego źródła. Ich blokady rozmiaru, redukcji oraz oczekującego V9 **nie dotyczą nowo wskazanego pliku**. Aktualną próbę i sprostowanie opisuje [osobny raport](C:/Projects/meshy2aurora/documentation/evidence/wolf-tripo-selected-source-correction-and-cwolf-attempt-2026-09-06.md).

Zlecenie właściciela: zastosować supermodel wolf przez pipeline aplikacji i wypisać braki Creature. Uruchomiono publiczną granicę WASM używaną przez Studio, bez zmiany kodu produktu, progów bezpieczeństwa, plików źródłowych ani zamrożonych kandydatów. **Nie powstał nowy gotowy model, MDL, HAK lub MOD.** Lista braków i wyniki prób są dostarczone; część dotycząca nowego modelu pozostaje zablokowana.

## 1. Dokładne wejście i zakres

Wybrano oryginalny model Meshy, którego dotyczył audyt zadania „Stwórz modele NWN w Blenderze” (`01a06e72-ad4e-7533-814c-12d41f2c1097`), nie jego przerobioną siatkę Blender v5 i nie niezależne źródło bind-v6.

| Wejście | Plik pod `sample-3d/borzoi-meshy-manual-p1997k-v1` | Rozmiar | Trójkąty |
| --- | --- | ---: | ---: |
| Pełne źródło wybrane przez właściciela | `source.glb` | 77 030 884 B | 1 997 064 |
| Istniejący, wcześniej przygotowany wariant budżetowy | `source-p300k.glb` | 29 889 104 B | 300 000 |

SHA-256 pełnego źródła: `71949d52f0f9e68da0642511d786e144bfdd47491cc7bae9c0fe09bfbb3fac54`.

SHA-256 wariantu budżetowego: `f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda`.

Referencja: dokładny binarny `c_wolf`, 340 812 B, SHA-256 `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`. Został rozwiązany w pamięci przez własne publiczne indexery KEY/BIF z lokalnego `nwn_base.key`. Nie zapisano payloadu retail ani nie sterowano grą/Toolsetem.

Ustawienia próby: `sourceForward=POSITIVE_Z`, `allowExcessiveBranchBoundaryRepair=false`, dokładny chain `c_wolf -> NULL`. Kierunek zachowuje udokumentowaną orientację tej próbki; nie zmieniano go metodą prób i błędów.

Pełne źródło ma jeden mesh/primitive, materiał, trzy tekstury, UV i normalne; nie ma rigu ani animacji. Stwierdza to dzisiejszy publiczny odczyt, a nie tylko historyczny manifest.

## 2. Fakty z dzisiejszego wykonania

| Publiczna funkcja | Źródło | Wynik |
| --- | --- | --- |
| `inspectHighPolyGlbJson` | pełne | Odczyt zakończony. `conversionEligible=false`: ponad 300 000 trójkątów oraz tryb inspection-only. |
| `prepareReferenceSupermodelRigV3` | pełne | `M2A-GLB-INPUT-LIMIT-EXCEEDED`: 77 030 884 B przekracza limit 67 108 864 B (64 MiB). Zatrzymanie przed nadawaniem wag. |
| `prepareReferenceSupermodelRigV3` | istniejące 300k | `M2A-REFERENCE-SUPERMODEL-SKIN-BRANCH-REPAIR-EXCESSIVE`: lokalna naprawa ciągłości wymagałaby zmiany przypisań 29 490 wierzchołków przy limicie 11 691. Nie pominięto limitu. |

Licznik 29 490 oznacza propozycję ponownego przypisania w algorytmie wag. **Nie jest pomiarem liczby wizualnie uszkodzonych wierzchołków ani pełną diagnozą anatomiczną.** Przygotowanie rigu zostało przerwane; dla tej próby nie ma wyniku kontroli wszystkich animacji, PASS eksportu ani wyniku NWN. Nie wolno przenosić na nią historycznego PASS innego modelu.

Wykonanie trwało około 7 sekund. Użyto zastanego pakietu aplikacji `crates/m2a-wasm/pkg`, zmodyfikowanego 2026-09-05, a nie nowo zbudowanego binarium z aktualnego dirty worktree. SHA-256 WASM: `bcd05d4230107b8af5c5d5f0610887dc857b5bde95c134d0ba963a0235d8627e`. Przegląd kodu dotyczył bieżącego drzewa przy HEAD `7289f2b0c8d385b2058912fb4fa6c0eb7e1990ba`; nie jest to deklaracja zgodności niezapisanych źródeł z binarium.

Dowody:

- [Metadane wykonania](C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-original-cwolf-request-20260906/run.json).
- [Odczyt pełnego GLB](C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-original-cwolf-request-20260906/inspect-original.json).
- [Próba pełnego źródła](C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-original-cwolf-request-20260906/prepare-original.json).
- [Próba wariantu 300k](C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-original-cwolf-request-20260906/prepare-budget.json).
- Adapter diagnostyczny: `.codex-tmp/borzoi-cwolf-request-20260906.mjs`, SHA-256 `cc038b33ace62fcd21d13db072f27d6d05d580f9f8baaaf2092e09d0ad46b84e`. Wywołuje publiczne API, zapisuje wyłącznie raporty i odmawia nadpisania katalogu wyjściowego. Timeout pojedynczej próby: 180 s; nie wystąpił.

## 3. Braki Creature — kolejność naprawy

P1 oznacza blokadę domknięcia tego workflow, P2 ograniczenie poprawności/obsługi wymagające naprawy. Poniższa lista nie oznacza wykonania tych zmian.

| Priorytet | Brak / dowód | Kryterium poprawnego rozwiązania |
| --- | --- | --- |
| **P1** | **Brak zintegrowanego przygotowania high-poly przed zastosowaniem supermodelu.** Potwierdzone dzisiejszą próbą. Apply wysyła niezmienione źródło; inspection-only nie tworzy dopuszczonego wariantu. | Przyjąć źródło do kontrolowanego przygotowania, utworzyć jawny wariant z provenance i budżetem **≤300 000**, zweryfikować geometrię i materiały. Nie podnosić limitu produktu. Oddzielić limit pamięci od budżetu modelu wynikowego. |
| **P1** | **Zbyt słaba kontrola jakości redukcji.** Obecny osobny simplifier usuwa ściany o squared cross magnitude ≤`1e-10`, następnie może uzupełniać liczbę trójkątów dzieleniem innych ścian. Historyczny wariant tej próbki miał 1 836 komponentów; audyt w Blenderze wykazał również liczne otwarte krawędzie. To nie jest dziś ponownie policzony wynik topologii. | Po redukcji porównać zamknięcie powierzchni po uwzględnieniu szwów UV, komponenty, sylwetkę, odległość powierzchni, UV i materiały. Dokładna liczba trójkątów nie wystarcza. Usuwanie drobnych ścian nie może być traktowane jako bezpieczna naprawa bez kontroli skutków. |
| **P1** | **Brak lokalnego dopasowania anatomii siatki do niezmiennego rigu.** Przegląd kodu: rejestracja stosuje jeden współczynnik wysokości i przesunięcie dolnego środka całego modelu. | Umożliwić dopasowanie lokalnych regionów głowy, nóg i ogona do właściwych przegubów bez zmiany referencyjnych macierzy bind. Mierzyć zgodność anatomii niezależnie od samego zachowania jointów. |
| **P1** | **Blokada naprawy wag nie prowadzi do praktycznej naprawy w UI.** Dzisiejszy komunikat wskazuje licznik i eksperymentalne obejście, lecz nie wskazuje użytkownikowi przestrzennie tych 29 490 wierzchołków. Numeryczne edycje wag, komponentów i list vertexów istnieją. | Pokaż problematyczny region i wpływy kości w viewport; umożliwiaj zaznaczenie, lokalną korektę/izolację sierści i ponowne sprawdzenie. Nie zastępować tego domyślnym wyłączeniem limitu. |
| **P1** | **Finalny podgląd odczytanego eksportu nie otrzymuje odziedziczonego chainu animacji.** Przegląd danych App → AuroraExportViewport → readback builder. Osobny podgląd supermodelu scala chain, finalny widok korzysta z lokalnych klipów MDL. | Odtworzyć odziedziczone klipy na rzeczywiście wyeksportowanym i ponownie odczytanym MDL z jego wagami i materiałami oraz dokładną, read-only referencją. Model nadal ma 0 lokalnych klipów. |
| **P2** | **Ograniczone pokrycie ekstremów ruchu.** Obecny sampler uwzględnia klucze, środki interwałów i eventy, ale redukuje próbki do nominalnie 9 na klip; obowiązkowe eventy mogą zwiększyć tę liczbę. To luka statycznie zweryfikowana, nie nowy false PASS tego modelu. | Wykryć krótką złą deformację między zwykłymi próbkami; stosować zagęszczanie zależne od deformacji i raportować najgorszą klatkę/region oraz faktyczne pokrycie. |
| **P2** | **Zapis pracy i ponowna analiza wymagają domknięcia.** Audyt 5 września odnotował reset recepty rigu przy ponownym Apply i brak pełnego wznowienia projektu Creature. | Zapis źródła, jego hashy, dokładnego chainu, materiałów, korekt siatki/wag i ustawień; ponowna analiza zachowuje edycje albo wymaga jawnego resetu. Wynik ma być odtwarzalny. |
| **P2** | **Niejawny zakres ustawień źródła dla wybranej trasy.** Opcje naprawy tekstur/high-poly/accessories w Source nie są wszystkie przekazywane w selected-supermodel Apply. Przegląd routingu nie dowodzi, że każda powinna tam działać. | Każda widoczna opcja albo ma udokumentowany i przetestowany wpływ na tę trasę, albo jest wyłączona z wyjaśnieniem. Ustawienia wpływające na wynik należą do jego tożsamości. |

Najważniejsze miejsca kodu:

- [WASM ingest](C:/Projects/meshy2aurora/crates/m2a-wasm/src/lib.rs:619) i [domyślne limity](C:/Projects/meshy2aurora/crates/m2a-core/src/glb/mod.rs:79).
- [Usuwanie drobnych ścian](C:/Projects/meshy2aurora/tools/meshy-exact-triangle-target.mjs:141) i [dzielenie ścian](C:/Projects/meshy2aurora/tools/meshy-exact-triangle-target.mjs:194).
- [Globalna rejestracja geometrii](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_generic.rs:1624).
- [Ograniczenia authoringu bind](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_authoring.rs:502) i [numeryczny edytor wag](C:/Projects/meshy2aurora/apps/studio-web/src/features/supermodels/RigJointEditor.tsx:179).
- [Finalny widok](C:/Projects/meshy2aurora/apps/studio-web/src/App.tsx:2372) i [animacje z lokalnego readback](C:/Projects/meshy2aurora/apps/studio-web/src/features/preview/AuroraReadbackViewport.tsx:282).
- [Próbkowanie animacji](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_motion.rs:10085).
- [Szerszy audyt Creature z 5 września](C:/Projects/meshy2aurora/documentation/audyt-creature-animacje-supermodele-2026-09-05.md).

## 4. Istniejący model i bramka kolejnej iteracji

**Istniejący testowy plik: `m2aborzmod9.mod`.** Nazwa w Toolsecie: **`Meshy2Aurora Borzoi c_wolf Demo V9`**. Dokładny Area: **`Meshy2Aurora Borzoi Test Area V9`** (`m2aborzarea9`).

Ten wcześniejszy model powstał przez pipeline na wariancie `f96be839…`. Nie jest wynikiem dzisiejszego wykonania. W zapisach nadal brakuje wyniku właściciela dla dokładnego V9; późniejszy V10 używa innego źródła `3efd673…` i jego PASS nie rozstrzyga jakości oryginału.

Zamrożony katalog: `proof-output/borzoi-c-wolf-demo-v9-20260825-full-skeleton`.

- MOD `m2aborzmod9.mod`: SHA-256 `2e97e2d708fbf881c466fdbc4658f2ea06fd401b4803568a6da369230e51e407`.
- HAK `m2aborzhak9.hak`: SHA-256 `bbb676933f9052948a0947991c9a5349a8961edae6f53d46199e727542a72f62`.
- MDL `m2aborzcre9.mdl`: historyczny SHA-256 `fefcd3298b711edb9d87dff394f408a6e52b7bfd831516b143ca627a89b1c64e`.
- Appearance `15100`, creature `m2aborzutc9`, placement `(10.0, 14.5, 0.0)`.

Dzisiejszy odczyt hashy z natywnych katalogów NWN potwierdził, że MOD i HAK V9 nadal istnieją i mają powyższe dokładne hashe. Nie kopiowano ani nie zmieniano żadnego z nich. Nie wykonano live proofu. Toolset i NWN zachowują historyczny brak wyniku: `modelVisibility=not_tested`, `proofCompleteness=missing`.

Nowy kandydat z nowymi bajtami/resrefami dla tego źródła wymaga zamknięcia obowiązującej bramki albo osobnej, jednoznacznej decyzji właściciela zezwalającej na konkretny wyjątek. Sama dzisiejsza blokada offline nie jest świeżym wizualnym niepowodzeniem V9. Nie utworzono następnego numeru w celu obejścia tej zasady.

## 5. Weryfikacja i kolejny krok

- Guard kanonicznego worktree: PASS przed zapisami.
- Gate `assert-meshy-asset-layout.ps1`: PASS.
- `node --check .codex-tmp/borzoi-cwolf-request-20260906.mjs`: PASS.
- Publiczne trzy próby: wykonane, wyniki opisane wyżej; blokady produktu są rezultatem, nie sukcesem modelowania.
- Nie uruchamiano nowego builda WASM, całego zestawu testów, nowej generacji Meshy/Tripo, Blendera, Toolsetu ani NWN. Nie wykonano rzeźbienia ani poprawiania wag poza pipeline'em.

Techniczny następny etap to przygotowanie wariantu redukcji zachowującego powierzchnię oraz lokalna korekta anatomii/regionów wag przed ponowną próbą. To są prace do wykonania, nie istniejąca funkcjonalność, którą dzisiejszy raport ogłasza naprawioną. Należy też rozstrzygnąć los dokładnego V9 przed zamrożeniem następnego kandydata.
