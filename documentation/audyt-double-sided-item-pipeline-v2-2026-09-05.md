# Audyt `doubleSided` w pipeline Item — V2

Data: 2026-09-05

Zakres: diagnoza offline i implementacja semantyki `doubleSided` dla statycznych
części Item. Nie uruchamiano ani nie sterowano Aurora Toolset lub NWN i nie
utworzono nowej iteracji hełmu.

## Objaw i tożsamość

Właściciel potwierdził w Toolsecie, że w kandydacie 227 wybór `Cloth 2` nie
barwi widocznej powierzchni wkładek oczu, mimo że materiał oczu jest widoczny
w Tripo.

Fakty ze źródła i readbacku:

- owner source `source.glb` ma SHA-256
  `c2f4370e24fe866bc9303f7b4264a389f7a2d940bb7baffa2c43b00b414444bd`;
- wszystkie pięć materiałów źródłowych deklaruje `doubleSided=true`;
- atlas `source-palette-atlas-v2.glb` ma SHA-256
  `f9a6349da28a4c270d3670ac6d507af33ad58a9e0269d63479a4922acd431062`
  i również deklaruje `doubleSided=true`;
- źródło po jedynym dopuszczonym usunięciu dokładnie zerowego trójkąta ma
  42 465 trójkątów;
- wynikowy MDL 227 zachowuje 42 465 trójkątów, w tym 430 trójkątów wkładek
  oczu; wszystkie 430 próbek UV trafiają na warstwę PLT 5 (`Cloth 2`);
- 415 z 430 face normals wkładek jest skierowanych przeciwnie do zewnętrznego
  frontu hełmu w przestrzeni docelowej.

Wniosek implementacyjny: V1 zachowywał geometrię, UV oraz warstwę PLT, lecz
nie przenosił semantyki `doubleSided` do klasycznego binary MDL. Tripo mógł
pokazać tylną stronę powierzchni, a wynik V1 pozostawiał decyzję rendererowi
Aurory. To wyjaśnia rozjazd lepiej niż wcześniejsza hipoteza o zbyt niskiej
luminancji PLT; podniesienie luminancji w 227 nie zmieniło wyniku właściciela.

## Wybrane rozwiązanie

Dodano wersjonowaną trasę `compile_meshy_static_item_part_v2`.

Dla każdego faktycznie użytego materiału źródłowego z `doubleSided=true` V2:

1. zachowuje oryginalne front faces;
2. duplikuje odpowiadające im wierzchołki;
3. zachowuje pozycje i UV;
4. odwraca normalne;
5. zachowuje tangent XYZ i odwraca tangent handedness W;
6. zachowuje weights i face surface metadata, gdy są obecne;
7. emituje odwrotny winding `[a, c, b]` dla drugiej strony.

To jest klasycznie kompatybilna emulacja dwustronności i nie zależy od
niepotwierdzonego połączenia modelowego PLT z MTR `twosided`.

Ekspansja następuje przed wspólną bramką
`AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000`. Raport rozdziela:

- `sourceTriangleCount`;
- `doubleSidedBackfaceTriangleCount`;
- końcowy `triangleCount`;
- zastosowaną politykę `doubleSidedPolicy`.

Stara funkcja V1 pozostaje bez zmiany geometrii i raportuje
`SOURCE_DOUBLE_SIDED_LEGACY_UNMAPPED_V1`, aby nie zmieniać po cichu zamrożonych
generatorów 225–227.

## Weryfikacja offline

Przeszły:

- `cargo test -p m2a-core --test item_part` — 8/8;
- `cargo test -p m2a-core --test item_package` — 8/8;
- `cargo check -p m2a-core --all-targets`.

Test regresyjny potwierdza pary front/back, odwrotny winding, przeciwne
normalne, identyczne UV i deterministyczny raport. Drugi test potwierdza, że
150 001 trójkątów dwustronnych daje 300 002 i zostaje zablokowane przez wspólny
budżet. Osobny test chroni historyczne zachowanie V1.

## Pozostałe ryzyko i następny krok

Nie ma jeszcze owner proofu Aurory dla outputu wygenerowanego przez V2.
W przypadku bieżącego atlasu V2 przewiduje 84 930 trójkątów, czyli pozostaje
poniżej wspólnego limitu 300 000. Następna dopuszczona iteracja hełmu powinna
użyć wyłącznie trasy V2, zachować mapowanie oczu na `Cloth 2` i przejść
readback geometry/UV/PLT przed przekazaniem właścicielowi do proofu wizualnego.
