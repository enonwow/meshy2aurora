# TLC Veiled Humanoid P100K — pełne nagranie Studio

Data: 2026-07-28
Status: `RECORDED / APP_PIPELINE_COMPLETED`

## Nagranie

- plik:
  `output/playwright/studio-p100k-pipeline-20260728/studio-p100k-full-pipeline.webm`;
- rozmiar: 10 776 020 bajtów;
- SHA-256:
  `15be08bc79eecbce3a9d088346a66dad3e19eec5aabd0b6e711484f4619a83d7`;
- podgląd końcowego ekranu:
  `output/playwright/studio-p100k-pipeline-20260728/final-artifacts.png`.

Plik ma poprawny nagłówek EBML/WebM `1A 45 DF A3`.

## Zarejestrowany przebieg

Nagranie pokazuje rzeczywistą aplikację, bez mocków:

1. ekran Source i jawny wybór `Experimental segmented 100K`;
2. lokalny wybór kanonicznego Meshy GLB oraz pełnej `appearance.2da`;
3. inspekcję Workera/WASM:
   - 102 335 surowych trójkątów,
   - 101 503 wierzchołki,
   - 24 kości,
   - 10 źródłowych klipów,
   - `conversionEligible = true`;
4. ekran Build i stan
   `The local Worker is executing the canonical pipeline`;
5. Review z kanonicznego binary-MDL readbacku:
   - 99 812 bezpiecznych trójkątów wejścia i wyjścia,
   - 98 767 wierzchołków wejścia i wyjścia,
   - 5 segmentów MDL,
   - 42 animacje,
   - 23/23 event hooks,
   - binary readback `PASS`,
   - writer semantic diff `PASS`;
6. listę sześciu artefaktów zwróconych przez `m2a-wasm` Worker;
7. rzeczywiste pobranie demonstracyjnego MOD-u.

## Tożsamość wejścia

- GLB:
  `sample-3d/tlc-veiled-humanoid-h1-p100k-v1/source.glb`;
- GLB SHA-256:
  `941affd66d2afe803b2a63ba72c41c30592813d3f1deeae4600f24e0fc566b7d`;
- wejściowa `appearance.2da` SHA-256:
  `815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a`.

## Tożsamość nagranego wyniku aplikacji

Studio nadało wynikowi deterministyczną tożsamość wyprowadzoną z SHA źródła:

| Artefakt | Bajty | SHA-256 |
|---|---:|---|
| `m2p1h941affd6.hak` | 29 667 213 | `779e54be23c0259ec387c9818aaae188a815a03e2c7d25e45cba16d29c89c328` |
| `m2p1m941affd6.mdl` | 10 182 680 | `1b018d81809d4c944af23c38c04efbb9e6d71ecdcc865293913c17a69c4ef648` |
| `m2p1d941affd6.mod` | 15 185 | `453ca0b19a6f4572182d7421d0ba2290ebfa2a138a1c9459d3e5c6c058a68dcb` |
| `inspection.json` | 1 129 039 | `6a57e38d55d8265e34a18ea1308c4fb2cacc3dcdcd3a25613eb75f6fda47f70a` |
| `conversion-manifest.json` | 3 191 | `7d833bb3ed228ef4297eb2da42ab1eac8405565cb45948b9d70b85c47515bf1f` |
| `summary.json` | 1 733 | `9f9a64c16af18f951937c33c0e6f38ffdc144f31b49cfce82e7e51d2c621f431` |

Jest to wynik przejścia UI z automatyczną tożsamością Studio, dlatego jego
resrefy różnią się od ręcznie zamrożonego `tlcv100demo1.mod`. Nagranie nie
zmienia, nie nadpisuje ani nie instaluje nowego kandydata NWN. Wcześniejszy
test exact replay tej samej ścieżki Worker/WASM z tożsamością zamrożonego
kandydata pozostaje opisany w
`tlc-veiled-humanoid-p100k-studio-pipeline-replay-2026-07-28.md`.

## Granica dowodu

Nagranie dowodzi pełnego lokalnego pipeline aplikacji od wejść do artefaktów.
Nie jest wizualnym dowodem modelu w Aurora Toolset ani NWN. Finalny test
zainstalowanego zamrożonego MOD/HAK pozostaje własnością właściciela.

Jedyny komunikat konsoli podczas nagrania dotyczył brakującego `favicon.ico`
HTTP 404. Worker, WASM, build i projekcja wyniku nie zgłosiły błędu.

## Amendment 2026-07-28 — instalacja wyniku Studio do testu właściciela

1. Exact test-module filename: `m2p1d941affd6.mod`.
2. Toolset module name: `Meshy2Aurora procedural humanoid proof`.
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`.
4. Ordered HAK dependency: `m2p1h941affd6.hak`.
5. Area resref: `m2p1a941affd6`.
6. Creature resref: `m2p1c941affd6`.
7. Appearance row: `15100`.
8. Fixture placement: `[10.0, 14.5, 0.0]`, oriented toward the player.

Studio zostało ponownie uruchomione z tym samym GLB, `appearance.2da` i jawnym
profilem `EXPERIMENTAL_P100K`. Worker/WASM ponownie zwrócił identyczne bajty:

| Artefakt | Bajty | SHA-256 |
|---|---:|---|
| `m2p1h941affd6.hak` | 29 667 213 | `779e54be23c0259ec387c9818aaae188a815a03e2c7d25e45cba16d29c89c328` |
| `m2p1d941affd6.mod` | 15 185 | `453ca0b19a6f4572182d7421d0ba2290ebfa2a138a1c9459d3e5c6c058a68dcb` |

Przed instalacją oba dokładne cele były nieobecne. Zainstalowano:

- MOD:
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2p1d941affd6.mod`;
- HAK:
  `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2p1h941affd6.hak`.

Hash każdego pliku docelowego po kopiowaniu jest identyczny z hashem źródła.
Raport aplikacji potwierdził `semanticReadbackStatus = PASS` oraz dokładne
powiązanie MOD/Area/UTC/HAK wymienione wyżej. Agent nie uruchamiał Aurora
Toolset ani NWN. Stan po instalacji pozostaje:

- `ready_for_owner_proof`;
- Toolset: `modelVisibility=not_tested`,
  `proofCompleteness=missing`;
- NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`.
