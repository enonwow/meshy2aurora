# Biblioteka supermodeli V1 — implementacja i stan weryfikacji

Data: 2026-08-21

## Cel

Studio ma pozwalać użytkownikowi przejrzeć wszystkie supermodele wykryte w
wybranych lokalnych źródłach przed konwersją Creature. Samo otwarcie biblioteki
i wskazanie kandydata nie zmienia modelu, konfiguracji eksportu ani artefaktów.

„Wszystkie” oznacza zasoby odkryte z indeksów dostarczonych przez użytkownika,
bez zaszytej listy nazw. W V1 obsługiwane są bazowe KEY/BIF, wybrane HAK-i oraz
luźne MDL/folder override. Kolejność wielu HAK-ów jest jawną kolejnością
podglądu Studio i nie jest deklarowana jako zgodna z kolejnością silnika.

## Zaimplementowany przepływ

1. Core odczytuje wszystkie lokatory MDL z KEY V1.
2. BIF jest indeksowany dwufazowo: najpierw 20-bajtowy nagłówek, następnie
   dokładny zakres tabeli zasobów.
3. Studio czyta małe prefiksy binarnych MDL. Pełny payload jest pobierany na
   etapie katalogowania tylko wtedy, gdy zasób jest tekstowym MDL.
4. Core buduje graf supermodeli bez rozróżniania wielkości liter i wylicza
   definicje, potomków, łańcuch dziedziczenia oraz stany `RESOLVED`, `MISSING`
   i `CYCLIC`.
5. Użytkownik może wyszukiwać i filtrować wpisy, oglądać metadane, a dla
   zasobów obsługiwanych przez ścisły reader binarny — model/szkielet i
   odziedziczone animacje. Najbliższa definicja klipu ma pierwszeństwo przed
   przodkiem.
6. Dla supermodelu bez użytecznej geometrii można wybrać model potomny jako
   nośnik reprezentatywny.
7. Nieobsługiwany element łańcucha nie przerywa całego podglądu. Studio pokazuje
   precyzyjne ograniczenie i nadal próbuje wyświetlić czytelnych przodków lub
   model reprezentatywny.

Sesja przechowuje uchwyty do lokalnych plików wyłącznie w pamięci przeglądarki.
Zasoby nie są wysyłane, kopiowane do projektu ani rozpakowywane do biblioteki
plików.

## Wynik na lokalnej instalacji referencyjnej

Test środowiskowy na `nwn_base.key` zakończył się następująco:

- 32 832 zadeklarowane i przeskanowane zasoby MDL;
- 206 wykrytych supermodeli;
- 62 binarne supermodele przechodzą pełny ścisły odczyt do viewportu;
- 29 binarnych supermodeli jest obecnie nieobsługiwanych przez ścisły reader
  (wszystkie deklarują lokalne animacje);
- 113 supermodeli jest zapisanych jako ASCII i ma w V1 podgląd katalogowy
  metadanych, bez natywnego renderu (51 deklaruje lokalne animacje).

Wniosek: V1 kataloguje i pozwala przeglądać wszystkie 206 wpisów, ale nie
oznacza jeszcze pełnego renderu i odtwarzania animacji dla każdego z nich.
Interfejs nie ukrywa tej różnicy.

## Kryteria zakończenia V1

- [x] Brak twardo zakodowanego ograniczenia do `c_wolf`.
- [x] Wszystkie supermodele wykryte w wybranych źródłach są obecne na liście.
- [x] Katalog raportuje kompletność oraz błędy odczytu zamiast pomijać je po
  cichu.
- [x] Dostępne są wyszukiwanie, filtry, łańcuch, potomkowie, źródło i metadane.
- [x] KEY/BIF, HAK i luźne MDL/override są obsługiwanymi źródłami.
- [x] Czytelne binarne zasoby mają podgląd modelu/szkieletu i animacji
  dziedziczonych.
- [x] Wybór jest kandydatem „bez nakładania” i nie unieważnia ani nie zmienia
  bieżącej konwersji.
- [x] Produkcyjny build WASM/TypeScript/Vite przechodzi.
- [x] Pełny zestaw Studio przechodzi: 55 plików, 314 testów.
- [x] Trasa Biblioteki i powrót do Source zostały sprawdzone w prawdziwej
  przeglądarce.

Pełny natywny viewport dla wszystkich formatów nie jest kryterium domkniętym w
V1. Wymaga osobnego rozszerzenia readera o ASCII MDL i pozostałe warianty
binarnych modeli. Nie należy przedstawiać samej obecności wpisu w katalogu jako
dowodu renderowalności w Toolset/NWN.
