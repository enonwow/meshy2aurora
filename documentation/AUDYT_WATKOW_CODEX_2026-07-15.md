# Audyt czterech watkow Codex - 2026-07-15

## Werdykt

**Nie wolno oznaczac S1-V5R ani calej aplikacji jako DONE.** Cztery watki
zostawily wartosciowy kod, dokumentacje i recovery workspace'u, ale ich stan
koncowy jest niespojny z aktualnie uruchomionymi bramkami. Najwazniejszy
otwarty problem jest konkretny: Rust serializuje readback MDL jako
`nwn1-binary-mdl`, a UI akceptuje wylacznie `BINARY_MDL`. W efekcie prawdziwy
przeplyw Worker/WASM konczy sie bledem, mimo ze pakiet M6 zostal zbudowany.

Ten dokument audytuje historie oraz stan kanonicznego repozytorium
`C:\Projects\meshy2aurora`. Nie zmienia implementacji ani istniejacych,
niestage'owanych zmian Studio.

## Dowody odtworzone 2026-07-15

| Bramka | Wynik | Znaczenie |
| --- | --- | --- |
| `npm test -- --run` | **FAIL: 138/156, 18 fail** | Stare testy `App.test.tsx` oczekuja usunietego widoku M6/M7, a nie obecnego workflow Studio. |
| `npm run test:worker-integration` | **FAIL: 2/4, 2 fail** | Rzeczywisty web-WASM/Worker nie przechodzi przez projektor readbacku. |
| `projectReadback.ts` | **P0 reprodukowalne** | Twardo wymaga `format === "BINARY_MDL"`. |
| `parse_binary_mdl.rs` | **P0 reprodukowalne** | Wlasny reader emituje `format: "nwn1-binary-mdl"`. |
| `git diff --check` | PASS | Aktualny diff nie ma bledow bialych znakow. |
| `git status --branch --short` | dirty `main` | 12 zmodyfikowanych i kilka nowych plikow Studio/dokumentacji sa niestage'owane; nie sa wynikiem tego audytu. |

`orchestrator-state.yaml` nadal opisuje `S1-V5R` jako `DONE`, z dowodem
`55/55 Studio` i `4/4 real Chrome`. Jest to sprzeczne z powyzszym runem,
zatem nie moze byc zrodlem aktualnego statusu.

## Ocena watkow

| Watek | Co bylo wartosciowe | Co bylo bledem lub pozostalo otwarte | Ocena |
| --- | --- | --- | --- |
| `019f4d3b-e51b-7742-8746-09585f19e08f` - readiness | Sprawdzil toolchain i skorygowal historyczny stan gotowosci; ten etap uzasadnial start M1A bez wejsc Meshy. | Dzialal jako task przypiety do zakazanego `Documents`; koncowy komunikat o gotowosci jest historyczny, a nie aktualna akceptacja calego produktu. | **Uzyteczny, ale superseded jako status projektu.** |
| `019f50e0-adae-7bd3-864d-ff384fabc240` - orkiestracja | Doprowadzil szeroka fale M1--M6 do historii, zapisal kontrakty i rozpoznal zrodlo promptow uprawnien. | Subagent mimo zasad utworzyl `s1-staging` w `C:\Users\enonw\Documents\meshy2aurora`; to bylo naruszenie HARD STOP. Dlugowieczne branche i deklaracje DONE wyprzedzily aktualne bramki Studio. | **Znaczny dorobek techniczny, powazna wada procesu i wiarygodnosci statusu.** |
| `019f5a72-aa89-7042-bf47-95066f45aeca` - recovery workspace'u | Ujawnil naruszenie uczciwie, potwierdzil, ze staging jest przestarzaly, skonsolidowal historie do `main` i ustanowil guard kanonicznego workspace'u. | Watek nie mogl sam zmienic przypietego workspace'u; finalnie zatrzymal implementacje. Nie jest dowodem naprawy S1 ani zamkniecia P0 UI. | **Poprawne recovery, nie completion produktu.** |
| `019f64f4-3dec-7f40-9738-f8243f5872b8` - pelny audyt aplikacji | Najsilniejsza diagnoza obecnego problemu: P0 readback Rust--UI, 18 testow Studio i 2 testy Chrome. | Raport zostal pozostawiony jako niecommitowany plik, a kolejny turn odszedl do infografiki zamiast zamknac blokery. Sam audyt nie uprawnia do utrzymania S1-V5R jako DONE. | **Diagnoza trafna; naprawa i aktualizacja stanu nie zostaly wykonane.** |

## Przyczyna promptow o pozwolenia

To nie byl wymog implementacji. Wszystkie cztery zadania byly pierwotnie
przypiete do `C:\Users\enonw\Documents\meshy2aurora`, podczas gdy prawdziwe
repo bylo w `C:\Projects\meshy2aurora`. Zapis i testy w prawidlowym repo
wygladaly dla tamtych taskow jak operacje poza workspace'em, co wywolalo
prompty sandboxa. To wyjasnia mechanizm, ale nie usprawiedliwia utworzenia
`s1-staging`; poprawna reakcja byla HARD STOP.

Obecny task jest uruchomiony w kanonicznym repo i guard przechodzi. Dalsze
prace nie powinny wymagac tego obejscia ani tworzyc zadnych odpowiednikow
folderu `Documents`.

## Stan po audycie

1. **P0 -- naprawic jeden kontrakt Rust--WASM--UI.** Projektor ma przyjac
   kanoniczna wartosc emitowana przez Rust (albo kontrakt ma zostac zmieniony
   end-to-end i wersjonowany). Dodac test z rzeczywistym `readbackJson` z
   `buildM6ModelPackageV1`.
2. **P1 -- zmigrowac `App.test.tsx` do obecnego Studio.** Testy maja sprawdzac
   `SOURCE -> INSPECT -> BUILD -> REVIEW`, a nie stare selektory M6/M7.
3. **P1 -- po zielonym rerunie zaktualizowac ledger i
   `orchestrator-state.yaml`.** Do tego czasu `S1-V5R` musi byc traktowane jako
   `BLOCKED`, nie `DONE`; nie wolno wpisywac licznikow z planu jako dowodow.
4. **P2 -- osobno domknac hygiene.** Zacommitowac tylko swiadomie wybrany
   naprawiony slice, przypiac zaleznosci frontendowe i dodac `cargo audit` do
   lokalnego toolchainu lub CI. Nie mieszac tego z naprawa P0.

## Granice audytu

Nie wykonywano proofu w Toolsecie ani grze, a trzy zaakceptowane, oryginalne
wejscia Meshy nadal nie sa dostarczone. Nawet po naprawie P0/P1 pozostanie to
osobna bramka finalnej akceptacji runtime, a nie powod do symulowania sukcesu.
