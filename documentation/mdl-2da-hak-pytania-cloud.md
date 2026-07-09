# mdl-2da-hak-pytania-cloud.md
Data: 2026-07-08 | Status: ZAMKNIETE PRZEZ `mdl-2da-hak-odpowiedz-codex.md`; DALSZE BLOCKERY PRZENIESIONE DO `engine-mdl-pytania-cloud.md` | Priorytet: HISTORYCZNE
Aktualizacja 2026-07-09: Q6 o ingest do `aurora-web` ma status reference-only po D7-D8. Aktywna implementacja `meshy2aurora` nie uzywa `aurora-web` jako runtime/test base.
Kontekst: `cel-projektu-cloud.md` — emisja natywnego contentu Aurory (MDL + 2DA + HAK) z modeli meshy (mesh + rig + animacje).

## Q1: Kontrakt ASCII MDL dla creature z własnymi animacjami
Jaka jest minimalna poprawna struktura ASCII MDL dla creature direct model z własnym szkieletem i własnymi animacjami (bez supermodelu lub z `setsupermodel NULL`)? Wypisz z dekompilacji: wymagane nagłówki (newmodel/setsupermodel/setanimationscale/classification), typy nodów (dummy/trimesh/skin/danglymesh?), strukturę bloku `newanim` (nazwy, length, transtime, animroot, eventy, keye pozycji/rotacji). Format ```yaml + kotwice w decompiled_all.c.

## Q2: Skin w ASCII MDL
Jak dokładnie wygląda blok skin/weights w ASCII MDL (składnia `weights`, limit wpływów, wymagania bind pose)? Czy parser aurora-web (aurora-mdl-ascii-to-glb.converter.ts) obsługuje pełny skin z ASCII — czy coś jest tylko w binary path?

## Q3: Nazwy animacji dla direct creature
Pełna lista nazw animacji, których engine oczekuje dla direct creature (prefiks c*? źródło: dekompilacja/2DA), z minimalnym zestawem wymaganym, żeby creature był funkcjonalny w grze (idle/walk/run/attack/damage/death). Które eventy w animacjach są wymagane (np. hit/footstep/snd_)?

## Q4: appearance.2da
Które kolumny appearance.2da są obowiązkowe dla nowego direct creature (RACE=model resref, MODELTYPE=S, MOVERATE, PERSPACE, HEIGHT, TARGETABLE...)? Podaj kontrakt kolumn z wartościami wzorcowymi na przykładzie istniejącego wiersza potwora + kotwice (2da w dekompilacji / narzędzie 2DA & TLK Editor jest zainstalowane u Mateusza).

## Q5: Budowa HAK
Czym budujemy HAK programowo (format ERF/HAK)? Czy w ekosystemie (aurora-web / nwn-* repa / nwn-lib-d) jest już kod do zapisu ERF? Jakie zasoby musi zawierać hak dla creature: mdl, tga/dds/plt, appearance.2da (+ ranking nadpisań)?

## Q6: Ingest haka do aurora-web
Jak aurora-web wciąga nowy hak do source layer (`__aurora/sources/hak/...`)? Czy wystarczy dodać katalog do mirrora i uruchomić sync derived, czy trzeba rejestracji modułu? Kroki + kotwice.

## Q7: Tekstury
Jaki format tekstur emitujemy dla zgodności gra+aurora-web: TGA wystarczy? Czy pipeline derived obsługuje dds/plt? Ograniczenia nazw (resref 16 znaków?).
