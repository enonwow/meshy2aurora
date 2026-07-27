# MOD: `m2a_h2r46.mod`

Toolset module name: `Meshy2Aurora procedural humanoid proof`

Area: `Meshy2Aurora M0 binary vertical-slice area`

Status: `ready_for_owner_proof`

## Co testować

W NWN wybierz dokładnie nowy lokalny moduł `m2a_h2r46`. Nie testuj
`m2a_h2r45.mod`: r45 pozostaje przypisany do potwierdzonego wyniku
`not_visible`.

Exact obiekt:

```text
Display name:     Meshy procedural humanoid
Tag:              m2a_procedural_creature
TemplateResRef:   m2a_h2utc46
Appearance row:   15220
HAK:              m2a_h2r46
Model resref:     m2a_h2p46
Texture resref:   m2a_h2t46
Creature:         [10.0, 14.5, 0.0], orientation [0.0, -1.0]
Player entry:     [10.0, 10.0, 0.0], direction [0.0, 1.0]
```

Creature stoi bezpośrednio przed graczem i jest zwrócony w jego stronę.
Oczekiwany wynik to widoczny proceduralny humanoid. Po teście podaj wynik dla
exact `m2a_h2r46.mod`; świeży, czytelny pusty kadr oznacza
`NWN modelVisibility=not_visible`, a widoczny model oznacza `visible`.

## Co zmienił r46

r46 zachowuje exact source GLB, teksturę, geometrię, skin, 42 stany type 5,
23 callbacki, dwa bazowe controllery SkinMesh, profil
`ActiveMonsterBaseline`, placement oraz jawny `Phenotype=INT 0`.

Zmiana jest ograniczona do dwóch potwierdzonych defektów r45:

1. model i wszystkie lokalne animacje mają natywne root-first numbering z
   rootem `part 0`; wszystkie parent links i skin maps zostały spójnie
   przemapowane;
2. wiersz direct-S `appearance.2da[15220]` jest pełnym klonem wszystkich
   35 komórek działającego stockowego donor row `102`; zmienione są tylko
   `LABEL` i `RACE`.

Hipoteza brakującego fenotypu została wykluczona: r45 już miał
`Phenotype=INT 0` w GIT i UTC, a direct `MODELTYPE=S` rozwiązuje model przez
`Appearance_Type -> appearance.2da.RACE`. r46 dodatkowo ma negatywny readback
gate, który odrzuca inny typ lub wartość fenotypu.

## Exact hashe i instalacja

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

Exact MOD i HAK zostały zainstalowane metodą create-new/no-clobber:

```text
C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_h2r46.mod
C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_h2r46.hak
```

Hashe obu destination są identyczne ze źródłami w kanonicznym workspace.
Agent nie uruchamiał ani nie sterował Toolsetem/NWN.

Pełny packet:

- `proof-output/h2-r46-root-first-full-appearance-20260727/ready-for-owner-proof.json`
- `proof-output/h2-r46-root-first-full-appearance-20260727/h2-r46-root-first-full-appearance-lineage-contract-v1.json`
- `documentation/evidence/h2-r45-post-failure-r46-admission-2026-07-27.json`
