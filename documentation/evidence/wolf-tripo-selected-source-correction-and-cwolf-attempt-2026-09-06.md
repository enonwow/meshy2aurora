# Właściwy plik wolf+3d+model.glb — sprostowanie i próba c_wolf

Data: 2026-09-06. Status: **STANDARD_INTAKE_PASS / AUTOMATIC_RIG_PREPARATION_BLOCKED**.

## Sprostowanie

Właściciel wskazał `C:/Users/enonw/Downloads/wolf+3d+model.glb` i zakwestionował podaną wcześniej liczbę prawie 2 mln trójkątów. Miał rację co do niezgodności źródła. Agent wybrał starszy model Meshy bez dostatecznego ustalenia tożsamości modelu oczekiwanego przez właściciela.

Wskazany GLB zawiera **20 534 trójkąty**, 25 830 wierzchołków i 61 602 indeksy. Jego rozmiar wynosi **6 687 432 B**. Zwykły importer aplikacji go przyjmuje. Nie potrzebuje redukcji ani podniesienia limitów. GLB zapisuje tutaj trójkąty; pierwotnej liczby czworokątów lub sposobu liczenia „około 10k poligonów” w innym programie nie ustalano.

Nie wolno przypisywać temu plikowi wyniku starszego `source-p300k.glb`, jego uszkodzeń topologii ani blokady iteracji V9. Nie wolno też przypisywać mu PASS odrębnego źródła bind-v6/V10.

## Dokładna tożsamość

- Właściciel: `C:/Users/enonw/Downloads/wolf+3d+model.glb`.
- Istniejący kanoniczny plik: `sample-3d/borzoi-tripo-dc22ecb0/source.glb`.
- SHA-256 obu plików: `dc22ecb0561fc10773e1a156b0d126225a15cdfb67cd7c0999a965a4f1d59689`; porównano bajtową tożsamość przez hash w bieżącym wznowieniu.
- Jeden mesh, jeden indeksowany primitive TRIANGLES, jeden materiał i jedna osadzona tekstura JPEG.
- Pozycje, normalne i UV obecne. Zero skinów i animacji; to statyczny model wejściowy, nie dowód wady.
- Kierunek: `POSITIVE_Z`. Potwierdzono na projekcji rzeczywistych trójkątów: pysk w +Z, ogon w -Z, łapy na Y=0. Nie obracano ani nie redukowano siatki.
- Referencja: dokładny binarny `c_wolf`, 340 812 B, SHA-256 `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`, chain `c_wolf -> NULL`, rozwiązany read-only z lokalnych KEY/BIF.

## Wykonane próby

Wznowiono pracę po przerwaniu. Zastane raporty Tripo i manifest zostały odczytane, nie nadpisane. Zastany log Vite nie jest dowodem wykonania próby w UI; nie przypisuje się mu takiego wyniku. Nowe sprawdzenia użyły publicznych funkcji WASM aplikacji bez zmiany kodu produktu.

| Etap | Polityka | Faktyczny rezultat |
| --- | --- | --- |
| Zwykłe `inspectGlbJson` | standardowe limity | PASS odczytu: 20 534 trójkąty, brak blokujących gate'ów; tylko ostrzeżenie, że transformacja do Aurory nastąpi później. |
| `prepareReferenceSupermodelRigV3` z przerwanej próby | `allowExcessiveBranchBoundaryRepair=false` | `M2A-REFERENCE-SUPERMODEL-SKIN-BRANCH-REPAIR-EXCESSIVE`: 2 140 zmienionych etykiet przypisania przy limicie 1 291. |
| Osobne rozszerzenie diagnostyki w tym wznowieniu | `allowExcessiveBranchBoundaryRepair=true`, bez API eksportu | `M2A-REFERENCE-SUPERMODEL-SKIN-LOCAL-COVERAGE-BLOCKED`: 16 wymaganych jointów nie otrzymało kwalifikującego się dominującego wierzchołka. Innych kontroli nie wyłączano. |

Rozszerzona próba trwała około 12,6 s. Zmieniono wyłącznie jawny parametr pozwalający algorytmowi przekroczyć limit liczby korekt **na potrzeby diagnostyki**. To nie jest poprawka produktu, przejście polityki domyślnej ani zgoda na publikację wyniku. Ta opcja istnieje również w API produktu, lecz tutaj nie wywołano żadnego product/export API.

Lista jointów z drugiej blokady:

```text
Wolf_LfrontbotlegB, Wolf_Lfrontpaw,
Wolf_Rfrontupperleg, Wolf_RfrontbotlegA, Wolf_RfrontbotlegB, Wolf_Rfrontpaw,
Wolf_tail, Wolf_tailend,
Wolf_Lbacktopleg, Wolf_Lbackmidleg, Wolf_Lbackbotleg, Wolf_Lbackpaw,
Wolf_Rbacktopleg, Wolf_Rbackmidleg, Wolf_Rbackbotleg, Wolf_Rbackpaw
```

Znaczenie liczb jest ograniczone:

- **2 140** to liczba render-wierzchołków, których główna etykieta kości różni się od etykiety na wejściu naprawy granic gałęzi. To około 8,29% z 25 830. Limit to `max(vertex_count / 20, 16)`, czyli 1 291. Nie są to usunięte wierzchołki ani policzone błędy wizualne.
- Kontrola pokrycia wymaga co najmniej jednego wierzchołka, dla którego dany wymagany joint jest wybraną największą wagą i ma wagę co najmniej 0,1. Nie jest to pomiar wielkości spójnego klastra. Komunikat nie mówi, czy joint ma zerowy wpływ, czy tylko wpływy drugorzędne.
- Wynik dowodzi, że **obecny automat nie potrafi poprawnie zakończyć przypisania tego źródła do wybranej referencji**. Nie dowodzi, że siatka jest z natury wadliwa, że model jest niewidoczny w NWN albo że liczba poligonów jest problemem.
- W tym przebiegu nie uzyskano gotowego rigu, nie wykonano finalnej walidacji bind/motion ani testu animacji w grze.

## Braki pipeline'u istotne dla TEGO pliku

1. **Korekta regionów nie jest dostępna przed obowiązkową automatyczną inicjalizacją.** Publiczne API najpierw wylicza bazowy rig, a dopiero potem stosuje poprawki autora. Gdy wyliczenie przerywa się tutaj, istniejące formy edycji wag nie pozwalają naprawić wejścia do algorytmu. Potrzebne są jawne wskazania regionów/kości przed inicjalizacją lub diagnostyczny częściowy wynik dopuszczony do edycji, lecz nadal niedopuszczony do eksportu.
2. **Dopasowanie geometrii jest globalne.** Ustawienie orientacji, jednolitej skali i dolnego środka nie zastępuje lokalnego dopasowania łap, głowy i ogona do niezmienionych przegubów. Potrzebna jest kontrola anatomii niezależna od zgodności samych macierzy szkieletu.
3. **Błędy nie pokazują problemu przestrzennie.** W pierwszym komunikacie mamy licznik, w drugim nazwy jointów, bez gotowej mapy regionów i porównania wag przed/po. Potrzebna jest wizualizacja regionów oraz możliwość bezpiecznej korekty z undo i zachowaniem UV/sylwetki.
4. **Edycja i walidacja muszą tworzyć pętlę naprawczą.** Obecne blokady są użyteczne do zatrzymania złego wyniku, ale nie domykają workflow właściciela. Kryterium ukończenia: poprawa konkretnych regionów pozwala ponownie wyliczyć wagi i przejść niezmienione kontrole, bez wymuszenia wag wyłącznie po to, by każdy joint miał jeden wierzchołek.
5. **Po naprawie nadal potrzebna jest kontrola ruchu i finalnego eksportu.** Gęstsze sprawdzanie ekstremów i odziedziczony chain w finalnym readback pozostają brakami ze statycznego audytu. Dzisiejszy przebieg nie doszedł do tych etapów i nie potwierdza żadnego ich wyniku dla tego modelu.

Za dużo poligonów, brak redukcji high-poly i oczekujący test starszego V9 **nie są blokadami tego źródła**. Mogą pozostać ogólnymi zagadnieniami produktu, ale nie powinny kierować dalszą pracą nad tym plikiem.

Podstawy kodowe:

- [Kolejność bazowego rigu i authoringu](C:/Projects/meshy2aurora/crates/m2a-wasm/src/lib.rs:707), [stosowanie edycji](C:/Projects/meshy2aurora/crates/m2a-wasm/src/lib.rs:751).
- [Limit naprawy granic](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_skinning.rs:941), [liczenie różnic etykiet](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_skinning.rs:1136).
- [Kwalifikacja dominujących wierzchołków](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_generic.rs:1314), [globalna rejestracja](C:/Projects/meshy2aurora/crates/m2a-core/src/reference_supermodel_generic.rs:1624).
- [Szerszy audyt Creature](C:/Projects/meshy2aurora/documentation/audyt-creature-animacje-supermodele-2026-09-05.md).

## Dowody i granica ukończenia

- [Standardowy odczyt wskazanego pliku](C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-tripo-cwolf-20260906-extended-diagnostic/standard-source-inspection.json).
- [Raport domyślnej próby](C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-tripo-cwolf-20260906/prepare-original.json).
- [Raport rozszerzonej diagnostyki](C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-tripo-cwolf-20260906-extended-diagnostic/prepare-original.json).
- [Metadane rozszerzonego wykonania](C:/Projects/meshy2aurora/artifacts/diagnostics/borzoi-tripo-cwolf-20260906-extended-diagnostic/run.json).

Użyty zastany WASM: SHA-256 `bcd05d4230107b8af5c5d5f0610887dc857b5bde95c134d0ba963a0235d8627e`. Nie przebudowano produktu; statyczne wnioski z bieżącego kodu i wykonanie tego konkretnego binarium są rozdzielone.

Guard kanonicznego worktree i gate układu assetów przeszły. Nie zmieniono źródła w Downloads ani kanonicznego GLB. Nie utworzono nowego MDL, HAK lub MOD, nie uruchomiono Toolsetu/NWN. Dawne V9/V10 pozostają nietknięte. Sprostowanie i lista braków są gotowe; zlecenie stworzenia gotowego modelu z `c_wolf` pozostaje nieukończone z powodu opisanej blokady automatycznego rigu.
