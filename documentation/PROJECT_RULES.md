# Project Rules

## 0. Kanoniczny workspace — HARD STOP

Jedynym repozytorium i zapisywalnym workspace projektu jest
`C:\Projects\meshy2aurora`.

`C:\Users\enonw\Documents\meshy2aurora` jest sciezka bezwzglednie zakazana.
Nie wolno tam tworzyc, edytowac, stage'owac, kopiowac, migrowac, testowac,
budowac ani przechowywac plikow tymczasowych projektu. Nie jest to klon,
staging, scratch, mirror ani fallback. Wlasciciel nigdy nie wskazal ani nie
autoryzowal tej sciezki.

Kazdy agent i subagent przed pierwszym zapisem musi rozwiazac repo root. Jezeli
nie jest nim dokladnie `C:\Projects\meshy2aurora`, ma wykonac HARD STOP bez
tworzenia plikow i bez obchodzenia problemu przez drugi katalog. Task trzeba
wznowic z repo kanonicznym jako workspace root. Pelny kontrakt znajduje sie w
`documentation/CANONICAL_WORKSPACE.md` i root `AGENTS.md`.

Obowiazkowy preflight:

`powershell -NoProfile -ExecutionPolicy Bypass -File assert-canonical-workspace.ps1`

## 1. Dokumentacja

Cala dokumentacja projektu znajduje sie w folderze `C:\Projects\meshy2aurora\documentation` i tam ma byc dopisywana. Nie tworzymy rozproszonych notatek poza tym folderem bez rownoczesnego wpisu lub przeniesienia do `documentation`.

Foldery o podobnej nazwie poza `C:\Projects\meshy2aurora` nie sa kanoniczne
dla tego projektu i nie wolno uzywac ich nawet jako tymczasowego stagingu.

## 2. Aurora First

ZASADA NAJWAZNIEJSZA DLA IMPLEMENTACJI: AURORA FIRST.

Dekompilacja Aurory jest glownym zrodlem wiedzy. Nie ma strzelania. Jezeli dany watek, model albo agent nie zna odpowiedzi, najpierw szuka jej w dekompilacji `C:\Projects\New Folder`, potem w lokalnych zasobach gry/CEP/NWN EE jako read-only reference, potem w `C:\Projects\aurora-web` jako read-only reference, a dopiero pozniej w Internecie. Internet moze uzupelniac brakujacy kontekst, ale nie moze zastapic lokalnego dowodu z Aurory, jezeli ten dowod jest dostepny.

`C:\Projects\aurora-web` jest osobnym projektem. W `meshy2aurora` wolno czytac jego kod, dokumenty i artefakty jako material porownawczy, ale nie wolno uzywac go jako dependency, CLI/subprocess, oracle, walidatora, fixture source ani elementu runtime/testow. Implementacja `meshy2aurora` ma miec wlasny parser MDL, wlasny emiter/writer, wlasny writer 2DA i wlasny writer ERF/HAK.

Formatem docelowym modelu dla gry jest natywny binary MDL oraz polityka MDX rozstrzygnieta dla pierwszego profilu w `engine-mdl-odpowiedz-codex.md` Q2. ASCII MDL nie jest sciezka runtime/proofu; moze istniec tylko jako debug dump albo golden snapshot do czytania przez czlowieka.

Aurora First dla modelu oznacza lancuch: realny model Aurory -> own reader -> mapa potwierdzonych invariantow -> own IR/writer -> own readback -> Toolset/game. Reader nie produkuje assetu do gry; odkrywa kontrakt, ktory writer musi spelnic. Nie wymagamy identycznych bajtow ani nie kopiujemy payloadu, ale wymagamy analogicznej semantyki, layoutu profilu i zachowania akceptowanego przez engine.

Przed implementacja obszaru trzeba sprawdzic `documentation/macierz-gotowosci-wiedzy-codex.md` i jego kanoniczny kontrakt. Odlegly etap nie moze pozostac bez kierunku: dopuszczalny jest otwarty runtime proof, ale nie brak decyzji, zrodla i testu zamykajacego.

Kazde twierdzenie implementacyjne musi byc oznaczone jako jedno z:

- fakt z dekompilacji Aurory;
- fakt z retail/resource/binary/proof;
- aktualny status `aurora-web` jako reference-only;
- wniosek implementacyjny;
- hipoteza do sprawdzenia.

Hipoteza nie jest podstawa implementacji bez testu albo proofu.

## 2.1 Licencje i provenance zrodel

Zewnetrzne repozytoria, dekompilacja, retail i CEP sa materialem read-only. Fakty o publicznym formacie wolno niezaleznie zaimplementowac, ale nie kopiujemy kodu GPL, payloadow, animacji, szkieletow, tekstur ani fixture z tych zrodel do `meshy2aurora`. Kazdy zewnetrzny fragment kodu wymaga przed uzyciem jawnej decyzji licencyjnej i zapisu provenance.

Domyslny proof base to wlasne syntetyczne fixture oraz wygenerowany przez `meshy2aurora` HAK/modul/asset. Licencja samego repozytorium pozostaje decyzja wlasciciela przed publicznym wydaniem; brak tej decyzji nie daje zgody na kopiowanie materialow referencyjnych.

Jeden realny model nie wystarcza do uznania parsera albo writera za zgodny. Reguly corpusu wielomodelowego sa w `documentation/korpus-referencyjny-mdl-codex.md`: realne zasoby sa czytane in-place przez env-gated testy regresyjne, a CI i finalny proof pozostaja oparte na fixture/generated content.

Kazdy faktycznie uruchomiony model referencyjny wymaga packetu `P-REF`: hash wejscia, raport naszego readera, wyniki invariantow i - po dodaniu preview - widoczny screenshot lub motion capture naszego preview. Screenshot Toolsetu z niezmienionym assetem gry nie jest proofem naszego kodu.

## 3. TDD

Implementujemy zgodnie z zasada TDD. Najpierw powstaje test lub gate opisujacy oczekiwane zachowanie, potem minimalna implementacja, potem refactor i proof. Dla modeli, animacji, parserow i konwersji assetow test musi byc oparty na realnym zasobie albo na minimalnej fixture opisanej wprost jako fixture.

Wyjatek harmonogramowy zatwierdzony przez wlasciciela dla aktywnej fali M7/S1
jest opisany w `documentation/suplement-implementation-first-m7-s1-codex.md`:
najpierw pierwsza implementacja wiekszosci vertical slices, potem wspolna faza
testow, review i proof gates. Wyjatek zmienia kolejnosc, nie koncowe wymagania
jakosci ani Definition of Done.

## 4. Pliki `*-cloud.md`

Jezeli w dokumentacji pojawia sie plik w formacie `[nazwa]-cloud.md`, oznacza to, ze trzeba dostarczyc suplement cloud do tresci bazowej dokumentacji. Taki plik nie zastepuje dokumentu glownego; dopisuje wymagania, ograniczenia, decyzje, instrukcje lub roznice potrzebne dla pracy w chmurze/Codex Cloud.

Format takich plikow i format odpowiedzi cloud sa opisane w `documentation/CLOUD_SUPPLEMENT_FORMAT.md`.

## 5. Pliki `*-odpowiedz-codex.md`

Pliki w formacie `[temat]-odpowiedz-codex.md` sa odpowiedziami lokalnego Codexa na pytania Cloud z plikow `[temat]-pytania-cloud.md`. Odpowiedz musi byc pod kazdym numerowanym pytaniem `Q1`, `Q2`, ... i miec status `POTWIERDZONE`, `HIPOTEZA` albo `NIE WIEM`.

Pelne reguly wymiany Cloud/Codex sa w `documentation/reguly-dokumentacji-cloud.md`. Wymagany zestaw startowy jest opisany w `documentation/wymagania-startowe-cloud.md`.

## 6. Aktualny produkt webowy

Od decyzji D12-D14 produktem jest aplikacja webowa local-first, a nie desktopowe narzedzie ani CLI jako interfejs uzytkownika. Aktywny kontrakt znajduje sie w:

- `documentation/decyzje-i-zadania-cloud.md` (D11 i D12);
- `documentation/architektura-meshy2aurora-codex.md`;
- `documentation/architektura-web-wasm-codex.md`;
- `documentation/audyt-gotowosci-startowej-2026-07-10-codex.md`;
- `documentation/macierz-gotowosci-wiedzy-codex.md`;
- `documentation/plan-implementacji-orkiestrator-codex.md`;
- `documentation/orchestrator-state.yaml`.

Starsze dokumenty zachowuja fakty i kontekst z chwili utworzenia, ale nie moga nadpisac D11-D14. Pelna klasyfikacja wszystkich plikow jest w `documentation/status-dokumentacji-web-2026-07-10-codex.md`. Nie edytujemy historycznych plikow `*-cloud.md` tylko po to, aby zmienic ich decyzje; ich status zmienia centralny indeks i nowszy suplement Codexa.
