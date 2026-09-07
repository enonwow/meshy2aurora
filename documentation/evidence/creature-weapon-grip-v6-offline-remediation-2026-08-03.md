# Creature weapon grip V6 — poprawka offline — 2026-08-03

Status: `ready_for_owner_proof`; dokładny MOD/HAK V6 utworzono i zainstalowano
4 sierpnia 2026 r. Proof wizualny Toolset/NWN pozostaje własnością właściciela.

## Przyczyna

Właścicielski proof dokładnego `m2aweapdemo5.mod` potwierdził widoczne Creature
i stockowy miecz przy prawej dłoni, ale broń była błędnie skręcona. V5 łączył
poprawny, geometrycznie zmierzony środek dłoni z nową ramą rotacji zbudowaną z
kierunku wrist-to-palm i globalnego `Z`. Kierunek podłużny nie wyznacza jednak
rollu przedmiotu, a globalny up nie jest natywnym basisem dłoni/item hooka.

Studio nie ujawniło wady, ponieważ renderowało uproszczony proceduralny miecz,
nie dokładną geometrię i pivot `nw_wswss001`. Parity Studio–MDL było prawdziwe,
ale dowodziło jedynie identycznej macierzy, nie jej poprawności dla stockowego
itemu.

## Implementacja V6

Kalibracja ma nazwę:

`MESHY_H1_PALM_CENTER_NATIVE_ITEM_BASIS_V6`

V6 rozdziela dwie odpowiedzialności:

- translacja pozostaje medianą skin-weighted powierzchni dłoni z V5;
- pełna rotacja X/Y/Z pochodzi z jednego audytowanego natywnego basisu V4;
- geometryczna ścieżka nie przebudowuje już rollu przez world-up;
- fallback bez wystarczającej powierzchni dłoni używa tego samego basisu;
- macierz pozostaje rigid, bez reflection, skali i shear;
- geometria, UV, materiały, skinning i animacje nie są zmieniane.

Exact Fogbound local matrices dla źródła `+Z`:

- right:
  `[-0.20927592, 0.30179292, 0.9301205; 0.93012184, 0.35499254, 0.09409291; -0.30178937, 0.88481694, -0.35499558; position 0.0108317435, 0.00013551023, 0.09678223]`;
- left:
  `[-0.20341122, -0.29308408, -0.9341977; -0.9341983, 0.3437125, 0.09557905; 0.29308274, 0.89216787, -0.34371376; position -0.0093700085, 0.0012637153, 0.10334423]`.

Exact source:

- `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`;
- SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`;
- triangles: `297190`.

## Studio preview

Preview nadal nie kopiuje retailowego payloadu NWN. Kontrakt został więc
uściślony z mylącego `CALIBRATION_PROXY` na
`NWN_SHORTSWORD_BASIS_PROXY`. Interfejs pokazuje jawne ostrzeżenie:
`Not exact item geometry or Toolset proof.`

Podgląd nadal wymaga parity z binary-MDL readbackiem, ale nie może już być
przedstawiany jako pixel-perfect proof konkretnego UTI.

## Bramki

- red-before-green regresja rollu V5: potwierdzona;
- `creature_equipment::tests`: `6/6` pass;
- exact Fogbound hook matrices, bez materializacji kandydata: pass;
- procedural product / fixture-module separation: pass;
- generator demo V6 compile check: pass;
- targeted native WASM product boundary: pass (`1/1`, 259.92 s);
- Studio preview + canonical-result tests: `37/37` pass;
- production WASM + TypeScript + Vite build: pass;
- `cargo fmt --all -- --check`: pass.

## Materializacja i handoff — 2026-08-04

Po osobnym poleceniu właściciela generator V6 został uruchomiony na dokładnym
źródle wskazanym powyżej. Powstał jeden kandydat:

- plik modułu: `m2aweapdemo6.mod`;
- nazwa modułu: `Meshy2Aurora Creature Weapon Grip V6`;
- Area: `Meshy2Aurora Creature Weapon Test V6`;
- HAK: `m2aweaphak6.hak`;
- model: `m2aweapcre6.mdl`;
- tekstura: `m2aweaptex6.tga`;
- Appearance row: `15104`, `MODELTYPE=L`;
- Creature: `V6 RIGHT HAND - native stock sword - native basis`;
- stockowy przedmiot: `nw_wswss001`, slot `right_hand`;
- trójkąty: `297190`;
- animacje i hook coverage: `42/42`;
- kalibracja: `MESHY_H1_PALM_CENTER_NATIVE_ITEM_BASIS_V6`.

Exact canonical lineage:

- MOD SHA-256:
  `9a24f82bb163268095bf0a5440d55969d2942b7cca6b9e6629ea0cb091607abc`;
- HAK SHA-256:
  `e0c40fc53e4ef5ac2a91b9d0bc28aeeda826e8cbb3cf60b7c54f278d966a4291`;
- MDL SHA-256:
  `26b33fb65785935fb99ce0882f174285cb36016d8c402bc4262679f327097cb6`;
- TGA SHA-256:
  `7e24d51726355bf3a4ee2f1429ea85f43a604bc2ee3e079c9fd5a306d4b1426a`;
- output `appearance.2da` SHA-256:
  `06fc4a5e530d4c5e51ff2574e003fa02bb9833194b55fc4e091af6af313dd276`;
- machine-readable packet:
  `proof-output/creature-weapon-grip-v6/handoff.json`.

Wszystkie pliki wymienione w `handoff.json` zostały po materializacji ponownie
zahashowane; długości i SHA-256 są zgodne. MOD i HAK skopiowano wyłącznie do
wcześniej nieistniejących celów, a następnie potwierdzono byte-for-byte:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aweapdemo6.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aweaphak6.hak`.

Agent nie uruchamiał ani nie kontrolował Toolsetu/NWN. Stan obu osi proofu
pozostaje `modelVisibility=not_tested`, `proofCompleteness=missing` do wyniku
właściciela. V5 pozostaje zamrożonym negatywnym proofem błędnego rollu.

## Wynik właściciela dla wyposażenia V6 — 2026-08-04

Właściciel zgłosił, że V6 nie demonstruje poprawnie normalnego przedmiotu.
Diagnoza offline wykazała, że `Equip_ItemList` wskazywał wyłącznie na zewnętrzny
resref `nw_wswss001`, a MOD nie zawierał własnego zasobu UTI. Lokalna instalacja
NWN:EE rzeczywiście indeksuje `nw_wswss001`, lecz takie odwołanie nie spełnia
samowystarczalnego kontraktu demonstracyjnego i utrudnia rozdzielenie problemu
rozwiązywania zasobu od problemu transformacji hooka.

Wynik ten jest funkcjonalną porażką wyposażenia V6. Nie zmienia go na
`modelVisibility=not_visible` i nie nadpisuje wcześniejszych obserwacji samego
modelu Creature. Minimalna delta V7: własny normalny longsword UTI w MOD oraz
identyczny resref w blueprintcie i instancji wyposażenia Creature.
