# `m2atgls6.mod`

Moduł w Toolset: `Meshy2Aurora TLC Guard Longsword item v6`

Area: `TLC Guard Longsword Item Properties V6`

Status: `ready_for_owner_proof`. Agent nie uruchamiał Aurora Toolset ani NWN.

## Dokładna linia kandydata

- MOD: `m2atgls6.mod`
  - SHA-256: `e783d1fb6e896ed65cff501ee342531860912ca64b6ae35989aeed7ef7d047bd`
  - źródło: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-properties-v6-20260804\m2atgls6.mod`
  - instalacja: `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2atgls6.mod`
- Ordered HAK: `m2atglh6.hak`
  - SHA-256: `8869c79d1cd75a88d4ebc3bca42451e258d5b714084ebd3d669928a273a483d3`
  - źródło: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-properties-v6-20260804\m2atglh6.hak`
  - instalacja: `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2atglh6.hak`
- Blueprint resref: `m2atglu6`
- Nazwa itemu: `Last City Bronze Guard Longsword`
- BaseItem: `1` (`longsword`, `WSwLs`, `ModelType 2`)
- UTI: `Identified=true`, `ModelPart1=23`, `ModelPart2=63`, `ModelPart3=23`
- Fixture: jeden ground Item, zero creatures.

Źródła i instalacje MOD/HAK mają identyczne rozmiary i SHA-256. Pełny raport: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-properties-v6-20260804\ready-for-owner-proof.json`.

## Co jest inne względem V5

- kompletna rama orientacji mapuje długość na `+Y`, szerokość na `+Z`, a głębokość/front na `+X`;
- `widthToDepthRatio=3.6271276474`, `handednessDeterminant=1`, pełna rama ma `status=PASSED`;
- wszystkie cztery warianty koloru przechodzą conformance złożenia MDL;
- wszystkie cztery warianty koloru przechodzą conformance trzech pełnopłótnowych warstw ikony;
- wybrana ikona koloru `3` zajmuje 82.03% wysokości płótna i nie jest krótką ukośną linią.

## Oczekiwany test właścicielski

1. Otwórz dokładnie `m2atgls6.mod` i Area `TLC Guard Longsword Item Properties V6`.
2. Otwórz properties itemu `Last City Bronze Guard Longsword`.
3. W głównym viewportcie oczekiwany jest jeden pionowy i połączony miecz Bottom → Middle → Top, pokazany szerokim przodem, a nie krawędzią.
4. W zakładce Appearance oczekiwana jest kompletna pionowa ikona złożona z trzech partów.
5. W Area oczekiwany jest jeden Item; moduł nie zawiera testowej creature. Ułożenie ground itemu nie służy do oceny orientacji Item Properties.
6. Wynik pozostaje `modelVisibility=not_tested`, `proofCompleteness=missing`, `visualAcceptance=not_tested` do czasu Twojego werdyktu.

Jeżeli wynik będzie błędny, potrzebny jest świeży screenshot związany z tym dokładnym MOD/HAK. Nie należy tworzyć kolejnej iteracji przed zapisaniem wyniku V6.
