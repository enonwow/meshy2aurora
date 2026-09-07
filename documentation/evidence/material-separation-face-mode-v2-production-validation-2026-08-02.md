# Material Separation Face Mode V2 — produkcyjna walidacja statku

Status: offline_implementation_complete / proof_iteration_gate_pending

## Wynik

Face Mode V2 został wdrożony jako target-neutralna warstwa selekcji źródłowych
trójkątów. Dla Placeable cały łańcuch od edytora przez worker/WASM do writerów
MDL/TGA/PWK obsługuje teraz dokładne zakresy twarzy. Produkcyjny model
tlc-ship-under-construction-s1-p150k-v1 otrzymał pięć material slotów i pięć
niezależnych tekstur bez usuwania, spawania, upraszczania ani ponownego
indeksowania geometrii.

Nie utworzono jeszcze nowego MOD/HAK. Zamrożony wcześniejszy kandydat statku
m2a_tlcs1_mod.mod nadal ma formalne osie not_tested/missing, więc obowiązujący
model-iteration gate zabrania materializacji kolejnego lineage, dopóki
właściciel nie poda świeżego wyniku dokładnie tego kandydata albo nie wyda
bezpośredniego wyjątku od gate dla material-only demo.

## Audyt źródła

- kanoniczny model:
  C:\Projects\meshy2aurora\sample-3d\tlc-ship-under-construction-s1-p150k-v1\source.glb;
- SHA-256:
  61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278;
- rozmiar: 11,498,076 bajtów;
- trójkąty: 152,574;
- wierzchołki: 176,201;
- źródłowe materiały/tekstury: 1 / 1;
- connected components: 21,936;
- mediana komponentu: 1 trójkąt;
- największy komponent: 200 trójkątów.

Wniosek: V1 oparty wyłącznie na connected components nie jest ergonomiczny ani
semantycznie wystarczający dla tego modelu. Face Mode V2 jest wymagany.

## Kontrakt Face Mode V2

V2 przechowuje half-open ranges lokalnych indeksów trójkątów związanych z
dokładnym sceneId/nodeId/primitiveId oraz SHA-256 GLB. Resolver:

- odrzuca niekanoniczne, nakładające się i out-of-bounds ranges;
- odrzuca overlap pomiędzy Component Mode i Face Mode;
- zachowuje jawny source fallback dla nieprzypisanych twarzy;
- zachowuje źródłowy porządek indeksów i UV0;
- wystawia tę samą target-neutralną projekcję triangle-to-material-slot dla
  istniejących writerów;
- nie wykonuje geometry cleanup.

Edytor Studio udostępnia domyślny Face Mode, Component Mode, click, Ctrl-click,
Alt-click region grow, Shift-drag rectangle, dokładny overlay, undo/redo oraz
apply/cancel. Lista komponentów jest limitowana, aby 21,936 komponentów nie
blokowało DOM.

## Recepta produkcyjna

Recepta:
C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\recipe-correct-gltf-uv\material-separation-v2.json

Separation SHA-256:
6eeb7877b24feb3f4a315a9ae68809dc5620ca4272bfb7702d3d159f67df4b3a

| Material | Trójkąty | Udział | Ranges |
|---|---:|---:|---:|
| Wood | 141,183 | 92.534% | 1,179 |
| Rope | 1,178 | 0.772% | 879 |
| Sail | 6,464 | 4.237% | 1,129 |
| Cloth | 3,633 | 2.381% | 899 |
| Metal | 116 | 0.076% | 110 |

Wood jest konserwatywnym dopełnieniem. Sail i Cloth odpowiadają dwóm obszarom
zrolowanego płótna na masztach. Rope i Metal są konserwatywnymi, źródłowo
związanymi obszarami testowymi. Model praktycznie nie zawiera jednoznacznej
geometrii metalowych okuć; dlatego nie przemalowano arbitralnie drewnianych
elementów tylko po to, aby zwiększyć licznik Metal.

Klasyfikator użyty do przygotowania tej recepty jest narzędziem source-bound,
nie automatyczną semantyczną gwarancją dla dowolnego GLB. Face Mode pozostawia
użytkownikowi dokładną korektę twarzy.

## Tekstury

### Korekta po odrzuceniu pierwszego preview

Pierwszy preview został odrzucony przez właściciela jako nieakceptowalny. Audyt
wykazał trzy niezależne przyczyny:

1. renderer próbkował jeden texel na cały trójkąt zamiast interpolować UV per
   pixel, co sztucznie tworzyło faceted look;
2. renderer i klasyfikator odwracały współrzędną V, chociaż
   [specyfikacja glTF 2.0](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#textures)
   definiuje `(0, 0)` w lewym górnym rogu obrazu;
3. pięć niezależnych seamless textures nie zachowywało semantycznego układu
   atlasu Meshy i niszczyło dopasowanie detali do UV.

Renderer interpoluje teraz UV i normalne per pixel, a klasyfikator i renderer
używają poprawnej konwencji glTF. Pierwotne bitmapy ImageGen i ich proweniencja
pozostają zachowane jako odrzucony eksperyment, ale fail-closed materializer nie
odwołuje się już do ich payloadów.

Aktywny zestaw składa się z pięciu pełnych wariantów oryginalnego atlasu
2048×2048. Wood otrzymuje ciemną, ciepłą krzywą oraz lokalny high-pass
odzyskujący istniejące szczeliny desek i zużycie bez generowania nowego detalu.
Rope otrzymuje złocisty hemp grade, Sail jasne ivory, Cloth średni taupe, a
Metal ciemny, chłodny grade. Wszystkie warianty zachowują lokalne deski,
krawędzie, płótno i układ wysp UV. Następnie są normalizowane do 1024×1024
filtrem Lanczos3.

Proweniencja wariantów atlasu:
C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\source-atlas-variants-v5-external-reference-midtones\source-atlas-variant-report.json

SHA-256 raportu:
c0e25617f669edebe48b2ed364c8f3c9453be6725bb1d3cd0b33af87fca2bb13

| Material | Resref | TGA SHA-256 |
|---|---|---|
| Cloth | m2a_tlcsm_tex | 3d77a98f75977ee73276e69a7cef88221f3d71aae0b00b33dcace20d671d2d54 |
| Metal | m2a_tlcsm_tex_m1 | 8e43c453ea6cac917bdb12fdf9db08c8f2b2874ec34c03fcaac60380a82d5f25 |
| Rope | m2a_tlcsm_tex_m2 | 85dfb60a186e10f2b526f8b589d25b17289f03965a9c3fb6cf7053b54b81b5c7 |
| Sail | m2a_tlcsm_tex_m3 | 4e8e12275df1192f654f04ed9a372a90f0014db206c5c69de391de6736af8037 |
| Wood | m2a_tlcsm_tex_m4 | 69080a78c99afaae764e30f5d2a1ee2cf37f688f4feae0e67f84656fbef0b499 |

Texture Authoring SHA-256:
ff2f7533c8e782a12c2d36b3e59cf3df9df5f0ba947785876c63d8a9deb15f83

Offline preview:
C:\Projects\meshy2aurora\artifacts\material-separation\tlc-ship-under-construction-v2\offline-preview-v9-external-reference-midtones

Preview zachowuje wszystkie 152,574 trójkąty i źródłowe UV0. Pokazuje
drewniany kadłub, jasny główny żagiel, ciemniejsze płótno drugiego masztu oraz
oddzielne olinowanie. Nie udaje poprawy słabej geometrii wygenerowanego statku;
Material Separation rozwiązuje podział i teksturowanie, nie remesh.

Wood v9 amendment (2026-08-02): the superseded v7 symmetric local high-pass
was replaced after an independent ship-in-construction reference comparison.
The active transform maps source values to a warm timber palette, retains only
bounded dark seam emphasis, and caps positive edge emphasis to suppress the
chalky halo. No external texture payload, synthetic grain, geometry, UV,
cleanup, or material assignment was introduced. The external-reference packet
is `documentation/evidence/tlc-ship-wood-external-reference-comparison-2026-08-02.md`.

## Walidacja

| Check | Wynik |
|---|---|
| canonical workspace | PASS |
| canonical Meshy asset layout | PASS |
| m2a-core Material Separation | 15 passed |
| m2a-core Placeable pipeline | 21 passed, 1 ignored |
| m2a-wasm Face Mode boundary | 1 passed |
| Studio typecheck | PASS |
| Studio unit/component tests | 258 passed |
| worker/WASM integration | 12 passed, 2 skipped |
| produkcyjne pokrycie trójkątów | 152,574 → 152,574 |
| output vertex count | 176,201 |
| duplicated boundary vertices | 0 |
| geometry cleanup | false |
| source UV0 preserved | true |
| texture resources | 5 distinct TGA |
| gated materializer admission tests | 6 passed |

## Fail-closed materializer

Przygotowano narzędzie:

`crates/m2a-core/examples/materialize_tlc_ship_material_separation_v2.rs`

Narzędzie nie zawiera przydzielonej z góry nowej tożsamości i nie może
zmaterializować kolejnego lineage bez zewnętrznych dokumentów admission oraz
identity. Przed buildem:

- wiąże admission z dokładnymi hashami zamrożonego `m2a_tlcs1_mod.mod` i
  `m2a_tlcs1_hak.hak`;
- sprawdza te same hashe w podanych natywnych katalogach NWN;
- wymaga świeżego `modelVisibility=not_visible` dla Toolset lub NWN albo
  bezpośredniego, niepustego wyjątku właściciela dla jednej iteracji
  material-only;
- odrzuca ponowne użycie filename/resref/tag starego lineage;
- wymaga 152,574 trójkątów, 176,201 wierzchołków, pięciu TGA, braku cleanup,
  poprawnego PWK oraz partycjonowania każdego strumienia MDL poniżej granicy
  binarnego formatu;
- buduje dwukrotnie i odrzuca niedeterministyczny MOD/HAK;
- zapisuje wyłącznie przez `create_new` oraz instaluje tylko do nieistniejącego
  celu albo wykorzystuje istniejący plik o identycznym SHA-256;
- tworzy handoff `ready_for_owner_proof`, ale nie uruchamia Toolset ani NWN.

Testy jednostkowe potwierdzają odrzucenie nieudowodnionej iteracji i złego
hasha starego kandydata oraz przyjęcie wyłącznie udokumentowanego
`not_visible` lub jawnego wyjątku właściciela.

## Aktualny gate i dokładny test właściciela

Exact aktualnie zainstalowane pliki są byte-identical z zamrożonym źródłem:

- MOD m2a_tlcs1_mod.mod,
  SHA-256 e02f71e5ba8b5f92ccf76486450ab4bdc78e8b019893fd364d1c109959eb9e83;
- HAK m2a_tlcs1_hak.hak,
  SHA-256 80ebb8bca45a671754bc39a6db690da0251963b128efbb25efbbf07cc2f8ff12.

Handoff wymagany przez obowiązujący gate:

1. test module file: m2a_tlcs1_mod.mod;
2. Toolset module name: The Last City - Ship Under Construction Demo;
3. Area: The Last City Shipyard Construction Demo;
4. zapisać osobno dla Toolset i NWN modelVisibility=visible|not_visible oraz
   proofCompleteness=verified|failed|missing.

Agent nie uruchamiał ani nie kontrolował Aurora Toolset/NWN. Nowa tożsamość,
Appearance row, MOD i HAK nie zostały jeszcze przydzielone ani zmaterializowane.
