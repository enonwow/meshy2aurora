# M0 v6 Toolset visual binding packet — 2026-07-20

Status: VERIFYING
Owner: Codex
Stage: Toolset visual proof; NWN runtime intentionally excluded

## Cel

Związać jeden świeży obraz właściwego viewportu Aurory z aktualnym, zapisanym
modułem M0 v6, jego HAK-em, Appearance, fixture'em i pozycją — bez wykonywania
żadnej kolejnej mutacji Toolsetu, kamery albo modelu.

To jest audytowy packet wiążący evidence. Nie jest centralnym profilem
`aur-s07-runtime-profile/v1` ani deklaracją ukończenia runtime.

## Aurora First / provenance

- Natywny readback GIT:
  `C:\Projects\aurora-web\backend\scripts\read-aurora-toolset-bootstrap-git-readback.mjs`.
- Właściwy obraz Area:
  `aurora-toolset-viewport-proof.mjs capture`, z walidowanym `TScrollBox`.
- Local read-only HAK/module identity; żadnego payloadu retailowego nie
  skopiowano do repozytorium.

## Związane tożsamości

| Element | Wartość | Dowód |
| --- | --- | --- |
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0v6.mod` | fresh native GIT readback |
| MOD SHA-256 | `698e469cac1467657a2fafc19c73897b7a2e67346d0cbc2a104abc038f9f3a37` | fresh native GIT readback |
| Area | `m2a_m0a6` | native GIT readback + capture `areaViewerTitle` |
| Fixture | `nw_dwarfmerc001`, tag `NW_DWARFMERC001` | native GIT readback |
| Appearance | `Appearance_Type=848` | native GIT readback; row is `M2A_M0_MESHY_RIGID` |
| Pozycja | `[7.5, 7.45, 3.2]` | native GIT readback |
| Ordered HAK | `[m2a_m0_v6]` | saved module evidence; single active HAK |
| HAK SHA-256 | `61a07a903c43c6af32c5180afb4c7678ff6eae09f1d87a296522081f63969ef4` | fresh `Get-FileHash` |
| Model resolution | row 848 → `m2a_m0p01` (type `2002`), SHA-256 `26001178e283745364a897ef8b0eb8ac91f9b9a574c477f467c2a0725b3014a6` | current-HAK identity recorded in [m0-viewport-evidence-discipline-2026-07-20.md](m0-viewport-evidence-discipline-2026-07-20.md) |
| Texture resolution | `m2a_m0t01` (type `3`), SHA-256 `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` | current-HAK identity recorded in [m0-viewport-evidence-discipline-2026-07-20.md](m0-viewport-evidence-discipline-2026-07-20.md) |

## Viewport artifact

| Element | Wartość |
| --- | --- |
| PNG | `proof-output/m0-v6-central-bootstrap-city-exterior-20260719/viewport-m0-visible-after-pitch-up-20260720/tscrollbox.png` |
| PNG SHA-256 | `0d503b5efa1cad499ca697b059cef7b6ec0baa7c6553c178f1799b3a5483e222` |
| Capture result | `proof-output/m0-v6-central-bootstrap-city-exterior-20260719/viewport-m0-visible-after-pitch-up-20260720/result.json` |
| Result SHA-256 | `8d6df432d13b9d21214e36c849a797a80e458052c2559d45ef6e4e431a9ffede` |
| Target | dokładnie jeden `TScrollBox`, `1275 × 839`, Area `m2a_m0a6` |
| Monitor | `\\.\DISPLAY1`, `primary=false` |
| Capture method | `physical-pixel capture of validated TScrollBox rectangle after WindowFromPoint ownership verification` |
| Input safety | `usesGlobalCursor=false`, `usesGlobalKeyboard=false`, `usesGlobalMouse=false` |

## Werdykt wizualny

Inspekcja powyższego PNG pokazuje duży, niestandardowy humanoidalny mesh M0 z
jasnoszarym/zielonym wyglądem. Nie jest to render domyślnego `Dwarf Mercenary`.

**Toolset visual M0: verified.**

## Granica packetu

Capture result nie zapisuje sam w sobie MOD SHA, Appearance ani HAK hash.
Powyższe wiązanie korzysta z readbacku tego samego zapisanego modułu po
capture oraz faktu, że od capture nie wykonano kolejnej mutacji. Jest to
wystarczające do audytowego stwierdzenia widoczności M0, lecz nie zastępuje
niemutowalnego centralnego proof profile i jego walidatora.

Aktualny live checkpoint pozostaje czysty: jeden responsywny `nwtoolset.exe`
`PID=36888`, tytuł `m2a_m0v6.mod` bez `*`, bez modalu. Nie ma uruchomionego
`nwmain`.

## NWN runtime

Status: **missing**.

Nie istnieje zaakceptowany profil AUR-S07 wiążący dokładnie
`m2a_m0v6.mod` / `m2a_m0a6` / aktualny MOD SHA. Istniejący bootstrap manifest
ma hash sprzed korekty pozycji, a M0 v5 jest innym, geometrycznie odrzuconym
profilem. Nie uzbrojono AUR-S07, nie uruchomiono Test Module, nie tworzono
lokalnego launchera i nie użyto przypadkowego screenshotu lub samego PID jako
dowodu.

## Następny krok

Tylko po udostępnieniu centralnego, niemutowalnego profilu AUR-S07 dla tej
dokładnej tożsamości: jego `dry-run`, a następnie `arm`; później pojedyncza,
user-owned akcja `Test Module` i centralny `observe`/`validate`. Do tego czasu
Toolset i procesy pozostają nietknięte.
