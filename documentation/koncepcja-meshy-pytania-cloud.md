# koncepcja-meshy-pytania-cloud.md

Status 2026-07-09: ZAMKNIETE HISTORYCZNE. Pytania o GLB v13, derived mirror i Creatures Mode dotycza starej sciezki `aurora-web`; po D7 sa reference-only.
Data: 2026-07-08 | Status: ZAMKNIĘTE (odpowiedź: koncepcja-meshy-odpowiedz-codex.md, wszystkie Q POTWIERDZONE) | Priorytet: BLOKUJĄCE
Kontekst: `koncepcja-meshy-cloud.md` (strategia B — surowa siatka meshy + transfer wag z modelu referencyjnego).

## Q1: Wykonalność strategii B względem GLB v13
Czy GLB emitowany przez meshy2aurora może być "wstrzyknięty" do derived mirrora obok plików z aurora-mdl-to-glb v13 i zostać poprawnie załadowany przez loader + buildAuroraSupermodelAnimationClips? Jakie extras/metadane GLB są obowiązkowe (supermodel, animationscale, animroot, inne?) — podaj pełny kontrakt extras w ```yaml na podstawie konwertera v13.

## Q2: Struktura skinningu w derived GLB
Jak konwerter v13 zapisuje skin w GLB (skins/joints/inverseBindMatrices vs transformy nodów)? Czy runtime creature wymaga SkinnedMesh, czy animuje transformy nodów (rigid per-node)? To decyduje, czy transfer wag w ogóle jest potrzebny, czy raczej segmentacja siatki per-node.

## Q3: Rejestracja nowego resref
Jak dodać nowy resref (spoza modułu) do katalogu creature w aurora-web, żeby Creatures Mode go widział i dało się zrobić proof CDP? Jeśli nie ma ścieżki — co najmniej inwazyjnie trzeba dodać?

## Q4: Bind pose referencji
Czy w `c_kocrachn.glb` bind pose (geometria w spoczynku) jest dostępna wprost (poza animacjami)? Podaj, jak ją wyciągnąć — potrzebna jako obraz referencyjny do generacji w meshy i jako baza transferu wag.
