# Porównanie runtime: Borzoi V10 `c_wolf` vs detaliczny Wilk

Data audytu: 2026-08-26

## Zakres i źródła

Porównano dwa nagrania dostarczone przez właściciela. Analiza była wyłącznie
offline; nie uruchamiano ani nie sterowano NWN lub Aurora Toolset.

| Materiał | Tożsamość widoczna w nagraniu | Parametry | SHA-256 |
|---|---|---|---|
| `Neverwinter Nights 2026.08.26 - 16.13.19.01.mp4` | `Meshy Borzoi - complete c_wolf skeleton` | 1920x1080, 60.001 FPS, 17.666 s | `e0cad71f679e733f6e62a21279f8f3d8dd430edf816774c3056e8f9f711f1b5b` |
| `Neverwinter Nights 2026.08.26 - 16.15.30.02.mp4` | detaliczny `Wilk` | 1920x1080, 60.001 FPS, 34.049 s | `23e4ba1e4f9b3a71eb693ed98221592ecfe8a195296a638360a5aedaa8678b2` |

Nagrania nie są zsynchronizowanym przechwyceniem tego samego klipu od tej
samej klatki. Pozwalają na porównanie jakości ruchu i deformacji, ale nie na
pomiar identyczności fazy lub czasu animacji 1:1.

## Potwierdzone obserwacje

1. **Dziedziczenie animacji działa na poziomie wyboru ruchu.** Borzoi wykonuje
   chód/ruch bojowy, skok do ataku i reakcje należące do rodziny ruchów wilka.
   Nie ma objawu braku klipu ani pozostawania w bind pose.
2. **Deformacja powierzchni nie odpowiada jakości detalicznego wilka.** W
   reprezentatywnej klatce bojowej Borzoja około `6.3 s` tylna część tułowia,
   udo i nasada ogona składają się w ostre kliny/płaty. W Wilku około `27.3 s`
   kończyny i tułów zachowują ciągłą, czytelną sylwetkę mimo skoku.
3. **Ogon Borzoja nie jest całkowicie nieruchomy.** Jego kąt i końcówka
   zmieniają położenie między klatkami. Ruch jest jednak w dużej części
   sztywny: większość ogona podąża za miednicą jako jeden element, podczas gdy
   Wilk pokazuje wyraźniejsze zgięcie rozłożone między nasadę i koniec ogona.
4. **Najważniejszy widoczny defekt jest geometryczny, nie teksturowy.** Ciemne
   linie i ostre granice na Borzoju zmieniają pozycję oraz kształt wraz z pozą.
   To odpowiada fałdowaniu/rozciąganiu trójkątów lub złemu skinningowi, a nie
   jednej stałej przerwie UV w teksturze.
5. **Nie potwierdzono stałej lewitacji V10.** W pozach naziemnych łapy dochodzą
   do powierzchni. Klatki w powietrzu występują w ruchach bojowych także u
   detalicznego Wilka. Nagrania nie są wystarczająco zsynchronizowane, aby
   rzetelnie zmierzyć foot sliding.

## Potwierdzenie w raporcie pipeline'u V10

Raport `product-report.json` wyjaśnia, dlaczego kandydat przeszedł bramkę
offline mimo widocznych problemów:

- `Wolf_tail` ma `259` vertexów w bezpośrednim widocznym klastrze, a
  `Wolf_tailend` ma `6191`. Około 96% vertexów dwóch klastrów ogona przypada
  więc na końcowy joint. Oba jointy są aktywne, ale rozkład segmentów jest
  silnie niezrównoważony.
- `motionQuality.status=PASS`, chociaż naliczono `1,678,244` próbek krawędzi
  poza twardym limitem. Globalny próg dopuszczał `1,888,632`, więc lokalne,
  bardzo widoczne uszkodzenia zostały rozmyte przez 188,863,290 wszystkich
  próbek gęstej siatki.
- Dla `ca1slashl` raportuje się maksymalne wydłużenie krawędzi około `26.59x`
  i skurczenie do `0.0232x`; dla `ccloseh` maksymalne wydłużenie wynosi około
  `30.55x`.
- `clipStartAnchorJumpViolationCount=896` nie zablokował statusu `PASS`.
- Raport podaje zero zapadnięć/ekspansji pól trójkątów, ale jednocześnie
  `triangleSampleCount=0`. Ta bramka nie mierzyła więc pól trójkątów V10.
- `worldNormalOppositionCount` jest tylko diagnostyczne, a nie blokujące.

Wniosek: obecny validator potwierdza obecność ruchu jointów i uśrednioną
trajektorię powierzchni, ale nie chroni przed lokalnym, wizualnie poważnym
fałdowaniem gęstej siatki ani nie wymusza prawidłowej krzywizny łańcucha ogona.

## Werdykt

| Punkt | Wynik |
|---|---|
| Model Borzoja jest widoczny w NWN | `visible` na materiale właściciela |
| Ruchy z rodziny `c_wolf` są wywoływane | potwierdzone obserwacyjnie |
| Ogon ma jakikolwiek ruch | tak |
| Ogon zachowuje się jak ogon detalicznego Wilka | nie |
| Deformacja siatki odpowiada jakości detalicznego Wilka | nie |
| Stała lewitacja | niepotwierdzona |
| Formalna kompletność runtime proof | `missing` — nagrania nie zawierają zwalidowanego pakietu ingestion z hashami MOD/HAK |

Nie należy opisywać V10 jako poprawnego nanoszenia supermodelu 1:1. Warstwa
wyboru animacji działa, natomiast warstwa automatycznego skinningu i bramka
jakości deformacji wymagają poprawy.

## Artefakty porównania

- `artifacts/evidence/borzoi-c-wolf-v10-owner-videos-20260826/video-1-contact-sheet.jpg`
- `artifacts/evidence/borzoi-c-wolf-v10-owner-videos-20260826/video-2-contact-sheet.jpg`
- `artifacts/evidence/borzoi-c-wolf-v10-owner-videos-20260826/video-1-tail-motion.jpg`
- `artifacts/evidence/borzoi-c-wolf-v10-owner-videos-20260826/video-2-tail-motion.jpg`
- `artifacts/evidence/borzoi-c-wolf-v10-owner-videos-20260826/comparison-board.png`
