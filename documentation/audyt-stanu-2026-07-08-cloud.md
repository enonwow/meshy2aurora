# audyt-stanu-2026-07-08-cloud.md

Status 2026-07-09: HISTORYCZNY AUDYT STANU. Aktualny porzadek po poprawie rozjazdow jest w `audyt-dokumentacji-plan-2026-07-09-codex.md`, `decyzje-i-zadania-cloud.md` i `architektura-meshy2aurora-codex.md`.
Data: 2026-07-08 | Autor: Claude | Status: AUDYT + lista oczekiwanych dostaw

## 1. Co mamy (zamknięte)

```yaml
wiedza_potwierdzona:
  cel: "meshy → MDL + 2DA + HAK, standalone (D6, D7)"
  format_mdl: "ASCII kontrakt + pełny layout binary (parser-ready)"
  animacje: "system, lista klipów c_Horror, minimum MVP (6 klipów)"
  "2da": "kontrakt kolumn aurora-web + minimalny wiersz; polityka: pełny plik w haku"
  hak_erf: "format V1.0, typy zasobów, limit resref 16"
  konwersja: "osie (x,z,y), skala przez bbox, budżet 1000-1500 tri, flip V, TGA type2, wagi max4 per segment referencji"
  weryfikacja: "runbook Toolset/gra + file-gates; ścieżki NWN EE potwierdzone"
  proces: "reguły dokumentacji, rejestr decyzji, struktura sample-2d/sample-3d"
```

## 2. Luki informacyjne (NOWE pytania → engine-mdl-pytania-cloud.md)

```yaml
luki_krytyczne:
  L1_ascii_mdl_w_engine:
    pytanie: "po korekcie D9 nie badamy ASCII jako runtime shortcut; potrzebny jest kontrakt binary MDL writer"
    wplyw: "decyduje, czy M1 kończy się na emiterze ASCII, czy trzeba też binary writer (duża różnica zakresu)"
  L2_mdx:
    pytanie: "runbook M1 wymienia w haku zasób MDX (2003) obok MDL — ale binary c_kocrachn ma MDX embedded (p_start_mdx). Kiedy MDX jest osobnym zasobem, a kiedy embedded? Co przy ASCII?"
    wplyw: "zawartość haka M3"
  L3_bind_pose_screenshot:
    pytanie: "czy podgląd nwnexplorer pokazuje bind pose, czy klatkę animacji (np. cpause1)?"
    wplyw: "jakość referencji dla sample-2d"
luki_niekrytyczne:
  - "soundset/appearancesndset dla creature (dźwięki) — odłożone po MVP"
  - "generacja UTC programowo — MVP robi blueprint ręcznie w Toolset"
  - "headless proof (B3) — manualny na MVP"
```

## 3. Oczekiwane dostawy (kto → co)

```yaml
od_mateusza:
  pilne:
    - "screenshoty c_kocrachn → sample-2d/_reference/c_kocrachn/ (odblokowuje prompty Codexa i całą linię sample)"
    - "P4: akceptacja kierunku M1-M5"
  nastepne:
    - "obrazy 2D z OpenAI → sample-2d/koc01/ (po promptach Codexa)"
    - "GLB z meshy → sample-3d/m2a_koc01/ (po akceptacji obrazów)"
  pozniej:
    - "P5: klucz API meshy + budżet (automatyzacja od M2)"
    - "manualne proofy w Toolset/grze (M4)"
od_codexa:
  pilne:
    - "odpowiedź na engine-mdl-pytania-cloud.md (L1-L3 — L1 blokuje zakres M1)"
    - "retail appearance.2da z NWN EE (KEY/BIF lub nwnexplorer export) — potrzebny do pełnego 2DA w haku (M3)"
  nastepne:
    - "prompty sample-2d → sample-2d-prompty-codex.md (po screenshotach referencji)"
    - "opcjonalnie: naprawa NwnMdlComp (B1) — odzyskalibyśmy niezależny oracle MDL"
od_claude:
  po_P4:
    - "init repo: .gitignore (dist/, *.tmp; sample GLB zostają), pierwszy commit dokumentacji"
    - "M1: package.json + czytnik HAK/ERF + parser binary MDL + testy syntetyczne + integracja cep3_core1 przez env"
    - "emiter ASCII MDL + pierwszy dump c_kocrachn do ręcznego potwierdzenia"
  rownolegle:
    - "weryfikacja checklist sample-2d i sample-3d, gdy pliki się pojawią"
```

## 4. Ryzyka procesowe (audyt)

```yaml
ryzyka:
  R1_repo_bez_commitow:
    stan: "git init bez żadnego commita; 42 pliki dokumentacji niezwersjonowane"
    mitygacja: "pierwszy commit jak najszybciej (dokumentacja to teraz największa wartość projektu)"
  R2_meshy_expiry:
    stan: "assety meshy wygasają ~3 dni od generacji"
    mitygacja: "pobieranie GLB natychmiast, zapis w sample-3d (wpisane w instrukcje)"
  R3_dwa_workspacy_codexa:
    stan: "wcześniej Codex pisał do Documents/meshy2aurora; ryzyko nawrotu"
    mitygacja: "Codex potwierdził czyszczenie; każda dostawa weryfikowana po ścieżce"
  R4_poza_meshy:
    stan: "dopasowanie pozy generowanego modelu do bind pose to najsłabsze ogniwo ścieżki A"
    mitygacja: "kontrakt obrazów 2D + checklist akceptacji + raport odchyleń s5 w konwerterze"
  R5_zakres_m1:
    stan: "jeśli L1 = 'engine wymaga binary', M1 rośnie o binary writer"
    mitygacja: "odpowiedź na L1 przed commitem do zakresu M1"
```

## 5. Kolejność krytyczna (ścieżka do pierwszego potwora w grze)

```text
[Mateusz] screenshoty referencji ──► [Codex] prompty 2D ──► [Mateusz] obrazy OpenAI ──► [Mateusz] meshy GLB ─┐
[Codex] L1-L3 + retail appearance.2da ──► [Claude] M1 parser/emiter ──► M2 konwersja ──► M3 2DA+HAK ─────────┴─► M4 proof Toolset/gra
```

Dwie linie są niezależne — sample i kod mogą iść równolegle; spinają się w M2.
