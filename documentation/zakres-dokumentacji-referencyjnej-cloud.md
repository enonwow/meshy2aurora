# zakres-dokumentacji-referencyjnej-cloud.md
Data: 2026-07-08 | Status: ZAMKNIETE 2026-07-09 - dokumenty 1-6 istnieja; `aurora-web-architektura-codex.md` jest reference-only po D7-D8 | Priorytet: HISTORYCZNE

Zamówienie na dokumentację referencyjną potrzebną do wyboru kierunku implementacji konwertera (cel: `cel-projektu-cloud.md`). To nie są pytania Q — to prośba o dokumenty przeglądowe autorstwa Codexa (`[temat]-codex.md`), pisane wg zasady Aurora First (kotwice do dekompilacji/źródeł przy każdym rozdziale).

## Dokumenty zamawiane u Codexa

### 1. `aurora-mdl-format-codex.md`
Pełna referencja formatu MDL z dekompilacji: nagłówek modelu, wszystkie typy nodów (dummy, trimesh, skin, danglymesh, emitter, light, reference, aabb), kontrolery i ich klucze, geometria (verts/faces/tverts), struktura animacji (newanim, animroot, eventy, klucze pozycji/orientacji/skali), różnice ASCII vs binary, classification. Priorytet: to, co potrzebne do EMISJI poprawnego creature MDL.

### 2. `aurora-animation-system-codex.md`
Jak engine odtwarza animacje: wybór klipu po nazwie, supermodel chain, animationscale, eventy (dźwięki/hity/kroki), blending/transtime, pętle vs one-shot, wymagane minimum dla funkcjonalnego creature w grze.

### 3. `aurora-2da-creature-codex.md`
appearance.2da kolumna po kolumnie (z semantyką z dekompilacji) + powiązane 2DA, jeśli dotyczą nowego creature (np. dźwięki — appearancesndset?). Wiersz wzorcowy potwora + minimalny poprawny wiersz dla nowego direct creature.

### 4. `aurora-hak-erf-codex.md`
Format binarny ERF/HAK (nagłówek, key list, resource list, typy zasobów i ich ID), ranking nadpisań zasobów w engine (hak > override > moduł?), istniejący kod zapisu/odczytu ERF w naszym ekosystemie (aurora-web, nwn-*, nwn-lib-d) z ścieżkami.

### 5. `aurora-web-architektura-codex.md`
Mapa modułów aurora-web istotna dla meshy2aurora: source layer (`__aurora/sources`), derived pipeline i wersjonowanie konwertera, asset manifest, catalog (creatures), proof tooling (CDP), env (local runtime, blob mirror). Jak nowy hak/źródło wchodzi do systemu end-to-end.

### 6. `ekosystem-narzedzia-codex.md`
Inwentarz lokalnych repo i narzędzi przydatnych dla meshy2aurora: co robią `nwn`, `nwn-features`, `nwn-conversation`, `nwn-localization`, `nwn-last-city`, `modules`, `2DA & TLK Editor`, NwnMdlComp (jeśli jest), Blender 5.1 (dostępny u Mateusza). Per pozycja: ścieżka, rola, czy reużywalne.

## Po stronie Cloud (Claude) — do własnego researchu

- `meshy-api-cloud.md` — możliwości meshy.ai istotne dla pipeline: eksport GLB/FBX z rigiem, struktura szkieletu auto-rig (nazwy kości), API animacji (dostępne klipy, FPS), limity. Źródło: docs.meshy.ai (Internet — dozwolony jako uzupełnienie, brak lokalnego źródła).
- Po otrzymaniu dokumentów 1–6: `kierunek-implementacji-cloud.md` — analiza wariantów i rekomendacja kierunku dla Mateusza.

## Kryterium ukończenia

Komplet 1–6 + `meshy-api-cloud.md` pozwala podjąć decyzję kierunkową bez zgadywania (Aurora First) i napisać spec konwertera z planem TDD.
