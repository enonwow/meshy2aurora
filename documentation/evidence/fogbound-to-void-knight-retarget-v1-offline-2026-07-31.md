# Fogbound → Void Crystal Knight — retarget V1 offline evidence

Data: 2026-07-31  
Status: `OFFLINE VERIFIED`  
Runtime: `modelVisibility=not_tested`, `proofCompleteness=missing`

## Zakres

Dowód obejmuje implementację kopiowania i retargetingu klipu animacji między
dwoma modelami o zgodnej semantycznej hierarchii oraz różnym rest pose. Nie
jest dowodem widoczności ani jakości ruchu w Aurora Toolset lub NWN.

Model-dawca:

- asset: `tlc-fogbound-claw-guard-h1-p300k-v1`;
- plik: `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`;
- SHA-256: `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`;
- klip: `ca1slashl`.

Model docelowy:

- asset: `void-crystal-knight-h1-v1`;
- plik: `sample-3d/void-crystal-knight-h1-v1/source.glb`;
- SHA-256: `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`.

Zachowany wariant Fogbound `source.glb`, odrzucony wcześniej przez gate
ciągłości śmierci, nie został użyty.

## Zaimplementowany przepływ

- Core jest jedynym źródłem klasyfikacji rigu, mapowania kości i matematyki
  ruchu;
- mapowanie używa unikalnej nazwy kości oraz nazwy parenta, nie node ID;
- dostępne są jawne tryby `EXACT_RIG_COPY_V1` i
  `SAME_HIERARCHY_RETARGET_V1`, bez ukrytego `AUTO`;
- WASM i Worker wystawiają inspekcję kompatybilności i retarget klipu;
- Studio pokazuje status, wszystkie różnice, oba modele i wynikowy ruch przed
  odblokowaniem `Copy retargeted to Custom`;
- dokument V3 zapisuje `RETARGETED_MODEL_COPY` wraz z donor/target SHA,
  sygnaturami rigów, fingerprintem zgodności, root scale, wersją algorytmu i
  fingerprintem wynikowego ruchu;
- zmiana targetu, dawcy, klipu, trybu lub authoring revision po preview blokuje
  commit wyniku.

## Dokładny wynik

Artefakt:

`artifacts/fogbound-to-void-knight-retarget-v1-2026-07-31/animation-transfer-result-v1.json`

- rozmiar: `230979` bajtów;
- SHA-256 artefaktu:
  `c394b7b9a00b4d0d182bd4415d190818003d32f7e8fa10f966b5fdcb57c035e3`;
- status: `RETARGETABLE_SAME_HIERARCHY`;
- mapowanie: 24 kości, zero różnic strukturalnych;
- rest pose: 24 różnice translacji i 23 różnice rotacji;
- klip: 2,50 s, 48 tracków, 24 docelowe node ID;
- root-motion scale: `1.2303443`;
- compatibility fingerprint:
  `b74157dbaced50928f8382246d2aed9be4b541dc9fbf67fe10d93175828bb108`;
- donor rig signature:
  `41ef058165d36cff936fc39e4fa3aa38d7da383f014eca47a69b0a484cd2a7c9`;
- target rig signature:
  `6ce084ed022853f6eb0fdca86ffa335aeb082dab61c2fee2193f48ebe046ba4e`;
- output motion fingerprint:
  `fc4e1a2f227c1cea33531a8b5efa2f3f8811861adfae5469c230944e288ec83a`;
- algorytm: `M2A_SAME_HIERARCHY_REST_DELTA_V1`;
- ograniczenia:
  `UNIQUE_NAMES|SAME_PARENT_GRAPH|TR_ONLY|LINEAR|NO_SCALE_OR_SHEAR`.

Exact corpus test materializuje wynik w pamięci jako `Custom` i sprawdza
możliwość przypisania go do wszystkich 42 slotów bazowych. Nie zapisuje MOD,
HAK ani nowej iteracji modelu.

### Regresja wszystkich klipów dawcy

Test
`exact_every_fogbound_clip_retargets_and_materializes_on_void_crystal_knight`
przechodzi przez każdy z dziewięciu klipów kanonicznego Fogbounda:

- `cpause1`;
- `cwalk`;
- `crun`;
- `ctaunt`;
- `ca1slashl`;
- `ca1slashr`;
- `cdamagel`;
- `cguptokdb`;
- `cdead`.

Dla każdego klipu test uruchamia publiczną granicę
`copy_animation_clip_between_models_v1`, wymaga statusu
`RETARGETABLE_SAME_HIERARCHY`, mapowania 24 kości, dokładnie 48 wynikowych
tracków, wyłącznie node ID rigu Void Knighta, skończonych wartości oraz pełnej
proweniencji V3. Następnie zapisuje wszystkie dziewięć wyników jako `Valid`,
wiąże je jako dziewięć `Custom` i materializuje wspólny komplet Base 42.

Wynik exact runu 2026-07-31: `1 passed`, `0 failed`, `108,18 s`.
Nie powstał MOD/HAK.

## Bramki

Zaliczone 2026-07-31:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`;
- exact ignored corpus test Fogbound → Void Knight: `1 passed`;
- Studio: `410 passed`, `3 skipped`;
- Worker integration: `25 passed`, `2 skipped`;
- browser persistence: `2 passed`;
- produkcyjny `npm run build`;
- kontrakt Worker/WASM: 20 requestów i 20 exhaustywnych case'ów;
- bundle budget: JS `1485165 / 1490000`, CSS `151507 / 152000`, WASM
  `3532160 / 4000000` bajtów.

Native Core i Node WASM są porównywane byte-for-byte w teście granicy.
Wejściowe GLB nie są zapisywane ani modyfikowane przez operację.

## Proof MP4 viewportu aplikacji

Pełny przepływ Studio został wykonany na żywo w lokalnej aplikacji:

1. dokładny Void Crystal Knight został wczytany jako target;
2. kanoniczny `source-death-continuous.glb` został wybrany jako donor;
3. Core zwrócił `Retargetable rig`, 24/24 mapped bones i 47 różnic rest pose;
4. wybrano `ca1slashl`, `Same hierarchy retarget` i uruchomiono jawny preview;
5. donor `ca1slashl` oraz target `imp_ca1slashl` były odtwarzane równolegle
   przez ponad trzy pełne cykle.

Nagranie:

`artifacts/fogbound-to-void-knight-retarget-v1-2026-07-31/fogbound-ca1slashl-to-void-knight-retarget-preview-proof-v1.mp4`

- SHA-256:
  `0e6ae62ca56c20cecc5282bb904c46d4f89ee3aeaf0423e005213418e8aa5b87`;
- `289` klatek, `30 fps`, `9,633 s`, `1266 × 712`;
- pierwsze 2 s utrwalają status, SHA, tryb i nazwy obu klipów;
- dalsze 7,633 s pokazuje odtwarzanie donor/target oraz root scale `1.2303` i
  motion SHA `fc4e1a2f227c…`;
- kontener MP4 i wszystkie klatki zostały odczytane ponownie przez lokalny
  decoder po kodowaniu.

Kadr identyfikacyjny:

`artifacts/fogbound-to-void-knight-retarget-v1-2026-07-31/fogbound-ca1slashl-to-void-knight-retarget-preview-proof-v1-poster.png`

- SHA-256:
  `dac9c38ae28d977af9faaa0bfbc10f1f6ad52e01f8b082f438d5c7626f3cc19`.

Jest to proof viewportu aplikacji i działania pipeline retargetingu. Nie jest
to proof Aurora Toolset ani NWN.

## Proof MP4 wszystkich dziewięciu animacji

Drugi proof viewportu obejmuje pełny kanoniczny zestaw klipów Fogbounda:
`cpause1`, `cwalk`, `crun`, `ctaunt`, `ca1slashl`, `ca1slashr`, `cdamagel`,
`cguptokdb` i `cdead`. Każdy fragment pokazuje równolegle model dawcy oraz
wynik retargetingu na Void Crystal Knighta, przygotowany przez normalny dialog
`Copy animation from another model` i Core/WASM pipeline aplikacji.

Nagranie:

`artifacts/fogbound-to-void-knight-retarget-v1-2026-07-31/fogbound-all-9-animations-to-void-knight-retarget-preview-proof-v1.mp4`

- SHA-256:
  `c2a366be84ebe1e07902775d23946a23c1e42734f210a14f7f84f2c2fe6e5fd3`;
- `891` klatek, `30 fps`, `29,7 s`, `1266 × 712`;
- wszystkie `594` przechwycone klatki PNG zostały poprawnie zdekodowane;
- cały wynikowy MP4 został ponownie odczytany: `891/891` klatek;
- podpis filmu rozróżnia donor Fogbound Claw Guard i retargetowany Void
  Crystal Knight oraz identyfikuje każdy z dziewięciu klipów.

Kadr identyfikacyjny:

`artifacts/fogbound-to-void-knight-retarget-v1-2026-07-31/fogbound-all-9-animations-to-void-knight-retarget-preview-proof-v1-poster.png`

- SHA-256:
  `3d7759244c11162f6bf9dc768a764a256f43b24fad2ff098535037859ed26c9f`.

Jest to proof viewportu aplikacji i pełnego zestawu retargetowanych animacji,
nie proof Aurora Toolset ani NWN. Podgląd nie zapisał klipów do bieżącego
projektu: żaden przycisk `Copy retargeted to Custom` nie został zatwierdzony.

## Otwarta granica proofu

Nie utworzono i nie zainstalowano MOD/HAK. Nie uruchamiano Aurora Toolset ani
NWN. Ocena wizualna ruchu w runtime oraz exact binary MDL readback dla
materializowanego produktu pozostają otwarte i wymagają osobnego,
autoryzowanego lineage oraz proofu wykonanego przez właściciela.
