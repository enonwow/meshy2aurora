# MOD: `m2a_h2r46.mod`

Toolset module name: `Meshy2Aurora procedural humanoid proof`

Area: `Meshy2Aurora M0 binary vertical-slice area`

Status przygotowania agenta: `ready_for_owner_proof` przez ponowne użycie
identycznego, immutable candidate r46.

## Decyzja

Nie utworzono r47, nowego resrefu ani nowej kopii artefaktu. Addytywna ścieżka
authoringu V4 została odtworzona w pamięci dla dokładnych wejść i tożsamości
r46. Wygenerowane MDL, TGA, `appearance.2da`, HAK i MOD były bajtowo identyczne
z istniejącym packetem r46.

To ponowne użycie nie omija model iteration gate: nie istnieje nowy candidate,
a immutable packet r46 nie został zmieniony.

## Dokument authoringu

```text
Profile:       DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1
MODELTYPE:     S
Base slots:    42
Source:        PROCEDURAL
Provider:      PROCEDURAL_GENERATOR
Asset:         m2a:procedural-humanoid-v1
Ownership:     PROJECT_GENERATED
Fallbacks:     0
Custom:        0
Core status:   READY
Fingerprint:   9e09be1e1307f96ae9a6b7e91a5cf6b33e6105d2ff3b1981df4ffe06d618affe
```

## Exact handoff

```text
MOD:              m2a_h2r46.mod
HAK:              m2a_h2r46
Model resref:     m2a_h2p46
Texture resref:   m2a_h2t46
Display name:     Meshy procedural humanoid
Tag:              m2a_procedural_creature
TemplateResRef:   m2a_h2utc46
Appearance row:   15220
Creature:         [10.0, 14.5, 0.0], orientation [0.0, -1.0]
Player entry:     [10.0, 10.0, 0.0], direction [0.0, 1.0]
```

## Exact hashe

```text
m2a_h2r46.mod
SHA-256 b206f5507ce14406d62716f9e75c029044e9090f8ead1cdebfc2574e887b959c
15157 bytes

m2a_h2r46.hak
SHA-256 027a9fc32fee9f75447c63f7308def35a1d098018d68d4d788d39c1174c7f98a
20851627 bytes

m2a_h2p46.mdl
SHA-256 d788f07137c7bf713f18654a14f0ce0559e46315cbb2558ea6dcb6fb7fc8d739
612752 bytes
```

Odczyt z 2026-07-28 potwierdził, że zainstalowane:

```text
C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_h2r46.mod
C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_h2r46.hak
```

mają dokładnie te same długości i SHA-256 co źródła w `proof-output`.
Nie wykonano ponownego kopiowania.

## Dowód i granica odpowiedzialności

Odtwarzalny audyt:

```text
cargo test -p m2a-core --test r46_v4_compatibility_audit -- --ignored --nocapture
```

Pełny wynik maszynowy:
`documentation/evidence/creature-animation-mapping-v4-r46-compatibility-2026-07-28.json`.

Wcześniejszy wynik właściciela pozostaje osobnym dowodem:

- Toolset: `modelVisibility=visible`,
  `proofCompleteness=missing`;
- NWN: `modelVisibility=visible`,
  `proofCompleteness=verified`.

Źródło:
`documentation/evidence/h2-r46-owner-toolset-nwn-visual-result-2026-07-27.json`.

Agent nie uruchamiał ani nie przejmował Aurora Toolset lub NWN. Zgodność V4
z r46 wynika z exact byte identity, a wynik wizualny wyłącznie z osobno
zapisanego raportu właściciela.
