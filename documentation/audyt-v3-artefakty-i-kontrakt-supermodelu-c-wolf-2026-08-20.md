# Audyt V3: artefakty renderingu i kontrakt supermodelu `c_wolf`

Data: 2026-08-20

Zakres audytu obejmuje dokładnie źródło
`sample-3d/borzoi-meshy-manual-p1997k-v1/source-p300k.glb` oraz zamrożoną
linię V3 z `proof-output/borzoi-c-wolf-demo-v3-20260820`. Audyt nie tworzy
nowego modelu, HAK-a ani modułu.

## Werdykt

Obie uwagi właściciela są zasadne.

1. **V3 ma potwierdzoną regresję materiałową w trasie demonstracyjnej.**
   Źródło wymaga renderowania dwustronnego i zawiera trzy obrazy, ale demo
   przenosi tylko obraz bazowy. Nie emituje ani nie pakuje MTR, dlatego gubi
   `doubleSided=true`, normal mapę i mapę metallic/roughness. Dla cienkich,
   rozłącznych powierzchni sierści utrata `doubleSided` bezpośrednio zmienia
   widoczność trójkątów i jest pierwszą przyczyną do usunięcia dla widocznych
   dziur oraz ciemnych wycięć.
2. **Obecna funkcja `retarget_static_mesh_to_reference_supermodel_v2` nie jest
   pełnym retargetingiem do supermodelu.** Sprawdza nazwę i topologię węzłów,
   konwertuje siatkę przy użyciu osobno dostarczonego riga, a następnie wpisuje
   `c_wolf` do nagłówka MDL. Nie sprawdza ani nie dopasowuje pozy spoczynkowej,
   lokalnych układów kości, inverse bind matrices, osi kości ani skali łańcuchów.
   Nazwa funkcji i komunikat produktu obiecują więc więcej niż realizuje kod.

V3 faktycznie odwołuje się do klipów `c_wolf` w silniku, ale dziedziczenie
klipów nie retargetuje automatycznie niezgodnej pozy bazowej dziecka. Obecny
wynik to połączenie **riga dopasowanego do powierzchni borzoja** z
**kontrolerami napisanymi dla lokalnych układów `c_wolf`**. To wyjaśnia inną
postawę, wąsko ustawione przednie łapy i sztuczne przejście do ruchu.

## 1. Audyt artefaktów V3

### 1.1. Źródło

Sprawdzony GLB ma następujące właściwości:

| Właściwość | Wartość |
|---|---:|
| SHA-256 | `f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda` |
| Trójkąty | 300 000 |
| Wierzchołki | 233 832 |
| Prymitywy / materiały | 1 / 1 |
| Obrazy | 3 |
| Rozmiar tekstury bazowej | 4096 × 4096 |
| `alphaMode` | `OPAQUE` |
| `doubleSided` | `true` |
| Rozłączne komponenty powierzchni | 1 836 |

Materiał wiąże osobno base color, metallic/roughness i normal mapę. Atlas UV
jest bardzo gęsty i składa się z wielu małych wysp. To ostatnie zwiększa ryzyko
bleedingu mipmap, ale samo w sobie nie zostało jeszcze dowiedzione jako
przyczyna artefaktów ze zrzutu.

Wynik analizy statycznej znajduje się w
`artifacts/diagnostics/borzoi-v3-source-static-audit-20260820/analysis.json`.
Projekcje diagnostyczne zachowują spójną sylwetkę źródła. Ich uproszczone
kolorowanie per trójkąt nie jest testem jakości finalnego renderera.

### 1.2. Co gubi demo

`materialize_borzoi_c_wolf_demo_v1.rs` poprawnie oczekuje trzech obrazów dla
V3, lecz potem:

- przypisuje do MDL wyłącznie jeden `diffuse_texture_resref`;
- dekoduje na TGA wyłącznie obraz o indeksie `0`;
- pakuje dokładnie MDL, jedną TGA i `appearance.2da`;
- nie uruchamia istniejącego kompilatora materiału EE;
- nie emituje MTR z `twosided 1`;
- nie pakuje normal mapy ani jawnie rozstrzygniętej mapy specular/roughness.

Potwierdza to `package-manifest.json` V3: jedynymi zasobami są
`m2aborzcre3:2002`, `m2aborztex3:3` oraz `appearance:2017`.

Kod projektu ma już właściwą regułę materiałową:

- dla profilu klasycznego aktywne `doubleSided` jest blokowane;
- dla profilu NWN:EE `doubleSided=true` mapuje się na `mtr.twosided`;
- serializer MTR potrafi zapisać `twosided 1`.

Defekt polega więc na tym, że demo omija istniejącą trasę materiałową, a jego
kontrakt wejścia sprawdza liczbę obrazów bez sprawdzenia, czy wszystkie istotne
semantyki zostały odwzorowane w wyjściu.

### 1.3. Co nie wskazuje obecnie na przyczynę

Model został deterministycznie podzielony na 14 strumieni mieszczących się w
limicie binarnego MDL. Kontrola readback nie wykazała różnic w kolejności i
liczbie trójkątów, pozycjach, normalnych, UV ani indeksach
(`semanticDiff=[]`). Nie ma obecnie dowodu, że sam podział geometrii stworzył
artefakty V3.

`semanticDiff=[]` nie dowodzi jednak poprawności materiału, ponieważ kontrola
nie obejmuje utraconego `doubleSided` i nieistniejącego MTR. To właśnie luka w
dotychczasowym kryterium.

Brak normal mapy może pogarszać kształtowanie światłem i uwydatniać ostre
granice. Mapy metallic/roughness z glTF nie wolno natomiast bezpośrednio nazwać
mapą specular Aurory; musi przejść jawną, udokumentowaną konwersję albo zostać
oznaczona jako świadomie niewspierana. Żadnego z tych kanałów nie wolno już
gubić po cichu.

## 2. Audyt dziedziczenia po `c_wolf`

### 2.1. Co zawiera obecny kontrakt

Każdy `ReferenceSupermodelNodeV1` zawiera tylko:

- `partNumber`;
- `name`;
- `parentPartNumber`.

Kontrakt V3 ma 30 węzłów, ale nie zawiera ani jednego pola pozy bazowej lub
układu lokalnego. Funkcja potrafi więc stwierdzić wyłącznie, że hierarchia ma
oczekiwane nazwy i rodziców. Nie potrafi wykazać, że węzły dziecka znajdują się
w tych samych lokalnych układach co kontrolery supermodelu.

`reference_supermodel.rs` opisuje tę granicę wprost: bind transforms, skin i
wagi pochodzą z osobnego `CreatureRigProfileV1`. Następnie writer zapisuje
przekazanego creature z nazwą supermodelu i raportuje `localAnimationCount=0`.

### 2.2. Skąd pochodzi pozycja V3

`build_c_wolf_compatible_rig_v2` nie odtwarza pozy `c_wolf`. Funkcja:

1. obraca osie GLB do układu Aurory, centruje model i ustawia minimum Z na
   ziemi;
2. wybiera klastry kontaktowe bezpośrednio z powierzchni borzoja;
3. dopasowuje do nich proceduralne landmarki;
4. tworzy wszystkie `bindLocalMatrix` jako same translacje między tymi
   landmarkami;
5. generuje proceduralne wagi na podstawie położenia w boundsach źródła.

Jedynie `rootdummy` ma ręczną aproksymację komentarzem odnoszącym się do
neutralnej translacji kontrolera `c_wolf`. Pozostałe lokalne układy nadal
pochodzą z proporcji borzoja. Jest to częściowa korekta jednego węzła, nie
zgodność riga z supermodelem.

### 2.3. Rzeczywiste znaczenie supermodelu w tym wyniku

Wpis `supermodel c_wolf` pozwala silnikowi rozwiązać klipy, których model V3 nie
ma lokalnie. To działa i dlatego w grze pojawia się ruch. Nie oznacza jednak:

- automatycznego przeliczenia pozy spoczynkowej borzoja na pozycję wilka;
- dopasowania długości kości;
- zmiany osi lokalnych stawów;
- wyprowadzenia prawidłowych inverse bind matrices;
- automatycznego skinningu siatki;
- zachowania kontaktu łap z ziemią.

Dlatego obecne API realizuje **emisję statycznej siatki z odwołaniem do
supermodelu i zgodną topologią nazw**, a nie „dziedziczenie wszystkiego po
supermodelu”. Z punktu widzenia obietnicy funkcjonalnej jest to błąd kontraktu
i walidacji, nawet jeśli wąska implementacja zachowuje się zgodnie ze starym
typem `ReferenceSupermodelContractV1`.

## 3. Przyczyny źródłowe

### A. Artefakty materiałowe — potwierdzone

`doubleSided=true` i dodatkowe mapy są poprawnie odczytane z GLB, lecz trasa
demo redukuje materiał do jednej tekstury diffuse. HAK nie ma zasobu MTR.

### B. Niezgodna poza — potwierdzone

Kontrakt supermodelu sprawdza topologię, lecz nie bind/rest pose. Rig V3 jest
dopasowywany do powierzchni borzoja niezależnie od lokalnych układów animacji
`c_wolf`.

### C. Niedostateczne testy — potwierdzone

Testy trasy supermodelowej potwierdzają nazwę supermodelu, nazwy/rodziców,
wagi, brak lokalnych animacji i binarny readback. Nie mają testu zgodności rest
pose, osi kości, inverse bind, kontaktu łap ani przejścia neutralna poza → klip.

### D. Bleeding atlasu UV — ryzyko wtórne, jeszcze nieudowodnione

Gęsty atlas i mipmapping mogą tworzyć cienkie ciemne obwódki. Nie należy jednak
zmieniać UV ani wypalać nowej tekstury przed weryfikacją tej samej geometrii z
poprawnym `twosided` i kompletnym materiałem.

## 4. Minimalny plan naprawy

### Etap 1 — domknięcie materiału V3

1. Demo creature musi korzystać z tego samego kompilatora materiałów NWN:EE co
   pozostałe trasy produktu.
2. Dla tego źródła musi powstać MTR z `twosided 1` oraz komplet wymaganych
   zasobów tekstur.
3. Normal mapa ma zostać przeniesiona zgodnie z kontraktem MTR/TXI.
4. Metallic/roughness musi otrzymać jawny wynik: poprawna konwersja do semantyki
   Aurory albo raportowane pominięcie. Ciche pominięcie jest błędem.
5. Nowy `materialSemanticDiff` ma blokować paczkę przy utracie `doubleSided`,
   kanału obrazu, UV setu, sampler policy albo wymaganej zależności MTR/TXI.

### Etap 2 — prawdziwy kontrakt zgodności supermodelu

1. Obecną trasę należy opisać/nazwać jako topology-only, np.
   `emit_static_mesh_with_supermodel_topology_v1`.
2. Tryb oferowany użytkownikowi jako „dziedziczenie animacji z supermodelu” ma
   otrzymać `ReferenceSupermodelCompatibilityV2`, obejmujący co najmniej:
   lokalne i globalne rest transforms, handedness, osie kości, uniform scale,
   inverse-bind consistency, wymagane węzły kontrolowane przez klipy oraz
   anchor points łap/roota.
3. Wybranie `c_wolf` ma być fail-closed, jeżeli rig nie przejdzie tego
   kontraktu. Samo dopasowanie nazw i rodziców nie może dawać statusu
   „compatible”.
4. Raport ma rozdzielać:
   `supermodelClipLookup=true/false` od `bindPoseCompatible=true/false` i
   `motionRetargeted=true/false`.

### Etap 3 — wybór legalnej i technicznej trasy ruchu

Są dwie poprawne możliwości:

- **zgodny child rig:** niezależnie zbudowany lub dostarczony przez właściciela
  szablon riga zgodnego z `c_wolf`, do którego skinujemy siatkę;
- **pełny motion retarget:** przeliczenie klipów na posiadany rig borzoja i
  emisja lokalnych animacji zamiast polegania na surowych kontrolerach
  supermodelu.

Reguły projektu zabraniają kopiowania retailowego szkieletu i animacji jako
payloadu. Retail może służyć do read-only odkrywania i walidacji semantyki, ale
nie może zostać po prostu skopiowany do produktu. Wybór jednej z dwóch tras
musi więc poprzedzić kolejną implementację modelu.

### Etap 4 — nowe bramki jakości deformacji

Poprzednia bramka V4 mierzyła zachowanie wewnątrz każdego z 1 836 rozłącznych
komponentów, ale nie mierzyła wzajemnej spójności między komponentami. Dlatego
mogła zaakceptować osobno zachowane trójkąty, które rozjechały się jak sztywne
płaty.

Nowa bramka musi dodatkowo zachowywać relacje sąsiedztwa powierzchniowego:

- pary bliskich wierzchołków z różnych komponentów w pozy bazowej;
- odległość i względny kierunek tych par po deformacji;
- ciągłość sylwetki i limity separacji per klip;
- kontakt czterech łap z płaszczyzną ziemi;
- szerokość rozstawu pary przednich i tylnych łap.

## 5. Kryteria zakończenia

### Automatyczne/offline

- [ ] Dla źródła `doubleSided=true` HAK zawiera MTR, a jego readback zawiera
      dokładnie `twosided 1`.
- [ ] Manifest paczki rozlicza wszystkie trzy obrazy źródła; żaden kanał nie
      znika bez jawnej dyspozycji `preserved`, `converted` albo `blocked`.
- [ ] Normal mapa jest poprawnie powiązana z MTR/TXI i przechodzi readback.
- [ ] 300 000 trójkątów, pozycje, normalne, UV, materiały i kolejność indeksów
      przechodzą source → segmenty → MDL readback bez niedozwolonej różnicy.
- [ ] Raport zgodności `c_wolf` zawiera rest/bind compatibility; topology-only
      nie może przejść jako pełna zgodność.
- [ ] Każdy ważony lub animowany węzeł ma zgodny układ lokalny albo jawnie
      wyliczoną korektę retargetingu.
- [ ] Test neutralna poza → idle/walk/run/attack nie wykazuje skoku root,
      skrzyżowania łap ani separacji komponentów poza limitem.
- [ ] Bramka międzykomponentowa odrzuca dokładny przypadek V4.
- [ ] Test regresyjny odrzuca dokładny przypadek V3: trzy obrazy wejściowe,
      jedna TGA wyjściowa i brak MTR.

### Właściciel: Toolset/NWN

- [ ] W statycznej pozie nie ma trójkątnych dziur, ciemnych wycięć ani
      znikających płatów sierści z żadnego boku modelu.
- [ ] Neutralna postawa ma cztery łapy na ziemi i naturalny, nieskrzyżowany
      rozstaw przednich łap.
- [ ] Idle, chód, bieg i co najmniej dwa ataki korzystają z oczekiwanej rodziny
      ruchu `c_wolf`, bez gwałtownej zmiany pozy przy starcie klipu.
- [ ] Sylwetka pozostaje ciągła; żaden fragment sierści lub kończyny nie odrywa
      się od ciała.
- [ ] Ten sam dokładny MOD/HAK/model przechodzi ocenę w Toolset i NWN.

## Konkluzja

Nie należy poprawiać V3 kolejnymi współczynnikami wag ani tworzyć V5 przed
domknięciem tych dwóch kontraktów. Najpierw trzeba naprawić zachowanie materiału
oraz uczciwie rozdzielić „lookup klipów supermodelu” od „zgodnego riga/pełnego
retargetingu”. Dopiero potem sens ma nowa iteracja wizualna oparta ponownie na
geometrii i ogólnym skinningu V3, nie na sztywnych komponentach V4.
