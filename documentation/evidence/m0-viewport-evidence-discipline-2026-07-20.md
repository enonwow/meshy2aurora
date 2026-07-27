# M0 viewport evidence discipline — 2026-07-20

Status: `OPEN / VISUAL PROOF MISSING`

## Cel

Zapobiec pomyleniu podglądu w dialogu `Creature Properties` ze stanem
widocznym po zatwierdzeniu i umieszczeniu creature na Area.

## Fakty

- `proof-output/m0-v6-central-bootstrap-city-exterior-20260719/creature-properties-selected-appearance.png`
  pokazuje wybraną etykietę Appearance `M2A_M0_MESHY_RIGID` oraz obraz w
  małym preview otwartego dialogu `Creature Properties`.
- Ten obraz nie wiąże preview z zapisanym GIT-em, aktualnym `TScrollBox` ani
  z klatką canvasa po zamknięciu dialogu. Nie jest więc dowodem rezultatu po
  placementcie.
- Nie ma świeżego, zwalidowanego capture `TScrollBox` dla bieżącego modułu
  `m2a_m0v6.mod`, Area `m2a_m0a6`, fixture'a i pozycji
  `(10.00, 10.00, 3.20)`.
- `proof-output/m0-v6-central-bootstrap-city-exterior-20260719/camera-zoom-out/zoom-out-1784551908649-after.png`
  jest jedynie obserwacją kadru bez profilu/manifestu wymaganego do
  zaakceptowanego proofu `TScrollBox`.

## Odrzucone twierdzenie

Nie wolno twierdzić, że canvas po placementcie wyświetla krasnoluda tylko na
podstawie mini-preview dialogu. Preview przed `OK` i renderer Area są różnymi
stanami UI. Ten artefakt nie dowodzi ani fallbacku, ani poprawnego załadowania
`m2a_m0p01`.

## Reguła dowodowa

Widoczność modelu może otrzymać status `verified` lub `failed` wyłącznie po
tej kolejności:

1. native readback: dokładny moduł/hash, uporządkowany HAK, Appearance row,
   tożsamość fixture'a i zapisana pozycja;
2. zatwierdzenie zmiany dialogu oraz readback po jego zamknięciu;
3. świeży capture właściwego `TScrollBox`, powiązany z tymi samymi
   tożsamościami;
4. wizualna inspekcja capture i walidator odpowiedniego profilu.

Przed krokiem 3 stan jest `missing`. Nie wolno oznaczać go jako `visible`,
`fallback` ani `failed` na podstawie wyglądu mini-preview.

## Read-only checkpoint po zapisie

Wykonano bez zmiany Toolsetu, HAK-a, Appearance ani pozycji.

| Element | Odczyt | Status |
| --- | --- | --- |
| MOD | `m2a_m0v6.mod`, deklarowany saved SHA-256 `029d1d113f537e8d210e891ecc8d254a2bd4dba3a90c43b8462f332b06722019` | `verified` z manifestu saved; plik pozostaje zablokowany przez Toolset do niezależnego hash readbacku |
| Area GIT | jedna instancja `nw_dwarfmerc001`, `Appearance_Type=848`, `X=10.0`, `Y=10.0`, `Z=3.200000047683716` | `verified` przez własny `inspect_module` / parser ERF+GFF |
| Kolejność HAK | `[m2a_m0_v6]` | `verified` przez saved manifest i uprzedni native HAK-list readback |
| HAK | `m2a_m0_v6.hak`, SHA-256 `61a07a903c43c6af32c5180afb4c7678ff6eae09f1d87a296522081f63969ef4` | `verified` przez read-only hash + parser ERF |
| Row 848 | `M2A_M0_MESHY_RIGID ... m2a_m0p01 ... R S ...` | `verified` w zasobie `appearance` type `2017` |
| Model | `m2a_m0p01`, type `2002`, 149352 B, SHA-256 `26001178e283745364a897ef8b0eb8ac91f9b9a574c477f467c2a0725b3014a6` | `verified` w tym samym HAK |
| Texture | `m2a_m0t01`, type `3`, 12582956 B, SHA-256 `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` | `verified` w tym samym HAK |

Checkpoint zamyka tylko łańcuch tożsamości `GIT → Appearance 848 → HAK →
MDL/TGA`. Nie jest dowodem renderowania.

## Próba świeżego capture `TScrollBox`

Data: 2026-07-20.

Uruchomiono dokładny hook z proof routing:

```text
aurora-toolset-viewport-proof.mjs capture
  --module m2a_m0v6.mod
  --area m2a_m0a6
```

Weryfikacja kontekstu przeszła do kroku obrazu: jeden responsywny Toolset PID
`36888`, dokładny frame modułu i Area viewer dla `m2a_m0a6`. Hook nie zapisał
PNG, ponieważ jego niezależna kontrola `WindowFromPoint` wykazała, że wszystkie
pięć punktów próbnych viewportu należy do `Chrome_RenderWidgetHostHWND`, PID
`44508`, a nie do Toolsetu. Błąd jest precyzyjnie:

```text
toolset_viewport_obscured_by_foreground_window
```

To jest `missing`, nie wynik modelu. Nie wykonano retry, zmiany pozycji,
Appearance, HAK-a, MDL, ustawień Toolsetu ani operacji z-order. Warunkiem
wznowienia tej samej obserwacji jest odsłonięty `TScrollBox`; dopiero wtedy
można odczytać, czy obiekt jest na canvasie i jaki model renderer faktycznie
pokazuje.

## Wymóg przekazania proofu użytkownikowi

Po każdym zaakceptowanym, świeżym capture `TScrollBox` operator musi:

1. obejrzeć dokładnie zapisany plik PNG;
2. pokazać ten sam plik w bieżącej rozmowie — nie wystarcza ścieżka do pliku ani sam manifest;
3. podać tożsamość modułu, HAK-a, Area i obiektu oraz jednoznaczny werdykt: docelowy M0 / brak obiektu / niewłaściwy model;
4. dopiero po pokazaniu obrazu w rozmowie oznaczyć proof wizualny Toolsetu jako `verified` albo `failed`.

Zapisany PNG/JSON, którego użytkownik nie może zobaczyć w rozmowie, pozostaje `missing` dla przekazania wyniku użytkownikowi.

## Problemy i błędy

```yaml
current_problems:
  - id: M0-VIEWPORT-PROOF-MISSING
    status: OPEN
    repro: "Brak świeżego capture TScrollBox dla m2a_m0v6.mod / m2a_m0a6."
    expected: "Niezależna obserwacja canvasa po zatwierdzeniu i saved readbacku."
    actual: "Dostępne są tylko preview dialogu oraz nieprofilowana obserwacja kadru."
    next_action: "Read-only checkpoint, potem istniejąca pełna transakcja AUR-S02-03 z baseline, native readbackiem i rollbackiem przy niepowodzeniu."
bugs:
  - id: M0-EVIDENCE-STATE-CONFLATION
    severity: P1
    status: FIXED
    repro: "Wnioskowanie o rendererze Area z preview dialogu sprzed OK."
    expected: "Twierdzenie o canvasie wyłącznie z capture TScrollBox."
    actual: "Preview zostało błędnie opisane jako możliwy stan canvasa."
    next_action: "Stosować regułę dowodową z tego wpisu."
```

## Następny krok

Nie zmieniać teraz Toolsetu, MDL, HAK-a ani shared tooling. Najpierw wykonać
wyłącznie read-only checkpoint `saved module / ordered HAK / row 848 /
resource mapping`, a następnie użyć istniejącej transakcji AUR-S02-03.

## Aktualizacja: zaakceptowany capture TScrollBox — 2026-07-20

Historyczny wpis `M0-VIEWPORT-PROOF-MISSING` opisuje stan sprzed odsłonięcia
viewportu i sprzed korekty kamery. Pozostaje historią błędu, ale nie jest już
bieżącym werdyktem.

| Dowód | Identyczność | Wynik |
| --- | --- | --- |
| `proof-output/m0-v6-central-bootstrap-city-exterior-20260719/viewport-after-7-50-7-45-20260720/tscrollbox.png` | świeży `TScrollBox`, po saved GIT `X=7.5`, `Y=7.449999809265137`, `Z=3.200000047683716` | `failed` wyłącznie dla widoczności w tym kadrze; nie identyfikuje modelu jako krasnoluda |
| `proof-output/m0-v6-central-bootstrap-city-exterior-20260719/viewport-m0-visible-after-pitch-up-20260720/tscrollbox.png` | niezależny, fizyczno-pikselowy capture właściwego `TScrollBox`; moduł `m2a_m0v6.mod`, Area `m2a_m0a6`, HAK `[m2a_m0_v6]`, `Appearance_Type=848` | `verified`: widoczny jest docelowy model M0, nie `Dwarf Mercenary` |

Drugi PNG został obejrzany przez operatora i wyświetlony w rozmowie z
właścicielem. Oznacza to spełnienie reguły przekazania proofu użytkownikowi
dla **Toolsetu**. Nie rozszerza werdyktu na NWN: dla runtime nadal brakuje
niemutowalnego profilu AUR-S07 ze zweryfikowaną geometrią i entry surface.
