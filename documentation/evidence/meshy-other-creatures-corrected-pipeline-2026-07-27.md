# H1 i N1 przez poprawiony pipeline creature

Data: `2026-07-27`

Status: `READY_FOR_OWNER_PROOF / RUNTIME NOT TESTED`

## Zakres

Na polecenie wlasciciela przepuszczono dwa pozostale modele creature z
kanonicznego `sample-3d` przez aktualny pipeline po naprawach r46. Nie
tworzono nowego lineage `rNN`. Utworzono dwa rozlaczne, jednoznaczne pakiety
MOD+HAK, wykonano binarny readback i zainstalowano exact pliki do natywnych
katalogow NWN. Nie uruchamiano ani nie przejmowano Aurora Toolset lub NWN;
proof wizualny nalezy do wlasciciela.

## H1 humanoid

Wejscie:

- `sample-3d/h1-humanoid-1500/source.glb`;
- bytes: `7,944,380`;
- SHA-256:
  `3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`;
- source GLB: `1` skin, `1` source animation.

Najpierw wykonano compatibility route `animated-h1`, ktory poprawnie
zmaterializowal pakiet 7-state. Nastepnie H1 przepuszczono przez dokladna
publiczna sciezke naprawionego r46:

`FULL_NATIVE42_PROCEDURAL_HUMANOID_V1`.

Finalny pakiet H1:

- materializacja zakonczona sukcesem;
- packet:
  `C:\Projects\meshy2aurora\proof-output\h1-full42-20260727`;
- test-module filename: `m2a_h1full42.mod`;
- Toolset module name: `Meshy2Aurora procedural humanoid proof`;
- Area resref: `m2a_h1area`;
- Area name: `Meshy2Aurora M0 binary vertical-slice area`;
- ordered HAK: `m2a_h1full42`;
- model resref: `m2a_h1mdl`;
- texture resref: `m2a_h1tex`;
- creature template resref: `m2a_h1utc`;
- creature display name: `Meshy procedural humanoid`;
- Appearance row: `15100`;
- output MDL: `537,980` bytes,
  SHA-256 `f6842fd353d98542f0055625ec496cee3d83aac0ac4310dadb348e1303d56ab0`;
- output HAK: `20,022,503` bytes,
  SHA-256 `ad70ee816414cef00a014c7a51ab6c8675a972150df33afcd5f364b94c7c1a3b`;
- output MOD: `15,163` bytes,
  SHA-256 `60dd43ac8d0e2d121e841b53742824e42858b496b37861804b04cbc145392ac2`;
- model root: `part 0`;
- `42` lokalne stany type `5`;
- wszystkie animation roots: `part 0`;
- direct-S appearance: pelne `35` komorek, appended row `15100`;
- skopiowany source GLB jest byte-identical z kanonicznym wejsciem.

Wczesniejszy lokalny, ignorowany compatibility output 7-state pozostaje
wylacznie materialem regresyjnym pod:

`C:\Projects\meshy2aurora\.codex-tmp\creature-pipeline-h1-20260727-a1`

## N1 quadruped

Wejscie:

- `sample-3d/n1-quadruped-1500/source.glb`;
- bytes: `6,414,260`;
- SHA-256:
  `0805699f8428707c7eb654361c6e74aa1211e4af1596aee7157ae31afe1b7784`;
- source GLB: `0` skins, `0` animations.

Proby route `animated-h1` oraz dokladnego
`FULL_NATIVE42_PROCEDURAL_HUMANOID_V1` zostaly poprawnie odrzucone:

```text
stage: PROFILE
code: M4A-MESHY-H1-SOURCE-INVALID
path: source.ir.skins
message: Meshy H1 route requires exactly one source skin
```

N1 zostal nastepnie przepuszczony przez jedyna zgodna z jego faktycznym
wejsciem sciezke `static-rigid-creature`, tym razem jako pelny canonical
runtime packet z full `appearance.2da` i caller-owned MOD/Area/HAK identity.

### Dlaczego N1 nie jest dzialajacym animowanym creature

Readback source N1 pokazuje:

- jeden node, ktory bezposrednio posiada mesh, bez przegubowej hierarchii;
- `0` skinow;
- `0` lanes `JOINTS`;
- `0` lanes `WEIGHTS`;
- `0` source animations.

To wyklucza oba obslugiwane mechanizmy ruchu creature:

1. nie ma SkinMesh, wag i joint binding potrzebnych do ciaglej deformacji;
2. nie ma tez wielu rigid segmentow parentowanych do animowanej hierarchii,
   ktore moglyby poruszac sie bez SkinMesh.

Pipeline static-rigid nie zgaduje anatomii, nie tworzy automatycznie szkieletu,
nie generuje wag i nie wymysla ruchu quadrupeda. Siedem lokalnych stanow type
`5` w output MDL to stany `static-identity`: zapewniaja zgodny kontener i nazwy
stanow testowych, ale wszystkie zachowuja nieruchoma poze. Ich obecnosc nie
zmienia source bez animacji w animowany model.

Dlatego test N1 ma dwie osobne oceny:

| Ocena | Czy N1 jest uprawniony? | Znaczenie |
| --- | --- | --- |
| Widocznosc static-rigid | tak | Model moze potwierdzic resolver MOD/HAK/2DA i draw base mesh. |
| Dzialajacy animowany creature | nie | Brak danych, ktore moglyby wykonac walk/run/attack/damage/death albo jakikolwiek widoczny motion. |

Jesli N1 bedzie widoczny w NWN, zaliczy jedynie **static visibility control**.
Nie zaliczy animowanego creature. Jesli nie bedzie widoczny, bedzie to osobny
wynik `modelVisibility=not_visible` dla exact statycznego kandydata, a nie dowod
dotyczacy poprawnej trasy H1 full-42.

Wynik:

- materializacja zakonczona sukcesem;
- packet:
  `C:\Projects\meshy2aurora\proof-output\n1-static-20260727`;
- test-module filename: `m2a_n1static.mod`;
- Toolset module name: `Meshy2Aurora M0 binary vertical slice`;
- Area resref: `m2a_n1area`;
- Area name: `Meshy2Aurora M0 binary vertical-slice area`;
- ordered HAK: `m2a_n1static`;
- model resref: `m2a_m0p01`;
- texture resref: `m2a_m0t01`;
- creature template resref: `nw_dwarfmerc001`;
- creature display name: `Meshy M0 binary vertical-slice fixture`;
- Appearance row: `15100`;
- output MDL: `113,936` bytes,
  SHA-256 `3bbb24fe1a2e904d8eea6db29fc24ade679e478fed00371ffd1a3d25bb22a86f`;
- output HAK: `19,598,465` bytes,
  SHA-256 `fa430e5c0f5e75d56c4f8690c5c5b479b6a14602941f898838624a6fba9aeb53`;
- output MOD: `13,183` bytes,
  SHA-256 `170523f89b70435eaf9899fa80bc519f54afb8e68bf3ab33d4e04c0f04a7460c`;
- model root: `part 0`;
- `7` lokalnych stanow type `5` o semantyce static identity;
- wszystkie animation roots: `part 0`;
- direct-S appearance: pelne `35` komorek, appended row `15100`;
- skopiowany source GLB jest byte-identical z kanonicznym wejsciem.

Lokalny, ignorowany output regresyjny:

`C:\Projects\meshy2aurora\.codex-tmp\creature-pipeline-n1-static-20260727-a1`

## Instalacja exact MOD/HAK

Przed kazda kopia cel zostal ponownie potwierdzony jako nieistniejacy.
Kopiowanie wykonano w trybie bez nadpisywania. SHA-256 pliku docelowego po
kopii jest rowny SHA-256 kanonicznego source:

| Plik natywny NWN | Bytes | SHA-256 |
| --- | ---: | --- |
| `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_h1full42.mod` | 15,163 | `60dd43ac8d0e2d121e841b53742824e42858b496b37861804b04cbc145392ac2` |
| `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_h1full42.hak` | 20,022,503 | `ad70ee816414cef00a014c7a51ab6c8675a972150df33afcd5f364b94c7c1a3b` |
| `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_n1static.mod` | 13,183 | `170523f89b70435eaf9899fa80bc519f54afb8e68bf3ab33d4e04c0f04a7460c` |
| `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_n1static.hak` | 19,598,465 | `fa430e5c0f5e75d56c4f8690c5c5b479b6a14602941f898838624a6fba9aeb53` |

## Werdykt

H1 przechodzi aktualny pelny, 42-state proceduralny pipeline creature offline.
N1 przechodzi wylacznie statyczny rigid pipeline. Nie ma skina/wag ani
alternatywnej animowanej hierarchii segmentow, a jego output states sa
`static-identity`. Nie wolno przedstawic wyniku N1 jako dzialajacego
animowanego quadrupeda. Oba MOD+HAK sa zmaterializowane, zweryfikowane i
zainstalowane. H1 ma status `ready_for_owner_proof`; N1 ma waski status
`ready_for_owner_static_visibility_proof` i `animated_creature_eligible=false`.

Widocznosc obu modeli pozostaje nieprzetestowana:

```text
modelVisibility = not_tested
proofCompleteness = missing
```

Offline build/readback nie jest wizualnym proofem Toolset/NWN.
