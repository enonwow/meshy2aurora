# Creature WebMCP v1 — 2026-09-06

Status: **sterowanie zaimplementowane i wywołane przez natywne WebMCP; model Tripo pozostaje zablokowany przed eksportem**.

## Cel i zakres

Zadanie właściciela 01a06e72-ad4e-7533-814c-12d41f2c1097: po odrzuceniu próby w Blenderze użyć dokładnie pipeline'u Creature w Meshy2Aurora dla dostarczonego Tripo GLB i c_wolf; uzupełnić szybkie sterowanie przez WebMCP. Przeczytano zadanie „Audit creature” (01a00f31-d2af-7d80-8043-3f5f7ed0eff8). Ostatnia próba tamtego zadania dotyczyła innego, starszego źródła Meshy o 1 997 064 trójkątach. Nie przeniesiono jego wyniku ani ograniczenia rozmiaru na plik Tripo.

Źródło bieżącej próby:
- sample-3d/borzoi-tripo-dc22ecb0/source.glb, kopia identyczna z C:/Users/enonw/Downloads/wolf+3d+model.glb;
- SHA-256 dc22ecb0561fc10773e1a156b0d126225a15cdfb67cd7c0999a965a4f1d59689;
- 6 687 432 B, 25 830 wierzchołków renderowania, 20 534 trójkąty;
- kierunek wejścia POSITIVE_Z, oryginalne materiały, geometria i tekstura zachowane.

Referencja c_wolf została odczytana w pamięci z lokalnego nwn_base.key / BIF własnym readerem aplikacji: 340 812 B, SHA-256 a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726. Łańcuch c_wolf → NULL. Nie zapisano payloadu retail w projekcie i nie zmieniono plików gry.

## Wdrożone sterowanie

Panel: [Creature w lokalnym Studio](http://127.0.0.1:5186/?creatureAgent=1). Trasa produktu: ?creatureAgent=1, w apps/studio-web/src/main.tsx. Jest to panel wywołań istniejącego StudioWorkerClient, nie drugi algorytm nadawania wag ani emiter MDL.

Implementacja: apps/studio-web/src/features/creature-automation/.

Natywna rejestracja używa document.modelContext.registerTool z AbortSignal i obsługuje wcześniejszy wariant navigator.modelContext. Oparta na aktualnym [kontrakcie WebMCP](https://webmachinelearning.github.io/webmcp/) i [dokumentacji Chrome](https://developer.chrome.com/docs/ai/webmcp/imperative-api). Gdy natywny mechanizm jest niedostępny, UI jawnie zgłasza fallback window.m2aCreatureTools; nie nazywa go aktywnym natywnym WebMCP.

| Narzędzie | Operacja |
| --- | --- |
| m2a_creature_status | Tożsamość źródła i chainu, rewizja, stan asynchronicznego zadania, blokada, artefakty |
| m2a_creature_load_source | Odczyt GLB z tego samego origin, sprawdzenie SHA-256, INSPECT_SOURCE dla Creature |
| m2a_creature_select_supermodel | Wybór dowolnego resrefu obsługiwanego przez reader retail, rozwiązanie chainu przez KEY/BIF |
| m2a_creature_prepare | PREPARE_REFERENCE_SUPERMODEL_RIG_V2, przy domyślnie włączonych kontrolach |
| m2a_creature_set_authoring | Dokument V2 komponentów/regionów/wag związany ze źródłem i szkieletem; bez przesuwania przegubów |
| m2a_creature_preview | BUILD_REFERENCE_SUPERMODEL_AUTHORED_PREVIEW, odczyt binary MDL i kontrola odziedziczonego ruchu |
| m2a_creature_build | BUILD_MODEL_PACKAGE / REFERENCE_SUPERMODEL_CREATURE po pełnym dopuszczeniu offline |
| m2a_creature_report | Stronicowany JSON i opcjonalny JSON Pointer path do poddrzewa, bez konieczności pobierania całej geometrii |
| m2a_creature_artifact | Ograniczone porcje base64 wyłącznie wygenerowanych artefaktów, z ich SHA-256 |

Każda mutacja wymaga expectedRevision z ostatniego statusu. Start zwraca identyfikator zadania bez blokowania narzędzia; status rozróżnia RUNNING/SUCCEEDED/FAILED. Równoległe mutacje i nieaktualne rewizje są odrzucane. Ponowne przygotowanie zachowuje dotychczasowy dokument korekt. Wybór nowego źródła lub referencji unieważnia zależne wyniki dopiero po udanym odczycie. Przy błędzie eksport pozostaje wyłączony.

Przykład wywołań przez obiekt narzędzi WebMCP przeglądarki:

```js
await tools.call('m2a_creature_load_source', {
  expectedRevision: 0,
  url: '/@fs/C:/Projects/meshy2aurora/sample-3d/borzoi-tripo-dc22ecb0/source.glb',
  sha256: 'dc22ecb0561fc10773e1a156b0d126225a15cdfb67cd7c0999a965a4f1d59689',
  sourceForward: 'POSITIVE_Z'
});
// Odczytaj status i poczekaj na zakończenie operacji przed kolejną mutacją.
await tools.call('m2a_creature_status', {});
await tools.call('m2a_creature_select_supermodel', {
  expectedRevision: 1, resref: 'c_wolf', nwnRootUrl: '/__m2a_nwn_reference'
});
await tools.call('m2a_creature_status', {});
await tools.call('m2a_creature_prepare', {expectedRevision: 2});
await tools.call('m2a_creature_report', {name: 'failure'});
```

Serwer deweloperski wymaga jawnego M2A_NWN_REFERENCE_ROOT. Nie rozszerzono istniejącego zakresu serwowania plików Vite. Proces serwera uruchomiono z windowsHide: true, bez konsoli; metadane i log w artifacts/diagnostics/borzoi-tripo-cwolf-webmcp-20260906.

## Wykonana próba i przyczyna zatrzymania

1. Publiczna funkcja WASM prepareReferenceSupermodelRigV3 na niezmienionym źródle: **M2A-REFERENCE-SUPERMODEL-SKIN-BRANCH-REPAIR-EXCESSIVE** — propozycja zmiany 2140 wierzchołków, limit strukturalny 1291.
2. Natywne WebMCP w przeglądarce Codex: 9 narzędzi odkrytych; load_source SUCCEEDED, select_supermodel SUCCEEDED, prepare FAILED z tym samym kodem i liczbami. UI potwierdził dokładny hash źródła i c_wolf; canExport=false, artifacts=[]. Nie było osobnego skryptu riggującego Blendera w tym przebiegu.
3. Osobna kontynuacja diagnostyczna publicznego WASM, z istniejącą eksperymentalną opcją pominięcia wyłącznie pierwszego limitu, nadal **nie wygenerowała MDL/HAK/MOD ani zatwierdzonego rigu**. Następna kontrola zwróciła M2A-REFERENCE-SUPERMODEL-SKIN-LOCAL-COVERAGE-BLOCKED: 16 wymaganych kości nie ma widocznego regionu siatki. Opcja nie jest dostępna w nowym WebMCP i nie zmieniono dopuszczenia eksportu.

Lista z drugiej kontroli: Wolf_LfrontbotlegB, Wolf_Lfrontpaw, Wolf_Rfrontupperleg, Wolf_RfrontbotlegA, Wolf_RfrontbotlegB, Wolf_Rfrontpaw, Wolf_tail, Wolf_tailend, Wolf_Lbacktopleg, Wolf_Lbackmidleg, Wolf_Lbackbotleg, Wolf_Lbackpaw, Wolf_Rbacktopleg, Wolf_Rbackmidleg, Wolf_Rbackbotleg, Wolf_Rbackpaw.

Wniosek implementacyjny: zwiększenie limitu naprawy nie wystarczy. Obecne automatyczne przypisania tracą wymagane regiony ruchu; nie ma podstaw, by wydać je jako poprawny model. Te wyniki nie dowodzą, że źródłowa siatka Tripo jest bezużyteczna ani że dokładnie 2140 jej wierzchołków jest wizualnie uszkodzonych.

Potwierdzona luka w kodzie: przygotowanie bazowego rigu wykonuje automatyczne skinning przed zastosowaniem dokumentu authoring. Gdy auto-skinning rzuci błąd, narzędzie set_authoring również nie uzyska bazowego dokumentu do korekty. Rejestracja WebMCP tego nie naprawia.

Dalsza potrzebna poprawka algorytmu: oddzielny wynik diagnostyczny rejestracji/anatomii i bazowych etykiet, dostępny także przy odrzuceniu wag; wizualizacja propozycji korekt oraz utraconych regionów; lokalne przypisania/ograniczenia i dopasowanie powierzchni stosowane przed obliczaniem wag, przy niezmiennym bind; następnie ponowne kontrole struktury i ruchu. Nie uznawać samego pominięcia limitu za naprawę. Pełny fitter anatomiczny nie został w tym zakresie zaimplementowany.

Dowody:
- artifacts/diagnostics/borzoi-tripo-cwolf-20260906/run.json i prepare-original.json;
- artifacts/diagnostics/borzoi-tripo-cwolf-diagnostic-continuation-20260906/run.json i prepare-original.json;
- artifacts/diagnostics/borzoi-tripo-cwolf-webmcp-20260906/browser-run.json;
- tests/typecheck zapisane obok browser-run.json.

## Weryfikacja i granice

Testy obejmują wymagania centralnego dopuszczenia, atomowość błędnego hasha, obcy origin, równoległość/stare rewizje, zachowanie strukturalnego błędu workera, odmowę eksportu przed dopuszczeniem, stronicowanie, zakaz eksportu retail, brak ogólnego eval/bypass, rejestrację/cleanup WebMCP i jawny fallback. Wykonano również pełną kontrolę TypeScript oraz guard canonical workspace/sample-3d.

Rzeczywisty test przeglądarkowy pokrywa load_source → select_supermodel → prepare → odczyt konkretnego błędu; nie jest pozytywnym testem eksportu tego psa. Nie powstał nowy poprawny model, nie zamrożono nowego MOD/HAK i nie rozpoczęto Toolset/NWN. Poprzedni odrzucony wynik Blendera i inne zamrożone linie pozostały bez zmian.

Ograniczenia wersji sterowania: nowy panel ma sesję pamięciową i zapis raportu, a nie pełne wznowienie projektu po przeładowaniu; selektor WebMCP obsługuje retail KEY/BIF, a istniejąca biblioteka HAK w głównym Studio wymaga osobnego podłączenia. Poprawność wyglądu oraz zgodność w NWN pozostają niezweryfikowane.

Próba użyła zastanego pakietu crates/m2a-wasm/pkg; backend Rust nie był modyfikowany ani przebudowywany w tym zadaniu. Hash i data WASM oraz hashe nowych plików są w implementation-files.json. Nie jest to deklaracja, że niezapisane zmiany innych zadań w Rust odpowiadają temu binarium.
