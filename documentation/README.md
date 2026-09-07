# Documentation

- [Audyt higieny Git — 2026-09-07](audyt-higieny-git-2026-09-07.md) — klasyfikacja lokalnych payloadów, porządki `.gitignore`, zachowanie plików po usunięciu z indeksu i wyniki bramek jakości przed push.

- [Creature remediation and authoring plan — 2026-09-05](brainstorm/creature-remediation-and-authoring-plan-2026-09-05.md) - proposed CR-00–CR-13 delivery backlog covering all 19 current audit findings, durable Creature projects, materials/export fidelity, animation/event authoring, visual weight repair, reusable recipes and owned supermodels; includes dependencies, tests, research gates and owner-verification boundaries.

- [badania-vfx-zrodla-i-wnioski-2026-09-02.md](badania-vfx-zrodla-i-wnioski-2026-09-02.md) - żywy rejestr kolejnych badań nad edytorem VFX: źródła NWN/Aurora i porównawcze, licencje, ograniczenia, architektura React + Rust/WASM, capability report, pipeline eksportu, testy i rekomendowane priorytety.

- [reference-supermodel-rig-authoring-v1-implementation-2026-08-25.md](reference-supermodel-rig-authoring-v1-implementation-2026-08-25.md) - wdrożony hash-bound authoring target riga dla dowolnego supermodelu: edycja TRS/roli/osi jointów i sparse wag w Studio, wspólny preview/product V3 oraz fail-closed ochrona nazw i hierarchii carrierów.

- [skeleton-joints-preview-v1-implementation-2026-08-21.md](skeleton-joints-preview-v1-implementation-2026-08-21.md) - wdrożony semantyczny podgląd riga w Studio: osobne kości, jointy, helpery/attachmenty, etykiety, X-Ray, bind/rest pose, inspector węzła i per-clip binding report; realny `c_wolf/crun` na `c_barghest` ma 25/25 związanych jointów.

- [creature-stages-1-4-5-6-7-implementation-2026-08-19.md](creature-stages-1-4-5-6-7-implementation-2026-08-19.md) - wdrożenie pełnego authoringu GIT/UTC i loadoutu V10, runtime envelope, rodzin chwytu, source-bound MotionPack/quadruped intake, animowanego MTR oraz raportów wydajności Creature.

- [audyt-tworzenia-przedmiotow-plan-implementacji-2026-08-17.md](audyt-tworzenia-przedmiotow-plan-implementacji-2026-08-17.md) - plan i checkpoint wdrożenia produkcyjnego targetu `ITEM`: gotowy częściowo offline Core/WASM/Worker, profile `baseitems.2da.ModelType` 0/1/2/3, party MDL, UTI, ikony, namespace/collision gates, HAK/MOD, otwarte Studio/E2E i human-owned proof oraz mierzalne Definition of Done.

- [placeable-v9-material-pipeline-implementation-2026-08-13.md](placeable-v9-material-pipeline-implementation-2026-08-13.md) - ukończona implementacja Placeable V9: compiler materiałów, quality gate, MDL/TGA/MTR/TXI, HAK/WASM/Worker, Aurora Export w Studio i pełna macierz regresji.

- [placeable-v9-implementation-checkpoint-2026-08-12.md](placeable-v9-implementation-checkpoint-2026-08-12.md) - checkpoint wstrzymanej na polecenie właściciela implementacji Placeable V9: wykonane etapy, potwierdzone testy i dokładna kolejność wznowienia.

- [weapon-grip-preview-v1-implementation-2026-08-03.md](weapon-grip-preview-v1-implementation-2026-08-03.md) - wdrożony podgląd chwytu Creature w Studio: proxy miecza na exact `rhand`/`lhand`, kontrolki Off/Right/Left, osie hooka oraz fail-closed zgodność raportu Core z binary-MDL readback.

- [audyt-separacji-materialow-plan-implementacji-2026-08-01.md](audyt-separacji-materialow-plan-implementacji-2026-08-01.md) - audyt repozytorium, wspólna architektura Material Separation dla Creature, Placeable, Tile i przyszłych Itemów, plan MS0-MS8 oraz mierzalne warunki ukończenia.
- [evidence/material-separation-offline-validation-2026-08-01.md](evidence/material-separation-offline-validation-2026-08-01.md) - kompletna walidacja Core/WASM/Worker/UI, realny preview A/B i finalne hashe bindingów.
- [evidence/material-separation-placeable-v1-ready-for-owner-proof-2026-08-01.md](evidence/material-separation-placeable-v1-ready-for-owner-proof-2026-08-01.md) - zamrożony exact MS1 z dwoma authored Material IDs; owner proof z 2026-08-02 wykazał pusty wynik, a audyt znalazł niedozwolone użycie testowego `placeables.2da` i Appearance `3`.
- [evidence/material-separation-ms1-owner-empty-module-result-2026-08-02.json](evidence/material-separation-ms1-owner-empty-module-result-2026-08-02.json) - exact owner report, offline GIT/HAK readback, diagnoza baseline'u 2DA i warunek dopuszczenia minimalnego MS2.

- [audyt-edycji-tekstur-placeable-plan-implementacji-2026-08-01.md](audyt-edycji-tekstur-placeable-plan-implementacji-2026-08-01.md) - wdrożony offline vertical slice podmiany tekstury z zachowaniem UV i niezależnych override'ów per material slot; owner proof Aurora/NWN pozostaje otwarty.
- [evidence/tlc-ship-placeable-texture-copy-offline-preview-2026-08-01.md](evidence/tlc-ship-placeable-texture-copy-offline-preview-2026-08-01.md) - realny statek: byte-identical kopia GLB w Studio, teal texture override przez Worker/WASM/Core oraz porównanie Source/Edited bez tworzenia nowego proof candidate.
- [evidence/tlc-ship-placeable-aged-wood-texture-offline-preview-2026-08-01.md](evidence/tlc-ship-placeable-aged-wood-texture-offline-preview-2026-08-01.md) - docelowy warm aged-wood override na tym samym statku i UV, exact TGA resolver oraz porównanie Source/Edited bez nowego MOD/HAK.
- [audyt-material-separation-programy-graficzne-2026-08-01.md](audyt-material-separation-programy-graficzne-2026-08-01.md) - porównanie Material ID/face groups/texture sets w Blenderze, Maya, 3ds Max, Houdini, Substance i Unreal oraz rekomendacja component-first dla Meshy2Aurora.

- [M0 source-topology rigid hierarchy experiment — offline evidence (2026-07-22)](evidence/m0-source-topology-rigid-hierarchy-experiment-2026-07-22.md)

- [M0 r30 Retail runtime-conformance integration candidate — offline evidence (2026-07-21)](evidence/m0-r30-retail-runtime-conformance-iteration-2026-07-21.md)

- [M0 runtime conformance/state-projection fix — offline evidence (2026-07-21)](evidence/m0-runtime-conformance-state-projection-fix-2026-07-21.md)

> **HARD STOP — workspace:** jedynym repo projektu jest
> `C:\Projects\meshy2aurora`. Nigdy nie uzywaj
> `C:\Users\enonw\Documents\meshy2aurora`, nawet jako stagingu, scratcha lub
> miejsca tymczasowego. Szczegoly: [CANONICAL_WORKSPACE.md](CANONICAL_WORKSPACE.md).

Ten folder jest jedynym miejscem dokumentacji projektu `meshy2aurora`.

Przed uzyciem starszego dokumentu sprawdz jego klase w [status-dokumentacji-web-2026-07-10-codex.md](status-dokumentacji-web-2026-07-10-codex.md). D11-D14 i aktywne dokumenty webowe maja pierwszenstwo przed historycznymi rekomendacjami CLI/Node/aurora-web.

## Aktualny kierunek

Aktualny kierunek po audycie 2026-07-09:

- `meshy2aurora` jest projektem standalone: Meshy -> natywny content Aurora/NWN (`binary MDL`, polityka `MDX`, `2DA`, `HAK`).
- `C:\Projects\aurora-web` jest tylko read-only reference. Nie jest dependency, CLI, oracle, walidatorem ani proof base dla `meshy2aurora`.
- Twardy proof podstawowy ma isc przez NWN EE Toolset/gra oraz wlasny wygenerowany HAK/modul testowy.
- `c_kocrachn` jest technicznym proxy dla creature pipeline, nie assetem The Last City.
- Produkt jest aplikacja webowa local-first: UI w przegladarce, Rust 1.96.1 skompilowany do WebAssembly oraz pobieranie wygenerowanych HAK/raportow jako plikow. Studio bedzie osobnym etapem po proofie M6.

> **HISTORYCZNE R33-R39 (2026-07-27):** rozne hashe obecnego writera sa
> wyjasnionym skutkiem obowiazkowej naprawy base controllerow SkinMesh w r45,
> a nie regresja migracji. Nie aktualizuj starych hashy i nie tworz nowego
> `rNN`; szczegoly i test delty `+60` bajtow:
> [audyt bramek pre-push](audyt-bramek-pre-push-2026-07-27.md).

## Dokumenty

- [CANONICAL_WORKSPACE.md](CANONICAL_WORKSPACE.md) - bezwzgledny invariant jednej kanonicznej lokalizacji oraz zakaz uzywania niekanonicznego folderu `Documents`.
- [PROJECT_RULES.md](PROJECT_RULES.md) - zasady projektu i implementacji.
- [audyt-dekompilacji-aurora-item-model-parts-2026-07-29.md](audyt-dekompilacji-aurora-item-model-parts-2026-07-29.md) - ponowny audyt modeli itemow skladanych z partow: `ModelType`, Bottom/Middle/Top, rozdzielenie MDL od warstw ikon `i...`, miejsce transformacji i kontrakt pipeline.
- [evidence/p-ref-item-wswls-parts-2026-07-29.md](evidence/p-ref-item-wswls-parts-2026-07-29.md) - exact P-REF trzech retailowych MDL `WSwLs` odczytanych in-place, z hashami, own-reader summary i invariantami part placement.
- [evidence/shared-render-model-triangle-budget-300000-2026-07-29.md](evidence/shared-render-model-triangle-budget-300000-2026-07-29.md) - aktualna decyzja i implementacja jednego budzetu 300 000 triangles dla wszystkich render-modeli, z automatyczna bezstratna segmentacja przy granicy writera 21 845 per mesh.
- [evidence/shared-render-model-triangle-budget-20000-2026-07-27.md](evidence/shared-render-model-triangle-budget-20000-2026-07-27.md) - historyczna, zastapiona 2026-07-29 decyzja o budzecie 20 000; zachowana dla provenance dawnych lineage.
- [evidence/tlc-powrotnik-cdead-only-v2-ready-for-owner-proof-2026-07-28.md](evidence/tlc-powrotnik-cdead-only-v2-ready-for-owner-proof-2026-07-28.md) - V2 zakończone owner wynikiem `deathSequence=failed`; skorygowany pipeline zachowuje cały Meshy `Dead` jako jednorazowe `ckdbckdie` i tworzy nieruchome `cdead` z ostatniej pozy, ale nie zmaterializowano jeszcze kolejnego kandydata.
- [evidence/tlc-powrotnik-death-routing-v3-ready-for-owner-proof-2026-07-28.md](evidence/tlc-powrotnik-death-routing-v3-ready-for-owner-proof-2026-07-28.md) - exact V3 po korekcie routingu zgonu: cały Meshy `Dead` jest w `ckdbckdie`, `cdead` jest statyczną ostatnią pozą, a hash-verified MOD/HAK są zainstalowane do owner proofu.
- [evidence/tlc-powrotnik-death-family-v4-ready-for-owner-proof-2026-07-28.md](evidence/tlc-powrotnik-death-family-v4-ready-for-owner-proof-2026-07-28.md) - exact V4 z jednym pełnym Meshy `Dead` w `ckdbck` i ciągłymi terminalnymi holdami; właściciel potwierdził w NWN `modelVisibility=visible`, `deathSequence=passed` i `deathEntryJump=passed`.
- [evidence/tlc-veiled-humanoid-p100k-studio-pipeline-replay-2026-07-28.md](evidence/tlc-veiled-humanoid-p100k-studio-pipeline-replay-2026-07-28.md) - historyczna korekta pełnej ścieżki aplikacji dla jawnego P100K: Studio -> Worker -> WASM -> core, capture-time profil 20K oraz byte-identical replay MOD/HAK/MDL/TGA/2DA bez nowej iteracji.
- [evidence/tlc-veiled-humanoid-p100k-studio-video-2026-07-28.md](evidence/tlc-veiled-humanoid-p100k-studio-video-2026-07-28.md) - pełne nagranie WebM realnego workflow Studio P100K: wejścia, Inspect, Worker/WASM Build, binary readback, 42 animacje, metryki, artefakty i pobranie MOD-u.
- [evidence/tlc-stoneback-brute-p300k-v1-ready-for-owner-proof-2026-07-28.md](evidence/tlc-stoneback-brute-p300k-v1-ready-for-owner-proof-2026-07-28.md) - jawny eksperyment Creature P300K z Meshy, pełny przebieg Studio -> Worker -> WASM -> core, 290 318 zapisanych trójkątów w 14 SkinMesh, 42 animacje oraz hash-verified MOD/HAK zainstalowane do owner proofu.
- [texture-artifact-cleanup-creature-2026-07-28.md](texture-artifact-cleanup-creature-2026-07-28.md) - opcjonalny checkbox `Repair texture artifacts`, konserwatywna naprawa izolowanych jasnych/ciemnych pikseli i jednopikselowych dziur alfa w base color oraz raportowanie przejścia Studio -> Worker -> WASM -> core.
- [evidence/tlc-stoneback-brute-p300k-cleanup-v2-ready-for-owner-proof-2026-07-28.md](evidence/tlc-stoneback-brute-p300k-cleanup-v2-ready-for-owner-proof-2026-07-28.md) - cleanup V2 exact Stoneback P300K z odrębną tożsamością `m2p3f*`, 1 050 naprawionymi pikselami oraz hash-verified MOD/HAK zainstalowanymi do owner proofu.
- [evidence/tlc-stoneback-brute-p300k-cleanup-v3-ready-for-owner-proof-2026-07-28.md](evidence/tlc-stoneback-brute-p300k-cleanup-v3-ready-for-owner-proof-2026-07-28.md) - robust local-neighbor cleanup V3 exact Stoneback P300K z tożsamością `m2p3g*`, 3 682 lokalnymi naprawami oraz hash-verified MOD/HAK zainstalowanymi do owner proofu.
- [evidence/tlc-stoneback-brute-p300k-cleanup-v4-ready-for-owner-proof-2026-07-29.md](evidence/tlc-stoneback-brute-p300k-cleanup-v4-ready-for-owner-proof-2026-07-29.md) - edge-aware Hampel cleanup V4 exact Stoneback P300K z tożsamością `m2p3h*`, 4 588 lokalnymi naprawami, ochroną krawędzi oraz hash-verified MOD/HAK zainstalowanymi do owner proofu.
- [evidence/tlc-stoneback-brute-p300k-geometry-ab-v5-ready-for-owner-proof-2026-07-29.md](evidence/tlc-stoneback-brute-p300k-geometry-ab-v5-ready-for-owner-proof-2026-07-29.md) - potwierdzony przez właściciela V5: `visible/verified` w Aurora Toolset i NWN, 296 276/296 276 zachowanych trójkątów oraz `geometryArtifactResult=fixed`; dokładny lineage jest rozpoznawany w Studio po hashach MDL/TGA/HAK/MOD.
- [creature-pipeline-debt-paydown-2026-07-28.md](creature-pipeline-debt-paydown-2026-07-28.md) - zamknięcie długu creature po działającym V4: lineage i kontrakty V2, ciągłość przejść, root motion, kinematyczne eventy, humanoidalny Appearance, świeża tożsamość Studio oraz produkcyjny artifact bez MOD/UTC.
- [creature-detached-skin-accessory-stabilization-2026-07-29.md](creature-detached-skin-accessory-stabilization-2026-07-29.md) - systemowy audyt i stabilizacja odłączonych skinned accessories po przestrzennym weldzie, tryby Auto/Keep/Select bone oraz exact replay czterech kryształów `void-crystal-knight-h1-v1` bez zmiany 19 704 trójkątów.
- [evidence/void-crystal-knight-h1-v1-demo2-product-v3-ready-for-owner-proof-2026-07-29.md](evidence/void-crystal-knight-h1-v1-demo2-product-v3-ready-for-owner-proof-2026-07-29.md) - exact `vckdemo2.mod` z Product V3, pełnymi 19 704 trójkątami, czterema ustabilizowanymi kryształami i hash-verified instalacją MOD/HAK do testu właścicielskiego.
- [audyt-motion-retargeting-plan-implementacji-2026-08-01.md](audyt-motion-retargeting-plan-implementacji-2026-08-01.md) - audyt możliwości współdzielenia animacji Meshy między różnymi humanoidalnymi Creature: rozdzielenie same-rig merge, static-mesh donor i motion-only retargetingu, pełna lista braków P0-P2, plan implementacji oraz mierzalne kryteria ukończenia.
- [audyt-trzymania-broni-creature-plan-implementacji-2026-08-01.md](audyt-trzymania-broni-creature-plan-implementacji-2026-08-01.md) - audyt trzymania broni z korektą właścicielską V2: `MODELTYPE=L`, hooki `rhand`/`lhand`, prawdziwy bazowy UTI oraz rozdzielenie attachmentu, chwytu i weapon-aware animacji.
- [evidence/creature-weapon-anchor-v1-ready-for-owner-proof-2026-08-01.md](evidence/creature-weapon-anchor-v1-ready-for-owner-proof-2026-08-01.md) - immutable negatywny wynik V1: model widoczny, ale broń niewyposażona; zapis przyczyn i supersession przez V2.
- [evidence/creature-weapon-anchor-v2-ready-for-owner-proof-2026-08-02.md](evidence/creature-weapon-anchor-v2-ready-for-owner-proof-2026-08-02.md) - poprawiony exact handoff `m2aweapdemo2.mod`: oryginalne źródło i orientacja, `MODELTYPE=L`, `rhand/lhand`, stockowy `nw_wswss001`, readback GIT/UTC oraz hash-verified instalacja MOD/HAK.
- [evidence/creature-weapon-grip-v5-ready-for-owner-proof-2026-08-03.md](evidence/creature-weapon-grip-v5-ready-for-owner-proof-2026-08-03.md) - exact handoff `m2aweapdemo5.mod` z geometrycznie skalibrowanym środkiem dłoni V5, stockowym mieczem w prawym slocie, 42/42 hook coverage oraz hash-verified instalacją MOD/HAK.
- [evidence/creature-weapon-grip-v6-offline-remediation-2026-08-03.md](evidence/creature-weapon-grip-v6-offline-remediation-2026-08-03.md) - poprawka rollu chwytu: środek dłoni V5 połączony z pełnym natywnym basisem V4, jawny basis-proxy w Studio i zielone bramki core/WASM/web bez materializacji nowego kandydata.
- [evidence/creature-held-item-v8-owner-nwn-failure-2026-08-17.md](evidence/creature-held-item-v8-owner-nwn-failure-2026-08-17.md) - właścicielski wynik NWN V8: Creature widoczny, modułowy UTI niewidoczny i demonstrator oparty na podmiocie facing-matrix; zapis dopuszczający minimalną deltę V9.
- [evidence/creature-held-item-v9-ready-for-owner-proof-2026-08-17.md](evidence/creature-held-item-v9-ready-for-owner-proof-2026-08-17.md) - nowy demonstrator na owner-proved Void Crystal Knight ze stockowym `nw_wswss001`, prawym slotem 16, korektą roll `+90°` i hash-verified instalacją MOD/HAK.
- [evidence/creature-weapon-grip-v3-offline-remediation-2026-08-02.md](evidence/creature-weapon-grip-v3-offline-remediation-2026-08-02.md) - właścicielski negatywny wynik chwytu V2 i systemowa kalibracja V3 z bind pose: natywny offset, kompensacja orientacji, exact macierze Fogbound oraz zatrzymanie przed nowym MOD/HAK przez model-iteration gate.
- [evidence/tlc-fogbound-claw-guard-p300k-v1-ready-for-owner-proof-2026-07-31.md](evidence/tlc-fogbound-claw-guard-p300k-v1-ready-for-owner-proof-2026-07-31.md) - nowy humanoidalny Creature The Last City wygenerowany przez Meshy za 70 kredytow: 297 190 trojkatow, dziewiec routowanych klipow zrodlowych, pelne 42 stany NWN, ciagly pojedynczy zgon oraz hash-verified MOD/HAK z realnego Studio Worker/WASM.
- [audyt-bramek-pre-push-2026-07-27.md](audyt-bramek-pre-push-2026-07-27.md) - zamknieta diagnoza historycznego driftu r33-r39: naprawa r45, dokladna delta `+60` core, macierz hashy i zabezpieczenia przed naruszeniem model iteration gate.
- [audyt-creature-the-last-city-codex-2026-07-27.md](audyt-creature-the-last-city-codex-2026-07-27.md) - offline audyt wszystkich 116 UTC i 184 osadzen creature w exact `the_last_city — codex.mod`, z rozstrzygnieciem appearance/MDL, HAK, spawnów NSS/NCS, zawartosci testowej i czterech twardych bledow.
- [audyt-gotowosci-startowej-2026-07-10-codex.md](audyt-gotowosci-startowej-2026-07-10-codex.md) - kanoniczny gate przed implementacja: stan repo, toolchain, bootstrap, CI, M1A DoD i otwarte decyzje.
- [macierz-gotowosci-wiedzy-codex.md](macierz-gotowosci-wiedzy-codex.md) - centralny stan wiedzy dla calego pipeline; oddziela ustalony kierunek, otwarte evidence i runtime proof.
- [mdl-binary-crosswalk-codex.md](mdl-binary-crosswalk-codex.md) - wspolny layout binary MDL, zakres profilu A i jawny konflikt wariantow skin header.
- [audyt-wspolnego-pipeline-parserow-mdl-creature-placeable-tile-2026-07-25.md](audyt-wspolnego-pipeline-parserow-mdl-creature-placeable-tile-2026-07-25.md) - audyt dekompilacji potwierdzajacy jeden wspolny loader/parser MDL oraz osobne resolvery creature, placeable i tile.
- [audyt-wymagan-pipeline-tile-aurora-nwn-2026-07-26.md](audyt-wymagan-pipeline-tile-aurora-nwn-2026-07-26.md) - audyt wymagan custom tile na podstawie dekompilacji Aurory, lokalnego corpusu retail SET/MDL/WOK i stanu Meshy2Aurora; zawiera architekture, blokery oraz etapowa checkliste implementacji i owner proof.
- [audyt-placeable-widocznosc-aurora-nwn-plan-implementacji-2026-07-25.md](audyt-placeable-widocznosc-aurora-nwn-plan-implementacji-2026-07-25.md) - audyt widocznosci placeable w Toolset/NWN: kontrakty MDL/2DA/UTP/GIT/GIC/HAK/MOD oraz etapowa checklista implementacji i owner proof.
- [web-placeable-pipeline-runbook-2026-07-26.md](web-placeable-pipeline-runbook-2026-07-26.md) - ilustrowana instrukcja uruchomienia Studio, wyboru lane placeable, wejsc GLB/placeables.2da oraz etapow Inspect/Build/Review/Download.
- [implementacja-edytora-elementow-placeable-2026-07-26.md](implementacja-edytora-elementow-placeable-2026-07-26.md) - wykonawcza macierz kompletnego edytora elementow placeable: Outliner, gizmo, hierarchia, snapping, flagi render/collision/shadow, diagnostyka, wspolny core/WASM bake i zielone testy.
- [evidence/s1-placeable-ritual-pedestal-p1-ready-for-owner-proof-2026-07-25.md](evidence/s1-placeable-ritual-pedestal-p1-ready-for-owner-proof-2026-07-25.md) - zamrozony pierwszy lineage statycznego placeable: exact MOD/HAK/resrefy, hashe, readback, test ledger i human-owned proof handoff.
- [evidence/s1-placeable-ritual-pedestal-p1-owner-visual-result-2026-07-25.json](evidence/s1-placeable-ritual-pedestal-p1-owner-visual-result-2026-07-25.json) - owner-supplied Toolset/NWN verdict `visible + verified`, screenshot hashes oraz osobno zapisana rozbieznosc display name Area.
- [evidence/p20k-placeable-stress-v1-ready-for-owner-proof-2026-07-25.md](evidence/p20k-placeable-stress-v1-ready-for-owner-proof-2026-07-25.md) - niezalezny placeable stress-test: jeden mesh, dokladnie 20 000 trojkatow, granica writera 21 845, exact MOD/HAK i checklista owner proof.
- [evidence/placeable-shadow-adjacency-fix-2026-07-26.md](evidence/placeable-shadow-adjacency-fix-2026-07-26.md) - audyt przyczyny pasiastych shadow volumes oraz wspolna poprawka adjacency odporna na szwy UV/normalnych.
- [evidence/tlc-meshy-p20k-shadow-adjacency-v2-ready-for-owner-proof-2026-07-26.md](evidence/tlc-meshy-p20k-shadow-adjacency-v2-ready-for-owner-proof-2026-07-26.md) - zamrozony i zainstalowany TLC shadow-v2: trzy placeable po 20 000 trojkatow, exact MOD/HAK, pomiary adjacency i checklista owner proof.
- [evidence/p8-placeable-pwk-collision-implementation-2026-07-25.md](evidence/p8-placeable-pwk-collision-implementation-2026-07-25.md) - historia implementacji P8, negatywnych owner testow V1/V2 i poprawki ASCII PWK.
- [evidence/p8-s1-collision-v1-owner-runtime-result-2026-07-25.json](evidence/p8-s1-collision-v1-owner-runtime-result-2026-07-25.json) - owner runtime verdict `not_blocking` dla exact V1; adjacency zachowane jako historyczna hipoteza odrzucona przez V2.
- [evidence/p8-s1-collision-v2-adjacency-ready-for-owner-proof-2026-07-25.md](evidence/p8-s1-collision-v2-adjacency-ready-for-owner-proof-2026-07-25.md) - zamrozony V2 z adjacency, obecnie `not_blocking`.
- [evidence/p8-s1-collision-v2-owner-runtime-result-2026-07-25.json](evidence/p8-s1-collision-v2-owner-runtime-result-2026-07-25.json) - machine-readable owner verdict i zwiazanie exact V2.
- [evidence/p8-s1-collision-v2-aurora-decomp-audit-2026-07-25.md](evidence/p8-s1-collision-v2-aurora-decomp-audit-2026-07-25.md) - potwierdzona root cause z exact `nwmain`/`nwserver`: runtime parsuje PWK jako tekst, a V2 emituje binary MDL; pelny corpus `9 752` zewnetrznych PWK jest ASCII, wraz z planem ASCII PWK writera/readbacku.
- [evidence/p8-s1-collision-v3-ascii-pwk-ready-for-owner-proof-2026-07-25.md](evidence/p8-s1-collision-v3-ascii-pwk-ready-for-owner-proof-2026-07-25.md) - exact V3 po poprawce: ASCII PWK, hashe MOD/HAK/PWK, offline gates i checklista testu kolizji przez wlasciciela.
- [mdx-polityka-codex.md](mdx-polityka-codex.md) - aktywna polityka appended volatile/MDX dla profilu A.
- [animacje-kontrakt-profil-a-codex.md](animacje-kontrakt-profil-a-codex.md) - self-contained kierunek animacji oraz korekta faktow o `c_kocrachn`/`c_Horror`.
- [hak-2da-gff-crosswalk-codex.md](hak-2da-gff-crosswalk-codex.md) - kontrakt writerow HAK/ERF, 2DA i GFF oraz generated module proof.
- [m0-runtime-fixture-standard.md](m0-runtime-fixture-standard.md) - jeden generowany Area, tile, entry point i pozycja fixture'a dla kolejnych testów eksportu Meshy M0 w Aurora/NWN.
- [korpus-referencyjny-mdl-codex.md](korpus-referencyjny-mdl-codex.md) - polityka wielomodelowej regresji bez kopiowania retail/CEP payloadow do repo.
- [reguly-dokumentacji-cloud.md](reguly-dokumentacji-cloud.md) - aktualne reguly wymiany plikow Cloud/Codex.
- [audyt-dokumentacji-plan-2026-07-09-codex.md](audyt-dokumentacji-plan-2026-07-09-codex.md) - aktualna mapa rozjazdow, luk i plan naprawczy.
- [architektura-meshy2aurora-codex.md](architektura-meshy2aurora-codex.md) - architektura standalone `meshy2aurora`.
- [architektura-web-wasm-codex.md](architektura-web-wasm-codex.md) - architektura webowa: Rust/WASM, React, Three.js, lokalne pliki i granice opcjonalnego backendu.
- [engine-mdl-odpowiedz-codex.md](engine-mdl-odpowiedz-codex.md) - aktualny stan odpowiedzi o binary writerze, polityce MDX i bind pose.
- [status-dokumentacji-web-2026-07-10-codex.md](status-dokumentacji-web-2026-07-10-codex.md) - klasyfikacja wszystkich dokumentow po decyzji D12.
- [plan-implementacji-orkiestrator-codex.md](plan-implementacji-orkiestrator-codex.md) - aktywny plan etapow, Definition of Done i kontrakt dla orkiestratora.
- [studio-v1-debug-implementation-suplement-codex.md](studio-v1-debug-implementation-suplement-codex.md) - aktywny kontrakt Studio V1: uproszczona powloka, Debug Drawer, Binary Inspector, `Generate Report`, Replay i First Difference.
- [studio-v1-frontend-implementation-suplement-codex.md](studio-v1-frontend-implementation-suplement-codex.md) - aktywny kontrakt implementacji frontendu na bazie mockupow: state machine, komponenty, dane Workera, vertical slices i DoD.
- [meshy-api-local-bridge-implementation-suplement-codex.md](meshy-api-local-bridge-implementation-suplement-codex.md) - proponowany podzial implementacji opcjonalnego Meshy Local Bridge i Meshy Lab.
- [meshy-local-bridge-runbook.md](meshy-local-bridge-runbook.md) - uruchomienie, granice bezpieczenstwa i reczny gate realnego Meshy E2E.
- [MESHY_ASSET_LAYOUT.md](MESHY_ASSET_LAYOUT.md) - jedyny kanoniczny uklad lokalnych modeli Meshy w `sample-3d`, manifesty oraz gate zapobiegajacy kolejnemu rozjazdowi.
- [mockups/studio-v1-2026-07-14/README.md](mockups/studio-v1-2026-07-14/README.md) - wersjonowany pakiet ekranow Studio V1 z macierza widokow zaakceptowanych, wymagajacych poprawy i odrzuconych.
- [orchestrator-state.yaml](orchestrator-state.yaml) - maszynowy aktualny stan etapow, blockerow, problemow, bledow i evidence.
- [evidence/README.md](evidence/README.md) - append-only szablon dowodow dla jednego etapu.
- [prompt-dla-claude-prototyp-parsera.md](prompt-dla-claude-prototyp-parsera.md) - gotowy prompt dla Claude: prototype M1a parsera binary MDL.
- [m1a-kontrakt-suplement-codex.md](m1a-kontrakt-suplement-codex.md) - aktywne rozstrzygniecia schematu JSON, bledow, offsetow i roli orkiestratora dla M1A.
- [prototyp-parsera-m1a-claude.md](prototyp-parsera-m1a-claude.md) - wynik implementacji M1A, klasyfikacja pol, testy, ograniczenia i changed files.
- [m1b-kontrakt-suplement-codex.md](m1b-kontrakt-suplement-codex.md) - aktywny deep-reader layout, oba profile skin, stop conditions oraz sekwencja M1B-M1C-P-REF.
- [raport-m1b-deep-reader-codex.md](raport-m1b-deep-reader-codex.md) - zielony synthetic/native/WASM checkpoint M1B w statusie VERIFYING oraz dokladny pozostaly warunek canonical P-REF po M1C.
- [przyszle-featurey-studio-codex.md](przyszle-featurey-studio-codex.md) - backlog Studio po MVP: diagnostyka, materialy, geometria, rig, animacje i proof packet.
- [audyt-kursow-blender-meshy2aurora-2026-07-13-codex.md](audyt-kursow-blender-meshy2aurora-2026-07-13-codex.md) - mapa kursow Blender/Claude na future backlog; nie zmienia aktywnej roadmapy.
- [neverblender-audyt-2026-07-09-codex.md](neverblender-audyt-2026-07-09-codex.md) - audyt NeverBlender jako narzedzia pomocniczego/debug dla modeli NWN/Aurora.
- [repozytoria-pomocnicze-2026-07-09-codex.md](repozytoria-pomocnicze-2026-07-09-codex.md) - mapa repozytoriow drugiej linii po Aurora First.
- [audyt-repozytoriow-pomocniczych-2026-07-10-codex.md](audyt-repozytoriow-pomocniczych-2026-07-10-codex.md) - lokalny audyt 27 repozytoriow z `C:\Projects\Claude`, z priorytetami, kluczowymi plikami i mapowaniem na pipeline.
- [wymagania-startowe-cloud.md](wymagania-startowe-cloud.md) - historyczne wymagania startowe sprzed decyzji D7; uzywac tylko jako kontekst.
- [CLOUD_SUPPLEMENT_FORMAT.md](CLOUD_SUPPLEMENT_FORMAT.md) - format odpowiedzi i suplementow dla plikow `*-cloud.md`.
- [aurora-models-animations-audit-2026-07-08.md](aurora-models-animations-audit-2026-07-08.md) - audyt modeli, animacji i systemow powiazanych z Aurora/NWN.
- [aurora-pipeline-odpowiedz-codex.md](aurora-pipeline-odpowiedz-codex.md) - odpowiedzi Codexa dla starego tematu pipeline `aurora-web`; reference-only.
- [aurora-animacje-odpowiedz-codex.md](aurora-animacje-odpowiedz-codex.md) - odpowiedzi Codexa dla tematu animacji.
- [meshy-input-odpowiedz-codex.md](meshy-input-odpowiedz-codex.md) - odpowiedzi Codexa dla tematu wejscia Meshy.
- [pliki-referencyjne-odpowiedz-codex.md](pliki-referencyjne-odpowiedz-codex.md) - odpowiedzi Codexa dla tematu plikow referencyjnych; sciezki `aurora-web` sa reference-only.
- [srodowisko-zakres-odpowiedz-codex.md](srodowisko-zakres-odpowiedz-codex.md) - odpowiedzi Codexa dla tematu srodowiska i zakresu.

## Zasada dopisywania

Nowe notatki, audyty, decyzje techniczne, wyniki proofow, listy zrodel i runbooki dopisujemy tutaj. Jezeli w innym repo istnieje dokument potrzebny temu projektowi, w `documentation` powinna powstac notatka indeksujaca albo przeniesiony/odswiezony dokument, z jasnym wskazaniem zrodla.

## Konwencja `*-cloud.md`

Pliki w formacie `[nazwa]-cloud.md` oznaczaja suplement cloud do dokumentu bazowego. Gdy taki plik istnieje albo zostanie wskazany, trzeba dopisac uzupelnienie dla pracy w chmurze, a nie traktowac go jako osobny zamiennik glownej dokumentacji.

Pelny uklad takiego suplementu opisuje [CLOUD_SUPPLEMENT_FORMAT.md](CLOUD_SUPPLEMENT_FORMAT.md).

## Konwencja Cloud/Codex

Aktualna wymiana z Cloud:

- `[temat]-cloud.md` - dokument Cloud.
- `[temat]-pytania-cloud.md` - pytania Cloud z sekcjami `Q1`, `Q2`, ...
- `[temat]-odpowiedz-codex.md` - odpowiedzi Codexa, z odpowiedzia pod kazdym `Q`.
