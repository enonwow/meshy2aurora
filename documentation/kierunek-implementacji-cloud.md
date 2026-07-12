# kierunek-implementacji-cloud.md

Status 2026-07-12: HISTORYCZNY KIERUNEK, ZASTAPIONY DLA M3 przez `m3-profile-a-conversion-kontrakt-suplement-codex.md` i `animacje-kontrakt-profil-a-codex.md`. Ponizsze twierdzenia o kopiowaniu szkieletu/wag `c_kocrachn` i dziedziczeniu `c_Horror` nie sa aktywna polityka produktu. CEP/retail sa read-only observations; eksportowalny rig ma provenance `SYNTHETIC`, `OWNED` albo `USER_PROVIDED` z `exportAllowed=true`. Zachowujemy tekst ponizej jako historie decyzji. Sekcje sprzed "AKTUALIZACJA 2026-07-08: decyzja D7" sa rowniez historyczne, jesli sugeruja `aurora-web` jako oracle/proof/CLI albo ASCII MDL jako format runtime. Aktualny kierunek: standalone `binary MDL/MDX policy + 2DA + HAK`, self-contained proof przez NWN EE.
Data: 2026-07-08 | Status: REKOMENDACJA (do akceptacji: Mateusz)
Podstawa: mdl-2da-hak-odpowiedz-codex.md, aurora-*-codex.md (komplet 2026-07-08), meshy-api-cloud.md, cel-projektu-cloud.md.

## Dwie ścieżki produktu (wynika z ograniczeń meshy)

Meshy auto-rig/animacje działają TYLKO dla humanoidów. Konwerter musi mieć dwie ścieżki:

```yaml
paths:
  A_creature_nonhumanoid:
    input: "meshy: sama siatka (GLB, textured)"
    skeleton: "kopiowany z referencyjnego creature Aurory (np. c_kocrachn)"
    weights: "transfer nearest-surface z referencji, max 4 wpływy, normalizacja"
    animations: "z supermodelu (setsupermodel c_Horror) — zero własnych animacji"
    output: "<resref>.mdl (binary) + MDX policy + <resref>.tga + appearance.2da row + m2a_*.hak"
  B_humanoid:
    input: "meshy: mesh + auto-rig + klipy z biblioteki (każdy klip osobny GLB)"
    skeleton: "szkielet meshy przemianowany/zmapowany do konwencji Aurora"
    animations: "własne bloki newanim w MDL; mapowanie nazw meshy→NWN (walk, run, pause1, ca1slashl...)"
    output: "jak wyżej, setsupermodel NULL"
```

## Rekomendacja: zaczynamy od ścieżki A (creature non-humanoid)

Uzasadnienie:
1. Historycznie ta ścieżka zakładała tekstowy/debugowy model i proof przez `aurora-web`; po D7-D9 jest to tylko materiał referencyjny. Aktywna ścieżka wymaga binary MDL writer, polityki MDX, 2DA writer i HAK writer.
2. Zero zależności od jakości auto-rigu meshy i mapowania animacji — najmniejsza powierzchnia błędu na pierwszy E2E.
3. Ścieżka B wymaga rozwiązania mapowania animacji (biblioteka meshy → nazwy NWN, retiming, animroot) — lepiej robić to na działającym fundamencie natywnego binary MDL/MDX, który ścieżka A i tak zbuduje (binary MDL writer, 2DA writer, HAK writer są wspólne).

## Plan implementacji (TDD, wg PROJECT_RULES)

```yaml
milestones:
  M1_mdl_writer:
    goal: "binary MDL writer: model z Meshy/reference structure -> natywny MDL zgodny z Toolset/gra; ASCII dump tylko opcjonalnie do debugowania"
    resref_convention: "m2a_* (np. m2a_koc01), lowercase, <=16 znaków"
  M2_mesh_swap:
    goal: "podmiana siatki: geometria meshy + szkielet/weights z referencji → m2a_koc01.mdl"
  M3_2da_hak:
    goal: "generacja appearance.2da (row 9000 wg kontraktu Q4) + ErfHakWriter (wg buildErfV10FromEntries) → m2a_test.hak"
  M4_nwn_ee_proof:
    goal: "generated HAK + generated/minimal module → NWN EE Toolset/gra → screenshot/proof notes dla resref m2a_*"
  M5_sciezka_B:
    goal: "humanoid: mapowanie rig/animacji meshy → własne newanim; osobna koncepcja po M4"
stack: "Node.js + TypeScript (decyzja D3), repo C:\\Projects\\meshy2aurora"
```

## Domknięcie NIE WIEM z odpowiedzi Codexa

- `retail appearance.2da`: NWN EE jest zainstalowane (Steam). Zadanie: wyciągnąć retail `appearance.2da` własnym KEY/BIF readerem albo ręcznym read-only exportem; `aurora-web` może być tylko referencją algorytmu. Dopisać wzorcowy wiersz potwora do `aurora-2da-creature-codex.md`.
- `eventy hit/footstep/snd`: zgodnie z regułą — nie emitujemy eventów w MVP (animacje z supermodelu i tak je niosą na ścieżce A).
- `classification`: na ścieżce A ustalamy wartość z binary reference/dekompilacji albo oznaczamy jako jawny default writer; bez zgadywania.
- `engine resource priority`: rozstrzygamy testem w NWN EE/Toolset albo osobnym proofem, nie testem `aurora-web`.

## AKTUALIZACJA 2026-07-08: decyzja D7 — projekt standalone

meshy2aurora nie może być powiązane z aurora-web. Korekty planu:

```yaml
standalone_corrections:
  m1_oracle: "NIE subprocess aurora-web CLI; NIE ASCII jako runtime shortcut; piszemy własny parser/writer binarnego MDL wedlug specyfikacji layoutu"
  własne_komponenty:
    - "parser binary MDL — do czytania referencji"
    - "writer binary MDL — docelowy format gry"
    - "opcjonalny ASCII/debug dump — tylko diagnostyka"
    - "2DA reader/writer"
    - "ERF/HAK writer (spec z aurora-hak-erf-codex.md, bez kopiowania kodu aurora-web)"
    - "parser GLB (@gltf-transform/core — niezależna biblioteka OSS)"
  weryfikacja:
    primary: "NWN EE toolset/gra (zainstalowane) — hak wgrany do Documents\\Neverwinter Nights\\hak, moduł testowy, screenshot proof"
    secondary_optional: "aurora-web jako zewnętrzny konsument haka (bez integracji w testach meshy2aurora)"
  otwarte: "standalone-pytania-cloud.md"
```

## Decyzje wymagane od Mateusza

- Akceptacja kierunku (ścieżka A pierwsza, plan M1–M5).
- P2: pierwszy model = wariant potwora zbliżony do c_kocrachn (rekomendacja) — konkretny wygląd dowolny.
- Klucz API meshy + budżet kredytów (potrzebny dopiero w M2 — do M1 wystarczą modele referencyjne z mirrora).
