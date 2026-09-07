# `m2aborzmod8.mod` — wynik właściciela: jakość i ruch ogona niezaliczone

Data: 2026-08-21

Status: `owner_runtime_quality_failed / exact_v8_frozen`

## Tożsamość sprawdzonego kandydata

- moduł: `m2aborzmod8.mod`;
- SHA-256 MOD: `a2b43eb8432b757ec9c46d28bc471c477b6763976c673c3bd06ce5c5acdb2853`;
- HAK: `m2aborzhak8.hak`;
- SHA-256 HAK: `63f1840e1ee1128d75031e547990d6b560834af334abd75c32b5719c041b2e23`;
- model: `m2aborzcre8.mdl`;
- SHA-256 MDL: `a2749e97a35dd6c41d9dd0cefbb3c20301927d453271b6d90c10ece44bcdfa5f`;
- tekstura: `m2aborztex8.tga`;
- SHA-256 TGA: `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1`;
- moduł widoczny w Toolsecie: `Meshy2Aurora Borzoi c_wolf Demo V8`;
- Area: `Meshy2Aurora Borzoi Test Area V8`.

## Wynik przekazany przez właściciela

Właściciel potwierdził, że model jest widoczny, ale wizualnie jest coraz
gorszy: nadal występują przerywane linie/rozcięcia powierzchni, a ogon nie
porusza się tak jak w modelu korzystającym poprawnie z `c_wolf`.

- `modelVisibility = visible`;
- `proofCompleteness = verified` dla zgłoszonego wyniku jakościowego;
- `qualityVerdict = failed`;
- `surfaceSeamVerdict = failed`;
- `tailMotionVerdict = failed`.

## Fakty z audytu offline exact V8

1. Po znormalizowaniu wyłącznie równodługich nazw zasobów
   `m2aborzcre7/8` oraz `m2aborztex7/8` binarne MDL V7 i V8 mają ten sam
   SHA-256:
   `db74d9e766ab67e6ffa2f058200f40ef7f017cb616bed5019fbc4192284cd0b9`.
   V8 nie zawiera więc zmiany geometrii, wag ani hierarchii względem V7.
2. `rig-profile.json` V7 i V8 jest byte-identical:
   `9c4d1ed90aa7e2d9977c4e2b49d4b482022251fa387cfa9b2b4d70d425a42cca`.
3. TGA V7 i V8 jest byte-identical:
   `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1`.
   Zgłoszonych linii nie można przypisać nowej wersji bitmapy V8.
4. Odczyt własnym parserem exact MDL V8 potwierdza 44 węzły: 30 węzłów
   nośnika `c_wolf` i 14 strumieni SkinMesh.
5. `Wolf_tail` jest mapowany w 14/14 strumieniach SkinMesh i ma niezerowe
   wagi 40 797 wpisów wierzchołkowych o łącznej wadze 16 919,932 po
   partycjonowaniu. `Wolf_tailend` jest mapowany w 12/14 strumieniach i ma
   niezerowe wagi 8 732 wpisów o łącznej wadze 2 122,014.

## Odrzucone wyjaśnienia

- V8 nie jest nową deformacją, która przypadkiem pogorszyła V7; jej istotny
  payload MDL jest taki sam jak V7.
- Brak ruchu ogona nie wynika z całkowitego pominięcia węzłów
  `Wolf_tail`/`Wolf_tailend` ani z całkowitego braku ich wag w SkinMesh.
- Linie nie są skutkiem innej bitmapy V8, ponieważ TGA jest byte-identical z
  V7.

## Diagnoza

Offline gate był semantycznie niewystarczający w dwóch miejscach:

1. Dopuścił bardzo dużą liczbę lokalnych naruszeń ciągłości powierzchni jako
   wynik mieszczący się w tolerancji: V8 miała 820 131 naruszeń przy limicie
   926 422. Runtime pokazuje je jako wyraźne linie/rozcięcia. `PASS` nie był
   dowodem braku artefaktów wizualnych.
2. Test `tail_tip_weighted_cluster` sprawdzał obecność klastra i wag, a nie
   rzeczywiste przemieszczenie widocznej geometrii ogona pod animacjami
   odziedziczonymi z supermodelu. Nazwa testu sugerowała mocniejszy kontrakt,
   niż faktycznie weryfikował.

To jeszcze nie rozstrzyga, czy końcowy błąd ogona leży w semantyce
dziedziczenia runtime, w bindzie odwrotnym SkinMesh czy w przypisaniu
widocznego obszaru ogona do kości. Rozstrzygnięcie wymaga testu amplitudy i
trajektorii zdeformowanej powierzchni względem exact referencyjnego `c_wolf`,
a nie kolejnego testu samej obecności nazw i wag.

## Wymagany następny krok

Przed kolejnym kandydatem pipeline musi fail-closed:

- mierzyć przemieszczenie i trajektorię widocznego klastra ogona w co najmniej
  `cpause1`, `cwalk` i `crun` względem ruchu `Wolf_tail` i `Wolf_tailend` z
  exact `c_wolf`;
- wykrywać linie jako błąd ciągłości zdeformowanej powierzchni, zamiast
  akceptować je w szerokim limicie statystycznym;
- dowodzić, że wyjściowy MDL różni się semantycznie od odrzuconego kandydata w
  obszarze, który ma naprawić zgłoszony defekt.

Nie utworzono nowego MDL, HAK ani MOD. Exact V8 pozostaje zamrożona jako
niezaliczony kandydat i nie jest bezpiecznym fallbackiem jakościowym.
