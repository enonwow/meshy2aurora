# Implementacja: materiał creature i kontrakt ruchu `c_wolf` V2

Data: 2026-08-21

Status: `IMPLEMENTATION_COMPLETE_OFFLINE / NOT_READY_FOR_OWNER_PROOF`

Dokument zamyka część implementacyjną planu
`plan-implementacji-material-i-pelna-zgodnosc-c-wolf-2026-08-20.md` bez
tworzenia kolejnego lineage modelu. Nie uruchamiano Toolsetu ani NWN i nie
zmieniono zamrożonych V3/V4.

## Wynik

Powstała jedna fail-closed trasa Core dla statycznego GLB creature, która
łączy:

- pełny materiał NWN:EE;
- osobną warstwę nośną kontrolerów supermodelu;
- osobne, należące do produktu węzły korekcyjne skinningu;
- read-only oracle dokładnego supermodelu;
- próbkowanie wymaganych klipów i blokujące bramki deformacji;
- deterministyczny MDL oraz HAK z pełnym readbackiem zasobów.

Publiczne wejścia to:

- `build_reference_supermodel_creature_package_v1`;
- `bind_static_mesh_for_inherited_supermodel_motion_v1`;
- `package_reference_supermodel_creature_hak_v1`.

Stara trasa `ReferenceSupermodelContractV1` ma teraz jawną klasyfikację
`TOPOLOGY_ONLY`. Sama zgodność nazw i rodziców nie może ustawić
`motionCompatible=true`.

## 1. Naprawa regresji materiałowej

Wspólny builder wykonuje pełny przepływ:

1. ingest GLB i zachowanie powiązań source material/image/texture;
2. `compile_gltf_materials_v1(..., NwnEeMtr)`;
3. bake metallic/roughness do specular/gloss;
4. pakowanie diffuse, normal, specular/gloss jako TGA + TXI;
5. generowanie MTR z `normalandspecmapped` i `twosided`;
6. zapewnienie tangentów;
7. zapis oraz material-extension binary MDL;
8. semantic readback MDL;
9. pakowanie całego zestawu do HAK-a;
10. ponowne parsowanie HAK-a i porównanie każdego zasobu bajt po bajcie.

`CreatureMaterialSemanticReportV1` rozlicza każdy source material, output slot
i każdy source image. Raport zawiera dokładne wyjściowe resrefy diffuse,
normal, specular i MTR oraz dyspozycję obrazu `PRESERVED`, `BAKED` albo
`DROPPED_WITH_WARNING`. Status `READY` wymaga obecności wszystkich zależności.

Syntetyczna fixture ma jeden materiał, trzy obrazy, normal mapę,
metallic/roughness i `doubleSided=true`. Test regresyjny usuwa MTR z pakietu i
otrzymuje `BLOCKED`; dawny pakiet V3 typu „MDL + jedna TGA + 2DA” nie może
przejść nowej trasy.

## 2. Rzeczywisty kontrakt dziedziczonego ruchu

`ReferenceSupermodelMotionContractV2` zapisuje:

- provenance read-only i brak kopiowania payloadu;
- classification oraz animation scale;
- uporządkowaną topologię i clean-room neutral carrier profile;
- wymagane kanały position/orientation/scale;
- role anatomiczne i osie stawów;
- osiem wymaganych klipów;
- sześć wymaganych eventów;
- wersjonowane tolerancje bind, deformacji, szwów, kontaktu łap i skoku
  początkowego.

Dla każdego target bone builder tworzy carrier oraz `m2a_corr_XX`. Korekta
spełnia w neutralnej pozie:

`worldCarrier * localCorrection = worldTarget`.

Wagi są przepinane wyłącznie na węzły korekcyjne. Readback potwierdza carrier,
correction, inverse bind i skin. Dopiero potem można obliczyć
`motionCompatible`.

## 3. Bramki klipów i powierzchni

Oracle próbuje wszystkie keyframes, środki każdego przedziału oraz punkty
25/50/75% dla:

- `cpause1`;
- `cwalk`;
- `crun`;
- `ca1slashl`;
- `ca1slashr`;
- `ca1stab`;
- `ckdbck`;
- `cdead`.

Raport per clip zawiera bounds, maksymalne przemieszczenie, root-motion
centroidu, edge ratio, triangle area ratio, odwrócenia normalnych oraz graf
bliskości pomiędzy rozłącznymi komponentami.

Dodatkowa bramka anatomiczna:

- rozwiązuje wagi do czterech ról paw z węzłów korekcyjnych;
- wymaga czterech niepustych klastrów;
- blokuje przejście łapy na przeciwną stronę modelu;
- dla `cpause1` wymaga kontaktu każdej łapy z ground plane do 1% przekątnej
  bind bounds;
- na początku każdego klipu blokuje skok anchorów powyżej 5% przekątnej.

Test syntetyczny potwierdza poprawny grounded idle i osobno odrzuca podniesioną
łapę, zamianę strony oraz zbyt duży start jump.

## 4. Dokładny read-only audit `c_wolf`

Oracle odczytał lokalny retail `c_wolf` in-place przez dokładny
`nwn_base.key`. Nie wypakowano ani nie zapisano retailowego payloadu.

Wynik:

```json
{
  "status": "PASS",
  "contractSha256": "90825064b91ced8bb508485dff23196c8db7bd647176b988b44a6004c9295f3a",
  "requiredClips": 8,
  "requiredEvents": 6
}
```

Audit potwierdza inventory klipów, eventów i wymaganych kontrolerów. Nie jest
wizualnym proofem jakości konkretnego psa.

## 5. WASM i Studio

WASM wystawia:

- capability
  `TOPOLOGY_BIND_SKIN_MOTION_V2_WITH_CARRIER_CORRECTION_AND_MATERIAL_LEDGER`;
- `buildCWolfMotionContractV2Json`;
- `buildMotionCorrectedRigV1Json`.

Worker Studio obsługuje osobne requesty budowy kontraktu i korekty. Odpowiedź
korekty może zgłosić `BIND_POSE_COMPATIBLE`, ale nie zgłasza pełnej zgodności
ruchu bez oracle. Dedykowany panel prezentacyjny Studio zostanie podłączony
dopiero do dozwolonej materializacji produktu; transport i typy są gotowe i
przetestowane.

## 6. Weryfikacja

Zaliczone:

- canonical workspace gate;
- canonical Meshy asset-layout gate;
- `cargo fmt --all -- --check`;
- `cargo clippy -p m2a-core -p m2a-wasm --all-targets -- -D warnings`;
- `cargo test --workspace`;
- `cargo check -p m2a-wasm --target wasm32-unknown-unknown`;
- `wasm-pack build` dla Node i Web;
- Node boundary: `PASS`;
- Studio typecheck;
- Studio: 51 plików, 303/303 testy;
- worker-WASM integration: 3 pliki, 15 testów zaliczonych, 2 pominięte przez
  jawne warunki środowiskowe;
- read-only audit dokładnego `c_wolf`: `PASS`.

## 7. Nienaruszalność V3/V4

Po implementacji ponownie odczytano SHA-256. Wszystkie wartości są identyczne
z zamrożonymi dokumentami:

| Lineage | MOD | HAK | MDL | TGA |
|---|---|---|---|---|
| V3 | `109dd1db637b1c744fd1a18b6daacfe5fc479e9cb1e04a5a7c44442a7f454c8e` | `60f5d7fa072b4375e8be95a7f995f0674d53a6847e067aa56ac3bea51e05e7ff` | `c12087f62201fdc13c021e158194e58378912acb529508e0aca42e7b8bdba978` | `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1` |
| V4 | `6a7b321c963d2425b371cc7f5132aea6e40c410cf6f9d88a83b3dc8d078fdfc6` | `fb772efc667cfadf4729c318fd7d7e83454ce64fc6f9f300e6a854df7beb607b` | `fa369a7259565efb073de13d5fa517acdc4d50776d4cc9769c0454f4ba493a69` | `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1` |

## Granica zakończenia

Implementacja offline jest zakończona. Nie utworzono V5, nowego resrefu,
HAK-a ani MOD-a, ponieważ byłaby to nowa iteracja modelu. Aktualna bramka nie
pozwala wyprowadzić nowego lineage wyłącznie z udanego buildu albo z nowego
offline oracle.

Stan nie brzmi `ready_for_owner_proof`, ponieważ nie istnieje nowy dozwolony,
zamrożony i zainstalowany kandydat. Następny etap po dopuszczeniu iteracji to
jednorazowe uruchomienie pełnego buildera na `source-p300k.glb`, zamrożenie
dokładnego MDL/TGA/TXI/MTR/2DA/HAK/MOD, instalacja no-replace i handoff do
właściciela. Visual quality w Toolset/NWN nadal rozstrzyga właściciel.
