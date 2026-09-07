# TLC sandstorm mask helmet 221 R2 — wynik właściciela i dopuszczenie R3

Data: 2026-09-04

## Dokładny kandydat

- MOD: m2assmpal221r2.mod
- SHA-256 MOD:
  389405717b3735acbdae9b29b8b51819526bc35f17795937df177dd7e563d6bd
- HAK modelu: m2assmh221.hak
- SHA-256 HAK:
  d6f34a55f1d845aa86fe35cdec4df3cd7c0e9494daf26bb8b028307abfbd84ed
- model: helm_221
- SHA-256 MDL:
  578c5fb940469ff31cf4aae9a50c02e497e7e92dca7c4d33328fb0c7e7f535be
- UTI: m2assmaskpal221
- uporządkowane HAK-i: [m2assmh221, lc_2da]

## Świeży wynik właściciela

Model 221 jest dostępny w selektorze, renderuje się w podglądzie właściwości i
jest widoczny po założeniu w NWN. Wynik zachowuje więc sukces widoczności:

- Toolset: modelVisibility=visible, proofCompleteness=verified
- NWN: modelVisibility=visible, proofCompleteness=verified
- akceptacja jakości: failed

Zgłoszone defekty:

1. widok przedmiotu leżącego w obszarze jest srebrny, a manipulacja jest
   niepraktyczna;
2. pola kolorów nie zmieniają modelu;
3. ikona ekwipunku jest czarnym kwadratem;
4. hełm jest wyraźnie za mały względem działającego hełmu referencyjnego.

Dowody:

- srebrny widok Toolset:
  tlc-sandstorm-mask-helmet-221-r2-owner-toolset-ground-silver-2026-09-04.png,
  SHA-256 fcd3aca50c53a01395f9dbe4be3b73296413e79970c6f8536f1dadfe1d6c4443
- selektor kolorów:
  tlc-sandstorm-mask-helmet-221-r2-owner-toolset-color-picker-2026-09-04.png,
  SHA-256 baefe390d7d44540455d171fde6512c669be5183ff5a9a1956d84186235b8a65
- czarna ikona:
  tlc-sandstorm-mask-helmet-221-r2-owner-runtime-black-icon-2026-09-04.png,
  SHA-256 91244c5fe14a674e93b54bab1515e28f8951bd587d7967419d95a96a67021eeb
- za mała skala:
  tlc-sandstorm-mask-helmet-221-r2-owner-runtime-scale-small-2026-09-04.png,
  SHA-256 254d28b4587c71f800da9fde961024dfdb0f84090aa6fc0a2f86cd356e386538
- hełm referencyjny:
  tlc-sandstorm-mask-helmet-221-r2-owner-runtime-reference-helmet-2026-09-04.png,
  SHA-256 f58070869b6cd90798e8cb73e78279c095a446805dd3db72ee5fe3d42287d57e

## Zdiagnozowana przyczyna i minimalna delta

Audyt dekompilacji jest zapisany w
documentation/AURORA_DECOMP_HELMET_MODELTYPE1_AUDIT_2026-09-04.md.

Minimalna delta R3:

- zamiana modelowej tekstury TGA na warstwowe PLT;
- zamiana ikony TGA na przezroczyste PLT typu 6;
- rzeczywiste warstwy Metal 1 i Cloth 1;
- jednostajna skala względem helm_035;
- redukcja 300k → 100k trójkątów dla responsywności edytora.

Ten świeży, dokładnie związany wynik wizualny R2 dopuszcza jedną iterację R3.
