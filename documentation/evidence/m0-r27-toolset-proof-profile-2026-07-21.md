# M0 r27 declarative Toolset proof profile

Data: 2026-07-21  
Status: `DECLARATIVE PROFILE VERIFIED / CENTRAL CONTINUATION PENDING`

## Zakres

Dodano wyłącznie caller-owned, deklaratywne wejście dla przyszłego centralnego
kontraktu `aurora-toolset-binary-module-toolset-proof-profile/v1`:

`C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-toolset-proof-v1.json`

Profil nie jest runnerem ani adapterem UI. Nie uruchamia Toolsetu lub NWN, nie
wykonuje Save i nie zmienia MOD, HAK, MDL, TGA, 2DA, fixture, resrefów ani
kolejności HAK-ów.

## Binding

Profil wiąże istniejący binary bootstrap profile:

- ścieżka:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-binary-bootstrap-v1.json`;
- SHA-256:
  `759f9a95dad7b7d8de6d2ca494800bc8d9c65c25a9dcab3c7651df07c9c4aa6c`;
- fixture: `m0_fixture`;
- native tree object text: `Dwarf Mercenary`, occurrence `0`;
- capture target: `TScrollBox` z `allowUniformScene=false`;
- `noSave=true` i `noLocalToolsetAdapter=true`.

Bazowy profil nadal jest jedynym źródłem tożsamości exact MOD-u, Area, entry,
fixture'a oraz ordered listy trzech HAK-ów. Nowy profil nie duplikuje ani nie
nadpisuje tych wartości.

## Lokalny kontrakt

Dodano test:

`C:\Projects\meshy2aurora\tools\m0-r27-toolset-proof-profile.contract.test.mjs`

Test jest read-only. Sprawdza pełną, niezmienną strukturę profilu, rzeczywisty
SHA-256 bazowego profilu oraz to, że `m0_fixture` występuje w nim dokładnie raz
i nadal wskazuje `nw_dwarfmerc001`, Appearance `848`, `(10,14.5,0)`.

## Granica centralna i następny gate

Centralny publiczny runner i packet validator dla tego kontraktu jeszcze nie
istnieją. Dopiero centralna no-Save continuation może legalnie wykonać:

`exact Area -> exact fixture selection/readback -> fresh validated TScrollBox -> profile-bound packet validation`.

Do tego czasu profil nie stanowi wizualnego proofu:

- Toolset r27: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN r27: `modelVisibility=not_tested`, `proofCompleteness=missing`.

Runtime-v2 nie został zmaterializowany. Jego `toolsetProof` musi wskazywać
rzeczywisty hash świeżego, zaakceptowanego packetu Gate B; wpisanie fikcyjnego
hasha przed tym packetem byłoby niedozwolonym obejściem bramki.

Nie zmieniono `proof-output/m0-r27-animation-type5-20260721/live` i nie
uruchomiono Toolsetu ani NWN.
