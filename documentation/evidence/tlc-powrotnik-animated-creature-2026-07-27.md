# TLC Powrotnik — animowany creature z Meshy

Data: `2026-07-27`

Status: `READY_FOR_OWNER_PROOF`

## Dokładny handoff

1. Test-module filename: `tlcpowdemo1.mod`
2. Toolset module name: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Area resref: `tlcpowarea1`. Creature `Meshy procedural humanoid`
(`tlcpowutc1`, Appearance `15100`) stoi w punkcie `(10.0, 14.5, 0.0)`,
bezpośrednio przed wejściem gracza `(10.0, 10.0, 0.0)`.

Zgodnie z decyzją projektu końcowy proof wizualny jest własnością właściciela.
Agent nie uruchamiał i nie przejmował Toolsetu ani NWN. Obie osie pozostają:

| Środowisko | modelVisibility | proofCompleteness |
| --- | --- | --- |
| Aurora Toolset | `not_tested` | `missing` |
| NWN runtime | `not_tested` | `missing` |

Nie jest to twierdzenie, że model jest już wizualnie potwierdzony. Kandydat
jest zamrożony, zweryfikowany offline, zainstalowany i gotowy do proofu
właściciela.

## Zainstalowane exact artefakty

| Artefakt | Kanoniczne źródło | Natywne miejsce instalacji | Bytes | SHA-256 |
| --- | --- | --- | ---: | --- |
| MOD | `proof-output/tlc-powrotnik-h1-p20k-v1/generated/tlcpowdemo1.mod` | `C:\Users\enonw\Documents\Neverwinter Nights\modules\tlcpowdemo1.mod` | 15,166 | `df4cf42200727c482c89e5698faa5bed9ea575ba51dc16755cfc5ac5a2122d53` |
| HAK | `proof-output/tlc-powrotnik-h1-p20k-v1/generated/tlcpowhak1.hak` | `C:\Users\enonw\Documents\Neverwinter Nights\hak\tlcpowhak1.hak` | 22,190,515 | `ba27904b51e815e04032e83c8c1114ccfec0db21ba13af665da602db4640637e` |

Oba cele były nieobecne przed kopiowaniem. Hash po instalacji jest
byte-for-byte identyczny z kanonicznym źródłem.

Pozostała exact identity:

- model resref `tlcpow_m1`, MDL `2,705,992` bytes,
  SHA-256 `78690cf9fbf2105961a0f24efbc9253792674e9372304b8387e3ea5722cf5662`;
- texture resref `tlcpow_t1`, TGA `12,582,956` bytes,
  SHA-256 `1ea9816d4ba4c5ccc77856516c78af3cb76eeea0c2306708633bbe56f53debe5`;
- HAK resref `tlcpowhak1`;
- module resref `tlcpowdemo1`;
- fixture tag `m2a_procedural_creature`.

## Meshy source i koszt

Kanoniczny asset:
`sample-3d/tlc-powrotnik-h1-p20k-v1/manifest.yaml`.

- primary concept SHA-256:
  `f915a3bafc2b7546edf01d2a27beee18a88d965e034ccdc12f6f45419fe6dca4`;
- model task: `019fa586-f172-763e-81d3-0cd3ace147f0`;
- rig task: `019fa589-6a12-77ae-9441-edd01a3ef0dc`;
- balance: `964 -> 899`;
- exact credit spend: `65`, poniżej limitu właściciela `250`;
- model: `30`, rig: `5`, dziesięć akcji po `3`.

Pierwszy run napotkał wyłącznie błąd pobrania podpisanego URL dla trzeciej
gotowej animacji. Recovery odczytał exact istniejący model, rig i ukończone
taski, a następnie utworzył tylko siedem brakujących akcji. Nie wygenerowano
drugiego modelu ani drugiego rigu.

| Meshy action | Task ID | NWN clip |
| --- | --- | --- |
| `0 Idle` | `019fa589-ea49-7723-b758-125694a59167` | `cpause1` |
| `30 Casual_Walk` | `019fa58a-2091-729b-abe4-ac3905e6e29d` | `cwalk` |
| `16 RunFast` | `019fa58a-57f0-7650-bb81-461171779f97` | `crun` |
| `198 Punch_Combo` | `019fa58d-779f-7397-bee2-fa1396bdbb52` | `ca1slashl` |
| `193 Left_Hook_from_Guard` | `019fa58d-ae4e-78f2-9004-192cdd6b241a` | `ca1slashr` |
| `178 Hit_Reaction` | `019fa58d-d977-78fe-a2a4-a40d1d65f78f` | `cdamagel` |
| `187 Knock_Down` | `019fa58e-04a0-7906-8f94-cd0f26c90729` | `ckdbck` |
| `8 Dead` | `019fa58e-3d1e-73c1-b3e2-d67c20dc4204` | `cdead` |
| `3 Arise` | `019fa58e-81d7-7810-8f23-2ae11b433b44` | `cguptokdb` |
| `2 Alert` | `019fa58e-aca3-792d-b918-fcc3801e571b` | `ctaunt` |

Dziesięć GLB tego samego rigu połączono bez przepisywania ruchu. Combined
source ma SHA-256
`d44e276d06de5b0a4b9343a3c3e995aa22b6a61e07d943dd76f0b8f7471d7ec0`.

## Limit geometrii i materializacja

Meshy zwrócił `20,535` trójkątów mimo targetu `20,000`. Lokalny, zachowujący
skin i animacje etap meshoptimizer zredukował ten sam source lineage do:

- `19,892` trójkątów;
- `22,413` wierzchołków;
- `1` skin;
- wszystkich `10` animacji;
- SHA-256 wejścia pipeline:
  `43bba1823c4cedb1f5a50da384bf4d65856fd19ccd160f3c6a5dd3c90c0849f1`.

Materializacja wyprodukowała pełny kontrakt `42/42` native creature:

- `10` jawnych klipów Meshy zachowanych bez zastąpienia;
- `32` brakujące stany uzupełnione proceduralnie;
- `0` aliases;
- `23/23` wymagane events;
- walk i run są różnymi ruchami;
- semantic MOD readback: `PASS`;
- HAK zawiera exact `appearance.2da`, model i teksturę;
- deformation conformance: `PASS`, wszystkie `22,413` wierzchołki poruszyły
  się w próbkach, m.in. `cwalk`, `crun`, `ca1slashl`, `cdamagel`,
  `ckdbckdie`.

Raporty offline:

- `proof-output/tlc-powrotnik-h1-p20k-v1/reports/summary.json`;
- `proof-output/tlc-powrotnik-h1-p20k-v1/reports/materialization-report.json`;
- `proof-output/tlc-powrotnik-h1-p20k-v1/reports/materialization-manifest.json`.

## Oczekiwany proof właściciela

Otwórz exact `tlcpowdemo1.mod`, następnie Area
`Meshy2Aurora M0 binary vertical-slice area`. Najpierw potwierdź, że exact
creature jest widoczny w Toolset, potem uruchom ten sam MOD+HAK lineage w NWN
i potwierdź widoczność oraz ruch. Dopiero raport właściciela może zmienić osie
`not_tested/missing` na wynik wizualny.

## Wynik właściciela — 2026-07-28

Właściciel uruchomił exact demo i zgłosił widoczny model oraz widoczną
sekwencję śmierci, która wygląda jak złożenie dwóch lub trzech osobnych
animacji. Końcowa część została oceniona jako poprawna i ma pozostać jedynym
widocznym ruchem zgonu.

Aktualizacja osi dla exact `tlcpowdemo1.mod` /
`df4cf42200727c482c89e5698faa5bed9ea575ba51dc16755cfc5ac5a2122d53`:

| Środowisko | modelVisibility | proofCompleteness |
| --- | --- | --- |
| Aurora Toolset | `not_tested` | `missing` |
| NWN runtime | `visible` | `verified` |

Osobny wynik zachowania:

- `deathSequence = failed`;
- źródłowy Meshy action `8 Dead` jest jednym klipem długości `3.0 s`,
  zawierającym `90` próbek;
- klip był przypisany do `cdead`, ale brakujące `ckdbckdie` otrzymało osobny
  ruch proceduralny;
- NWN przechodzi przez death-transition i stan `cdead`, więc dwa ruchome
  payloady dają obserwowane złożenie, a poprzedzający damage/knockdown może
  wyglądać jak trzeci etap.

Właściciel bezpośrednio polecił przygotowanie nowego demo zachowania. Minimalny
delta nie generuje nowego modelu Meshy ani rigu i nie zmienia geometrii,
tekstury lub dziesięciu pobranych klipów:

1. zachować exact pełny ruch Meshy wyłącznie w `cdead`;
2. gdy `cdead` jest jawny, a `ckdbckdie` brak, utworzyć dla `ckdbckdie`
   minimalny nieruchomy bridge z pierwszej pozy `cdead`, aby zachować pełny
   namespace bez drugiego widocznego upadku;
3. dostosować behavior oracle do jawnego wariantu
   `CDEAD_OWNS_DEATH_MOTION_V1`;
4. nadać demo świeżą immutable identity i wymagać nowego owner proofu.

### Amendment 2026-07-28 po owner proof V2

Exact `tlcpowdemo2.mod` zachowało ten sam błędny efekt mimo nieruchomego
`ckdbckdie`, więc powyższa diagnoza i reguła
`CDEAD_OWNS_DEATH_MOTION_V1` zostały odrzucone. Właściciel obejrzał osobny
offline preview wszystkich `90` klatek Meshy action `8 Dead` i zaakceptował
cały klip jako właściwą, pojedynczą animację zgonu.

Potwierdzona korekta `MESHY_DEAD_TO_CKDBCKDIE_V1` umieszcza pełny ruch Meshy
w jednorazowym `ckdbckdie`, a `cdead` tworzy wyłącznie z ostatniej pozy jako
nieruchomy stan zwłok. Pipeline nie przycina klipu i nie dodaje drugiego
proceduralnego ruchu zgonu. Szczegóły i testy znajdują się w
`tlc-powrotnik-cdead-only-v2-ready-for-owner-proof-2026-07-28.md`.
