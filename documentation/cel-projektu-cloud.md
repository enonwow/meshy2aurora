# cel-projektu-cloud.md

Status 2026-07-09: OBOWIAZUJACE po doprecyzowaniu D7-D9. `aurora-web` wystepuje tu tylko jako zewnetrzny konsument standardowego HAK-a, nie jako zaleznosc, proof base ani narzedzie wykonawcze `meshy2aurora`. Format docelowy projektu to natywny binary MDL + polityka MDX + 2DA + HAK; ASCII moze byc tylko debug dump/snapshot.
Data: 2026-07-08 | Status: OBOWIĄZUJĄCE (źródło: Mateusz, czat Cloud 2026-07-08)

## Cel właściwy projektu

W meshy.ai generujemy model wejściowy: siatkę, tekstury, opcjonalnie szkielet i animacje. `meshy2aurora` konwertuje taki model do natywnego contentu Aurory/NWN:

1. **binary MDL** — natywny model creature/placeable/item zgodny z formatem Aurora/NWN,
2. **MDX policy** — dane geometrii zapisane embedded w MDL albo jako osobny zasób MDX, zgodnie z rozstrzygnięciem `engine-mdl-pytania-cloud.md` Q2,
3. **2DA** — odpowiednie wpisy, co najmniej `appearance.2da` dla creature,
4. **tekstury** — format zgodny z Aurorą/NWN, na MVP TGA/TXI,
5. **HAK** — spakowanie modeli, tekstur i 2DA do haka gotowego do użycia.

Wynik ma działać jak każdy inny custom content NWN: najpierw w NWN EE Toolset/grze. `aurora-web` może później czytać ten sam standardowy HAK jako zewnętrzny konsument, ale nie jest częścią implementacji ani proof base `meshy2aurora`.

## Konsekwencje dla wcześniejszych ustaleń

- `koncepcja-meshy-cloud.md` (strategia B: surowa siatka + transfer wag z `c_kocrachn`) jest historycznym wariantem badawczym. Nie jest aktywnym celem produktu, jeżeli sugeruje GLB/CDP albo `aurora-web` jako proof.
- Dla modeli z poprawnym rigiem/animacjami z Meshy mapujemy szkielet i nazwy animacji na konwencję NWN.
- Dla modeli bez użytecznego rigu/animacji używamy ścieżki referencyjnej: własny transfer wag/szkieletu i animacje z referencji/supermodelu, ale nadal emitujemy natywny binary MDL.

## Pipeline docelowy

```text
meshy.ai (GLB/FBX: mesh + rig + animacje)
  → meshy2aurora:
      - parse wejścia Meshy (mesh, materiały, tekstury, skin, animacje)
      - normalizacja osi, skali i nazw nodów wg konwencji Aurora/NWN
      - mapowanie/transfer szkieletu, skin weights i animacji
      - zapis natywnego binary MDL
      - zapis MDX embedded albo osobnego zasobu MDX zgodnie z polityką Q2
      - konwersja/przygotowanie tekstur do formatu Aurory
      - generacja wierszy 2DA (appearance.2da, ewent. inne)
      - budowa HAK
  → użycie: NWN EE Toolset/gra
  → opcjonalnie później: aurora-web jako zewnętrzny konsument standardowego HAK-a
```

## Otwarte tematy

Pytania aktywne do Codexa: `engine-mdl-pytania-cloud.md` Q1-Q2 (minimalny binary MDL writer i polityka MDX). Dokumenty `mdl-2da-hak-*` zostają referencją formatu i 2DA/HAK, ale nie definiują już ASCII jako ścieżki wykonawczej.
