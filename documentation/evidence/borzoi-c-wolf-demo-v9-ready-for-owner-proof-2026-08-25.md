# `m2aborzmod9.mod` — Borzoi `c_wolf` V9 gotowy do proofu właściciela

Moduł wyświetlany w Toolsecie: `Meshy2Aurora Borzoi c_wolf Demo V9`.

Dokładny Area: `Meshy2Aurora Borzoi Test Area V9` (`m2aborzarea9`).

Status: `ready_for_owner_proof`. Agent nie uruchamiał Toolsetu ani NWN.

## Zamrożone artefakty

- MOD `m2aborzmod9.mod`: 15 281 B, SHA-256
  `2e97e2d708fbf881c466fdbc4658f2ea06fd401b4803568a6da369230e51e407`;
- HAK `m2aborzhak9.hak`: 82 737 243 B, SHA-256
  `bbb676933f9052948a0947991c9a5349a8961edae6f53d46199e727542a72f62`;
- MDL `m2aborzcre9.mdl`: 25 503 880 B, SHA-256
  `fefcd3298b711edb9d87dff394f408a6e52b7bfd831516b143ca627a89b1c64e`;
- diffuse `m2aborztex9.tga`: SHA-256
  `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1`;
- minimalny MTR `m2aborztex9_m0.mtr`: SHA-256
  `c81f93e226ad0a40161bec958c2c86cec0b42f832f7db7f79946891a0d205baf`.

Źródłem jest kanoniczny `source-p300k.glb`, SHA-256
`f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda`,
z dokładnie 300 000 trójkątów.

## Co rzeczywiście zmienia V9

V9 powstał przez aktualny, wybieralny pipeline referencyjnych supermodeli, a
nie przez dawny generator specjalny dla `c_wolf`. Pipeline odczytał z
retailowego MDL pełną hierarchię i kontrolery, zbudował 30 carrierów, przypisał
23 aktywne kości do powierzchni psa i pozostawił 0 lokalnych klipów w modelu.
Animacje są dziedziczone z `c_wolf`.

Walidacja obejmuje wszystkie 42 klipy oraz 907 wymaganych par joint×clip.
`Wolf_tail` i `Wolf_tailend` mają ważone klastry powierzchni, a semantyczna
różnica względem odrzuconego V8 została potwierdzona. Wynik offline:

- `motionQuality=PASS`;
- 42/42 klipy i 907/907 joint×clip;
- 0 przerw szwów;
- 0 błędów kontaktu i stron łap;
- 30 carrierów i 23/23 aktywne kości;
- 14 segmentów renderujących ma `shadow=0`, co usuwa źródło ciemnych,
  przerywanych pasów samo-cieniowania widocznych w V8;
- model wskazuje `c_wolf` i nie zawiera lokalnych animacji.

## Instalacja i scena

Dokładne pliki zostały skopiowane do wcześniej nieistniejących celów i po
kopii zweryfikowane SHA-256:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aborzmod9.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aborzhak9.hak`.

Moduł ma jeden HAK `m2aborzhak9`, Appearance row `15100` i creature template
`m2aborzutc9`. Pies stoi w `(10.0, 14.5, 0.0)` naprzeciw gracza.

## Kryteria proofu właściciela

1. Otworzyć `m2aborzmod9.mod` i Area
   `Meshy2Aurora Borzoi Test Area V9`.
2. Potwierdzić widoczność psa i brak crasha.
3. W NWN sprawdzić idle, chód/bieg i walkę.
4. Szczególnie sprawdzić ruch geometrii ogona oraz końcówki ogona.
5. Sprawdzić, czy zniknęły ciemne przerywane linie z powierzchni sierści.

Do wyniku właściciela osie pozostają:
`modelVisibility=not_tested`, `proofCompleteness=missing`.
