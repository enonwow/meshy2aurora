# korpus-testowy-cloud.md

Status 2026-07-09: PLAN POZNIEJSZY. Wartosc z `aurora-web`/v13 moze byc uzyta jako punkt odniesienia, ale corpus/testy `meshy2aurora` musza byc standalone, syntetyczne albo env-gated read-only.
Data: 2026-07-08 | Status: NA PÓŹNIEJ — wchodzi w życie z M1 (parser jeszcze nie istnieje; teraz priorytet ma linia sample: sample-2d → sample-3d)

Strategia testowa: nie jeden wzorzec, tylko 4 poziomy korpusu o rosnącej szerokości.

## Poziom 1 — golden (1 model, asercje szczegółowe)

c_kocrachn: pełne asercje pole-po-polu (hierarchia, wagi, klipy, nagłówek) względem wartości potwierdzonych przez Codexa. Wykrywa subtelne błędy parsera.

## Poziom 2 — zmierzona grupa kontrolna (asercje liczbowe)

Modele, dla których Codex zmierzył wartości niezależnie (konwersja-meshy-odpowiedz-codex.md). Parser musi odtworzyć te liczby:

```yaml
control_group:
  - { resref: c_kocrachn,  vertices: 1311, triangles: 1130, meshes: 24, hierarchy_nodes: 38, supermodel: c_Horror }
  - { resref: c_drider,    vertices: 2022, triangles: 1238, meshes: 38, hierarchy_nodes: 46, supermodel: c_driderchf }
  - { resref: c_bugbearb,  vertices: 1254, triangles: 740,  meshes: 17, hierarchy_nodes: 26, supermodel: c_bugbeara }
  - { resref: c_goblinb,   vertices: 742,  triangles: 424,  meshes: 20, hierarchy_nodes: 29, supermodel: c_goblina }
  - { resref: c_driderchf, vertices: 2201, triangles: 1343, meshes: 44, hierarchy_nodes: 52 }
  - { resref: c_bathorror, vertices: 2002, triangles: 1014, meshes: 9,  hierarchy_nodes: 20 }
pokrycie_budowy_ciala: "czworonóg (kocrachn), pająkokształtny (drider), humanoid-potwór (bugbear, goblin), latający (bathorror)"
zrodlo_wartosci: "pomiary Codexa z v13 GLB — reference baseline, nie oracle meshy2aurora po D7-D8"
```

## Poziom 3 — sweep masowy (inwarianty, bez wartości oczekiwanych)

Przejście po WSZYSTKICH zasobach MDL w dostępnych hakach (cep3_core1.hak ma 6402 zasoby; do tego pozostałe cep3_*, lc_*). Dla każdego binarnego MDL:

```yaml
sweep_invariants:
  - "parser nie crashuje; błędy zgłaszane jako strukturalne, nie wyjątki"
  - "wszystkie pointery MDL/MDX w granicach pliku"
  - "count_vertexes > 0 dla nodów z has_mesh"
  - "liczba wpisów weights == count_vertexes dla has_skin"
  - "suma wag każdego wierzchołka ∈ [0.99, 1.01]"
  - "nazwy kości z wag istnieją w hierarchii"
  - "node tree acykliczne, parent wskazuje istniejący node"
statystyka: "raport zbiorczy: ile plików OK / z ostrzeżeniami / nieparsowalnych + histogramy (tri, kości, wpływy)"
cel: "parser hartowany na całej różnorodności formatu, nie na jednym pliku; ASCII MDL znalezione w sweep = darmowe fixtury dla parsera ASCII"
```

## Poziom 4 — vanilla (po KEY/BIF readerze)

Po zdjęciu blockera B2: sweep po modelach retail (nwn_base.key). Rozszerza korpus o modele Bioware'u (inne wersje kompilatora binary niż CEP).

## Zasady

1. Poziomy 1–2: pełne asercje w CI; poziom 3: test wolny, odpalany na żądanie (`npm run test:sweep`); poziom 4: po B2.
2. Emiter ASCII testowany lustrzanie: parse → emit → parse → te same wartości (self-roundtrip) na całej grupie kontrolnej, nie tylko golden.
3. Wartości oczekiwane poziomu 2 pochodzą wyłącznie z niezależnych pomiarów (Codex/narzędzia zewnętrzne) — nigdy z naszego parsera (zakaz samopotwierdzenia).
4. Żadne assety gry nie trafiają do repo — korpus czytany przez env paths (polityka standalone-odpowiedz-codex.md Q5).

## Zadanie dla Codexa (rozszerzenie grupy kontrolnej)

Dostarcz `korpus-testowy-oracle-codex.md` z pomiarami dla ~10 dodatkowych modeli z cep3_core1.hak o zróżnicowanej budowie (wąż/bezkończynowy, wielonogi, ogon, bardzo mały, bardzo duży) — te same pola co control_group + lista skin nodes z influencingBoneNames. To będzie też katalog kandydatów na referencje przyszłych typów potworów (decyzje P2 dla kolejnych modeli).
