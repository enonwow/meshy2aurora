# Creature weapon grip V3 — poprawka offline — 2026-08-02

## Werdykt

V2 rozwiązał UTI, slot wyposażenia, `MODELTYPE=L` oraz nazwy hooków, lecz
pozostawił hook jako jednostkowe dziecko kości dłoni. Meshy i natywne Creature
NWN używają innych bind-basis kości. Skutkiem był widoczny miecz odsunięty od
dłoni i obrócony w złej osi.

Właścicielski wynik dokładnego `m2aweapdemo2.mod`:

- model: `visible`;
- miecz: wyposażony i renderowany;
- chwyt: `failed` — błędna pozycja i rotacja;
- screenshot SHA-256:
  `d10867cb1ac2600be054fbb8b4e3145cef400ebc7e5db62bc4de95ddb80aca8c`.

## Dowód strukturalny

Binary readback V2 (`m2aweapcre2.mdl`, SHA-256
`b587ee2419f875b8081023bea4615930b2c585468698534a2177cdec454747cd`):

- `RightHand -> rhand`;
- `rhand.position = (0, 0, 0)`;
- `rhand.orientation = (0, 0, 0, 1)`;
- analogiczny identity transform dla `lhand`.

Read-only retail `c_lich.mdl`:

- `rhand_g -> rhand`;
- `rhand.position = (0.0110699, 0.0000002, -0.0961304)`;
- `lhand.position = (-0.0153355, -0.0000002, -0.0945176)`;
- hook znajduje się około `0.33` długości segmentu przedramię→dłoń poza
  kością dłoni.

## Implementacja V3

`derive_humanoid_weapon_anchor_options_v2`:

1. odczytuje dokładną hierarchię i bind transform dłoni;
2. wyznacza kierunek segmentu przedramię→dłoń w world bind pose;
3. przedłuża go o natywny współczynnik chwytu (`0.32856` prawa,
   `0.32511` lewa);
4. przelicza offset do lokalnej przestrzeni dłoni;
5. kompensuje pełną bind-world rotację dłoni, aby hook otrzymał kanoniczną
   ramę przedmiotu NWN;
6. nie zmienia geometrii, UV, materiałów, wag ani animacji.

Exact offline audit oryginalnego Fogbound Claw Guard wyliczył:

- right matrix translation: `(-0.0089670, 0.0179733, 0.0809859)`;
- left matrix translation: `(0.0096558, 0.0203296, 0.0826496)`;
- oba hooki mają nie-identity proper rotation i zachowują determinant `+1`;
- nie utworzono nowego MDL, HAK ani MOD.

## Gates

- testy kalibracji/odrzucenia degeneratów/idempotencji: `5 passed`;
- produktowy binary MDL: niezerowy offset, nie-identity orientation oraz
  obecność `rhand/lhand` w `42/42` klipach: pass;
- exact source-only Fogbound audit: pass;
- geometria i animacje pozostają poza zakresem mutacji.

## Model-iteration gate

Nie powstał `r3`. Właścicielski wynik V2 zachowuje
`modelVisibility=visible`, a twarda reguła projektu dopuszcza nowy model/HAK/MOD
dopiero po świeżym `modelVisibility=not_visible` albo po bezpośredniej decyzji
właściciela zmieniającej tę granicę. Kod jest gotowy offline; materializacja i
instalacja pozostają zatrzymane na tej bramce.
