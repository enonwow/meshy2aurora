# Czyszczenie workspace — 7 września 2026

Na polecenie właściciela usunięto odtwarzalne buildy i cache z kanonicznego
repozytorium. Rozmiar plików spadł z **134,39 GB do 42,92 GB**, czyli
z **125,156 GiB do 39,976 GiB**. Odzyskano około **91,46 GB / 85,18 GiB**.

Pomiar obejmuje główne repozytorium i worktree pod jego `.worktrees`.
Sumuje długości plików; nie jest pomiarem fizycznej alokacji bloków NTFS.
Stan końcowy zmierzono przed dopisaniem tego raportu i jego evidence.

| Usunięty katalog | Bajty | Pliki |
| --- | ---: | ---: |
| `target` | 57 082 535 643 | 49 184 |
| `.worktrees/creature-supermodel-remediation/target` | 34 380 317 765 | 25 859 |
| `.tmp` — cache kompilacji Node | 1 058 164 | 397 |
| `tools/__pycache__` | 42 731 | 4 |

Dwa buildy usunięto przez `cargo clean` z jawnymi, absolutnymi ścieżkami
manifestów i katalogów docelowych. Cache usunięto przez `git clean -fdX`
ograniczone do dwóch wskazanych ścieżek, po sprawdzeniu identycznego dry-run.
Nie wykonywano globalnego `git clean`.

Przed usuwaniem potwierdzono kanoniczny workspace, dokładne ścieżki,
brak reparse points i śledzonych plików w targetach, brak pracujących
kompilatorów oraz procesów wykonujących pliki z tych targetów. Po operacji
oba targety nie istnieją, a porównanie pełnego `git status --porcelain`
potwierdziło niezmieniony stan wszystkich siedmiu zarejestrowanych worktree,
w tym ich lokalnych zmian i nieśledzonych plików źródłowych.

Zachowano modele `sample-3d`, immutable `proof-output`, wygenerowane
deliverables i diagnostykę w `artifacts`, dokumentację, zasoby referencyjne,
historię Git oraz same worktree. Istotne składniki pozostałego rozmiaru to
główny `proof-output` (20,09 GB), `artifacts` (8,20 GB), zawartość worktree
po usunięciu builda (10,13 GB) i `.git` (2,07 GB). Nie zaklasyfikowano
unikalnych modeli, plików Blender, zrzutów i wyników diagnostyki jako cache.

Przyczyną wzrostu są ponownie targety Cargo — ten sam mechanizm opisano
w [audycie z 2 sierpnia](audyt-wzrostu-rozmiaru-workspace-2026-08-02.md).
Nie zmieniano profili kompilacji. Pełny build może ponownie odtworzyć duże
targety; nie uruchamiano go po czyszczeniu, ponieważ odwróciłby jego efekt.

Dokładne rozmiary, polecenia i wyniki kontroli:
[workspace-cleanup-2026-09-07.json](evidence/workspace-cleanup-2026-09-07.json).
