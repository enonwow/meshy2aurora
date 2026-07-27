# Dowód przesunięcia dolnej misy reaktora

Data: 2026-07-26  
Werdykt: **PASS — rozwiązanie potrafi przesunąć dolną misę jako osobny zespół.**

Uwaga: była to demonstracja możliwości przesunięcia o `Z = +0,10 m`, a nie
docelowa korekta. Kandydat V2 zachowuje misę wycentrowaną pomiędzy nogami,
podnosi ją o `Y = +0,05 m` i pochyla zakotwiczony strumień do wnętrza. Handoff:
[`tlc-meshy-loot-workstations-v2-reactor-alignment-ready-for-owner-proof-2026-07-26.md`](tlc-meshy-loot-workstations-v2-reactor-alignment-ready-for-owner-proof-2026-07-26.md).

## Zakres dowodu

Dowód wykonano offline w aplikacji webowej i współdzielonym rdzeniu
`m2a-core`, na prawdziwym modelu `tlc_upgrad01.glb`:

- SHA-256:
  `cf2f6a7b43c6ccaf09ebb2d560a62bed0245ec4734b8c8df5d9b9e4dd0776ebd`;
- 20 000 trójkątów;
- 810 rozłącznych komponentów;
- bez uruchamiania Aurora Toolset lub NWN;
- bez tworzenia MOD/HAK.

To nie jest końcowy dowód wizualny w Aurorze/NWN. Potwierdza, że edytor i
pipeline potrafią wykonać oraz zachować żądaną operację na geometrii przed
konwersją.

## Co zostało przesunięte

Dolna misa Meshy nie jest jednym połączonym komponentem. W edytorze:

1. rozbito źródłowy render-node na komponenty;
2. wybrano 16 komponentów tworzących rant, ścianki, dno i powierzchnię lawy;
3. odizolowano zaznaczenie i potwierdzono wizualnie, że tworzy dolną misę;
4. połączono zaznaczenie w grupę `Lower basin`;
5. ustawiono grupie `Location Z = +0.10 m`;
6. strumień lawy pozostawiono poza grupą i bez transformacji.

Wybrane komponenty:
`31, 33, 34, 35, 45, 66, 74, 76, 84, 92, 144, 155, 170, 600, 602, 603`.
Łącznie: 2 565 trójkątów.

## Dlaczego przesunięcie naprawia położenie

Strumień lawy, komponent `C766`, ma środek na osi Z równy
`0.22687239 m`.

Przed edycją:

- powierzchnia lawy w misie (`C76`) obejmuje Z od `-0.16121443` do
  `0.16158299 m`;
- górny rant misy (`C34`) obejmuje Z od `-0.18979363` do `0.19049801 m`;
- środek strumienia znajduje się poza oboma zakresami.

Po przesunięciu grupy o `+0.10 m`:

- powierzchnia lawy obejmuje Z od `-0.06121443` do `0.26158299 m`;
- rant obejmuje Z od `-0.08979363` do `0.29049801 m`;
- nieprzesunięty strumień na Z `0.22687239 m` znajduje się wewnątrz obu
  zakresów.

## Materiał dowodowy

- [Widok przed przesunięciem](../../output/playwright/reactor-basin-before.png)
- [Odizolowana dolna misa](../../output/playwright/reactor-basin-isolated-before-move.png)
- [Widok po przesunięciu Z +0,10 m](../../output/playwright/reactor-basin-after-z-plus-010.png)
- [Odczyt UI: grupa i Location Z = 0,1](../../output/playwright/reactor-basin-after-with-transform-ui.png)
- [Manifest dowodu](tlc-upgrading-reactor-basin-authoring-proof-2026-07-26.json)

Hashe plików dowodowych znajdują się w manifeście JSON.

## Weryfikacja regresji

- `cargo test -p m2a-core --test placeable_authoring`
  — 5/5 testów przeszło;
- dodany test
  `group_translation_moves_only_its_component_children` potwierdza, że
  transformacja grupy przesuwa tylko jej dzieci i nie zmienia komponentu
  kontrolnego;
- trzy zestawy testów webowego edytora — 7/7 testów przeszło.

## Wniosek

Przesunięcie dolnej misy nie wymaga regeneracji modelu przez Meshy.
Możemy wykonać tę korektę w naszym edytorze, a wspólny dokument authoringu
przeniesie tę samą transformację do etapów renderu, kolizji i cienia podczas
następnej konwersji.
