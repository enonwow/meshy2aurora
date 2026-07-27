# Standard fixture'a runtime M0

Status: `ACTIVE / generator contract`.

Ten dokument definiuje jedyny układ używany przez pipeline dla kolejnych
eksportów statycznego modelu Meshy M0. Nie opisuje historycznych modułów
`m2a_m0v6` ani `m2a_m0v12`; one zachowują swoje dane jako evidence, ale nie są
wzorcem dla następnego eksportu.

## Stały Area i tile

| Właściwość | Wartość |
| --- | --- |
| Tileset | `tms01` (Aurora UI: MicroSet) |
| Rozmiar Area | `2 x 2` tile'e |
| Rozmiar świata | `0..20` na osi X i Y |
| Uporządkowany `Tile_List` | `(Tile_ID, Tile_Orientation) = [(12,2), (12,1), (12,3), (12,3)]`; każdy rekord ma `Tile_AnimLoop1/2/3=1`, wysokość `0` |
| Liczba wpisów | dokładnie `4` (`Width * Height`) |

To jest wygenerowany przez Meshy2Aurora kontrakt oparty na własnym odczycie
ARE stworzonego przez Aurorę dla r21 oraz świeżym `TScrollBox`, na którym
widoczny jest M0. Zastępuje on wyłącznie dla kolejnych binarnych vertical
slice'ów M0 historyczny, nierenderowalny układ `tdc01` / `5,0` × 4. Nie
dobieramy tile'a heurystycznie i nie zmieniamy tilesetu per model.

## Start modułu i fixture modelu

| Element | X | Y | Z | Kierunek |
| --- | ---: | ---: | ---: | --- |
| Entry modułu | `10.0` | `10.0` | `0.0` | `[0.0, 1.0]` (północ) |
| Jedyny fixture modelu | `10.0` | `14.5` | `0.0` | niezmieniany przez fixture |

Fixture jest 4.5 jednostki bezpośrednio przed entry pointem. Dzięki temu
pierwszy widok runtime może odpowiedzieć na jedno pytanie: czy bieżący eksport
pipeline'u renderuje się w NWN. Nie testujemy przypadkiem kamery ani położenia
poza początkowym polem widzenia.

## Reguła dla każdego nowego eksportu Meshy

Nowy eksport jest dopuszczony wyłącznie przez bramkę iteracji z
`PROJECT_RULES.md`: poprzedni dokładny kandydat musi mieć świeży, związany z
jego hashami wizualny błąd w Aurora Toolset albo NWN. Brak proofu lub problem
workflow nie jest nowym eksportem.

1. Pipeline mapuje nowy GLB i jego tekstury na nowy resref MDL/TGA oraz jeden
   nowy wiersz `appearance.2da`.
2. Generator tworzy MOD z dokładnie powyższym Area, entry i jedną instancją
   `nw_dwarfmerc001`; zmienia wyłącznie `Appearance_Type`/łańcuch zasobów
   aktualnego eksportu.
3. Własny readback MOD musi potwierdzić HAK, wiersz Appearance, cztery tile'e,
   entry i pozycję fixture'a przed uruchomieniem Aurora lub NWN.
4. Proof wymaga świeżego `TScrollBox` w Aurora oraz świeżego obrazu NWN. Brak
   obiektu na obrazie najpierw obala fixture/capture; nie jest automatycznie
   diagnozą MDL.
5. Toolset proof wiąże dokładny fixture przez wybór i natywny readback obiektu
   oraz świeży `TScrollBox`. `Focus on Object` i kadrowanie są opcjonalne.
   Jeżeli model jest widoczny, ten sam MOD, HAK, MDL, TGA i wiersz Appearance
   przechodzą bezpośrednio do testu NWN.
6. Nowy numer `rNN`, moduł, HAK lub resref bez świeżego błędu wizualnego jest
   naruszeniem kontraktu, nawet gdy payload jest bajtowo identyczny.

## Zamrożony baseline bieżącej linii M0

Dla iteracji `r1`–`r21` nie istnieje żaden świeży, związany z kandydatem wynik
`modelVisibility=not_visible` w Aurora Toolset. Właściciel obserwował model w
viewporcie od `r1`, a artefakt r21 również zawiera widoczną sylwetkę modelu.
Brak opcjonalnego focusu lub kadrowania nie może zmienić tego faktu ani obniżyć
kompletności poprawnie związanego capture. Niepełny capture lub packet zapisuje
się wyłącznie jako `proofCompleteness=missing`, nigdy „model missing”.

Późniejszy historyczny run r21 w NWN zwrócił `modelVisibility=not_visible`,
ale jego fixture był poza kontraktem `[10,14.5,0]`; dopuściło to wyłącznie
placement-only następcę. R25 naprawił pozycję, a świeży r26 (Toolset
`visible`, NWN `not_visible`) obalił identity-controller-only jako poprawkę.
Ten wynik wizualny dopuścił dokładnie jeden dalszy kandydat: **r27**.

Bieżącym kandydatem jest więc r27: zachowuje MOD r25, Area, entry,
fixture, Appearance `848`, `MODELTYPE=S`, `RACE=m2a_m0p01`, teksturę i całą
geometrię r26; zmienia wyłącznie siedem bajtów local-animation
`type: 0 -> 5`. Ma zweryfikowany offline profile/geometry gate, lecz jego
Toolset i NWN pozostają `modelVisibility=not_tested`,
`proofCompleteness=missing`. `next iteration admitted=false` obowiązuje do
czasu świeżego, hash-bound wyniku r27; nie tworzyć r28 z powodu braku proofu.

## Opcjonalny focus i opcjonalne kadrowanie

Przed ocena widocznosci modelu w Aurora Toolset operator musi:

1. potwierdzic tozsamosc dokladnego obiektu w drzewie Area;
2. zaznaczyc ten obiekt i zachowac natywny readback jego tozsamosci;
3. wykonac swiezy capture zwalidowanego `TScrollBox`.

`Focus on Object`, recentering, zoom, pitch i rotacja sa **opcjonalne**. Wolno
ich uzyc wylacznie jako pomocy w poprawieniu czytelnosci kadru. Nie sa osobna
bramka, ich brak nie obniza `proofCompleteness` i nie blokuje przejscia do NWN,
jezeli tozsamosc obiektu oraz swiezy capture wystarczaja do werdyktu.

Sukces samego transportu kamery nie jest dowodem. Jezeli aktualny kadr nie
pozwala ocenic obiektu, capture ma `proofCompleteness=missing` i
`modelVisibility=not_tested`; nie jest to `modelVisibility=not_visible`.

## Ledger iteracji

Każdy kandydat zapisuje jeden rekord:

`candidate hashes -> Toolset modelVisibility/proofCompleteness -> NWN modelVisibility/proofCompleteness -> diagnosed cause -> intended delta -> next iteration admitted`.

`next iteration admitted` może być `true` wyłącznie po świeżym
`modelVisibility=not_visible` dla Toolsetu lub NWN i po zapisaniu konkretnej
przyczyny oraz minimalnej delty. `proofCompleteness=missing` zawsze pozostawia
wartość `false`; nie jest wynikiem widoczności modelu.

## Kontrakt pakietu do capture

Dla lane `M0_BINARY_VERTICAL_SLICE` generator zapisuje
`m0RuntimeFixtureContract` równocześnie w raporcie, summary i manifeście
pakietu. Kontrakt jest rekonstruowany z wygenerowanych bajtów, nie z samego
żądania eksportu, i wiąże:

- SHA-256 oraz resref MOD, HAK, MDL, TGA i `appearance.2da`;
- jawny `stateProjectionProfile=RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1`, brak
  proweniencji CEP, pełne payload-free `DirectCreatureStructuralSummaryV1`
  oraz SHA-256 jego kanonicznego JSON;
- uporządkowaną listę `Mod_HakList`, entry IFO oraz jedyny fixture GIT;
- `Appearance_Type`, fizyczny wiersz `appearance.2da`, `LABEL`, `MODELTYPE`
  i `RACE`;
- `FullRuntimeAppendV1`: pełny prefix wejściowego `appearance.2da`, dokładnie
  jeden append i fizyczny row fixture;
- wynik bramki geometrii MDL oraz zweryfikowaną Retail projekcję każdego
  drzewa stanu jako kompletne `0x01` dummy bez mesh/skin/raw-MDX payloadu.

Przed powiązaniem obrazu Aurora albo NWN z eksportem należy uruchomić
`verify_m0_binary_runtime_fixture_contract_v1`. Zmiana pozycji fixture'a,
kolejności/hasza HAK albo mapowania `RACE` powoduje błąd kontraktu. Zielony
wynik nadal oznacza wyłącznie zgodność artefaktów — proof wizualny wymaga
osobnych, świeżych capture'ów obu środowisk.

Builder kontraktu i verifier niezależnie parsują dokładne bajty MDL, wywołują
`verify_direct_creature_state_projection_v1` z profilem Retail i brakiem
proweniencji oraz ponownie wyliczają summary/digest. Samozgodna podmiana hashy,
HAK-a i summary nie może dopuścić drzewa `0x21` ani rodziny CEP. Osobny profil
`CepRigidPlaceholderV1` jest legalny wyłącznie poza M0 i tylko z dokładnym
`MdlStateProjectionProvenanceV1` jednego z audytowanych zasobów
`cep3_core1` R3; brak lub błędna proweniencja jest błędem conformance.

Zewnętrzne witnessy corpusu nie są częścią zwykłego zielonego test runu.
Testy są jawnie `ignored` i przechodzą fail-closed tylko po uruchomieniu:

```powershell
$env:M2A_REQUIRE_RUNTIME_WITNESSES='1'
cargo test -p m2a-core --test runtime_witness_conformance -- --ignored
```

Brak któregokolwiek wymaganego źródła obala ten run. Zwykły `cargo test`
raportuje te przypadki jako `ignored` i nie jest dowodem walidacji witnessów.

Kontrakt jest egzekwowany w `crates/m2a-core/src/proof_module.rs` i przez
`crates/m2a-core/src/model_pipeline.rs`,
`crates/m2a-core/tests/binary_m0_vertical_slice_module.rs` oraz
`crates/m2a-core/tests/model_pipeline.rs`.

### Granica aktualnego kontraktu v1 dla r27

`M0RuntimeFixtureContractV1` jest celowo kontraktem **jednego** generowanego
HAK-a: jego builder i verifier odrzucają `Mod_HakList` o długości innej niż
`1`. Nie można więc przedstawiać zielonego pojedynczego kontraktu v1 jako
readbacku faktycznego r27, którego zapisany MOD r25 ma kolejność
`[m2a_m0r27, m2a_m0r26, m2a_m0r21]`. Jest to luka eligibility/proof aplikacji,
nie nowy wynik widoczności modelu ani zgoda na r28.

Przed uznaniem capture r27 za `proofCompleteness=verified` potrzebny jest
osobny, test-first kontrakt V2: ma związać hash każdego HAK-a, zachować
kolejność `Mod_HakList` i jawnie wyprowadzić zwycięzcę dla
`appearance`/2017, `m2a_m0p01`/2002 oraz `m2a_m0t01`/3. Szczegółowa diagnoza,
granica xoreos i minimalne testy są w syntezie §55. Do tego czasu nie zmieniać
r27, HAK-list ani zasobów, a brak V2 zapisywać jako
`proofCompleteness=missing`.
