# Creature Animation Mapping v2

Data: 2026-07-28

Status: mockup koncepcyjny po korekcie modelu stanow i slotow animacji
Aurora. Nie jest proofem zaimplementowanego UI.

## Ekran

- `01-aurora-state-mapping.png` - mapowanie semantycznego stanu Aurory na
  rodzine `MODELTYPE`, bazowy slot `S/L` i konkretne zrodlo animacji.
- `02-aurora-state-mapping-ux-corrected.png` - poprawiona wersja po audycie
  UX; jest rekomendowanym mockupem do dalszego projektowania.

## Decyzje pokazane w mockupie

1. `Base 42` jest oddzielnym, stalym katalogiem bazowych slotow profilu
   `S/L`.
2. Gracz wybiera stan/akcje Aurory, a pipeline rozwiazuje wariant klipu wedlug
   `MODELTYPE`.
3. `local`, `inherited`, `generated` i `Meshy clip` opisuja zrodlo realizacji
   slotu, a nie stan.
4. Typ odtwarzania (`Loop`, `One-shot`) jest metadana mapowania.
5. Animacje uzytkownika sa pokazane osobno jako `Custom animations` i nie
   zastepuja katalogu `Base 42`.
6. UI nie wymaga od gracza surowej sciezki do modelu referencyjnego ani
   supermodelu.

## Korekty UX w wersji 02

1. Katalog ma widoki `Needs attention`, `Base 42` i `Custom`, wyszukiwanie
   oraz akcje automatycznego mapowania.
2. Stan, `MODELTYPE`, wynikowy slot i typ odtwarzania sa wyliczane z profilu
   Aurory i pokazane jako read-only.
3. Edytowalna sekcja dotyczy realizacji zrodla: inherited, Meshy clip albo
   generated.
4. Provider, asset i ownership sa trzema osobnymi wymiarami provenance.
5. Fallback ma status `Review fallback` i wymaga jawnej akceptacji.
6. Custom animation korzysta z opcjonalnych faz `Start / Loop / End`; nie
   istnieje bledne polaczenie `customXlp + One-shot + required End`.
7. Preview ma porownanie Source/Retargeted oraz kontrole skeleton, root motion,
   loop seam, ground contact, root drift i duration.
8. Jeden dolny pasek statusu rozroznia mapped, review i blocker. Przejscie
   dalej jest zablokowane, dopoki istnieje blocker.

Kanoniczny kontrakt terminologii i provenance znajduje sie w
[`aurora-animation-system-codex.md`](../../aurora-animation-system-codex.md).

## Wykonanie

Mockup zostal wygenerowany wbudowanym trybem ImageGen z wykorzystaniem
istniejacego mockupu Tileset Builder jako reference-only wskazowki stylu.
Finalny obraz zostal zapisany w kanonicznym worktree na branchu `animation`.
