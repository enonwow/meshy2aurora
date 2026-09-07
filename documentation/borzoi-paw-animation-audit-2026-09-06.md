# Borzoj / c_wolf — audyt animacji i miejscowa korekta łap, 2026-09-06

Status: **poprawiony model diagnostyczny, nadal niedopuszczony do eksportu produktu NWN**.

## Wynik

Wybrany model znajduje się w `artifacts/creatures/borzoi-cwolf-paw-repair-20260906`. GLB: `borzoi-cwolf-paw-repair.glb`, Blender: `borzoi-cwolf-rigged.blend`. Hashe wszystkich plików i raporty zawiera tamtejszy `manifest.json`.

Źródło do dalszego authoringu: `sample-3d/borzoi-tripo-dc22ecb0/source-cwolf-paw-local-final.glb`, SHA-256 `cc8e663457706c4f105d4af9dd82996d43ad1edb8e0d91b6609e1405c908bfd4`. Supermodel c_wolf: SHA-256 `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`. Żadne transformacje referencyjnego riga nie zostały zmienione.

Całościowa kontrola deformacji: **7/42 klipów FAIL**, wcześniej 10/42. Nie ma nowych FAIL względem bazowego gradient-v8. Pozostają: `cconjure1`, `ckdbck`, `ckdbckdie`, `ccwalkf`, `ccwalkb`, `ctaunt`, `cdead`. Nie oznacza to poprawności wszystkich lokalnych fragmentów pozostałych 35 animacji.

## Przyczyny

1. Dopasowanie źródła było głównie globalnym przeskalowaniem. Dolne łapy nie otaczały rzeczywistych osi i przegubów riga.
2. Przejścia wag w tylnych łapach rozciągały dolne segmenty i zapadały powierzchnię przy stawach skokowych. Samo globalne wygładzanie nie naprawiało położenia geometrii.
3. Przeniesienie korekty wyżej na nogi zmieniało też otoczenie tułowia i wprowadziło regresje w unikach/parowaniu. Ta próba została odrzucona.
4. Dotychczasowa kontrola całego komponentu tolerowała nieliczne silne błędy. Dlatego cwalk mógł otrzymać perComponentDeformationCoverage=true mimo wad łap widocznych dla użytkownika. Progi produktu nie zostały poluzowane.
5. Niezależny pomiar długości kości w klipach potwierdził ich stałość z dokładnością zmiennoprzecinkową. Nie stwierdzono rozciągania samego szkieletu.

## Zmiana

Dolne łapy zarejestrowano lokalnie względem osi kości. Środki przekrojów wyliczono z przecięć trójkątów, nie z liczby wierzchołków w pasie. Korekta jest pełna poniżej Z=0.32 i wygasa do Z=0.44. Wagi tylnych nóg lokalnie łączą wcześniejszy wynik z polem wyznaczonym wokół przegubów; w przednich goleniach ograniczono wpływy odległych przegubów: od Z=0.20 do Z=0.34 dominuje właściwa kość odcinka, z łagodnym wejściem od Z=0.14 i wyjściem do Z=0.44. Palce zachowują wcześniejsze przypisania. Powierzchnię przy dwóch tylnych stawach skokowych lokalnie zwężono: 219 wierzchołków, maksymalny ruch 0.019398 jednostki sceny.

Wygładzanie obsługuje teraz `lockedVertexIndices`. Zablokowano 20 221 wierzchołków powyżej Z=0.44, aby korekta łap nie przekształcała wag tułowia. W ostatniej korekcie przednich goleni dodatkowo zablokowano 25 134 wierzchołki, obejmując także tylne łapy. Test na rzeczywistym modelu: zmiana zablokowanych wag przed serializacją dokładnie 0. Maksymalna zmiana pozycji powyżej Z=0.44: 2.98e-8 (roundtrip float32). UV, indeksy, obrazy i materiały zachowane. 25 830 wierzchołków, 20 534 trójkąty, 30 węzłów riga.

## Niezależna kontrola lokalna

Dodano tryby `region-audit` i `joint-audit` do istniejącego adaptera `crates/m2a-wasm/examples/diagnose_creature_skinning.rs`. Badanie regionalne próbuje wszystkie unikalne krawędzie przy 30 kl./s dla 42 klipów, używając odczytu wygenerowanego MDL i prawdziwych animacji referencji. Zapisuje tylko diagnostykę własnej powierzchni; nie kopiuje payloadów retail.

Liczba próbek krawędź×czas w dolnych łapach o długości <25% lub >400% długości bazowej:

| Klip | Przed | Po |
|---|---:|---:|
| ca1stab | 136 | 2 |
| cdamagel | 4 | 0 |
| ckdbckdie | 93 | 31 |
| cwalk | 109 | 2 |
| crun | 20 | 0 |

To wskaźnik miejscowej deformacji, nie liczba wad ani klatek. **W cwalk nadal są dwie złe próbki**, a upadek wciąż wymaga pracy. Zielona kontrola nazewnictwa/bindingu 23/23 w podglądzie nie dowodzi jakości ruchu.

## Odtwarzanie i testy

Receptury i próby: `artifacts/diagnostics/borzoi-paw-repair-20260906`. Wybrany wynik pipeline: `fore-shaft-preview`; gęsty audyt: `regions-fore-shaft`. `local-final-preview` zachowuje wcześniejszy wynik przed korektą przednich goleni. Receptury: `cage-recipe-local.json`, `hock-envelope-local-recipe.json`, `weight-blend-recipe.json`, `fore-shaft-blend-recipe.json`, a ostatni authoring: `fore-shaft-authoring.json`.

Narzędzia: `tools/fit-creature-limb-cages.mjs`, `tools/fit-creature-joint-envelope.mjs`, `tools/author-creature-limb-weight-blend.mjs` oraz istniejący `tools/regularize-creature-weight-gradients.mjs`. Źródło i authoring wracają przez standardowe przygotowanie i authored preview Creature. Wszystkie procesy uruchamiano z windowsHide:true.

Sprawdzone: build adaptera Rust; 42 klipy w kontroli standardowej i regionalnej; zero naruszeń walidacji wag wybranego riga; odrzucenie niedopasowanego rig/source przed utworzeniem pliku; reprodukcja lokalnego mieszania wag (maks. błąd 2.12e-8; przednie golenie 2.22e-16); zachowanie zablokowanych wag; niezmienność topologii/UV/tekstury; import do Blender 5.1.2 (błąd bind 2.39e-7), reakcja czterech łap, głowy i ogona na próby obrotu. Takie próby nie zastępują pełnych animacji.

Toolset/NWN: NOT_TESTED. Nie tworzono produktu, HAK ani MOD do dowodu w grze. Nie należy oznaczać tego modelu jako gotowego do NWN.
