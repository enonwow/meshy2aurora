# Studio V1 - pakiet mockupow 2026-07-14

Status: `AKTYWNY KONTRAKT WIZUALNY Z OZNACZONYMI POPRAWKAMI`

Ten katalog zachowuje pierwsza pelna serie ekranow Studio V1. Obrazy sa referencja ukladu i przeplywu, ale nie wszystkie sa gotowe do implementacji 1:1. Status w tabeli ponizej ma pierwszenstwo przed wygladem pojedynczego obrazu.

## 1. Zamrozony przeplyw V1

`Source -> Inspect -> Build -> Review Output -> Download`

- `Source` wybiera lokalny GLB i bazowy `appearance.2da`.
- `Inspect` pokazuje zrodlo, statystyki, animacje i walidacje wejscia.
- `Build` pokazuje pipeline, Stage Ledger i stan wykonania bez viewportu.
- `Review Output` sprawdza Converted Model, wpis `appearance.2da` i zawartosc HAK.
- `Download` pobiera wynikowa paczke oraz pozwala pobrac pojedyncze artefakty.

## 2. Macierz ekranow

| ID | Plik | Widok | Status | Wymagana korekta |
|---|---|---|---|---|
| 01 | [01-source-empty.png](01-source-empty.png) | Source empty | `ACCEPTED_BASE` | Zachowac jako stan pusty. |
| 02 | [02-inspect-source.png](02-inspect-source.png) | Inspect | `ACCEPTED_WITH_FIXES` | Usunac `GLTF` z drop zone; V1 przyjmuje GLB i `appearance.2da`. |
| 03 | [03-build-running-rework.png](03-build-running-rework.png) | Build running | `REWORK_REQUIRED` | Usunac viewport. Pelny srodek ma zajac Pipeline Progress, Stage Ledger, hashe i aktualny etap. |
| 04 | [04-build-completed-obsolete.png](04-build-completed-obsolete.png) | Build completed | `OBSOLETE_DO_NOT_IMPLEMENT` | Nie tworzyc osobnego ekranu sukcesu. Udany Build automatycznie otwiera Review Output. |
| 05 | [05-review-model-details.png](05-review-model-details.png) | Review: Model Details | `ACCEPTED_WITH_FIXES` | Zmienic nazwy Source/Binary Readback zgodnie z sekcja 4 i utrzymac staly Reference Trace Bar. |
| 06 | [06-review-appearance-2da.png](06-review-appearance-2da.png) | Review: appearance.2da | `ACCEPTED_WITH_FIXES` | Zastapic plywajacy popup stalym Reference Trace Bar w tym samym miejscu co lineage w 05. |
| 07 | [07-review-package-contents.png](07-review-package-contents.png) | Review: Package Contents | `ACCEPTED_WITH_FIXES` | Dependency chain pokazac w tym samym stalym Reference Trace Bar; dane zasobow musza byc dynamiczne. |
| 08 | [08-download.png](08-download.png) | Download | `ACCEPTED_WITH_FIXES` | Nazwe menu zmienic na `Individual Artifacts (3)`; raport pozostaje w Export. |
| 09 | [09-build-failed-rework.png](09-build-failed-rework.png) | Build failed | `REWORK_REQUIRED` | Uzyc tego samego pelnego ukladu pipeline co Build running, bez viewportu; Diagnostics moze byc rozwiniete ponizej. |
| 10 | [10-debug-binary-needs-polish.png](10-debug-binary-needs-polish.png) | Debug: Binary | `ACCEPTED_WITH_FIXES` | Ujednolicic zakladki i przywrocic akcje workflow Review; `Generate Debug Report` nalezy do Export. |
| 11 | [11-debug-export-needs-polish.png](11-debug-export-needs-polish.png) | Debug: Export | `ACCEPTED_WITH_FIXES` | Ujednolicic zakladki i akcje workflow; zawartosc Export jest zaakceptowana bazowo. |

## 3. Decyzje wynikajace z przegladu

### 3.1. Build

Build ma dwa trwale stany wizualne korzystajace z tego samego ukladu:

- `running`: etapy zakonczone, aktywny i oczekujace,
- `failed`: ten sam Stage Ledger, czerwony etap bledu, `Not Run` dla dalszych etapow i dostepny raport.

Sukces nie dostaje osobnego pelnego ekranu. Po sukcesie Build otrzymuje checkmark, a aplikacja przechodzi do Review Output. Ostrzezenia sa prezentowane w Review.

### 3.2. Converted Model, readback i Binary Inspector

- Zakladki viewportu maja nazywac sie `Source Model` i `Converted Model`.
- `Converted Model` jest renderowany z semantycznego readbacku wygenerowanego binary MDL.
- Kontrolka `Verified by binary readback - Inspect Binary` otwiera `Debug Drawer -> Binary`.
- Klikniecie `Inspect Binary` uruchamia macierz bajtow; nie zastepuje ona renderowanego modelu w viewportcie.
- Binary Inspector pozostaje read-only.

### 3.3. Review Output

Kazda zakladka Review ma staly `Reference Trace Bar` w tym samym miejscu i o tej samej wysokosci:

- Model Details: `GLB node -> canonical mesh -> normalized mesh -> MDL node -> binary offset`,
- appearance.2da: `appearance row -> model resref -> binary MDL -> textures -> HAK resource`,
- Package Contents: `source entity -> generated resource -> dependencies -> HAK package`.

Klikniecie wiersza, pola albo zasobu aktualizuje ten pasek. Nie uzywamy plywajacych popupow jako jedynego sposobu prezentacji zaleznosci.

### 3.4. Conversion Readiness

- Brak arbitralnego wyniku 0-100 i oceny artystycznej.
- Kategorie pokazuja `PASS`, `WARNING`, `FAIL`, `NOT CHECKED` albo `OPEN` oraz liczbe wykonanych regul.
- `Package Assembly` i `Runtime Proof` sa osobnymi pozycjami.
- `OPEN_M6` dotyczy runtime proof i nie ukrywa sie pod zielonym sukcesem.
- Kazdy warning z Readiness musi miec odpowiadajacy wpis w Validation.

### 3.5. Animacje i podglad

Read-only Animation Player jest wymagany w Inspect i Review Model Details:

- wybor klipu,
- play/pause, stop i loop,
- timeline oraz current/duration,
- predkosc,
- poprzednia/nastepna klatka.

Debug Overlays obejmuje skeleton, bone names, selected bone, skin weights, wireframe, normals, bounds i axes. Nie jest to edytor animacji, rigu ani wag.

### 3.6. Debug Drawer

Jedyny uklad zakladek V1:

`Diagnostics | Binary | Pipeline Data | Export`

- `Log` i `IR Graph` nie sa osobnymi glownymi zakladkami; ich dane trafiaja do Pipeline Data.
- `Generate Debug Report` znajduje sie w Export, z wyjatkiem bezposredniej akcji ratunkowej po Build failed.
- Rozwiniecie Debug Drawera nie zmienia akcji glownego workflow.
- Drawer jest domyslnie zwiniety na zwyklych ekranach.

## 4. Globalne poprawki dla calej serii

- Uzywac tylko wejsc `GLB + appearance.2da`; usunac `GLTF` z tekstow V1.
- Ukonczony krok zawsze ma checkmark, nie zielony numer.
- Tabele `appearance.2da` i HAK sa zasilane faktycznym schematem oraz manifestem, a liczby w mockupach sa tylko ilustracja.
- Absolutne sciezki, nazwa uzytkownika, dane maszyny, tokeny i konfiguracja Aurory/NWN nie trafiaja do UI ani raportu.
- `Individual Artifacts` jest akcja dodatkowa; glowna akcja Download pobiera Result Package.
- Normalny Review zachowuje `Continue to Download` niezaleznie od otwartej zakladki Debug Drawera.

## 5. Brakujace lub wymagane ponowne mockupy

Przed uznaniem pakietu za `FINAL_VISUAL_CONTRACT` trzeba przygotowac:

1. Source z wybranymi plikami,
2. Build running bez viewportu,
3. Build failed w identycznym ukladzie bez viewportu,
4. Pipeline Data z docelowym ukladem czterech zakladek,
5. poprawiony Binary z akcjami Review,
6. poprawiony Export z akcjami Review.

## 6. Regula implementacyjna

Implementacja korzysta z ekranow `ACCEPTED_BASE` i `ACCEPTED_WITH_FIXES` lacznie z opisanymi korektami. Widokow `REWORK_REQUIRED` nie wolno kopiowac 1:1. Widok `OBSOLETE_DO_NOT_IMPLEMENT` pozostaje w repo wylacznie jako zapis odrzuconej iteracji.
