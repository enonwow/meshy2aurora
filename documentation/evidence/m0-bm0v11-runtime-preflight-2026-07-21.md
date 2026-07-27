# M0 BM0v11 — preflight runtime po ustaleniu fixture'a (2026-07-21)

Status: `verified` dla nowego binarnego MOD i jego powiązania z HAK-em;
`missing` dla natywnej geometrii Toolsetu, viewportu i runtime NWN.

## Cel przebiegu

Ten przebieg nie zmienia modelu Meshy ani HAK-a. Materializuje świeży,
jednoznaczny moduł testowy dla istniejącego statycznego M0, aby odróżnić
problem kamery/pozycji od problemu renderera NWN.

## Ustalona tożsamość

| Element | Wartość |
| --- | --- |
| MOD | `m2a_bm0v11.mod` |
| MOD SHA-256 | `87c01dfa96ae3c99c2eef3cf564266d9a1dbbd6b465d1c2f021aa128619bbf28` |
| Area | `m2a_bm0a11`, `tdc01`, 2×2, tile `5` / orientacja `0` ×4 |
| Start | `[10, 10, 0]`, kierunek `[0, 1]` |
| Jedyny fixture | `nw_dwarfmerc001`, `Appearance_Type=848`, `[10, 14.5, 0]` |
| HAK | `m2a_m0v10.hak`, SHA-256 `04afbb1dc1e2f5cbb601e5d005f81396a044fc7b16597022b104367c5ee74966` |
| Resolution | `848 -> M2A_M0_MESHY_RIGID -> m2a_m0p01` |

Centralny `validate-aurora-toolset-binary-module-bootstrap.mjs --mode
preflight` przeszedł: potwierdza hash MOD/HAK, IFO entry, pełną listę tile'i,
ordered HAK list oraz jeden rekord GIT z dokładną pozycją fixture'a.

## Bramka live Toolsetu

Nie uruchomiono Toolsetu ani NWN. Read-only
`startup-admission-dry-run` centralnej trasy binary-native-geometry zwrócił
`binary_native_mru_target_mismatch`: bieżące MRU wskazuje historyczny
`m2a_m0v12.mod`, a nowy moduł to `m2a_bm0v11.mod`.

To nie jest zgoda na zmianę `nwtoolset.ini`, MRU ani ustawień użytkownika.
Aktualny standard wyraźnie tego zabrania, dlatego właściwy status live lane
pozostaje `missing`, a nie „naprawiony przez konfigurację”.

## Następna bramka

Wznowienie wymaga centralnej, wspieranej ścieżki otwarcia dokładnego,
nieistniejącego wcześniej MOD bez mutowania MRU, albo już otwartej i
zweryfikowanej sesji dokładnie tego MOD. Po tym kolejność jest stała:

1. native Save + geometry gate;
2. świeży `TScrollBox` z widocznym M0;
3. AUR-S07 z wiążącym profilem, logiem klienta i capturem NWN.

Nie należy zmieniać `meshType`, MDL, HAK-a, appearance ani tekstury na
podstawie poprzedniego kadru NWN — poprzedni MOD miał inną pozycję fixture'a.
