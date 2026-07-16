# Pelny audyt aplikacji - 2026-07-15

## Werdykt

**NIE GOTOWA do scalenia ani oznaczenia jako `S1-V5R DONE`.** Rdzen Rust,
WASM i produkcyjny build przechodza, ale aktualny Studio nie potrafi pokazac
wyniku prawdziwego buildu M6: kontrakt `readbackJson` jest niezgodny miedzy
Rustem a nowym projektorem UI. Dodatkowo pelna bramka Studio/Chrome jest czerwona
i dokumentacja statusowa podaje nieaktualny wynik.

Audyt dotyczy kanonicznego repozytorium `C:\Projects\meshy2aurora`, galezi
`codex/m6-gff-module-proof`, w stanie roboczym zastanym 2026-07-15. Nie
zmieniano plikow implementacyjnych; w drzewie istnialy niezatwierdzone zmiany
Studio przed rozpoczeciem audytu.

## Zakres i wykonane bramki

| Obszar | Wynik | Dowod |
| --- | --- | --- |
| Format Rust | PASS | `cargo fmt --all -- --check` |
| Analiza Rust | PASS | `cargo clippy --workspace --all-targets -- -D warnings` |
| Testy native | PASS | `cargo test --workspace` |
| Build WASM | PASS | `cargo build -p m2a-wasm --target wasm32-unknown-unknown` |
| Publiczny WASM / Node | PASS | `wasm-pack test --node crates/m2a-wasm` (20/20), generated Node boundary PASS |
| TypeScript | PASS | `npm run typecheck` |
| Build Studio | PASS z ostrzezeniem | `npm run build`; glowny JS: 908.79 kB minified / 242.72 kB gzip |
| Testy Studio | **FAIL** | `npm test`: 1 plik / 18 testow fail, lacznie 138/156 pass |
| Real Chrome Worker + web-WASM | **FAIL** | `npm run test:worker-integration`: 2/4 pass, 2/4 fail |
| Audyt zaleznosci npm | PASS | `npm audit --omit=dev --audit-level=high`: 0 vulnerabilities |
| Audyt Cargo CVE | NIEWYKONANY | `cargo audit` nie jest zainstalowany lokalnie |
| Higiena diff | PASS | `git diff --check` |

## Znalezione problemy

### P0-1: prawdziwy build M6 zawsze konczy sie bledem w UI

**Dowod.** Publiczny adapter Rust serializuje wynik `inspect_binary_mdl`, ktory
ustawia `format` na `nwn1-binary-mdl`
([parse_binary_mdl.rs](../crates/m2a-core/src/mdl/parse_binary_mdl.rs)). Nowy
projektor Studio odrzuca wszystko poza `BINARY_MDL`
([projectReadback.ts](../apps/studio-web/src/features/results/projectReadback.ts)).
Dlatego realny test Chrome konczy sie:

```
Canonical readback field readbackJson.format is missing or has the wrong type
```

`App` lapie ten wyjatek i przechodzi do `BUILD_FAILED`, wiec nie pokazuje
widoku Review mimo poprawnego wygenerowania HAK/MDL przez Worker/WASM.

**Naprawa.** Ustalic jeden wersjonowany kontrakt na granicy WASM--UI. Najmniejsza
zmiana: projektor ma akceptowac i zachowac kanoniczne `nwn1-binary-mdl`; nie
wolno udawac, ze Rust zwrocil `BINARY_MDL`. Dodac test kontraktowy oparty na
rzeczywistym `readbackJson` z `buildM6ModelPackageV1`, potem odpalic cala
bramke Studio i Chrome.

### P1-1: pelny zestaw Studio jest czerwony, a testy nie zostaly zmigrowane do nowego workflow

**Dowod.** 18 testow w `apps/studio-web/src/App.test.tsx` oczekuje starego
`[aria-label="Local file session"]`, przycisku `Build model package` i panelu
M7. Aktualny `App.tsx` renderuje workflow `SOURCE -> INSPECT -> BUILD -> REVIEW`
z `BuildStep` i `ReviewModelDetails`, wiec stare selektory nie istnieja.
Pierwsze dwa testy dodatkowo klikaja pierwszy przycisk strony zamiast akcji
Build. To blokuje wymagane w CI `npm test`.

**Naprawa.** Nie usuwac tych testow. Zastapic je testami nowego workflow:
wybor plikow -> poprawne inspekcje -> przejscie do Build -> sukces/failure/
cancel -> Review -> invalidacja po zmianie wejscia. Oddzielnie utrzymac testy
M7, jezeli panel M7 nadal jest czescia produktu; w przeciwnym razie jawnie
oznaczyc go jako usuniety zakres i usunac z CI oraz dokumentacji w tym samym
commicie.

### P1-2: dokumentacja i stan orkiestracji deklaruja wyniki sprzeczne z obecnym stanem

**Dowod.** `documentation/evidence/S1-deferred-test-ledger.md` podaje jednoczesnie
aktualne `3/4 PASS` i nizszy wpis `4 realne testy Chrome ... PASS`;
`documentation/orchestrator-state.yaml` dalej deklaruje `55/55 Studio`, `4/4
real Chrome` oraz `P1=0, P2=0`. Rzeczywisty audyt wykazal 138/156 Studio i 2/4
Chrome.

**Naprawa.** Przed kolejnym statusem DONE zaktualizowac ledger i
`orchestrator-state.yaml` do faktycznie uruchomionych wynikow, dodac P0-1 oraz
P1-1 jako otwarte blockers i usunac twierdzenie o finalnym re-review bez
aktualnego czystego runu.

### P2-1: wersje zaleznosci Studio sa oznaczone jako `latest`

`package-lock.json` obecnie stabilizuje `npm ci`, lecz `package.json` pozostawia
React, Three, Vite, TypeScript i wiekszosc narzedzi jako `latest`. Kazde
odswiezenie lockfile moze jednoczesnie zmienic caly stos frontendowy. To nie
jest znaleziona podatnosc (npm audit: 0), ale jest ryzykiem powtarzalnosci.

**Naprawa.** Przypiac wersje bezposrednich zaleznosci do zweryfikowanych
wersji semver, zachowac lockfile i dodac kontrolowany harmonogram aktualizacji.

### P2-2: przekroczony budzet glownego bundle

Vite raportuje `index-*.js` **908.79 kB** po minifikacji (limit ostrzezenia:
500 kB). Nie blokuje lokalnego produktu, lecz zwieksza czas pierwszego wejscia.

**Naprawa.** Zmierzyc najpierw udzial Three/WASM, potem lazy-loadowac widoki
preview/review i ewentualnie rozdzielic chunk Three. Ustalic budzet oraz test
CI zamiast wylaczac ostrzezenie.

## Bezpieczenstwo i integralnosc

- Aplikacja jest local-first: w kodzie Studio nie znaleziono wywolan sieciowych,
  storage sesji, `eval`, ani dynamicznego HTML.
- `npm audit --omit=dev --audit-level=high` zwrocil 0 znanych podatnosci.
- `cargo audit` nie byl dostepny (`no such command: audit`), dlatego skan CVE
  Cargo jest luka w tym audycie i powinien trafic do lokalnego toolchainu lub CI.
- Skan nazw sekretow znalazl tylko material instrukcyjny, m.in. przykladowe
  `password = "admin123"` w dokumentacji; nie jest to credential aplikacji.
- Nie znaleziono sledzonych plikow wiekszych niz 5 MiB. `git diff --check` jest
  czysty.

## Kolejnosc wyjscia z blokady

1. Zamknac P0-1 jednym kontraktem readback Rust--WASM--UI i testem z realnym
   artefaktem.
2. Zmigrowac testy `App.test.tsx` do obecnego workflow oraz rozstrzygnac status
   M7 w glownym App.
3. Uruchomic ponownie: `npm test`, `npm run test:worker-integration`,
   `npm run build`, wszystkie bramki Rust/WASM oraz `git diff --check`.
4. Dopiero po zielonym komplecie zaktualizowac ledger, stan orkiestracji i
   ocene S1-V5R.
5. Dodac `cargo-audit`, przypiete wersje frontendowe oraz budzet bundla jako
   osobny hardening.

## Ograniczenia audytu

Brak trzech zatwierdzonych, oryginalnych wejsc Meshy nadal oznacza, ze nie
wykonano koncowego E2E dla prawdziwego assetu ani proofu w Aurora/NWN. Nie
blokowalo to oceny obecnego synthetic/local Studio, ale niezaleznie blokuje
twierdzenie o koncowej akceptacji runtime.
