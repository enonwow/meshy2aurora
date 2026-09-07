# Audyt jakości Borzoi `c_wolf` V3

Data: 2026-08-20\
Kandydat: `m2aborzmod3.mod` / `m2aborzhak3.hak` / `m2aborzcre3.mdl`\
Wynik właściciela: `modelVisibility=visible`, `proofCompleteness=verified`,
`animationInheritance=observed`, `qualityVerdict=failed`

## Wniosek

V3 nie jest problemem widoczności ani braku animacji. Model jest rysowany, a
stany walki `c_wolf` są aktywne. Główna awaria leży w zgodności przestrzennej
szkieletu i jakości wag SkinMesh. Dodatkowo demo omija istniejący preflight
źródła i pełny kompilator materiałów, więc do NWN trafia bardzo rozdrobniona
siatka z samą teksturą diffuse.

Najlepszą ścieżką naprawy nie jest dalsze poszerzanie progów XYZ. Potrzebny jest
szkielet zgodny z przestrzenią przegubów `c_wolf`, deformacyjna klatka/cage,
component-aware transfer wag oraz offline oracle, który przed utworzeniem MOD-a
próbkuje rzeczywiste odziedziczone klipy.

## Dowody i fakty

### Wynik runtime

Dowód właściciela:
`documentation/evidence/borzoi-c-wolf-demo-v3-owner-nwn-result-2026-08-20.png`,
SHA-256 `6d477f5eb0eb84a05e3ece4397ad8271bbb7667303697485fdb111d712736e80`.

Na obrazie model jest widoczny i walczy, lecz:

- głowa i szyja są zgniecione oraz skręcone;
- sierść i rejony kończyn tworzą rozciągnięte płaty i kolce;
- sylwetka animowanego psa nie spełnia kryterium jakości.

### Niezgodna przestrzeń przegubów

`build_c_wolf_compatible_rig_v3` buduje kości z własnych znormalizowanych
punktów oraz zapisuje wyłącznie translację lokalną i identycznościową
orientację bind. Odziedziczone klipy `c_wolf` mają kontrolery orientacji dla
23–25 z 30 nodów oraz translację `Wolf_rootdummy`. Zgodność nazw i hierarchii
nie gwarantuje więc zgodności osi, pivotów ani długości segmentów.

Przykładowe lokalne translacje z binary readbacku:

| Node | retail `c_wolf` | V3 |
|---|---|---|
| `Wolf_rootdummy` | `(0, -0.38948, 0.78035)` | `(0, -0.38929, 0.86509)` |
| `Wolf_ribcage` | `(0, 0.26822, -0.03232)` | `(0, 0.61716, -0.07493)` |
| `Wolf_neck` | `(0, 0.45126, 0.05846)` | `(0, 0.18990, 0.14986)` |
| `Wolf_head` | `(-0.00748, 0.13953, 0.17848)` | `(0, 0.22787, 0.09536)` |
| `Wolf_Lfrontupperleg` | `(-0.14551, 0.31512, 0.00961)` | `(0.07095, 0.03511, -0.04087)` |
| `Wolf_pelvis` | `(0, 0.30233, -0.03133)` | `(0, 0.12343, -0.12942)` |

`Wolf_Lfrontupperleg` ma nie tylko inną długość i kierunek, ale także przeciwny
znak osi X. Kod po zmianie bazy `[-x, z, y]` nadal klasyfikuje dodatnie X jako
`left`, podczas gdy retailowy lewy górny przegub znajduje się po ujemnej stronie
lokalnego X. To wymaga jawnego gate'u handedness, a nie dalszej heurystyki.

### Dopasowanie szkieletu jest zbyt płytkie

V3 wykrywa z powierzchni tylko cztery klastry kontaktu łap. Bark, łokieć,
nadgarstek, biodro, kolano/skok, szyja, głowa i ogon powstają z hardkodowanych
proporcji bounds albo liniowej interpolacji do łapy. Dla smukłego borzoja o
długiej szyi i sierści takie przeguby nie muszą pokrywać anatomii.

Gate `primary_vertex_count > 0` potwierdza jedynie, że każda kość nogi jest
największym wpływem dla co najmniej jednego wierzchołka. Nie sprawdza:

- czy wpływane wierzchołki należą do właściwej części anatomicznej;
- czy lewa i prawa strona nie są zamienione;
- czy fragment sierści nie łączy kości nogi z tułowiem;
- czy klip nie odwraca trójkątów i nie rozciąga krawędzi;
- czy głowa, szyja, łapy i ogon zachowują objętość oraz sylwetkę.

### Źródło jest mocno rozdrobnione

Diagnostyka exact `source-p300k.glb` wykazała:

| Metryka | Wynik |
|---|---:|
| wierzchołki | 233 832 |
| trójkąty | 300 000 |
| rozłączne komponenty | 1 836 |
| komponenty 1-trójkątowe | 311 |
| komponenty o najwyżej 2 trójkątach | 435 |
| komponenty o najwyżej 100 trójkątach | 1 391 |
| trójkąty z jednym primary bone | 290 287 |
| trójkąty z dwoma primary bones | 9 675 |
| trójkąty z trzema primary bones | 38 |
| komponenty obejmujące więcej niż jeden primary bone | 363 |
| małe komponenty `<100` wierzchołków obejmujące więcej niż jeden primary bone | 179 |

Nie znaleziono geometrycznie zdegenerowanych trójkątów, więc sam writer ani
zerowe pole trójkąta nie są główną przyczyną. Problemem jest połączenie wielu
luźnych wysp sierści z wagami liczonymi z globalnego XYZ.

Projekt ma już `inspect_model_source_quality_v1`, który raportuje fragmentację,
UV i czytelność tekstury. Jednorazowy materializer borzoja nie wywołuje tego
preflightu. Przy tej próbce istniejący gate co najmniej ostrzegłby o ponad 256
komponentach o najwyżej dwóch trójkątach.

### Wagi V3 nie rozumieją topologii ani anatomii

`procedural_weights_v3` używa pasów znormalizowanego Y/Z, odległości XYZ od
łańcucha i progu zależnego od szerokości modelu. Poza pasami cały wierzchołek
otrzymuje blend `ribcage/pelvis`. Nie ma geodezyjnego dystansu po powierzchni,
przypisania komponentu, masek anatomicznych ani transferu z klatki
deformacyjnej. Długi włos znajdujący się blisko nogi może więc dostać wagę nogi,
mimo że wyrasta z tułowia.

### Obecny oracle mierzy ruch, nie jakość

Repozytorium ma `evaluate_skin_deformation_v1`, ale działa na klipach zapisanych
lokalnie w tym samym MDL. Borzoi ma zero klipów lokalnych i dziedziczy je z
`c_wolf`, więc materializer ogranicza się do liczby aktywnych kości, topologii,
readbacku i primary regions. Nie składa drzewa bazowego borzoja z drzewem klipu
supermodelu i nie mierzy zniekształceń.

### Materiał demo traci dwa z trzech mapowań

Źródło zawiera:

- base color JPEG 4096×4096;
- metallic/roughness JPEG 2048×2048;
- normal JPEG 4096×4096.

Demo dekoduje bezpośrednio `images[0]` do pojedynczego TGA. HAK zawiera tylko
MDL, diffuse TGA i `appearance.2da`; nie zawiera normal mapy, mapy specular,
MTR ani TXI. Repozytorium ma już compiler MTR/TXI i profile materiałowe
Creature, ale ta trasa ich nie używa. Nie wyjaśnia to katastrofalnej deformacji,
lecz obniża jakość powierzchni po naprawie rigu.

### Maksymalny budżet nie jest celem jakościowym

V3 używa dokładnie 300 000 trójkątów, czyli maksymalnego wspólnego budżetu.
Wynik to 14 strumieni SkinMesh, MDL 25,5 MB, diffuse TGA 50,3 MB i HAK 76,2 MB.
Produktowy preset Creature sugeruje 100 000 dla `Standard`, 200 000 dla `High`
i 300 000 dopiero dla `Maximum`. Więcej trójkątów nie naprawia anatomii ani
wag; utrudnia iterację i zwiększa koszt animacji.

## Co trzeba poprawić

### P0 — wymagane przed następnym modelem

1. **Supermodel-resolved deformation oracle.**
   Rozszerzyć offline evaluator tak, aby łączył exact bazowy MDL celu z exact
   read-only klipami `c_wolf`, tak jak robi to runtime. Próbkować przynajmniej
   `cpause1`, `cwalk`, `crun`, oba podstawowe ataki, damage i death; finalnie
   wszystkie 42 stany.

2. **Pełny kontrakt jakości deformacji.**
   Mierzyć edge stretch, zmianę pola, poprawnie zdefiniowaną lokalną zmianę
   orientacji i ekspansję bounds; surowy iloczyn skalarny normalnych w świecie
   nie jest testem triangle flip, ponieważ poprawny obrót zmienia jego znak,
   zachowanie objętości głowy/tułowia, odjazd łap od podłoża i kontakt czterech
   łap. Progi należy skalibrować na retailowym `c_wolf` i małym corpusie psów,
   a nie wymyślać dla jednego kadru.

3. **Naprawić handedness oraz przestrzeń przegubów.**
   Lewa/prawa kość, pivot i oś każdego łańcucha muszą zostać potwierdzone
   względem kontraktu supermodelu. Gate powinien porównywać znaki stron,
   kierunki segmentów i neutralną pozę pierwszej klatki klipów.

4. **Zastąpić bezpośrednie XYZ-weighting klatką deformacyjną.**
   Zbudować niskopoligonową, ciągłą klatkę psa dopasowaną do `c_wolf`, nadać
   jej stabilne wagi, zweryfikować animacje, a następnie przenieść wagi na
   render mesh barycentrycznie/geodezyjnie. Render mesh nie powinien sam być
   szkieletem referencyjnym dla 233 832 niezależnie klasyfikowanych vertexów.

5. **Component-aware fur policy.**
   Każda mała wyspa sierści powinna dziedziczyć wagi od miejsca zakotwiczenia w
   ciele albo od jednego/dwóch sąsiednich kości. Komponent nie może łączyć
   nieprzyległych semantycznie kości. Luźne 1–2-trójkątowe śmieci należy usuwać
   lub blokować przed rigowaniem.

### P1 — jakość anatomii i powierzchni

6. **Bogatsze landmarki lub korekta użytkownika.**
   Wymagane są co najmniej: barki, łokcie, nadgarstki, biodra, kolana/skoki,
   łapy, początek/koniec kręgosłupa, szyja, czaszka, nasada i koniec ogona.
   Gdy automatyczne wykrycie ma niską pewność, Studio powinno pokazać kości na
   modelu i pozwolić przesunąć landmarki przed eksportem.

7. **Retopologia deformacyjna.**
   Dla końcowego assetu przygotować czystą bazę wokół barków, łokci, bioder,
   kolan, szyi i ogona. Wysokopoligonowy Meshy surface może pozostać źródłem
   bake'u, lecz nie powinien być bezpośrednio traktowany jako gotowa topologia
   animacyjna. Preferowany preset po retopologii: `Standard` 100k lub `High`
   200k, zależnie od wyniku silhouette/bake, nie automatyczne `Maximum`.

8. **Transfer normalnych i tekstur z high-poly.**
   Po retopologii bake'ować normal mapę i ponownie obliczyć spójne normalne oraz
   tangenty. Dla sierści ustawić materiał niemetaliczny i przekształcić
   roughness do specular/gloss zgodnie z profilem NWN:EE.

9. **Włączyć pełny material compiler.**
   Demo Creature powinno używać tej samej trasy co produkt: diffuse, normal,
   specular, MTR/TXI, tangenty i jawna polityka double-sided/winding. Bez tego
   naprawiony rig nadal będzie wyglądał bardziej płasko niż źródło Meshy.

### P2 — workflow i diagnostyka

10. **Animowany preview przed MOD-em.**
    Studio powinno pokazywać skeleton overlay, kolory primary bone, heatmapę
    stretch/flip i odtwarzać resolved `c_wolf` idle/walk/run/attack. Eksport ma
    być zablokowany na czerwonym gate deformacji.

11. **Zintegrować demo z product route.**
    Materializer borzoja omija source-quality, material i performance profile.
    Następny quadruped powinien przechodzić wspólny pipeline Creature, a demo
    ma tylko pakować exact zaakceptowany wynik, nie implementować własną skróconą
    konwersję.

## Proponowane kryteria zakończenia naprawy

- exact źródło przechodzi Creature source-quality albo ma jawnie zaakceptowaną
  politykę naprawy komponentów;
- side/handedness gate potwierdza wszystkie cztery kończyny;
- wszystkie wymagane kości mają regiony semantycznie zgodne z anatomią, nie
  tylko niezerową liczbę vertexów;
- żaden mały komponent sierści nie obejmuje nieprzyległych kości;
- resolved deformation oracle przechodzi wszystkie 42 klipy według progów
  skalibrowanych na retail/corpusie;
- idle, walk, run i attack preview zachowują szyję, głowę, tułów, ogon oraz
  czteropunktowy kontakt bez kolców, zapadania i triangle flips;
- output używa pełnego profilu materiałowego NWN:EE i zachowuje diffuse oraz
  normal/specular zgodnie z readbackiem;
- performance report nie przekracza wybranego presetu; użycie `Maximum` wymaga
  jawnego uzasadnienia jakościowego;
- właściciel zatwierdza exact zamrożony kandydat w NWN.

## Granica bieżącej pracy

Sam audyt nie utworzył nowego MDL, HAK ani MOD i nie zmienił lineage V3. Po jego
przekazaniu właściciel bezpośrednio polecił wykonanie poprawek i obejrzenie V4
na podstawie exact widocznego, zdeformowanego V3. Ta decyzja ustanowiła wyjątek
jakościowy od visibility-only gate i dopuściła jedną linię V4 z minimalną deltą:
handedness, przestrzeń przegubów, component-aware weights oraz resolved
supermodel deformation gate.
