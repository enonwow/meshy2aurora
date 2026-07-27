# MOD: `m2a_h2r45.mod`

**Module Name w Toolset:** `Meshy2Aurora procedural humanoid proof`  
**Area:** `Meshy2Aurora M0 binary vertical-slice area`

## Status

Exact r45 jest `ready_for_owner_proof`. Nie ma jeszcze deklaracji, że model jest
widoczny: Toolset i NWN pozostają `modelVisibility=not_tested`, dopóki
właściciel nie poda wyniku dla tej dokładnej lineage.

## Co naprawiono po wyniku r44

Właścicielski wynik exact `m2a_h2r44.mod` został zapisany jako
`NWN modelVisibility=not_visible`. Audyt wykazał brak wymaganej części ABI
bazowego węzła SkinMesh:

- r44 miał `0` base controllerów na SkinMesh;
- w odczytanym natywnym korpusie CEP wszystkie `1437` binarnych węzłów
  SkinMesh miały co najmniej `position` i `orientation`;
- minimalna r45 dodaje wyłącznie stały `position` typu `8` o wartości
  `[0,0,0]` oraz stały `orientation` typu `20` o wartości
  `[0,0,0,1]`.

Porównanie semantyczne r44/r45 potwierdziło zachowanie geometrii, indeksów,
normalnych, UV, map i wag SkinMesh, referencji kości, bindów, listy animacji,
profilu creature, Appearance i placementu. Funkcjonalna różnica SkinMesh to
`0 -> 2` base controllery.

## Dokładna lineage

```text
MOD filename:       m2a_h2r45.mod
Module Name:        Meshy2Aurora procedural humanoid proof
Area:               Meshy2Aurora M0 binary vertical-slice area
Area resref:        m2a_h2a45
HAK:                m2a_h2r45
Appearance row:     15219
Model:              m2a_h2p45
Texture:            m2a_h2t45
Creature template:  m2a_h2utc45
Creature tag:       m2a_procedural_creature
Player position:    [10.0, 10.0, 0.0]
Creature position:  [10.0, 14.5, 0.0]
```

Creature stoi bezpośrednio przed graczem. Model ma jeden direct-root SkinMesh,
22 aktywne kości, 42 stany animacji typu 5 i 23 callbacki.

## Zainstalowane pliki

Instalacja została wykonana create-new/no-clobber. Źródła i native destination
są bajtowo identyczne:

```text
C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_h2r45.mod
SHA-256 c6ce6e814fcd56b0ce8e86738b4cba4b01174065689f5a546d6206f9adc4955e

C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_h2r45.hak
SHA-256 de8adc62b0dedadf0485f7a6072e9e5ce235255fdc20c7e0beef3182362e7937
```

Model MDL:

```text
m2a_h2p45.mdl
SHA-256 ba10e8b3221e5748a7c8072e3364815ed5e932adb03d45234bb4106c98a8e1dc
```

## Test właścicielski

W NWN wybierz lokalny moduł `m2a_h2r45`. Po wejściu do Area oczekiwany humanoid
powinien znajdować się na wprost gracza. Wynik należy przypisać wyłącznie tej
lineage:

```text
NWN modelVisibility = visible | not_visible
```

Nie należy testować ani oceniać starego `m2a_h2r44.mod` jako r45.

Pełny maszynowy pakiet przekazania:
`proof-output/h2-r45-skin-bind-controllers-20260726/ready-for-owner-proof.json`.
