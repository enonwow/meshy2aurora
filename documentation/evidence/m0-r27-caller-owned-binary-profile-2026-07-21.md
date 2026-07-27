# M0 r27 caller-owned binary/native-geometry profile

Data: 2026-07-21  
Status: `OFFLINE PROFILE VERIFIED / LIVE PROOF MISSING`

## Cel i granice

Zadanie miało odblokować centralny tor proofu dla dokładnego kandydata r27 bez
zmiany MOD, HAK, MDL, TGA, 2DA, fixture, resrefu ani kolejności HAK-ów. Nie
uruchamiano Toolsetu ani NWN, nie wykonywano natywnego Save i nie używano
`tools/m2a-aurora-proof.mjs`.

Historyczny profil centralny `m0-static-rigid-v5` nie jest rozszerzalnym
punktem wejścia: jego module registry, queue, fixture i packet validator są
związane z innym kandydatem. Legalnym istniejącym punktem wejścia jest
caller-owned kontrakt `aurora-toolset-binary-module-bootstrap-profile/v1`
obsługiwany przez publiczne centralne skrypty:

- `validate-aurora-toolset-binary-module-bootstrap.mjs`;
- `aurora-toolset-binary-module-native-geometry.mjs`.

## Implementacja

Dodano jeden deklaratywny profil:

`C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-binary-bootstrap-v1.json`

Profil wiąże:

- istniejący MOD `m2a_m0r25.mod`, SHA-256
  `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257`;
- Area `m2a_m0a25`, rozmiar `2x2`, entry `(10,10,0)`;
- ordered HAK list `[m2a_m0r27, m2a_m0r26, m2a_m0r21]` wraz z dokładnymi
  ścieżkami i hashami;
- fixture `m0_fixture`, template `nw_dwarfmerc001`, Appearance `848`, pozycję
  `(10,14.5,0)`;
- `noLocalToolsetAdapter=true`.

Dodano niezależny test immutability/hash contract:

`C:\Projects\meshy2aurora\tools\m0-r27-binary-bootstrap-profile.contract.test.mjs`

Test nie steruje UI. Odrzuca każdą zmianę pełnej struktury profilu, w tym MOD,
Area, entry, fixture, kolejność/hash HAK-ów i flagę zakazującą lokalnego
adaptera, a następnie hashuje istniejący MOD i trzy HAK-i.

## Walidacja

Wykonano obowiązkowy workspace guard:

```text
canonical-workspace-ok: C:\Projects\meshy2aurora
```

Wyniki bramek:

| Bramka | Wynik |
| --- | --- |
| lokalny contract test | `m0_r27_binary_bootstrap_profile_contract_valid` |
| centralny schema dry-run | `binary_bootstrap_profile_schema_valid` |
| centralny structural preflight | `binary_bootstrap_structural_readback_valid` |
| centralny native-geometry dry-run | `binary_native_geometry_plan_valid` |
| centralny explicit-open admission dry-run | `binary_native_explicit_open_startup_admitted` |
| `git diff --check` | PASS |

Structural preflight odczytał z istniejącego MOD-u dokładnie Area `2x2`, entry,
jeden fixture i trzy HAK-i w kolejności r27/r26/r21. Tym samym centralna trasa
nie wymaga nowego MOD destination ani single-HAK wariantu.

Read-only startup admission wykrył `MRU0=m2a_now01.mod`, czyli konflikt z
targetem r27. Kontrakt v3 zaakceptował ten stan wyłącznie jako kontekst:
`mruConflict=true`, `startsToolsetWithoutModuleArgument=true`,
`forbidsIniOrMruMutation=true` i `openOnlyFromUnownedWelcomeOrBlankFrame=true`.
Nie zmieniono `nwtoolset.ini` ani MRU.

Po walidacji nie istniał żaden proces `nwtoolset`/`nwmain`, a caller-owned
directory
`proof-output/m0-r27-animation-type5-20260721/live` pozostał pusty. Dry-runy
nie zapisały manifestu.

## Handoff do centralnego proofu

Profil jest gotowy do bezpośredniego użycia przez centralny publiczny runner:

```powershell
$profile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-binary-bootstrap-v1.json'
$outDir = 'C:\Projects\meshy2aurora\proof-output\m0-r27-animation-type5-20260721\live'

node C:\Projects\aurora-web\backend\scripts\aurora-toolset-binary-module-native-geometry.mjs preflight --profile $profile --outDir $outDir
```

Następne fazy live pozostają osobno autoryzowanymi fazami centralnej trasy.
Nie wolno zastąpić ich lokalnym runnerem ani wywołaniem leaf atomu. Szczególnie
`geometry-observe` wykonuje natywny Save i może zmienić hash MOD-u; nie został
uruchomiony w tej implementacyjnej turze, której granica wymagała zerowej
zmiany lineage.

### Central-only blocker po preflighcie

Kod blockera:

`CENTRAL_BINARY_R27_TSCROLLBOX_ROUTE_MISSING`

Aktualny publiczny binary/native-geometry runner ma komendy do structural
preflightu, startu jednej sesji, native Save/readbacku i finalizacji profilu
geometrii. Nie ma publicznej fazy, która dla caller-owned profilu:

1. otwiera dokładne Area bez zmiany MOD-u;
2. wybiera i natywnie odczytuje dokładny fixture z profilu;
3. zapisuje świeży capture zwalidowanego `TScrollBox`;
4. składa i waliduje packet wiążący profil/hash MOD-u, ordered HAK-i, Area,
   fixture, selection readback i capture.

`aurora-toolset-viewport-proof.mjs` jest centralnym capture hookiem, ale samo
bezpośrednie wywołanie leaf atomu nie jest publiczną continuation binary
profile i nie zapewnia exact-object binding/packet validatora. Meshy2Aurora nie
może dodać lokalnego wrappera, UI adaptera ani runnera, aby obejść ten brak.

Gotowy centralny handoff:

- rozszerzyć istniejący publiczny
  `backend/scripts/aurora-toolset-binary-module-native-geometry.mjs` albo dodać
  inną nazwaną publiczną continuation dla
  `aurora-toolset-binary-module-bootstrap-profile/v1`;
- wejście: caller profile, jego SHA-256 oraz caller-owned `outDir`;
- preconditions: przechodzący structural preflight, exact explicit-open owner,
  exact module/Area identities i dokładnie jeden responsywny Toolset;
- mutacja live: żadnego Save, HAK/module edit ani konfiguracji; wyłącznie Area
  navigation, exact fixture selection/readback i centralny capture hook;
- packet: module/profile hash, ordered HAK resref/hash, Area resref/native tree
  text, fixture template/Appearance/position, native selection readback,
  `TScrollBox` PNG/result hashes, freshness i safety;
- validator/testy negatywne: wrong MOD/hash, wrong HAK order/hash, wrong Area,
  wrong fixture/Appearance/position, brak selection readbacku, zły target
  capture, stale/non-PNG/unsafe capture;
- bezpośrednie leaf invocation i project-local runner pozostają odrzucone.

Warunek wznowienia: centralna publiczna continuation i jej contract tests są
dostępne, albo aktualny standard wskaże już istniejącą równoważną publiczną
trasę. Do tego czasu profil odblokowuje structural/native admission, lecz pełny
Toolset proof pozostaje `missing`.

## Stan proofu

Ta implementacja odblokowuje wejście do centralnego toru, ale nie jest proofem
wizualnym:

- Toolset r27: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN r27: `modelVisibility=not_tested`, `proofCompleteness=missing`.

Nie powstał r28 ani żaden nowy/kopiowany artefakt modelu.
