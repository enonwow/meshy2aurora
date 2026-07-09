# decyzje-i-zadania-cloud.md
Aktualizacja Codex 2026-07-09: aktywny kierunek po audycie to standalone `meshy2aurora`. Stare wzmianki o `aurora-web` jako target/proof/CLI sa historyczne lub reference-only, chyba ze nowsza decyzja mowi inaczej.

Rejestr decyzji projektowych i otwartych zadań. Aktualizowany przez Claude po każdej rundzie. Ostatnia aktualizacja: 2026-07-08.

## Decyzje podjęte

| ID | Decyzja | Kto | Data |
|---|---|---|---|
| D1 | Kanoniczny folder dokumentacji: `C:\Projects\meshy2aurora\documentation` | Mateusz | 2026-07-08 |
| D2 | Protokół wymiany: `reguly-dokumentacji-cloud.md` | Claude+Codex | 2026-07-08 |
| D3 | Stack konwertera: Node.js + TypeScript, CLI batch-first | Codex | 2026-07-08 |
| D4 | MVP: creature (direct model), wzorzec c_kocrachn/c_horror | Mateusz | 2026-07-08 |
| D5 | Animacje: retarget na istniejące klipy NWN, nie 1:1 z meshy | Codex+Claude | 2026-07-08 |

| D6 | Cel właściwy: pełny model z meshy (mesh+rig+animacje) → natywny content Aurory: MDL + 2DA + HAK. Strategia B zdegradowana do skrótu testowego. | Mateusz | 2026-07-08 |
| D7 | meshy2aurora jest projektem STANDALONE — bez zależności runtime/build od aurora-web (żadnych importów ani subprocess CLI w kodzie i testach). Wolno POSIŁKOWAĆ SIĘ rozwiązaniami aurora-web jako referencją wiedzy (czytanie kodu, kotwice, algorytmy), ale implementacja musi być własna — może okazać się skuteczniejsza. Weryfikacja podstawowa: NWN EE (toolset/gra); aurora-web jako zewnętrzny konsument standardowego haka. | Mateusz | 2026-07-08 (doprecyzowane) |
| D8 | Rozjazdy po audycie 2026-07-09 poprawiamy przez zasade: dokumenty sprzed D7 zostaja jako historia/reference-only, a aktywna implementacja idzie przez `architektura-meshy2aurora-codex.md`, `standalone-odpowiedz-codex.md`, `engine-mdl-pytania-cloud.md` i ten rejestr. | Codex | 2026-07-09 |
| D9 | Nie robimy ASCII MDL jako ścieżki runtime/proofu. Aktywny output projektu to natywne zasoby gry: binary MDL (+ MDX zgodnie z polityką Q2), 2DA, HAK. ASCII może być tylko debug dump/golden snapshot. | Mateusz+Codex | 2026-07-09 |

## Decyzje oczekujące

| ID | Decyzja | Czeka na | Dokument |
|---|---|---|---|
| P1 | ~~Strategia B~~ ZASTĄPIONA przez D6 | — | `cel-projektu-cloud.md` |
| P2 | Wybór pierwszego creature do wygenerowania | Mateusz | `koncepcja-meshy-cloud.md` |
| P3 | ~~Animacje meshy vs supermodel~~ Rozstrzygnięte przez ograniczenie meshy (auto-rig tylko humanoidy) → ścieżka A: supermodel; ścieżka B (humanoid, animacje meshy) jako M5 | — | `kierunek-implementacji-cloud.md` |
| P4 | Akceptacja kierunku: ścieżka A pierwsza, plan M1–M5 | Mateusz | `kierunek-implementacji-cloud.md` |
| P5 | Klucz API meshy + budżet kredytów (potrzebne od M2) | Mateusz | `kierunek-implementacji-cloud.md` |
| P-tech | Potwierdzenie stacku standalone: Node.js >=22 + TypeScript 5.9 + Jest/ts-jest + npm | Mateusz | `architektura-meshy2aurora-codex.md` |
| P-proof | Potwierdzenie, ze pierwszy proof techniczny idzie na proxy `c_kocrachn`, a The Last City ma osobny pozniejszy kontrakt artystyczny | Mateusz | `audyt-dokumentacji-plan-2026-07-09-codex.md` |

## Zadania otwarte

## Aktywny plan po audycie 2026-07-09

```yaml
active_truth:
  project: "C:\\Projects\\meshy2aurora"
  documentation: "C:\\Projects\\meshy2aurora\\documentation"
  architecture: "C:\\Projects\\meshy2aurora\\documentation\\architektura-meshy2aurora-codex.md"
  audit: "C:\\Projects\\meshy2aurora\\documentation\\audyt-dokumentacji-plan-2026-07-09-codex.md"
  implementation_target: "standalone converter: Meshy -> native binary MDL/MDX policy + 2DA + HAK"
  primary_proof: "NWN EE Toolset/gra"
  aurora_web_policy: "read-only reference; not dependency/CLI/oracle/validator/test runtime"
  external_assets_policy: "retail/CEP read-only via env; generated proof assets inside meshy2aurora"
open_decisions:
  P_tech: "confirm Node.js >=22 + TypeScript 5.9 + Jest/ts-jest + npm"
  P_proof: "confirm first technical proof on c_kocrachn proxy; The Last City later as separate original asset contract"
```

Dokumenty sprzed D7, ktore opisuja `aurora-web` jako target/proof/CLI, sa od 2026-07-09 traktowane jako historyczne albo reference-only. Nie wolno z nich brac aktywnego planu implementacji, jesli przecza D7 albo temu blokowi.

### Codex
- [x] `koncepcja-meshy-pytania-cloud.md` → odpowiedziano 2026-07-08 (wszystkie POTWIERDZONE).
- [x] `mdl-2da-hak-pytania-cloud.md` → odpowiedziano 2026-07-08 (Q1–Q7).
- [x] Dokumentacja referencyjna (6 dokumentów `aurora-*-codex.md`, `ekosystem-narzedzia-codex.md`) → dostarczono 2026-07-08.
- [ ] Wyciągnąć retail `appearance.2da` z instalacji NWN EE (czytnik KEY/BIF w aurora-web) i dopisać wzorcowy wiersz potwora do `aurora-2da-creature-codex.md` (domyka NIE WIEM).
- [x] `implementacja-m1-pytania-cloud.md` → odpowiedziano 2026-07-08 (Q1–Q7 POTWIERDZONE).

### Blockery otwarte (z standalone-odpowiedz-codex.md)
- B1: NwnMdlComp nie znajduje instalacji NWN → tylko przyszły cross-check, nie oracle M1.
- B2: dokładna lokalizacja retail c_horror w KEY/BIF nieustalona → test integracyjny retail czeka na nasz KEY/BIF reader lub ręczne potwierdzenie w nwnexplorer.
- B3: brak headless proofu nwmain/nwtoolset → proof M1 manualno-wizualny (screenshoty wg toolset_runbook).

### Claude — plan M1 STANDALONE (zaktualizowany)
- [ ] Repo: package.json (npm, TS 5.9, jest/ts-jest, Node >=22), tsconfig strict, env config (M2A_NWN_ROOT, M2A_CEP_CORE1_HAK...).
- [ ] Własny czytnik HAK/ERF V1.0 (read) + parser binary MDL wg layoutu z standalone-odpowiedz-codex.md (header→node tree→controllery→mesh/MDX→skin→animacje).
- [ ] Testy: syntetyczne fixtures w repo; integracja z cep3_core1.hak/c_kocrachn przez env (skip gdy brak). Bez commitowania assetów gry (polityka Q5).
- [ ] Binary MDL writer + MDX policy + golden snapshot strukturalny. ASCII dump tylko opcjonalnie do debugowania.

### Claude (nowe, po konwersja-meshy-odpowiedz-codex.md)
- [ ] Wygenerować fixtury syntetyczne: axis-orientation-probe.glb (strzałka +Z, kolorowe osie XYZ, marker góra/przód) i uv-probe.glb (quad z kolorowanymi rogami UV) → `C:\Projects\meshy2aurora\fixtures\` — rozstrzygają Q1 (forward) i Q6 (flip V) testem wizualnym.
- [ ] Wpisać do spec konwertera potwierdzone parametry: skala przez bbox referencji (nie metry), budżet 1000–1500 tri (warn>5000, reject>10000), tekstury TGA type2 512–1024, flip V przy TEXCOORD_0→tverts, transfer wag per segment referencji ograniczony do influencingBoneNames.

### Codex
- [ ] **PILNE**: odpowiedzieć na `engine-mdl-pytania-cloud.md` (Q1: minimalny binary MDL writer; Q2: MDX embedded vs osobny zasób; Q3: bind pose w nwnexplorer).
- [x] `architektura-meshy2aurora-codex.md` -> utworzono 2026-07-09 jako aktywny dokument architektury standalone.
- [ ] `sample-2d-generacja-cloud.md`: przygotować prompty OpenAI dla próbki koc01 (po otrzymaniu screenshotów referencji) → `sample-2d-prompty-codex.md`; poprosić Mateusza o generację obrazów.
- [ ] (PÓŹNIEJ, z M1) `korpus-testowy-cloud.md`: dostarczyć `korpus-testowy-oracle-codex.md` — pomiary ~10 modeli z cep3_core1.hak (oracle poziomu 2).

### Mateusz
- [ ] **KROK 1**: screenshoty c_kocrachn z nwnexplorer → `sample-2d\_reference\c_kocrachn\` (front/side/quarter + manifest).
- [ ] **KROK 2**: wygenerować w OpenAI obrazy 2D wg promptów Codexa → `sample-2d\koc01\`.
- [ ] **KROK 3**: meshy Image-to-3D wg `meshy-przygotowanie-modelu-cloud.md` → `sample-3d\m2a_koc01\` (GLB + manifest).
- [ ] P4: akceptacja kierunku M1–M5 (ścieżka A pierwsza).

### Claude
- [ ] Po odpowiedzi Codexa na Q1–Q2: spec konwertera `konwerter-spec-cloud.md` i start implementacji szkieletu CLI.
- [ ] Przygotować obraz referencyjny bind pose c_kocrachn dla generacji meshy (po odpowiedzi na Q4).

## Kolejność

AKTUALNE PO D7/D8:

```text
F0 dokumentacja/rules -> F1 engine-mdl Q1-Q3 + P-tech -> scaffold standalone CLI
-> parser/writer TDD -> generated HAK/module -> proof NWN EE Toolset/gra
-> dopiero potem opcjonalny test aurora-web jako zewnetrznego konsumenta HAK-a
```

Historyczna kolejnosc sprzed D7 zostaje zamrozona i nie jest planem wykonawczym:

```text
Q1-Q4 (Codex) -> P1/P2 (Mateusz) -> spec + szkielet CLI (Claude) -> siatka z meshy (Mateusz) -> stary nieaktywny CDP proof
```
