# Badania VFX: źródła internetowe i wnioski dla Meshy2Aurora

Data utworzenia: 2026-09-02\
Ostatnia aktualizacja: 2026-09-02 — runda internetowa R7\
Status: `LIVING RESEARCH LEDGER / BACKLOG PO AKTYWNYCH GATE'ACH`\
Zakres: narzędzie do tworzenia, podglądu, walidacji i pakowania VFX dla
Neverwinter Nights: Enhanced Edition.

## 1. Cel i sposób użycia dokumentu

Ten dokument scala kolejne rundy badań internetowych wykonane w wątku Codex
dotyczącym przyszłego edytora VFX. Zawiera:

- źródła specyficzne dla NWN/Aurory;
- porównawcze silniki i edytory VFX;
- biblioteki UI, osi czasu, grafów i walidacji shaderów;
- legalne źródła materiałów testowych;
- wnioski produktowe i architektoniczne;
- ryzyka licencyjne, zgodnościowe i proofowe.

Dokument ma być aktualizowany po każdej kolejnej rundzie wyszukiwania. Nowe
źródło powinno otrzymać opis przydatności, status licencji i decyzję: użycie,
eksperyment, obserwacja albo odrzucenie.

## 2. Granice i hierarchia dowodów

Badania internetowe nie zmieniają zasady `Aurora First` z `PROJECT_RULES.md`.
Dla zachowania silnika kolejność autorytatywności pozostaje następująca:

1. fakt z dekompilacji Aurory;
2. fakt z retail/resource/binary albo zatwierdzonego proofu;
3. własny reader, writer i readback Meshy2Aurora;
4. zewnętrzne implementacje jako porównanie lub dodatkowy oracle;
5. dokumentacja społeczności i dyskusje jako hipotezy do sprawdzenia.

Zewnętrzne repozytoria pozostają read-only reference. Nie kopiujemy kodu,
fixture'ów, tekstur ani payloadów bez osobnej decyzji licencyjnej i zapisu
provenance. GPL oznacza domyślnie analizę zachowania i niezależną implementację,
nie przenoszenie kodu do Meshy2Aurora.

## 3. Najważniejszy wynik badań

Przyszły produkt nie powinien być tylko formularzem parametrów jednego
emitera. Format docelowy NWN rozdziela VFX na co najmniej trzy warstwy:

```text
warstwa 1: model MDL i node'y emitter
warstwa 2: visualeffects.2da + progfx.2da + dźwięki i fazy efektu
warstwa 3: NWN:EE MTR + SHD + rozszerzone materiały i shadery
```

Rekomendowany roboczy zakres produktu to zatem `Aurora VFX Studio`, które:

- tworzy i importuje ograniczony, jawnie zgodny model efektu;
- pokazuje deterministyczny podgląd oraz różnice względem możliwości NWN;
- kompiluje projekt do własnego IR, a następnie do natywnych zasobów;
- generuje binary MDL, tekstury, MTR/SHD, 2DA, HAK i opcjonalny moduł galerii;
- wykonuje własny readback i raportuje `supported`, `approximated` lub
  `blocked` dla każdej użytej funkcji;
- kończy po stronie agenta statusem `ready_for_owner_proof`; wizualny wynik
  Toolset/NWN pozostaje decyzją właściciela.

## 4. Źródła bezpośrednio związane z NWN i Aurorą

### 4.1 Specyfikacje i dokumentacja formatu

#### NWN Wiki: MDL ASCII Emitter Nodes

- URL: <https://nwn.wiki/pages/viewpage.action?pageId=139690011>
- Rola: katalog parametrów oraz trybów klasycznych emitterów Aurora.
- Wartość: punkt startowy dla schematu authoringu i nazw pól.
- Ograniczenie: opis społecznościowy; znaczenie binarne i runtime trzeba
  potwierdzić przez Aurora First oraz retail corpus.

#### NWN Wiki: MTR

- URL: <https://nwn.wiki/spaces/NWN1/pages/12027232/MTR>
- Rola: format materiału NWN:EE.
- Potwierdzone elementy: `texture0`-`texture10`, `renderhint`, sparowane
  `customShaderVS`/`customShaderFS`, parametry float/int/vec4, `transparency`,
  `twosided`, `sample_framebuffer`, `volumetric` i limit nazwy 16 znaków.
- Ważna pułapka: MTR powiązany z PLT powinien pomijać `texture0` całkowicie;
  `texture0 null` również może powodować błędne zachowanie.

#### NWN Wiki: Standard material inputs

- URL:
  <https://nwn.wiki/spaces/NWN1/pages/38175898/Standard%2Bmaterial%2Binputs>
- Rola: mapowanie PBR w NWN:EE.
- Sloty: diffuse/base color, normal, specular, roughness, height i
  self-illumination.
- Wniosek: edytor VFX powinien umieć tworzyć materiały dla mesh/chunk VFX, ale
  nie może udawać obsługi wejść, których NWN nie ma. AO trzeba ewentualnie
  bake'ować do diffuse zgodnie z jawną polityką.

#### NWN Wiki: Shader Engine Support

- URL:
  <https://nwn.wiki/spaces/NWN1/pages/14614573/Shader%2BEngine%2BSupport>
- Rola: uniformy, macierze, tekstury silnikowe, `#define` i różnice wersji.
- Ważny fakt: wskazanie custom shadera w MTR wymaga jawnego dostarczenia obu
  części VS i FS.
- Wniosek: projekt shadera musi mieć `targetRuntimeProfile`, ponieważ dostępne
  uniformy i ich semantyka zmieniają się między buildami NWN:EE.

#### NWN Wiki: Shaders

- URL: <https://nwn.wiki/spaces/NWN1/pages/60981936/Shaders>
- Rola: ładowanie i organizacja plików `.shd`, stockowe entrypointy oraz
  zachowanie `#include`.
- Ryzyko: nadpisywanie stockowych shaderów wymaga wersjonowania. Stary sposób
  wyboru shaderów przez TXI nie powinien być generowany dla nowych efektów.

#### NWN Wiki: visualeffects.2da

- URL:
  <https://nwn.wiki/pages/viewpage.action?navigatingVersions=true&pageId=129237061>
- Rola: wiązanie modeli, dźwięków, efektów impact/duration/cessation i ProgFX.
- Wniosek: projekt VFX powinien mieć jeden dokument kompozycji, z którego
  generowane są wymagane wiersze oraz zasoby, zamiast ręcznie edytować kilka
  luźnych plików.

#### NWN Wiki: progfx.2da

- URL: <https://nwn.wiki/spaces/NWN1/pages/38175071/progfx.2da>
- Rola: trzynaście silnikowych typów programowanych efektów dostępnych przez
  parametry 2DA.
- Wniosek: edytor powinien udostępniać typowane formularze per `Type`, a nie
  dowolne osiem pól bez opisu. Nie jest to system własnego kodu GPU.

#### NWN Wiki: 2DA Files

- URL: <https://nwn.wiki/spaces/NWN1/pages/38174875/2da%2BFiles>
- Rola: lista tabel, miejsce ładowania i znane limity wierszy.
- Istotny limit: identyfikator `visualeffects.2da` jest ograniczony przez
  reprezentację sieciową do zakresu `0..65535`.

#### NWN Wiki: cachedmodels.2da

- URL: <https://nwn.wiki/spaces/NWN1/pages/145457156/cachedmodels.2da>
- Rola: preładowanie modeli ograniczające przycięcie przy pierwszym VFX.
- Wniosek: Studio może raportować kandydatów do pre-loadu, ale nie powinno
  automatycznie umieszczać wszystkich efektów w tej tabeli.

#### NWN Wiki: Introduction to Hakpacks

- URL:
  <https://nwn.wiki/spaces/NWN1/pages/60981970/Introduction%2Bto%2BHakpacks>
- Rola: nazewnictwo, kolejność HAK-ów, precedencja i konflikty 2DA.
- Wniosek: eksport wymaga raportu kolizji resrefów i wierszy 2DA oraz jawnej
  informacji o wymaganej pozycji HAK-a w load order.

#### NWN Wiki: Models

- URL:
  <https://nwn.wiki/pages/viewpage.action?navigatingVersions=true&pageId=106856461>
- Rola: aktualne ostrzeżenia dotyczące narzędzi modelowych.
- Ważne: `nwnmdlcomp`/starszy NWN Explorer mogą nie obsługiwać poprawnie
  nowych danych NWN:EE i nie mogą być jedynym oracle zgodności.

### 4.2 Implementacje i narzędzia NWN

#### nwn_mdl_webviewer

- URL: <https://github.com/dunahan/nwn_mdl_webviewer>
- Licencja deklarowana przez repo: MIT.
- Rola: najpełniejszy browserowy punkt odniesienia dla ASCII/binary MDL,
  animacji, emitterów, TGA/DDS/PLT, MTR/TXI i sprite-sheetów.
- Przydatne wzorce: pool cząstek, `birthratekey`, `xgrid`/`ygrid`, obsługa
  texture flip, tryby billboardów, status brakującej tekstury.
- Decyzja: silny reference i oracle porównawczy, ale Meshy2Aurora zachowuje
  własny reader, symulator oraz renderer.

#### NWN Emitter Editor

- URL: <https://github.com/varenx/nwn_emitter_editor>
- Strona projektu:
  <https://neverwintervault.org/project/nwnee/other/tool/nwn-emitter-editor>
- Licencja: GPL-3.0.
- Rola: bezpośredni przykład UI do wizualnej edycji emitterów w ASCII MDL.
- Ograniczenie deklarowane przez autora: wygląd w grze może znacznie różnić
  się od preview. ASCII MDL nie jest docelową ścieżką runtime Meshy2Aurora.
- Decyzja: referencja UX i pokrycia pól, bez kopiowania kodu.

#### rollNW

- URL: <https://github.com/jd28/rollnw>
- Dokumentacja: <https://rollnw.readthedocs.io/>
- Licencja: MIT.
- Rola: parsery, formaty i implementacja symulacji emitterów cytowana również
  przez inne narzędzia NWN.
- Decyzja: jeden z najważniejszych niezależnych cross-checków semantyki, ale
  nie zamiennik dekompilacji Aurory.

#### Radoub

- URL: <https://github.com/LordOfMyatar/Radoub>
- Dokumentacja implementacji:
  <https://github-wiki-see.page/m/LordOfMyatar/Radoub/wiki/Radoub-UI-Developer>
- Licencja repozytorium: GPL-3.0.
- Rola: świeża implementacja renderowania emitterów w wielonarzędziowym,
  cross-platformowym toolsecie NWN.
- Przydatne zachowanie: typowany compile emittera, seeded RNG, spawn
  accumulator, cone emission, grawitacja, drag, prewarm, rozmiar X/Y,
  billboard basis, animowany transform i gate emittera zależny od aktywnej
  animacji.
- Jawna luka projektu: Beam, Mesh i LinkedChain nie mają wiernej implementacji
  i wpadają w billboard fallback.
- Decyzja: bardzo wartościowy oracle zachowania i lista przypadków testowych;
  implementacja clean-room po potwierdzeniu przez Aurora/retail.

#### Borealis NWN Model Viewer

- URL: <https://github.com/varenx/borealis_nwn_model_viewer>
- Licencja: GPL-3.0.
- Rola: Qt6/OpenGL viewer modeli NWN, materiałów i renderera.
- Decyzja: reference-only, szczególnie dla porównania parsowania i flag
  renderowania; nie kopiować kodu.

#### NWN Explorer

- URL: <https://github.com/virusman/nwnexplorer>
- Releases: <https://github.com/virusman/nwnexplorer/releases>
- Rola: ekstrakcja zasobów BIF/KEY oraz historyczna wiedza o binary MDL.
- Ograniczenie: część ścieżki modelowej opiera się na przestarzałym
  `nwnmdlcomp`; nowe modele EE mogą się nie dekompilować poprawnie.
- Decyzja: narzędzie pomocnicze do oglądania zasobów, nie jedyny walidator.

#### Clean Models: EE

- CLI: <https://github.com/plenarius/cleanmodels>
- UI: <https://github.com/plenarius/cleanmodels-qt>
- Opis: <https://nwn.wiki/spaces/NWN1/pages/64815203/Clean%2BModels%2BEE>
- Rola: sprawdzanie, naprawa i dekompilacja modeli, dostępna również jako WASM
  w innych viewerach.
- Decyzja: niezależny cross-check wejść/wyjść, nie produkcyjny dependency
  Meshy2Aurora bez osobnej decyzji licencyjnej.

#### neverwinter.nim

- URL: <https://github.com/niv/neverwinter.nim>
- Licencja: MIT.
- Rola: rozbudowany zestaw parserów i CLI dla zasobów NWN, w tym ERF, GFF,
  2DA, skrypty i resource manager.
- Decyzja: reference dla formatów i testów różnicowych. Produkt zachowuje
  własne writery wymagane przez `PROJECT_RULES.md`.

#### nwnrs

- URL: <https://github.com/urothis/nwnrs>
- Rola: ekosystem Rust dla pakietów i zasobów NWN.
- Wartość: naturalny technologicznie punkt porównania dla `m2a-core`.
- Decyzja: przed użyciem kodu wymagany osobny audyt zakresu, aktywności i
  licencji poszczególnych crate'ów.

#### xoreos i xoreos-tools

- Engine: <https://github.com/xoreos/xoreos>
- Tools: <https://github.com/xoreos/xoreos-tools>
- Rola: szeroka reimplementacja silników BioWare i narzędzia archeologii
  formatów.
- Decyzja: reference-only; zakres wielogrowy i niepełna zgodność NWN oznaczają,
  że nie jest to runtime oracle dla naszego eksportu.

#### NWNFileFormats

- URL: <https://github.com/Liareth/NWNFileFormats>
- Rola: C++ reader/writer dla GFF, ERF, KEY, BIF, TLK oraz narzędzia do merge
  2DA.
- Wartość: dodatkowy parser i zbiór przypadków pakowania.
- Ograniczenie: projekt deklaruje zakres NWN 1.69/1.74, więc rozszerzenia EE
  wymagają osobnej weryfikacji.

#### Neverwinter Toolkit (NWT)

- URL: <https://github.com/jeffmcclure/nwt>
- Licencja: MIT.
- Rola: build system i edytowalne JSON5 dla MOD/HAK/GFF.
- Wartość: wzorzec rozdzielenia czytelnego dokumentu projektu od natywnego
  artefaktu wynikowego oraz raportowania nieznanych pól.

#### Nasher

- URL: <https://github.com/squattingmonk/nasher>
- Licencja: MIT.
- Rola: pakowanie/rozpakowywanie MOD, HAK i ERF, wiele targetów, integracja z
  version control.
- Decyzja: inspiracja dla manifestu i reprodukowalnych buildów. Nie zastępuje
  własnego writera Meshy2Aurora.

#### VFX Explorer Module

- URL:
  <https://neverwintervault.org/project/nwn2/other/vfx-explorer-module>
- Rola: moduł NWN2 iterujący po efektach i pokazujący etykietę wiersza 2DA.
- Ograniczenie: NWN2 używa innego systemu VFX; formaty nie są źródłem prawdy
  dla NWN1.
- Decyzja: przejąć wyłącznie wzorzec galerii i test harnessu.

#### NWN2 Particle Effects Tutorial i Toolset FAQ

- Tutorial:
  <https://neverwintervault.org/project/nwn2/other/visual-effect/nwn2-particle-effects-tutorial>
- FAQ:
  <https://www.neverwintervault.org/project/nwn2/other/nwn2-toolset-beta-test-faq>
- Rola: wzorce kompozycji wielu elementów, timeline/eventów i osobnych plików
  definicji.
- Ograniczenie: inspiracja UX wyłącznie; Electron Engine nie jest Aurorą NWN1.

#### NWNX:EE

- Player VFX API: <https://nwnxee.github.io/unified/group__player.html>
- Effect API: <https://nwnxee.github.io/unified/group__effect.html>
- Debug VFX events:
  <https://nwnxee.github.io/unified/nwnx__events_8nss.html>
- Rola: opcjonalne uruchamianie efektu per gracz z transformacją oraz
  obserwacja debugowych wywołań `dm_visualeffect`.
- Decyzja: późniejszy, opcjonalny test harness dla środowisk NWNX. Podstawowy
  eksport i proof nie mogą wymagać serwera NWNX.

#### Beamdog nwn-issues

- URL: <https://github.com/Beamdog/nwn-issues>
- Rola: pierwotne źródło regresji, zmian renderera i różnic buildów NWN:EE.
- Decyzja: przy każdej funkcji zależnej od wersji sprawdzić tracker i zapisać
  minimalny wspierany build w `targetRuntimeProfile`.

#### Przykładowe shadery NWN:EE

- Tutoriale historyczne:
  <https://github.com/mtijanic/nwnee-shader-tutorials>
- Aktualny przykład materiałów i sparowanych shaderów:
  <https://github.com/Txpple/nwn-vegetation-shaders>
- Status: tutoriale same ostrzegają, że są przestarzałe; vegetation shaders są
  na MIT i zawierają konkretne MTR, VS, FS, teksturę noise i uniformy wiatru.
- Decyzja: pierwszy zestaw tylko edukacyjny; drugi jest dobrym fixture'em do
  walidacji eksportera, po odrębnej decyzji provenance.

## 5. Silniki i edytory VFX jako wzorce produktu

Źródła z tej sekcji nie definiują semantyki Aurory. Służą do projektowania UI,
dokumentu authoringu, deterministycznej symulacji, krzywych, gradientów,
timeline'u, presetów i raportów możliwości.

### 5.1 Najbardziej przydatne implementacje webowe

#### three.quarks

- URL: <https://github.com/Alchemist0823/three.quarks>
- Licencja: MIT.
- Wzorce: data-driven particle system, behaviors, emittery, JSON, wizualny
  editor i preview Three.js.
- Użycie: referencja dla modelu authoringu i serializacji, nie bezpośrednie
  mapowanie wszystkich modułów na Aurorę.

#### babylon.quarks standalone

- URL: <https://github.com/Soullnik/babylon.quarks-standalone>
- Wzorce: inspektor modułów, curves, gradients, timeline, trails, mesh
  particles, sub-emitters, import/export JSON.
- Ryzyko: funkcje GPU i zachowania nowoczesnego silnika często nie mają
  odpowiednika w NWN; exporter musi zgłaszać niezgodność zamiast degradować po
  cichu.

#### Plume

- URL: <https://github.com/travisdmathis/plume>
- Licencja: MIT; status pre-1.0.
- Wzorce: osobne fazy Spawn/Init/Update/Render, node graph, live preview,
  preset gallery, round-trip JSON, deterministyczne demo twins, cleanup
  zasobów podczas hot reloadu.
- Decyzja: bardzo dobry wzorzec architektury IR i UI; nie brać GPU compute jako
  części eksportu Aurora.

#### Sparcoon Editor

- URL: <https://github.com/jango-git/sparcoon-editor>
- Status: pre-release.
- Wzorce: oddzielny behavior graph, render graph i timeline; kompilacja przy
  authoringu; brak `eval` w wyniku; gotowy artefakt zamiast runtime graphu.
- Wniosek: kompilacja grafu do ograniczonego Aurora IR jest bezpieczniejsza niż
  próba zachowania dowolnego grafu w runtime.

#### NewKrok three-particles i editor

- Runtime: <https://github.com/NewKrok/three-particles>
- Editor: <https://github.com/NewKrok/three-particles-editor>
- Wzorce: live update, trail/mesh renderers, soft particles, force fields,
  krzywe i gradient presets, przykłady oraz benchmarki.
- Decyzja: źródło UX i zestawu testów wydajności; eksport NWN musi pozostać
  ograniczony macierzą możliwości.

#### three-nebula

- URL: <https://github.com/creativelifeform/three-nebula>
- Licencja: MIT.
- Wzorce: JSON, sprites i mesh particles, initializer/behavior split oraz
  osobny designer.
- Wartość: prostszy model danych niż pełny VFX Graph, dobry do porównania MVP.

#### Three-VFX

- URL: <https://github.com/mustache-dev/Three-VFX>
- Wzorce: nowoczesny runtime WebGPU i integracja z React Three Fiber.
- Decyzja: obserwować jako opcję eksperymentalnego preview, ale nie wiązać
  dokumentu authoringu z WebGPU ani konkretnym rendererem.

#### webgpu-vfx

- URL: <https://github.com/tigerabrodi/webgpu-vfx>
- Wzorce: rozdzielenie runtime od inspektora/editor UI i podejście WebGPU-first.
- Decyzja: watch list; młody projekt nie powinien stanowić fundamentu formatu.

#### NixieFX

- Dokumentacja: <https://nixiefx.com/vfx-runtime-docs/>
- Repo runtime: <https://github.com/azakhary/nixie-fx>
- Wzorce: manifest, jawne backend capabilities, wymagane zasoby, seeded
  runtime i ten sam bundle dla adapterów 2D/3D.
- Wniosek: nasz export report również powinien wiązać wszystkie zasoby i
  jawnie wykazywać możliwości konkretnego targetu.

#### Babylon.js Node Particle Editor

- Źródło:
  <https://github.com/BabylonJS/Babylon.js/tree/master/packages/tools/nodeParticleEditor>
- Online: <https://npe.babylonjs.com/>
- Wzorce: osadzalny edytor NodeParticleSet, wspólny ekosystem node editorów i
  możliwość działania online/offline.
- Decyzja: wzorzec powłoki node editor, nie format eksportu Aurora.

#### PlayCanvas Editor i particle system

- Editor: <https://github.com/playcanvas/editor>
- Particle system:
  <https://developer.playcanvas.com/user-manual/editor/scenes/components/particlesystem/>
- Wzorce: czytelny inspector, emitter shape, local/world space, sprite sheets,
  prewarm, blend/depth/sort oraz dwie krzywe wyznaczające losowy zakres.
- Wniosek: najlepszy wzorzec formularza dla pojedynczego emitera Aurora.

#### Photon

- URL: <https://github.com/Low-Drag-MC/Photon>
- Wzorce: particle/trail/beam, nieliniowy timeline, shader graph, krzywe,
  gradienty, preset packs i kompletne `.fxpack` z zależnościami.
- Ograniczenie: runtime Minecraft/Java nie jest zgodny z Aurorą.
- Wniosek: dobry wzorzec warstwowego effect packa i authoringu dla osób bez
  wiedzy shaderowej.

### 5.2 Edytory i silniki spoza webu

#### Effekseer

- URL: <https://github.com/effekseer/Effekseer>
- Rola: dojrzały open-source editor efektów, hierarchiczne emittery,
  timeline, krzywe, materiały i wiele backendów.
- Wniosek: wzorzec UX oraz biblioteki presetów; jego runtime i format nie
  powinny stać się zależnością wynikowego HAK-a.

#### Talos

- URL: <https://github.com/rockbite/talos>
- Rola: node-based VFX editor i runtime w ekosystemie libGDX.
- Wzorce: graf modułów, podgląd i kompozycja złożonych efektów.
- Wniosek: graf powinien być nakładką kompozycyjną, a nie bezpośrednią kopią
  parametrów nowoczesnego GPU particle system.

#### Hanabi / bevy_hanabi

- URL: <https://github.com/djeedai/bevy_hanabi>
- Rola: GPU particle system dla Bevy.
- Wzorce: jawne moduły spawn/init/update/render, expressions i testy
  deterministyczności.
- Ograniczenie: GPU behavior znacznie przekracza możliwości Aurory.

#### Godot GPUParticles3D

- URL:
  <https://docs.godotengine.org/en/stable/classes/class_gpuparticles3d.html>
- Rola: oficjalny wzorzec API emitera, draw order, trails, sub-emitter i
  visibility AABB.
- Wniosek: przydatny do nazewnictwa i organizacji UI, nie do automatycznego
  mapowania funkcji.

#### Godot VFX Library

- URL: <https://github.com/haowg/GODOT-VFX-LIBRARY>
- Rola: biblioteka przykładowych efektów i shaderów do analizy konstrukcji
  presetów.
- Decyzja: reference only; żadnych assetów bez weryfikacji licencji każdego
  elementu.

#### Stride particles

- Dokumentacja: <https://doc.stride3d.net/latest/en/manual/particles/>
- Rola: modułowa organizacja emitterów, initializers, updaters, spawners i
  materiały.
- Wniosek: pomocne przy rozdzieleniu edycji od kompilacji do targetu.

#### Unreal Niagara

- Dokumentacja:
  <https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-niagara-effects-for-unreal-engine>
- Rola: referencja klasy produktu: system/emitter/module, data interfaces,
  debug i scalability.
- Decyzja: inspiracja UX i diagnostyki, nie kod ani realistyczny zakres MVP.

#### PopcornFX

- Dokumentacja: <https://www.popcornfx.com/docs/popcornfx-v2/>
- Rola: referencja dla authoringu node-based, bibliotek efektów, profilerów i
  wielu platform docelowych.
- Decyzja: porównanie produktowe; rozwiązanie komercyjne nie jest bazą kodu.

#### Phaser3 Particle Editor

- URL: <https://github.com/koreezgames/phaser3-particle-editor>
- Licencja: MIT; repozytorium zarchiwizowane 2026-08-02.
- Wzorce: mały, prosty edytor i eksport konfiguracji.
- Decyzja: historyczna referencja dla minimalnego UX, nie aktywna zależność.

## 6. UI, timeline, graf i walidacja

#### React Flow / xyflow

- URL: <https://github.com/xyflow/xyflow>
- Rola: dojrzały Reactowy canvas grafu.
- Wniosek: naturalny kandydat dla obecnego React/Vite Studio; decyzja powinna
  wynikać z prototypu dużego grafu i testów undo/redo, nie z popularności.

#### PCUI Graph

- URL: <https://github.com/playcanvas/pcui-graph>
- Licencja: MIT.
- Rola: node graph w TypeScript, porty wejścia/wyjścia, pan/zoom, zapis JSON.
- Decyzja: kandydat porównawczy do React Flow; zależność od JointJS wymaga
  osobnego sprawdzenia rozmiaru i licencji transitive dependencies.

#### React Timeline Editor

- URL: <https://github.com/xzdarcy/react-timeline-editor>
- Rola: timeline i edycja akcji w React.
- Wniosek: kandydat dla warstwy kompozycji, lecz pojedynczy emitter powinien
  najpierw działać bez obowiązkowego timeline'u.

#### Reze Studio

- URL: <https://github.com/AmyangXYZ/reze-studio>
- Rola: browser-native timeline, dopesheet i Bézier curve editor w React 19.
- Wzorce: rozdzielenie dokumentu, selection i transport state; playhead
  aktualizowany poza reaktywnym renderem; preview/commit dla dragów.
- Wniosek: cenne dla płynnego UI krzywych i undo, mimo że domeną jest MMD.

#### Alma

- URL: <https://github.com/emilwidlund/alma>
- Rola: node-based graf generujący WebGL/GLSL.
- Wniosek: wzorzec kompilacji grafu do kodu shadera; eksport NWN musi dodatkowo
  wstrzyknąć i zwalidować prelude/uniformy konkretnego buildu gry.

#### Natron

- URL: <https://github.com/NatronGitHub/Natron>
- Rola: dojrzały node graph, curve editor, dope sheet, cache i render w
  osobnym procesie.
- Wniosek: dobry wzorzec niezawodności edytora i czytelnych plików projektu;
  zakres compositingu jest poza produktem Meshy2Aurora.

#### Graphite

- URL: <https://github.com/GraphiteEditor/Graphite>
- Rola: niedestrukcyjny node graph i architektura edytora grafiki w Rust.
- Wniosek: źródło wzorców dla wersjonowanego dokumentu i grafu, nie VFX oracle.

#### glslang

- URL: <https://github.com/KhronosGroup/glslang>
- Rola: referencyjny parser i validator GLSL, możliwy build JS/WASM.
- Wniosek: dobry lint pierwszego poziomu. NWN używa własnych include'ów,
  uniformów i wariantów, więc sukces glslang nie jest dowodem działania w grze.

#### SPIRV-Cross i SPIRV-Tools

- SPIRV-Cross: <https://github.com/KhronosGroup/SPIRV-Cross>
- SPIRV-Tools: <https://github.com/KhronosGroup/SPIRV-Tools>
- Rola: reflection, walidacja i narzędzia pośrednie dla shaderów.
- Decyzja: opcjonalny eksperyment dla analizy interfejsu VS/FS. Nie wolno
  automatycznie przekształcać shadera NWN przez SPIR-V i zakładać równoważności.

#### Playwright i pixelmatch

- Playwright: <https://github.com/microsoft/playwright>
- pixelmatch: <https://github.com/mapbox/pixelmatch>
- Rola: testy UI, stabilne screenshoty i różnice obrazów preview.
- Ograniczenie: browser visual regression sprawdza nasze Studio, a nie runtime
  Aurory. Nie zastępuje owner proofu Toolset/NWN.

#### Brackeys Particle Controls

- URL: <https://github.com/Brackeys/brackeys-particle-controls>
- Rola: małe kontrolki pracy z particle preview w Godot.
- Wniosek: referencja ergonomii transportu: play, pause, restart, step i speed.

#### pmndrs postprocessing / ShaderPass

- URL: <https://github.com/pmndrs/postprocessing>
- Custom passes:
  <https://github.com/pmndrs/postprocessing/wiki/Custom-Passes>
- Rola: wzorzec rozdzielenia passów i uniformów w webowym preview.
- Ograniczenie: postprocessing preview nie powinien upiększać efektu, jeśli
  odpowiednik nie będzie emitowany do NWN.

## 7. Tekstury, materiały i legalny corpus startowy

#### Kenney Particle Pack

- URL: <https://kenney.nl/assets/particle-pack>
- Licencja: CC0.
- Zawartość: 80 tekstur VFX o rozmiarze 512x512.
- Decyzja: bardzo dobry kandydat na legalny corpus wejściowy do testów importu,
  tintingu, alpha, atlasów i konwersji TGA/DDS. Włączenie plików do repo nadal
  wymaga jawnego manifestu provenance.

#### Material Maker

- URL: <https://github.com/RodZill4/material-maker>
- Rola: proceduralne, node-based tworzenie tekstur i materiałów.
- Wniosek: inspiracja dla późniejszego generatora flipbooków/noise; pełny
  material graph byłby poza pierwszym zakresem VFX Studio.

#### Laigter

- URL: <https://github.com/azagaya/laigter>
- Licencja: GPL-3.0.
- Rola: generowanie normal, specular, occlusion i parallax map, także przez CLI
  i z presetem.
- Decyzja: narzędzie referencyjne lub osobny ręczny etap. Nie integrować kodu
  GPL bez świadomej decyzji licencyjnej.

#### Nora Normal Map Generator

- URL: <https://github.com/josephbk117/Normal-Map-Generator-Tool>
- Licencja: MIT.
- Rola: warstwowa edycja normal map i podgląd PBR na modelu.
- Wniosek: wzorzec diagnostyki map, nie priorytet implementacyjny VFX MVP.

#### Material Map Generator

- URL: <https://github.com/joeyballentine/Material-Map-Generator>
- Licencja: Apache-2.0.
- Rola: automatyczne generowanie normal/displacement/roughness z diffuse.
- Ryzyko: wynik generatywny nie jest fizyczną prawdą materiału; wymaga
  preview, raportu i świadomej akceptacji autora.

## 8. Proponowany dokument projektu VFX

Należy rozdzielić trwały dokument authoringu od natywnych plików wynikowych.
Roboczy schemat `VfxProjectV1`:

```yaml
version: 1
identity:
  project_id: "..."
  target_runtime_profile: "nwn-ee-build-..."
preview:
  random_seed: 1
  fixed_timestep_seconds: 0.0333333333
  prewarm_seconds: 0
  camera_preset: "aurora-default"
emitters: []
timeline:
  duration_seconds: 1
  tracks: []
materials: []
shader_programs: []
aurora_bindings:
  visualeffects_rows: []
  progfx_rows: []
  sounds: []
package:
  resref_namespace: "..."
  requested_hak_name: "..."
capx_report: []
provenance: []
```

To jest wniosek projektowy, nie zatwierdzony jeszcze kontrakt pliku. Przed
implementacją wymaga testu schematu, porównania z aktualnym state Studio i
decyzji właściciela.

## 9. Rekomendowana architektura

```text
React/TypeScript Studio
        |
        v
wersjonowany VfxProject + komendy undo/redo
        |
        v
Rust/WASM: walidator + deterministic emitter compiler/simulator
        |
        +--> browser preview
        |
        +--> own binary MDL/TGA/DDS/MTR/SHD/2DA/HAK writers
                    |
                    v
             own readback + validation report
                    |
                    v
           fixture/gallery MOD -> owner proof
```

Zasady:

- logika symulacji, walidacji i kompilacji należy do Rust/WASM, nie do
  komponentów React;
- UI wysyła intencje i pokazuje readback, a nie utrzymuje drugiego modelu
  fizyki emitera;
- preview używa tego samego skompilowanego IR co exporter;
- kod eksportowany nie zawiera runtime graphu ani `eval`;
- każda funkcja ma target capability i nie może znikać podczas eksportu bez
  błędu lub jawnej aproksymacji zaakceptowanej przez użytkownika;
- binary MDL pozostaje formatem runtime; ASCII jest tylko debug dumpem lub
  golden snapshotem.

## 10. Minimalna macierz możliwości

Przed kodowaniem każde pole powinno otrzymać jeden ze stanów:

```text
verified_exact       potwierdzone przez Aurora/retail i testy
supported_unverified zaimplementowane, ale oczekuje proofu
approximated         preview lub export świadomie przybliża zachowanie
blocked              nie ma bezpiecznego mapowania na target
not_researched       brak wystarczającej wiedzy
```

Pierwszy zakres badań i implementacji:

| Obszar | Priorytet | Warunek |
|---|---:|---|
| birth rate, burst, lifetime | P0 | semantyka i jednostki potwierdzone |
| velocity, spread, mass, gravity, drag | P0 | fixed-seed golden simulation |
| start/mid/end color, alpha i size | P0 | interpolation parity |
| sprite sheet `xgrid/ygrid/fps/frame*` | P0 | TGA/DDS orientation fixture |
| blend, depth i alpha/cutout | P0 | kontrolowany scene fixture |
| local/world transform i animowany parent | P0 | retail multi-animation corpus |
| billboard modes | P0 | kamera i basis golden tests |
| chunk emitter | P1 | własny binary readback i runtime evidence |
| beam, lightning i linked chain | P1 | osobna specyfikacja; bez fallbacku |
| mesh particles | P1 | format i budżet potwierdzone |
| visualeffects/progfx composition | P1 | writer 2DA + collision tests |
| MTR i standardowe shadery | P2 | target build profile |
| własny shader graph | P3 | kompilacja do ograniczonego SHD subsetu |
| postprocessing | poza MVP | tylko gdy istnieje prawdziwy target NWN |

Nazwy i priorytety są rekomendacją badawczą; nie zmieniają aktywnej roadmapy
bez osobnej decyzji.

## 11. Wymagania deterministycznego preview

1. Stały seed zapisany w projekcie i raporcie.
2. Stały krok symulacji oraz jawne oddzielenie render FPS od simulation tick.
3. Ograniczony i raportowany catch-up po długiej klatce.
4. Restart, pause, step, scrub i prewarm bez zmiany wyniku początkowego.
5. Ten sam emitter compiler dla preview i eksportu.
6. Kamera, tło, światło, ground plane i skala zapisane w presecie porównawczym.
7. Readback wyniku po eksporcie jako główny widok `Aurora Preview`.
8. Overlay informacji o wszystkich aproksymacjach i brakach.
9. Brak ukrytego bloom, tone mappingu lub postprocessingu, którego HAK nie
   dostarcza do NWN.

## 12. Proponowane ekrany produktu

### Emitter Inspector

- Basic: enabled, loop, duration, prewarm, seed.
- Spawn: rate, burst, emitter geometry i spread.
- Motion: velocity, random velocity, mass, gravity, drag, space.
- Over Life: size X/Y, rotation, color i alpha jako curve/gradient.
- Render: billboard mode, blend, depth, texture i sprite sheet.
- Aurora: exact nazwa pola MDL, zakres, jednostka i capability state.

### Composition Graph

Graf powinien służyć do kompozycji, nie do ukrywania ograniczeń formatu:

- emitter/model;
- delay i burst;
- impact/duration/cessation;
- dźwięk;
- visualeffects/progfx binding;
- parametr materiału;
- attachment/root/body node;
- wynikowy VFX entry.

Każdy node musi kompilować się do jawnego elementu Aurora IR. Dowolny shader
lub skrypt wykonujący się w edytorze nie jest dozwolonym fallbackiem.

### Timeline

- wiele emitterów i modeli na jednej osi;
- zdarzenia burst/start/stop;
- keyframes transformacji i parametrów możliwych do wyemitowania;
- marker fazy impact/duration/cessation;
- loop range;
- podgląd czasu oraz koszt aktywnych cząstek.

### Validation and Capability Report

- walidacja resrefów i limitu 16 znaków;
- brakujące tekstury, shader pair i include;
- niezgodne sloty MTR i przypadek PLT;
- konflikty HAK/load order i 2DA;
- unsupported emitter/render modes;
- target build i minimalna wersja NWN:EE;
- liczba aktywnych emitterów/cząstek oraz ryzyko first-use stutter;
- lista wszystkich aproksymacji preview.

## 13. Pipeline eksportu

Rekomendowany wynik jednego projektu:

```text
<resref>.mdl          binary model runtime
<texture>.tga/.dds    sprite/atlas/material inputs
<material>.mtr        opcjonalny materiał NWN:EE
<shader-vs>.shd       opcjonalny vertex shader
<shader-fs>.shd       opcjonalny fragment shader
visualeffects.2da     scalony lub fragment do merge
progfx.2da            opcjonalny fragment
<package>.hak         natywny pakiet
<gallery>.mod         opcjonalny moduł testowy
manifest.json         hashe, provenance i zależności
validation-report.json
```

Writer musi:

- deterministycznie przydzielać resrefy lub fail-closed zgłaszać kolizję;
- nigdy nie nadpisywać istniejącego celu po cichu;
- wykonać own readback każdego wygenerowanego formatu;
- raportować wymagany HAK load order;
- generować moduł galerii z etykietą efektu i kontrolowanym placementem;
- pozostawić finalny proof wizualny właścicielowi.

## 14. Strategia testów

### Offline

- syntetyczny emitter dla każdego pola i każdej granicy;
- fixed-seed golden frames/states, nie tylko screenshot;
- round trip `VfxProject -> IR -> binary -> own readback`;
- deterministyczny HAK/MOD build i manifest hashy;
- parser differential tests przeciwko kilku niezależnym implementacjom;
- negatywne testy brakujących tekstur, shaderów, include'ów i kolizji 2DA;
- visual regression browser preview, jawnie oznaczona jako nie-runtime proof.

### Corpus read-only

- wiele realnych emitterów retail/CEP czytanych in-place;
- osobne przypadki ambient, spell impact, duration, destruction, chunk, beam i
  emitter związany z animacją;
- każdy faktycznie uruchomiony model referencyjny zgodnie z polityką `P-REF`.

### Toolset/NWN

- agent przygotowuje exact MOD/HAK, hashe i instrukcję;
- właściciel wykonuje visual proof;
- wynik zapisuje oddzielnie `modelVisibility` i `proofCompleteness`;
- błąd mechanizmu proof nie jest automatycznie błędem modelu ani zgodą na nową
  iterację.

## 15. Priorytety po badaniach

### P0 — warto zrobić najpierw

1. Aurora/retail emitter corpus i pełny typed inventory pól.
2. Własny binary emitter reader/readback w `m2a-core`.
3. Deterministyczny CPU simulator w Rust używany przez WASM preview.
4. Minimalny inspector podobny do PlayCanvas, bez node graphu.
5. Sprite sheets, curves/gradients i jawne billboard modes.
6. Capability report oraz fixture per funkcja.

### P1 — po wiernym pojedynczym emitterze

1. Writer binary emitter node.
2. Kompozycja wielu emitterów na timeline.
3. Writer `visualeffects.2da` i `progfx.2da`.
4. HAK/MOD gallery builder z konfliktem resref/2DA fail-closed.
5. Koszt efektu i rekomendacje dla `cachedmodels.2da`.

### P2 — rozszerzenia NWN:EE

1. MTR editor i texture-slot validation.
2. Sparowany SHD editor z prelude konkretnego buildu.
3. glslang lint oraz własna walidacja include/uniformów NWN.
4. Dynamiczne parametry materiału i kontrolowane presety.

### P3 — później

1. Ograniczony node graph kompilowany do Aurora IR.
2. Biblioteka presetów z pełnym provenance.
3. Opcjonalny NWNX test harness.
4. Import wybranych neutralnych formatów tylko przez jawny capability mapping.

## 16. Czego nie robić

- Nie budować pełnego odpowiednika Niagara przed wiernym pojedynczym emitterem.
- Nie stosować cichego billboard fallbacku dla Beam/Mesh/LinkedChain.
- Nie uznawać ładniejszego preview WebGPU za dowód zgodności z NWN.
- Nie uzależniać HAK-a od webowego runtime'u VFX.
- Nie używać ASCII MDL jako formatu produkcyjnego.
- Nie kopiować kodu GPL ani assetów demonstracyjnych bez decyzji licencyjnej.
- Nie traktować tutoriali NWN2 jako specyfikacji NWN1.
- Nie oznaczać samego sukcesu glslang jako sukcesu shadera w NWN.
- Nie automatyzować finalnego proofu Toolset/NWN wbrew aktualnej decyzji
  właściciela.

## 17. Rejestr dalszych pytań

1. Jaki dokładnie binary layout i controller IDs obowiązują dla każdego trybu
   emittera w aktualnym profilu NWN:EE?
2. Które tryby są wykorzystywane przez retail i które są martwe albo częściowo
   obsługiwane?
3. Jak silnik interpoluje start/mid/end i percent keys przy brakujących polach?
4. Jak zachowują się randomization, fps, frame start/end i texture orientation
   po kompilacji do binary MDL?
5. Jakie są rzeczywiste limity emitterów i cząstek przed degradacją Toolsetu i
   NWN na docelowym sprzęcie?
6. Które ProgFX Types wymagają osobnych modeli, tekstur lub dźwięków?
7. Jak wersjonować stock shader includes bez globalnego override'u?
8. Czy shader graph ma realną wartość przed zakończeniem typed MTR/SHD editor?
9. Jaki zakres CC0 preset corpus może zostać zapisany w `sample-3d` albo
   osobnym, zatwierdzonym katalogu bez naruszenia kanonicznego asset layoutu?

## 18. Zasada kolejnych aktualizacji

Po każdym kolejnym wyszukiwaniu należy:

1. dopisać źródło do odpowiedniej sekcji;
2. zapisać datę weryfikacji i status projektu, jeśli jest zmienny;
3. sprawdzić licencję przed rekomendacją użycia kodu lub assetu;
4. oddzielić fakt źródłowy od wniosku dla Meshy2Aurora;
5. dodać nowy wniosek do architektury, testów lub listy pytań;
6. nie usuwać wcześniejszego wpisu bez zapisu powodu i supersession;
7. nie zmieniać aktywnej roadmapy samym wpisem badawczym.

## 19. Podsumowanie decyzji badawczej

Najmocniejsza droga nie polega na osadzeniu jednego z istniejących runtime'ów
VFX. Meshy2Aurora ma już właściwy stos dla tego zadania: React/Vite dla Studio
oraz Rust/WASM dla wspólnej logiki i natywnych writerów. Zewnętrzne projekty
najlepiej wykorzystać jako:

- Radoub, rollNW i nwn_mdl_webviewer — porównawcze oracle semantyki;
- NWN Wiki, retail i dekompilacja — wejście do własnego kontraktu;
- PlayCanvas — wzorzec pojedynczego inspectora;
- Plume, Sparcoon, Photon i Babylon NPE — wzorce późniejszej kompozycji;
- glslang — pomocniczy lint shaderów;
- Kenney Particle Pack — potencjalny legalny corpus tekstur;
- VFX Explorer — wzorzec modułu galerii;
- własny binary readback i owner proof — ostateczna bramka produktu.

## 20. Runda internetowa R2 — 2026-09-02

Zakres rundy: brakujące narzędzia NWN/Aurora, semantyka aplikowania efektów,
grafy i timeline, dokument projektu, deterministyczny preview, flipbooki,
diagnostyka wydajności i ograniczenia licencyjne. Źródła powtarzające wpisy z
sekcji 4–7 zostały użyte do porównania, ale nie otrzymały duplikatów.

### 20.1 Nowe źródła NWN/Aurora

#### NeverBlender 4.1.0 oraz ograniczenie podglądu emitterów

- Strona wydania:
  <https://neverwintervault.org/project/nwnee/other/tool/neverblender-40>
- Dyskusja o tworzeniu VFX:
  <https://forum.neverwintervault.org/t/neverblender-creating-new-spell-effect-vfx-from-scratch/4221>
- Status zweryfikowany 2026-09-02: wydanie 4.1.0 dla Blender 4.0/3.6; strona
  Vault oznacza licencję jako `Open - Free & open only if project also open`.
- Fakt źródłowy: add-on importuje i eksportuje format MDL. Według odpowiedzi
  autora/społeczności podstawowe właściwości emittera można przechować i
  edytować, ale cząstki NWN nie są renderowane w Blenderze, ponieważ nie są
  zgodne z jego systemem cząstek.
- Wniosek: NeverBlender jest ważnym testem interoperacyjności import/export,
  ale nie jest oracle podglądu. Brak renderowania wzmacnia sens własnego,
  wyspecjalizowanego preview Meshy2Aurora.
- Decyzja: `reference-only`; nie kopiować add-onu, manuala ani fixture. Wersję
  Blender/NeverBlender traktować jako parę, a nie niezależne „latest”.

#### niv/nwn-tools i nwnmdlcomp

- Repozytorium: <https://github.com/niv/nwn-tools>
- Implementacja kompilatora:
  <https://github.com/niv/nwn-tools/blob/master/nwnmdlcomp/nwnmdlcomp.cpp>
- Status zweryfikowany 2026-09-02: historyczny compiler/decompiler modeli i
  skryptów. Nagłówek źródła `nwnmdlcomp` zawiera liberalne warunki
  redystrybucji typu BSD.
- Przydatność: dodatkowy oracle historycznego układu binary MDL, kolejności
  bloków i konwersji ASCII/binary.
- Ograniczenie: nie zakładać kompletnej obsługi NWN:EE ani nowych pól. Wynik
  narzędzia nie może zastąpić realnego retail corpus, dekompilacji Aurory,
  własnego writera i własnego readbacku.
- Decyzja: `reference-only / differential oracle` po przypięciu dokładnej
  rewizji i zapisaniu provenance.

#### NWN Wiki: Model Table of Parameters

- URL: <https://nwn.wiki/spaces/NWN1/pages/53671005/Model%2BTable%2Bof%2BParameters>
- Status zweryfikowany 2026-09-02: rozwijana tabela parametrów ASCII MDL,
  typów węzłów i ograniczeń nazw.
- Przydatność: zbiera wymagania przekrojowe, które dotyczą również modeli VFX:
  unikalność i długość nazw węzłów, znaczenie kolejności węzłów, główny dummy,
  limity resrefów oraz różnice między tekstowym polem a binary storage.
- Wniosek: walidator VFX nie może sprawdzać tylko pól emittera. Musi uruchamiać
  również wspólne gate'y modelu, hierarchii, nazw, rodziców i kolejności.
- Ograniczenie: część opisów jest społecznościową hipotezą; każde wymaganie
  runtime trzeba potwierdzić według hierarchii `Aurora First`.

#### NWN Lexicon: aplikowanie efektów i beamów

- `EffectVisualEffect`:
  <https://nwnlexicon.com/EffectVisualEffect>
- `EffectBeam`: <https://www.nwnlexicon.com/EffectBeam>
- `ApplyEffectAtLocation`:
  <https://nwnlexicon.com/ApplyEffectAtLocation>
- Status zweryfikowany 2026-09-02: dokumentacja społecznościowa NWScript,
  zawierająca także dopiski NWN:EE i znane błędy.
- Przydatność: pokazuje, że gotowy zasób to nie tylko model i wiersz 2DA.
  Sposób uruchomienia zależy od kategorii `Type_FD`, celu (obiekt/lokacja),
  czasu trwania, body node, transformacji i specjalnej semantyki beamów.
- Wniosek: kompozycja powinna mieć jawny `application profile`, np.
  `instant_at_location`, `instant_on_object`, `duration_on_object` albo
  `beam_between_objects`, z walidacją zgodności z `visualeffects.2da` i
  `progfx.2da`.
- Wniosek testowy: generator fixture/handoff powinien generować właściwy
  scenariusz NWScript dla klasy efektu; jeden uniwersalny skrypt demonstracyjny
  może dawać fałszywie negatywne wyniki.
- Ograniczenie: znane błędy, m.in. zachowanie duration/placeable i ponowne
  wejście w zasięg, wymagają oddzielnych testów runtime właściciela.

### 20.2 Nowe wzorce edytorów i runtime'ów

#### Unity Visual Effect Graph

- Logika grafu:
  <https://docs.unity.cn/Packages/com.unity.visualeffectgraph@16.0/manual/GraphLogicAndPhilosophy.html>
- Contexts:
  <https://docs.unity.cn/Packages/com.unity.visualeffectgraph@8.1/manual/Contexts.html>
- Events:
  <https://docs.unity.cn/Packages/com.unity.visualeffectgraph@10.2/manual/Events.html>
- Krzywe właściwości:
  <https://docs.unity.cn/6000.0/Documentation/Manual/varying-particle-system-properties-over-time.html>
- Status: własnościowe narzędzie i dokumentacja Unity; tylko wzorzec produktu.
- Najważniejszy wzorzec: rozdzielenie cyklu na `Spawn`, `Initialize`, `Update`
  i `Output`, z typowanymi połączeniami, eventami i blokami wykonywanymi w
  jawnej kolejności.
- Wniosek: przyszły graph Meshy2Aurora powinien najpierw odzwierciedlać
  potwierdzony cykl życia Aurory. Nie należy kopiować pełnej ontologii Unity,
  lecz jej podział dobrze porządkuje capability mapping i komunikaty o
  funkcjach niemożliwych do eksportu.

#### mo.js

- Repozytorium: <https://github.com/mojs/mojs>
- Licencja: MIT; zweryfikowana 2026-09-02.
- Przydatność: osobne moduły player, curve editor i timeline editor oraz
  deklaratywne efekty `burst`, `shape`, `swirl` i `stagger`.
- Wniosek: dobry, mały wzorzec interakcji dla edycji krzywych i burstów.
  Nie jest rendererem 3D NWN ani podstawą symulacji.
- Decyzja: `UX reference`; ewentualne użycie kodu wymaga osobnej decyzji
  dependency/provenance.

#### Motion Graphics Studio

- Repozytorium: <https://github.com/brylie/motion-graphics-studio>
- Status zweryfikowany 2026-09-02: SvelteKit/TypeScript/WebGL2, timeline SVG,
  automation lanes, warstwy shaderów, scrubbing i Playwright.
- Licencja: plik `LICENSE` zawiera Apache-2.0, ale README deklaruje MIT.
  To niespójność metadanych, więc bez wyjaśnienia nie wolno kopiować kodu.
- Przydatność: wzorzec osi czasu z osobnymi lane'ami automatyzacji oraz testów
  edytora. Svelte nie pasuje bezpośrednio do stosu React Meshy2Aurora.
- Decyzja: `reference-only`, z blokadą reuse do czasu jednoznacznego audytu
  licencji dla konkretnej rewizji.

#### FableCut

- Repozytorium: <https://github.com/ronak-create/FableCut>
- Licencja: MIT; zweryfikowana 2026-09-02.
- Przydatność: cała oś czasu jest jednym czytelnym `project.json`; UI i agent
  edytują ten sam dokument, dostępne są keyframes, easing, wykres wartości,
  undo/redo oraz szybkie przeładowanie projektu.
- Wniosek: dobry wzorzec „project document is the interface”. VFX Studio
  powinno umożliwiać deterministyczne odtworzenie dokładnie tego samego efektu
  z pliku, bez ukrytego stanu UI.
- Ograniczenie: to młody edytor wideo, nie system cząstek ani format NWN.
- Decyzja: `architecture/UX reference`.

#### Party

- Repozytorium: <https://github.com/cazala/party>
- Licencja: MIT; zweryfikowana 2026-09-02.
- Przydatność: wspólne moduły symulacji z dwiema implementacjami WebGPU i CPU,
  serializacja sesji, oscylatory parametrów, undo/redo i interaktywny
  playground React.
- Wniosek: rozdzielenie kontraktu modułu od backendu obliczeń jest dobrym
  wzorcem. Dla Meshy2Aurora CPU/reference path może służyć deterministycznym
  testom, a szybszy preview nie może zmieniać semantyki.
- Ograniczenie: fizyka Party jest znacznie szersza niż Aurora i nie jest
  oracle zgodności NWN.
- Decyzja: `architecture reference`; nie osadzać runtime'u bez odrębnej
  decyzji.

#### threeparticles

- Repozytorium: <https://github.com/threeparticles/threeparticles>
- Edytor: <https://editor.threeparticles.com>
- Licencja: MIT; zweryfikowana 2026-09-02.
- Przydatność: mały, konfiguracyjny silnik WebGPU dla Three.js z gradientami,
  alpha i podstawowym emitterem oraz oddzielnym edytorem wizualnym.
- Ryzyko: bardzo młody projekt o minimalnej adopcji i bez wykazanego CPU
  reference path. API/status należy uznać za niestabilny.
- Decyzja: `watchlist`, nie rekomendowane dependency P0/P1.

#### cel-lab

- Repozytorium: <https://github.com/FoundryLogger/cel-lab>
- Licencja: GPL-3.0-or-later; zweryfikowana 2026-09-02.
- Przydatność: całkowicie lokalny edytor w jednym HTML, keyframe timeline,
  krzywe Béziera, presety JSON, billboard particles, proceduralne shadery i
  warianty eksportu.
- Wniosek: interesujący wzorzec szybkiej iteracji, presetów i kontrolowanego
  randomizera, ale zestaw funkcji jest stylizowany i nie odpowiada Aurorze.
- Decyzja: `behavior/UX reference only`; GPL wyklucza kopiowanie do produktu.

#### Premation

- Repozytorium: <https://github.com/isroil01/premation>
- Licencja i status: AGPL-3.0-only, pre-1.0; zweryfikowane 2026-09-02.
- Przydatność: jeden silnik renderuje viewport i eksport, projekt jest
  katalogowym bundlem z osobnymi JSON-ami, assetami adresowanymi SHA-256,
  lokalną historią wersji i manifestem zapisywanym na końcu. Projekt deklaruje
  również golden-image tests oraz deterministyczny eksport.
- Wniosek: dobry wzorzec atomicznego zapisu, content-addressed assets i
  wspólnego kontraktu preview/export. Meshy2Aurora nie powinno kopiować jego
  implementacji ani przechodzić na Electron.
- Decyzja: `reference-only`; AGPL i pre-1.0 są twardą czerwoną flagą.

#### Theatre.js

- Repozytorium: <https://github.com/theatre-js/theatre>
- Licencje: większość core Apache-2.0, natomiast `@theatre/studio` jest
  AGPL-3.0; zweryfikowane 2026-09-02.
- Przydatność: wysokiej jakości sekwencje, keyframes i edycja dowolnych
  zmiennych JS, także scen Three.js.
- Wniosek: silny wzorzec timeline/curve UX, ale najciekawsza dla nas część
  edytorska ma inną, copyleftową licencję niż runtime.
- Decyzja: `UX reference`; nie traktować licencji core jako zgody na osadzenie
  Studio.

### 20.3 Graf, inspector i format projektu

#### Rete.js

- Repozytorium: <https://github.com/retejs/rete>
- Dokumentacja licencji: <https://retejs.org/docs/licensing/>
- Licencja: core i większość pakietów MIT. Niektóre zaawansowane pluginy,
  m.in. wskazane w dokumentacji `rete-structures` i `rete-scopes-plugin`, mają
  CC-BY-NC-SA-4.0; zweryfikowane 2026-09-02.
- Przydatność: TypeScript, renderer React, typowane sockety, dataflow/control
  flow oraz osobne przetwarzanie grafu po stronie klienta lub serwera.
- Wniosek: dobry wzorzec typowania połączeń i oddzielenia modelu grafu od UI.
  React Flow pozostaje prostszym kandydatem UI; Rete wymagałoby pełnej macierzy
  licencji wszystkich pluginów przed adopcją.
- Decyzja: `evaluate later`, bez zmiany obecnego kierunku.

#### LiteGraph.js

- Repozytorium: <https://github.com/jagenjo/litegraph.js>
- Licencja: MIT; zweryfikowana 2026-09-02.
- Przydatność: graf eksportowany jako JSON, możliwy do wykonania bez edytora,
  subgraphs i obsługa setek węzłów.
- Wniosek: potwierdza potrzebę oddzielenia serializacji/ewaluacji od canvasu,
  ale jego własny renderer Canvas2D i starszy styl API nie są naturalnym
  rozszerzeniem obecnego React Studio.
- Decyzja: `serialization/runtime reference`, nie domyślny komponent UI.

#### BaklavaJS

- Repozytorium: <https://github.com/newcat/baklavajs>
- Licencja: MIT; zweryfikowana 2026-09-02.
- Przydatność: TypeScript, typowane interfejsy, automatyczne konwersje typów i
  osobny engine.
- Ograniczenie: oficjalny renderer jest oparty na Vue, podczas gdy produkt
  używa React.
- Decyzja: `concept reference`; brak przewagi uzasadniającej drugi framework
  UI.

#### Tweakpane

- Repozytorium: <https://github.com/cocopon/tweakpane>
- Licencja: MIT; zweryfikowana 2026-09-02.
- Przydatność: mały inspector dla liczb, kolorów i wektorów, foldery/tabs,
  monitoring wartości oraz import/export JSON.
- Wniosek: dobre źródło benchmarku ergonomii inspectora i plugin API.
  Gotowy wygląd nie powinien zastąpić design systemu istniejącego Studio.
- Decyzja: `UX benchmark`; dependency tylko po osobnym spike'u.

#### JSON Schema Draft 2020-12

- Specyfikacja: <https://json-schema.org/draft/2020-12>
- Przydatność: jawna walidacja struktury, wersjonowane `$schema`, `$defs`,
  tuple/array constraints oraz ustandaryzowany format wyników walidacji.
- Wniosek: `VfxProjectV1` powinien mieć maszynowo sprawdzalny schema artifact
  albo równoważny kontrakt generowany z jednego źródła typów. YAML może
  pozostać formatem użytkownika po sparsowaniu do tego samego modelu danych.
- Ograniczenie: schema sprawdza strukturę, ale nie zastąpi semantycznych gate'ów
  Aurory, readbacku ani capability report.

#### OpenTimelineIO

- Repozytorium:
  <https://github.com/AcademySoftwareFoundation/OpenTimelineIO>
- Serializowany schema:
  <https://github.com/AcademySoftwareFoundation/OpenTimelineIO/blob/main/docs/tutorials/otio-serialized-schema.md>
- Licencja: Apache-2.0; stabilny i szeroko wdrożony projekt; zweryfikowane
  2026-09-02.
- Przydatność: `RationalTime`, tracks, clips, markers, schema versions,
  downgrade i adaptery formatów.
- Wniosek: nie adoptować całego OTIO dla emitera, ale przejąć idee jawnej bazy
  czasu, wersjonowanych obiektów i migracji/downgrade testowanych na goldenach.
- Decyzja: `schema/time reference`.

#### glTF: KHR_animation_pointer i EXT_mesh_gpu_instancing

- Animowanie właściwości:
  <https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_animation_pointer/README.md>
- Instancing:
  <https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Vendor/EXT_mesh_gpu_instancing/README.md>
- Status: oba rozszerzenia są oznaczone jako ratyfikowane; zweryfikowane
  2026-09-02.
- Przydatność: `KHR_animation_pointer` pokazuje jawne adresowanie animowanej
  właściwości i ograniczenia interpolacji według typu; instancing pokazuje
  rozdzielenie wspólnej geometrii od per-instance transform/koloru.
- Wniosek: te mechanizmy mogą inspirować wewnętrzne typed property paths i
  bufor preview dla mesh particles. Nie są formatem eksportu do NWN i nie
  dowodzą obsługi odpowiedników w Aurorze.

### 20.4 Tekstury i flipbooki

#### DirectXTex / texconv

- Repozytorium: <https://github.com/microsoft/DirectXTex>
- Dokumentacja texconv:
  <https://github.com/microsoft/DirectXTex/wiki/Texconv>
- Licencja: MIT; zweryfikowana 2026-09-02.
- Przydatność: odczyt/zapis DDS i TGA, resize, mipmaps, block compression oraz
  narzędzia diagnostyczne.
- Wniosek: wartościowy zewnętrzny oracle diagnostyczny dla tekstur, ale nie
  wolno uczynić go writerem produktu ani założyć, że każdy standardowy DDS jest
  akceptowany przez NWN. Canonical writer/readback pozostaje własny.
- Decyzja: `optional differential oracle`, exact revision + provenance.

#### Atlast i Atlasify

- Atlast: <https://github.com/spencerjbeckwith/atlast>
- Atlasify: <https://github.com/soimy/atlasify>
- Licencje: MIT; zweryfikowane 2026-09-02.
- Przydatność: powtarzalne pakowanie obrazów, padding/extrusion i metadata JSON.
- Krytyczne ograniczenie: ogólny max-rects atlas nie odpowiada semantyce
  regularnej siatki `xgrid × ygrid` emittera NWN.
- Wniosek: Meshy2Aurora potrzebuje własnego deterministycznego packera
  **uniform-grid flipbook**, który waliduje identyczny rozmiar klatek, kolejność,
  alpha, orientację, puste komórki, `frameStart/frameEnd` i hash wyniku.
- Decyzja: projekty są wzorcami preprocessing/metadata, nie gotowym exporterem
  NWN.

#### EmberGen, IlluGen i VectorayGen

- EmberGen: <https://docs.jangafx.com/embergen/index.html>
- Export flipbooków:
  <https://docs.jangafx.com/embergen/pages/references/node_list.html>
- VectorayGen: <https://jangafx.com/software/vectoraygen>
- Status: zewnętrzne narzędzia JangaFX; pobranie lub darmowy wariant nie
  oznacza otwartej licencji kodu ani assetów.
- Przydatność: proceduralne tworzenie flipbooków, channel packing, frame
  stride, wiersze/kolumny, flow maps i vector fields.
- Wniosek: mogą być opcjonalnym źródłem **własnych** tekstur użytkownika, ale
  Meshy2Aurora powinno przyjmować wynik przez udokumentowany import i
  walidację, nie integrować zamkniętego runtime'u.
- Decyzja: `external authoring reference`; provenance i licencja każdego
  importowanego assetu pozostają obowiązkowe.

### 20.5 Diagnostyka wydajności i zgodności preview

#### Unity bounds, Niagara profiling i Three.js transparency

- Bounds VFX:
  <https://docs.unity.cn/Packages/com.unity.visualeffectgraph@15.0/manual/visual-effect-bounds.html>
- Niagara performance:
  <https://dev.epicgames.com/documentation/unreal-engine/measuring-performance-in-niagara>
- Three.js transparency:
  <https://threejs.org/manual/en/transparency.html>
- Fakty porównawcze: zbyt małe bounds prowadzą do znikania widocznych cząstek,
  zbyt duże do zbędnej pracy; koszt VFX zależy od scenariusza wielu instancji,
  ticków i overdraw; sortowanie przezroczystości ma fundamentalne kompromisy.
- Wniosek dla Meshy2Aurora: capability/performance report powinien osobno
  podawać co najmniej:
  - szczytową liczbę żywych cząstek per emitter i kompozycję;
  - szacunkowe bounds w czasie oraz przekroczenie deklarowanego zasięgu;
  - liczbę aktywnych emitterów/instancji i translucent layers;
  - orientacyjny screen coverage/overlap dla reprezentatywnych kamer;
  - blend mode, `renderorder`, sort mode i ryzyko różnicy preview/runtime;
  - czas symulacji CPU/reference oddzielnie od czasu renderu preview.
- Ograniczenie: nie przenosić budżetów Unity/Unreal do NWN. Progi produktu
  muszą wynikać z pomiarów na retail corpus i owner runtime proof.

### 20.6 Wyniki odrzucone albo zdegradowane

- Tutoriale i FAQ NWN2 ponownie odrzucono jako specyfikację NWN1. Zachowują
  wartość wyłącznie jako wzorzec UX kompozycji SEF/eventów.
- Posty Reddit bez repozytorium, dokumentacji lub powtarzalnego przykładu nie
  zostały awansowane do źródeł implementacyjnych.
- Ogólne demonstratory flow-field, fluid/SPH i „miliony cząstek WebGPU” nie
  rozwiązują zgodności z emitterem Aurory; trafiają najwyżej na watchlistę.
- KTX2/BasisU nie jest formatem docelowym NWN. Może kiedyś zoptymalizować
  wyłącznie webowy cache preview, ale nie powinien komplikować P0/P1.
- Ogólne max-rects atlasy zostały odrzucone jako bezpośredni eksport
  `xgrid/ygrid`, ponieważ zmienny prostokąt klatki łamie model siatki NWN.
- Projekty GPL/AGPL pozostają referencją zachowania i UX; brak bezpośredniego
  kopiowania obejmuje również „małe” komponenty edytora.

### 20.7 Nowe wnioski architektoniczne R2

1. **Dwa poziomy authoringu, nie jeden wielki graph.** P0/P1 powinno używać
   typed inspectora i timeline'u zgodnych z emitterem NWN. Graph staje się
   warstwą kompozycji dopiero po potwierdzeniu mapowania cyklu życia.
2. **Jawny lifecycle IR.** Wewnętrzny model powinien rozróżniać spawn,
   inicjalizację, update, render/output i event/application, nawet jeśli binary
   MDL koduje część z nich inaczej.
3. **Application profile jest częścią artefaktu.** Sam wpis 2DA nie wystarcza;
   eksport i fixture muszą wiedzieć, czy efekt jest location/object/duration/
   beam oraz jakich body nodes potrzebuje.
4. **Jedno źródło stanu.** Projekt na dysku musi całkowicie odtwarzać UI,
   preview i eksport. Kamera robocza może być metadata, ale nie może zmieniać
   skompilowanego efektu bez jawnej operacji.
5. **CPU/reference przed GPU-fast path.** Deterministyczna ścieżka odniesienia
   jest ważniejsza od liczby cząstek. WebGPU można dołączyć później jako
   optymalizację z testami zgodności wyników statystycznych i snapshotów.
6. **Uniform-grid flipbook jako funkcja produktu.** Import sekwencji, packing,
   podgląd klatek, kontrola alpha/orientacji i zapis `xgrid/ygrid/fps` powinny
   być jednym powtarzalnym workflow.
7. **Walidacja ma trzy osie.** Należy rozdzielić: zgodność formatu, wspieranie
   semantyki przez preview oraz ryzyko wydajności/runtime. Zielony schema nie
   oznacza zielonego capability ani proofu.
8. **Licencja per pakiet, nie per marka.** Theatre.js i Rete.js pokazują, że
   różne części jednego ekosystemu mogą mieć różne licencje; manifest dependency
   musi przechowywać dokładny pakiet, wersję, rewizję i licencję.
9. **Preview nie powinno ukrywać problemów przezroczystości.** UI powinno
   oferować diagnostyczny widok render order/overdraw i jawnie oznaczać miejsca,
   gdzie Three.js tylko przybliża kolejność Aurory.
10. **Atomiczny zapis i migracje są funkcją jakości.** Nowy schema wymaga
    goldenów migracji `V1 → Vn`, jawnej bazy czasu oraz zapisu manifestu po
    payloadach, aby przerwany zapis nie stworzył pozornie poprawnego projektu.

### 20.8 Kandydaci do kolejnych spike'ów po aktywnych gate'ach

Poniższa lista nie zmienia aktywnej roadmapy i nie autoryzuje implementacji:

1. corpus-driven mapa `Emitter field → lifecycle stage → binary controller`;
2. typed `applicationProfile` powiązany z `Type_FD` i fixture NWScript;
3. deterministyczny uniform-grid flipbook packer z TGA/DDS readbackiem;
4. CPU reference simulation z seedem, fixed timestep i rewind/prewarm;
5. porównanie React Flow z Rete core wyłącznie na minimalnym typed graph spike;
6. performance report: live count, bounds, overlap, instance scenario;
7. schema/migration golden oparty na jednym `VfxProjectV1` source of truth;
8. licencyjny SBOM/provenance dla każdego faktycznie wybranego dependency.

### 20.9 Nowe pytania po R2

1. Czy każdy `Type_FD` ma jednoznaczny poprawny application profile, czy część
   wierszy retail łamie kategorię?
2. Które body nodes działają dla creature, placeable i door w exact runtime
   profile oraz jaki jest fallback niepoprawnego node'a?
3. Czy bounds emittera istnieją jawnie w binary MDL, czy trzeba je estymować z
   rozkładu pozycji i czasu życia?
4. Jaka jest zgodna z Aurorą kolejność blend/render dla wielu emitterów o tym
   samym `renderorder`?
5. Czy preview wymaga dokładnego odwzorowania klientowego cullingu efektów po
   wejściu/wyjściu z zasięgu, czy wystarczy osobny runtime warning?
6. Które formaty DDS/TGA, mipmapy i alpha są faktycznie akceptowane przez exact
   build NWN:EE dla texture-grid emitterów?
7. Czy projekt powinien być pojedynczym plikiem z asset references, czy
   atomowym katalogowym bundlem z content-addressed payloadami?
8. Jak mierzyć screen-space overdraw w podglądzie tak, aby wynik był stabilny i
   użyteczny, ale nie udawał profilu GPU klienta NWN?

## 21. Runda internetowa R3 — 2026-09-02

Zakres rundy: rozwiązywanie zasobów i formatów tekstur przez NWN, specjalne
węzły przyłączenia VFX, conjuration/casting i efekty trwałe, TXI, authoring
shaderów, diagnostyka WebGL/WebGPU oraz deterministyczne testy między
TypeScript i Rust/WASM. Źródła już obecne w sekcjach 4–20 zostały użyte jako
punkty odniesienia bez tworzenia powtórnych wpisów.

### 21.1 NWN: zasoby, węzły przyłączenia i rodziny efektów

#### NWN Wiki: formaty tekstur i ich zastosowanie

- Tekstury: <https://nwn.wiki/spaces/NWN1/pages/38174958/Textures>
- MDL ASCII:
  <https://nwn.wiki/spaces/NWN1/pages/12027273/MDL%2BASCII>
- Status zweryfikowany 2026-09-02: dokumentacja społecznościowa NWN:EE.
- Fakty przydatne dla produktu: modele mogą odwoływać się między innymi do
  MTR, DDS, TGA i PLT, a TXI o tej samej nazwie jest ładowany jako sidecar.
  Zastosowanie formatów nie jest zamienne: wiki rekomenduje DDS dla typowych
  tekstur modeli, ale wskazuje szczególne przypadki TGA i PLT oraz różnice
  między grą, Toolsetem i NUI.
- Wniosek: `TextureAssetV1` potrzebuje jawnych pól `role`, `sourceFormat`,
  `targetFormat`, `alphaMode`, `compression`, `mipmapPolicy`, `dimensions`,
  `txiSidecar` i `runtimeProfile`. Samo rozszerzenie pliku nie wystarcza do
  oceny zgodności.
- Gate: writer musi po zapisie odczytać nagłówek i payload, potwierdzić format,
  wymiary, mipmapy i alpha, a następnie włączyć te dane do capability report.

#### NWN Wiki: Content Load Order

- URL: <https://nwn.wiki/spaces/NWN1/pages/38174823/Content%2BLoad%2BOrder>
- Kluczowy fakt: zwycięski zasób zależy równocześnie od typu kontenera i
  formatu. Dla DDS/TGA opisano osobny porządek między override/development,
  NWSync, ERF/HAK i BIF. Dwa pliki o tej samej nazwie nie muszą więc zachowywać
  się jak prosty „DDS zawsze wygrywa z TGA”.
- Dodatkowe ryzyko: kolejność key tables w tej samej kategorii nie jest
  gwarantowana, a nakładające się zasoby w wielu HAK-ach mogą dawać inny wynik
  niż podgląd pojedynczego katalogu roboczego.
- Wniosek: eksport powinien generować **Resource Resolution Report**, który
  dla każdego `(resref, resourceType)` pokazuje wszystkich kandydatów,
  kontener, priorytet, hash i przewidywanego zwycięzcę. Kolizja o
  niejednoznacznej kolejności jest błędem fail-closed, nie ostrzeżeniem.
- Preview powinno wyświetlać provenance faktycznie wczytanego zasobu. Zielona
  miniatura bez informacji o źródle może ukryć, że autor ogląda plik inny niż
  trafi do HAK/MOD.

#### Model Special Nodes, spells.2da i conjuration/casting

- Specjalne węzły modeli:
  <https://nwn.wiki/spaces/NWN1/pages/38176272/Model%2BSpecial%2BNodes>
- `spells.2da`:
  <https://nwn.wiki/spaces/NWN1/pages/38175046/spells.2da>
- Conjuration i casting VFX:
  <https://nwn.wiki/spaces/NWN1/pages/38176363/Conjuration%2Band%2BCasting%2BVisual%2BEffects>
- Fakty: `headconjure`, `handconjure`, `root`, `head` i `impact` mają odrębne
  role; część węzłów ma wymaganych rodziców, a dostępność zależy od typu modelu
  stworzenia. `spells.2da` rozdziela wizuale głowy, dłoni i ziemi oraz fazy
  conjuration/casting. Nie każdy model ma każdy punkt przyłączenia.
- Wniosek: `applicationProfile` z R2 wymaga rozszerzenia o:
  `targetModelClass`, `attachmentSemantic`, `requiredNode`,
  `expectedParent`, `fallbackPolicy` i `spellPhase`.
- UI nie powinno prezentować dowolnego tekstowego body node. Powinno oferować
  typowaną listę zależną od profilu i wyświetlać brak wymaganego węzła jako
  błąd scenariusza testowego, a nie „model niewidoczny”.
- Fixture dla efektu zaklęcia musi osobno sprawdzać fazę conjuration i cast;
  jeden zrzut po rzuceniu czaru może nie obejmować krótkiej fazy wcześniejszej.

#### Persistent VFX to oddzielna rodzina eksportu

- Nowy fakt uzyskany przez krzyżowe sprawdzenie istniejących źródeł
  `2da Files` i `progfx.2da`: `vfx_persistent.2da` obsługuje zasoby tworzone
  przez `EffectAreaOfEffect`, ale nie może korzystać z `progfx.2da` poza
  historycznym, specjalnym przypadkiem wbudowanym w silnik.
- Limity wierszy i zachowanie efektu mobilnego/statycznego zależą również od
  profilu wersji NWN:EE; po zmianach 8193.36 nie należy utrwalać dawnych
  założeń o ośmiobitowym zakresie jako jednego globalnego limitu.
- Wniosek: dokument projektu powinien rozróżniać co najmniej
  `visualEffect2da`, `persistentAreaEffect2da` oraz `spellConjuration`.
  `persistent=true` na zwykłym efekcie byłoby mylącym i niepełnym modelem.
- Capability report musi jawnie powiedzieć, że programowane składowe progfx
  nie przechodzą do rodziny persistent, zamiast po cichu je usuwać.

#### TXI: format tekstowy wymaga zachowania nieznanych dyrektyw

- Historyczny przykład BioWare:
  <https://www.neverwintervault.org/article/tutorial/bioware-txi-example>
- Typowany parser `nwnrs-types`:
  <https://docs.rs/nwnrs-types/latest/nwnrs_types/txi/index.html>
- Przykład BioWare dokumentuje między innymi `blending`, `mipmap`, `filter`,
  `gamma`, proceduralne tekstury, wielowierszowe `channelscale` i
  `channeltranslate`. Jest wartościowym corpus source, ale nie pełną
  współczesną specyfikacją NWN:EE.
- Dokumentacja `nwnrs-types` pokazuje użyteczny wzorzec: zachowanie kolejności
  dyrektyw, continuation lines i nieznanych wpisów, a pola typowane są widokiem
  nad oryginalnym strumieniem zamiast jego stratnym zamiennikiem.
- Wniosek: własny parser/writer TXI powinien być **lossless-by-default**.
  Znane dyrektywy dostają typowaną walidację, nieznane pozostają w AST i
  round-tripie z ostrzeżeniem. Uproszczony parser `split whitespace` utraci
  strukturę i może zmienić znaczenie wielowierszowych wartości.
- Ważne rozdzielenie: animacja siatki emitera (`xgrid/ygrid` i kontrolery MDL)
  oraz animacja/procedura tekstury w TXI to dwa mechanizmy. UI i walidator nie
  mogą scalać ich w jedno pole „flipbook FPS”.
- Decyzja: oba źródła `reference/corpus only`; zgodnie z regułami projektu
  produkt zachowuje własny parser, writer i readback.

#### NWN Crunch i macierz profili DDS

- URL: <https://nwn.wiki/spaces/NWN1/pages/60984230/NWN%2BCrunch%2BGuide>
- Przydatność: opisuje warianty DXT1/DXT5A/DXN, alpha threshold, generowanie
  mipmap, filtry, gamma, renormalizację normal map i konwersję źródeł.
- Wniosek: zamiast jednego przycisku „Export DDS” potrzebna jest wersjonowana
  macierz `TextureEncodingProfile`, powiązana z rolą mapy. Przykładowo diffuse
  bez alpha, diffuse z alpha, normal, specular i illumination nie powinny
  dzielić automatycznie tych samych parametrów kompresji.
- NWN Crunch może być zewnętrznym differential oracle. Nie staje się writerem
  produktu i nie zastępuje odczytu wyjścia ani proofu właściciela.

#### NWN Explorer jako negatywny oracle

- URL: <https://nwn.wiki/spaces/NWN1/pages/64815206/NWN%2BExplorer>
- Ograniczenie: opisana historyczna wersja podglądu nie obsługuje poprawnie
  współczesnego zakresu modeli i standardowych DDS NWN:EE.
- Wniosek: „otwiera się w NWN Explorer” nie może być bramką zgodności. Może
  służyć do inspekcji części zasobów, ale wynik negatywny nie dowodzi błędu
  artefaktu NWN:EE.

#### PLT i historyczna przezroczystość placeable — wyniki pomocnicze

- PLT: <https://nwn.wiki/spaces/NWN1/pages/14618045/PLT>
- Placeable/TXI transparency:
  <https://nwn.wiki/spaces/NWN1/pages/38176400/Placeable%2BModel%2BTXI%2BTransparency>
- PLT jest istotne dla tintowanych części stworzeń, skrzydeł, ogonów i
  wybranych ikon, ale nie jest domyślnym formatem sprite'ów VFX. Trafia do
  późniejszego profilu mesh/chunk particles, nie P0 flipbooków.
- Strona o przezroczystości placeable sama zaznacza, że opisywany workaround
  dotyczy 1.69, nie aktualnego EE. To dobry przykład, dlaczego każdy przepis
  musi mieć `runtimeProfile`; nie należy przenosić go do współczesnego writera.

### 21.2 Authoring shaderów: użyteczne wzorce, ale nie nowy format docelowy

#### ShaderFrog GLSL parser i Core

- Parser: <https://github.com/ShaderFrog/glsl-parser>
- Graph Core: <https://github.com/ShaderFrog/core>
- Przydatność: parser GLSL ES 1.00/3.00 buduje AST, zachowuje whitespace i
  komentarze, obsługuje preprocessing oraz visitor API. Core pokazuje
  kompilację graphu do pośredniego wyniku i osobne adaptery silników.
- Krytyczne ograniczenia: parser deklaruje ograniczoną analizę semantyczną,
  więc wygenerowany shader nadal musi trafić do prawdziwego kompilatora.
  Graph Core jest oznaczony jako eksperymentalny i zmienny.
- Stan licencyjny zweryfikowany 2026-09-02: widoki root obu repozytoriów nie
  zawierają pliku `LICENSE`, a GitHub nie pokazuje rozpoznanej licencji.
  Określenie „open source” w README nie jest wystarczającą zgodą na reuse.
- Decyzja: `behavior/reference only`; kod zablokowany do jednoznacznego
  potwierdzenia licencji dla przypiętej rewizji.
- Wniosek architektoniczny: jeśli powstanie edytor shaderów, transformacje
  powinny działać na AST, ale ostateczne błędy muszą pochodzić z glslang i
  rzeczywistego kompilatora preview, już wskazanych w R1.

#### MaterialX

- Repozytorium: <https://github.com/AcademySoftwareFoundation/MaterialX>
- Specyfikacja:
  <https://github.com/AcademySoftwareFoundation/MaterialX/blob/main/documents/Specification/MaterialX.Specification.md>
- Licencja: Apache-2.0; zweryfikowana 2026-09-02.
- Przydatność: dojrzały wzorzec DAG, typowanych portów, NodeDefs, implementacji
  zależnych od targetu i rozdzielenia materiału od przypisania do geometrii.
- Ograniczenie: MaterialX opisuje bogate materiały/look development, a nie
  emitter MDL, TXI ani dokładny interfejs shaderów NWN:EE.
- Decyzja: `schema/UX reference`, nie format projektu VFX i nie dependency P0.
  Wewnętrzny graph powinien być mniejszy i ograniczony przez capability mapę
  Aurory.

#### Interactive Shader Format (ISF)

- Specyfikacja: <https://github.com/mrRay/ISF_Spec>
- Strona projektu i licencja: <https://isf.video/>
- Licencja: MIT dla specyfikacji/codebase/shaderów według strony projektu;
  provenance pojedynczego importowanego shaderu nadal wymaga sprawdzenia.
- Przydatność: prosty wzorzec „GLSL + JSON metadata”, automatyczne UI dla
  typowanych inputów, jawne passy, bufory trwałe i wersja formatu.
- Wniosek: podobny mechanizm może opisywać **preview-only shader preset** z
  typowanymi uniformami, ale nie wolno eksportować ISF jako shader NWN ani
  zakładać zgodności jego passów i buforów z interfejsem silnika.
- Decyzja: `metadata/UX reference`; biblioteka publicznych shaderów nie jest
  automatycznie legalnym corpusem produktu bez manifestu per asset.

#### glsl_analyzer

- URL: <https://github.com/nolanderc/glsl_analyzer>
- Licencja: GPL-3.0; zweryfikowana 2026-09-02.
- Przydatność: autocomplete, go-to-definition, hover, formatter i obsługa
  `#include` pokazują minimalny poziom ergonomii dla edycji GLSL.
- Decyzja: `UX reference only`; nie osadzać ani nie dystrybuować w produkcie
  bez osobnej decyzji licencyjnej. P0/P1 nie potrzebuje pełnego shader IDE.

### 21.3 Diagnostyka preview WebGL/WebGPU

#### SpectorJS

- Repozytorium: <https://github.com/BabylonJS/Spector.js>
- Licencja: MIT; zweryfikowana 2026-09-02.
- Przydatność: przechwycenie jednej klatki WebGL/WebGL2 wraz z listą komend,
  stanem kontekstu i zasobami. Może działać jako rozszerzenie lub narzędzie
  deweloperskie dołączone do strony.
- Wniosek: dobry oracle diagnostyczny dla regresji podglądu, szczególnie gdy
  efekt znika przez stan blend/depth/texture. Nie powinien być domyślnym
  elementem bundle produkcyjnego ani dowodem zgodności z NWN.

#### stats-gl

- Pakiet i dokumentacja: <https://www.npmjs.com/package/stats-gl>
- Status zweryfikowany 2026-09-02: wersja 4.2.3, MIT; monitoruje FPS, CPU i GPU
  dla WebGL/WebGPU oraz wspiera Three.js.
- Wniosek: sensowny kandydat do opcjonalnego panelu deweloperskiego, ale każde
  pole musi mieć capability flag. Brak timer query nie może być pokazywany jako
  `0 ms GPU`, tylko jako `unavailable`.
- Decyzja: `evaluate as dev-only dependency`; przed adopcją sprawdzić koszt
  bundle, cleanup, zachowanie w workerze i zgodność z dokładnie przypiętą
  wersją Three.js.

#### Three.js renderer.info i kompilacja shaderów

- Dokumentacja:
  <https://threejs.org/docs/pages/WebGLRenderer.html>
- `renderer.info` udostępnia liczby aktywnych geometrii/tekstur oraz na klatkę:
  draw calls, triangles, points i lines. `compileAsync` pozwala prekompilować
  materiały przy wsparciu `KHR_parallel_shader_compile`.
- Wniosek: te liczniki powinny wejść do baseline panelu bez nowego dependency.
  Pomiar należy wykonywać w stałym scenariuszu, kamerze, rozdzielczości i
  device-pixel-ratio; inaczej porównania nie są powtarzalne.
- Prekompilacja powinna oddzielić koszt pierwszej klatki od steady-state, ale
  oba wyniki należy raportować — ukrycie shader warm-up zafałszuje UX.

#### EXT_disjoint_timer_query_webgl2

- Specyfikacja Khronos:
  <https://registry.khronos.org/webgl/extensions/EXT_disjoint_timer_query_webgl2/>
- Rozszerzenie mierzy czas komend GPU bez synchronicznego blokowania, ale wynik
  jest dostępny później i jest nieważny, gdy ustawiony jest `GPU_DISJOINT_EXT`.
- Wniosek: profiler potrzebuje asynchronicznej kolejki próbek, odrzucania
  disjoint results oraz jawnego fallbacku. Pojedynczy odczyt czasu JS nie jest
  równoważny czasowi GPU.
- Raport ma używać nazw `previewCpuMs` i `previewGpuMs`; nie wolno nazywać ich
  „NWN CPU/GPU cost”.

### 21.4 Deterministyczność, property tests i golden output

#### Stały krok symulacji

- Źródło: <https://gafferongames.com/post/fix_your_timestep/>
- Przydatność: opisuje oddzielenie render framerate od stałych kroków
  symulacji oraz accumulator dla zmiennej częstotliwości wyświetlania.
- Wniosek: scrub, play, prewarm i eksport próbki muszą wywoływać ten sam
  `step(dt)` w Rust. Renderer może interpolować obraz, ale nie może dopisywać
  dodatkowej, zależnej od FPS ewolucji cząstek.
- Należy zapisać w projekcie `simulationHz`, wersję algorytmu i regułę
  zaokrąglania czasu. Samo pole `seed` nie daje deterministyczności.

#### PCG jako jawny kandydat RNG

- Referencyjna implementacja C: <https://github.com/imneme/pcg-c>
- Dokumentacja użycia/seeding:
  <https://www.pcg-random.org/using-pcg-c.html>
- Licencje: Apache-2.0 lub MIT; zweryfikowane 2026-09-02.
- Przydatność: jawny stan i strumień, reprodukowalne wyniki oraz referencyjne
  testy. Wywołanie bounded random może zużywać zmienną liczbę próbek, dlatego
  kolejność pobrań jest częścią kontraktu symulacji.
- Wniosek: przed wyborem algorytmu trzeba zamrozić dokładny wariant, inicjalny
  state/sequence, kolejność bitów, mapowanie na float i golden vectors.
- Preferowany wariant architektury: RNG istnieje tylko w Rust i jest używany
  przez native i WASM. Druga „zgodna” implementacja w TypeScript tworzyłaby
  niepotrzebną powierzchnię rozjazdu.

#### WebAssembly ma mały, ale realny zakres niedeterministyczności

- Lista przypadków:
  <https://github.com/WebAssembly/design/blob/main/Nondeterminism.md>
- Aktualna specyfikacja core: <https://www.w3.org/TR/wasm-core/>
- Fakty: specyfikacja dopuszcza między innymi różne bitowe reprezentacje NaN,
  nondeterminism dla shared-memory races i relaxed SIMD oraz różne limity
  zasobów środowiska.
- Wniosek: stan symulacji nie może dopuszczać NaN/Infinity. Reference path nie
  powinien używać relaxed SIMD ani współbieżnych zapisów do stanu. Snapshot
  bajtów float wymaga kanonikalizacji albo wcześniejszego fail-closed.
- To nie podważa wyboru WASM; precyzuje warunki, pod którymi hash stanu jest
  wiarygodnym goldenem między przeglądarką i testem natywnym.

#### wasm-bindgen jako cienka granica, nie drugi model domeny

- Repozytorium: <https://github.com/rustwasm/wasm-bindgen>
- Przydatność: generuje statycznie typowane powiązania między eksportami Rust
  i JavaScript oraz glue tylko dla faktycznie używanego interfejsu.
- Wniosek: API preview powinno być małe: załaduj wersjonowany dokument, ustaw
  seed/czas, wykonaj N kroków, odczytaj płaskie bufory pozycji/koloru/klatki i
  diagnostics. Nie serializować pełnego obiektu cząstki jako tysiące obiektów
  JS w każdej klatce.
- Three.js ma renderować bufory wyprodukowane przez ten sam core, który
  generuje testowe snapshoty; TypeScript pozostaje UI i adapterem renderera.

#### fast-check i proptest

- TypeScript: <https://github.com/dubzzz/fast-check>
- Rust: <https://github.com/proptest-rs/proptest>
- Licencje: fast-check MIT; proptest MIT lub Apache-2.0. Zweryfikowane
  2026-09-02.
- Oba narzędzia generują wiele wejść i minimalizują przypadek po wykryciu
  błędu. Proptest jest stabilny, ale repo opisuje go jako projekt głównie w
  trybie pasywnego utrzymania; nie jest to blocker dla dev dependency, lecz
  wymaga przypięcia wersji.
- Rekomendowane properties:
  - parse → serialize → parse zachowuje semantykę MDL/TXI/2DA;
  - writer → readback zachowuje wszystkie wspierane pola;
  - `aliveCount` nigdy nie przekracza pool/capability limit;
  - stan nie zawiera NaN/Infinity i pozostaje w policzalnych bounds;
  - rewind + ponowne dojście do czasu T daje ten sam hash;
  - ten sam seed/dokument/wersja core daje te same golden vectors native/WASM;
  - nieobsługiwana kombinacja nie jest cicho redukowana, tylko zwraca ten sam
    klasyfikowany błąd.
- Podział: proptest testuje core i formaty; fast-check testuje granicę
  TypeScript/WASM, migracje projektu i sekwencje akcji UI. Nie należy testować
  tej samej fizyki dwiema niezależnymi implementacjami.

#### Snapshoty Rust i TypeScript

- Rust `insta`: <https://github.com/mitsuhiko/insta>
- Vitest snapshots: <https://vitest.dev/guide/snapshot.html>
- Licencja `insta`: Apache-2.0. Vitest jest już obecny w stosie projektu.
- Przydatność: czytelne review dużych wyników strukturalnych i wykrywanie
  niezamierzonych zmian.
- Wniosek: snapshoty powinny obejmować canonical project JSON, capability
  report, diagnostykę, ASCII MDL/TXI/2DA i znormalizowany binary readback.
  Surowe zrzuty klatek nie zastępują snapshotów stanu, ponieważ sterownik,
  alpha i antyaliasing tworzą szum wizualny.
- Aktualizacja goldena musi być jawną decyzją review; automatyczne
  `accept all` w CI zniszczyłoby wartość testu.

### 21.5 Wyniki odrzucone albo zdegradowane w R3

- ShaderFrog Core/parser nie może zostać dependency bez jednoznacznej
  licencji, nawet jeśli rozwiązanie AST jest technicznie atrakcyjne.
- `glsl_analyzer` pozostaje wzorcem UX z powodu GPL-3.0 i nie jest konieczny
  do P0/P1.
- MaterialX i ISF nie stają się formatem projektu ani eksportu NWN. Są większe
  lub semantycznie inne niż potwierdzony zakres Aurory.
- SpectorJS, stats-gl i timer queries profilują preview, nie klienta NWN.
  Ich wyniki nie mogą ustalać budżetów runtime bez corpus i owner proof.
- NWN Explorer nie jest walidatorem współczesnego DDS/binary MDL.
- Historyczny przykład TXI i workaround przezroczystości 1.69 są corpusem i
  wskazówką archeologiczną, nie współczesną specyfikacją wykonawczą.
- PLT nie wchodzi do podstawowego pipeline'u sprite/flipbook tylko dlatego, że
  ma wyższy priorytet niż TGA w części ścieżek modelu.
- Nie wprowadzać drugiego symulatora i RNG w TypeScript. Łatwiejszy debug nie
  rekompensuje stałego ryzyka rozjazdu względem Rust/native/WASM.

### 21.6 Nowe wnioski architektoniczne R3

1. **Resource resolution jest częścią poprawności artefaktu.** HAK zawierający
   poprawny plik nadal może uruchomić inny zasób o tym samym resrefie. Raport
   kolizji, priorytetów i hashy musi być artefaktem builda.
2. **Application profile musi znać punkt przyłączenia i fazę.** Target object,
   model class, head/hand/ground/impact oraz conjuration/cast są danymi
   authoringu i fixture, a nie ukrytym szczegółem skryptu testowego.
3. **Persistent VFX jest osobną gałęzią kompilacji.** Nie należy udawać, że
   `visualeffects.2da + progfx.2da` i `vfx_persistent.2da` mają jeden wspólny
   zestaw możliwości.
4. **TXI wymaga lossless typed AST.** Nieznane dyrektywy, ich kolejność i
   continuation lines muszą przeżyć import/export; semantyka tekstury i
   emitera pozostaje rozdzielona.
5. **Tekstura ma wersjonowany profil kodowania.** Alpha, kompresja, mipmapy,
   gamma, rola mapy, format i runtime profile muszą być jawne i czytelne po
   readbacku.
6. **Shader graph jest dodatkiem po emitter fidelity.** AST i typowane porty są
   użyteczne, lecz authoring shaderów nie może wyprzedzić wiernego emitera,
   resource resolvera i capability report.
7. **Preview metrics mają własną przestrzeń nazw.** `previewCpuMs`,
   `previewGpuMs`, draw calls, live particles i overdraw nie są pomiarem NWN;
   raport musi utrzymywać to rozdzielenie.
8. **Jeden symulator, dwa hosty.** Native Rust i WASM wykonują ten sam core;
   TypeScript steruje czasem i renderuje płaskie bufory, ale nie implementuje
   ponownie dynamiki cząstek.
9. **Deterministyczność to pełny kontrakt.** Obejmuje RNG variant/state/stream,
   fixed timestep, kolejność próbek, floating-point policy, core version i
   canonical serialization, nie tylko seed.
10. **Property tests i snapshoty uzupełniają się.** Generatory szukają nowych
    skrajnych przypadków, a snapshoty chronią czytelne, zatwierdzone wyniki.
11. **Wersje zależności muszą być zamrożone przed implementacją VFX.** Obecny
    `apps/studio-web/package.json` używa `latest` dla podstawowego stosu.
    Reprodukowalny profiler, renderer i golden images wymagają dokładnie
    przypiętych wersji oraz aktualizacji wykonywanych osobną zmianą.

### 21.7 Kandydaci do spike'ów po aktywnych gate'ach

Lista nie zmienia aktywnej roadmapy i nie autoryzuje implementacji:

1. własny `ResourceResolutionReportV1` na corpusie MOD/HAK/override z
   kolizjami DDS/TGA/MTR/TXI i 2DA;
2. własny lossless TXI parser/writer z corpus round-trip i differential
   porównaniem do `nwnrs-types`;
3. typed attachment matrix dla creature full/parts/limited/simple,
   placeable, door, location i beam;
4. osobny mini-schema oraz fixture generator dla `persistentAreaEffect2da`;
5. panel baseline oparty najpierw na `renderer.info`, później warunkowo na
   timer query/stats-gl;
6. PCG golden vectors + fixed-step native/WASM state hashes;
7. proptest dla parserów/writerów i fast-check dla migracji oraz granicy WASM;
8. snapshoty capability report i binary readback przed golden images;
9. osobny dependency-freeze dla Three/Vite/Vitest/React przed rozpoczęciem
   właściwej gałęzi VFX.

### 21.8 Nowe pytania po R3

1. Jaki dokładny porządek HAK-ów i resource types stosuje nasz generator
   modułu, gdy dwa kontenery zawierają ten sam resref?
2. Które specjalne węzły istnieją w exact retail corpus dla każdego model
   class i jakie są rzeczywiste fallbacki silnika?
3. Jakie kolumny oraz modele `vfx_persistent.2da` są rzeczywiście authorable
   bez hardcoded wyjątku i jak różni się static od mobile AOE?
4. Które dyrektywy TXI są wymagane dla cycle/alpha/mipmap w dokładnym profilu
   NWN:EE, a które są historyczne lub ignorowane?
5. Czy ten sam texture resref z MTR, DDS, PLT i TGA rozwiązuje się identycznie
   w Toolsecie i runtime dla wybranych klas VFX?
6. Czy snapshot stanu powinien używać float32 dokładnie jak bufory preview, czy
   core oblicza w float64 i kwantyzuje dopiero na granicy renderera?
7. Jakie golden vectors RNG i jaka polityka NaN/Infinity będą częścią
   `VfxSimulationProfileV1`?
8. Które metryki preview są stabilne na CI bez gwarancji sprzętowego GPU, a
   które wymagają jawnie oznaczonego hardware benchmark lane?

## 22. Runda internetowa R4 — 2026-09-02

Zakres rundy: kontrolery i animacje MDL, niezależne źródła struktury binary
MDL, przeniesienie preview poza główny wątek, transfer dużych buforów,
odtwarzanie po utracie kontekstu WebGL, fuzzing parserów i writerów oraz
stabilizacja testów wizualnych. Źródła z rund R1–R3 wykorzystano jako kontekst
bez ponownego wpisywania ich adresów.

### 22.1 Animacja Aurory i niezależne źródła struktury MDL

#### xoreos-docs: template binary oraz archiwum specyfikacji

- Repozytorium: <https://github.com/xoreos/xoreos-docs>
- Template 010 Editor:
  <https://github.com/xoreos/xoreos-docs/blob/master/templates/NWN1MDL.bt>
- Archiwum notatek Torlacka:
  <https://github.com/xoreos/xoreos-docs/tree/master/specs/torlack>
- Status zweryfikowany 2026-09-02: repozytorium dokumentacyjne reverse
  engineeringu. README deklaruje CC0-1.0 dla plików z katalogu `templates`;
  pozostałe materiały mają różne pochodzenie i pozostają wyłącznie źródłem
  historycznym do niezależnej implementacji.
- Template pokazuje osobne tablice kluczy i danych kontrolerów. Rekord
  kontrolera zawiera typ zależny od rodzaju node'a, liczbę wartości, offset
  kluczy czasu, offset danych i liczbę kolumn. Nagłówek animacji ma osobne
  `length`, transition, nazwę root node'a i tablicę eventów.
- Wniosek: binarny kontroler nie jest po prostu listą `(time, value)`. Własny
  IR musi zachować typ kontrolera, arity/column count, czas, payload, docelowy
  node i pochodzenie. Template jest dodatkowym oracle strukturalnym, nie
  specyfikacją wykonawczą ani kodem do skopiowania.

#### nwnrs-types: warstwy authored, binary, semantic i scene

- Dokumentacja modułu:
  <https://docs.rs/nwnrs-types/latest/nwnrs_types/mdl/index.html>
- Wartość: biblioteka rozdziela syntax-faithful ASCII, compiled binary,
  semantic model i scenę oraz jawnie wystawia `AnimationEvent`,
  `BinaryController`, transform/material/animmesh tracks i diagnostykę.
- Przydatność: dobry model porównawczy dla naszego własnego podziału
  `source AST → typed semantic IR → render scene → writer/readback`.
- Ograniczenie: to zewnętrzna implementacja i nie zastępuje Aurora First,
  retail corpus ani własnych readerów/writerów wymaganych w projekcie.

#### NWN Wiki: compiled MDL i aktualność narzędzi

- Compiled vs ASCII MDL:
  <https://nwn.wiki/spaces/NWN1/pages/38175669/MDL>
- Przegląd narzędzi modelowych:
  <https://nwn.wiki/pages/viewpage.action?pageId=118390871>
- Status: dokumentacja społecznościowa NWN:EE, zweryfikowana 2026-09-02.
- Wiki wskazuje, że ASCII jest interpretowany/kompilowany przy ładowaniu, co
  może być kosztowne dla dużych lub często ładowanych modeli. Compiled MDL
  koduje także wartości domyślne, lecz starsze dekompilatory nie rozumieją
  wszystkich zmian NWN:EE i mogą się wywracać lub gubić dane.
- Wniosek: czytelny ASCII jest formatem inspekcji i diagnostyki, ale
  deliverable powinien przechodzić przez własny binary writer i własny
  semantic readback. Stary `nwnmdlcomp`/NWN Explorer nie może być złotym
  oracle współczesnego MDL.

#### Kontrakt timeline wynikający z połączenia źródeł

W połączeniu z opisem ASCII z sekcji 4.1 źródła prowadzą do następującego
minimum `ControllerTrackV1`:

```text
targetNodeId
controllerKind
valueArity
keyEncoding        # single, counted list, key/endlist
interpolationKind  # jawne, nie domyślne z UI
timeDomain         # clip time, transition albo event time
keys[]             # stable order + source spans
diagnostics[]
```

Reguły authoringu i walidacji:

- `length` klipu i `transtime` to dwa różne pojęcia; UI nie może pokazywać
  transition jako końcówki tej samej osi danych;
- eventy są pierwszoklasowymi markerami. Dla eventów o tym samym czasie musi
  istnieć deterministyczny tie-breaker zachowujący source order;
- czas klucza nie może wypadać poza długość klipu;
- animation node musi wiązać się ze znanym geometry node'em, a zmiana nazwy
  node'a musi bezpiecznie przepisać referencje albo zakończyć się blokadą;
- każdy rodzaj node'a ma własny dozwolony zbiór kontrolerów i arity. Jedna
  uniwersalna tabela `property: number[]` ukryłaby błędy aż do writer/readback;
- unknown controller lub encoding należy zachować jako lossless source
  element albo jawnie zablokować. Ciche pominięcie jest niedopuszczalne.

### 22.2 Preview poza głównym wątkiem

#### Standardy i praktyczne wejścia

- OffscreenCanvas:
  <https://developer.mozilla.org/en-US/docs/Web/API/OffscreenCanvas>
- `requestAnimationFrame` w dedicated workerze:
  <https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/requestAnimationFrame>
- Transferable objects:
  <https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects>
- Worker lifecycle, errors i `terminate()`:
  <https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers>
- Praktyczny przykład Three.js:
  <https://threejs.org/manual/en/offscreencanvas.html>
- Rekomendowana składnia bundlowania workera przez Vite:
  <https://vite.dev/guide/features.html#web-workers>
- Status: aktualne dokumentacje web platform, Three.js i Vite; sprawdzone
  2026-09-02.

`OffscreenCanvas` jest transferowalny i może tworzyć kontekst WebGL w
workerze. Worker nie ma DOM, więc rozmiar, DPR, pointer/keyboard input oraz
widoczność dokumentu muszą przychodzić przez jawny protokół. Manual Three.js
pokazuje potrzebę proxy tych danych i fallbacku na main thread. Jego stare
uwagi o dostępności przeglądarek nie są źródłem aktualnej macierzy wsparcia;
do niej używamy aktualnego MDN i testu feature detection.

`ArrayBuffer` można przenieść bez kopiowania, ale po transferze bufor nadawcy
jest detached. Pula buforów musi zatem mieć jawnego właściciela i stan:
`available → filling → transferred → returned`. Wysyłanie tysięcy obiektów JS
albo przypadkowe użycie odłączonego bufora byłoby źródłem kosztów i błędów.

#### Proponowany protokół `PreviewWorkerProtocolV1`

```text
control: init, loadProject, resize, setInput, seek, step, play, pause, dispose
bulk:    transfer particle/state ArrayBuffer, return buffer ownership
events:  ready, frame, diagnostics, contextLost, contextRestored, fatal
stamp:   protocolVersion, coreVersion, projectHash, seed, tick, requestId
```

Najważniejsze zasady:

- canonical simulation pozostaje w tym samym Rust core dla native i WASM;
  worker jest hostem i schedulerem, nie drugą implementacją fizyki;
- `requestAnimationFrame` planuje prezentację, lecz nie jest zegarem prawdy.
  Callback może zostać wstrzymany w tle, dlatego stan wynika z jawnego ticka
  fixed-step, a nie z liczby callbacków;
- każdy request ma `requestId`, a odpowiedź zawiera tick i hash projektu.
  Spóźniona klatka starego projektu nie może nadpisać nowego preview;
- main-thread fallback wykonuje ten sam protokół przez lokalny adapter. Nie
  utrzymujemy osobnego kodu domenowego dla dwóch ścieżek;
- po przeniesieniu kontroli nad canvasem awaria workera wymaga jawnej strategii
  odtworzenia hosta/canvasu, nie tylko ponownego `new Worker()`.

#### Comlink: tylko control plane

- Repozytorium: <https://github.com/GoogleChromeLabs/comlink>
- Licencja: Apache-2.0; zweryfikowana 2026-09-02.
- Wartość: mała warstwa RPC nad `postMessage` z proxy i wsparciem TypeScript.
- Decyzja: dobry kandydat do prototypu metod kontrolnych, ale nie powinien
  ukrywać lifecycle, wersji protokołu ani własności dużych buforów. Dla
  particle/state buffers używamy jawnych transferables. Dependency dopiero po
  benchmarku względem prostego typed `postMessage`.

#### SharedArrayBuffer i wielowątkowy WASM: nie do P0

- Wymagania `crossOriginIsolated`:
  <https://developer.mozilla.org/en-US/docs/Web/API/Window/crossOriginIsolated>
- WebAssembly threads:
  <https://web.dev/articles/webassembly-threads>
- Historyczne, zarchiwizowane repo:
  <https://github.com/GoogleChromeLabs/wasm-bindgen-rayon>
- Aktualna dokumentacja kontynuowanego crate'a:
  <https://docs.rs/wasm-bindgen-rayon/latest/wasm_bindgen_rayon/>
- `SharedArrayBuffer` i wspólna `WebAssembly.Memory` wymagają cross-origin
  isolation przez COOP/COEP. Wpływa to na ładowanie cross-origin resources,
  okna/popupy i hosting, a Rust/WASM threads nadal wymagają bardziej złożonego
  toolchainu i feature detection.
- Historyczne repo Google zostało zarchiwizowane w 2024; aktualne prace są w
  innym repo. To wyraźny sygnał, by nie przypinać architektury produktu do
  starego adresu.
- Decyzja: P0/P1 używa jednego workera i transferowanych `ArrayBuffer`.
  Shared memory/Rayon jest P2 tylko wtedy, gdy profil pokaże, że copy/transfer
  lub pojedynczy core jest rzeczywistym bottleneckiem i mamy testowany fallback
  bez izolacji origin.

#### Odporność na utratę kontekstu GPU

- Odtworzenie kontekstu:
  <https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/webglcontextrestored_event>
- Testowa utrata/odtworzenie:
  <https://registry.khronos.org/webgl/extensions/WEBGL_lose_context/>
- Po odtworzeniu wcześniejsze texture, buffer, framebuffer, shader i program
  handles są nieważne. Renderer musi odbudować GPU cache ze stanu CPU/WASM,
  a nie traktować `contextrestored` jak wznowienia istniejących zasobów.
- `WEBGL_lose_context` pozwala wymusić tę ścieżkę. Test powinien wywołać
  utratę w kilku fazach: przed load, w trakcie uploadu i podczas odtwarzania.
- Wniosek: canonical project i simulation snapshot nie mogą żyć wyłącznie w
  obiektach WebGL. Po restarcie powinna powstać ta sama klatka dla tego samego
  `projectHash + seed + tick`, a asynchroniczne wyniki sprzed utraty muszą być
  odrzucone po generation id.

### 22.3 Fuzzing parserów, writerów i lowering

#### Narzędzia

- `cargo-fuzz`: <https://github.com/rust-fuzz/cargo-fuzz>
- `arbitrary`: <https://github.com/rust-fuzz/arbitrary>
- Structure-aware fuzzing:
  <https://rust-fuzz.github.io/book/cargo-fuzz/structure-aware-fuzzing.html>
- libFuzzer: <https://llvm.org/docs/LibFuzzer.html>
- Licencje: `cargo-fuzz` i `arbitrary` są MIT lub Apache-2.0;
  zweryfikowane 2026-09-02.
- libFuzzer jest coverage-guided i rozwija corpus tak, by zwiększać pokrycie.
  Wspiera dictionaries, minimalizację corpus oraz zachowanie crash inputs.
  Autorzy LLVM zaznaczają, że jest utrzymywany, ale główny rozwój nowych
  funkcji przeniósł się gdzie indziej.
- `Arbitrary` pozwala generować poprawne, typowane wartości z surowych bajtów,
  dzięki czemu test dochodzi głębiej niż parser odrzucający prawie każde
  losowe wejście na pierwszym tokenie.

Rekomendowane są dwie niezależne warstwy:

1. **Byte fuzzing** dla ASCII/binary MDL, TXI, 2DA, DDS/TGA i kontenerów.
   Properties: brak panic/OOM, limit alokacji i rekurencji, klasyfikowany błąd,
   brak czytania poza zadeklarowany payload.
2. **Structure-aware fuzzing** dla typed IR i sekwencji transformacji.
   Properties: validate → write → readback, canonicalization idempotence,
   source-order/event-order stability, brak NaN/Infinity, zachowanie unknowns,
   deterministyczny error kind i brak cichego obniżenia capabilities.

Corpus startowy powinien mieć małe poprawne i niepoprawne próbki: minimalne
node'y, każdy wspierany controller/emitter mode, granice liczników i offsetów,
ucięte sekcje, nakładające się tablice, duplikaty node'ów, zero-length clip,
klucze na/poza granicą oraz unknown controller ids. Słownik powinien zawierać
tokeny MDL/TXI/2DA i znane magic bytes. Każdy znaleziony crash po minimalizacji
staje się trwałym regression fixture z provenance i oczekiwanym error kind.

Ważne ograniczenie hosta: `cargo-fuzz` wymaga nightly, LLVM sanitizers i
systemu Unix-like; repo wprost nie wspiera Windows. Dla tego projektu:

- krótki corpus regression i property tests działają lokalnie na Windows;
- fuzz smoke oraz dłuższe kampanie działają w przypiętym Linux CI/container;
- CI archiwizuje zminimalizowany input, target, seed/options, toolchain i hash
  commitów; brak artefaktu oznacza niepowtarzalny wynik;
- fuzzing nie zastępuje retail corpus ani differential readbacku. Ma szukać
  błędów bezpieczeństwa i invariantów, a nie definiować semantykę Aurory.

### 22.4 Kolor i testy wizualne bez fałszywej stabilności

#### Jawny pipeline przestrzeni barw

- Three.js Color Management:
  <https://threejs.org/manual/en/color-management.html>
- Three.js pracuje w Linear-sRGB, podczas gdy color textures i wyświetlany
  wynik typowo używają sRGB; non-color textures nie powinny dostać oznaczenia
  przestrzeni barw. Custom shaders muszą wykonać właściwą konwersję output.
- Wniosek: fixture i golden zapisują co najmniej `workingColorSpace`,
  `outputColorSpace`, tone-mapping profile, exposure oraz semantyczną rolę
  każdej tekstury. Dwa obrazy w innych przestrzeniach barw nie mogą być
  porównywane jednym progiem pixel diff.
- Preview fidelity do NWN nie wynika automatycznie z poprawnego workflow
  Three.js. Profil Three jest bazą powtarzalnego preview; różnica do retail
  renderer nadal jest jawnie raportowaną aproksymacją i owner proofem.

#### Powtarzalne screenshoty

- Playwright visual comparisons:
  <https://playwright.dev/docs/test-snapshots>
- Emulacja viewport/DPR/media:
  <https://playwright.dev/docs/emulation>
- Playwright ostrzega, że wynik zależy od OS, wersji przeglądarki, ustawień,
  sprzętu, zasilania i trybu headless. Baseline i porównanie muszą zatem
  działać w tym samym przypiętym środowisku.
- Fixture zamraża browser build, OS/container image, viewport, DPR,
  `colorScheme`, `reducedMotion`, output color profile, seed, tick, kamerę,
  rozmiar canvasu i wersję GPU/backendu. Aktualizacja baseline jest osobnym,
  ręcznie recenzowanym działaniem.
- Najpierw sprawdzamy hash stanu, counts, bounds i diagnostics; dopiero potem
  obraz. Stabilizowanie przypadkowo niedeterministycznej klatki przez szeroki
  próg pixel diff ukryłoby błąd symulacji.

#### SSIMULACRA 2 jako diagnostyka, nie samotny gate

- Repozytorium: <https://github.com/cloudinary/ssimulacra2>
- Licencja: BSD-3-Clause; zweryfikowana 2026-09-02.
- Metryka porównuje obraz referencyjny z badanym w percepcyjnej przestrzeni,
  uwzględnia wiele skal, blur i ringing. Była strojona głównie pod jakość i
  artefakty kompresji obrazów.
- Decyzja: może klasyfikować wielkość zmiany i wspierać triage goldenów, ale
  nie może samodzielnie zatwierdzać semantyki VFX. Mały, lecz krytyczny brak
  emitera może zginąć w dobrym wyniku całej klatki.
- Minimalny pakiet niezgodności powinien zawierać expected, actual, pixel diff,
  perceptual score, state snapshot, alpha oraz metadane środowiska. Dla
  trudnych przypadków warto dodać osobne debug render targets: particle id,
  depth i overdraw; nie są one obrazem użytkowym ani dowodem NWN.

### 22.5 Wyniki odrzucone albo zdegradowane w R4

- Archiwalne template'y i notatki xoreos/Torlack nie są pełną współczesną
  specyfikacją NWN:EE. Są dodatkowymi oracle i generatorami hipotez.
- Nie opierać compilera na starym `nwnmdlcomp` lub NWN Explorer; ich znane
  problemy z nowszym compiled MDL mogą dać fałszywy błąd albo stratny readback.
- Comlink nie może ukryć granicy bulk data i własności bufora. Najpierw mały,
  jawny protokół, dopiero potem wygoda RPC.
- SharedArrayBuffer/Rayon nie jest wymogiem architektury P0/P1. Koszt COOP,
  COEP, nightly/toolchainu i synchronizacji wymaga dowodu z benchmarku.
- Zarchiwizowane repo `GoogleChromeLabs/wasm-bindgen-rayon` nie może być
  aktywną zależnością; co najwyżej historyczny wskaźnik do kontynuowanego
  projektu.
- `requestAnimationFrame` nie jest zegarem symulacji. Background throttling
  nie może zmieniać wynikowego stanu VFX.
- Golden screenshot bez zamrożonego koloru, DPR, browsera, ticka i backendu
  nie jest testem regresji, tylko niekontrolowaną obserwacją.
- SSIMULACRA 2 powstała dla jakości obrazu i kompresji; nie zastępuje
  strukturalnych asercji, pixel diffu ani ludzkiego werdyktu wizualnego.

### 22.6 Nowe wnioski architektoniczne R4

1. **Timeline potrzebuje typowanego controller IR.** Controller kind, arity,
   encoding, interpolation, time domain, source order i target node muszą
   przeżyć import, edycję, binary writer i readback.
2. **Eventy animacji są danymi, nie callbackami UI.** Mają stabilny porządek,
   source span i te same wyniki dla scrub/play/export.
3. **ASCII jest widokiem audytowym, binary deliverable'em.** Oba prowadzą do
   jednego semantic IR, ale każdy ma własny parser, writer i corpus.
4. **Worker ma wersjonowany protokół.** Lifecycle, request/generation id,
   tick, project hash i własność buforów są jawne i testowalne.
5. **OffscreenCanvas jest optymalizacją z fallbackiem.** Nie może być jedyną
   ścieżką uruchomienia ani miejscem przechowywania canonical state.
6. **Transfer ownership jest częścią modelu stanu.** Pool `ArrayBuffer` musi
   wykrywać detached, double-send, late-return i użycie po dispose.
7. **rAF planuje obraz, fixed-step tworzy stan.** Pause/background/seek nie
   zmieniają wyniku dla tego samego seed + tick.
8. **Renderer musi przeżyć context loss.** GPU cache jest odbudowywalny ze
   snapshotu CPU/WASM, a stare async jobs są odcinane generation id.
9. **Fuzzing ma dwa poziomy.** Surowe bajty bronią parserów; strukturalne
   generatory penetrują lowering/writer/readback i inwarianty domenowe.
10. **Fuzzing jest osobnym Linux lane.** Windows zachowuje szybkie properties
    i regression corpus, ale nie udaje pełnej kampanii libFuzzer.
11. **Kolor jest częścią fixture contract.** Working/output space, rola
    tekstury i tone mapping są równie ważne jak viewport i seed.
12. **Wizualny gate jest warstwowy.** Najpierw state/diagnostics, potem pixel
    diff, opcjonalna metryka percepcyjna i na końcu owner proof w Aurorze/NWN.

### 22.7 Kandydaci do spike'ów po aktywnych gate'ach

Lista pozostaje badaniem i nie autoryzuje implementacji:

1. własny `ControllerTrackV1` dla transform, emitter, light i animmesh wraz z
   ASCII/binary round-trip na minimalnym corpusie;
2. differential structural probe: własny reader kontra `NWN1MDL.bt` i
   `nwnrs-types`, z retail bytes jako źródłem rozstrzygającym;
3. `PreviewWorkerProtocolV1` na jednym workerze z main-thread adapterem i
   dwoma transferowanymi buforami ping-pong;
4. test late-frame rejection po `loadProject` i po szybkim seek;
5. context-loss test przez `WEBGL_lose_context`, w tym odbudowa texture,
   particle buffer, shader i render target;
6. Linux CI fuzz targets: raw MDL/TXI/2DA, structured controller IR,
   writer/readback i project migration;
7. minimalny versioned fuzz corpus oraz słownik tokenów z raportem
   coverage/cmin i retencją crash artifacts;
8. color-contract fixture w Three.js oraz Playwright golden w jednym
   przypiętym środowisku;
9. porównanie pixelmatch i SSIMULACRA 2 wyłącznie jako narzędzie triage dla
   małych, lokalnych zmian emitera;
10. benchmark jednego workera + transferables przed jakimkolwiek spike'iem
    SharedArrayBuffer/Rayon.

### 22.8 Nowe pytania po R4

1. Które controller ids i column counts występują w exact retail corpus dla
   emitter, light, animmesh i transform nodes?
2. Czy wszystkie kontrolery wymagają tej samej interpolacji, czy część ma
   semantykę step/bezier/quaternion wynikającą z typu i wersji formatu?
3. Jak retail engine porządkuje dwa eventy o identycznym czasie i czy kolejność
   source/binary array jest obserwowalna?
4. Czy binary writer ma kanonizować różne równoważne formy ASCII, czy zachować
   ich source encoding tylko dla ponownego eksportu tekstowego?
5. Jaki maksymalny rozmiar frame/state transfer pozostaje tańszy od renderingu
   na `OffscreenCanvas` w exact wspieranych przeglądarkach i sprzęcie?
6. Czy utrata kontekstu w workerowym OffscreenCanvas jest raportowana i
   odtwarzana identycznie na wszystkich wspieranych backendach?
7. Które third-party resources hostingu blokują COEP/COOP i czy produkt w
   ogóle potrzebuje cross-origin isolation przed etapem P2?
8. Jakie limity input bytes, node count, recursion, allocation i timeout są
   właściwe dla każdego fuzz targetu?
9. Które golden tests mogą działać na software rendererze, a które wymagają
   przypiętej maszyny GPU i osobnego baseline'u?
10. Jaki obraz debug najlepiej wykrywa brak pojedynczego emitera: particle id,
    alpha, overdraw czy strukturalny wykaz draw instances?

## 23. Runda internetowa R5 — 2026-09-02

Zakres rundy: prawidłowość przezroczystości i blendowania cząstek, wyraźne
oddzielenie fidelity preview od trybów diagnostycznych, trwały autosave i
odzyskiwanie projektu w aplikacji webowej, ochrona przed jednoczesnym zapisem
z wielu kart oraz kontrolowany eksport klatek i wideo. Źródła istniejące w
rundach R1–R4 wykorzystano jako kontekst bez duplikowania adresów.

### 23.1 Przezroczystość: zgodność docelowa kontra wygodny podgląd

#### WebGL i Three.js: alpha jest częścią kontraktu renderera

- WebGL 1.0 living specification:
  <https://registry.khronos.org/webgl/specs/latest/1.0/index.html>
- Three.js Material:
  <https://threejs.org/docs/pages/Material.html>
- Status: dokumentacja normatywna WebGL i aktualna dokumentacja Three.js;
  sprawdzone 2026-09-02.
- WebGL domyślnie tworzy drawing buffer z alpha, depth, antialias i
  `premultipliedAlpha=true`. Przy premultiplied alpha składowe RGB przekazane
  compositorowi powinny być nie większe niż alpha; wyjście spoza tego zakresu
  może mieć niezdefiniowany wizualnie rezultat.
- Three.js rozdziela `transparent`, `alphaTest`, `alphaHash`,
  `alphaToCoverage`, blend factors/equations, `depthTest`, `depthWrite` i
  `premultipliedAlpha`. Te pola rozwiązują różne problemy i nie mogą zostać
  skompresowane do jednego przełącznika „przezroczysty”.
- `alphaHash` używa losowego progu i wprowadza ziarno, a alpha-to-coverage
  zależy od MSAA. Oba mogą być dobrymi trybami diagnostycznymi preview, lecz
  nie odtwarzają automatycznie renderera Aurory.

W połączeniu z istniejącą dokumentacją NWN emitterów, `renderorder`, blend
`normal | punch-through | lighten`, kolejnością node'ów i ograniczeniami TXI
potrzebny jest jawny `TransparencyProfileV1`:

```text
targetBlendMode
targetRenderOrder
targetDepthPolicy
textureAlphaMode       # straight, premultiplied, punch-through, none
previewSortPolicy      # target-like albo diagnostic-only
previewDepthWrite
previewAlphaTest
diagnosticEnhancements # OIT, soft particles, alphaHash; nigdy implicit
```

Podgląd powinien mieć co najmniej dwa nazwane tryby:

1. **Aurora fidelity** — używa wyłącznie mapowania potwierdzonych pól
   docelowych, zachowuje renderorder i pokazuje znane artefakty/ograniczenia.
2. **Diagnostic enhanced** — może włączać lepsze sortowanie, OIT, soft
   particles i debug depth, ale zawsze pokazuje badge `NOT EXPORTED / NOT NWN`.

Bez tego rozdzielenia preview może wyglądać lepiej niż efekt w NWN i stworzyć
fałszywe poczucie zgodności.

#### Weighted blended OIT

- Artykuł JCGT McGuire'a i Bavoila:
  <https://jcgt.org/published/0002/02/09/>
- Status: publikacja naukowo-techniczna z 2013 r.; źródło algorytmu i jego
  ograniczeń, nie kod produktu.
- Metoda przybliża przezroczystość bez sortowania wszystkich fragmentów,
  korzystając z bounded memory i klasycznego blendowania. Jest atrakcyjna dla
  smoke, fire, forcefields i systemów cząstek, gdzie zmiany kolejności mogą
  powodować popping.
- OIT nie jest równoważne poprawnemu back-to-front `OVER`; zmienia operator
  compositingu. Może zatem stabilizować diagnostykę gęstego efektu, ale nie
  może być goldenem zgodności z Aurorą.
- Kandydat dopiero po baseline: porównać target-like sorting, brak sortowania
  i weighted OIT na tym samym seed/tick oraz zapisać koszt MRT, draw calls i
  różnicę obrazu.

#### Soft particles i koszt overdraw

- NVIDIA GPU Gems 3, rozdział o off-screen particles:
  <https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-23-high-speed-screen-particles>
- Status: historyczne, lecz nadal użyteczne źródło techniczne; reference-only.
- Rozdział pokazuje, że efekty dymu/fog często generują duży overdraw, a
  low-resolution particle buffer może być akceptowalny dla miękkich,
  niskoczęstotliwościowych obrazów. Nie nadaje się jednak równie dobrze do
  ostrych odłamków i drobnych detali.
- Soft particle porównuje głębokość sceny z głębokością cząstki i wygasza alpha
  blisko przecięcia zamiast wykonywać twardy discard.
- Decyzja: soft particles, half/quarter-resolution rendering i bilateral
  upsample są późnymi opcjami wydajności/diagnostyki preview. Nie wchodzą do
  P0, dopóki retail corpus nie potwierdzi odpowiadającej semantyki docelowej.

### 23.2 Autosave, odzyskiwanie i zapis pliku użytkownika

#### OPFS jako recovery cache, nie jedyna kopia projektu

- Origin private file system:
  <https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system>
- StorageManager:
  <https://developer.mozilla.org/en-US/docs/Web/API/StorageManager>
- Quota i eviction:
  <https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria>
- Status: aktualna dokumentacja MDN; sprawdzona 2026-09-02.
- OPFS daje origin-private, plikopodobny magazyn oraz szybszy synchroniczny
  `FileSystemSyncAccessHandle` dostępny tylko w workerze. Nadaje się do
  snapshotów, journalu, cache dekodowanych tekstur i dużych blobów.
- OPFS jest jednak niewidoczny jako normalny plik użytkownika, podlega quota,
  jest związany z originem i może zostać usunięty wraz z danymi witryny.
  Best-effort storage może być automatycznie evictowany pod presją miejsca.
- `navigator.storage.persist()` jedynie prosi o persistent storage; wynik może
  być odmowny i użytkownik nadal może usunąć dane. UI musi pokazać rzeczywisty
  status zamiast komunikatu „projekt bezpiecznie zapisany” po samym autosave.

Rekomendowany model trzech poziomów:

```text
L1 command journal       częsty, mały, recovery po crashu
L2 canonical snapshot   rzadszy, hash + schema/core version
L3 user-visible export  jawny plik VfxProject poza origin storage
```

OPFS obsługuje L1/L2. L3 pozostaje właściwym dokumentem użytkownika i ma
osobny przycisk `Save / Export project`. Po starcie recovery porównuje hash,
revision i timestamp z ostatnio otwartym L3; nigdy nie nadpisuje pliku bez
pokazania różnicy i decyzji użytkownika.

#### IndexedDB dla metadanych i transakcyjnego wskaźnika HEAD

- Praktyczny przewodnik IndexedDB:
  <https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Using_IndexedDB>
- Durability hint:
  <https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction/durability>
- IndexedDB zapisuje zmiany transakcyjnie, lecz shutdown może przerwać aktywną
  transakcję. Nie wolno odkładać pierwszego zapisu projektu do `unload`.
- `durability: strict` prosi user agenta, by uznał commit dopiero po zapisie na
  trwałe medium, ale nadal jest hintem i nie zastępuje eksportu użytkownika.
- Decyzja: IndexedDB przechowuje małe metadane, listę recovery sessions,
  revision/HEAD oraz handle, jeśli przeglądarka na to pozwala. Duże asset blobs
  i append-only journal lepiej umieścić w OPFS.
- Snapshot najpierw zapisuje content-addressed payload, wykonuje flush/close i
  readback hash, a dopiero potem jedną transakcją przesuwa HEAD. Niepoprawny
  payload pozostaje orphanem do późniejszego GC, ale nie psuje poprzedniej
  wersji recovery.

#### User-visible save i uprawnienia

- File System Access API:
  <https://developer.chrome.com/docs/capabilities/web-apis/file-system-access>
- `createWritable()`:
  <https://developer.mozilla.org/en-US/docs/Web/API/FileSystemFileHandle/createWritable>
- API umożliwia prawdziwe otwieranie i nadpisywanie pliku po zgodzie
  użytkownika. Zmiany są finalizowane dopiero po zamknięciu writable stream.
- Wsparcie i trwałość uprawnień różnią się między przeglądarkami; wymagany jest
  fallback `Blob + download`, który tworzy nową kopię zamiast udawać overwrite.
- Udany `close()` nie kończy naszej walidacji: jeśli API pozwala, czytamy plik
  ponownie i porównujemy SHA-256 canonical payload. UI rozróżnia `saved`,
  `downloaded copy`, `autosaved locally` i `save failed`.
- Handle i zgoda użytkownika nie są przenośną częścią `VfxProject`; to stan
  sesji hosta, który może wygasnąć albo zostać odwołany.

#### Ochrona przed zapisem z wielu kart

- Web Locks API:
  <https://developer.mozilla.org/en-US/docs/Web/API/Web_Locks_API>
- Broadcast Channel API:
  <https://developer.mozilla.org/en-US/docs/Web/API/Broadcast_Channel_API>
- Web Locks pozwala kartom i workerom jednego originu uzyskać nazwany lock
  exclusive/shared. Autosave jednego `projectId` powinien mieć najwyżej
  jednego writer ownera.
- BroadcastChannel rozsyła `opened`, `dirty`, `saved`, `headChanged` i
  `closed`, ale nie definiuje protokołu ani konsensusu. Jest powiadomieniem,
  nie blokadą i nie źródłem prawdy.
- Druga karta domyślnie otwiera ten sam projekt read-only. Przejęcie edycji
  wymaga jawnej decyzji, sprawdzenia revision/HEAD i zapisania fork/recovery
  snapshotu. Opcja Web Locks `steal` nie jest bezpiecznym automatycznym
  mechanizmem przejęcia.

### 23.3 Kontrolowany eksport klatek i wideo

#### Canvas capture: szybki zapis podglądu

- `captureStream()`:
  <https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/captureStream>
- Ręczne żądanie klatki:
  <https://developer.mozilla.org/en-US/docs/Web/API/CanvasCaptureMediaStreamTrack/requestFrame>
- `captureStream(0)` wyłącza automatyczne pobieranie klatek, a `requestFrame()`
  pozwala wskazać moment capture. Jest to lepsze od przechwytywania zależnego
  od wall-clock FPS.
- Canvas musi pozostać origin-clean; tekstura cross-origin bez poprawnego CORS
  blokuje capture. Asset ingest powinien wykryć to przed długim eksportem.
- Minimalny eksport wykonuje: `seek exact tick → render → requestFrame`, z
  jawnym FPS, rozdzielczością, DPR=1 dla pliku, kolorem tła i liczbą klatek.

#### MediaRecorder: preview recording, nie reproducible render

- API: <https://developer.mozilla.org/en-US/docs/Web/API/MediaRecorder>
- MediaRecorder jest łatwą warstwą nad MediaStream, lecz faktyczny MIME/codec i
  bitrate zależą od user agenta; bitrate w użyciu może różnić się od żądanego.
- Decyzja: nadaje się do przycisku „record what I see” i szybkiego share, ale
  nie do deterministycznego render job ani goldena. Przed startem trzeba użyć
  `isTypeSupported`, zapisać wybrany MIME i jasno nazwać wynik `realtime
  capture`.

#### WebCodecs: dokładne timestampy i jawny backpressure

- Guide:
  <https://developer.mozilla.org/en-US/docs/Web/API/WebCodecs_API/Using_the_WebCodecs_API>
- Specyfikacja:
  <https://www.w3.org/TR/webcodecs/>
- Sprawdzenie konfiguracji:
  <https://developer.mozilla.org/en-US/docs/Web/API/VideoEncoder/isConfigSupported_static>
- Queue/backpressure:
  <https://developer.mozilla.org/en-US/docs/Web/API/VideoEncoder/encodeQueueSize>
- Finalizacja kolejki:
  <https://developer.mozilla.org/en-US/docs/Web/API/VideoEncoder/flush>
- Zwolnienie klatki:
  <https://developer.mozilla.org/en-US/docs/Web/API/VideoFrame/close>
- WebCodecs daje niskopoziomowy `VideoEncoder`, `VideoFrame` i timestampy w
  mikrosekundach. Nie gwarantuje konkretnego codeca w każdym user agencie i
  nie definiuje kontenera pliku.
- Eksporter musi sprawdzić config przed pracą, ograniczać kolejkę przez
  backpressure, zamykać każdą `VideoFrame`, wykonać `flush()` i dopiero potem
  finalizować muxer. Brak `close()` może wyczerpać zasoby GPU/codec system.
- Timestamp klatki powinien wynikać z indeksu i rational FPS, nie z czasu
  renderowania. Dla `frameIndex = i` zapisujemy dokładną regułę zaokrąglenia
  mikrosekund, żeby nie akumulować dryfu.
- Hardware encoders mogą wygenerować inne bajty na różnych hostach. Test
  sprawdza liczbę klatek, timestampy, duration, rozdzielczość, codec profile i
  zdekodowane klatki kontrolne; nie wymaga identycznego SHA całego MP4/WebM.

#### Mediabunny jako kandydat do muxingu

- Repozytorium: <https://github.com/Vanilagy/mediabunny>
- Licencja: MPL-2.0; zweryfikowana 2026-09-02.
- Projekt oferuje browserowy, tree-shakeable mux/demux dla MP4/WebM i innych
  formatów oraz integrację z WebCodecs, canvasem i streaming I/O.
- Wartość: eliminuje potrzebę pisania kontenera MP4/WebM od zera dla funkcji
  pobocznej względem głównego eksportu Aurora.
- Decyzja: mocny kandydat do spike'a, ale MPL-2.0 wymaga osobnego zapisu
  zależności i obowiązków przy modyfikacji plików biblioteki. Dependency dopiero
  po sprawdzeniu bundle size, browser matrix, codec matrix, alpha i streaming
  output do File System Access/OPFS.

### 23.4 Wyniki odrzucone albo zdegradowane w R5

- Weighted OIT, alphaHash i soft particles nie są trybem Aurora fidelity.
  Ulepszają czytelność preview, ale mogą ukryć dokładnie te błędy sortowania i
  przecięć, które wystąpią w NWN.
- Half-resolution particles nie są uniwersalną optymalizacją: ostre sparks,
  odłamki, lightning i mały teksturowy detal mogą widocznie stracić jakość.
- OPFS nie jest formatem dokumentu ani backupem poza przeglądarką. Użytkownik
  może wyczyścić dane originu, zmienić origin albo stracić best-effort storage.
- `navigator.storage.persist()` nie jest gwarantowanym sukcesem i nie zwalnia
  z jawnego eksportu pliku projektu.
- Nie zapisywać po raz pierwszy w `unload`; browser może przerwać transakcję.
  Autosave działa stale podczas edycji, a unload co najwyżej ostrzega o
  nierozliczonych operacjach.
- BroadcastChannel nie zapobiega lost update. Do pojedynczego writera służy
  lock + revision check.
- `MediaRecorder` nie jest deterministycznym offline rendererem.
- WebCodecs nie zawiera muxera i nie gwarantuje jednego zestawu codeców ani
  bit-identycznego outputu na wszystkich hostach.
- Mediabunny nie staje się automatycznie zależnością przez szeroki zakres
  funkcji; potrzebny jest mały spike i osobna decyzja licencyjna.

### 23.5 Nowe wnioski architektoniczne R5

1. **Transparency ma dwa jawne profile.** `aurora-fidelity` pokazuje target,
   `diagnostic-enhanced` pomaga autorowi, ale nie może być pomylony z eksportem.
2. **Blend, alpha encoding, depth i sort są oddzielne.** Jedna flaga
   `transparent` nie opisuje ani Three.js, ani Aurory.
3. **Preview ulepszenia są non-exportable capabilities.** Capability report
   wymienia OIT, soft particles, alphaHash i low-res pass jako `preview-only`.
4. **Canonical state nie mieszka w GPU.** Ten sam snapshot odbudowuje scenę po
   context loss i służy do offline capture.
5. **Autosave ma journal, snapshot i HEAD.** HEAD przesuwa się dopiero po
   zweryfikowanym zapisie payloadu.
6. **OPFS jest cache'em odzyskiwania.** Jedyną przenośną kopią pozostaje jawny
   plik projektu lub przyszły, osobno zaprojektowany backup.
7. **Status zapisu jest precyzyjny.** UI odróżnia autosave, persistent storage,
   zapis do istniejącego pliku, pobraną kopię i błąd uprawnień/readbacku.
8. **Jedna karta jest writerem.** Web Lock chroni zapis; BroadcastChannel
   informuje pozostałe karty i nie udaje mechanizmu spójności.
9. **Eksport offline jest tick-driven.** Każda klatka wynika z exact seed,
   tick, kamery i profilu kolorów, niezależnie od szybkości encodera.
10. **Są dwa rodzaje wideo.** MediaRecorder daje szybki realtime capture;
    WebCodecs + muxer daje kontrolowany render job.
11. **Backpressure i resource lifetime są częścią export pipeline.** Queue,
    flush, close, anulowanie i częściowy output wymagają jawnego state machine.
12. **Deterministyczny film nie oznacza identycznych bajtów codeca.** Golden
    obejmuje input frames i semantykę strumienia; compressed bytes są
    host/profile-dependent, chyba że wybierzemy własny software encoder.

### 23.6 Kandydaci do spike'ów po aktywnych gate'ach

Lista nie zmienia roadmapy ani nie autoryzuje implementacji:

1. `TransparencyProfileV1` z fixture normal, punch-through, lighten,
   renderorder collision i straight/premultiplied alpha;
2. potrójne porównanie target-like sort, unsorted i weighted OIT dla gęstego
   dymu oraz krzyżujących się billboardów;
3. context-clean/tainted canvas test dla lokalnej i cross-origin tekstury;
4. `RecoveryStoreV1`: command journal + content-addressed snapshots +
   transactional HEAD w OPFS/IndexedDB;
5. crash injection po payload write, przed HEAD commit i podczas compaction;
6. multi-tab writer election przez Web Locks oraz read-only notification przez
   BroadcastChannel;
7. Save/Save As z File System Access, download fallback i SHA-256 readback;
8. `captureStream(0) + requestFrame()` dla krótkiego tick-driven WebM;
9. WebCodecs exporter z rational timestamps, bounded queue, cancel i flush;
10. Mediabunny spike dla MP4/WebM streaming output bez trzymania całego filmu
    w RAM;
11. test semantyczny wideo: frame count, timestamps, duration, color metadata
    i wybrane decoded-frame goldens;
12. UI, które jednocześnie pokazuje status fidelity mode, autosave/recovery i
    rodzaj eksportu `realtime` albo `offline deterministic input`.

### 23.7 Nowe pytania po R5

1. Jaki dokładny blend equation odpowiada każdemu `normal`, `punch-through` i
   `lighten` w retail rendererze, włączając alpha channel?
2. Czy `renderorder` porządkuje emitter instances, particles czy draw batches i
   jaki jest tie-breaker dla tej samej wartości?
3. Które klasy efektów NWN używają straight, a które efektywnie
   premultiplied alpha po dekodowaniu TGA/DDS/TXI?
4. Czy diagnostic OIT warto implementować przed exact target-like sorting, czy
   dopiero jako narzędzie porównawcze po uzyskaniu baseline'u?
5. Jaki limit OPFS i polityka retencji chronią użytkownika przed zapełnieniem
   quota przez asset blobs, snapshots i eksporty?
6. Jak często journal jest kompaktowany oraz ile poprzednich snapshotów
   recovery zachowujemy?
7. Co ma się stać, gdy dwie karty otworzą ten sam project file handle, ale
   należą do różnych originów/wersji wdrożenia i nie współdzielą Web Lock?
8. Które przeglądarki są wspierane dla true Save/overwrite, a które wyłącznie
   przez download fallback?
9. Jaki bazowy format udostępniania wybieramy: WebM/VP9, MP4/H.264 czy oba po
   runtime capability check?
10. Czy wymagamy alpha w eksporcie wideo, czy przezroczyste rezultaty eksportują
    PNG sequence/WebM tylko w profilach, które potwierdzą alpha support?
11. Czy dźwięk VFX ma być włączany do filmu, a jeśli tak, jak mapujemy event
    timeline na sample-accurate audio timestamps?
12. Które metadane codec/color muszą trafić do sidecar manifestu, żeby wynik
    eksportu dało się powtórzyć i zdiagnozować?

## 24. Runda internetowa R6 — 2026-09-02

R6 nie powtarza katalogu silników z poprzednich rund. Domyka pięć luk, które
pozostały po R5: nowe, bardzo świeże narzędzia VFX; odbudowę podglądu po utracie
urządzenia GPU; stabilną tożsamość i migrację dokumentu; bezpieczeństwo
ruchomego podglądu; oraz sample-accurate audio w preview i eksporcie.

### 24.1 Nowe repozytoria warte obserwowania

#### Pyrotechnique

- Repozytorium: <https://github.com/elodin-sys/pyrotechnique>
- Status podczas R6: bardzo młody projekt, 19 commitów i 1 gwiazdka; licencja
  Apache-2.0. To ważna referencja eksperymentalna, nie kandydat na zależność.
- Projekt rozdziela edytowalne pliki efektów, deklarację sceny, obrazy
  referencyjne i wygenerowane capture. CLI oraz UI uruchamiają ten sam skład
  sceny.
- Autor jawnie rozdziela deterministyczność symulacji od deterministyczności
  pikseli: stały timestep, seeded CPU/GPU RNG i start zegara dopiero po
  załadowaniu assetów dają powtarzalną symulację, lecz kolejność blendowania GPU
  nadal może powodować małe różnice obrazu.
- Przydatne wzorce: named scenarios, camera presets, side-by-side capture,
  debounce autosave z flush przy zmianie projektu oraz pliki efektów możliwe do
  edycji poza GUI.
- Decyzja: `capture/scenario UX reference`. Nie kopiować RON ani runtime Bevy;
  przejąć rozdział `effect + scene + scenario + target + shot` i uczciwy opis
  granicy goldena pikselowego.

#### VFX Mesh Lab

- Repozytorium: <https://github.com/PudinKiller/VFXMeshLab>
- Status podczas R6: MIT, 99 gwiazdek, narzędzie Unity 6/URP.
- Najcenniejszy nie jest backend Unity, lecz wąski workflow dla geometrii VFX:
  proceduralne pierścienie, łuki, wstęgi i crossed cards; uporządkowany stos
  modyfikatorów; generowanie UV i vertex data; widoki normals, UV, wireframe,
  topology i czerwonych backfaces.
- Regeneracja może zachować istniejącą referencję zasobu, a diagnostyczny tryb
  podglądu nie zmienia zapisywanego mesha. To dobry wzorzec dla zasady
  `diagnostic view != artifact mutation`.
- Decyzja: `P2 effect-mesh authoring reference`. Ewentualny generator Aurory
  musi dodatkowo egzekwować wspólny budżet 300 000 trójkątów i partycjonowanie
  per-stream; standardowy Unity Mesh nie jest artefaktem docelowym.

#### VFX Texture Lab

- Repozytorium: <https://github.com/PudinKiller/VFXTextureLab>
- Status podczas R6: MIT, 87 gwiazdek, narzędzie Unity 6.
- Wzorce: batch operation stack, oryginał/wynik obok siebie, gradient mapping,
  levels, threshold, posterize, blur, dilate/erode, channel packing oraz jawny
  wybór `Data Linear` kontra `Color sRGB`.
- Szczególnie przydatne jest traktowanie masek, noise, flow i packed channels
  jako danych liniowych oraz pozostawienie kolorowych tekstur w profilu sRGB.
  Narzędzie pokazuje także, że nieliniowe operacje na prawie-szarych kanałach
  mogą ujawnić kolorowe artefakty.
- Decyzja: `texture-operations UX reference`. Meshy2Aurora nadal potrzebuje
  własnego, wersjonowanego TGA/DDS writer/readbacku; PNG/EXR i import settings
  Unity nie określają zgodności z NWN.

### 24.2 WebGPU: utrata urządzenia, błędy i cykl życia zasobów

#### Utrata urządzenia nie jest wyjątkiem „na koniec świata”

- Dokumentacja `GPUDevice.lost`:
  <https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/lost>
- `lost` jest Promise rozwiązywanym po utracie urządzenia. Urządzenie może
  zostać utracone w dowolnym momencie, a po uzyskaniu nowego trzeba utworzyć od
  nowa wszystkie bufory, tekstury, pipeline'y i pozostałe zasoby starego
  urządzenia.
- Po recovery adapter może udostępnić inny zestaw features/limits. Nie wolno
  przywracać starego `CapabilityReportV1`; trzeba policzyć go ponownie i
  oznaczyć preview jako `recovered`, `degraded` albo `unavailable`.
- Stan dokumentu, seed, tick i manifest assetów pozostają po stronie
  CPU/storage. GPU cache jest jednorazową projekcją tego stanu, oznaczoną
  `rendererGeneration`.
- Każdy wynik Promise ze starej generacji musi być odrzucony. Inaczej późny
  upload lub pipeline może zatruć świeżo odbudowany renderer.

#### Error scopes zamiast przypadkowych błędów w konsoli

- Początek zakresu:
  <https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/pushErrorScope>
- Koniec zakresu:
  <https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/popErrorScope>
- Zakresy przechwytują kategorie `validation`, `out-of-memory` i `internal`, a
  `popErrorScope()` zwraca wynik asynchronicznie. Operacje importu tekstury,
  utworzenia pipeline'u i odbudowy po device loss powinny mieć własne,
  zagnieżdżone zakresy oraz stabilny identyfikator etapu.
- Globalny fallback:
  <https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/uncapturederror_event>
- `uncapturederror` nadaje się do telemetryki i jawnego bannera awarii. Nie
  powinien być zwykłym control flow, ponieważ nie wiąże błędu z konkretną
  komendą użytkownika ani węzłem grafu.

#### Kompilacja shaderów ma osobny stan gotowości

- Diagnostyka modułu WGSL:
  <https://developer.mozilla.org/en-US/docs/Web/API/GPUShaderModule/getCompilationInfo>
- Asynchroniczny pipeline:
  <https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/createRenderPipelineAsync>
- `getCompilationInfo()` zwraca komunikaty kompilatora z typem i lokalizacją,
  które można mapować na source span wygenerowany przez compiler grafu.
  `createRenderPipelineAsync()` rozdziela stan `compiling` od `ready` i
  odrzuca Promise przy błędzie pipeline'u.
- Dla WebGL analogiczny mechanizm oferuje rozszerzenie:
  <https://registry.khronos.org/webgl/extensions/KHR_parallel_shader_compile/>
- Wniosek: edycja grafu tworzy nową generację kompilacji. UI zachowuje ostatni
  poprawny preview z etykietą `stale`, aż nowy pipeline będzie gotowy; błąd nie
  może zostawić czarnego canvasu ani podmienić poprawnego artefaktu.

#### Jawne zwalnianie zasobów

- Three.js disposal guide:
  <https://threejs.org/manual/en/how-to-dispose-of-objects.html>
- Texture, geometry, material, render target, skeleton i `ImageBitmap` mają
  różne zasady własności. Usunięcie obiektu JavaScript nie jest równoznaczne ze
  zwolnieniem zasobu GPU, a `ImageBitmap.close()` wymaga decyzji właściciela.
- Wniosek: `PreviewResourceRegistryV1` powinien liczyć właścicieli, pokazywać
  typ i rozmiar logiczny zasobu oraz pozwalać testowi wykryć wzrost po 100
  cyklach hot reload. Sam garbage collector nie jest polityką pamięci.

### 24.3 Stabilny hash dokumentu i bezpieczne migracje

#### Dwa hashe zamiast jednego niejasnego „project hash”

- JSON Canonicalization Scheme:
  <https://www.rfc-editor.org/rfc/rfc8785.html>
- JCS definiuje powtarzalne kodowanie JSON przez ustaloną serializację liczb,
  rekursywne sortowanie nazw pól i wynik UTF-8. Zabrania `NaN` i Infinity,
  wymaga liczb możliwych do reprezentacji jako IEEE 754 double i nie wykonuje
  automatycznej normalizacji Unicode.
- Dokument powinien przechowywać dwie niezależne tożsamości:

  - `rawFileSha256`: hash dokładnych bajtów zaimportowanego YAML/JSON;
  - `projectSemanticSha256`: hash kanonicznego, przetworzonego modelu danych bez
    stanu UI, cache, czasu zapisu i innych pól niedeterministycznych.

- `projectSemanticSha256` nie zastępuje hashy assetów ani wynikowych HAK/MOD.
  Proof lineage nadal potrzebuje całej krotki: semantic project hash, asset
  manifest hash, compiler revision, target profile i hashe artefaktów.
- Spike musi porównać implementację TypeScript i Rust na testach JCS, w tym
  `-0`, wykładniki, astral Unicode, control characters, duże liczby i odmowę
  `NaN`/Infinity. Zwykłe `JSON.stringify` z ręcznym sortowaniem „na oko” nie
  wystarcza jako kontrakt między językami.

#### JSON Pointer/Patch jako interchange, nie wewnętrzny model komend

- JSON Pointer:
  <https://www.rfc-editor.org/rfc/rfc6901.html>
- JSON Patch:
  <https://www.rfc-editor.org/rfc/rfc6902.html>
- Pointer daje standardowe adresowanie pól i elementów tablic, a Patch dodaje
  `add`, `remove`, `replace`, `move`, `copy` i `test`. Operacja `test` może
  chronić zapis przed zastosowaniem zmiany do innej wersji bazowej.
- Surowe ścieżki indeksów tablic są jednak kruche dla punktów krzywej, węzłów i
  tracków po sortowaniu lub równoległej edycji. Journal powinien przechowywać
  typowane komendy na stabilnych ID i opcjonalnie eksportować JSON Patch do
  diagnostyki, a nie odwrotnie.
- JSON Merge Patch:
  <https://www.rfc-editor.org/rfc/rfc7396.html>
- Merge Patch używa `null` jako usunięcia i słabo opisuje struktury oparte na
  tablicach. Dlatego nie nadaje się na undo/redo ani journal złożonego grafu;
  może najwyżej obsłużyć proste ustawienia, w których `null` nie jest legalną
  wartością domenową.

#### Migracje wymagają korpusu historycznego

- Analiza zamrożenia formatu `.motion`:
  <https://github.com/isroil01/premation/blob/main/docs/MOTION_FORMAT_FREEZE.md>
- Dokument opisuje bundle z wersją, hashami chunków i content-addressed assets,
  ordered migration ladder oraz potrzebę testu otwierającego po jednym
  historycznym bundle'u z każdej wersji. Najważniejsza obserwacja: „nie rzuciło
  wyjątku” nie dowodzi poprawnej migracji; wynik trzeba porównać z oczekiwanym
  dokumentem i sprawdzić zachowanie nieznanych pól/chunków.
- Źródło samo zaznacza, że jest planem, a nie zakończonym freeze. Traktujemy je
  jako dobry design review, nie jako dowód wdrożenia.
- Wniosek dla `VfxProjectV1`: pojedynczy punkt `parse -> validate source schema
  -> migrate one step at a time -> validate current schema -> semantic hash`.
  Wersja z przyszłości musi otworzyć się read-only albo zostać jawnie odrzucona;
  nigdy nie może zostać automatycznie zapisana w okrojonej postaci.

### 24.4 Lifecycle strony i bezpieczny podgląd ruchu

#### Ukryta karta nie może zmieniać wyniku symulacji

- Page Visibility Level 2:
  <https://www.w3.org/TR/page-visibility-2/>
- API rozróżnia `visible` i `hidden` oraz emituje `visibilitychange`. Specyfikacja
  wprost wskazuje możliwość pauzowania lub ograniczenia pętli animacji.
- Po przejściu do `hidden` preview powinien zatrzymać render i dźwięk, zachować
  ostatni zatwierdzony tick oraz kontynuować autosave niezależnie od klatek.
  Powrót do `visible` nie może nadrabiać minut symulacji z wall-clock; resume
  startuje z zapisanego ticka lub wykonuje jawny `restart`.
- Testy eksportu i goldeny nie mogą zależeć od `visibilityState`. Są osobnymi,
  sterowanymi tickami jobami, nie odmianą pętli interaktywnej.

#### VFX może być fizycznie niebezpieczny dla części użytkowników

- WCAG 2.2, Three Flashes or Below Threshold:
  <https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold>
- Kryterium ogranicza więcej niż trzy general/red flashes w sekundzie, chyba że
  migający obszar pozostaje poniżej zdefiniowanego progu. Prosta liczba zmian
  średniej jasności całego canvasu nie wystarcza do wydania werdyktu.
- Pause, Stop, Hide:
  <https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html>
- Ruch lub miganie uruchamiane automatycznie i trwające ponad pięć sekund
  powinno mieć dostępny mechanizm pauzy, zatrzymania albo ukrycia. Jeden globalny
  `Pause Preview` jest lepszy niż szukanie przełącznika każdego emitera.
- Animation from Interactions:
  <https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions>
- Podgląd efektu jest istotną funkcją edytora, ale animacje chrome UI, przejścia
  paneli i automatyczne ruchy kamery nie są. Powinny respektować reduced motion.
- Proponowany `SafePreviewProfileV1`:

  - projekt otwiera się zatrzymany albo z łatwo dostępnym globalnym stop;
  - frame-step, scrub i statyczna miniatura działają bez ciągłej animacji;
  - dźwięk jest domyślnie wyciszony do świadomej akcji użytkownika;
  - analizator klatek zgłasza `flash-risk`, obszar, częstotliwość i przedział
    czasu, ale nie używa etykiety „WCAG pass” bez zwalidowanego algorytmu;
  - safe preview nigdy nie zmienia eksportowanego VFX — jest trybem oglądania,
    nie cichą korektą treści.

### 24.5 Dźwięk zsynchronizowany z tickiem i eksportem

#### OfflineAudioContext jako odpowiednik kontrolowanego render job

- Specyfikacja Web Audio:
  <https://webaudio.github.io/web-audio-api/>
- Dokumentacja `OfflineAudioContext`:
  <https://developer.mozilla.org/en-US/docs/Web/API/OfflineAudioContext>
- Offline context nie odtwarza przez hardware. Renderuje graf tak szybko, jak
  może, do `AudioBuffer` o jawnej liczbie sample frames, kanałach i sample rate.
- Precyzyjne planowanie źródła:
  <https://developer.mozilla.org/en-US/docs/Web/API/AudioBufferSourceNode/start>
- `start(when, offset, duration)` używa osi czasu kontekstu. Każdy event dźwięku
  VFX powinien najpierw otrzymać całkowity `sampleIndex`, wyliczony z rational
  tick time według jednej opisanej reguły zaokrąglania; dopiero potem jest
  planowany w sekundach. Sumowanie kolejnych zmiennoprzecinkowych `dt` tworzy
  dryf między obrazem i audio.
- Najprostszy powtarzalny baseline eksportu to sekwencja obrazów plus PCM/audio
  manifest. Mux i kompresja są kolejnym etapem, który nie może zmienić czasu
  eventów.

#### Preview audio wymaga jawnego sterowania użytkownika

- Web Audio best practices:
  <https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Best_practices>
- Realtime `AudioContext` może pozostać `suspended` do gestu użytkownika.
  Produkt potrzebuje widocznego `Enable audio`, mute, volume i stop; brak
  dźwięku przed aktywacją nie jest błędem assetu ani eksportera.
- Scrub nie powinien przypadkowo odgrywać setek krótkich próbek. Osobna polityka
  określa audition on scrub, minimalny odstęp retrigger i zachowanie przy
  cofaniu czasu.

#### WebCodecs audio ma te same bramki co wideo

- Runtime capability check:
  <https://developer.mozilla.org/en-US/docs/Web/API/AudioEncoder/isConfigSupported_static>
- Backpressure:
  <https://developer.mozilla.org/en-US/docs/Web/API/AudioEncoder/encodeQueueSize>
- Zwolnienie danych:
  <https://developer.mozilla.org/en-US/docs/Web/API/AudioData/close>
- Konfigurację kodeka trzeba sprawdzać w runtime, ograniczać kolejkę i jawnie
  zwalniać `AudioData`. `flush`/close/cancel muszą być częścią stanu joba tak
  samo jak w exporterze wideo z R5.
- Nawet dla identycznego PCM wynik skompresowanego encodera może zależeć od
  hosta. Raport powinien odróżnić `sample-deterministic input` od
  `byte-deterministic encoded artifact` i zapisać codec/config w manifeście.

### 24.6 Wyniki odrzucone albo zdegradowane w R6

- Nowe, młode repozytoria nie stają się zależnościami produktu tylko dlatego,
  że ich README dobrze opisuje nasze problemy. Są źródłami spike'ów i UX.
- WebGPU device recovery nie przywraca starych uchwytów. Każda próba ich
  zachowania po `lost` jest błędem architektury.
- `uncapturederror` nie zastępuje lokalnych error scopes ani przypisania błędu
  do operacji użytkownika.
- Asynchroniczna kompilacja nie oznacza, że wolno pokazać pół-zbudowany efekt.
  Ostatni poprawny pipeline jest stale preview, a nie dowodem aktualnego grafu.
- JCS nie kanonikalizuje źródłowego YAML i nie waliduje semantyki Aurory.
  Semantic hash powstaje dopiero po parse/validation/migration.
- JSON Patch nie zastępuje typowanego command modelu, a Merge Patch nie nadaje
  się do grafów, tracków i krzywych opartych na tablicach.
- Heurystyczny flash detector nie może wystawiać certyfikatu dostępności.
- Safe preview nie może cicho osłabiać jasności albo częstotliwości artefaktu
  eksportowanego do NWN.
- `OfflineAudioContext` stabilizuje harmonogram wejścia, lecz sam nie daje
  gwarancji identycznych bajtów stratnego kodeka na wszystkich hostach.
- Unityowe narzędzia mesh/texture są referencją workflow, nie częścią
  eksportera Aurory ani uzasadnieniem dla dodatkowego formatu pośredniego.

### 24.7 Nowe wnioski architektoniczne R6

1. **Renderer jest wymienialną projekcją dokumentu.** Utrata WebGL/WebGPU nie
   może zagrozić projektowi, journalowi ani source assetom.
2. **Recovery ma generation ID.** Każdy async upload, compile i readback jest
   związany z konkretną generacją renderera; wynik starej generacji odpada.
3. **Capabilities liczymy ponownie po recovery.** Nowe urządzenie może mieć
   inne features i limits, więc dawny raport nie jest dziedziczony.
4. **Ostatni poprawny preview może być pokazany jako stale.** Jest to lepsze niż
   czarny canvas, jeśli UI jednoznacznie wskazuje, że nowy graf jeszcze się nie
   skompilował.
5. **Tożsamość projektu ma kilka warstw.** Raw-file, semantic-document, asset
   manifest, compiler/target profile i final artifacts mają osobne hashe.
6. **Kanoniczny semantic hash ma jeden kontrakt między TS i Rust.** Nie może
   zależeć od kolejności pól, locale, pretty-printingu ani przypadkowego YAML.
7. **Journal używa stable IDs i typed commands.** JSON Patch może być formatem
   wymiany lub debug view, ale nie powinien definiować modelu domenowego.
8. **Migracje są częścią kompatybilności produktu.** Każda historyczna wersja
   potrzebuje zamrożonego fixture i oczekiwanego dokumentu po migracji.
9. **Hidden-tab lifecycle nie steruje czasem efektu.** Symulacja zatrzymuje się
   na ticku; eksport pozostaje osobnym jobem z własnym zegarem.
10. **Safe Preview jest wymaganiem produktu, nie kosmetyką.** Globalny stop,
    frame-step, reduced-motion chrome, mute i flash-risk diagnostics powinny
    wejść do projektu UI wcześnie.
11. **Audio i obraz korzystają ze wspólnej racjonalnej osi czasu.** Event
    wizualny i sample audio pochodzą z tego samego ticka, nie z dwóch zegarów.
12. **Narzędzia do tekstur i meshów są małymi, opcjonalnymi laboratoriami.** Ich
    operation stacks mogą później zasilić effect assets, ale dopiero po
    wiernym P0 emitera i target writer/readbacku.

### 24.8 Kandydaci do spike'ów po aktywnych gate'ach

1. `RendererRecoveryV1`: wymuszone unieważnienie generacji w trakcie texture
   upload, shader compile i frame readback;
2. WebGPU recovery test: utrata/destroy, ponowny adapter/device request,
   ponowne capabilities i odbudowa tej samej sceny z CPU state;
3. error-scope matrix dla validation/OOM/internal z przypisaniem do node ID,
   asset ID i operation ID;
4. leak soak: 100 zmian projektu/hot reloadów z kontrolą liczby texture,
   geometry, pipeline, render targets i otwartych `ImageBitmap`;
5. JCS corpus wykonywany identycznie w TypeScript i Rust, zakończony wspólnym
   SHA-256;
6. `rawFileSha256 != projectSemanticSha256` golden dla YAML różniącego się
   komentarzami, kolejnością pól i formatowaniem, ale nie semantyką;
7. migration corpus `V1 -> current`, future-version refusal i preservation
   nieznanych pól/chunków bez ponownego zapisu stratnego;
8. JSON Patch experiment pokazujący, dlaczego indeks tablicy jest gorszy od
   stable curve-point/node ID po reorder;
9. lifecycle test: hide/show nie zmienia ticka, seeda ani golden frame, a
   exporter działa niezależnie od widoczności UI;
10. flash-risk analyzer na syntetycznych sekwencjach: whole-screen white,
    saturated red, mały obszar i szum/checkerboard;
11. offline audio golden: dwa eventy na granicach tick/sample, stały PCM hash i
    test braku dryfu po długiej kompozycji;
12. mini operation stack dla linear mask/alpha z preview, undo i docelowym
    TGA/DDS readbackiem — bez wprowadzania PNG jako target truth.

### 24.9 Nowe pytania po R6

1. Czy P0 uruchamia preview w WebGL2, a WebGPU jest wyłącznie opcjonalnym
   backendem, czy oba muszą mieć recovery przed pierwszym wydaniem?
2. Ile automatycznych prób ponownego uzyskania urządzenia wykonujemy i kiedy
   przechodzimy do statycznego/read-only preview?
3. Jakie zasoby są współdzielone między emitterami i kto jest ich formalnym
   właścicielem podczas hot reloadu?
4. Czy stale preview ma przyjmować dalsze zmiany inspectora, czy UI blokuje
   export do czasu skompilowania bieżącej generacji?
5. Które pola trafiają do semantic hash, a które są tylko workspace state,
   recovery metadata albo cache?
6. Czy wszystkie liczby dokumentu mieszczą się w kontrakcie IEEE 754 double,
   czy ticki, asset sizes i identyfikatory wymagają string/int policy?
7. Jak zachować nieznane pola dokumentu z przyszłej wersji bez dopuszczania ich
   do compilera i bez utraty podczas Save As?
8. Czy undo/redo journal ma być migrowany razem z projektem, czy po otwarciu
   starej wersji zaczyna się nowa historia od zmigrowanego snapshotu?
9. Czy podgląd startuje zatrzymany globalnie, czy zapamiętuje świadomą
   preferencję użytkownika per workspace?
10. Jaki minimalny flash-risk test jesteśmy gotowi nazwać ostrzeżeniem, a jaki
    wymaga zewnętrznego, zwalidowanego analizatora?
11. Jak mapujemy tick, duration i looping dźwięku NWN na sample indices, gdy
    sample rate nie dzieli się całkowicie przez simulation Hz?
12. Czy bazowym artefaktem udostępniania ma być niemy film + osobny WAV, czy
    od początku jeden muxowany plik z capability-dependent kodekiem?

## 25. Runda internetowa R7 — 2026-09-02

R7 szukała mechanizmów, których nie domknęły wcześniejsze katalogi edytorów:
odtwarzalnego authoringu flipbooków, weryfikowanego compilera grafu, uczciwego
budżetu pamięci oraz przenośnej paczki projektu z pełnym provenance. Największa
nowa wartość nie pochodzi z kolejnego particle runtime, lecz z połączenia
wzorców narzędzi filmowego VFX, kompilatorów i systemów dystrybucji assetów.

### 25.1 Flipbook jako niedestrukcyjna receptura, nie jednorazowy obraz

#### Unity VFX Toolbox i Image Sequencer

- Repozytorium: <https://github.com/Unity-Technologies/VFXToolbox>
- Dokumentacja Image Sequencer:
  <https://github.com/Unity-Technologies/VFXToolbox/blob/master/Documentation~/ImageSequencer.md>
- Stan podczas R7: około 1,3 tys. gwiazdek i 84 commity. Projekt jest
  dostarczany `as-is`, bez gwarancji aktualizacji, na Unity Companion License.
  To oznacza `workflow/UX reference only`; kodu nie przenosimy bez osobnej
  analizy licencji.
- Image Sequence Asset jest trwałym szablonem projektu: wskazuje klatki
  źródłowe, uporządkowany stos processorów i ustawienia eksportu. Autor może
  wrócić do źródeł, zmienić operację i ponownie wygenerować wynik jednym
  poleceniem.
- Przydatne operacje obejmują assemble/disassemble sheet, retime, decimate,
  loop blend, crop, border fix, transformacje koloru i eksport alpha razem lub
  osobno. Podgląd można zatrzymać na wejściu, dowolnym processorze albo wyniku.
- Narzędzie rozdziela color, mask i normal-map output, sRGB od linear data,
  frame playback od mip preview oraz źródła pośrednie od finalnego assetu.
  Eksport opisuje grid, wrap, filter i mipmap policy zamiast pozostawiać je
  jako ukryte ustawienia importera.
- Wniosek: flipbook Meshy2Aurora powinien być wersjonowanym `BakeRecipeV1`, nie
  tylko ścieżką do wygenerowanego DDS/TGA. Minimalny zapis receptury:

```text
BakeRecipeV1
  sourceFrames[]: logicalPath + sha256 + dimensions + declaredColorRole
  processors[]: stableId + kind + version + enabled + parameters
  layout: ordering + rows + columns + padding + orientation
  outputProfile: targetFormat + alphaMode + mipmapPolicy + colorEncoding
  expectedOutputs[]: logicalPath + semanticRole
```

- Każdy processor dostaje jawny kontrakt wejścia/wyjścia. Zmiana kolejności,
  wersji algorytmu lub parametru zmienia semantic recipe hash. Cache jest
  pochodną `source hashes + normalized recipe + implementation revision`, a
  nie nieweryfikowanym plikiem tymczasowym.
- Podgląd potrzebuje przełączników RGBA, checkerboard, frame-step, grid, seam,
  mip level i `before/after`. Zablokowanie widoku na wcześniejszym etapie nie
  może zablokować edycji dalszej części stosu.
- Source frames pozostają niezmienne. Aktualizacja wyniku najpierw generuje
  nowy kandydat, wykonuje własny TGA/DDS readback, sprawdza wymiary, orientację,
  alpha i mip chain, a dopiero potem atomowo przełącza output reference.

#### Jawne zarządzanie kolorem bez budowania studia filmowego w P0

- Repozytorium OpenColorIO:
  <https://github.com/AcademySoftwareFoundation/OpenColorIO>
- Authoring konfiguracji:
  <https://opencolorio.readthedocs.io/en/latest/guides/authoring/authoring.html>
- Displays i Views:
  <https://opencolorio.readthedocs.io/en/latest/guides/authoring/displays_views.html>
- OCIO rozdziela przestrzenie robocze, role, file rules, transformacje
  wyświetlania i finalny output. Konfiguracja jest wersjonowana oraz ma osobny
  validator. Repozytorium jest BSD-3-Clause i szeroko używane w VFX, lecz pełna
  integracja OCIO byłaby zbyt duża dla pierwszego edytora Aurory.
- Wzorzec, który warto przejąć od razu: `source encoding`, `working encoding`,
  `display transform` i `target encoding` są czterema osobnymi pojęciami.
  Display transform zmienia wyłącznie oglądanie; nie wolno go przypadkiem
  wypiec w masce, flow mapie ani finalnej teksturze NWN.
- `ColorManagementProfileV1` powinien jawnie oznaczać przynajmniej:
  `color-srgb`, `color-linear`, `data-linear`, `normal-data` i `unknown`.
  `unknown` blokuje automatyczną nieliniową transformację i wymaga wyboru.
- Jeżeli później dojdzie OCIO, manifest musi wiązać exact config/LUT hashes,
  role oraz display/view użyte wyłącznie do podglądu. Sama nazwa `ACES` albo
  `sRGB` nie wystarcza do powtórzenia renderu.

### 25.2 Graf VFX powinien zachowywać się jak mały, kontrolowany kompilator

#### MLIR jako wzorzec kontraktów IR i pass pipeline

- Operation Definition Specification:
  <https://mlir.llvm.org/docs/DefiningDialects/Operations/>
- Pass infrastructure:
  <https://mlir.llvm.org/docs/PassManagement/>
- ODS pokazuje centralną, typowaną definicję operacji, operandów, wyników,
  właściwości, constraints i verifierów zamiast rozsianych porównań stringów.
  Pass manager rozdziela transformacje, może przerwać pipeline po naruszeniu
  inwariantu i potrafi zapisać reproducer z wejściowym IR, pipeline'em i
  opcjami.
- Nie proponujemy zależności od MLIR. Przejmujemy małą dyscyplinę:

```text
VfxProject source
  -> parse + source schema validation
  -> normalized typed graph
  -> target-independent semantic IR
  -> Aurora capability lowering
  -> resource/artifact plan
  -> native writers
  -> semantic readback
```

- Po każdej strzałce działa verifier. Jeżeli pass pozostawi niepoprawny stan,
  dalsze etapy nie uruchamiają się, a ostatni poprawny preview pozostaje
  oznaczony `stale`. Compiler nie naprawia potajemnie grafu i nie emituje
  częściowego HAK/MOD.
- Node definition jest pojedynczym źródłem prawdy dla typu pinów, wymaganych
  pól, zakresów, skutków ubocznych, legalnych targetów i diagnostyki. UI,
  parser, migracje i compiler konsumują ten sam katalog definicji.
- Każda transformacja ma stabilną nazwę, wersję, opcje, hash wejścia i hash
  wyjścia. Reproducer błędu zawiera minimalnie source project hash, wejściowe
  IR danego passu, ordered pipeline, compiler revision i target profile.
- Optymalizacja nie może zmienić liczby spawnów, kolejności eventów,
  zaokrągleń czasu albo float semantics bez osobnego dowodu równoważności.
  Pierwszy pipeline ma być prosty i audytowalny, nie „sprytny”.

#### Naga i WGSL jako drugi verifier podglądu, nie prawdy Aurory

- Aktywna dokumentacja Naga w repozytorium wgpu:
  <https://github.com/gfx-rs/wgpu/blob/trunk/naga/README.md>
- Specyfikacja WGSL: <https://www.w3.org/TR/WGSL/>
- Naga parsuje i waliduje WGSL oraz tłumaczy kilka wejść/wyjść shaderowych.
  Może działać offline/CI, zanim przeglądarka utworzy `GPUShaderModule` i
  pipeline. To daje niezależną warstwę diagnostyki dla generowanego shadera.
- WGSL rozdziela shader-creation, pipeline-creation i dynamic errors. Raport
  nie powinien scalać ich w ogólne `shader failed`: zapisuje phase, source
  span, node ID, property path, backend i shader generation.
- Decyzja: `optional dev/CI oracle`. Browserowe compilation info pozostaje
  runtime oracle, a retail/Aurora pozostaje jedyną prawdą targetu. Poprawny
  WGSL nie dowodzi poprawnego emittera NWN.
- Stare samodzielne repozytorium Naga:
  <https://github.com/gfx-rs/naga>
  zostało zarchiwizowane; development przeniósł się do `gfx-rs/wgpu`. Nie
  przypinamy narzędzia do archiwalnego źródła ani nie dublujemy obu zależności.

### 25.3 Budżet zasobów: własna księga, nie zgadywanie wolnego VRAM-u

#### Limits i features są capabilities, nie miernikiem wolnej pamięci

- `GPUSupportedLimits`:
  <https://developer.mozilla.org/en-US/docs/Web/API/GPUSupportedLimits>
- `GPUAdapter.requestDevice()`:
  <https://developer.mozilla.org/en-US/docs/Web/API/GPUAdapter/requestDevice>
- `GPUSupportedFeatures`:
  <https://developer.mozilla.org/en-US/docs/Web/API/GPUSupportedFeatures>
- Adapter opisuje dostępne feature'y i górne limity konstrukcji, a
  `requestDevice` odrzuca niespełnione wymagania. Nie jest to API podające
  aktualnie wolny VRAM. Jest to wniosek z zakresu udostępnionych interfejsów,
  nie deklaracja konkretnego sterownika.
- Żądać należy tylko tych limits/features, które są potrzebne bieżącemu
  backendowi. Maksymalne wartości „na zapas” nie tworzą pamięci ani jakości,
  za to mogą niepotrzebnie odrzucić słabsze urządzenie.
- `CapabilityReportV1` odpowiada więc na pytanie „czy operacja jest legalna”,
  a osobny `ResourceLedgerV1` na „ile sami zamierzamy zaalokować i jaki będzie
  szczyt”. Tych raportów nie wolno scalać.

#### Pomiar pamięci strony ma ograniczony zakres

- `measureUserAgentSpecificMemory()`:
  <https://developer.mozilla.org/en-US/docs/Web/API/Performance/measureUserAgentSpecificMemory>
- API szacuje pamięć aplikacji, iframe'ów i workerów, wymaga secure context oraz
  cross-origin isolation, nie ma pełnego wsparcia przeglądarek, a wyniki są
  zależne od implementacji i nieporównywalne między przeglądarkami/wersjami.
- Decyzja: może wykrywać trend i regresję podczas soak testu, ale nie może być
  bramką importu, obietnicą wolnego RAM/VRAM ani wspólnym benchmarkiem hostów.

#### Praktyczna referencja do logicznego liczenia WebGPU

- Repozytorium: <https://github.com/greggman/webgpu-memory>
- Stan podczas R7: MIT, 81 gwiazdek i 50 commitów. Instrumentuje tworzenie
  bufferów, tekstur, canvasów i innych resource types oraz podaje bieżące i
  szczytowe wartości.
- Autor jawnie nazywa wynik przybliżeniem: GPU mogą mieć inne alignment i
  koszty wewnętrzne. Projekt nie obsługuje OOM ani lost context, a czas zwolnienia
  obiektów zarządzanych przez GC nie jest określony.
- Decyzja: `diagnostic/spike reference`, nie produkcyjna prawda pamięci.
  Produkcyjny ledger powinien być częścią naszych wrapperów zasobów i testów,
  a narzędzie zewnętrzne może sprawdzić, czy niczego oczywistego nie pominięto.
- `ResourceLedgerV1` powinien liczyć co najmniej:

  - bajty źródłowe, zdekodowane klatki i CPU simulation buffers;
  - jawny `size` wszystkich GPUBuffer;
  - szacowaną sumę mipów/layers/texel blocks każdej GPUTexture;
  - canvas, depth, color, MSAA i tymczasowe render targets;
  - staging/readback oraz jednoczesny stary i nowy generation podczas swapu;
  - cache bake'ów, blob storage i rozpakowany rozmiar importowanej paczki;
  - current, peak, owner, generation, resource kind i source node/asset ID.

- Każda ciężka komenda najpierw tworzy `ResourcePlan`: expected delta, peak
  delta, top offenders i reclaim candidates. Własny miękki limit daje warning,
  twardy limit blokuje przed alokacją. `GPUOutOfMemoryError` pozostaje awarią
  ostatniej szansy, nie mechanizmem planowania.

### 25.4 Przenośny VFX pack z domknięciem zależności i provenance

#### USDZ jako dojrzały wzorzec pojedynczego, walidowalnego transportu

- Specyfikacja USDZ: <https://openusd.org/release/spec_usdz.html>
- USDZ pokazuje istotny problem DCC: modularny asset jest wygodny w authoringu,
  lecz trudny do przekazania, jeśli część zależności rozwiązuje się tylko na
  komputerze autora. Narzędzie `usdzip --asset` odnajduje zależności, lokalizuje
  je w paczce, poprawia odwołania i może wykonać compliance check przed
  utworzeniem wyniku.
- Dobre wzorce dla VFX pack: jeden default/root document, allowlista typów,
  package-relative anchored paths, read-only published package, list/inspect
  bez pełnej ekstrakcji oraz validator uruchamiany przed finalizacją.
- Nie przyjmujemy USDZ jako formatu Meshy2Aurora. Jego alignment, dozwolone
  typy, brak kompresji i semantyka USD odpowiadają innemu runtime. Przejmujemy
  wyłącznie regułę: publiczna paczka musi zawierać pełne, lokalnie rozwiązywalne
  dependency closure albo jawnie nie przechodzi validation.

#### Minimalny manifest zamiast pełnego standardu SBOM

- SPDX 3.0.1 scope:
  <https://spdx.github.io/spdx-spec/v3.0.1/scope/>
- SPDX License List i expressions: <https://spdx.org/licenses/>
- CycloneDX object model:
  <https://cyclonedx.org/specification/overview/>
- SPDX i CycloneDX potrafią opisywać komponenty, pliki, zależności, licencje,
  hashe i provenance. CycloneDX rozróżnia nawet complete/incomplete/unknown
  composition oraz observed/declared formulation.
- Pełny SBOM w każdym artystycznym presecie byłby przerostem formy. Warto
  jednak użyć ich słownika: exact digest, stable component/asset reference,
  dependency edges, license expression, provenance oraz jawna kompletność.
- Proponowany `VfxPackManifestV1`:

```text
packVersion + packId + createdBy
rootProject: logicalPath + mediaType + sha256
files[]: logicalPath + role + mediaType + bytes + sha256
         + licenseExpression + provenance + required
dependencies[]: fromAssetId + toAssetId + kind
compatibility: projectSchema + compilerRange + targetProfiles
composition: complete | incomplete | unknown
```

- `NOASSERTION`, brak licencji i brak provenance są różnymi stanami. UI nie
  powinno nazywać ich `public domain`, a exporter może mieć politykę blokującą
  dystrybucję paczki przy niepełnej deklaracji.

#### Hash wiąże bajty; attestation może później związać wynik z producentem

- in-toto Statement:
  <https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md>
- Model walidacji:
  <https://github.com/in-toto/attestation/blob/main/docs/validation.md>
- Statement wiąże predicate z immutable subject identyfikowanym digestem.
  Walidator najpierw sprawdza envelope/signature, potem typ statementu, a
  następnie dopasowuje subject przez hash rzeczywistych bajtów.
- Wniosek: nie wdrażać podpisów w P0, ale nie projektować ślepego zaułka.
  `BuildStatementV1` powinien już dziś wiązać semantic project hash, recipe
  hashes, wszystkie input asset hashes, compiler revision, target profile,
  validation report i final artifact hashes. W przyszłości tę samą treść można
  otoczyć podpisanym envelope bez zmiany znaczenia builda.
- Sam hash odpowiada na pytanie „czy to te same bajty”, nie „kto je zatwierdził”
  ani „czy wolno je dystrybuować”. Podpis nie zastępuje licencji i provenance.

#### SRI jest dobrym wzorcem integralności, ale nie podpisem paczki

- Subresource Integrity: <https://www.w3.org/TR/SRI/>
- SRI wiąże pobierany zasób z silnym digestem i wymaga od przeglądarki
  weryfikacji dostarczonych bajtów. Ten sam model warto stosować do zdalnych
  assetów: manifest podaje algorytm i digest, download trafia do staging, a
  dopiero zgodny hash dopuszcza blob do content-addressed store.
- SRI nie dowodzi autorstwa i standard obecnie nie jest ogólnym mechanizmem dla
  wszystkich rodzajów zasobów. Nie opisujemy go jako podpisu, DRM ani licencji.

### 25.5 Import archiwum jest transakcją na niezaufanych danych

- OWASP File Upload Cheat Sheet:
  <https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html>
- OWASP archive traversal, symlink i decompression-bomb tests:
  <https://wstg.owasp.org/latest/4-Web_Application_Security_Testing/10-Business_Logic_Testing/09-Test_Upload_of_Malicious_Files/>
- Rust `zip::ZipFile::enclosed_name()`:
  <https://docs.rs/zip/latest/zip/read/struct.ZipFile.html>
- OWASP zaleca allowlistę typów, niezależną walidację treści, limity nazwy i
  wielkości oraz liczenie rozmiaru także po dekompresji. Testy pokazują osobne
  zagrożenia: path traversal, symlinks i archive/decompression bombs.
- `enclosed_name()` jest użytecznym kandydatem dla backendu Rust, bo odrzuca
  absolute paths, NUL i ścieżki wychodzące poza katalog. Nie wystarcza sam:
  importer nadal musi odrzucić duplicate normalized names, alternatywne
  kodowania separatorów, nested archives, symlinks i nieznane typy.
- Bezpieczna kolejność importu `VfxPackV1`:

  1. odczytaj central directory i manifest w izolowanym staging/workerze;
  2. zastosuj limity compressed bytes, expanded bytes, file count, dimensions,
     nesting depth i per-file size bez zapisu do projektu;
  3. znormalizuj logical paths i odrzuć absolute, traversal, NUL, collision,
     reserved names, symlinks oraz wpis poza allowlistą;
  4. sprawdź magic/content i schema niezależnie od extension/MIME;
  5. strumieniowo policz SHA-256 i porównaj każdy wpis z manifestem;
  6. sprawdź root, dependency closure, required assets, licenses i target
     compatibility;
  7. wykonaj parse/migration/semantic validation projektu w pamięci;
  8. zapisz blobs do content-addressed store, a HEAD projektu przełącz dopiero
     w jednej końcowej transakcji.

- Import nie uruchamia JS, WASM, plugina ani shadera z paczki. Tekst SHD może
  być zachowany jako niezaufany source asset, ale pozostaje disabled/read-only,
  dopóki jawna polityka targetu i walidator go nie dopuszczą.
- Nie trzeba zachowywać bajtowego hasha paczki jako jedynej tożsamości
  semantycznej, ponieważ kolejność wpisów i metadata ZIP mogą zmienić archive
  bytes bez zmiany zawartości. Zapisujemy osobno:

  - `packFileSha256` dokładnego transportu;
  - `packSemanticSha256` kanonicznego manifestu i ordered file digests;
  - każdy indywidualny `assetSha256`.

### 25.6 Wyniki odrzucone albo zdegradowane w R7

- Kolejne młode runtime'y WebGPU powtarzają listę z wcześniejszych rund. Bez
  nowego formatu, testów, licencji albo workflow nie zwiększają już jakości
  decyzji i nie zostały ponownie dopisane.
- Unity VFX Toolbox nie jest zależnością produktu z powodu Unity Companion
  License i statusu `as-is`. Jego Image Sequencer pozostaje bardzo dobrą
  referencją zachowania.
- Pełne OCIO nie wchodzi do P0. Wchodzą jawne role koloru i rozdział working,
  display oraz target transform; ewentualna biblioteka wymaga osobnego spike'u.
- MLIR byłby nieproporcjonalnym dependency. Wchodzą verifier-per-pass,
  typowane operacje, tekstowa identyfikacja pipeline'u i crash reproducer.
- Zarchiwizowane repo Naga nie jest źródłem aktualnej wersji; właściwa linia
  rozwoju znajduje się w wgpu.
- Naga i poprawny WGSL nie walidują semantyki Aurory ani wyglądu retail.
- Adapter limits nie są wielkością wolnego VRAM-u, a pomiar pamięci strony nie
  zastępuje deterministycznej księgi własnych alokacji.
- `webgpu-memory` jest przybliżeniem i nie obsługuje device loss/OOM. Nie może
  stanowić bramki produkcyjnej.
- USDZ jest analogią package/dependency closure, nie nowym targetem ani
  pośrednim formatem VFX.
- Pełny SPDX/CycloneDX BOM w każdym presecie jest zbyt ciężki. Zachowujemy
  kompatybilne identyfikatory licencji i podstawowe pola provenance.
- SRI i SHA-256 nie są podpisem autora, oceną bezpieczeństwa ani prawem do
  redystrybucji.
- Zwykłe `unzip` do katalogu projektu jest niedopuszczalne. Walidacja całej
  paczki poprzedza jakąkolwiek widoczną zmianę stanu projektu.

### 25.7 Nowe wnioski architektoniczne R7

1. **Flipbook jest buildem z receptury.** Źródła, ordered processors, profile
   koloru, layout i output policy mają własną wersję oraz semantic hash.
2. **Podgląd procesu jest adresowalny.** Użytkownik może oglądać wejście,
   dowolny processor, final sheet, kanał i mip bez mutowania artefaktu.
3. **Display transform nigdy nie jest ukrytym bake'em.** Color assets i data
   assets mają różne role; `unknown` wymaga świadomej decyzji.
4. **Compiler ma verifier po każdym etapie.** Błąd zatrzymuje pipeline przed
   writerem, a diagnostyka wraca do node ID i property path.
5. **Każdy pass jest reprodukowalny.** Nazwa, wersja, opcje, input/output hash
   i compiler revision pozwalają odtworzyć awarię bez całego UI.
6. **Shader preview ma dwa oracles.** Naga/CI i browser runtime sprawdzają
   generowany WGSL, ale żaden nie zastępuje Aurora/retail proofu.
7. **Capabilities i budgets są osobne.** Limits mówią, co wolno utworzyć;
   ResourceLedger mówi, co produkt planuje, posiada i powinien zwolnić.
8. **Budżet obejmuje peak transients.** Hot swap, bake i readback chwilowo
   trzymają więcej niż finalna scena; tylko current bytes ukryłyby awarie.
9. **VFX pack ma jeden root i pełny dependency closure.** Każdy wymagany asset
   ma package-relative path, digest, rolę, licencję i provenance.
10. **Transport hash i semantic pack hash są niezależne.** Zmiana metadata ZIP
    nie musi oznaczać zmiany projektu, ale exact pobrane bajty nadal są znane.
11. **Import jest fail-closed i atomowy.** Staging, limity, ścieżki, content,
    hashe, schema i closure przechodzą validation przed zmianą HEAD projektu.
12. **Attestation-ready nie oznacza signed-now.** Build statement można od
    początku wiązać digestami, a podpis i politykę zaufania dodać później.

### 25.8 Kandydaci do spike'ów po aktywnych gate'ach

1. `BakeRecipeV1` dla 8 lossless frames: retime, loop blend, crop, dilation,
   uniform grid, TGA/DDS export i własny readback;
2. recipe cache invalidation matrix: source byte, processor order, parameter,
   implementation revision i target profile;
3. color fixture: color-sRGB, data-linear i normal-data pokazane przez ten sam
   display view bez zmiany target bytes;
4. pięcioetapowy mini-compiler z `verifyEach`, source spans i celowo wadliwym
   passem, który nie może dotrzeć do writera;
5. crash reproducer zawierający wejściowe IR, ordered pass pipeline, opcje i
   compiler revision;
6. Naga CLI/library validation na corpusie poprawnych i błędnych shaderów
   wygenerowanych z node graphu, porównana z browser compilation info;
7. `ResourceLedgerV1` z exact buffer bytes, block-rounded texture estimate,
   render targets, owner/generation i peak podczas hot swapu;
8. soak 100 project reloads: ledger wraca do baseline, a pomiar strony służy
   wyłącznie jako trend pomocniczy;
9. `VfxPackManifestV1` z root project, dwoma teksturami, audio, shader source,
   dependency edges, SPDX expressions i complete composition;
10. deterministyczny pack test: różne ZIP metadata dają różny transport hash,
    lecz ten sam semantic pack hash;
11. malicious pack corpus: traversal, absolute path, NUL, duplicate normalized
    name, symlink, nested archive, ratio bomb, oversized image i hash mismatch;
12. transactional import crash injection przed i po blob write, ale przed HEAD
    commit; żaden przypadek nie może odsłonić połowy projektu.

### 25.9 Nowe pytania po R7

1. Które processory flipbooka wchodzą do P0: crop, pad/dilate, retime, loop,
   channel pack, levels i grid — czy jeszcze transform/rotate?
2. Czy receptura zapisuje per-frame duration, czy tylko jedną rational FPS i
   mapowanie output frame do source time?
3. Jak dokładnie border dilation współdziała z alpha i mip generation dla
   texture atlasu Aurory?
4. Czy color management P0 ma wyłącznie stałe profile sRGB/linear/data, czy od
   początku przyjmuje zewnętrzny config hash?
5. Które inwarianty są lokalne dla node'a, które dla całego grafu, a które
   dopiero dla target capability lowering?
6. Jakie passy wolno cache'ować i czy ich wynik jest stabilny między TS, WASM i
   native Rust?
7. Jaki miękki i twardy memory budget przyjmujemy dla desktop browser, a jaki
   dla słabszego compatibility profile?
8. Czy ledger liczy także persistent OPFS/IndexedDB quota, czy storage ma
   osobny budżet i politykę retencji?
9. Czy pierwsza wersja VFX pack używa ZIP bez kompresji, zwykłego ZIP z
   allowlistą metod, czy innego kontenera po benchmarku streaming/readback?
10. Czy shader source może być częścią publicznej paczki domyślnie, czy wymaga
    osobnego trust flag i ręcznego włączenia?
11. Czy niepełne provenance blokuje tylko publikację, czy również lokalny
    import i edycję w trybie quarantine/read-only?
12. Jaki podmiot i klucz mógłby kiedyś podpisywać build statement: lokalny
    użytkownik, CI projektu, katalog presetów czy kilka niezależnych ról?
