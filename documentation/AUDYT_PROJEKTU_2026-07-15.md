# Audyt projektu meshy2aurora - 2026-07-15

## Werdykt

Projekt ma **zielony rdzen Rust/WASM i czerwony aktywny frontend Studio**.
Aktualny worktree nie nadaje sie do commita ani pushu: produkcyjny przeplyw
Studio przez Worker/web-WASM nie przechodzi, a wymagane przez CI testy UI sa
czerwone. Nie jest to jednak regresja przypisana zatwierdzonemu `main`:
niezgodnosc formatu readbacku zostala dodana w obecnym, niestage'owanym
refaktorze Studio.

Audyt obejmuje dokladnie kanoniczne repozytorium
`C:\Projects\meshy2aurora`, z `HEAD` `9c1c56a` oraz zastanym dirty worktree.
Nie zmieniono kodu istniejacego slice'u; dodano wylacznie ten raport.

## Aktualizacja po naprawie

Ten audyt zarejestrowal stan przed naprawa. Nastepnie kontrakt readbacku oraz
testy Studio zostaly poprawione i uruchomione ponownie: Rust/Clippy/workspace,
typecheck, build, `npm test` **141/141**, Chrome/Worker/web-WASM **4/4** oraz
`npm audit` przechodza. Finalna akceptacja na trzech oryginalnych wejsciach
Meshy i w Aurora/NWN nadal pozostaje osobna, otwarta bramka.

## Aktualne bramki

| Obszar | Wynik | Dowod / wniosek |
| --- | --- | --- |
| Canonical workspace | PASS | `assert-canonical-workspace.ps1` rozpoznaje `C:\Projects\meshy2aurora`. |
| Git hygiene | PASS z zastrzezeniem | `git diff --check` przechodzi, ale `main` ma 12 zmodyfikowanych oraz kilka nowych plikow Studio i dokumentacji. |
| Rust format i Clippy | PASS | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`. |
| Rust workspace | PASS | `cargo test --workspace` przechodzi. |
| Publiczny WASM | PASS | `cargo build -p m2a-wasm --target wasm32-unknown-unknown` przechodzi. |
| Studio typecheck i production build | PASS | `npm run typecheck`, `npm run build`. |
| Studio unit tests | **FAIL** | `npm test -- --run`: 138/156 testow, 18 fail w `src/App.test.tsx`. |
| Real Studio Worker + web-WASM | **FAIL** | `npm run test:worker-integration`: 2/4; build M6 pada w projektorze readbacku, test App dodatkowo szuka usunietego UI. |
| Production npm CVE | PASS | `npm audit --omit=dev --audit-level=high`: 0 vulnerabilities. |
| Cargo CVE | **NIEWYKONANE** | `cargo audit` nie jest zainstalowane (`no such command: audit`). |
| Skan obvious secrets w kodzie | PASS | Brak wartosci credential w kodzie aplikacji; dopasowania to zwykle tokeny parsera 2DA. |

## P0 - kontrakt readbacku blokuje prawdziwy build Studio

W aktualnym diffie `projectReadback.ts` wprowadzil warunek
`format === "BINARY_MDL"`. Wlasny reader Rust emituje natomiast
`"nwn1-binary-mdl"` w `parse_binary_mdl.rs`. Realny test Chrome potwierdza
skutek: `Canonical readback field readbackJson.format is missing or has the
wrong type`.

To jest **regresja niezatwierdzonego refaktoru**, nie defekt obecnego `HEAD`:
w `HEAD` projektor przekazywal format z raportu bez tej walidacji. Nie wolno
wiec ani commitowac obecnego UI, ani zmieniac historycznego statusu `main` na
podstawie tej konkretnej usterki. Nalezy wybrac jeden wersjonowany kontrakt na
granicy Rust--WASM--UI i dopisac test oparty o rzeczywiste `readbackJson` z
`buildM6ModelPackageV1`.

## P1 - testy i dokumentacja nie opisuje tego samego produktu

### Testy Studio

18 bledow `App.test.tsx` oczekuje starego panelu `Local file session`,
przycisku `Build model package` oraz panelu M7 w glownym `App`. Nowy UI ma
flow `SOURCE -> INSPECT -> BUILD -> REVIEW`; testy nalezy migrowac, a nie
wylaczac. Drugi test Chrome rowniez korzysta ze starych selektorow, ale
pierwszy test Chrome ujawnia niezalezny P0 kontraktu readbacku.

### Statusy i evidence

`documentation/evidence/S1-deferred-test-ledger.md` deklaruje jednoczesnie
aktualne `3/4 PASS` i historyczne `55` testow Studio oraz `4` realne testy
Chrome. `documentation/orchestrator-state.yaml` nadal oznacza `S1-V5R` jako
`DONE` i powtarza `55/55` oraz `4/4`. Biezacy rerun obecnego worktree daje
odpowiednio `138/156` i `2/4`.

Wnioski sa dwa:

1. historyczne liczby w ledgerze nie sa dowodem dla nowego refaktoru;
2. po naprawie testow trzeba zaktualizowac oba dokumenty jednym snapshotem
   dowodowym, bez mieszania wyniku poprzedniego UI z wynikiem obecnego.

Do tego momentu aktywny slice Studio jest **BLOCKED**, a nie `DONE`.

## P2 - utrzymanie i dostarczanie

- `package.json` trzyma React, Three, Vite, TypeScript i wiekszosc typow jako
  `latest`. Lockfile stabilizuje obecny checkout, lecz kolejna instalacja moze
  zmienic wiele warstw naraz. Przypiac bezposrednie wersje po zamknieciu P0.
- CI uruchamia format, Clippy, Rust, WASM, build Studio oraz oba zestawy testow
  Studio, wiec obecny worktree zostanie zatrzymany przed publikacja. Nie ma w
  nim jednak `npm audit` ani `cargo audit`; pierwszy przechodzi lokalnie,
  drugiego nie da sie jeszcze uruchomic.
- Vite ostrzega o `908.79 kB` JS (`242.72 kB gzip`) i `2.14 MB` WASM
  (`752.72 kB gzip`). To nie jest blocker funkcjonalny, ale nalezy ustalic
  budzet i zbadac lazy loading po przywroceniu bramek.
- 12 sledzonych mockupow Studio zajmuje `21,072,893` B. Jest to sensowny,
  jawny koszt dowodow wizualnych, ale powinien pozostac ograniczonym katalogiem
  referencyjnym, a nie wzorem dla zrzutow z kazdego przebiegu.

## Rekomendowana kolejnosc naprawy

1. Zamknac P0 jednym testem kontraktowym Rust--WASM--UI i poprawic format
   readbacku bez zmiany semantyki danych z readera.
2. Przepisac `App.test.tsx` oraz produkcyjny test Chrome na aktualny workflow;
   objac success, failure, cancel, stale response i reset inputu.
3. Uruchomic ponownie pelny zestaw z tabeli, w tym `npm audit`; dopiero wtedy
   zaktualizowac ledger oraz `orchestrator-state.yaml` aktualnymi licznikami.
4. Zrobic jeden celowy commit tylko po zielonych bramkach. Nie mieszac go z
   przypieciem zaleznosci, `cargo audit` ani optymalizacja bundla.

## Granice

Brak trzech zatwierdzonych, oryginalnych modeli Meshy nadal blokuje koncowe
E2E na prawdziwym wejsciu oraz proof w Aurora/NWN. Zielony Rust/WASM i
syntetyczny browser test nie zastepuja tej koncowej bramki runtime.
