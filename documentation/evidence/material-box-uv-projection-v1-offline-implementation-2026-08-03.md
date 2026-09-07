# Material Box UV V1 — implementacja i weryfikacja offline

Data: 2026-08-03

## Zakres

Zaimplementowano pierwszy etap naprawy problemu wykazanego w audycie statku
TLC: zastąpienie projekcji UV wykonywanej osobno dla tysięcy drobnych
komponentów spójną projekcją w przestrzeni całego materiału.

Zmiana jest ogólną funkcją pipeline'u, a nie ręczną edycją statku. Nie zmienia
źródłowego GLB, liczby trójkątów ani kolizji Placeable.

## Fakty implementacyjne

- Core udostępnia dwa nowe tryby:
  - `MATERIAL_LONG_AXIS` — jeden układ UV dla wszystkich fragmentów materiału;
  - `MATERIAL_BOX` — jeden układ materiału oraz dobór płaszczyzny XY/XZ/YZ na
    podstawie geometrycznej normalnej ściany.
- `MATERIAL_BOX` nie używa fazy zależnej od indeksu komponentu.
- Przebudowa render mesh ponownie wykorzystuje wierzchołki mające ten sam
  source index, UV i płaszczyznę projekcji.
- Wierzchołki są rozdzielane tylko na rzeczywistych szwach płaszczyzn UV.
- Tangenty są akumulowane i normalizowane po przebudowie indeksów.
- `SOURCE` nadal zachowuje źródłowe UV0.
- Publiczny Placeable pipeline V8 i WASM eksport
  `buildMeshyStaticPlaceablePackageV8` przyjmują jawny dokument projekcji.
- Worker wybiera V8 tylko wtedy, gdy użytkownik przekazał dokument UV wraz z
  kompletnym Face Mode V2 i texture authoring.
- Dokument UV jest częścią deterministycznej tożsamości Placeable i jest
  emitowany jako osobny artefakt JSON.

## Zachowanie aplikacji

W edytorze Material Separation dla Placeable każdy materiał ma opcjonalny
checkbox `Coherent UV (experimental)`.

- Domyślnie checkbox jest wyłączony.
- Aplikacja nie zgaduje, który materiał jest drewnem.
- Użytkownik wybiera dokładny materiał, np. `Wood`.
- Wybrany materiał otrzymuje `MATERIAL_BOX`, pełny zakres atlasu `V=0..1`,
  `U repeats=1` i brak losowej fazy.
- Żagle, liny, metal i pozostałe materiały zachowują źródłowe UV, dopóki nie
  zostaną jawnie zaznaczone.
- Tekst UI mówi wprost, że geometria i collision pozostają niezmienione.

## Weryfikacja

Przeszły:

- `cargo test -p m2a-core --lib`:
  `102 passed`, `3 ignored`, `0 failed`;
- `cargo test -p m2a-core --test placeable_pipeline`:
  `22 passed`, `1 ignored`, `0 failed`;
- testy UV obejmują:
  - wspólną przestrzeń dla odłączonych fragmentów;
  - dobór płaszczyzny projekcji z geometrii;
  - reuse indeksowanych wierzchołków;
  - rozdzielenie na rzeczywistym szwie projekcji;
  - zachowanie liczby trójkątów i byte-identical PWK;
- `cargo check -p m2a-wasm --all-targets` — passed;
- Clippy z trzema jawnymi wyłączeniami istniejących klas długu
  (`too_many_arguments`, `needless_range_loop`, testowe `dead_code`) — passed;
- `npm test` w Studio: `265 passed`, `0 failed`;
- `npm run typecheck` — passed;
- `npm run build` wraz z nowym buildem WASM i Vite — passed.

Pełne `cargo test -p m2a-core` zostało dodatkowo uruchomione, lecz globalna
komenda przekroczyła limit 184 sekund bez końcowego werdyktu. Wymagane dla tej
zmiany lib i Placeable integration suites następnie uruchomiono osobno i oba
zakończyły się powodzeniem.

## Status wizualny

`WNIOSEK IMPLEMENTACYJNY`: nowy tryb usuwa główną przyczynę patchworku —
lokalne normalizowanie i przesuwanie UV dla 21 060 komponentów drewna — oraz
powinien znacznie ograniczyć dotychczasowe wzmocnienie liczby wierzchołków.

`NIEZWERYFIKOWANE WIZUALNIE`: nie utworzono nowego MOD/HAK ani nowego preview
tego samego statku. Bieżący exact kandydat jest widoczny w Toolset, więc
aktualna bramka iteracji wymaga najpierw testu tej samej linii w NWN i nie
pozwala zamrozić kolejnego kandydata tylko na podstawie odrzucenia
estetycznego.

## Pozostałe ograniczenie

`MATERIAL_BOX` jest automatyczną poprawą pierwszego etapu. Nie zastępuje
docelowego podziału drewna na `hull`, `structure`, `deck` i `masts_spars`.
Jeżeli box projection nadal nie zapewni wymaganej jakości, kolejnym minimalnym
krokiem jest użycie istniejącego Face Mode V2 do utworzenia tych grup i nadanie
im osobnych skal/kierunków UV — bez dodatkowych generacji Meshy.
