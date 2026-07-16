# Ponowny audyt projektu meshy2aurora - 2026-07-16

## Werdykt

Aktywny slice Studio jest **zielony na wszystkich uruchomionych bramkach kodowych i CI**. Nie stwierdzono otwartego P0 ani P1 w przejrzanym przeplywie `Source -> Inspect -> Build -> Review`: natywny core, publiczny WASM, Worker, produkcyjny build Studio i test Chrome/Worker/WASM przechodza.

Projekt **nie ma jeszcze finalnej akceptacji produktu**. Brakuje trzech zatwierdzonych oryginalnych wejsc Meshy oraz proofu wygenerowanego assetu w Aurora/NWN EE. To sa jawne, zewnetrzne bramki akceptacyjne, a nie ukryte bledy zielonego test suite'u.

Audyt wykonano w kanonicznym repozytorium `C:\Projects\meshy2aurora`, przy `HEAD` `9c1c56a`, po potwierdzeniu `assert-canonical-workspace.ps1`. Zastany worktree pozostaje niezatwierdzony: 15 zmodyfikowanych plikow sledzonych i 7 niesledzonych sciezek, glownie aktywny refaktor Studio oraz raporty. Audyt nie stage'uje ani nie commit'uje tych zmian.

## Potwierdzone bramki

| Obszar | Wynik biezacego rerunu | Dowod |
| --- | --- | --- |
| Workspace | PASS | Guard rozpoznal tylko `C:\Projects\meshy2aurora`. |
| Hygiene | PASS | `git diff --check` bez bledow. |
| Rust | PASS | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; lista testow zawiera 319 pozycji. |
| WASM / Node | PASS | Build `wasm32-unknown-unknown`, `wasm-pack test --node` 20/20, wygenerowany pakiet Node i boundary M5/M7. |
| Studio | PASS | `npm run typecheck`, `npm run build`, `npm test -- --run` 23 pliki / 141 testow. |
| Produkcyjny Studio Worker | PASS | `npm run test:worker-integration` 4/4: produkcyjny App, modul Worker i publiczny web-WASM. |
| NPM production CVE | PASS | `npm audit --omit=dev --audit-level=high`: 0 vulnerabilities. |

Poprzedni P0 kontraktu readbacku nie wystepuje: projektor UI przyjmuje tylko faktyczny format readera Rust `nwn1-binary-mdl`, a realny test Chrome przechodzi przez pelny build model package i widoczny review output.

## Otwarte P2

### P2-01: security gate nie jest czescia CI

`npm audit` jest zielony lokalnie, ale workflow CI nie uruchamia `npm audit`. `cargo audit` nie jest zainstalowany lokalnie, a CI rowniez nie ma bramki advisory dla Cargo. Przed publikacja nalezy dodac wersjonowana bramke zaleznosci: co najmniej `npm audit --omit=dev --audit-level=high` oraz `cargo audit`.

### P2-02: manifest Studio dopuszcza plywajace wersje zaleznosci

`apps/studio-web/package.json` deklaruje kluczowe zaleznosci jako `latest`. Lockfile stabilizuje obecny checkout i `npm ci`, ale nowa instalacja lub odswiezenie lockfile moze zmienic React, Vite, TypeScript albo Three bez celowej decyzji. Przypiac bezposrednie wersje przed wydaniem.

### P2-03: brak budzetu dla produkcyjnego bundla

Biezacy build ostrzega o glownym pliku JS `908.79 kB` (`242.73 kB` gzip) oraz WASM `2,140.22 kB` (`752.72 kB` gzip). Build jest poprawny, ale nie ma ustalonego budzetu ani gate'u regresji rozmiaru. Nalezy najpierw zdefiniowac budzet, potem zbadac code splitting/lazy loading i koszt WASM.

### P2-04: czesc liczb w evidence jest historyczna

`documentation/evidence/S1-deferred-test-ledger.md` nadal podaje starszy rozmiar glownego bundla `822.74 kB`, podczas gdy biezacy build daje `908.79 kB`. W `orchestrator-state.yaml` pole `full_test_phase` wciaz wspomina `55_STUDIO`, mimo ze aktywna bramka Studio ma 141 testow. Zaktualizowac te liczby jednym snapshotem dowodowym przy nastepnym celowym commicie; nie mieszac ich z wynikiem przyszlego E2E na prawdziwym modelu.

## Bramka poza zakresem napraw kodowych

Finalny status M7/S1 pozostaje `IN_PROGRESS`, dopoki wlasciciel nie dostarczy trzech zatwierdzonych oryginalnych modeli Meshy. Wtedy wymagany jest lokalny browser E2E dla prawdziwych inputow oraz packet proofu w Aurora/NWN EE z wygenerowanym artefaktem. Syntetyczny fixture i own-readback sa mocnym dowodem kontraktu, lecz nie zastepuja tej akceptacji runtime.

## Rekomendowana kolejnosc

1. Zachowac aktywny slice Studio i zrobic jeden celowy review/commit po decyzji wlasciciela o zakresie zmian.
2. Dodac P2-01 i przypiac zaleznosci z P2-02, a dla bundla wprowadzic mierzalny budzet z P2-03.
3. Przy tym samym swiezym rerunie skorygowac P2-04 w ledgerze i stanie orkiestracji.
4. Po dostarczeniu wejsc wykonac realny Meshy -> Studio -> Aurora/NWN proof; dopiero ten krok moze zamknac finalna akceptacje.
