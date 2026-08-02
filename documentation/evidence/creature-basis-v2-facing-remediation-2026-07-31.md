# Creature Basis V2 — poprawne ustawienie humanoida przodem

Data: 2026-07-31
Status implementacji: `OFFLINE_COMPLETE / OWNER_VISUAL_PROOF_REQUIRED`

## Objaw i zakres

Humanoidalne modele Creature przechodzące historyczny kontrakt Profile A były
widoczne w Aurora Toolset i NWN, ale stały bokiem względem oczekiwanego kierunku
obiektu. Problem dotyczył wspólnej transformacji geometrii, szkieletu i
animacji, a nie pojedynczego modelu ani animacji Meshy.

Placeable oraz statyczna ścieżka M0 nie otrzymują tej zmiany. Nie mają
kontraktu humanoidalnego kierunku patrzenia i zachowują zamrożony układ legacy.

## Fakty i diagnoza

- Fakt z formatu źródłowego: wejście glTF używa osi `+Y` jako góry.
- Fakt z badanego źródła humanoidalnego: przód modelu jest zgodny z `+Z`.
- Fakt z retail/native Creature: oczekiwany przód Aurory jest zgodny z `-Y`,
  przy osi `+Z` jako górze.
- Fakt z implementacji legacy: macierz `[x, y, z] -> [x, z, y]` mapowała
  źródłowe `+Z` na aurorowe `+Y`. Była odbiciem o wyznaczniku `-1`, a nie
  właściwym obrotem.
- Wniosek implementacyjny: wspólną naprawą dla humanoidalnego Creature jest
  właściwy obrót `[x, y, z] -> [x, -z, y]`, czyli `+Z -> -Y`, `+Y -> +Z`,
  o wyznaczniku `+1`.

## Implementacja

Wprowadzono wersjonowany kontrakt `Creature Basis V2`:

- jawna polityka osi
  `GltfYUpPositiveZForwardToAuroraZUpNegativeYForwardV2`;
- macierz `B` i jej rzeczywista odwrotność `B^-1`;
- geometria, normalne, tangenty i bounds używają tej samej macierzy;
- lokalne bind pose oraz klucze rotacji są konwertowane przez `B * R * B^-1`;
- translacje animacji są konwertowane przez `B * v`;
- kolejność indeksów i handedness tangentów wynikają z wyznacznika pełnej
  transformacji, bez ręcznego podwójnego odwracania;
- profil H1 V2 ma identyfikator `meshy-h1-derived-user-rig-v2`;
- aktualne produktowe ścieżki Creature są fail-closed, jeżeli raport nie
  potwierdza mapowania `GLTF_POSITIVE_Z_TO_AURORA_NEGATIVE_Y`, wyznacznika
  `+1` i statusu `CREATURE_BASIS_V2_RESOLVED`;
- Studio pokazuje nierozwiązany historyczny facing jako `FAIL`, a poprawny
  kontrakt V2 jako oczekujący wyłącznie na proof właścicielski.

Stare, zamrożone API i lineage zachowują politykę legacy. Test kandydata H2
r42 potwierdza identyczność bajtową; dzięki temu naprawa nie zmienia
historycznych proofów ani ich hashy.

## Kryteria i dowody offline

Spełnione kryteria:

1. Źródłowy przód `+Z` mapuje się dokładnie na aurorowe `-Y`.
2. Źródłowa góra `+Y` mapuje się dokładnie na aurorowe `+Z`.
3. Transformacja ma wyznacznik `+1`, więc nie odbija modelu.
4. `B * B^-1` jest macierzą jednostkową, a bind pose i animacje używają tego
   samego kontraktu.
5. Geometria, UV, materiały i liczba trójkątów nie są zmieniane przez samą
   naprawę orientacji.
6. Ustawienie sprzecznej polityki basis/winding jest odrzucane.
7. Zamrożony kandydat legacy H2 r42 zachowuje dokładną tożsamość bajtową.
8. Studio nie przedstawia nierozwiązanego facing jako gotowego wyniku.

Regresja na rzeczywistym źródle
`sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`
potwierdziła:

- SHA-256 wejścia:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`;
- 297 190 trójkątów przed i po konwersji;
- 9 zachowanych klipów źródłowych;
- brak mutacji bajtów źródła;
- raport `CREATURE_BASIS_V2_RESOLVED`, `+Z -> -Y`, determinant `+1`.

Końcowe bramki offline:

- `cargo fmt --all -- --check` — PASS;
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS;
- `cargo test -p m2a-core --test profile_a` — 44 PASS;
- `cargo test -p m2a-core --test model_pipeline` — 29 PASS, 1 świadomie
  pominięty test wymagający lokalnego runtime witness;
- `cargo test -p m2a-wasm --lib` — 34 PASS;
- realny test Fogbound Basis V2 — PASS;
- zamrożony `h2_r42_visibility_candidate` — PASS;
- Studio `vitest` — 38 plików, 236 testów PASS;
- Studio `tsc -b --pretty false` — PASS.

Pełny audyt stabilizacji odłączonych akcesoriów dla tego źródła jest osobnym,
kosztownym etapem i w dotychczasowym przebiegu przekroczył 10 minut. Nie
podważa to testu orientacji, ale pozostaje ryzykiem wydajnościowym pipeline'u
dla modeli bliskich 300 tys. trójkątów.

## Granica ukończenia

Testy offline mogą udowodnić spójność osi, geometrii, szkieletu, animacji i
raportu. Ostateczne potwierdzenie, że obiekt jest zwrócony przodem w Aurora
Toolset i NWN, pozostaje proofem wizualnym właściciela.

Nie utworzono nowego MOD/HAK. Aktualny dokładny kandydat jest widoczny, a
bramka iteracji dopuszcza nową materializację dopiero po świeżym wyniku
`modelVisibility=not_visible`. Następny bezpieczny krok to materializacja
dopiero w dozwolonym lineage i właścicielski proof dokładnie tego artefaktu.
