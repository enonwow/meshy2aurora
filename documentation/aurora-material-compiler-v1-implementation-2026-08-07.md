# Aurora Material Compiler V1 — implementacja sześciu etapów

Data: 2026-08-07\
Status: `IMPLEMENTED_OFFLINE / 6_OF_6_COMPLETE / OWNER_PROOF_SEPARATE`

## Aktualizacja 2026-08-12 — sześć etapów ukończonych

Plan implementacyjny został domknięty w kodzie i testach offline:

1. executor bake tworzy rzeczywiste piksele diffuse oraz specular/gloss, zapisuje
   TGA i porównuje wynik po odczycie;
2. deterministyczne writery i parsery MTR/TXI walidują obsługiwane dyrektywy,
   kodowanie ASCII i typy zasobów `2072`/`2022`;
3. rozszerzony writer binary MDL zapisuje wartości materiału, `texture1`,
   `texture2`, `materialname`, UV1..UV3, tangenty oraz kontrolery alpha i
   self-illumination, a następnie wykonuje semantyczny readback;
4. source-quality gate raportuje texel density, błędne i zdegenerowane UV,
   fragmentację komponentów oraz kontrast mipów;
5. Studio ma tryb `Aurora Export`, który rekonstruuje podgląd z końcowego
   binary-MDL readback i dokładnych, zweryfikowanych SHA-256 plików TGA;
6. pion E2E prowadzi GLB przez compiler materiałów i writer
   MDL/TGA/MTR/TXI do HAK/MOD, po czym odczytuje i porównuje zasoby oraz
   tożsamość modułu.

Etap szósty potwierdza integralność pakietu offline. Wynik testowy celowo ma
`ready_for_owner_proof = false` i blocker
`ARTIFACTS_NOT_FROZEN_OR_INSTALLED`. Status gotowości można nadać dopiero po
zamrożeniu dokładnego kandydata oraz instalacji i weryfikacji jego MOD/HAK;
zgodnie z decyzją właściciela końcowy proof w Toolset/NWN wykonuje człowiek.

## Cel

Zaimplementowano plan poprawy zgodności materiałów glTF/Meshy z Aurorą:
wspólny kontrakt materiału dla Creature, Placeable, Tile i przyszłych celów,
rzeczywisty bake zasobów, rozszerzony zapis/readback MDL, kontrolę jakości
źródła, podgląd finalnego eksportu oraz deterministyczny pion pakietowania E2E.

Implementacja i syntetyczny test E2E nie tworzą nowej iteracji istniejącego
modelu statku ani nie zmieniają zamrożonych kandydatów proof.

## Podstawa Aurora First

Fakty z lokalnej dekompilacji Aurory użyte przez kontrakt:

- parser mesh rozpoznaje `diffuse`, `ambient`, `specular`, `shininess`,
  `selfillumcolor`, `alpha`, `texture0..texture2`, `materialname` i
  `renderhint`;
- parser MTR rozpoznaje `texture0..texture10`, `renderhint`, `transparency`,
  `twosided` i pozostałe pola materiałowe;
- `NormalAndSpecMapped` jest rozpoznawanym render hintem;
- MTR jest zasobem typu `2072`;
- parser TXI rozpoznaje między innymi `mipmap`, `filter`, `gamma`, `clamp`,
  `isbumpmap`, `bumpmapscaling` i `specularcolor`.

Jest to kontrakt planowania. Samo rozpoznawanie pola przez parser Aurory nie
jest jeszcze dowodem poprawności automatycznego authoringu ani proofem
wizualnym.

## Zaimplementowany zakres

Dodano `AuroraMaterialIrV1` oraz czysty kompilator
`compile_gltf_material_v1`/`compile_gltf_materials_v1`.

Obsługiwane profile planowania:

1. `AURORA_CLASSIC_SAFE`
   - `baseColorFactor` mapuje do koloru diffuse;
   - `baseColorTexture` mapuje do `texture0`;
   - metallic/roughness tworzą jawną operację bake, a nie cichy drop;
   - emissive factor mapuje do `selfillumcolor`;
   - normal map jest jawnie `DROPPED_WITH_WARNING`;
   - `MASK`, `BLEND` i aktywne `doubleSided` są `BLOCKED`;
   - profil nie wymaga MTR.

2. `NWN_EE_MTR`
   - diffuse mapuje do `texture0`;
   - normal map mapuje do `texture1`;
   - metallic/roughness tworzą plan bake specular/gloss;
   - `NormalAndSpecMapped`, `twosided`, `PUNCHTHROUGH` i transparency są
     reprezentowane w planie MTR;
   - dokładny glTF `alphaCutoff` pozostaje jawnie
     `DROPPED_WITH_WARNING`, ponieważ punch-through nie zachowuje dowolnego
     progu 1:1.

Każdy materiał otrzymuje dokładnie jedną decyzję dla każdego źródłowego
kanału:

- `PRESERVED`;
- `BAKED`;
- `DROPPED_WITH_WARNING`;
- `BLOCKED`.

Nieznany alpha mode, niefinitywne parametry, cutoff poza `0..=1`, niewspierany
kanał UV oraz zły hash źródła są obsługiwane fail-closed.

Pełna tabela materiałów jest związana z SHA-256 źródłowego GLB i agreguje
status: jeden zablokowany materiał blokuje wynik całej tabeli.

## Integracja

- istniejący raport `M6MaterialFidelityReportV1` zawiera teraz opcjonalny,
  zgodny wstecznie `auroraCompiler`;
- obecna ścieżka M6 uruchamia profil `AURORA_CLASSIC_SAFE` podczas
  przygotowania tekstury;
- rozszerzony writer MDL/TGA/MTR/TXI jest opt-in i zachowuje zgodność istniejącej
  ścieżki bazowej;
- WASM udostępnia `compileAuroraMaterialsV1Json`, dzięki czemu Studio może
  odczytać ten sam source-bound kontrakt co Rust core;
- `Aurora Export` odtwarza wynik z finalnego binary-MDL readback i dokładnych
  artefaktów TGA po walidacji SHA-256;
- pion E2E tworzy i odczytuje MDL, TGA, MTR, TXI, HAK i MOD bez uruchamiania
  Toolset/NWN.

## Weryfikacja offline

Uruchomione testy:

- `cargo test -p m2a-core` — PASS (pełny pakiet; testy wymagające lokalnych,
  Git-ignored korpusów pozostają jawnie `ignored`);
- zestaw regresji nowych writerów/gate/E2E i istniejącego MDL writera — 62/62
  PASS;
- `cargo test -p m2a-wasm aurora_material_boundary` — 1/1 PASS;
- pełny `npm test` w Studio — 279/279 PASS;
- `npm run typecheck`, `cargo check -p m2a-core -p m2a-wasm`,
  `cargo fmt --all -- --check` i `git diff --check` — PASS.

Testy pokrywają profil klasyczny i EE MTR, realne piksele bake, pełny ledger
kanałów, mapowanie normal/specular/alpha/two-sided, parser/writer MTR/TXI,
rozszerzone pola i streamy MDL, source-quality gate, SHA-256 artefaktów Studio
oraz deterministyczny round-trip HAK/MOD.

## Pozostała granica odbioru

Żaden z sześciu etapów implementacyjnych nie pozostaje otwarty. Osobnym,
nieautomatyzowanym krokiem odbioru jest human-owned proof wizualny dokładnego,
zamrożonego kandydata w Aurora Toolset i NWN. Ewentualna porażka wizualna
otwiera minimalną kolejną iterację zgodnie z model iteration gate; sama
implementacja pipeline'u nie omija tej bramki.
