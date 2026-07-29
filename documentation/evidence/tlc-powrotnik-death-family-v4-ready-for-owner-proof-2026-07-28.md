# TLC Powrotnik continuous death-family V4

Data: `2026-07-28`

Status: `OWNER_RUNTIME_PROVED`

## Dokładny handoff

1. Test-module filename: `tlcpowdemo4.mod`
2. Toolset module name: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Area resref: `tlcpowarea4`. Jedyna creature ma nazwę
`Meshy procedural humanoid`, template resref `tlcpowutc4` i Appearance
`15100`. Stoi w punkcie `(10.0, 14.5, 0.0)`, bezpośrednio przed wejściem
gracza `(10.0, 10.0, 0.0)`.

Agent nie uruchamiał ani nie kontrolował Toolsetu lub NWN.

| Środowisko | modelVisibility | proofCompleteness |
| --- | --- | --- |
| Aurora Toolset | `not_tested` | `missing` |
| NWN runtime | `visible` | `verified` |

Właściciel uruchomił exact V4 i 2026-07-28 zgłosił: `V4 działa`.
Wynik jest związany z `tlcpowdemo4.mod` i `tlcpowhak4.hak` o hashach
zapisanych niżej. Zamyka to NWN `modelVisibility=visible`,
`deathSequence=passed` oraz `deathEntryJump=passed`. Nie promuje osobnej osi
Aurora Toolset, której właściciel nie zgłosił.

## Podstawa iteracji

Exact V3:

- MOD SHA-256
  `d841312e000a137e147cbf6af567dd7082951457d497db272a60739c656444b4`;
- model był widoczny w NWN;
- `deathSequence=failed`;
- `deathEntryJump=failed`.

Audyt exact V3 wykazał, że niepoprawiona granica
`ckdbck -> ckdbckdie` łączyła Meshy `Knock_Down` z Meshy `Dead` przy różnicy
`Hips=2.837246 m`, `Hips rotation=77.989°` i maksymalnej różnicy rotacji kości
`84.386°`. Exact retail `c_squirrel` ma na tej granicy `0 m` i najwyżej
`0.051°`.

## Minimalna delta V4

Nie uruchomiono nowego zadania Meshy i nie zmieniono źródłowego GLB. Koszt:
`0` kredytów.

1. Pełny Meshy action `8 Dead` jest jedynym ruchomym upadkiem i zajmuje
   `ckdbck`.
2. Osobny Meshy action `187 Knock_Down` nie jest doklejany przed nim w
   death-family.
3. `ckdbckps`, `ckdbckdie` i `cdead` są nieruchomymi holdami `1/30 s`
   wyprowadzonymi z ostatniej pozy tego samego Meshy `Dead`.
4. Behavior oracle blokuje nieciągłość pozycji powyżej `0.001 m` lub rotacji
   powyżej `1°` na granicach `ckdbck -> ckdbckdie` i
   `ckdbckdie -> cdead`.
5. SkinMesh oracle próbkuje ruchomy `ckdbck`.

Exact input:

- `sample-3d/tlc-powrotnik-h1-p20k-v1/source-budget20000.glb`;
- `10,696,908` bytes;
- SHA-256
  `43bba1823c4cedb1f5a50da384bf4d65856fd19ccd160f3c6a5dd3c90c0849f1`;
- `19,892` trójkąty przy limicie `20,000`;
- `10` jawnych klipów Meshy.

## Offline readback exact V4

- pełne namespace `42/42`;
- behavior candidate: `eligible`;
- `deathFamilyBoundaryContinuous=true`;
- events: `23/23`;
- SkinMesh animation conformance: `PASS`;
- `ckdbck`: `3.0 s`, `48` decoded, `48` changing i `48` terminal-pose
  controllers;
- motion SHA-256 `ckdbck`:
  `ac16dfdf874e5e65bb3eebb796b4168714df42b510a10f1b80b35fbc6d8228c1`;
- ten motion hash jest identyczny z pełnym Meshy `Dead` z V2/V3;
- `ckdbckps`, `ckdbckdie` i `cdead`: każdy `0.033333335 s`, `48` decoded,
  `0` changing controllers;
- wspólny motion SHA-256 holdów:
  `f51707d3309a16ec0891c5723f43204fb6e1cba681887b94799db6745a82ca17`;
- `ckdbck -> ckdbckdie`: maksymalna różnica pozycji `0 m`, maksymalna
  różnica rotacji `0.043635°`;
- `ckdbckdie -> cdead`: maksymalna różnica pozycji `0 m`, maksymalna
  różnica rotacji `0.043635°`;
- `ckdbck` SkinMesh: `22,413` poruszonych wierzchołków, maksymalne
  przemieszczenie `2.4093707`, non-rigid deformation `PASS`.

Testy:

- `cargo test -p m2a-core --lib -- --test-threads=1`: `68 passed`,
  `1 ignored`;
- `cargo test -p m2a-core --test mdl_writer -- --test-threads=1`:
  `43 passed`;
- `cargo test -p m2a-core --test model_pipeline -- --test-threads=1`:
  `27 passed`, `1 ignored`.

## Exact artefakty i instalacja

| Artefakt | Kanoniczne źródło | Natywna instalacja | Bytes | SHA-256 |
| --- | --- | --- | ---: | --- |
| MOD | `proof-output/tlc-powrotnik-death-family-v4/generated/tlcpowdemo4.mod` | `C:\Users\enonw\Documents\Neverwinter Nights\modules\tlcpowdemo4.mod` | 15,166 | `5b0fd9713a0644e0d95d45e20d4bc3c971b1adea5b64428f2752dc347461624a` |
| HAK | `proof-output/tlc-powrotnik-death-family-v4/generated/tlcpowhak4.hak` | `C:\Users\enonw\Documents\Neverwinter Nights\hak\tlcpowhak4.hak` | 22,127,907 | `926da4db4f64226f1e2163c05b58547f966bfa8b79c32718f1b2fd5cd6751598` |
| MDL | `proof-output/tlc-powrotnik-death-family-v4/generated/tlcpow_m4.mdl` | HAK resource `tlcpow_m4:2002` | 2,643,384 | `500fcaea1621db5c30213db121a3be6bd28c35c097367fe660b17fde3bf3c18e` |
| TGA | `proof-output/tlc-powrotnik-death-family-v4/generated/tlcpow_t4.tga` | HAK resource `tlcpow_t4:3` | 12,582,956 | `1ea9816d4ba4c5ccc77856516c78af3cb76eeea0c2306708633bbe56f53debe5` |
| appearance.2da | `proof-output/tlc-powrotnik-death-family-v4/generated/appearance.2da` | HAK resource `appearance:2017` | 6,901,311 | `4ed4aaa757c4d64654b0048e857e9fdcd21f1470fc544ee1009fbcab3152a2f0` |

Cele MOD i HAK były nieobecne przed kopiowaniem. Po instalacji oba natywne
pliki zostały ponownie zahashowane i są byte-for-byte identyczne z
kanonicznymi źródłami.

Raporty:

- `proof-output/tlc-powrotnik-death-family-v4/reports/summary.json`;
- `proof-output/tlc-powrotnik-death-family-v4/reports/materialization-report.json`,
  SHA-256
  `d937c5a56bc79c4e7f93335a1aecfcd37acfa22072606bcb9cdcbfed69472f80`;
- `proof-output/tlc-powrotnik-death-family-v4/reports/materialization-manifest.json`,
  SHA-256
  `61a7499d64fd300401e2639ebabdc1d2a6565c85d7981620234c471ad088a7be`.

## Owner proof

Otwórz exact `tlcpowdemo4.mod`, następnie Area
`Meshy2Aurora M0 binary vertical-slice area`. W NWN zabij jedyną creature i
sprawdź:

1. czy śmierć rozpoczyna od razu pełny Meshy `Dead`, bez wcześniejszego
   Meshy `Knock_Down`;
2. czy zniknął odskok/teleport między dwoma upadkami;
3. czy Meshy `Dead` odtwarza się dokładnie raz;
4. czy po zakończeniu model pozostaje nieruchomo w ostatniej pozycji zwłok.
