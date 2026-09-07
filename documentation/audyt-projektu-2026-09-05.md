**Audyt meshy2aurora — 5 września 2026**

Bieżący stan wymaga napraw przed wydaniem. Najważniejsze problemy to możliwość podwójnego uruchomienia zadań Meshy, nieskuteczne anulowanie między etapami, utrata edycji materiałów w Studio oraz eksport, który pomija część zaakceptowanych ustawień. Poniżej znajduje się dziewięć ustaleń funkcjonalnych i dotyczących builda oraz osobny wynik bramek jakości.

Audyt obejmuje bieżące drzewo `C:\Projects\meshy2aurora`, z plikami zmienionymi i nieśledzonymi. HEAD przy rozpoczęciu: `7289f2b0c8d385b2058912fb4fa6c0eb7e1990ba` z 2026-08-02; `git status --porcelain` zwracał 410 pozycji. Nie jest to audyt samego commita. Podczas pracy zmieniał się katalog źródła `tlc-bronze-mask-tripo-20260905-v3`, więc wyniki jego kontroli manifestu są obserwacjami z danego momentu.

Zakres: Rust core i publiczna granica WASM, Studio React/Worker, lokalny bridge, testy, CI oraz Dockerfile. Nie zmieniano kodu produktu. Reprodukcje bridge używały wyłącznie lokalnego HTTP i wstrzykniętej atrapy `meshFetch`; nie wykonywano płatnych wywołań. Nie wykonywano wizualnego proofu w Aurora Toolset ani NWN. Ustalenia o wyjściowym modelu są wnioskami z kodu i matematyki, nie nowym wynikiem działania silnika.

P1 oznacza problem do naprawy przed wydaniem dotkniętej funkcji; P2 — konkretny błąd wymagający zaplanowanej poprawki. „Dowód statyczny” oznacza prześledzenie implementacji, bez uruchomienia danego scenariusza w UI.

**1. [P1] Jedno potwierdzenie może utworzyć dwa przebiegi Meshy**

Źródło: [index.mjs:835](C:/Projects/meshy2aurora/tools/meshy-local-bridge/index.mjs:835), zakres 835–842; analogiczny wzorzec w 738–743 i 767–772.

`usedNonces.has()` wykonuje się przed `await` sprawdzającym saldo, a `usedNonces.add()` dopiero po nim. Dwa żądania z tym samym `confirmationNonce` mogą więc równocześnie przejść kontrolę i otrzymać różne identyfikatory przebiegu.

Reprodukcja offline: uruchomiono `createLocalBridge` z atrapą Meshy, wysłano dwa równoległe `POST /v1/runs` z jednym nonce i zatrzymano obie odpowiedzi `/balance` barierą Promise. Po ich zwolnieniu otrzymano HTTP `[200, 200]`, dwa różne `runId`, oba kończące jako `READY`. Atrapa odnotowała cztery POST-y etapów: `[preview, preview, refine, refine]`. To potwierdza możliwość podwójnego zlecenia operacji, bez potrzeby wywoływania rzeczywistego Meshy.

Naprawa: atomowo zarezerwować nonce przed pierwszym oczekiwaniem asynchronicznym; przechowywać wynik lub Promise przypisany do potwierdzenia. Zdefiniować ponawianie po błędzie i zastosować tę samą zasadę w image-runs oraz retexture. Test akceptacyjny powinien uruchamiać równoległe żądania z kontrolowaną barierą.

**2. [P1] Anulowany przebieg może uruchomić kolejny etap Meshy**

Źródło: [index.mjs:309](C:/Projects/meshy2aurora/tools/meshy-local-bridge/index.mjs:309), zakres 309–313; przejście do refine w 418–420.

`waitForTask` sprawdza `CANCELED` przed oczekiwaniem na odpowiedź. Anulowanie podczas tego oczekiwania nie zostaje ponownie sprawdzone po odpowiedzi `SUCCEEDED`. Następny etap ustawia `REFINING` i wysyła kolejny POST, nadpisując anulowanie.

Reprodukcja offline: zatrzymano GET zadania preview, wykonano `/cancel` i otrzymano `CANCELED`. Następnie zwolniono GET z wynikiem `SUCCEEDED`. Atrapa zarejestrowała dodatkowy POST refine, a przebieg zakończył się jako `READY`. Problem dotyczy uruchomienia kolejnej operacji po anulowaniu; nie jest jedynie brakiem możliwości cofnięcia już wysłanego zadania.

Naprawa: użyć monotonicznej flagi lub tokenu anulowania, weryfikowanego po każdym `await` i bezpośrednio przed rozpoczęciem następnego etapu. Anulowanie powinno pozostać stanem końcowym lokalnego przebiegu.

**3. [P1] Powrót z Build do Inspect resetuje edytor materiałów i tekstur**

Źródło: [MaterialSeparationEditor.tsx:76](C:/Projects/meshy2aurora/apps/studio-web/src/features/material-separation/MaterialSeparationEditor.tsx:76), zakres 76–94; [App.tsx:2230](C:/Projects/meshy2aurora/apps/studio-web/src/App.tsx:2230).

Scenariusz: utworzyć autorski materiał, przypisać teksturę override, zastosować zmiany, przejść do Build i wrócić do Inspect. Zmiana kroku odmontowuje edytor. Nowe montowanie inicjuje reducer z pierwotnego `bootstrap.document`, mimo że App przechowuje zaakceptowany dokument w osobnym stanie. `textureDocument` powstaje z domyślnego `resolution.textureAuthoring`, a `textureFiles` z pustej mapy. Efekt przekazuje ten reset do App i nadpisuje zachowany snapshot tekstur. Ponowne Apply może również zastąpić zaakceptowane przypisania dokumentem początkowym.

Dowód statyczny: osobne gałęzie Inspect/Build w App 2209 i 2291; props edytora 2230–2235; zapis Apply 2254–2259; reset stanu 76–94 i obsługa snapshotu w App 594–602.

Naprawa: przekazywać do edytora bieżący dokument i snapshot plików z App albo utrzymywać ich stan w jednym trwałym właścicielu. Testować pełną sekwencję Inspect → Build → Inspect, także po nieudanym Build.

**4. [P1] Dwie trasy Creature pomijają zaakceptowane materiały**

Źródło: [App.tsx:1810](C:/Projects/meshy2aurora/apps/studio-web/src/App.tsx:1810), zakres 1810–1818; druga gałąź w 1624–1639.

Studio udostępnia Material Separation dla `PRODUCT_300K` i przygotowuje dokument oraz payload tekstur. Request `M0_STATIC_RIGID` wysyła jednak tylko GLB, appearance i nazwę trasy. Request `REFERENCE_SUPERMODEL_CREATURE` także pomija dokumenty i payload materiałów.

Scenariusz: statyczny GLB bez skin/animacji albo Creature z zastosowanym supermodelem; edytować materiały i tekstury, Apply, Build. Wyeksportowany wynik powstaje bez tych zmian. W M0 pomijany jest również wybrany `sourceForward`.

Dowód statyczny całej ścieżki: App 1458–1471 przygotowuje dane, lecz wskazane requesty ich nie wysyłają. Worker 1786–1790 wywołuje dwuargumentowe API M0; core [model_pipeline.rs:2332](C:/Projects/meshy2aurora/crates/m2a-core/src/model_pipeline.rs:2332) dekoduje oryginalny obraz GLB. Wywołanie reference w Worker 1348–1377 także nie przyjmuje autorskich materiałów.

Naprawa: doprowadzić te wejścia przez protokół Worker, WASM i core do obu tras; do tego czasu capability UI powinno jawnie blokować niedostępną edycję. Testować zmienione wyjściowe bajty i semantykę materiału, a nie tylko obecność kontrolki.

**5. [P1] Profil EE dopuszcza UV1–UV3, których import i eksport nie zachowują**

Źródło: [aurora_material.rs:370](C:/Projects/meshy2aurora/crates/m2a-core/src/aurora_material.rs:370), [glb/mod.rs:933](C:/Projects/meshy2aurora/crates/m2a-core/src/glb/mod.rs:933), [placeable.rs:2627](C:/Projects/meshy2aurora/crates/m2a-core/src/placeable.rs:2627), [model_pipeline.rs:4071](C:/Projects/meshy2aurora/crates/m2a-core/src/model_pipeline.rs:4071).

Kompilator `NWN_EE_MTR` akceptuje zestawy UV 0–3. Import odczytuje jednak tylko `TEXCOORD_0`, a wskazane produktowe ścieżki wypełniają `uv1`, `uv2`, `uv3` pustymi tablicami.

Scenariusz: poprawny GLB z odmiennymi UV0 i UV1, w którym normal mapa ma `normalTexture.texCoord=1`. Plan materiału uznaje kanał za zachowany, lecz wyjściowy model nie zawiera wymaganych współrzędnych UV1. Powstaje rozbieżność między raportem zachowania danych a faktycznym payloadem. Dowód statyczny; nie wykonywano wizualnego porównania w NWN.

Naprawa: przenieść wymagane strumienie przez importer, IR, segmentację i writer oraz zweryfikować ich readback. Alternatywnie obecne trasy muszą odrzucać materiał wymagający niezachowywanego zestawu UV.

**6. [P2] Generator tangentów gubi znak orientacji lustrzanych UV**

Źródło: [aurora_material_package.rs:718](C:/Projects/meshy2aurora/crates/m2a-core/src/aurora_material_package.rs:718), zakres 718–722; [model_material_uv_projection.rs:361](C:/Projects/meshy2aurora/crates/m2a-core/src/model_material_uv_projection.rs:361), zakres 361–363.

Obie implementacje zapisują czwarty składnik tangenta jako `+1.0`. Nie obliczają znaku orientacji z bitangenta. Writer zachowuje otrzymaną wartość w rozszerzeniu materiałowym.

Deterministyczny kontrprzykład matematyczny: trójkąt `P=[(0,0,0),(1,0,0),(0,1,0)]`, normalna `+Z`, `UV=[(0,0),(1,0),(0,-1)]`, bez źródłowych tangentów. Pochodne dają `T=+X`, `B=-Y`, więc znak wynosi `-1`. Generator zwraca `T=+X, w=+1`, co rekonstruuje przeciwny bitangent. Normal mapa na takim UV ma odwrócony kierunek jednej osi. Nie jest to wynik nowego proofu silnika.

Naprawa: wyznaczać tangent i bitangent, ortogonalizować względem normalnej i obliczać znak `dot(cross(N,T),B)`. Regresja powinna obejmować zwykłą i lustrzaną wyspę oraz przeciwne strony projekcji box.

**7. [P2] Różne konfiguracje supermodelu dostają te same nazwy zasobów**

Źródło: [App.tsx:1618](C:/Projects/meshy2aurora/apps/studio-web/src/App.tsx:1618), zakres 1618–1622; seed identyfikatora w 280–300.

Identyfikator produktu nie zawiera wybranego supermodelu, hashy jego łańcucha, dokumentu rig authoring ani opcji branch repair. Gałąź reference dodaje pola do zwykłego identity Creature, pozostawiając model/texture/HAK resrefy bez zmiany. Te pominięte parametry są jednocześnie wejściami konwersji.

Scenariusz: zachować GLB, appearance i inne opcje, a następnie wyeksportować dwie poprawne konfiguracje szkieletu. Zmiana bajtów modelu nie zmieni jego resrefu ani nazwy HAK. Wyniki kolidują przy współistnieniu w jednej przestrzeni zasobów; instalacja przestrzegająca zakazu nadpisania różnych hashy zatrzyma się. Dowód statyczny, bez instalowania takiego zestawu.

Naprawa: hashować pełną recepturę reference, w tym exact chain, rig authoring, opcje wpływające na wynik i wersję generatora. Test powinien sprawdzać zarówno stabilność identycznego przepisu, jak i zmianę nazw po zmianie szkieletu.

**8. [P1] Domyślne testy CI wymagają plików nieobecnych w Git**

Źródło: [h2_r42_visibility_candidate.rs:24](C:/Projects/meshy2aurora/crates/m2a-core/tests/h2_r42_visibility_candidate.rs:24), odczyty plików w 35–38; [ci.yml:24](C:/Projects/meshy2aurora/.github/workflows/ci.yml:24).

Zwykły test `exact_h2_r42_uses_the_owned_runtime_positive_type0_root_rigid_family` nie ma `#[ignore]` ani warunku środowiskowego. Bezwarunkowo czyta lokalne `sample-3d/h2-clockwork-sentinel-1500/source.glb` i `local-reference-assets/appearance.2da`. Test jest śledzony, oba wejścia są ignorowane przez Git, a CI wykonuje `cargo test --workspace` po checkoutcie.

Dowód: `git ls-files` dla testu oraz `git check-ignore` dla obu wejść. Test przeszedł na tym komputerze, gdzie pliki są dostępne; nie uruchamiano osobnego czystego checkoutu. Z odczytu bezwarunkowego i nieobecności wejść w Git wynika błąd braku pliku w takim checkoutcie. Podobne testy historycznych kandydatów wymagają przeglądu.

Naprawa: oddzielić testy z lokalnym korpusem od domyślnego CI, z jawnym wymaganiem odpowiednich wejść. Podstawowe kontrakty powinny mieć własne syntetyczne fixture. Nie kopiować lokalnych danych referencyjnych do repo tylko po to, aby zazielenić test.

**9. [P1] Dockerfile pomija obowiązkowy crate workspace**

Źródło: [Dockerfile:50](C:/Projects/meshy2aurora/Dockerfile:50), zakres 50–55; analogiczny zestaw COPY w 29–31; [Cargo.toml:2](C:/Projects/meshy2aurora/Cargo.toml:2).

Workspace zawiera `m2a-win32-noreplace`, a core ma do niego zależność ścieżkową. Obrazy kopiują tylko core i WASM. Target `quality` nie załaduje kompletnego workspace przy pierwszej operacji Cargo. Sam build `studio-dev` może dojść do końca, ponieważ wykonuje na tym etapie npm; uruchomienie jego `predev/build:wasm` bez bind mountu także napotka brak crate. Pełny bind mount w Compose może maskować ten drugi przypadek.

Dowód statyczny manifestów i COPY; nie wykonywano `docker build`. Naprawa: kopiować pełen zestaw wymaganych członków i plików wejściowych, a następnie zweryfikować `quality` oraz uruchomienie dev zgodnie z deklarowanym trybem.

**Weryfikacja i pozostałe ograniczenia**

| Kontrola | Wynik |
| --- | --- |
| Canonical workspace guard | PASS |
| `npm run typecheck` | PASS |
| `npm test -- --reporter=dot` | 65 plików, 351/351 PASS |
| Bridge: `bridge.test.mjs`, `merge-animation-glbs.test.mjs`, `run-store.test.mjs` | 28/28 PASS |
| Dwie dodatkowe reprodukcje race bridge | Oba defekty odtworzone; wyłącznie atrapa Meshy |
| `cargo fmt --all --check` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | FAIL, exit 101; kompilator podsumował 54 błędy dla lib i 40 dla lib test, częściowo wspólne |
| `cargo test --workspace` | FAIL: core lib 216 PASS, 2 FAIL, 3 ignored; polecenie zatrzymało dalsze testy |
| `cargo test --workspace --no-fail-fast` | FAIL, exit 101: 843 PASS, 3 FAIL, 47 ignored; zakończono wszystkie targety |
| `npm run build` — świeży web/WASM, TypeScript, Vite | PASS, exit 0; ostrzeżenie Vite o rozmiarze chunka |
| `node tests/prepare-worker-fixture.mjs` oraz `node node_modules/vitest/vitest.mjs run --config vitest.worker-integration.config.ts` | PASS, exit 0: 3 pliki, 15 PASS, 2 skipped; prawdziwy Worker i świeży web/WASM |
| Asset layout guard | Końcowy PASS, exit 0; dwa wcześniejsze odczyty FAIL podczas równoległej pracy nad źródłem v3 |

Dwie porażki core lib są niezaktualizowanymi oczekiwaniami tekstowego provenance w `reference_supermodel_surface_anatomy.rs:657` i `:687`. Implementacja zwraca odpowiednio `project_to_semantically_compatible_authoritative_surface` i `virtual_welded_authoritative_low_surface`, a stare testy oczekują `project_to_primary_surface` i `authoritative_component_low_surface`. Poprzedzające je asercje geometrii przeszły. Te wyniki blokują zieloną bramkę testów, ale nie dowodzą regresji geometrii.

Test WASM `ascii_supermodel_completes_analysis_preview_and_product_offline` także zawodzi: `lib.rs:2646` otrzymuje `M2A-REFERENCE-SUPERMODEL-REFERENCE-RENDER-ENVELOPE-MISSING`. Wynik odtworzono osobno przez uruchomienie istniejącego pliku testowego EXE z `--exact` i `--nocapture`, bez kolejnej kompilacji. Próbka ASCII w `lib.rs:2546` zawiera tylko trzy węzły dummy, a aktualny kontrakt w `reference_supermodel_generic.rs:2057` wymaga geometrii referencji. To niezgodność fixture z wymaganiami trasy. Obecny test nie potwierdza całego przebiegu ASCII; nie stanowi też dowodu awarii samego parsera.

Pierwszy przebieg kontroli asset layout zastał `tlc-bronze-mask-tripo-20260905-v3` bez manifestu. Drugi zastał już manifest, ale zgłosił nieopisany `source-palette-atlas-v2.glb` oraz brak jego SHA-256 i rozmiaru. Pliki zmieniały się w trakcie audytu, więc nie zaliczono tej obserwacji do dziewięciu trwałych defektów. Końcowe powtórzenie zakończyło się `meshy-asset-layout-ok`, exit 0; audyt nie modyfikował tych źródeł ani manifestów.

Logi uruchomień audytowych znajdują się w `.codex-tmp/audit-20260905-*.log`; ich kluczowe wyniki są zapisane w tym dokumencie. Testy oznaczone `ignored` lub warunkowane lokalnym korpusem nie stanowią dowodu testowania rzeczywistych modeli. Wynik unit testów Studio nie obejmuje wizualnej zgodności z Toolsetem/NWN.

**Kolejność napraw**

1. Atomowe potwierdzenia i monotoniczne anulowanie w bridge, z testami kontrolującymi kolejność odpowiedzi.
2. Zachowanie stanu edytora oraz pełne przekazywanie materiałów do każdej udostępnionej trasy konwersji.
3. Zgodność raportu materiałów z rzeczywistym eksportem UV i poprawna orientacja tangentów.
4. Pełna tożsamość zasobów reference supermodel.
5. Odtwarzalny build i testy na czystym checkoutcie, poprawka Dockerfile oraz zamknięcie czerwonych bramek Rust. Po poprawkach modeli osobno pozostaje właścicielska weryfikacja wizualna.
