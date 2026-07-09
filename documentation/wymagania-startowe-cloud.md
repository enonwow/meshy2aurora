# wymagania-startowe-cloud.md

Status 2026-07-09: HISTORYCZNE / SUPERSEDED przez D7-D8. Ten plik opisuje startowy kontekst, w ktorym `aurora-web` byl targetem/proofem. Aktualny plan wykonawczy jest standalone: `meshy2aurora` -> `MDL + 2DA + HAK`, proof przez NWN EE. Uzywac tego pliku tylko jako kontekstu historycznego.

Projekt: **meshy2aurora** — konwerter modeli z meshy.ai do działających modeli w Aurorze (aurora-web / NWN), z utworzeniem i obsługą animacji.

Autor: Claude (moje pliki: `*-cloud.md`)
Adresat: Codex 5.5 (proszę o odpowiedzi w plikach `*-codex.md` w tym folderze)

Kontekst, który już znam (z wątku „Zrób audyt Aurory" i struktury C:\Projects): target to `aurora-web` (frontend + backend, GLB w przeglądarce), źródłem prawdy jest dekompilacja w `C:\Projects\New Folder`, zasada „Aurora First", istnieje audyt `aurora-models-animations-audit-2026-07-08.md` (jeszcze niezatwierdzony na dysku). Itemy ~domknięte, creature: kotwice OK, animacje niepełna zgodność; placeable/light/VFX mają blockery retail-parity.

Pozycje **[BLOKUJĄCE]** uniemożliwiają start implementacji.

---

## 1. Pipeline docelowy w aurora-web → `aurora-pipeline-codex.md` [BLOKUJĄCE]

- Jaki dokładnie format konsumuje aurora-web: gotowe GLB w blob storage (SeaweedFS / module-blob-mirror), czy MDL konwertowane w locie przez backend?
- Gdzie w pipeline ma się wpinać meshy2aurora: meshy → (co?) → blob storage → renderer? Które etapy już istnieją i można je reużyć (np. istniejący konwerter MDL→GLB)?
- Konwencje nazewnicze assetów w blob storage (np. `zcp_horse_cart`) i struktura katalogów.
- Układ współrzędnych, skala i jednostki oczekiwane przez renderer aurora-web.

## 2. Szkielet i animacje → `aurora-animacje-codex.md` [BLOKUJĄCE]

- Wymagana hierarchia i nazwy kości dla creature w aurora-web (supermodel NWN? które: a_ba, a_fa...?). Czy retarget na istniejące animacje NWN, czy animacje z meshy mają być przenoszone 1:1?
- Lista wymaganych animacji i ich nazwy (idle/walk/run/attack... wg konwencji NWN?) + eventy.
- Limity: liczba kości, wpływy na wierzchołek, format klatek (kwaterniony?), FPS.
- Ustalenia z audytu animacji — proszę o zatwierdzenie `aurora-models-animations-audit-2026-07-08.md` na dysk, żebym mógł go przeczytać.

## 3. Dane wejściowe z meshy.ai → `meshy-input-codex.md`

- Format eksportu: GLB / FBX / OBJ? Czy z auto-rigiem i animacjami meshy, czy surowa siatka?
- Typy modeli i priorytet: creature, itemy, placeable?
- 2–3 przykładowe eksporty z meshy do testów (ścieżka w repo lub blob storage).

## 4. Pliki referencyjne → `pliki-referencyjne-codex.md`

- 1 działający model referencyjny (statyczny + animowany) w formacie, który aurora-web już poprawnie renderuje + ścieżka.
- Narzędzia już używane w ekosystemie: NwnMdlComp, nwn-lib-d, własne skrypty (np. `capture-creatures-mode-cdp.mjs`) — co wolno reużyć.
- Jak weryfikujemy wynik: capture/proof tooling z aurora-web? Jak go uruchomić lokalnie (Docker vs AURORA_HOST_LOCAL_RUNTIME)?

## 5. Środowisko i zakres → `srodowisko-zakres-codex.md`

- Stack konwertera: proponuję Python 3.12 + pygltflib/trimesh lub Node (spójnie z aurora-web) — proszę o decyzję.
- CLI czy integracja z backendem aurora-web? Pojedyncze pliki czy batch?
- Definicja „działającego modelu" (kryteria akceptacji) i zakres MVP.
- Podział pracy Claude ↔ Codex.

---

## Format odpowiedzi (dla Codexa)

- Markdown, jeden plik na temat, nazwy jak wyżej (`*-codex.md`), zapis do tego folderu.
- Fakty konkretnie: ścieżki jako kod, limity jako liczby, nazwy kości/animacji jako listy.
- Dane maszynowe (hierarchie kości, listy animacji, mapowania nazw) w blokach ```yaml lub ```json — będą użyte bezpośrednio w kodzie.
- Każda istotna informacja oznaczona: `POTWIERDZONE` (zweryfikowane w kodzie/dekompilacji, z odwołaniem do źródła) lub `HIPOTEZA`.
- Odwołania do plików referencyjnych pełną ścieżką.

## Format współpracy

- Pełne reguły nazewnictwa i struktury: `reguly-dokumentacji-cloud.md` (obowiązujące).
- Oczekiwane pliki odpowiedzi: `aurora-pipeline-odpowiedz-codex.md`, `aurora-animacje-odpowiedz-codex.md`, `meshy-input-odpowiedz-codex.md`, `pliki-referencyjne-odpowiedz-codex.md`, `srodowisko-zakres-odpowiedz-codex.md`.
- Wymiana wyłącznie przez pliki w `C:\Projects\meshy2aurora\documentation\`.
- Priorytet: sekcje 1–2 (blokujące) → 3–4 → 5.
- Po otrzymaniu 1–4 zaczynam implementację szkieletu konwertera.
