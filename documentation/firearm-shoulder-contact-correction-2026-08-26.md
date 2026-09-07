# Korekta chwytu strzelby: bark, kolba i cheek-weld — 2026-08-26

## Problem

Podgląd broni potrafił raportować `stockShoulderError=0`, mimo że kolba była
wizualnie osadzona przy środku klatki. Walidacja była kołowa: solver i readback
korzystały z tego samego syntetycznego punktu barku wyliczonego względem głowy.
W dokładnym `pfh0` ten punkt miał pozycję boczną około `x=0.061`, podczas gdy
rzeczywisty prawy staw barkowy `rbicep_g` znajduje się około `x=0.138`.

Drugim błędem był socket kolby wyznaczony jako procent AABB. Wybierał dolną
ćwiartkę kolby zamiast fizycznej tylnej powierzchni stopki. Punkt oka był z kolei
traktowany jak pivot `head_g`, bez małego przesunięcia do dominant eye/cheek-weld.

## Zaimplementowana korekta

- punkt shoulder pocket jest kotwiczony do rzeczywistego `rbicep_g`, z małym
  proporcjonalnym przesunięciem do przedniej powierzchni miękkiej kieszeni
  barkowej; `rbicep_g` jest środkiem stawu, a nie powierzchnią skóry;
- socket kolby jest centroidem wierzchołków rzeczywistej tylnej powierzchni
  stopki, a nie procentem AABB;
- punkt oka jest wyprowadzany proporcjonalnie z `head_g`, rozstawu barków i
  niewielkiego opuszczenia do cheek-weldu;
- solver przeszukuje ograniczoną pozycję broni przy nieruchomej kolbie, a
  `low-ready` obraca lufę w dół wokół barku;
- podgląd pokazuje niezależnie `kolba→pocket`, odległość do prawdziwego stawu
  barkowego, `oko→linia`, położenie oka wzdłuż linii i kąty broni/łokci;
- zamrożona geometria i transformacje modelu V12 nie zostały zmienione.

## Wynik aplikacyjny

Dla dokładnego `pfh0` i modelu V12, w `firearm_ready_custom`:

- `kolba→pocket = 0.0000`;
- `kolba→staw barku = 0.0358 m`;
- stopka znajduje się przed środkiem stawu, zamiast około 2,5 cm za nim;
- `oko→linia = 0.0229 m`;
- oko pozostaje za przyrządami: `-0.4190 m` wzdłuż osi;
- broń: pitch/yaw/roll `-7.0 / -3.0 / -5.0°`;
- łokcie: `56.4 / 146.6°`.

W `firearm_idle_custom` kolba pozostaje osadzona, a pitch broni wynosi `-45°`.
W `firearm_shot_custom` kolba także pozostaje osadzona podczas odrzutu.

## Weryfikacja

- 23 testy celowane w sześciu plikach: zaliczone;
- TypeScript `tsc -b --pretty false`: zaliczony;
- bieżący podgląd aplikacji: `CURRENT PIPELINE OVERLAY`;
- test wzrokowy Toolset/NWN pozostaje własnością właściciela i nie jest objęty
  tym wynikiem aplikacyjnym.

Implementacja znajduje się w zarejestrowanym worktree
`C:\Projects\meshy2aurora\.worktrees\items-agent-remediation-final` na gałęzi
`codex/items-agent-remediation-final`.
