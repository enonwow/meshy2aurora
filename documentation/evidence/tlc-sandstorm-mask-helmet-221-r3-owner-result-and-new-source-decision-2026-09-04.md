# Wynik właściciela R3 i decyzja o osobnym źródle hełmu

Data: 2026-09-04

## Wynik przekazany przez właściciela

Właściciel przekazał w rozmowie:

> r3 działa zadawalajaco - duzy problme jest z samym modelem

Fakty wynikające z raportu właściciela:

- dokładna linia R3 pozostaje zamrożona jako `helm_221`;
- funkcjonalność R3 została oceniona jako zadowalająca;
- model jest widoczny i działa, lecz jego jakość wizualna została odrzucona;
- raport nie rozdziela osobno wyniku Toolsetu i NWN, dlatego nie służy do
  przypisania dwóch niezależnych lane verdicts;
- nie wolno nadpisywać ani przepakowywać R3 pod jego istniejącymi nazwami.

## Decyzja dla nowego pliku

Plik `bronze mask 3d model.glb`, SHA-256
`17ea0a4c777352885d81d311c7c6996bad2d3106e62c2f11d990bc3eecc429b5`,
jest nowym, jawnie wybranym przez właściciela źródłem. Nie jest iteracją ani
podmianą bajtów zamrożonego `helm_221` R3.

Nowy zasób otrzymuje niezależną linię:

- canonical asset id: `tlc-bronze-mask-tripo-20260904-v1`;
- Appearance: `helm_222`;
- icon: `ihelm_222`;
- osobny HAK i MOD;
- moduł palette-only, bez UTC i bez stworzenia.

Wynik nowej linii pozostanie `modelVisibility=not_tested` oraz
`proofCompleteness=missing` do czasu wizualnego testu właściciela.
