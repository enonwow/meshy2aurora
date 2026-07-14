# Studio V1 i tryb debug - suplement implementacyjny

Data: 2026-07-14
Autor: Codex
Status: `AKTYWNY KONTRAKT ROBOCZY`
Zakres: Studio V1, obserwowalnosc pipeline i paczka diagnostyczna

## 1. Cel suplementu

Ten dokument zapisuje decyzje wlasciciela dotyczace pierwszej widocznej wersji aplikacji oraz szczegolowego trybu debug. Jest uzupelnieniem, a nie zamiennikiem:

- [suplement-implementation-first-m7-s1-codex.md](suplement-implementation-first-m7-s1-codex.md),
- [plan-implementacji-orkiestrator-codex.md](plan-implementacji-orkiestrator-codex.md),
- [przyszle-featurey-studio-codex.md](przyszle-featurey-studio-codex.md).

Suplement doprecyzowuje S1 i nie otwiera automatycznie pelnego S2 ani backlogu F1-F10. Zaawansowane funkcje autorskie pozostaja odlozone, ale diagnostyka potrzebna do implementacji i zglaszania bledow wchodzi do V1.

Zewnetrzny mockup jest kierunkiem wizualnym, nie kontraktem kompletnego zakresu. Trwaly kontrakt zakresu znajduje sie w tym dokumencie.

## 2. Pytania, na ktore ma odpowiadac V1

Uzytkownik ma bez zgadywania zobaczyc:

1. co zostalo wczytane,
2. jak wyglada wynik konwersji,
3. co jest poprawne, ostrzegawcze albo zablokowane,
4. jakie artefakty mozna pobrac,
5. jakie dane diagnostyczne mozna przekazac razem ze zgloszeniem bledu.

## 3. Docelowa powloka Studio V1

Glowne Studio ma byc zwarte i oparte na jednym przeplywie:

`Source -> Inspect -> Build -> Review Output -> Download`

### 3.1. Elementy wymagane

| Obszar | Kontrakt V1 |
|---|---|
| Wejscie | Lokalny plik GLB oraz bazowy `appearance.2da` |
| Viewport | Dwa tryby uzytkowe: `Source Model` i `Converted Model` |
| Wynik | `Converted Model` jest renderowany z kanonicznego readbacku wygenerowanej binarki, a nie tylko ze stanu posredniego IR |
| Review | Osobny krok pokazuje Model Details, dodany wiersz `appearance.2da` i Package Contents przed pobraniem |
| Podsumowanie | Zwarte statusy geometrii, tekstur, rigu/animacji i paczki |
| Walidacja | Krotka lista bledow i ostrzezen z mozliwoscia przejscia do szczegolu |
| Budowanie | Jednoznaczna akcja `Build Package` |
| Pobieranie | Wygenerowany binary MDL, HAK i raport diagnostyczny |
| Granica | `OPEN_M6` pozostaje jawnie widocznym stanem, a nie ukrytym sukcesem |
| Debug | Domyslnie zwiniety `Debug Drawer`, nie konkurujacy z glownym przeplywem |

Osobny podglad `Aurora IR` nie jest wymagany w glownym UI V1. Moze pozniej wejsc jako widok developerski, jezeli readback nie wystarczy do lokalizacji bledu.

### 3.2. Uproszczony uklad

```text
+--------------------------------------------------------------+
| meshy2aurora   Source > Inspect > Build > Review > Download  |
+----------------------+---------------------------------------+
| Inputs               | Source Model | Converted Model         |
| - model.glb          |                                       |
| - appearance.2da     |             VIEWPORT                  |
|                      |                                       |
+----------------------+---------------------------------------+
| Conversion readiness | Validation                            |
| Geometry  PASS       | 1 warning: ...        [show detail]   |
| Texture   WARN       |                                       |
| Rig        PASS      |                                       |
| Package Assembly PASS| Runtime Proof OPEN_M6                 |
+--------------------------------------------------------------+
| Context action: Continue / Build / Review / Download          |
+--------------------------------------------------------------+
| Debug Drawer (collapsed)                                     |
+--------------------------------------------------------------+
```

### 3.3. Elementy odlozone po V1

W glownym Studio V1 nie umieszczamy:

- dziewiecioetapowej szyny procesu,
- wyboru wielu projektow,
- wyboru platformy lub profilu docelowego,
- edycji transformacji, materialow, szkieletu i animacji,
- edytora UV oraz galerii miniaturek materialow,
- zaawansowanego authoringu,
- ekranu runtime proof NWN,
- M7 Corpus jako glownej funkcji UI,
- edycji binarki albo zapisu zmian z Binary Inspectora.

Te elementy moga pozostac w [przyszle-featurey-studio-codex.md](przyszle-featurey-studio-codex.md). Nie powinny blokowac zbudowania widocznego, uzytecznego V1.

## 4. Jedno zrodlo danych diagnostycznych

UI i eksport raportu musza korzystac z tego samego modelu `DebugSnapshot`. Nie wolno utrzymywac osobnej, ubozszej prawdy dla UI i osobnej dla raportu.

Minimalne grupy danych `DebugSnapshot`:

```text
DebugSnapshot
|- session
|- sourceIdentity
|- stageLedger[]
|- pipelineEvents[]
|- ingestIr
|- normalizedIr
|- conversionReport
|- animationMapping
|- writerLayout
|- readback
|- validationReport
|- packageManifest
|- dataLineage[]
|- dependencyGraph
|- invariants[]
|- workerTrace
|- artifacts[]
`- runtimeDiagnostics
```

Snapshot jest tylko do odczytu. Zbieranie albo ogladanie diagnostyki nie moze zmieniac wyniku konwersji.

## 5. `Generate Report`

`Generate Report` generuje jedna przenosna paczke wszystkich dostepnych danych diagnostycznych.

### 5.1. Zachowanie

- Akcja jest dostepna po sukcesie i po bledzie.
- Przy bledzie paczka zawiera wszystkie zakonczone etapy oraz jawna liste elementow, ktore nie powstaly.
- W UI paczka jest tworzona jako Blob/ZIP i pobierana bez proszenia o dostep do systemu plikow.
- Tryb developerski lub testowy moze opcjonalnie zapisac ten sam format do repo-lokalnego, ignorowanego katalogu `.m2a-debug/`.
- Raport nie uruchamia Aurory, NWN ani zewnetrznego walidatora.
- Raport nie modyfikuje plikow wejsciowych, wygenerowanych artefaktow ani konfiguracji uzytkownika.

### 5.2. Struktura paczki V1

```text
m2a-debug-<source-sha>.zip
|- issue-summary.md
|- repro.json
|- canonical/
|  |- session.json
|  |- source-identity.json
|  |- stage-ledger.json
|  |- ingest-ir.json
|  |- normalized-ir.json
|  |- conversion-report.json
|  |- animation-mapping.json
|  |- writer-layout.json
|  |- readback.json
|  |- validation-report.json
|  |- package-manifest.json
|  |- data-lineage.json
|  |- dependency-graph.json
|  |- invariants.json
|  |- binary-hexdump.txt
|  `- pipeline-events.jsonl
|- runtime/
|  |- worker-trace.json
|  `- environment.json
|- artifacts/
|  |- model.mdl
|  `- package.hak
`- screenshots/                 # opcjonalne
   |- source-front.png
   |- output-front.png
   |- output-side.png
   |- output-three-quarter.png
   |- output-wireframe.png
   |- output-bounds-axes.png
   `- output-skeleton-normals.png
```

Pola zalezne od czasu, przegladarki, GPU, wydajnosci i przebiegu Workera trafiaja do `runtime/`. Dane kanoniczne i hashe wyniku trafiaja do `canonical/`. Dzieki temu roznice srodowiskowe nie udaja regresji konwertera.

Jesli MDL albo HAK nie powstal, odpowiedni plik jest nieobecny, a przyczyna jest zapisana w `stage-ledger.json`, `validation-report.json` oraz `issue-summary.md`.

## 6. `repro.json`

`repro.json` ma pozwolic odtworzyc dokladnie ten sam przebieg bez polegania na opisie z pamieci. Minimalny zakres:

- wersja aplikacji i commit Git,
- wersja WASM oraz schematu snapshotu,
- SHA-256 i rozmiar pliku wejsciowego,
- opcje konwersji oraz profil,
- kolejnosc etapow,
- oczekiwane hashe danych kanonicznych i artefaktow,
- podstawowy kod bledu,
- informacja, ktorych plikow wymaganych do replay brakuje.

Lokalna sciezka do pliku, nazwa uzytkownika i dane maszyny nie sa wymagane do reprodukcji i nie moga byc eksportowane domyslnie.

## 7. Stage Ledger i First Difference

Kazdy etap zapisuje stan wejscia, wyjscia i wynik:

`GLB -> ingest -> canonical IR -> normalize -> binary writer -> readback -> HAK package`

Minimalny rekord etapu:

```json
{
  "stage": "binary-writer",
  "status": "pass",
  "inputHash": "sha256:...",
  "outputHash": "sha256:...",
  "startedAfter": "normalize",
  "diagnosticCodes": [],
  "missingOutputs": []
}
```

Porownanie dwoch raportow wskazuje pierwszy etap, na ktorym zmienil sie hash albo status. To rozdziela klasy problemow:

- parser/ingest,
- konwersja i normalizacja,
- writer binarny,
- reader/readback,
- pakowanie HAK,
- frontend lub sam sposob prezentacji.

Nie trzeba wtedy zgadywac na podstawie koncowego komunikatu.

## 8. Data Lineage

Obiekty zachowuja stabilna tozsamosc przez pipeline:

`GLB node -> IR mesh -> normalized mesh -> MDL node -> binary offset -> HAK resource`

Kazdy element diagnostyczny powinien, jezeli to mozliwe, wskazywac:

- stabilny identyfikator,
- identyfikator rodzica,
- nazwe/resref,
- etap utworzenia,
- zakres bajtow w artefakcie,
- zasob wynikowy w HAK.

Dzieki temu klikniecie bledu w UI moze zaznaczyc element viewportu i przejsc do poprawnego offsetu w Binary Inspectorze.

## 9. Macierz invariantow

Walidacja nie moze ograniczac sie do `PASS/FAIL`. Kazdy invariant zapisuje oczekiwanie, wartosc rzeczywista i dowod.

Minimalny zestaw V1:

- indeksy mieszcza sie w zakresie vertexow,
- pointery i zakresy binarne mieszcza sie w artefakcie,
- sekcje, ktore nie moga sie nakladac, nie nachodza na siebie,
- wagi kosci sumuja sie zgodnie z kontraktem i nie przekraczaja limitu czterech,
- referencje kosci istnieja,
- klucze animacji sa uporzadkowane,
- resref spelnia limit i format,
- tekstury oraz modele wskazane przez zasoby istnieja w HAK,
- semantyczny readback odpowiada kanonicznemu IR.

Przyklad rekordu:

```json
{
  "code": "MDL_POINTER_IN_RANGE",
  "status": "fail",
  "entityId": "mesh:body",
  "expected": "offset + length <= artifactSize",
  "actual": "18432 + 4096 > 20480",
  "artifact": "model.mdl",
  "offset": 18432
}
```

## 10. Graf zaleznosci zasobow

`dependency-graph.json` zapisuje zaleznosci potrzebne do wykrywania brakow w paczce:

`appearance row -> model resref -> MDL -> texture resref -> TGA -> HAK`

Graf powinien rozrozniac:

- zasob wymagany i obecny,
- zasob wymagany i brakujacy,
- zasob wygenerowany,
- zasob wejsciowy,
- zasob nieuzywany.

## 11. Read-only Binary Inspector

Binary Inspector jest czescia Debug Drawera w V1. Jest przydatny dla uzytkownika zglaszajacego blad i dla implementatora, ale nie jest edytorem binarnym.

### 11.1. Funkcje V1

- wybor artefaktu `MDL` albo `HAK`,
- offset i 16 bajtow w wierszu,
- podswietlenie wybranego zakresu,
- interpretacje `uint8`, `uint16`, `uint32`, `float32`,
- nazwa pola/sekcji pochodzaca z `writer-layout` lub readback,
- klikniecie diagnostyki przenosi do powiazanego offsetu,
- pobranie aktualnie ogladanego artefaktu,
- domyslnie zwiniety panel.

Viewport nie jest macierza binarna. Zakladka `Converted Model` pokazuje model odtworzony z semantycznego readbacku binary MDL. Kontrolka `Verified by binary readback - Inspect Binary` otwiera `Debug Drawer -> Binary` na powiazanym artefakcie i offsetach. Dzieki temu renderowany wynik oraz macierz bajtow sa dwoma zsynchronizowanymi widokami tego samego artefaktu.

### 11.2. Poza V1

- edycja bajtow,
- writeback do MDL/HAK,
- disassembler,
- zaawansowany diff bajt po bajcie,
- modyfikowanie konfiguracji Aurory lub NWN.

## 12. Debug Drawer

Zakladki V1:

`Diagnostics | Binary | Pipeline Data | Export`

### 12.1. Diagnostics

- filtry `error`, `warning`, `info`,
- kod, opis, etap i element,
- oczekiwane/rzeczywiste wartosci,
- akcje `show in viewport` i `show offset`, jezeli istnieje lineage.

### 12.2. Binary

Read-only Binary Inspector z sekcji 11.

### 12.3. Pipeline Data

- Stage Ledger,
- dane wybranego wezla: path, geometria, material, joints i resref,
- statystyki source kontra output,
- graf zaleznosci,
- jawny ostatni ukonczony etap.

### 12.4. Export

- `Generate Report`,
- lista plikow, ktore wejda do paczki,
- jawne ostrzezenie o nieobecnym artefakcie,
- opcjonalny wybor standardowych screenshotow,
- widoczny hash modelu, ktorego dotyczy raport.

### 12.5. Narzedzia viewportu

Tryb debug moze przelaczac:

- grid,
- axes,
- wireframe,
- bounds,
- normals,
- skeleton,
- bone names,
- selected bone,
- skin weights.

Sa to warstwy odczytowe. Nie zmieniaja modelu ani eksportu.

### 12.6. Animation Player

Read-only Animation Player jest wymagany w `Inspect` oraz `Review Output -> Model Details` i zawiera:

- wybor klipu,
- play/pause, stop i loop,
- timeline oraz biezacy czas/dlugosc,
- predkosc odtwarzania,
- poprzednia/nastepna klatke.

Player dziala dla `Source Model` i `Converted Model`. Nie pozwala edytowac klipow, rigu ani wag.

### 12.7. Reference Trace Bar

Kazda zakladka Review Output ma staly pasek zaleznosci w tym samym miejscu:

- Model Details: data lineage od GLB do offsetu binary MDL,
- `appearance.2da`: dodany wiersz, model resref, MDL, tekstury i zasob HAK,
- Package Contents: encja zrodlowa, zasob wynikowy, zaleznosci i HAK.

Wybor wiersza lub zasobu aktualizuje pasek. Plywajacy popup moze byc dodatkiem, ale nie jedynym miejscem prezentacji referencji.

## 13. Worker i crash trace

Raport z awarii Workera powinien zachowac co najmniej:

- ostatni rozpoczety i ostatni ukonczony etap,
- request/revision ID,
- typ zadania,
- rozmiary transferowanych danych,
- stan `cancelled`, `stale`, `timeout` albo `worker-lost`,
- panic/stack, jezeli runtime go udostepnia,
- ostatnie bezpiecznie zapisane zdarzenia pipeline.

Brak stack trace nie moze zablokowac wygenerowania pozostalej czesci raportu.

## 14. Tryb developerski: Import, Replay, Compare

Po zbudowaniu eksportu V1 tryb developerski powinien dostac przeplyw:

1. `Import Report` - odczyt poprzedniej paczki,
2. `Attach Source` - wskazanie GLB; hash musi odpowiadac `source-identity.json`,
3. `Replay` - ponowienie konwersji z `repro.json`,
4. `Compare` - porownanie snapshotow i artefaktow,
5. `First Difference` - wskazanie pierwszego roznego etapu, obiektu i offsetu.

To narzedzie jest przeznaczone do implementacji i regresji. Nie musi byc eksponowane w podstawowym przeplywie uzytkownika V1.

## 15. Zglaszanie bledu przez GitHub

Rekomendowany komplet zgloszenia:

1. krotki opis problemu,
2. zrodlowy model GLB z Meshy dolaczony swiadomie przez uzytkownika, jezeli jego prawa na to pozwalaja,
3. ZIP wygenerowany przez `Generate Report`.

`issue-summary.md` ma byc gotowym do wklejenia szkieletem zgloszenia i zawierac:

- wersje aplikacji/commit,
- kod glownego bledu,
- ostatni poprawny etap,
- liste brakujacych wynikow,
- identyfikacje modelu przez nazwe, rozmiar i SHA-256,
- kroki replay,
- liste dolaczonych artefaktow,
- informacje o usunietych danych prywatnych.

GLB nie jest automatycznie kopiowany do paczki. Uzytkownik dolacza go oddzielnie i swiadomie.

## 16. Prywatnosc i bezpieczenstwo

Eksport domyslnie usuwa:

- absolutne sciezki lokalne,
- nazwe konta i hosta,
- tokeny, cookies i dane sesji,
- przypadkowe fragmenty innych plikow,
- pelny fingerprint maszyny, jezeli nie jest niezbedny.

Do raportu trafiaja tylko dane dotyczace wskazanego przebiegu. Aurora i NWN pozostaja read-only i nie sa modyfikowane w zadnej fazie debugowania.

## 17. Definition of Done tego zakresu

Zakres Studio V1 + Debug jest domkniety dopiero, gdy:

- glowna powloka realizuje piec etapow `Source -> Inspect -> Build -> Review Output -> Download`,
- viewport pokazuje source oraz wynik z kanonicznego readbacku,
- Build running i Build failed korzystaja z tego samego pelnego widoku pipeline bez viewportu,
- udany Build przechodzi do Review Output bez osobnego pelnego ekranu sukcesu,
- Review pokazuje Model Details, faktycznie dodany wiersz `appearance.2da` oraz manifest HAK,
- wszystkie zakladki Review zachowuja staly Reference Trace Bar,
- UI rozroznia sukces, warning, blad i `OPEN_M6`,
- mozna pobrac kazdy faktycznie wygenerowany MDL i HAK,
- `Generate Report` dziala po sukcesie i po kontrolowanym bledzie,
- paczka przechodzi walidacje schematu i nie zawiera zabronionych danych prywatnych,
- diagnostyka i eksport sa zasilane przez ten sam `DebugSnapshot`,
- klikniecie diagnostyki z offsetem otwiera poprawny zakres Binary Inspectora,
- Stage Ledger wskazuje ostatni ukonczony etap i pierwszy etap roznicy,
- read-only narzedzia debug nie zmieniaja hashy artefaktow,
- przebieg mozna odtworzyc na podstawie `repro.json` po dolaczeniu zgodnego GLB,
- istnieje co najmniej jeden zapisany raport sukcesu i jeden raport kontrolowanego bledu jako evidence.

## 18. Kolejnosc implementacji

Ta kolejnosc minimalizuje przepisywanie UI i formatow:

1. zamrozic schemat `DebugSnapshot`, `repro.json` i Stage Ledger,
2. zasilic snapshot z obecnego Workera i pipeline bez zmiany semantyki konwersji,
3. zbudowac deterministyczny exporter ZIP oraz walidacje prywatnosci,
4. zbudowac powloke Studio z piecioma etapami,
5. podlaczyc `Converted Model` do kanonicznego readbacku i `Inspect Binary` do macierzy bajtow,
6. dodac Debug Drawer i read-only Binary Inspector,
7. dodac standardowe screenshoty i `issue-summary.md`,
8. dodac developerskie Import/Replay/Compare/First Difference,
9. zapisac evidence sukcesu i kontrolowanego bledu,
10. dopiero potem rozwazac kolejne funkcje authoringu z backlogu F1-F10.

Kazdy zamkniety, samodzielny slice implementacyjny powinien konczyc sie przegladem zmian, adekwatna weryfikacja, commitem i pushem zgodnie z ustalonym workflow repo.

## 19. Wnioski i decyzje wlasciciela

### 2026-07-14

- Mockup jest kierunkiem wizualnym, ale ma za duzo funkcji jak na V1.
- V1 ma szybko pokazac realny wynik pracy, nie tylko techniczne kroki pipeline.
- Binary MDL, HAK, ich zapis/pobieranie i raport nie sa dodatkami; sa podstawowym wynikiem produktu.
- `Converted Model` ma byc renderowany z readbacku rzeczywistej binarki.
- Visual Binary Inspector zostaje w V1 jako przydatne, read-only narzedzie.
- Wszystkie dane debug maja powstawac jako jedna paczka przez `Generate Report`.
- Ten sam model diagnostyczny ma obslugiwac UI i eksport, aby raport odpowiadal temu, co widzial uzytkownik.
- Raport wraz z osobno dolaczonym modelem Meshy ma umozliwiac kompletne zgloszenie bledu na GitHubie.
- Tryb debug ma sluzyc rowniez Codexowi podczas dalszej implementacji: replay i first difference maja zastepowac reczne zgadywanie.
- Debugowanie pozostaje read-only wobec Aurory, NWN, plikow konfiguracyjnych i artefaktow wejsciowych.
- Testy runtime Aurory/NWN oraz modele referencyjne nie blokuja pierwszej implementacji Studio; pozostaja w odroczonym suplemencie/proof phase zgodnie z aktualnym planem.
- Workflow V1 ma piec krokow: `Source -> Inspect -> Build -> Review Output -> Download`.
- Build nie pokazuje viewportu; jego centralny obszar zajmuja pipeline, Stage Ledger, hashe i aktualny etap.
- Build running i Build failed maja ten sam uklad. Sukces automatycznie przechodzi do Review Output, wiec osobny ekran Build completed jest odrzucony.
- `Binary Readback` nie jest nazwa macierzy bajtow: `Converted Model` renderuje wynik readbacku, a `Inspect Binary` otwiera macierz w Debug Drawerze.
- Review Output zawiera Model Details, read-only tabele dodanego wpisu `appearance.2da` i Package Contents.
- Kazdy widok Review ma staly Reference Trace Bar w tym samym miejscu.
- Conversion Readiness nie uzywa arbitralnego wyniku 0-100; pokazuje deterministyczne statusy i liczbe regul.
- Package Assembly oraz Runtime Proof sa oddzielne; `OPEN_M6` dotyczy runtime proof.
- Read-only Animation Player i warstwy szkieletu/wag sa czescia Inspect oraz Review Model Details.
- Docelowe zakladki Debug Drawera to wylacznie `Diagnostics | Binary | Pipeline Data | Export`.

## 20. Odlozone rozszerzenia diagnostyczne

Po V1 mozna rozwazyc:

- automatyczny `Reduce failing model`, ktory minimalizuje GLB przy zachowaniu tego samego kodu bledu,
- semantyczny diff dwoch MDL/HAK,
- rozbudowany viewer grafu zaleznosci,
- integracje tworzaca szkic GitHub Issue,
- dodatkowe proof packety NWN EE.

Nie sa one blockerami obecnego zakresu.

## 21. Kontrakt wizualny i stan mockupow

Pelna seria ekranow oraz jawna klasyfikacja widokow znajduje sie w:

- [mockups/studio-v1-2026-07-14/README.md](mockups/studio-v1-2026-07-14/README.md).

Pakiet nie jest jeszcze w calosci `FINAL_VISUAL_CONTRACT`. Build running i Build failed wymagaja przebudowy bez viewportu, ekran Build completed jest odrzucony, a wybrane ekrany Review/Debug wymagaja opisanych korekt spojnosci. Implementacja nie moze kopiowac 1:1 obrazow oznaczonych `REWORK_REQUIRED` ani `OBSOLETE_DO_NOT_IMPLEMENT`.
