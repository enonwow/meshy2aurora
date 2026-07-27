# M0 binary vertical slice — stan i blokada standardu (2026-07-19)

## Cel

Zweryfikować wygenerowany przez Meshy2Aurora model w nowym, projektowym
module NWN: najpierw w Aurora Toolset, potem w runtime NWN.

## Fakty

- Historyczny profil `m0-static-rigid-v5` jest odrzucony: Aurora pokazała
  czarny obszar, a natywny Save odrzucił dwie creature poza dostępną
  powierzchnią tile'i. Nie jest on wejściem ani celem obecnego przebiegu.
- Nowy, caller-owned MOD został wygenerowany binarnie przez własny writer
  Meshy2Aurora, bez sterowania Aurora Toolsetem:
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_bm0p1.mod`.
- Związany SHA-256 MOD-a:
  `215df808dc5617f48c7d10e1f07176b4264f3c615f0bb001f72ef55ea7992d76`.
- Moduł deklaruje Area `m2a_bm0a1` 2×2, entry `[5, 5, 0]`, jeden HAK
  `m2a_m0_proof` (SHA-256
  `7211c1a016c2b36320f7ce2a399d2833b8f2d7901d2896261d01a93161600200`)
  oraz jedną fixture `nw_dwarfmerc001`, Appearance_Type `15109`,
  pozycja `[10, 10, 0]`.
- Profil centralnego bootstrapu znajduje się w
  `proof-output/vertical-slice-m0-binary-20260719/binary-module-bootstrap-profile.json`.

## Wynik centralnej bramki offline

`validate-aurora-toolset-binary-module-bootstrap.mjs --mode dry-run` zwrócił
`binary_bootstrap_profile_schema_valid`.

`--mode preflight` nie ocenił jeszcze serializacji modułu: centralny validator
wywołuje helper Area-readback pod nieistniejącą ścieżką zależną od CWD
`C:\Projects\meshy2aurora\backend\scripts\read-aurora-toolset-bootstrap-area-readback.mjs`.
Prawidłowy helper należy do centralnego standardu w
`C:\Projects\aurora-web\backend\scripts`. To defekt centralnego resolvera,
zgłoszony do tasku standardu `019f6203-f618-7b82-8800-6c6a3e8c044f`; projekt
nie wprowadza lokalnego adaptera ani obejścia.

Po zgłoszeniu centralny standard zakotwiczył również helpery przechodnie względem
`import.meta.url` i dodał regresję pełnego preflightu uruchamianego z katalogu
konsumenta. Niezależny ponowny przebieg zwrócił
`binary_bootstrap_structural_readback_valid`: hash MOD-a, Area 2x2, entry,
fixture i kolejność HAK są zgodne z profilem. Nie uruchomiono Toolsetu, NWN ani
globalnego inputu.

## Status proofu

| Punkt | Status |
| --- | --- |
| Struktura profilu binarnego | `verified` (dry-run) |
| Centralny binary preflight | `verified` |
| Natywny Save/geometria Aurora | `missing` |
| TScrollBox visual proof Aurora | `missing` |
| AUR-S07 / proof runtime NWN | `missing` |

## Warunek wznowienia

Po przejściu `preflight` potrzebna jest wyłącznie centralna, nazwana trasa dla
nowego profilu binarnego: native Save/readback geometrii, następnie TScrollBox
i AUR-S07. Została zgłoszona jako kolejna luka standardu. Stary PID/Build Module
nie jest dotykany przez ten przebieg.

## Bieżąca bramka live

Centralny native-geometry dry-run dla tego profilu przeszedł jako
`binary_native_geometry_plan_valid`. Jego read-only `preflight` prawidłowo
zwrócił `STALE_TOOLSET_SESSION_REJECTED` dla responsywnego PID `13584` i nie
uruchomił ani nie zamknął żadnego procesu. Trasa wymaga zera procesów Toolsetu,
zanim utworzy jedną świeżą sesję dla `m2a_bm0p1.mod`.

Proces nie jest zamykany automatycznie: potrzebne jest wyraźne upoważnienie
właściciela do zamknięcia wyłącznie tego starego PID-u albo ręczne zamknięcie
go przez właściciela. Dopiero po czystym preflightcie wolno wykonać centralne
`open` i `geometry-observe`.
