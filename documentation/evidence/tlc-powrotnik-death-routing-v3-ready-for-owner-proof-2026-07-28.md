# TLC Powrotnik death-routing V3

Data: `2026-07-28`

Status: `OWNER_RUNTIME_BEHAVIOR_FAILED`

## Dokładny handoff

1. Test-module filename: `tlcpowdemo3.mod`
2. Toolset module name: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Area resref: `tlcpowarea3`. Jedyna creature ma nazwę
`Meshy procedural humanoid`, template resref `tlcpowutc3` i Appearance
`15100`. Stoi w punkcie `(10.0, 14.5, 0.0)`, bezpośrednio przed wejściem
gracza `(10.0, 10.0, 0.0)`.

Agent nie uruchamiał ani nie kontrolował Toolsetu lub NWN. Exact V3 ma:

| Środowisko | modelVisibility | proofCompleteness |
| --- | --- | --- |
| Aurora Toolset | `not_tested` | `missing` |
| NWN runtime | `visible` | `verified` |

Właściciel uruchomił exact V3 i zgłosił:

- `deathSequence=failed`;
- `deathEntryJump=failed`;
- wynik: przy umieraniu nadal występuje ten sam widoczny odskok.

To jest świeży, candidate-bound wynik exact V3. Widoczność modelu pozostaje
monotonicznie `visible`; błąd dotyczy zachowania death-family.

## Audyt przyczyny po owner proof

V3 naprawiło granicę `ckdbckdie -> cdead`, ale nie wcześniejszą granicę
`ckdbck -> ckdbckdie`. Exact V1, V2 i V3 zachowały ten sam ruchomy klip
`ckdbck` z Meshy action `187 Knock_Down`:

`b1191ae2d30845d9d5716e57559d220779b105c25104a076f64e992131313364`.

Exact binary readback V3 wykazał:

| Klip | Długość | Zmieniające kontrolery | Rola |
| --- | ---: | ---: | --- |
| `ckdbck` | `2.5333333 s` | `48` | Meshy `Knock_Down` |
| `ckdbckdie` | `3.0 s` | `48` | Meshy `Dead` |
| `cdead` | `0.033333335 s` | `0` | terminalny hold |

Koniec `ckdbck` i początek `ckdbckdie` nie opisują tej samej pozy:

- różnica pozycji `Hips`: `2.837246 m`;
- różnica rotacji `Hips`: `77.989°`;
- największa różnica rotacji kości: `84.386°` (`LeftArm`);
- `Hips` kończy `ckdbck` w
  `(0.37408733, -2.7337651, 0.20301229)`, natomiast Meshy `Dead` zaczyna się
  w `(-0.030768877, -0.00089268386, 0.84912777)`.

Wartości te pochodzą już z exact output MDL. Niezależny odczyt źródłowych GLB
potwierdza, że pipeline zachował dwa wzajemnie niezgodne originy osobnych zadań
Meshy:

- `meshy-knockdown-action-187.glb`, pierwsze `Hips`:
  `(30.1869717, 108.0051422, -175.5581360)` cm;
- `meshy-dead-action-8.glb`, pierwsze `Hips`:
  `(-3.0768878, 84.9127731, -0.0892676)` cm.

Dla porównania exact retail `c_squirrel` ma na granicy
`ckdbck -> ckdbckdie` różnicę pozycji `0` i maksymalną różnicę rotacji tylko
`0.051°`; analogicznie ciągła jest granica `ckdbckdie -> cdead`.

Przyczyna jest zatem potwierdzona: pipeline traktował trzy nazwy jako
niezależne klipy, ale nie egzekwował ciągłości sekwencji death-family. NWN
odtwarza ruchome `ckdbck` przed następnym stanem śmierci, więc model kończy
Meshy `Knock_Down`, teleportuje się o prawie trzy metry do pierwszej pozy Meshy
`Dead` i dopiero wtedy odtwarza zaakceptowany zgon. Zmiana samego
`cdead <-> ckdbckdie` nie mogła usunąć tego odskoku.

Minimalna następna delta pipeline:

1. jeden zaakceptowany Meshy `Dead` ma być jedynym ruchomym upadkiem
   death-family;
2. kolejne stany tej rodziny mają zaczynać się od dokładnej pozy końcowej
   poprzednika;
3. pipeline ma odrzucać artefakt, gdy granice `ckdbck -> ckdbckdie` albo
   `ckdbckdie -> cdead` mają nieciągłość pozycji lub rotacji;
4. osobny Meshy `Knock_Down` nie może być bezwarunkowo sklejony przed Meshy
   `Dead` w sekwencji zgonu;
5. V4 nie może powstać, dopóki ta delta nie ma testu negatywnego i pozytywnego.

## Autoryzacja i delta

V3 zostało bezpośrednio autoryzowane przez właściciela po exact V2
`deathSequence=failed`. Zakres decyzji obejmuje jedną korektę mapowania stanów
zgonu mimo zachowanego wyniku V2 `modelVisibility=visible`.

Nie uruchomiono żadnego zadania Meshy i nie zmieniono source GLB. Koszt:
`0` kredytów. Exact input:

- `sample-3d/tlc-powrotnik-h1-p20k-v1/source-budget20000.glb`;
- `10,696,908` bytes;
- SHA-256
  `43bba1823c4cedb1f5a50da384bf4d65856fd19ccd160f3c6a5dd3c90c0849f1`;
- `19,892` trójkąty, limit `20,000`;
- `10` jawnych klipów Meshy.

Jedyna zmiana zachowania to `MESHY_DEAD_TO_CKDBCKDIE_V1`:

1. pełny trzysekundowy Meshy `Dead` jest zachowany jako jednorazowe
   `ckdbckdie`;
2. `cdead` jest nieruchomym hold'em `1/30 s` z ostatniej pozy tego samego
   klipu;
3. nie powstaje drugi proceduralny ruch zgonu;
4. SkinMesh oracle probkuje `ckdbckdie`, nie `cdead`.

## Offline readback

- pełne namespace `42/42`;
- behavior candidate: `eligible`;
- `activeMotionComplete=true`;
- events: `23/23`;
- SkinMesh animation conformance: `PASS`;
- `ckdbckdie`: `48` decoded controllers, `48` changing controllers,
  `48` terminal-pose controllers;
- motion SHA-256 `ckdbckdie`:
  `ac16dfdf874e5e65bb3eebb796b4168714df42b510a10f1b80b35fbc6d8228c1`;
- powyższy motion hash jest identyczny z exact Meshy `Dead`, który V2 trzymało
  pod błędną nazwą `cdead`;
- `ckdbckdie` SkinMesh: `22,413` poruszonych wierzchołków, maksymalne
  przemieszczenie `2.4093707`, nierigid deformation `PASS`;
- `cdead`: `48` decoded controllers, `0` changing controllers.

Testy poprzedzające materializację:

- `cargo test -p m2a-core --lib -- --test-threads=1`: `67 passed`,
  `1 ignored`;
- `cargo test -p m2a-core --test mdl_writer -- --test-threads=1`:
  `43 passed`;
- `cargo test -p m2a-core --test model_pipeline -- --test-threads=1`:
  `27 passed`, `1 ignored`;
- `cargo fmt --all -- --check`: `PASS`;
- canonical workspace i Meshy asset layout: `PASS`.

## Exact artefakty i instalacja

| Artefakt | Kanoniczne źródło | Natywna instalacja | Bytes | SHA-256 |
| --- | --- | --- | ---: | --- |
| MOD | `proof-output/tlc-powrotnik-death-routing-v3/generated/tlcpowdemo3.mod` | `C:\Users\enonw\Documents\Neverwinter Nights\modules\tlcpowdemo3.mod` | 15,166 | `d841312e000a137e147cbf6af567dd7082951457d497db272a60739c656444b4` |
| HAK | `proof-output/tlc-powrotnik-death-routing-v3/generated/tlcpowhak3.hak` | `C:\Users\enonw\Documents\Neverwinter Nights\hak\tlcpowhak3.hak` | 22,187,923 | `ea49c8cc4b8f36facc9ae81b3193b6dd2e9589e97c58a52fbbe7a77d184f0580` |
| MDL | `proof-output/tlc-powrotnik-death-routing-v3/generated/tlcpow_m3.mdl` | HAK resource `tlcpow_m3:2002` | 2,703,400 | `2ab71546cd3af17866af090ee0af7c8b56eba16e7e307bcfaab9d84cb48f1ba2` |
| TGA | `proof-output/tlc-powrotnik-death-routing-v3/generated/tlcpow_t3.tga` | HAK resource `tlcpow_t3:3` | 12,582,956 | `1ea9816d4ba4c5ccc77856516c78af3cb76eeea0c2306708633bbe56f53debe5` |
| appearance.2da | `proof-output/tlc-powrotnik-death-routing-v3/generated/appearance.2da` | HAK resource `appearance:2017` | 6,901,311 | `b3d77572b7e67322079047f4e448d5f9ea8f0a4a9444c40340f23a734cd367d2` |

Cele MOD i HAK były nieobecne przed kopiowaniem. Po instalacji oba natywne
pliki zostały ponownie zahashowane i są byte-for-byte identyczne z
kanonicznymi źródłami.

Raporty:

- `proof-output/tlc-powrotnik-death-routing-v3/reports/summary.json`;
- `proof-output/tlc-powrotnik-death-routing-v3/reports/materialization-report.json`;
- `proof-output/tlc-powrotnik-death-routing-v3/reports/materialization-manifest.json`,
  SHA-256
  `67b1efcfe13c1d3cd1de31d95c3785953a568aed1f15bc27bfbefb02836cb839`.

## Owner proof

Otwórz exact `tlcpowdemo3.mod`, następnie Area
`Meshy2Aurora M0 binary vertical-slice area`. W NWN zabij jedyną creature i
sprawdź:

1. czy zgon odtwarza dokładnie raz pełny klip Meshy pokazany wcześniej w
   offline preview;
2. czy przed nim ani po nim nie pojawia się drugi proceduralny upadek;
3. czy po zakończeniu model pozostaje nieruchomo w ostatniej pozycji zwłok;
4. czy ruch zgonu nie zapętla się w stanie `cdead`.
