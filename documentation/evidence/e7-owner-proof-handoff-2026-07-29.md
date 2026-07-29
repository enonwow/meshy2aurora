# MOD: `m2a_h2r46.mod`

Toolset module name: `Meshy2Aurora procedural humanoid proof`

Area: `Meshy2Aurora M0 binary vertical-slice area`

Status: exact, istniejący candidate r46; bez nowej iteracji i bez modyfikacji
immutable proof packetu.

## Tożsamość

```text
HAK:              m2a_h2r46.hak
Ordered HAK:      m2a_h2r46
Model resref:     m2a_h2p46
Texture resref:   m2a_h2t46
Module resref:    m2a_h2r46
Area resref:      m2a_h2a46
TemplateResRef:   m2a_h2utc46
Appearance row:   15220
```

Exact object:

```text
Display name:     Meshy procedural humanoid
Tag:              m2a_procedural_creature
Creature:         [10.0, 14.5, 0.0]
Orientation:      [0.0, -1.0]
Player entry:     [10.0, 10.0, 0.0]
Player direction: [0.0, 1.0]
```

Źródło: `h2-clockwork-sentinel-1500`, SHA-256
`f8cf0af21c8143a62b64c490a81dd2855ad3c3f9865922e3854f84b714dec3a3`.

Mapping V4 ma authoring revision `1` i fingerprint
`9e09be1e1307f96ae9a6b7e91a5cf6b33e6105d2ff3b1981df4ffe06d618affe`.

## Exact artefakty

```text
m2a_h2r46.mod
15157 bytes
SHA-256 b206f5507ce14406d62716f9e75c029044e9090f8ead1cdebfc2574e887b959c

m2a_h2r46.hak
20851627 bytes
SHA-256 027a9fc32fee9f75447c63f7308def35a1d098018d68d4d788d39c1174c7f98a

m2a_h2p46.mdl
612752 bytes
SHA-256 d788f07137c7bf713f18654a14f0ce0559e46315cbb2558ea6dcb6fb7fc8d739
```

Odczyt z 2026-07-29 ponownie potwierdził, że canonical MOD/HAK i istniejące
pliki w natywnych katalogach NWN mają te same długości i SHA. Niczego nie
kopiowano ani nie nadpisywano.

## Oczekiwane wywołanie animacji

Bazowy stan `aurora.direct-creature.idle.pause` mapuje się na `cpause1`.
Podstawowa próba to pozostawienie exact creature w zarządzanym przez silnik
stanie idle/pause.

Jeżeli właściciel chce wymusić kontrolowaną próbę skryptową, oczekiwane
wywołanie NWScript na obiekcie o exact tagu brzmi:

```c
object oCreature = GetObjectByTag("m2a_procedural_creature");
AssignCommand(
    oCreature,
    ActionPlayAnimation(ANIMATION_LOOPING_PAUSE, 1.0, 6.0)
);
```

To jest oczekiwana droga testowa. Istniejący raport właściciela potwierdza
widoczność r46, a nie nowy agentowy werdykt zachowania tej wymuszonej animacji.

## Statusy dowodowe

Przed raportem właściciela:

```text
Toolset modelVisibility = not_tested
Toolset proofCompleteness = missing
NWN modelVisibility = not_tested
NWN proofCompleteness = missing
```

Osobno zapisany wcześniejszy wynik właściciela:

```text
Toolset modelVisibility = visible
Toolset proofCompleteness = missing
NWN modelVisibility = visible
NWN proofCompleteness = verified
```

Źródło:
`documentation/evidence/h2-r46-owner-toolset-nwn-visual-result-2026-07-27.json`.

## Granica V5

Test E7 potwierdził import, walidację, preview i materializację kompatybilnego
klipu dawcy na prawdziwym H1. Nie zamrożono jednak nowego MOD/HAK Animation
Studio. V5 pozostaje niedopuszczone przez model iteration gate. Dalszy candidate
wymaga bezpośredniego wyjątku właściciela dla jednego exact V5 albo świeżego,
exact candidate-bound wyniku `modelVisibility=not_visible`.

Agent nie uruchamiał ani nie przejmował Aurora Toolset lub NWN.

Pełny zapis maszynowy:
`documentation/evidence/e7-owner-proof-handoff-2026-07-29.json`.
