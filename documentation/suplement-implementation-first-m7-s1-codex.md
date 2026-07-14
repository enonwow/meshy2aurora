# Suplement implementation-first dla M7 i S1

Data: 2026-07-14 | Autor: Codex | Status: AKTYWNY | Decyzja wlasciciela

## 1. Cel i pierwszenstwo

Ten suplement zmienia kolejnosc pracy dla aktywnej fali M7/S1. Najpierw
powstaje pierwsza, spojna implementacja wiekszosci pionowych slice'ow, a
dopiero potem rozpoczyna sie wspolna faza testow, napraw, niezaleznego review
i proof gates.

Suplement ma pierwszenstwo przed dotychczasowym wymaganiem uruchamiania pelnej
bramki testowej po kazdym malym slice oraz przed harmonogramem TDD z sekcji 3
`documentation/PROJECT_RULES.md`. Nie znosi wymaganej jakosci koncowej ani
Definition of Done. Zmienia tylko moment tworzenia i uruchamiania testow.

## 2. Twarde granice

- Implementacja i artefakty robocze powstaja tylko w
  `C:\Projects\meshy2aurora`.
- `C:\Projects\aurora-web`, Aurora/NWN EE, konfiguracje, katalogi uzytkownika,
  Toolset i gra sa w tej fali read-only i nie sa uruchamiane ani modyfikowane.
- Brak trzech oryginalnych modeli Meshy nie blokuje implementacji kontraktu,
  runnera i interfejsu, ale blokuje wykonanie korpusu oraz finalny claim M7.
- Syntetyczne fixture nie moga zostac przedstawione jako trzy profile Meshy.
- Nie implementujemy klas gameplay, pelnego generatora modulu, S2 ani F1-F10.

## 3. Zamrozony inventory vertical slices

Pierwsza fala obejmuje dziesiec slice'ow:

| ID | Slice | Pierwsza implementacja oznacza |
|---|---|---|
| M7-V1 | Corpus contract | wersjonowany manifest trzech profili, provenance, hash i wymagane role |
| M7-V2 | Corpus intake | bezpieczne rozwiazanie lokalnych plikow, kompletne wejscia i diagnostyka brakow |
| M7-V3 | Canonical batch runner | wywolanie istniejacego Rust pipeline dla kazdego wpisu bez alternatywnego konwertera |
| M7-V4 | Per-profile proof packet | deterministyczny raport, output inventory, hash i status nieudowodnionego live proofu |
| M7-V5 | Real three-profile execution | humanoid animated, non-humanoid oraz static model na oryginalnych eksportach Meshy |
| S1-V1 | Studio shell and local files | React/TypeScript shell, lokalny file picker i jawny stan sesji bez uploadu |
| S1-V2 | Canonical WASM Worker | Worker laduje publiczny adapter `m2a-wasm`; brak drugiego parsera lub mock convertera |
| S1-V3 | Source viewport | Three.js renderuje dane source/canonical IR z widoczna provenance |
| S1-V4 | Aurora/readback and validation | podglad output/readback, diagnostyka i powiazanie zaznaczenia z czescia modelu |
| S1-V5 | Artifact downloads | pobranie wygenerowanego HAK i raportow z bajtow zwroconych przez Worker |

`M7-V5` pozostaje wejściowo zablokowany do czasu dostarczenia trzech
oryginalnych modeli Meshy. Nie zmniejsza to inventory i nie pozwala oznaczyc
M7 jako DONE.

## 4. Prog rozpoczecia fazy testowej

`wiekszosc vertical slices` oznacza co najmniej `6/10` slice'ow z tabeli,
ktore maja pierwsza implementacje polaczona w jednym worktree. Do progu musza
nalezec co najmniej:

- `M7-V1` i `M7-V3`;
- `S1-V1`, `S1-V2` i `S1-V5`;
- jeden z `S1-V3` albo `S1-V4`.

Pelna faza testowa moze zaczac sie po osiagnieciu tego progu. Preferowany
moment to zakonczenie wszystkich aktualnie implementowalnych slice'ow, ale
testow nie wolno przesuwac poza pierwszy spojny przebieg implementacyjny.

## 5. Co robimy podczas implementation-first

Podczas pierwszej implementacji:

- piszemy kod pionowo od wejscia do rzeczywistego outputu;
- integrujemy slice z istniejacymi publicznymi API zamiast budowac atrapy;
- zapisujemy przy kazdym slice odroczony ledger testow: happy path, bledy,
  granice i oczekiwane artefakty;
- dopuszczamy tylko minimalne kontrole techniczne potrzebne do dalszego
  skladania: parser TypeScript, `cargo check`, typecheck albo pojedynczy smoke
  uruchamiany wtedy, gdy bez niego nie da sie bezpiecznie kontynuowac;
- nie uruchamiamy po kazdym slice pelnego `cargo test --workspace`, clippy,
  wasm-pack, macierzy browserowej, niezaleznego review ani proof gates;
- nie zatrzymujemy nastepnego slice tylko po to, aby rozbudowac pokrycie
  poprzedniego.

Minimalna kontrola nie jest dowodem zakonczenia slice i nie moze zmienic jego
statusu na DONE.

## 6. Odroczona faza integracji i testow

Po osiagnieciu progu orkiestrator wykonuje jedna wspolna fale:

1. domkniecie ledgerow testowych dla zaimplementowanych slice'ow;
2. testy jednostkowe i negatywne Rust core/WASM;
3. testy Worker protocol, transferu bajtow i obslugi bledow;
4. testy UI dla file pickera, stanow, provenance, walidacji i downloadow;
5. integracyjny local-file -> Worker -> WASM -> HAK/report flow;
6. `cargo fmt --all --check`, clippy, workspace tests i build wasm32;
7. frontend typecheck/build oraz wymagane testy komponentow/integracji;
8. niezalezne review implementacji i poprawki P1/P2;
9. dopiero po realnych wejsciach Meshy: trzy proof packety M7;
10. finalna akceptacja zewnetrzna w osobnej pozniejszej fazie, bez
    modyfikowania Aurory/NWN ani ich konfiguracji.

Nie wolno uznac zielonego waskiego smoke'a za pelna bramke. Nie wolno tez
oznaczyc M7/S1 jako DONE przed wykonaniem ich koncowych testow i wymaganych
artefaktow.

## 7. Zasady commitow i review

- Przed progiem `6/10` dopuszczalne sa robocze, niespublikowane zmiany w
  worktree; nie tworzymy fikcyjnych commitow `DONE`.
- Pierwszy checkpoint powstaje po spojnej implementacji wiekszosci i przejsciu
  wspolnej fazy testowej.
- Review jest odroczone razem z pelna bramka testowa, aby oceniac polaczone
  pionowe przeplywy zamiast izolowanych fragmentow.
- Koncowy commit i push wymagaja zielonych bramek dla calego objętego zakresu.

## 8. Raportowanie stanu

Przed progiem raportujemy osobno:

```yaml
implementation_wave:
  inventory_total: 10
  first_pass_complete: 0
  test_phase_threshold: 6
  full_test_phase: DEFERRED
  review_phase: DEFERRED
  real_meshy_execution: BLOCKED_ON_INPUT
```

Status `first_pass_complete` oznacza tylko obecna implementacje w worktree.
Nie oznacza PASS, DONE ani runtime acceptance.
