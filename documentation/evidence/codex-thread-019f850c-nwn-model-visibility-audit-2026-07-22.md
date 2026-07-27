# Audyt wątku 019f850c — dlaczego nie mamy jeszcze widocznego modelu w NWN

Data audytu: 2026-07-22  
Wątek: `019f850c-3cf3-7cb1-8853-cbdb4cfa8b05` (`Orkiestrator`)  
Zakres: odczyt historii wątku, artefaktów `r31`/`r32`, profili proof, testów kontenera oraz stanu ścieżki Toolset → NWN. Audyt nie uruchamiał Toolsetu ani NWN i nie zmieniał natywnych katalogów użytkownika.

## Werdykt

Nie można obecnie uczciwie stwierdzić, że poprawiony model `r32` nadal jest niewidoczny w NWN. Dla `r32` istnieje świeży dowód `Toolset: visible/verified`, ale nie istnieje runtime PNG ani werdykt NWN. Poprawna klasyfikacja to:

- Toolset `r32`: `modelVisibility=visible`, `proofCompleteness=verified`;
- NWN `r32`: `modelVisibility=not_tested`, `proofCompleteness=missing`.

Najważniejsza odpowiedź na pytanie „czemu jeszcze go nie widzimy” brzmi więc: wątek nie zachował obrazu z jedynej uruchomionej sesji NWN `r32`, zamknął dokładny proces, a ponowny przebieg zatrzymał się przed READY na pozostawionym `modules\temp0`. To jest błąd ciągłości orkiestracji i toru proof, nie kolejny udowodniony negatyw modelu.

Poprzedni `r31` rzeczywiście był niewidoczny w NWN na świeżym, związanym i zaakceptowanym obrazie. Ten wynik pozostaje `not_visible/verified`, ale późniejszy audyt wykazał, że jego MOD zawierał zbyt ubogi rekord creature. Niezależny parser odrzuca go na `area.git.Creature List[0].MaxHitPoints`; brakuje też pełnego `SkillList`. `r31` nie był zatem czystym testem samego MDL.

## Stan obu lineage

### R31 — świeży negatyw, lecz z wadliwym kontenerem creature

Zaakceptowany proof `r31` wiąże:

- MOD SHA-256 `8575465ef683256a54c101a3f4445a4e22803194c9b899a0e43e3fd069fdeafd`;
- HAK SHA-256 `ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12`;
- MDL SHA-256 `fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6`;
- Area `m2a_m0a31`, fixture 4,5 m przed graczem;
- Toolset `visible/verified`;
- NWN `not_visible/verified`.

Obraz NWN pokazuje gracza i czytelny, niezasłonięty teren przed nim bez oczekiwanego fixture. Ten negatyw przyjął koordynator 120 s. Artefakty:

- `proof-output/m0-r31-hierarchy-only-20260722/live/model-proof-120s-run-6/runtime/capture.png`;
- `proof-output/m0-r31-hierarchy-only-20260722/live/model-proof-120s-run-6/runtime/verdict.json`;
- `proof-output/m0-r31-hierarchy-only-20260722/live/model-proof-120s-run-6/final-model-proof.accepted-at-4104003757313.json`.

Po tym wyniku znaleziono konkretny defekt MOD-u: historyczny `r31` jest odrzucany przez niezależny parser pełnego GIT/UTC na `MaxHitPoints`. To jest potwierdzona przyczyna wadliwości kontenera i prawidłowa podstawa do utworzenia `r32` bez zmiany HAK/MDL/TGA/2DA.

### R32 — kontener naprawiony, Toolset zaliczony, NWN nieprzetestowany

`r32` zmienia tylko MOD/Area i pełny envelope creature. Reużywa byte-identical `m2a_m0r31.hak` oraz ten sam MDL/TGA/appearance row. Potwierdzone:

- MOD `m2a_m0r32.mod` SHA-256 `a952e05746710c8461e74b38dbf93e6cc5c86314771330d68724579c11693573`;
- pełny GIT/UTC: `MaxHitPoints=13`, 28 wpisów `SkillList`;
- Toolset capture SHA-256 `77f1d3fa3324cfd8e3c2c14275e57bec6f15090a7a35d5626750673ad8f1a41d`;
- Toolset verdict po 28,258 ms: `visible/verified`;
- uruchomiony jeden `nwmain`, PID `36000`, exact Area `m2a_m0a32`.

Runtime zatrzymał się na:

`nwmain_zorder_lift_not_unobscured:50140,50140,20856`

Przyczyną infrastrukturalną było rozciągnięcie okna do pełnego `Screen.Bounds` i sondowanie obszaru pod paskiem zadań zamiast `WorkingArea`/client area. Katalog `runtime` nie zawiera `capture.png`, `capture.json`, `verdict.json` ani `packet.json`. Blocker poprawnie zachował:

```text
toolset = visible
runtime = not_tested
```

Artefakty:

- `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-1/toolset/capture.png`;
- `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-1/toolset/verdict.json`;
- `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-1/blocker.json`.

## Co wątek zrobił źle

### 1. Utracił jedyną możliwość rozstrzygnięcia `r32`

Po błędzie centralnego capture dokładny proces NWN nadal działał. Wątek najpierw twierdził, że screenshoty obu programów już istnieją, a następnie zakończył `nwmain:36000` i `nwtoolset:52904`. W finalnej odpowiedzi przyznał, że zwykły screenshot klienta nie został wykonany przed pojawieniem się modala Quit i zamknięciem procesu.

Trwałe artefakty rozstrzygają tę sprzeczność: dla `r32` nie istnieje runtime PNG. Nie wolno traktować wcześniejszej wypowiedzi „screenshoty już są” jako dowodu.

### 2. Zakończył turę mimo aktywnego, bezpiecznego następnego kroku

Finalna odpowiedź wątku mówi tylko, że obrazu nie da się odzyskać. Cel „model widoczny w NWN” oraz bezpieczny następny krok — wznowienie tego samego `r32` — pozostały aktywne. Jest to naruszenie audit exit gate / task continuity. Brak systemowego `cancelled` nie zmienia wyniku.

### 3. Przesunął uwagę z dowodu na narzędzia przed zachowaniem obserwacji

Wątek rozpoczął zmianę centralnego capture, trzech skilli i polityki cleanup, chociaż powinien najpierw zachować judgeable obraz istniejącego, exact-bound procesu NWN. Poprawka `Screen.WorkingArea` + client-area capture jest technicznie uzasadniona i ma testy offline, ale nie odzyskuje utraconego werdyktu `r32`.

### 4. Drugi przebieg nie doszedł do READY

Po zamknięciu procesów `prepare` zatrzymał się fail-closed:

- `blockerCode=toolset_temp0_requires_clean_start`;
- `modules\temp0\module.ifo` istnieje;
- `canUseVerifiedTemp0Cache=false`;
- Toolset/NWN `0/0`;
- zegar 120 s nie wystartował.

Nie istnieje obecnie publiczna, identity-bound adopcja tego proof-owned cache. Stary emergency cleanup nie jest właściwą trasą, bo może mutować INI. To jest obecny pre-start blocker tego samego `r32`, a nie dowód braku modelu.

### 5. Dokumentacja i kontrakt lineage są rozjechane z późniejszym stanem

`documentation/evidence/m0-r32-corrected-container-iteration-2026-07-22.md` i lineage contract deklarują, że runtime profile nie został utworzony. Obecnie istnieją:

- `proof-profiles/m0-r32-corrected-container-runtime-v1.json`;
- `proof-profiles/m0-r32-corrected-container-model-proof-120s-v1.json`;
- `proof-profiles/m0-r32-corrected-container-model-proof-120s-route-fix-v1.json`.

W rezultacie `node tools/m0-r32-corrected-container-lineage.contract.test.mjs` failuje na `existsSync(runtimePath)`: oczekiwano `false`, otrzymano `true`. Kontrakt i durable record nie opisują już faktycznego lineage/proof state.

### 6. Powstał drugi profil proof bez negatywu `r32`

Po `runtime=not_tested/missing` utworzono `...model-proof-120s-route-fix-v1.json` z nowym `outputRoot=...run-2`. Lokalne reguły zaliczają nowy proof profile do nowej iteracji. Nie ma trwałego uzasadnienia, które wyłączałoby route-hash-only zmianę spod tej bramki. Co najmniej jest to nierozstrzygnięte naruszenie model-iteration gate; nie powinno być powtarzane bez jawnej korekty kontraktu.

## Co nadal może być nie tak z modelem

Poniższe punkty są potwierdzonymi różnicami strukturalnymi, ale jeszcze nie potwierdzonymi przyczynami niewidoczności `r32`:

- `supermodelName="NULL"`;
- brak donor-compatible szkieletu/hierarchii;
- tylko 7 syntetycznych animacji (`cappear`, `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `cdead`), po 1 s, bez kontrolerów i bez payloadu mesh/skin;
- appearance row 15100 ma poprawne 35 kolumn, `RACE=m2a_m0p01` i `MODELTYPE=S`, ale nie jest pełną kopią wiersza dawcy; `STRING_REF`, `NAME` i większość parametrów ruchu/kolizji to `****`.

Najmocniejszą hipotezą modelową jest brak funkcjonalnej rodziny animacji/supermodelu zgodnego z geometrią creature. Pełny donor row jest także brakującym kontraktem. Nie wolno jednak ogłosić tych punktów root cause ani tworzyć `r33`, dopóki ten sam poprawiony `r32` nie otrzyma świeżego, judgeable wyniku NWN `not_visible`.

## Suplement: checklista creature i wyjaśnienie supermodelu

Przeanalizowano dodatkowo pliki użytkownika:

- `C:\Users\enonw\Downloads\checklista_model_creature_nwn_ee.txt`;
- `C:\Users\enonw\Downloads\supermodel_nwn_ee_wyjasnienie.txt`.

Dokumenty prawidłowo wskazują dwa dozwolone warianty creature: zgodny supermodel/dawca albo kompletne animacje lokalne. Nie są jednak same w sobie dowodem, że `supermodel=NULL` powoduje brak renderu. Dekompilacja Aurory pokazuje, że parser `setsupermodel` (`FUN_00a5e058`) rozwiązuje nazwę przez `FUN_00a5de94`, a wartość null daje pusty wskaźnik zamiast twardego błędu ładowania. `NULL` jest legalnym wariantem self-contained; wtedy wszystkie wymagane animacje muszą istnieć w bieżącym MDL.

### Dokładny stan `r32`

`r32` reużywa MDL z `r31`, którego readback potwierdza:

- `supermodelName="NULL"`;
- `animationScale=1.0`;
- wejściowy GLB: `sourceSkinCount=0`, `sourceAnimationCount=0`;
- baza: 4 węzły i 1 rigid mesh;
- 7 lokalnych animacji: `cappear`, `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `cdead`;
- dla każdej animacji: 4 węzły, 0 eventów, 0 controller keys, puste geometry arrays.

To nie jest funkcjonalny zestaw animacji. Są to placeholdery nazw i topologii, a nie ruch, który klient NWN może zastosować do modelu.

Dla porównania lokalny referencyjny `c_squirrel.mdl` także deklaruje `setsupermodel c_squirrel NULL`, ale zawiera:

- 30 węzłów, w tym 2 skin nodes;
- 43 lokalne animacje;
- 43 bloki `positionkey`;
- 912 bloków `orientationkey`.

To porównanie obala prostą tezę „NULL jest błędem” i potwierdza właściwą: `NULL` wymaga kompletnej lokalnej rodziny animacji, której `r32` nie ma.

### Doprecyzowanie z `model_bez_supermodelu_nwn_ee.txt`

Trzeci dokument użytkownika potwierdza, że supermodel jest mechanizmem dziedziczenia, a nie obowiązkowym warunkiem załadowania geometrii. Wnosi jednak ważne ograniczenie dla diagnozy `r32`: model bez supermodelu i bez funkcjonalnych animacji może nadal zostać wyrenderowany w pozie bazowej albo przesuwać się bez ruchu kończyn. Brak animacji jest zatem potwierdzonym defektem profilu creature, lecz sam nie dowodzi przyczyny całkowitego braku modelu na ekranie NWN.

Zalecany w dokumencie szybki test przez wyszukanie `newanim` jest dla `r32` niewystarczający. Model zawiera siedem deklaracji animacji, więc prosty test tekstowy dałby wynik pozytywny, ale ich readback pokazuje `0` controller keys, `0` eventów i puste tablice geometrii. W audycie trzeba rozróżniać:

- obecność nazw i sekcji animacji;
- obecność funkcjonalnych kluczy/kontrolerów ruchu;
- wizualne wykonanie animacji w NWN.

Pierwszy punkt jest spełniony, drugi nie, a trzeci pozostaje nieprzetestowany. Ten dokument wzmacnia więc wymaganie naprawy animacji dla docelowego creature, ale jednocześnie obniża wiarygodność hipotezy, że to właśnie brak animacji powoduje brak draw w `r32`.

### Ocena `analiza_modelu_m2a_m0r31_nwn_ee.txt`

Raport prawidłowo potwierdza istniejące fakty: łańcuch MOD → HAK → `appearance.2da` → MDL → TGA jest spójny; źródło nie ma rigu, skinu, jointów ani wag; MDL zawiera pojedynczy rigid `TriMesh`; lokalne animacje są pustymi deklaracjami; wiersz 15100 jest syntaktycznie szeroki, lecz semantycznie skąpy; materiały PBR są opcjonalne dla pierwszej widoczności. Nie dostarcza jednak nowego dowodu runtime i nie może zmienić stanu `r32` z `not_tested/missing`.

Raport błędnie podnosi bajt nagłówka modelu `0x71=0` do rangi niezgodności. Stara dokumentacja Torlack/xoreos opisuje to nieznane pole jako inicjalizowane do `1`, ale bezpośrednie odczyty bieżących zasobów dają:

- exact `r31` `m2a_m0p01`: `[0x70,0x71]=[0,0]`;
- retailowy NWN:EE `c_horror`, SHA-256 `2faf553a0665da200b232bd52d03c0e1d79b88959cabdbe840f35f16e5878c8e`: `[0,0]`;
- runtime-visible, wygenerowany przez projekt H1 v20, SHA-256 `6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd`: `[0,0]`;
- binarny referencyjny `c_Direwolf`, SHA-256 `121b63cd51ff46c3633c740951752110bebdeab7db139719ff6ff7db077f0fd9`: `[0,0]`.

Własny reader słusznie klasyfikuje oba bajty jako `runtimeUnknownBytes`, a nie „model offset”. `0x71=0` nie jest więc kandydatem na przyczynę niewidoczności i nie wolno tworzyć iteracji zmieniającej ten bajt na `1`.

Zalecenie, aby zastąpić własny binary writer eksportem NeverBlender/nwnmdlcomp, jest niezgodne z kontraktem produktu. Meshy2Aurora ma emitować natywny binary MDL własnym writerem; ASCII i zewnętrzne kompilatory mogą służyć wyłącznie jako read-only materiał porównawczy. Dodatkowo H1 v20 dowodzi, że MDL wygenerowany przez ten sam projektowy writer może zostać narysowany przez NWN. Nie eliminuje to defektu konkretnego profilu M0, ale eliminuje tezę, że sam fakt użycia własnego writera jest przyczyną.

Pełne sklonowanie wiersza dawcy `appearance.2da` pozostaje uzasadnioną hipotezą diagnostyczną, nie potwierdzonym root cause. Wiersze 90 `Shield_Guardian`, 92 `Golem_Stone` i 102 `Hook_Horror` są pełnymi lokalnymi referencjami. Wybór dawcy dla pól fizycznych 2DA nie dowodzi jednak zgodności jego szkieletu lub supermodelu z geometrią Meshy. Ewentualna zmiana 2DA może zostać rozważona jako izolowana delta dopiero po świeżym `r32` `not_visible/verified`; obecnie bramka iteracji nadal jej zabrania.

### Brak na poziomie implementacji projektu

Obecny writer hardkoduje `NULL` w `write_binary_mdl.rs`, a semantic readback traktuje `NULL` jako wymagany default profilu. Jednocześnie `m7_corpus.rs` jawnie klasyfikuje reference-supermodel route jako `NonHumanoidReferenceSupermodelDeferredM7V5` z komunikatem, że kanoniczna ścieżka supermodelu nie została zaimplementowana.

Projekt ma częściowy writer skin/controllerów, lecz gotowość pozostaje `EVIDENCE_PARTIAL`: wizualna deformacja skinu w NWN i mapowanie stanów gameplay nie mają finalnego proofu. Dla tego konkretnego źródła nie ma dodatkowo żadnego skinu ani animacji do przeniesienia.

### Brak w `appearance.2da`

Generator `r31/r32` nie kopiuje wiersza dawcy. Ustawia tylko 9 pól:

- `LABEL`;
- `MOVERATE`;
- `MODELTYPE`;
- `RACE`;
- `PORTRAIT`;
- `ENVMAP`;
- `BLOODCOLR`;
- `WEAPONSCALE`;
- `SIZECATEGORY`.

Pozostałe pola są `****`. Wiersz ma poprawną szerokość 35 kolumn, więc nie jest przesunięty, ale nie zachowuje kontraktu ruchu, kolizji, wysokości, dystansów, dźwięku i head tracking wybranego dawcy. `STRING_REF` i `NAME` są głównie metadanymi/nazwami i nie są samodzielnym wyjaśnieniem braku draw; ważniejszy jest brak świadomie wybranego dawcy oraz pełnych wartości runtime.

### Uporządkowana lista braków

1. Brak jawnie wybranego modelu-dawcy odpowiedniego do anatomii i zachowania źródła Meshy.
2. Brak donor-compatible hierarchii węzłów i szkieletu.
3. Brak skinu i wag w źródłowym GLB (`0` skinów).
4. Brak funkcjonalnych animacji lokalnych (`0` kontrolerów w 7 placeholderach).
5. Brak zaimplementowanej kanonicznej ścieżki reference-supermodel w produkcie.
6. Brak pełnej kopii wiersza `appearance.2da` dawcy i potwierdzenia, że `MODELTYPE=S` jest właściwy dla wybranej rodziny.
7. Brak świeżego runtime PNG `r32`, więc wpływ punktów 1–6 na samą widoczność nadal jest nieprzetestowany.

Jeżeli asset ma pozostać nieruchomą figurą/dekoracją, semantycznie pasuje bardziej do ścieżki placeable niż do kompletnego creature. Jeżeli ma być creature, pipeline musi dostarczyć jedną z dwóch pełnych dróg: rigged Meshy source z lokalnymi animacjami albo donor-compatible rig + działający supermodel. Samo wpisanie np. `a_ba` bez przebudowy hierarchii nie jest poprawką.

## Co jest już wykluczone lub ma niższy priorytet

- Nazwa pliku MDL, internal root, `RACE` i klucz HAK wskazują `m2a_m0p01`.
- MDL i TGA istnieją w jedynym attached HAK-u i mają związane hashe.
- Physical row 15100 ma 35 kolumn i zgadza się z `Appearance_Type` w UTC/GIT.
- MOD deklaruje dokładnie jeden HAK; nie znaleziono konkurencyjnego `appearance.2da` w tym lineage.
- Exact obiekt jest widoczny w zwalidowanym `TScrollBox`, więc podstawowa rezolucja HAK/2DA/MDL działa w Toolsecie.
- Błąd paska zadań i `temp0` są błędami proof lane; nie dowodzą, że model jest niewidoczny.

## Wymagany plan naprawczy

1. Nie tworzyć `r33`, nowego HAK-a, resrefu, appearance row ani kolejnego proof profile.
2. Dokończyć publiczną, read-only i hash-bound adopcję dokładnego proof-owned `temp0`, albo zatrzymać lane z konkretnym blockerem. Bez ręcznego kasowania i bez mutacji INI.
3. Wznowić dokładnie `m2a_m0r32.mod` + `m2a_m0r31.hak` przez publiczne `prepare → preflight → ready → toolset → runtime`.
4. Zachować pierwszy fresh, identity-bound runtime PNG i werdykt przed jakimkolwiek cleanupem. Dopiero potem zakończyć exact PID+StartTime.
5. Jeśli `r32` będzie widoczny: zamknąć problem samej widoczności, a osobno testować animacje/zachowanie.
6. Jeśli `r32` będzie `not_visible/verified`: zapisać negatyw, diagnozę i minimalną deltę. Pierwszy kandydat do delty to donor-compatible supermodel/szkielet z funkcjonalnymi animacjami oraz pełny donor appearance row, nie kolejna zmiana UTC/GIT.
7. Zaktualizować lineage contract i durable record `r32`, aby profile i rzeczywisty proof state nie przeczyły testowi.

## Weryfikacja wykonana w audycie

- Obejrzano accepted runtime PNG `r31`: model nieobecny w czytelnym polu przed graczem.
- Obejrzano `TScrollBox` `r32`: exact fixture ma niepustą geometrię i `visible/verified`.
- `cargo test -p m2a-core --test m0_r32_corrected_container exact_r31_sparse_container_is_rejected_and_r32_corrected_container_is_runtime_complete -- --ignored --exact` — PASS, 1/1.
- `node tools/m0-r32-corrected-container-lineage.contract.test.mjs` — FAIL: `true !== false`, ponieważ runtime profile istnieje wbrew kontraktowi.
- Stan końcowy procesu podczas audytu: brak `nwtoolset.exe` i `nwmain.exe`.

## Gotowy prompt korygujący dla wątku

> Wznów exact `r32` od ostatniego zweryfikowanego stanu. Nie twórz `r33`, nowego HAK-a, resrefu ani proof profile. Najpierw użyj lub dokończ publiczną read-only, hash-bound adopcję proof-owned `temp0`; potem wykonaj jedną trasę `prepare → preflight → ready → toolset → runtime`. Zachowaj fresh identity-bound runtime PNG i werdykt przed cleanupem. Raportuj `r32` jako NWN `not_tested/missing` aż do obrazu; dopiero judgeable `not_visible/verified` może dopuścić donor-based iterację modelu. Zaktualizuj także r32 lineage contract, który obecnie failuje po utworzeniu runtime profile.

## Suplement: pakiet `neverblender_diagnostyka_m2a_m0p01`

Przeanalizowano bez uruchamiania Blendera, Toolsetu ani NWN:

- `neverblender_diagnostyka_m2a_m0p01_README.txt`, SHA-256
  `4bd4895a85ecf487672315d2be03bab4f1c9becdbfe3b636747224fd8e258875`;
- `analiza_efektu_eksportu_neverblender.txt`, SHA-256
  `191df2dd6285b82dc22795c0664b45b6daab4efd7df8134c28f0d15cf8a18799`;
- `analiza_modelu_m2a_m0r31_nwn_ee.txt`, SHA-256
  `ee2a4a34548901453e44465adf0278a1e95e961ad40603e70c336e9a288b9c2d`;
- `m2a_m0p01_neverblender_style_ascii.mdl`, SHA-256
  `48e7b639ead7b47db9f6b4223c4f9df12b68dcdab9d700a1ed3febbfe8210acc`;
- `neverblender_diagnostyka_m2a_m0p01.zip`, SHA-256
  `e67e4a1bbed084b7d2c6ea3a797c0e84267b1ea13ba40b8a2a6f4cf9753e20c5`.

### Co faktycznie zawierają pliki TXT

README uczciwie stwierdza, że ASCII MDL nie został wyeksportowany przez UI
Blendera. Został ręcznie zrekonstruowany na podstawie oczekiwanej struktury
NeverBlendera i geometrii odczytanej z binarnego MDL. Raport
`analiza_efektu_eksportu_neverblender.txt` również mówi, że Blender i `nwmain`
nie zostały uruchomione. Pakiet nie jest więc wynikiem eksportu NeverBlendera
ani runtime testu; jest syntetyczną propozycją diagnostyczną.

TXT prawidłowo rozpoznają, że wejściowy asset ma zero skinów, rigu, jointów,
wag i animacji, a NeverBlender nie stworzyłby ich automatycznie z samego
statycznego mesha. Prawidłowa jest też hipoteza, że rigid `TriMesh` może być
narysowany w pozie bazowej. Nie wynika z tego jednak, że brak skinu lub
animacji sam wyjaśnia całkowity brak draw.

Wniosek raportu „jeżeli ASCII działa, należy zastąpić własny writer
NeverBlenderem/nwnmdlcomp” nie jest dopuszczalnym wnioskiem produktowym.
Kontrakt Meshy2Aurora wymaga własnego native binary MDL writera, a wcześniejszy
runtime-visible H1 v20 potwierdza, że model z własnego writera może być
narysowany przez NWN. Ewentualny wynik ASCII byłby wskazówką do porównania
konkretnego profilu serializacji, nie zgodą na zmianę docelowej architektury.

### Analiza syntetycznego ASCII MDL

Plik jest wewnętrznie spójny na poziomie podstawowej geometrii:

- 4 węzły: 3 `dummy` i 1 `trimesh`;
- 2380 pozycji, 2380 UV, 2380 normalnych i 1569 trójkątów;
- indeksy mieszczą się w zakresie `0..2379`;
- bounds około `[-0.54365,-0.31951,0]..[0.54365,0.31951,1.892962]`;
- tekstura `m2a_m0t01`.

Po znormalizowaniu indeksów do `u16` ich SHA-256 jest identyczny z exact
binary MDL:

`e80a9bb222a2cc57f191a95579ac512aeddb0e5666daebd31fc76e23259da1f2`.

Nie jest to jednak semantycznie jednoosiowa kopia binarnego modelu:

- pozycje, UV i normalne są zapisane jako zaokrąglony tekst i ich payload
  `f32` nie jest byte-identical z oryginałem;
- binary MDL zawiera 2380 vertex colors, których ASCII nie zawiera;
- binary MDL ma 7 pustych placeholderów animacji, ASCII ma 0 `newanim`;
- ASCII pomija część jawnych pól runtime mesha, między innymi `render`,
  `shadow`, `beaming`, `transparency`, `renderHint`, `tileFade` i `meshType`,
  polegając na defaultach parsera;
- zmienia cały format serializacji binary na ASCII.

Wariant „ASCII model only” nie izoluje zatem jednego parametru. Jednocześnie
zmienia format, precyzję, vertex colors, tablicę animacji i część jawnych pól
mesha. Może służyć jako read-only debug/golden reference, ale według
kanonicznego kontraktu projektu nie jest ścieżką runtime/proofu.

### Zawartość ZIP i warianty HAK

ZIP ma pięć wpisów: README, ASCII MDL oraz trzy HAK-i. Archiwa HAK mają
poprawne, czytelne tablice ERF i po trzy zasoby: `appearance.2da`,
`m2a_m0p01.mdl` i `m2a_m0t01.tga`.

| Wariant | SHA-256 HAK | `appearance.2da` | MDL | TGA |
|---|---|---|---|---|
| 01 ASCII only | `16b69eaf8c6df5246ad8cbd36a96118a8d1b655c46537617300ab1a77db6bfd6` | exact oryginał r31 | syntetyczny ASCII | exact oryginał |
| 02 appearance only | `a6dc31398df72a403bc7b1bd2240abd733921d5951cd5c97c11e2ce89fff6742` | zmieniony row 15100 | exact binary r31 | exact oryginał |
| 03 oba | `3e0c44640f83a083097ef5cd92f7fbf5ff4a391ba26eb9355664b0e062b85520` | zmieniony row 15100 | syntetyczny ASCII | exact oryginał |

W wariantach 02/03 prefiks 2DA przed row 15100 jest byte-identical z
oryginałem. Zmieniony row ma poprawne 35 kolumn. W porównaniu z lokalnym
wierszem 92 `Golem_Stone` identyczne są 29 z 35 pól. Różnią się:

| Pole | Golem Stone | Wariant diagnostyczny |
|---|---|---|
| `LABEL` | `Golem_Stone` | `M2A_M0_MESHY_RIGID` |
| `STRING_REF` | `2071` | `****` |
| `NAME` | `Golem_Stone` | `M2A_M0_MESHY_RIGID` |
| `RACE` | `c_GolStone` | `m2a_m0p01` |
| `BLOODCOLR` | `W` | `R` |
| `WEAPONSCALE` | `****` | `1.0` |

Jest to więc donor-completed row zachowujący część wartości własnego profilu,
nie pełna kopia Stone Golema 1:1. Jako przyszła hipoteza wariant 02 jest
znacznie lepiej izolowany niż wariant 01 lub 03: zmienia tylko
`appearance.2da`, zachowując exact binary MDL i TGA. Nadal nie jest jednak
aktualnie dopuszczoną iteracją.

### Dlaczego tych HAK-ów nie wolno teraz instalować

Wszystkie trzy warianty nazywają się `m2a_m0r31.hak`, ale mają inne hashe niż
exact HAK r31/r32:

`ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12`.

Instrukcja skopiowania ich do katalogu użytkownika i nadpisania istniejącego
HAK-a łamie immutable lineage, zasadę absent-target/hash verification oraz
model-iteration gate. Taki podmieniony plik miałby tę samą nazwę, lecz inną
treść, więc dotychczasowe MOD-y, capture i werdykty przestałyby mieć
jednoznaczną tożsamość.

Pakiet odwołuje się do historycznego r31. Jego NWN `not_visible/verified`
został zachowany, ale później potwierdzono wadliwy envelope creature w MOD-zie.
Aktualnym kandydatem jest poprawiony r32, który zachowuje exact HAK/MDL/2DA z
r31. Dla r32 Toolset ma `visible/verified`, natomiast NWN nadal ma
`modelVisibility=not_tested` i `proofCompleteness=missing`. Pakiet nie zawiera
nowego modułu z poprawionym envelope, manifestu lineage, hash-bound profilu,
capture ani runtime verdictu.

### Decyzja audytu

Pakiet wnosi dwie użyteczne rzeczy:

1. syntetyczny, strukturalnie spójny ASCII dump do porównań offline;
2. konkretną hipotezę uzupełnienia row 15100 parametrami Stone Golema.

Nie wnosi rzeczywistego eksportu NeverBlendera, skompilowanego przez niego
binary MDL ani dowodu widoczności w NWN. Nie wolno instalować żadnego z trzech
HAK-ów i nie wolno podmieniać obecnego `m2a_m0r31.hak`.

Stan r32 pozostaje bez zmian: NWN `not_tested/missing`. Najpierw trzeba
dokończyć exact r32. Dopiero świeży, candidate-bound
`not_visible/verified` dopuści jedną nową minimalną deltę. Najczystszą pierwszą
hipotezą z tego pakietu byłaby wtedy 2DA-only donor-completed row, wykonany
własnym writerem w świeżym, niekolidującym lineage. Wariant mieszany 03 nie
powinien być pierwszym testem, bo uniemożliwia przypisanie skutku do modelu albo
2DA.

## Suplement: ocena pogłębionego researchu creature

Przeanalizowano:

`C:\Users\enonw\Downloads\raport_nwn_ee_creature_models_poglebiony_research.txt`

- długość: 62 319 bajtów;
- SHA-256:
  `7ee25f3e044f2d1b19e29d133409af0678212ac071dd16a21f1af3c51467c63c`;
- raport jawnie nie wykonuje runtime testu badanego modelu;
- źródła obejmują bieżące nwn.wiki, kod NeverBlendera, nwnmdlcomp, xoreos,
  kopie modeli w repozytoriach społeczności oraz raporty forum.

### Tezy dobrze potwierdzone

Research prawidłowo wzmacnia następujące ustalenia:

1. `SkinMesh` nie jest ogólnym warunkiem narysowania geometrii. Jest
   rozszerzeniem mesha o deformację przez bone map, inverse bind data i wagi.
2. Creature może używać rigid części parentowanych do animowanych node'ów.
3. `MODELTYPE=S` oznacza pojedynczy MDL używający rodziny animacji `c*`.
4. `setsupermodel ... NULL` jest legalnym wariantem `CHARACTER`; nie jest
   samodzielnym błędem ładowania.
5. Nie znaleziono wymogu, aby `cpause1`, `cappear` albo `cwalk` były obecne
   przed wykonaniem bazowego draw.
6. `LABEL`, `STRING_REF` i `NAME` nie wybierają geometrii runtime. `RACE`,
   `MODELTYPE` oraz integralność wiersza są znacznie ważniejsze dla resource
   resolution.
7. Pola ruchu, kolizji i dźwięku nie są udokumentowanymi przełącznikami
   `render=0`. Sparse row jest słabym kontraktem, ale nie jest potwierdzoną
   przyczyną no-draw.
8. Klient NWN:EE potrafi czytać ASCII MDL i dynamicznie tworzyć z niego
   strukturę runtime. Komendy `compileloadedmodels`,
   `compileloadedasciimodels` i `compilemodel` są rzeczywistą funkcjonalnością
   EE.
9. NeverBlender nie tworzy automatycznie anatomii, armature, SkinMesh, wag ani
   animacji z arbitralnego statycznego GLB/OBJ.
10. Ręcznie napisany „NeverBlender-style ASCII” nie jest wynikiem ani testem
    eksportera NeverBlender.
11. `nwnmdlcomp` definiuje bajty model header `0x70/0x71` jako dwuelementowe
    flags i inicjalizuje oba na zero. Razem z lokalnymi retail/owned odczytami
    usuwa to `0x71=0` z listy racjonalnych przyczyn niewidoczności.
12. Wariant ASCII z pakietu zmienia wiele osi, wariant 2DA-only jest warunkowo
    izolowany, a wariant mieszany nie rozstrzyga przyczyny.

Bieżące nwn.wiki potwierdza, że `S` jest simple/single-model profile, że
`LABEL/STRING_REF/NAME` są metadanymi oraz że zalecaną praktyką dla nowego
appearance jest rozpoczęcie od podobnego pełnego wiersza:

- `https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da`;
- `https://nwn.wiki/spaces/NWN1/pages/38175669/MDL`;
- `https://nwn.wiki/spaces/NWN1/pages/38175598/Console%2BCommands`.

### Tezy, które raport formułuje zbyt mocno

1. Dokładny przypadek „jeden rigid TriMesh, `NULL`, zero efektywnych animacji,
   `MODELTYPE=S`, bieżący NWN:EE” nadal nie ma w raporcie candidate-bound
   reprodukcji. Sekcja A1 prawidłowo nazywa go bardzo prawdopodobnym, lecz
   końcowa odpowiedź zmienia to na kategoryczne „TAK”. Właściwy status to
   silny wniosek, nie zamknięty fakt runtime.
2. `c_a_chicken`, `c_a_bat` i `c_flybookc` pochodzą z repozytoriów osób
   trzecich. Są użytecznymi przykładami strukturalnymi, ale same linki nie
   dowodzą byte-identical retail provenance. Lokalny odczyt `c_squirrel` jest
   silniejszym dowodem legalności `NULL`.
3. `c_relic` jest przywołany jako model `NULL` bez lokalnych animacji, ale nie
   otrzymał osobnej pozycji źródłowej ani runtime reprodukcji.
4. Zasłanianie animacji supermodelu przez pusty lokalny `newanim` wynika z
   lookupu xoreos. Raport uczciwie oznacza je jako prawdopodobne; nie wolno
   podnosić go do faktu o zamkniętym kliencie Beamdog bez testu.
5. Nie ma dowodu, że Toolset i runtime używają różnych parserów MDL. Raport
   poprawnie pozostawia tę tezę nierozstrzygniętą.
6. Ranking przyczyn (`UTC`, resource stack, writer, 2DA...) jest ogólną
   heurystyką. Nie jest diagnozą exact r32, ponieważ raport nie ma jego
   runtime PNG ani packetu.

### Rozjazd z kontraktem Meshy2Aurora

Research proponuje produktowy pipeline:

`NeverBlender ASCII -> kompilator klienta NWN:EE -> binary`.

Może on być zewnętrznym punktem porównawczym, ale nie może zastąpić produktu.
Meshy2Aurora wymaga własnego readera, IR, native binary MDL writera, 2DA writera
i HAK writera. ASCII jest wyłącznie debug dump/golden snapshot, a
NeverBlender, nwnmdlcomp i kompilator gry są reference-only. H1 v20 już
potwierdza, że binary MDL własnego writera może zostać narysowany przez NWN;
pozostaje możliwy defekt konkretnego profilu M0, nie całego writera.

Proponowana baza donor profiles także wymaga ograniczenia provenance. Do
produktu wolno przenieść niezależnie potwierdzone reguły kompatybilności i
nazwy kontraktowe. Nie wolno kopiować retailowych szkieletów, rest-transform
payloadów, wag ani animacji. Odwołanie do istniejącego supermodel resrefu i
read-only analiza lokalnego zasobu nie oznaczają zgody na dystrybucję jego
payloadu.

Sekwencja `TEST 0..8` nie jest obecnie wykonalnym planem projektu. Tworzyłaby
nowe moduły, HAK-i, rows i modele przed rozstrzygnięciem r32 oraz proponuje
operowanie na `override/development`. Naruszałoby to immutable lineage,
model-iteration gate i granice konfiguracji użytkownika.

### Decyzja po researchu

Raport obniża priorytet trzech hipotez:

- brak SkinMesh jako samodzielny powód no-draw;
- brak animacji jako samodzielny powód no-draw;
- sparse fizyczne pola 2DA jako samodzielny powód no-draw.

Nie ustala jednak root cause badanego M0. Stan exact r32 pozostaje:

- Toolset: `modelVisibility=visible`,
  `proofCompleteness=verified`;
- NWN: `modelVisibility=not_tested`,
  `proofCompleteness=missing`.

Jedynym aktualnym krokiem jest dokończenie tego samego exact r32. Dopiero jego
świeży `not_visible/verified` może dopuścić jedną minimalną deltę. Z researchu
nie wynika zgoda na uruchomienie wariantów ASCII, instalację diagnostycznych
HAK-ów, nowy moduł ani zastąpienie własnego binary writera.
