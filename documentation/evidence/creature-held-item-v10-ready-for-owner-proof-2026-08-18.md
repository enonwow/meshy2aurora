# Creature held item V10 — ready for owner proof — 2026-08-18

1. Plik modułu: `m2aweapdemo10.mod`.
2. Nazwa modułu w Toolset/NWN: `Meshy2Aurora correct held-item demo V10`.
3. Dokładna Area: `Meshy2Aurora correct held-item test area V10`.

Status: `ready_for_owner_proof`.

Agent nie uruchamiał, nie przejmował ani nie kontrolował Aurora Toolset lub NWN.
Dokładne MOD/HAK zostały wygenerowane offline, zainstalowane bez overwrite i
zweryfikowane byte-for-byte. Końcowy werdykt wizualny należy do właściciela.

## Co jest nowe i dlaczego

- V10 używa kanonicznego `tlc-stoneback-brute-h1-p300k-v1/source.glb`, a nie
  eksperymentalnego modelu użytego do odwracania przodu postaci.
- Ten sam Stoneback i ta sama para kontraktów facing/placement były wcześniej
  widoczne w Toolset i NWN jako dokładna linia
  `tlc-stoneback-brute-p300k-geometry-ab-v5`; właściciel opisał ją jako
  „appears correctly and looks very good”.
- Mapowanie facing to `GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y`, determinant `1`;
  instancja stoi w `[10.0, 14.5, 0.0]` z orientacją `[0.0, -1.0]`, dokładnie
  przed wejściem gracza `[10.0, 10.0, 0.0]`.
- Broń nie jest już krótkim `EquippedRes`. GIT zawiera pełną osadzoną instancję
  stockowego bastard sword `nw_wswbs001` w natywnym slocie prawej ręki `16`,
  BaseItem `3`, części `[41, 11, 11]`, oraz feat `44` wymagany do posługiwania
  się bronią. Kontrakt pochodzi z poprawionego przez właściciela
  `m2aweapdemo9.mod`, SHA-256
  `b948538adbddcc096996fde66288f981043481fd5c65f535a9d0e10ae6adab57`.
- Model zawiera prawdziwe zachowane źródłowe klipy `ca1slashl` i `ca1slashr`,
  a profil Creature jest aktywnym, wrogim monster baseline z domyślnymi
  skryptami NWN. To pozwala testować atak w samym demie.

## Granica „wszystko z pipeline’u”

Pipeline Meshy2Aurora wykonał ingest kanonicznego GLB, rig/animacje, binarny
MDL, TGA, wpis Appearance 2DA, HAK oraz MOD/GFF. Dane wejściowe to kanoniczny
model Meshy i odczytywany tylko jako baza lokalny `appearance.2da`. Geometria
broni jest stockowym zasobem NWN; pipeline nie kopiuje retail UTI do HAK/MOD,
lecz zapisuje w GIT pełną instancję przedmiotu zgodną z działającym modułem
właściciela. MOD nie zawiera żadnego modułowego UTI.

## Offline readback i testy

- źródłowy GLB: 296 276 trójkątów, SHA-256
  `0eeb135c1077f2207b1f362222829a2493c003c64276db98633cf2af5c8385b3`;
- binary MDL: `m2aweapcre10`, 41 węzłów i 42 animacje;
- `ca1slashl`: `PRESERVED_SOURCE`, 48/48 zmieniających się controllerów,
  event `hit` w `1.3333334 s`;
- `ca1slashr`: `PRESERVED_SOURCE`, 48/48 zmieniających się controllerów,
  event `hit` w `0.36666667 s`;
- `rhand -> RightHand` i `lhand -> LeftHand` występują w modelu oraz w obu
  klipach ataku; pełna bramka generatora potwierdziła oba hooki w `42/42`
  animacjach;
- `MODELTYPE=L`, Appearance row `15100`;
- niezależny audyt MOD: dokładnie 6 zasobów (`ARE`, `GIT`, `GIC`, `UTC`, `IFO`,
  `FAC`), zero `UTI`, EE `Mod_MinGameVer=1.89`, `Expansion_Pack=3`, ARE version
  `2`, dokładnie jeden HAK `m2aweaphak10`;
- niezależny audyt HAK: dokładnie `appearance.2da`, `m2aweapcre10.mdl` i
  `m2aweaptex10.tga`, wszystkie zgodne z wygenerowanymi payloadami;
- `cargo test -p m2a-core --lib`: `118 passed`, `0 failed`, `3 ignored`
  (lokalne testy korpusowe);
- `cargo fmt --all -- --check`: PASS;
- canonical workspace guard i Meshy asset-layout guard: PASS.

## Exact frozen lineage

- MOD `m2aweapdemo10.mod`: 16 102 bytes, SHA-256
  `704195a693d42a24f0fcf155058ec0ffdeadee3f1446fd02e3db692b78defa14`;
- HAK `m2aweaphak10.hak`: 43 739 200 bytes, SHA-256
  `3e4074cb036a2ee4ee3286388f5768c67a689234c156f88a11016d667a7350bb`;
- MDL `m2aweapcre10.mdl`: 24 254 668 bytes, SHA-256
  `887eb09b9766f58db9c64584648b378e67356900b1a752ef6ece35a5c489e282`;
- TGA `m2aweaptex10.tga`: 12 582 956 bytes, SHA-256
  `2730d81faf9f6b9304721f15db3e3dead1857c3ec2daddd89cb102160aa06d91`;
- `appearance.2da`: 6 901 320 bytes, SHA-256
  `84460ffa9ad161b4cfed84f19c22e0ebe24d4aa653964a4f223dcf837910ad26`;
- `materialization-report.json`: SHA-256
  `8378ca7662caca43013aab03923772f7986ada974140bfef7867f5ce21761964`;
- `materialization.json`: SHA-256
  `0f454ee757948b4d16bdac5d91ffc5ae7cb49f70408eda6b7cf0d11b2ed729ca`.

Pełny packet: `proof-output/creature-held-item-v10`.

## Instalacja

Oba cele były nieobecne. Kopiowanie wykonano z `File.Copy(..., overwrite:
false)`, a następnie potwierdzono identyczne SHA-256 źródła i celu:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aweapdemo10.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aweaphak10.hak`.

## Kryteria zakończenia właścicielskiego testu

Uruchomić dokładnie `m2aweapdemo10.mod` i Area
`Meshy2Aurora correct held-item test area V10`. Jedyny Creature to
`Stoneback Brute with owner-corrected held item`, template `m2awrhand10`,
ustawiony bezpośrednio przed graczem.

Proof kończy się powodzeniem tylko wtedy, gdy jednocześnie:

1. Stoneback jest widoczny i zwrócony prawidłowym przodem, nie tyłem ani bokiem.
2. Bastard sword jest widoczny w prawej dłoni Creature.
3. Wroga Creature podejmuje walkę, a atak pokazuje rzeczywisty ruch modelu,
   szczególnie `ca1slashl` lub `ca1slashr`, bez statycznej pozy.
4. Broń pozostaje związana z prawą dłonią podczas ruchu i ataku.

Jeżeli którykolwiek punkt nie przejdzie, wynik należy zapisać jako świeżą,
candidate-bound porażkę dokładnie tej linii MOD/HAK/MDL; nie tworzyć kolejnego
wariantu przed diagnozą minimalnej przyczyny.
