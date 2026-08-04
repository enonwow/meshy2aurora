# `m2atgls7.mod`

Moduł w Toolset: `Meshy2Aurora TLC Guard Longsword item v7`

Area: `TLC Guard Longsword Item Properties V7`

Status: `ready_for_owner_proof`. Agent nie uruchamiał Aurora Toolset ani NWN.

## Dokładna linia kandydata

- MOD: `m2atgls7.mod`
  - SHA-256: `9cab09be040c5451461a1f65d904eacf1d28322e2c197fe96fd140dfdb277081`
  - źródło: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-properties-v7-20260804\m2atgls7.mod`
  - instalacja: `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2atgls7.mod`
- Ordered HAK: `m2atglh7.hak`
  - SHA-256: `6c45a4fc242cb2b744a501361b75c1fd848d4b43350082dd80eed02b9e17f74f`
  - źródło: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-properties-v7-20260804\m2atglh7.hak`
  - instalacja: `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2atglh7.hak`
- Blueprint resref: `m2atglu7`
- Nazwa itemu: `Last City Bronze Guard Longsword`
- BaseItem: `1` (`longsword`, `WSwLs`, `ModelType 2`)
- UTI: `Identified=true`, `ModelPart1=23`, `ModelPart2=63`, `ModelPart3=23`
- Fixture: jeden ground Item, zero creatures.

Źródła i instalacje MOD/HAK mają identyczne rozmiary i SHA-256. Pełny raport: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-properties-v7-20260804\ready-for-owner-proof.json`.

## Minimalny delta V6 → V7

- wszystkie 12 MDL są byte-for-byte identyczne z V6;
- wszystkie 12 tekstur modelu jest byte-for-byte identyczne z V6;
- nie zmieniono fitu, obrotu modelu, proporcji ani łączników;
- zmieniono dokładnie 12 warstw ikon — po trzy part-y dla czterech wariantów koloru;
- wspólna rama ikon została obrócona o 180°, bez niezależnego obracania, centrowania lub skalowania partów;
- nowa bramka `LONG_VERTICAL_PART_ORDER_V2` wymaga w odczytanym TGA kolejności Top nad Middle nad Bottom.

Dla wybranego koloru `3` odczytane zakresy Y wynoszą:

- Top: `12..91`;
- Middle: `91..98`;
- Bottom: `97..117`.

Złożenie ma `partOrderStatus=PASSED`, bbox `2,12..30,117` i axial fill `0.8203125`.

## Oczekiwany test właścicielski

1. Otwórz dokładnie `m2atgls7.mod` i Area `TLC Guard Longsword Item Properties V7`.
2. Otwórz properties itemu `Last City Bronze Guard Longsword`.
3. Duży lewy viewport powinien wyglądać dokładnie jak zaakceptowany V6 — model nie został zmieniony.
4. Mała ikona po prawej powinna być obrócona względem V6 o 180°: ostrze u góry, jelec pośrodku, rękojeść i głowica na dole.
5. W Area pozostaje jeden Item i zero testowych creature.
6. Wynik pozostaje `modelVisibility=not_tested`, `proofCompleteness=missing`, `visualAcceptance=not_tested` do czasu Twojego werdyktu.

Jeżeli wynik będzie błędny, potrzebny jest świeży screenshot związany z tym dokładnym MOD/HAK. Nie należy tworzyć kolejnej iteracji przed zapisaniem wyniku V7.
