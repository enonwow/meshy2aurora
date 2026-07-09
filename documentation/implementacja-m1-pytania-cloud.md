# implementacja-m1-pytania-cloud.md

Status 2026-07-09: ZAMKNIETE HISTORYCZNE. Q2/Q5 o `aurora-web` CLI/CDP zostaja jako kontekst sprzed D7; aktywna implementacja nie uzywa `aurora-web` jako CLI/proof.
Data: 2026-07-08 | Status: ZAMKNIĘTE (odpowiedź: implementacja-m1-odpowiedz-codex.md, Q1–Q7 POTWIERDZONE) | Priorytet: BLOKUJĄCE dla M1–M4
Kontekst: pytania praktyczne przed napisaniem pierwszego kodu i testów (TDD).

## Q1: Postać źródłowego c_kocrachn.mdl
Czy `__aurora/sources/hak/cep3_core1/c_kocrachn.mdl` w mirrorze to ASCII czy binary MDL? Jeśli binary: wskaż creature z ASCII source do golden testu, albo potwierdź, że golden test M1 ma porównywać semantycznie GLB z dwóch ścieżek (binary source → v13 vs nasz ASCII emit → v13).

## Q2: Uruchamianie konwertera v13 z meshy2aurora
Dokładna komenda odpalenia `aurora-mdl-to-glb` (interpreter: node/tsx/ts-node? wersja node? cwd? zależności aurora-web backend?). Czy meshy2aurora może importować `aurora-mdl-ascii-to-glb.converter.ts` jako moduł (ścieżka względna do C:\Projects\aurora-web), czy tylko przez CLI? Rekomendacja + kotwica (package.json aurora-web).

## Q3: Semantyka appearance.2da z haka
Gdy hak zawiera `appearance.2da`, czy zastępuje CAŁY plik (wtedy musimy spakować pełny retail appearance.2da + nasz wiersz 9000), czy aurora-web/engine merguje wiersze z warstw? Kotwica z runtime-settings/catalog. To decyduje o M3 i potwierdza pilność wyciągnięcia retail 2da z NWN EE.

## Q4: Rozwiązywanie supermodelu cross-source
`m2a_koc01.mdl` (hak) ma `setsupermodel ... c_Horror` (vanilla). Jak pipeline derived + loader znajdują c_horror: czy c_horror.glb musi istnieć w derived, czy chain działa cross-source hak→vanilla automatycznie przy sync? Kotwica + czy trzeba coś dodać do haka.

## Q5: Runbook proofu dla nowego resref
Krok po kroku komendy od zera do proofu CDP dla m2a_koc01: uruchomienie stacka (docker vs AURORA_HOST_LOCAL_RUNTIME — co teraz działa), ustawienie runtime settings (includeHaks, hakPriorityList, hakDirectoryPath), sync, jak wskazać nowego creature w Creatures Mode (templateId? syntetyczny template?), wywołanie capture-creatures-mode-cdp.mjs z parametrami. Format: blok komend do wklejenia.

## Q6: Konwencje repo meshy2aurora
Wersja node i package manager zgodne z aurora-web (npm? pnpm?), bazowy tsconfig/eslint do skopiowania, framework testowy używany w aurora-web (jest? vitest?) — żeby meshy2aurora było spójne. Czy dopuszczalna zależność od aurora-web przez ścieżkę względną w package.json?

## Q7: Wartości do skopiowania z referencji
Odczytaj wprost ze źródła c_kocrachn.mdl i podaj: dokładną linię `setsupermodel`, `setanimationscale` (0.72?), `classification`, oraz nazwę `bitmap/texture0` skin nodes. Do golden testu M1 (emisja musi odtworzyć te wartości 1:1).
