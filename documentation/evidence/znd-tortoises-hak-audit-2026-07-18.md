# Audit referencyjnego HAK: ZND Tortoises (2026-07-18)

## Cel i granica

Cel: odczytac in-place strukture HAK-a wskazanego przez wlasciciela jako
niezalezny, publicznie udostepniony punkt odniesienia dla problemu "model
widoczny w Toolsecie, niewidoczny w kliencie NWN".

Wejscie (read-only, nie zostalo skopiowane do repozytorium ani do proofu):

`C:\Users\enonw\Downloads\znd_tortoise\hak\ZND_Tortoise.hak`

SHA-256 calego wejscia:

`3f85946f1e86e20b4d9c6fae311f214587da4009080b64376711f630b0327021`

Provenance: [ZugothNDeadly's Tortoises on Neverwinter Vault](https://neverwintervault.org/project/nwn1/model/zugothndeadlys-tortoises).
Zasoby zewnetrzne pozostaja wylacznie materialem referencyjnym; nie wolno ich
kopiowac do implementacji, fixture ani HAK-a Meshy2Aurora.

## Fakty z odczytu kontenera

- Kontener jest `HAK V1.0`, ma 18 zasobow i 85 620 933 B.
- Zawiera dokladnie: `appearance.2da` (1), ASCII `MDL` (6) oraz `TGA` (11).
  Nie ma `anim.2da`, `txi`, MDX ani innych pomocniczych 2DA.
- `appearance.2da` ma szesc dodanych wierszy: 15077--15082. Kazdy wskazuje
  odpowiednio na `c_tortoise0`--`c_tortoise5` jako `NAME`, a `MODELTYPE` ma
  wartosc `S`.
- Kazdy wpis MDL ma tekstowy naglowek `#MAXMODEL ASCII`, jest oznaczony jako
  wygenerowany przez CleanModels 3.7.0 i nie jest skompilowanym binary MDL.
- Kazdy model zawiera `classification CHARACTER`,
  `setsupermodel c_tortoiseN NULL` oraz `setanimationscale 1`.
- Kazdy wariant jest samowystarczalny: ma 48 wezlow geometrii statycznej
  (6 `dummy`, 41 `trimesh`, 1 `skin`) oraz 44 nazwane animacje. Wsrod nich sa
  `cpause1`, `cwalk`, `crun` i `cappear`.
- Wlasciwy `skin` ma `render 1`, `shadow 1`, `alpha 1`, teksture
  `bitmap c_tortoiseN` i jedna wage na wierzcholek. Warianty 0/1/3/5 maja
  1 752 wierzcholki i 3 409 faces; 2 ma 2 024/3 894, a 4 ma 2 107/3 907.
- Tekstury glownych modeli to nieskompresowane 32-bit TGA: warianty 0, 2 i 5
  maja 512x512, a 1, 3 i 4 maja 1024x1024. Pary `_n` istnieja dla 0, 1, 2, 3
  i 5; `c_tortoise4_n.tga` nie istnieje.
- Zadna z szesciu definicji MDL nie ma dyrektywy `bumpmap` ani `normalmap`.
  Brak `c_tortoise4_n.tga` nie jest wiec brakujaca zaleznoscia tego modelu.

## Wniosek dla aktualnego problemu runtime

To jest mocny, ale ograniczony comparator z realnego pakietu postaci. Pokazuje,
ze kompletne minimum po stronie zasobow dla takiego modelu moze pozostac bardzo
proste: `appearance.2da` + nazwa MDL + odpowiadajaca tekstura. Nie ma tu
ukrytego `anim.2da` ani normal mapy, ktorych brak sam przez sie wyjasnialby
niewidocznosc naszego H1.

Jednoczesnie referencja ma trzy istotne roznice wobec aktualnego, natywnie
binarnie generowanego H1 Meshy2Aurora: jawna klasyfikacje i self-supermodel,
pelna hierarchie szkieletu/skin oraz kompletny zestaw 44 animacji. Dlatego nie
mozna jeszcze odrzucic hipotezy o kontrakcie animacji albo hierarchii; nie wolno
tez uznac pakietu za proof w kliencie wlasciciela, bo w tej czynnosci go nie
instalowano ani nie uruchamiano.

## Wybrane nastepne podejscie

Porownac semantyke wlasnego H1 binary MDL z powyzszym referencyjnym kontraktem
w naszym readerze/writerze: klasyfikacje, supermodel, korzen, skin, flagi
renderingu, material/bitmap oraz wymagane nazwy i dane animacji. Dopiero po
zamknieciu tych roznic mozna przygotowac nowy, wlasny asset i wykonac oddzielny
proof Aurora Toolset, a nastepnie NWN na drugim ekranie.

## Wynik i pozostale ryzyko

Audit kontenera zakonczony. Nie zmodyfikowano HAK-a, konfiguracji gry,
instalacji Aurora/NWN ani zadnego assetu proof. Pozostaje otwarte pytanie,
ktory z kontraktow binary MDL rozni sie od runtime NWN; sam odczyt tego
zewnetrznego ASCII MDL nie jest jeszcze dowodem akceptacji naszego writera.

## Dodatkowa obserwacja wlasciciela

Po audycie wlasciciel podlaczyl powyzszy, niezmieniony HAK i potwierdzil, ze
model dziala w jego runtime NWN. Jest to obserwacja wykonana przez wlasciciela,
nie screenshot/proof wykonany przez Codex. Jej wartosc diagnostyczna jest
jednak jednoznaczna: w tym samym srodowisku dziala podpiecie HAK-a,
`appearance.2da`, wpis `MODELTYPE S` i ASCII model postaci ze zdefiniowanymi
teksturami. Hipotezy ograniczajace problem H1 do samego ladowania HAK-a albo
samego wpisu appearance sa przez to oslabione; w centrum pozostaje semantyka
wygenerowanego native binary MDL i jego zgodnosc z kontraktem runtime.

## Shortlista dalszego corpus referencyjnego z Neverwinter Vault

Ponizsze projekty zostaly wyszukane jako read-only kandydaci. Zaden nie zostal
jeszcze pobrany, zainstalowany ani wlaczony do proofu Meshy2Aurora.

1. [Skinmesh Animals](https://neverwintervault.org/project/nwn1/model/skinmesh-animals)
   (NWN1, Ancarion): 37 zwierzat, w tym squirrel i turtle, oraz osobny plik z
   liniami `appearance.2da`. To najlepszy nastepny control dla prostego
   skinned creature z kompletnego HAK-a. Komentarze projektu rejestruja tez
   realny przypadek uszkodzonej/nieprawidlowo skonwertowanej tekstury jako
   zrodla bledow Toolsetu, wiec nalezy osobno sprawdzic pary MDL--tekstura.
2. [NWN2 Creature conversion for NWN1](https://neverwintervault.org/project/nwn1/model/nwn2-creature-conversion-nwn1):
   autor odnotowal dwa bezposrednio relewantne bledy po konwersji -- zbyt
   duzo bones per skinmesh powodowalo crash przy wychodzeniu z Toolsetu, a
   gargoyle potrzebowal brakujacych dummy helper nodes. Pakiet jest najlepszym
   referencyjnym przypadkiem do testow invariantow skin/bones/hierarchy.
3. [The Witcher 1 Geralt models 4--6](https://neverwintervault.org/project/nwn1/model/witcher-1-geralt-models-4-6-ravens-armor-nwn1-ee):
   modele sa eksportowane bez wlasnych animacji i uzywaja `a_ba` jako
   supermodelu. To wyizolowany control dla tezy, ze model moze byc poprawnie
   widoczny, gdy animacje dostarcza zewnetrzny supermodel zamiast lokalnego
   zestawu animacji.
4. [The Witcher 1 Child (Alvin)](https://neverwintervault.org/project/nwn1/model/witcher-1-child-alvin-nwn1-ee):
   rigowany w Blenderze przez Neverblender na szkielecie `c_yKid_m`; opis
   wprost rozroznia skinmesha od low-poly pseudobones uzytych dla cieni. Autor
   raportuje, ze `shadow` wlaczony na wysokopoligonowym skinmeshu crashuje
   engine, a wylaczenie go na skinie pozwala modelowi dzialac.
5. [The Witcher 2 Harpies](https://neverwintervault.org/project/nwnee/model/witcher-2-harpies-nwn1-ee):
   zawiera HAK, modul demonstracyjny i test LOD. Dokumentuje zarazem znaczenie
   `MODELTYPE` (`S` zostal zmieniony na `L`, aby pokazac bron) oraz granice
   liczby bones -- rig jest zbyt duzy dla 1.69. To kandydat EE-only, nie
   pierwszy control dla malego modelu H1.
6. [CleanModels3](https://neverwintervault.org/project/nwn1/other/cleanmodels3)
   nie jest corpus-modelem, ale opisuje konkretne kontrakty, ktore warto
   odtworzyc w naszych invariantach: skinmesh parentowany do model base z
   pozycja/orientacja zero, poprawne listy wag i limit bones per skinmesh.

Decyzja robocza: zaczac od punktu 1, nastepnie punktu 2. To daje najpierw
porownywalny maly creature, a potem celowe przypadki awarii; pakiety Witchera
zostawic jako kontrole granic supermodelu, cieni i wysokiej liczby bones.
