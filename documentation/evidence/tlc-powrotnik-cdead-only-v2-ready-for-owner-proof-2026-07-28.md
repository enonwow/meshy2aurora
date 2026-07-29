# TLC Powrotnik cdead-only V2

Data: `2026-07-28`

Status: `OWNER_RUNTIME_BEHAVIOR_FAILED`

## Dokładny handoff

1. Test-module filename: `tlcpowdemo2.mod`
2. Toolset module name: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Area resref: `tlcpowarea2`. Creature `Meshy procedural humanoid`
(`tlcpowutc2`, Appearance `15100`) stoi bezpośrednio przed wejściem gracza.

Agent nie uruchamiał Toolsetu ani NWN. Nowy kandydat ma:

| Środowisko | modelVisibility | proofCompleteness |
| --- | --- | --- |
| Aurora Toolset | `not_tested` | `missing` |
| NWN runtime | `visible` | `verified` |

Właściciel przetestował exact `tlcpowdemo2.mod` i zgłosił, że sekwencja zgonu
jest nadal taka sama jak w V1. Wynik jest związany z MOD SHA-256
`d33be09aa6116c5e444c8da12b66b4e42740c776135a919fdd29b6640393b34d`:
`deathSequence=failed`. Widoczność modelu pozostaje potwierdzona; awaria
dotyczy zachowania animacji.

## Powód rewizji

Właściciel potwierdził widoczność poprzedniego exact
`tlcpowdemo1.mod` w NWN, ale zgłosił `deathSequence=failed`: zgon wyglądał jak
złożenie dwóch lub trzech animacji, a końcowa część była poprawna.

Hipoteza użyta do zbudowania V2:

- Meshy action `8 Dead` jest jednym klipem `3.0 s`, `90` próbek;
- jawny klip był zachowany jako `cdead`;
- brakujące `ckdbckdie` dostało osobny proceduralny ruch;
- NWN odtworzył death-transition i późniejszy `cdead`, więc dwa ruchome
  payloady złożyły się w wieloetapowy zgon.

Test właściciela V2 obalił tę hipotezę. Po zastąpieniu `ckdbckdie` nieruchomym
bridge'em zachowanie nie zmieniło się.

## Minimalny delta V2

Nie utworzono żadnego nowego zadania Meshy, modelu, rigu ani animacji. Koszt
tej rewizji to `0` kredytów. Source GLB pozostał exact:

`43bba1823c4cedb1f5a50da384bf4d65856fd19ccd160f3c6a5dd3c90c0849f1`.

Pipeline stosuje `CDEAD_OWNS_DEATH_MOTION_V1`:

- jawny 3-sekundowy `cdead` Meshy pozostaje byte-semantically zachowany i jest
  jedynym widocznym ruchem zgonu;
- `ckdbckdie` zachowuje wymagany wpis namespace i `48` kontrolerów, ale ma
  długość `1/30 s`, `0` zmieniających kontrolerów i trzyma pierwszą pozę
  `cdead`;
- pięcioklipowy SkinMesh oracle probkuje teraz `cdead` jako faktycznego
  właściciela ruchu śmierci;
- jawne source `ckdbckdie`, jeśli kiedyś zostanie dostarczone, nadal ma
  pierwszeństwo i nie jest przepisywane.

Offline readback:

- pełne namespace `42/42`;
- `10` jawnych klipów Meshy i `32` proceduralne uzupełnienia;
- behavior candidate: `eligible`;
- events: `23/23`;
- `ckdbckdie`: `48` decoded controllers, `0` changing controllers;
- `cdead`: `48` decoded controllers, `48` changing controllers i `48`
  terminal-pose controllers;
- motion SHA-256 `cdead` pozostał identyczny z V1:
  `ac16dfdf874e5e65bb3eebb796b4168714df42b510a10f1b80b35fbc6d8228c1`;
- `cdead` SkinMesh: `22,413` poruszonych wierzchołków, maksymalne
  przemieszczenie `2.4093707`, nierigid deformation `PASS`.

## Audyt źródłowego klipu i skorygowana diagnoza

Offline, klatka po klatce, sprawdzono exact
`sample-3d/tlc-powrotnik-h1-p20k-v1/meshy-dead-action-8.glb`.
Animacja `Armature|Dead|baselayer` ma `3.0 s`, `90` klatek i `48`
istotnych kanałów translacji/rotacji. Właściciel obejrzał podgląd wszystkich
`90` klatek exact GLB i potwierdził, że cały ten ruch jest oczekiwaną animacją
zgonu. Hipoteza o potrzebie przycięcia pierwszej sekundy została odrzucona.

Potwierdzona przyczyna leży w mapowaniu stanów NWN:

- V2 umieściło pełny ruch Meshy w `cdead`;
- `ckdbckdie` było nieruchomym bridge'em z pierwszej pozy;
- `ckdbckdie` jest jednorazowym przejściem zgonu, natomiast `cdead` powinno
  utrzymywać końcową pozę zwłok.

Poprawiona implementacja `MESHY_DEAD_TO_CKDBCKDIE_V1` zachowuje cały exact
Meshy `Dead` jako ruchome `ckdbckdie` i wyprowadza nieruchome `cdead` z jego
ostatniej próbki. Dla zgodności naprawia również istniejące źródła, które
historycznie nazwały ruchomy klip `cdead`; nowe mapowania powinny kierować
Meshy `Dead` bezpośrednio do `ckdbckdie`.

Offline TDD:

- ruchome source `cdead` bez `ckdbckdie` jest przenoszone bez zmiany wartości,
  czasu, transition i eventów do `ckdbckdie`;
- jawne source `ckdbckdie` bez `cdead` pozostaje bez zmiany;
- w obu wariantach `cdead` ma `1/30 s`, zero zmieniających kontrolerów i dwie
  identyczne próbki ostatniej pozy `ckdbckdie`;
- behavior oracle odrzuca nieruchome `ckdbckdie` nawet wtedy, gdy `cdead`
  zawiera ruch;
- SkinMesh oracle zawsze probkuje `ckdbckdie` jako właściciela ruchu śmierci;
- `cargo test -p m2a-core --lib -- --test-threads=1`: `67 passed`,
  `1 ignored`;
- `cargo test -p m2a-core --test mdl_writer -- --test-threads=1`:
  `43 passed`;
- `cargo test -p m2a-core --test model_pipeline -- --test-threads=1`:
  `27 passed`, `1 ignored`.

Nie zmaterializowano V3. Obowiązująca bramka iteracji dopuszcza nowy
MOD/HAK/model/resref dopiero po wyniku `modelVisibility=not_visible`, natomiast
owner proof V2 potwierdził `modelVisibility=visible`. Do przygotowania V3
potrzebna jest osobna decyzja właściciela rozszerzająca bramkę o potwierdzoną
awarię zachowania animacji.

### Decyzja właściciela dopuszczająca V3

Data: `2026-07-28`.

Po przedstawieniu powyższej blokady właściciel jawnie polecił wykonać pozostałe
czynności prowadzące do wygenerowania nowego demo. Decyzja rozszerza bramkę
wyłącznie dla korekty potwierdzonego `deathSequence=failed` exact V2; nie
zmienia wyniku `modelVisibility=visible` i nie zezwala na dodatkową iterację
modelu Meshy.

Minimalna autoryzowana delta V3:

- source GLB pozostaje byte-identical:
  `43bba1823c4cedb1f5a50da384bf4d65856fd19ccd160f3c6a5dd3c90c0849f1`;
- pełny Meshy `Dead` przechodzi z ruchomego `cdead` do `ckdbckdie`;
- `cdead` staje się nieruchomą ostatnią pozą tego samego klipu;
- nie zmieniają się geometria, tekstura, rig ani pozostałe dziewięć klipów
  Meshy; koszt Meshy wynosi `0`;
- świeża identity: model `tlcpow_m3`, tekstura `tlcpow_t3`, HAK
  `tlcpowhak3`, MOD `tlcpowdemo3`, Area `tlcpowarea3`, UTC `tlcpowutc3`;
- planowany packet:
  `proof-output/tlc-powrotnik-death-routing-v3`.

## Exact artefakty i instalacja

| Artefakt | Kanoniczne źródło | Natywna instalacja | Bytes | SHA-256 |
| --- | --- | --- | ---: | --- |
| MOD | `proof-output/tlc-powrotnik-cdead-only-v2/generated/tlcpowdemo2.mod` | `C:\Users\enonw\Documents\Neverwinter Nights\modules\tlcpowdemo2.mod` | 15,166 | `d33be09aa6116c5e444c8da12b66b4e42740c776135a919fdd29b6640393b34d` |
| HAK | `proof-output/tlc-powrotnik-cdead-only-v2/generated/tlcpowhak2.hak` | `C:\Users\enonw\Documents\Neverwinter Nights\hak\tlcpowhak2.hak` | 22,187,923 | `bfa5ad166cf919941d29a7be561b85de87e222c04e8d80bf4362b611c4401d23` |
| MDL | `proof-output/tlc-powrotnik-cdead-only-v2/generated/tlcpow_m2.mdl` | HAK resource `tlcpow_m2:2002` | 2,703,400 | `866d820da0dc43cd8a3a9be65940325674614baab901f2da1037e49fb97a5aa4` |
| TGA | `proof-output/tlc-powrotnik-cdead-only-v2/generated/tlcpow_t2.tga` | HAK resource `tlcpow_t2:3` | 12,582,956 | `1ea9816d4ba4c5ccc77856516c78af3cb76eeea0c2306708633bbe56f53debe5` |

MOD i HAK miały nieobecne cele przed kopiowaniem. Hash natywnej instalacji
jest byte-for-byte identyczny z kanonicznym źródłem.

Raporty:

- `proof-output/tlc-powrotnik-cdead-only-v2/reports/summary.json`;
- `proof-output/tlc-powrotnik-cdead-only-v2/reports/materialization-report.json`;
- `proof-output/tlc-powrotnik-cdead-only-v2/reports/materialization-manifest.json`.

## Zakończony owner proof V2

Wynik właściciela: model jest widoczny, ale zgon nadal wygląda jak kilka
następujących po sobie animacji. Runtime behavior proof V2 zakończył się
wynikiem `deathSequence=failed`.
