# Kontrakt runtime creature MDL w NWN

Data: 2026-07-26  
Status: `STATIC_CONTRACT_KNOWN / EXACT_R43_STATIC_DRAW_ELIGIBLE /
DYNAMIC_BINDING_NOT_CAPTURED`

## 1. Odpowiedź

Wiedza o tym, co musi spełnić creature w NWN, istnieje i została odtworzona z
czterech niezależnych warstw:

1. oficjalnej specyfikacji creature i `appearance.2da` firmy BioWare;
2. odwróconej specyfikacji binary MDL Torlacka;
3. kodu implementującego resolver creature w xoreos;
4. aktualnie zainstalowanego `nwmain.exe`, który jest rozstrzygający dla
   końcowych bramek instancji i draw.

Najważniejszy wynik:

> Creature `MODELTYPE=S` nie używa osobnego „formatu MDL creature”. Resolver
> bierze `Appearance_Type`, odczytuje `appearance.2da[row]`, pobiera z `RACE`
> resref jednego MDL-a i przekazuje go do wspólnego loadera modelu.

> Po utworzeniu `PartTriMesh` aktualny klient nie wymaga do samego draw ani
> SkinMesh, ani 42 animacji, ani supermodelu. Wymaga włączonej części,
> istniejącego noda, `render != 0` oraz niepustej geometrii.

SkinMesh, szkielet, nazwy stanów, type-5 animation geometry i supermodel są
osobnym kontraktem poprawnego ruchu i zachowania creature. Nie są uniwersalną
bramką pierwszego statycznego piksela.

## 2. Dokładny klient, dla którego odtworzono bramki

```text
path:
  C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\bin\win32\nwmain.exe
version:
  89.8193.37-17
length:
  21071872
lastWriteTimeUtc:
  2026-01-22T20:41:56.8540929Z
sha256:
  3b7cb1252e0edb2ce22d7971f333aade027039ae30a45b4bc64732c3e6bec73a
```

Adresy niżej odnoszą się wyłącznie do tego dokładnego pliku.

## 3. Kontrakt `GIT/UTC -> appearance.2da -> MDL`

### 3.1 Instancja creature

`CNWSArea::LoadCreatures`:

- czyta top-level `Creature List`;
- wymaga StructID `4`;
- ładuje każdy element niezależnie;
- błąd jednego elementu nie przerywa automatycznie ładowania pozostałych.

`CNWSCreature::LoadCreature` odrzuca instancję, gdy
`CNWSCreatureStats::ReadStatsFromGff` zwróci błąd. W aktualnym kliencie jawne
bramki obejmują:

- `Race` poza zakresem;
- `PerceptionRange == 12`;
- istniejący, lecz pusty `ClassList`;
- nieprawidłową albo niespójną klasę;
- brak wymaganej klasy.

Oficjalny dokument BioWare jest bardziej restrykcyjny dla authoringu:

- `Appearance_Type` jest indeksem `appearance.2da`;
- `ClassList` musi mieć od 1 do 3 elementów;
- `PerceptionRange` ma należeć do zakresu 9–13.

Dlatego `PerceptionRange=0` w historycznych generated H2/Hook jest potwierdzonym
naruszeniem oficjalnego kontraktu GFF, nawet jeśli ten exact klient nie zwraca
za zero kodu błędu. Nie wolno przedstawiać tej wartości jako poprawnego
profilu produkcyjnego. Nie jest ona jednak dowodem przyczyny totalnego no-draw,
bo aktualna funkcja odrzuca dokładnie wartość 12, nie 0.

### 3.2 Resolver appearance

Dla `MODELTYPE=P` silnik składa model z części.

Dla `MODELTYPE != P`, w tym `S`, resolver przekazuje do loadera dokładnie:

```text
appearance.2da[Appearance_Type].RACE
```

Oficjalna semantyka `MODELTYPE=S`:

- creature używa pojedynczego MDL;
- tekstura jest pojedynczym TGA albo DDS;
- kolory nie są wybieralne;
- wygląd nie zmienia się od zbroi;
- wyposażona broń nie jest wyświetlana.

Minimalny kontrakt resolution:

1. `Appearance_Type` wskazuje istniejący fizyczny wiersz;
2. `MODELTYPE` ma legalną wartość; dla naszego direct modelu jest to `S`;
3. `RACE` jest niepustym, legalnym resrefem modelu;
4. zasób `RACE.mdl`, Resource Type `2002`, jest osiągalny w faktycznym
   porządku zasobów modułu/HAK;
5. do loadera docierają bajty właściwego zasobu, a nie inny zasób o tym samym
   resrefie.

`PORTRAIT`, `ENVMAP`, `MOVERATE`, `SIZECATEGORY`, `TARGETABLE`,
`PERCEPTIONDIST`, dźwięki, krew, head tracking oraz uzbrojenie mają znaczenie
dla kompletności appearance i zachowania. Nie zastępują jednak rozstrzygającej
pary `MODELTYPE/RACE`.

## 4. Kontrakt binary MDL

### 4.1 Kontener

Binary MDL ma trzy części:

1. 12-bajtowy file header;
2. model/core data;
3. raw vertex data, zwyczajowo nazywane MDX.

Pierwsze DWORD ma wartość `0`, co odróżnia binary MDL od ASCII. Wskaźniki core
są offsetami od początku model data. Wskaźniki raw są offsetami od początku
raw data, a sentinel nieobecnego raw pointera to `0xFFFFFFFF`.

Wymagane są:

- zakresy core/raw bez wyjścia poza plik;
- legalne offsety i array counts;
- istniejący model geometry header;
- root node prowadzący do drzewa nodów;
- legalne content flags nodów;
- legalne zakresy streamów vertex/index.

Routine words z obrazu 32-bitowego nie są trwałym identyfikatorem typu noda.
Właściwym identyfikatorem jest bitmask `content_flags`, między innymi:

```text
Dummy    0x00000001
TriMesh  0x00000021
SkinMesh 0x00000061
```

### 4.2 Semantyczny nagłówek creature

Dla bezpośredniego modelu creature projekt wymaga:

```text
geometryType  = 2   # model geometry
classification = 4 # Character
```

`classification=4` jest prawidłową klasyfikacją dla creature. Placeable także
może używać klasyfikacji `Character`, więc sama klasyfikacja nie rozróżnia obu
resolverów obiektowych.

Model musi mieć:

- niepuste drzewo base geometry;
- co najmniej jeden mesh, który może wygenerować piksele;
- skończone transformacje i skończoną geometrię;
- sensowne bounds/radius, aby culling nie usuwał modelu;
- poprawne mapowanie nazw/partów, jeżeli animacje albo supermodel mają
  sterować nodami.

## 5. Rozstrzygająca bramka TriMesh aktualnego `nwmain.exe`

### 5.1 Instancjowanie

`CreateInstanceTreeR` pod `0x14008DAB0`:

- dla każdego istniejącego noda wywołuje jego wirtualne
  `InternalCreateInstance`;
- rekurencyjnie instancjuje wszystkie dzieci;
- nie sprawdza w tym miejscu SkinMesh, liczby animacji ani supermodelu.

`MdlNodeTriMesh::InternalCreateInstance` pod `0x140091B80`:

- tworzy `PartTriMesh`;
- przepisuje surface/mesh metadata;
- ustawia enabled na podstawie `node.render != 0`.

### 5.2 Renderability

`PartTriMesh::IsPartRenderable` pod `0x1400A2EB0` zwraca true, gdy:

1. `PartTriMesh.enabled != 0`;
2. wskaźnik noda istnieje;
3. `node.render != 0`;
4. `vertexCount != 0` albo istnieje alternatywny vertex-data pointer.

`PartTriMesh::Draw` pod `0x1400A24C0` powtarza te same warunki i przechodzi do
renderera materiału.

Ani `IsPartRenderable`, ani początek `Draw` nie sprawdzają:

- liczby animacji;
- nazw 42 stanów;
- `animationType`;
- SkinMesh;
- obecności szkieletu;
- `supermodel != NULL`;
- vertex colors.

Dla rzeczywiście widocznych pikseli trzeba dodatkowo mieć niezerową legalną
topologię: twarze/indeksy odwołujące się do istniejących wierzchołków oraz
transformację i kamerę umieszczające geometrię w widoku. Sama pozytywna funkcja
`IsPartRenderable` nie gwarantuje, że obiekt znajduje się przed kamerą.

## 6. Czego potrzeba do poprawnego animowanego creature

To jest kontrakt zachowania, nie pierwszego draw:

- geometry root i animation root muszą mapować się na te same nazwy/party;
- kontrolery muszą mieć legalne zakresy i skończone wartości;
- skala nie może zerować, odwracać lub wyrzucać modelu poza widok;
- lokalne animation geometry powinno mieć natywny type `5`;
- creature musi dostarczyć stany używane przez gameplay albo odziedziczyć je
  przez kompatybilny supermodel;
- przy SkinMesh wagi, bone references, bone map oraz inverse bind muszą być
  wzajemnie spójne;
- przy rigid creature mesh może być przypięty do animowanych nodów bez
  SkinMesh; będzie poruszał się segmentami;
- supermodel jest opcjonalny, ale jeżeli jest użyty, jego resource i nazwy
  nodów muszą być kompatybilne.

Korpus retail `c_Direwolf` i `c_Horror` ma kompletne lokalne 42 stany type 5.
CEP `c_kocrachn` może mieć zero stanów lokalnych, ponieważ dziedziczy
kompatybilny `c_Horror`. To są dwie potwierdzone rodziny kompletnego zachowania.
Nie wynika z nich ukryta reguła „dokładnie 42 albo zero pikseli”.

SkinMesh jest wymagany dla ciągłej deformacji ważonej, nie dla samego
istnienia statycznego lub segmentowego creature.

## 7. Co dokładnie spełnia r43

`m2a_h2p43.mdl`:

- przechodzi własny parser oraz niezależny `nwnmdlcomp`;
- ma legalny binary header i legalne core/raw ranges;
- ma `geometryType=2`, `classification=4`, `fog=1`;
- ma jedno niepuste drzewo base geometry;
- ma renderowalny `TriMesh` z `render=1`;
- ma 2678 wierzchołków i 1543 twarze;
- ma legalne indeksy, winding, normals oraz skończone bounds;
- ma legalne streamy MDX;
- ma `supermodel=NULL`;
- ma siedem lokalnych stanów type 0.

Exact `appearance.2da` row 15100 ma:

```text
MODELTYPE=S
RACE=m2a_h2p43
```

Wniosek jest twardy:

> Jeżeli aktualny klient rozwiąże `m2a_h2p43.mdl`, utworzy jego model instance
> i umieści tę instancję w renderowanej scenie, jawne bramki `PartTriMesh`
> dopuszczą ten mesh do draw.

Z tego wynika równie twardo:

> Totalna nieobecność exact r43 nie jest wyjaśniona przez uniwersalny wymóg
> SkinMesh, 42 animacji albo nie-NULL supermodelu. Siedem stanów type 0 jest
> potwierdzonym brakiem kompletności zachowania, lecz aktualny kod klienta nie
> pokazuje, aby było to odrzucenie base TriMesh.

## 8. Dlaczego działający placeable nie rozstrzyga creature

Placeable:

```text
GIT Placeable List
  -> placeables.2da[Appearance].ModelName
  -> MDL
  -> wspólny model/mesh renderer
```

Creature:

```text
GIT Creature List / UTC state
  -> Appearance_Type
  -> appearance.2da[MODELTYPE, RACE]
  -> client creature model attachment
  -> animation/state transforms
  -> wspólny model/mesh renderer
```

Widoczny placeable potwierdza:

- podstawową obsługę binary MDL;
- działający rigid TriMesh writer;
- działające vertex/index/UV/material streams;
- osiągalność przynajmniej jego zasobów w HAK;
- końcowy wspólny renderer.

Nie potwierdza:

- `Creature List` i client creature object;
- creature `appearance.2da`;
- exact resref custom creature MDL;
- attachment modelu do creature;
- creature state/animation transforms;
- poprawnego kadru runtime dla instancji creature.

Brak PWK placeable odpowiada za brak kolizji. Nie ma związku z niewidocznością
creature.

## 9. Pełna lista potwierdzonych problemów bieżącej linii

1. H1 v20 był błędnie promowany z corrupt-draw witness do wzorca poprawności.
2. r43 odziedziczył siedem stanów type 0 i nie stanowi kompletnego,
   poprawnie animowanego creature.
3. Historyczny generated runtime profile zapisuje `PerceptionRange=0`,
   niezgodne z oficjalnym authoring contract BioWare 9–13. Produkcyjny
   `ActiveMonsterBaseline` używa już wartości 11.
4. Custom row r43 jest sparse. Kluczowe `MODELTYPE=S` i `RACE=m2a_h2p43` są
   poprawne, ale row nie jest pełnym klonem semantycznym retail appearance.
5. R43 nie zawierał czystej ćwiartki
   `native-complete profile + native appearance/model`: owner-added Drider
   został celowo przełączony na custom row 15100.
6. Generated Hook Horror używał generated GIT/runtime profile. Jego brak
   obciąża wspólną ścieżkę creature/container/scene, nie custom MDL.
7. Nie zachowano exact runtime capture/log/trace, który odpowiadałby, czy
   `RACE=m2a_h2p43` został rozwiązany, czy model instance powstał i czy creature
   object dotarł do sceny klienta.
8. Static byte audit r43 kończy się pozytywnie na finalnym predykacie
   TriMesh. Nierozstrzygnięta granica leży przed nim albo w scenie/kamerze.

## 10. Rozstrzygnięcie

Kontrakt MDL dla pierwszego statycznego draw creature jest poznany.
Kontrakt kompletnego animowanego creature także jest poznany na poziomie
strukturalnym i został zaimplementowany offline w nowym profilu
SkinMesh/weighted skeleton/type-5 states.

Niepoznany pozostaje wyłącznie dynamiczny fakt dotyczący jednego runu r43:

```text
czy resolver zwrócił exact MDL
-> czy powstała CAuroraModel/model instance
-> czy instancja została podłączona do client creature
-> czy creature znalazł się w renderowanej scenie i kadrze
```

Bez tego śladu nie wolno zmieniać kolejnego headera MDL i nazywać go root
cause. Taka zmiana byłaby strzałem, ponieważ exact r43 już spełnia poznane
statyczne bramki draw.

Istniejący czysty test kontenera, który nie tworzy nowej iteracji modelu:

```text
MOD filename:
  m2a_van02.mod
Toolset module name:
  Meshy2Aurora creature comparison
Area:
  Meshy2Aurora M0 binary vertical-slice area
appearance/model:
  stock c_horror through complete cloned appearance row 15219
```

Wynik tego exact stock-only testu rozdziela błąd wspólnego
GIT/container/scene od błędu custom `appearance -> MDL`.

## 10A. Uzupełnienie exact SkinMesh r45

Po właścicielskim wyniku `m2a_h2r45.mod = not_visible/failed` przebadano także
skinned draw aktualnego klienta.

`PartSkin::Draw` pod `0x1400DEB40` używa wspólnego
`PartTriMesh::IsPartRenderable` pod `0x1400A2EB0`. Exact r45 ma:

```text
Part.enabled source = mesh.render = 1
vertex count        = 2678
Skin palette usage  = 22 z implementowanych 64
```

Niezerowa liczba wierzchołków oznacza, że alternatywny runtime resource pod
`MdlNodeTriMesh+0x278` nie jest wymagany. Loader pomija źródłowe pole
`node+0x264`, więc jego wartość także nie stanowi kontraktu draw.

Jeżeli exact model zostanie rozwiązany, instancja zostanie utworzona i SkinMesh
zostanie dołączony do sceny, te jawne bramki nie odrzucą r45. Aktualna
nierozstrzygnięta granica leży wcześniej:

```text
appearance row -> RACE resource -> binary model load
-> model instance -> client creature attachment -> scene submission
```

Pełny wynik:
[Audyt exact r45 po wyniku „dalej pusto”](audyt-r45-dalej-pusto-w-nwn-2026-07-26.md).

## 11. Źródła

- [BioWare Creature Format, immutable mirror](https://github.com/xoreos/xoreos-docs/blob/4e1c197aa09b532ef466ff8ceccfd6221e80c3c9/specs/bioware/Creature_Format.pdf)
- [Torlack NWN Binary Model Files Basics, immutable mirror](https://github.com/xoreos/xoreos-docs/blob/4e1c197aa09b532ef466ff8ceccfd6221e80c3c9/specs/torlack/binmdl.html)
- [xoreos creature resolver, immutable source](https://github.com/xoreos/xoreos/blob/91d5b40a463a8627adccdfa991d61ca09f18d37d/src/engines/nwn/creature.cpp#L556-L603)
- `C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\bin\win32\nwmain.exe`
  — read-only export/disassembly audit exact SHA-256 podanego w rozdziale 2.
- [Audyt systemowy r43](audyt-systemowy-dlaczego-generowane-creature-nie-dzialaja-w-nwn-2026-07-26.md)
- [Owner-bound wynik r43](evidence/h2-r43-owner-added-native-control-result-2026-07-26.json)

## 12. Aktualizacja po działającym V4 — 2026-07-28

Sekcje r43/r45 powyżej pozostają historycznym zapisem diagnozy. Nie opisują
już najwyższej potwierdzonej linii produktu.

Właściciel potwierdził w NWN exact `tlcpowdemo4.mod` +
`tlcpowhak4.hak`: model jest widoczny, sekwencja śmierci przechodzi i nie ma
skoku wejściowego. V4 zamyka dynamiczny brak wiedzy dla własnej, dokładnie
związanej linii; nie przepisuje wstecz wyników r43/r45.

Po tym wyniku wdrożono bramki V2 dla lineage, przejść, root motion i timingów
eventów oraz rozdzielono produkcyjny HAK od fixture MOD/UTC. Szczegóły:
[spłata długu pipeline creature](creature-pipeline-debt-paydown-2026-07-28.md).
