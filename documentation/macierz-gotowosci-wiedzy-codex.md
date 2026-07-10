# Macierz gotowosci wiedzy

Data: 2026-07-10 | Status: AKTYWNY GATE WIEDZY PRZED IMPLEMENTACJA

## 1. Cel

Ten dokument oddziela trzy stany, ktore wczesniej byly mieszane:

- `DIRECTION_LOCKED` - wiemy, jaka sciezka implementacyjna ma byc uzyta;
- `EVIDENCE_PARTIAL` - kierunek jest ustalony, ale konkretny wariant binarny wymaga readbacku albo proofu;
- `RUNTIME_PROVED` - wygenerowany przez nas asset przeszedl Toolset i gre.

Brak etapu M4/M5 w biezacym sprincie nie oznacza zgody na brak wiedzy. Przed M1A kierunek dla calego natywnego pipeline ma byc zapisany, a otwarte kwestie musza miec jednoznaczny test zamykajacy.

## 2. Stan wiedzy

| Obszar | Stan wiedzy | Kierunek | Co pozostaje do proofu |
|---|---|---|---|
| Produkt i granice repo | `DIRECTION_LOCKED` | Web local-first, Rust/WASM, `meshy2aurora` standalone | Brak; decyzja produktowa zamknieta |
| Binary MDL reader | `DIRECTION_LOCKED` | Checked little-endian reader, core pointers wzgledem bajtu 12, raw pointers wzgledem bloku MDX | M1A/M1B readback na syntetycznych fixture i opcjonalnych lokalnych referencjach |
| Binary MDL writer | `EVIDENCE_PARTIAL` | Writer sklada `12-byte header + core + volatile`, tylko profil A i jawnie wspierane typy nodow | Rozstrzygnac konflikt skin header 17/64 mapowan kosci przez M1B corpus report; potem wlasny write/readback i Toolset |
| MDX | `DIRECTION_LOCKED` dla profilu A | Jeden zasob HAK typu 2002; MDX jest doklejonym blokiem volatile, bez osobnego typu 2003 | Wlasny write/readback i NWN EE proof |
| Animacje | `DIRECTION_LOCKED`, semantyka gameplay `EVIDENCE_PARTIAL` | Produktowy proof ma byc self-contained; klipy wlasne/generowane, retail/CEP tylko do obserwacji struktury | Mapowanie wymaganych stanow gry, eventow i petli w NWN EE |
| `appearance.2da` | `DIRECTION_LOCKED` | Uzytkownik dostarcza jawny base table lub HAK; zachowujemy kolumny i wiersze, dopisujemy nowy wiersz na koncu | Proof, ze wybrany `MODELTYPE=S`, `RACE` i `Appearance_Type` sa rozwiazywane przez gre |
| HAK/ERF | `DIRECTION_LOCKED` | Wlasny HAK V1.0 writer w Rust; deterministyczne key/resource lists; zasoby bez kompresji | Readback i NWN EE proof |
| GFF/UTC/IFO | `DIRECTION_LOCKED` strukturalnie | Wlasny GFF V3.2 writer; wygenerowany UTC i minimalny modul; `Mod_HakList` zachowuje kolejnosc | Minimalny zestaw pol UTC/GIT/IFO i runtime proof |
| Priorytet wielu HAK | `EVIDENCE_PARTIAL` | Pierwszy proof uzywa jednego wygenerowanego HAK, wiec nie zalezy od niepotwierdzonej kolejnosci override | Osobny test konfliktu dwoch HAK, jezeli produkt zacznie je laczyc |
| Licencje i provenance | `DIRECTION_LOCKED` procesowo | Zrodla GPL i zasoby gry sa reference-only; nie kopiujemy kodu ani payloadow; proof fixtures sa generowane | Wlasciciel wybiera licencje repo przed publicznym wydaniem |
| Finalna akceptacja | `NOT_PROVED` | Toolset i gra sa jedynym finalnym proofem natywnego outputu | M6 |

## 3. Kanoniczne dokumenty wiedzy

```yaml
knowledge_contracts:
  mdl: "documentation/mdl-binary-crosswalk-codex.md"
  mdx: "documentation/mdx-polityka-codex.md"
  animation_profile_a: "documentation/animacje-kontrakt-profil-a-codex.md"
  hak_2da_gff: "documentation/hak-2da-gff-crosswalk-codex.md"
  live_state: "documentation/orchestrator-state.yaml"
```

Starsze dokumenty nadal zachowuja kontekst, ale powyzsze kontrakty maja pierwszenstwo, gdy starszy tekst mowi `NIE WIEM`, wybiera wolny wiersz 2DA albo opiera produkcyjny proof na `aurora-web`.

## 4. Warunek startu implementacji

M1A nie wymaga runtime proofu M4-M6, ale wymaga spelnienia obu warunkow:

1. kierunek dla kazdego obszaru jest zapisany bez zgadywania;
2. kazda nierozstrzygnieta roznica ma nazwany test zamykajacy i nie przecieka jako stale zalozenie do kodu.

Aktualnie warunek wiedzy jest spelniony. Operacyjny start M1A nadal wymaga toolchainu Rust/WASM i jawnej zgody wlasciciela zgodnie z `audyt-gotowosci-startowej-2026-07-10-codex.md`.
