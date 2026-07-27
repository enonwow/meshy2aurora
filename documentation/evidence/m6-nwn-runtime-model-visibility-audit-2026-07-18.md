# Audyt braku widocznosci modelu w NWN (2026-07-18)

Status: `ROOT CAUSE NIEZAMKNIETE. v19 pokazuje, ze sama obecnosc siedmiu
podstawowych nazw klipow nie rozwiazuje braku widocznosci. v20 ma ten sam
profil animacji i renderuje jako RIGID, wiec animacja nie jest globalnym
gate'em ladowania/draw. Pozostaje otwarta interakcja animacja -> kosci ->
skin. Audyt Aurora First z 2026-07-18 nie potwierdza bledu indeksowania
slotow skina ani kolejnosci skinowego quaternionu; aktywna pozostaje
semantyka deformacji/renderera.`

## Objaw

`m2a_m6p01` jest odczytywany i pokazywany przez Toolset/Aurora, lecz model nie
jest widoczny w sesji NWN. Ten audyt rozdziela walidacje wlasnego writera i
Toolsetu od rzeczywistej akceptacji renderera gry.

## Dowody wykluczajace dolne warstwy lancucha

| Warstwa | Wynik |
| --- | --- |
| MOD / HAK link | `m2a_test.mod` i `m2a_codex_aproof.mod` maja dokladnie `Mod_HakList[0].Mod_Hak = m2a_codex_aproof`. Reczny MOD Toolsetu takze nie renderuje modelu w grze. |
| `appearance.2da` | Wiersz `15100` ma `MODELTYPE=S` i `RACE=m2a_m6p01`; to jest poprawny direct-creature mapping, nie przesuniecie kolumn. |
| HAK | `m2a_codex_aproof.hak`, SHA-256 `da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756`, zawiera `appearance` (2017), `m2a_m6p01` (2002) oraz `m2a_m6t01` (3). |
| MDL/MDX range | Model ma `204312` B, core `120260` B i volatile/MDX `84040` B; `12 + core + MDX` konczy sie dokladnie na EOF. Wlasny reader nie zglasza diagnostyk. |
| TGA | `m2a_m6t01` jest niekompresowanym TGA true-color `2048x2048`, 24 bpp (SHA-256 `ab8f11c7f448801d905594802a223a1ed5969bc5d09ce06e6588cbe83b6a86b4`). |

Wniosek: brak widocznosci nie jest wyjasniany przez brak wpisu HAK, zly resref
w 2DA, nieobecny payload MDL/tekstury ani przez wczesniej wykryte rozjazdy
generatora MOD-a. Szczegoly porownania MOD-ow sa w
`m2a-test-vs-generated-module-comparison-2026-07-18.md`.

## Rozjazd wygenerowanego modelu od dzialajacego direct creature

Lokalny model referencyjny `c_squirrel` (read-only `cep3_core1.hak`) ma:

- 43 klipy animacji, w tym `cappear`, `cpause1`, `cwalk`, `crun`, `cdead` i
  stany walki;
- `animationRoot=c_squirrel` dla kazdego z tych klipow;
- deklarowany zakres modelu `[-5,-5,-1]..[5,5,10]` i promien `7`.

Wygenerowany `m2a_m6p01` (SHA-256
`2f8c4eb0bdedb53955c9a4404796ffb811d53830202cea3514b82003858197bd`) ma:

- tylko jeden klip: `cpause1`, o dlugosci `4.0333333 s`, z
  `animationRoot=Hips`;
- brak `cappear`, ruchu, przejsc i stanow wymaganych przez zwykly lifecycle
  stworzenia NWN;
- bardzo maly deklarowany zakres `[-0.65,-0.24,0]..[0.65,0.24,1.70]` i promien
  `1.70`.

Fakty te sa zgodne z wczesniejsza obserwacja Toolsetu, ze model byl bardzo
maly. Semantyka promienia dla cullingu nie jest jeszcze potwierdzona kotwica
Aurora, dlatego ma status hipotezy. Niepelny zestaw animacji jest natomiast
bezposrednim, potwierdzonym rozjazdem kontraktu z dzialajacym direct creature:
gra moze przy spawnie i zmianie stanu zadac `cappear` lub innego klipu,
ktorego wygenerowany asset nie posiada.

## Dlaczego zielony readback nie wystarcza

`materialization-report.json` dla aktualnego modelu ma pusty `semanticDiff`,
ale ten sam raport jawnie oznacza jako `OPEN_M6` pola, ktorych nie
zweryfikowano w NWN:

- runtime model/node/mesh fields;
- topologie faces i mesh tail fields;
- aktywne sloty skin, stale kosci, kolejnosc quaternionu i runtime deformation;
- zeroowane pola opaque animacji, animroot, drzewa klipu i routing stanow.

Dlatego wlasny reader potwierdza tylko, ze writer i reader zgadzaja sie ze
soba. Toolset pokazuje statyczny odczyt. Zadna z tych bramek nie dowodzi, ze
renderer gry przyjal runtime profile.

## Log gry

`nwengineLog.txt` potwierdza uruchomienie `m2a_test` o `00:29:33`, ale nie
zawiera komunikatu o braku `m2a_m6p01`, MDL, MDX ani tekstury. Wystepuja
`Empty field label` dla serializowanego `CURRENTGAME` oraz ostrzezenia
`Invalid Class`; nie wskazuja one zasobu modelu i nie sa dowodem jego odrzucenia.

## Werdykt

Najbardziej prawdopodobna przyczyna braku widocznosci lezy w **nieukonczonym
runtime profilu binarnego MDL**, przede wszystkim w niepelnej maszynie stanow
animacji (`cpause1` bez `cappear` i pozostalych klipow), a potencjalnie takze w
niepotwierdzonych polach skin/mesh i zbyt malym model bounds/radius. Nie ma
jeszcze dowodu pozwalajacego wskazac jeden offset jako finalny root cause.

## Kolejny test rozstrzygajacy

Przed zmiana writera trzeba dodac test/gate, ktory dla profilu direct creature
wymaga co najmniej animacji spawn/idle/ruchu oraz zgodnej nazwy `animroot`, a
nastepnie wykonac nowy proof w grze. Osobny wariant powinien uzyc bezpiecznych
referencyjnych bounds/radius dla profilu creature. Dopiero obraz/capture z NWN
po takim przebiegu moze zamknac `RUNTIME-MODEL-ROOT-CAUSE`.

## Uzgodnienie sentinela supermodelu (wariant v15)

**Fakt z lokalnego binary reference:** samodzielny direct creature
`c_squirrel` zapisuje w naglowku modelu sentinel braku supermodelu jako
uppercase `NULL`. Wygenerowany wariant v14 zapisywal lowercase `null`.

**Wniosek implementacyjny:** writer zapisuje teraz dokladnie `NULL`, a own
semantic readback oraz zamrozone hashe artefaktu zostaly odpowiednio
zaktualizowane. To usuwa niepotrzebna roznice od natywnego modelu referencyjnego
i jest objete testem deterministycznym.

**Pozostale ryzyko:** dekompilacja potwierdza parser `setsupermodel`, ale ten
pojedynczy string nie ma jeszcze runtime proofu jako samodzielna przyczyna
niewidocznosci. Wymaga instalacji wariantu v15 i obserwacji w NWN wraz z
uprzednio dodanym profilem `cappear`/`cpause1`/`cwalk`/`crun`.

## Uzgodnienie root i animroot (wariant v16)

**Fakt z lokalnego binary reference:** `c_squirrel` ma zgodne nazwy modelu,
root geometry oraz `animroot` kazdego klipu. W poprzednich artefaktach Meshy
root i `animroot` byly `Hips`, podczas gdy resref modelu to `m2a_m6p01`.

**Wniosek implementacyjny:** po konwersji z GLB pipeline normalizuje nazwe
jednego root node oraz `animroot` wszystkich wlasnych klipow do
`m2a_m6p01`. Nie zmienia to identyfikatorow nodow, wag, keyframes ani
zrodlowego GLB. Bezposredni odczyt binary v16 potwierdza: model/root/
`animroot` = `m2a_m6p01`, `supermodel = NULL`, cztery klipy i puste
`semanticDiff`.

**Pozostale ryzyko:** to nadal hipoteza kompatybilnosci runtime, nie proof
widocznosci. Rozstrzygajacy krok pozostaje ten sam: instalacja v16 HAK/MOD i
obserwacja modelu w NWN.

## Uzgodnienie bounds/radius (wariant v17)

**Fakt z lokalnych binary references:** `c_squirrel`, `c_vampire_f`,
`c_kocrachn`, `c_phod_horror_b`, `c_eye` i `c_nulltail` maja identyczne
model-level bounds `[-5,-5,-1]..[5,5,10]` i radius `7`, niezaleznie od
rozmiaru payloadu, liczby animacji i animation scale. Wcześniejszy model
Meshy zapisywal ciasne bounds geometrii i radius ok. `1.7`; Toolset byl
obserwowany jako pokazuacy go bardzo malego.

**Wniosek implementacyjny:** profil `M4_DIRECT_CREATURE_EXTENDED64_V1`
zapisuje potwierdzony native culling envelope dla naglowka modelu, zachowujac
source-derived bounds na poziomie mesh. Nie skaluje geometrii, skin weights,
keyframes ani zrodlowego GLB.

**Weryfikacja i ryzyko:** binary v17 odczytuje dokladnie `[-5,-5,-1]`,
`[5,5,10]`, `radius=7`, model/root/animroot `m2a_m6p01`, `NULL`, cztery klipy
i puste `semanticDiff`. Nadal brakuje jedynego rozstrzygajacego dowodu:
uruchomienia v17 w NWN.

## Native proof v18/v19 (2026-07-18)

**Fakt z proofu:** v18 zostal uruchomiony standardowa trasa Toolset
`Open Module -> View Area -> Build/Test Module`; Toolset i NWN byly na
`\\.\DISPLAY1` (`primary=false`). Log silnika zawiera `Loading Module:
m2a_codex_aproof`. H1 zostal celowo przesuniety z `(10,10)` na `(14,10)`, aby
nie nakladal sie na spawn gracza Test Module. Kadr NWN nadal pokazuje tylko
postac gracza, bez widocznego modelu Meshy:
`proof-output/meshy-h1-nwn-runtime-isolation-proof-v18/live/nwn-test-module.png`.

**Wniosek implementacyjny:** hipoteza o samej kolizji H1 z player-start nie
wyjasnia objawu. Zmiana pozycji pozostaje w fixture proofu i nie zmienia
geometrii, skinu, tekstury ani binarnego writera MDL.

**Fakt z lokalnego kontraktu:** `aurora-mdl-format-codex.md` wskazuje dla
direct creature minimum `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel`
i `cdead`; wariant v18 mial tylko spawn/idle/ruch. Wariant v19 dodaje trzy
brakujace clean-room aliasy dostarczonego `cpause1`, wiec emituje dokladnie
siedem klipow: `cappear`, `cpause1`, `cwalk`, `crun`, `ca1slashl`,
`cdamagel`, `cdead`.

**Weryfikacja:** regresje profilu i packetu przeszly (`model_pipeline` 7/7),
a HAK/MOD v19 zostaly zainstalowane po zgodnym hash-check. Pelny native proof
ponownie potwierdzil w Toolsecie modul, obszar `m2a_caproof_area`, live menu
`Build`/pozycja `2`/ID `121` oraz start NWN na drugim ekranie. Kadr gry nadal
nie pokazuje H1:
`proof-output/meshy-h1-nwn-runtime-animation-gate-v19/live/nwn-test-module.png`.

**Werdykt:** v19 jest waznym, lecz nieudanym runtime attemptem. Nie wolno
twierdzic, ze model jest widoczny ani ze animacja gra. Brak H1 po poprawnym
zaladowaniu MOD/HAK, izolacji od player-start i komplecie minimalnych stanow
przesuwa kolejny Aurora First audit na layout runtime geometry/skin/mesh
binary MDL, a nie na workflow Toolsetu, kontener MOD lub nazwy podstawowych
klipow.

## RIGID skin-isolation v20: widocznosc runtime, ale uszkodzona geometria (2026-07-18)

**Cel A/B:** v20 zachowuje ten sam GLB H1, material, teksture, HAK, MOD i
fixture proofu co v19, ale emituje jeden zwykly `trimesh` (`RIGID`) bez nodu
`skin`, bez aktywnych kosci i bez tablic runtime skin. Wlasny readback
potwierdza `1334` wierzcholki, `1556` trojkatow, `RIGID` i `0` aktywnych
jointow. To jest diagnostyka, nie produktowy fallback.

**Pakiet i bramki:**

- model: SHA-256 `6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd`
  (`557268` B);
- HAK: SHA-256 `6692e83f6f398609085db5e25bfbc5a8ddca20f58561fa68d72b33a47259d4e7`;
- MOD: SHA-256 `34838f87f65b89c4e05057acfe04531c4f39c6a3675902558de47958f2777fc1`;
- Toolset pokazal H1 jako czytelna sylwetke w obszarze proofu:
  `proof-output/meshy-h1-nwn-runtime-rigid-isolation-v20/toolset-h1-visible-rigid-v20.png`
  (SHA-256 `7ae6daca6ce05ae758b14a70218507cc324ebda1dfaa9a62d67af62e58ad0bde`);
- standardowe `Build -> Test Module` uzylo zywego ID menu `121`; logi serwera
  i klienta potwierdzaja `Loading Module: m2a_codex_aproof`;
- klient uruchomil sie na `\\.\DISPLAY1` zgodnie ze standardem.

**Fakt z runtime:** kadr poczatkowy nie obejmuje H1
(`nwn-rigid-v20-initial.png`, SHA-256
`24691f23da215eea95a3bb9a96877cf99806c8909ec8f149c1209603ccc97c85`). Po
standardowym, krotkim powrocie postaci w okolice instancji H1 widoczna jest
duza, wyraznie zdeformowana/poszarpana figura w miejscu proofowej instancji:
`nwn-rigid-v20-return-start.png` (SHA-256
`4db5f1a5e13f29e7619533507fed340d068192083db28a17dea7f5568c1f31c1`).

**Wniosek implementacyjny:** engine renderuje payload RIGID, zatem poprzednia
hipoteza, ze sam brak `skin` albo lifecycle animacji calkowicie blokuje
widocznosc, jest odrzucona. Jednoczesnie obraz nie jest poprawnym humanoidem,
wiec ten wynik nie zamyka celu jakosciowego ani nie dowodzi poprawnej geometrii.
Najsilniejszym obecnym kandydatem jest wspolny runtime layout mesha/MDX
(strumien wierzcholkow, indeksy/twarze, offsety albo transformacja), ktory
Toolset toleruje inaczej niz renderer gry.

**Kolejny krok Aurora First:** zestawic kazde pole wspolnej sciezki `trimesh`
w naszym `write_binary_mdl.rs` z odczytem common-mesh w dekompilacji Aurory;
nie wprowadzac poprawki na podstawie wygladu ani ASCII NeverBlendera. Osobny
record referencyjny dodatku Blender jest w
`m6-neverblender-reference-comparison-v1-2026-07-18.md`.

## Aurora First: audyt loadera i deformacji skina (2026-07-18)

### Zakres i zrodlo

To jest audyt read-only dekompilacji
`C:\Projects\New Folder\export\decompiled_all.c`, SHA-256
`36bb8b1031afe2abf23f0e18180a5ad649401d9ea5e078e5170a31a96167c572`.
Badany artefakt H1 to nieudany proof v19:
`proof-output/meshy-h1-nwn-runtime-animation-gate-v19/generated/m2a_m6p01.mdl`,
SHA-256 `625974fda868edd9631a43341c5238669ff39e210bb50f10705ffb6cfec2f38f`.
Nie uruchamiano Toolsetu ani NWN i nie zmieniano zadnego assetu.

### Fakty z dekompilacji Aurora

1. `FUN_00a3dd4c` laduje skin. Po wspolnym loaderze mesha
   `FUN_00a3e19c` kopiuje z MDX po `16` bajtow wag i `8` bajtow referencji na
   vertex. Nastepnie, gdy `mapCount` (`node + 0x288`) jest niezerowy, kopiuje
   forward map oraz tablice `qBoneRefInv`, `tBoneRefInv` i constants.
2. `FUN_00a3f188` kopiuje dokladnie `0x80` bajtow inline mappingu z
   on-disk `node + 0x2b0` do runtime `+0x2b4`.
3. `FUN_00a932cc` iteruje zawsze po wszystkich 64 pozycjach tej mapy.
   Odczytuje kolejno ordynal kosci z inline mapy, pomija `-1`, a dla indeksu
   mieszczacego sie w liczbie `tBoneRefInv` laczy aktualny transform kosci z
   `tBoneRefInv` i `qBoneRefInv`, po czym wysyla palete do renderera.
4. Wzory w `FUN_00a932cc` potwierdzaja kolejnosc skinowego quaternionu
   `WXYZ` zapisana przez writer. Nie ma podstaw do odwracania jej na `XYZW`.
5. `FUN_00a92490` (sciezka budowania danych skina) tworzy odwrotnosc relacji:
   raw reference wskazuje kompaktowy slot, a inline mapping tego slotu wskazuje
   ordynal nodu w drzewie. To jest ten sam kontrakt, ktory realizuje writer.

### Odczyt binarny H1 v19

Niezalezny od own-readback odczyt bajtow H1 daje:

| Wlasciwosc | Wynik |
| --- | --- |
| reachable nodes / vertices | `25` / `1334` |
| map, q, t, constants count | `25`, `25`, `25`, `25` |
| uzyte sloty raw refs | dokladnie `0..21` |
| dodatnie lane wagi | `3635`; zaden nie wskazuje `0xffff`, slotu `>=64` ani pustej/nieistniejacej pozycji inline |
| inline mapping dla aktywnych slotow | dokladnie ordynaly `0..21` |
| nieuzyte referencje `0xffff` | `1701`, wylacznie przy niedodatniej wadze |
| suma wag na vertex | od `0.9999999553` do `1.0000000382` |
| wartosci `q/t` | skonczone |

Zatem dla v19 nie ma binarnego dowodu na zly slot, zla wartosc sentinela dla
dodatniej wagi, zly zakres inline mappingu ani zla kolejnosc skinowego
quaternionu.

### Porownanie z referencja zewnetrzna

CleanModelsowy `incaxje.mdl` ma inny wariant: `mapCount=qCount=tCount=0`, ale
ma 64-elementowy inline mapping i raw refs w slotach `0..21`. Loader Aurory
ma jawna galez dla zerowych countow, wiec roznica sama w sobie nie jest
dowodem blednego layoutu H1. Nie jest tez proofem runtime: bieacy ASCII A/B
nie przeszedl bramki widocznosci w Toolsecie i NWN nie zostal uruchomiony.

### Wniosek i nastepny krok

**Odrzucone dla v19:** poprawka polegajaca jedynie na przestawieniu slotow,
usuwaniu `0xffff` z lane o zerowej wadze lub zamianie `WXYZ` na `XYZW` bylaby
zgadywaniem sprzecznym z dekompilacja.

**Otwarte:** dekoder i renderer wykorzystuja dla aktywnych kosci rzeczywiste
wartosci inverse-bind `q/t`. Kolejny test musi zatem porownac znaczenie tych
wartosci oraz transformacje wspolnej sciezki mesh/MDX z native binary
reference, a nie tylko ich offsety, count i skonczenosc. Wymagany jest nowy
gate przed zmiana writera oraz pozniejszy proof Toolset -> NWN wedlug
istniejacego standardu.

### Uzgodnienie wspolnej sciezki mesh/MDX

`FUN_00a3e19c` kopiuje pozycje (`12` B na vertex), UV (`8` B) i normale
(`12` B) z MDX oraz rozwija indeksy na podstawie `indexCounts/indexOffsets`.
`FUN_00a61538` pracuje potem na tych indeksach i buduje przyleglosci twarzy.
`FUN_00a5e508` odrzuca node renderingu dopiero gdy jednoczesnie nie ma
vertices i nie ma geometrii runtime.

Niezalezny od own-readback odczyt H1 v19 potwierdza `1334` vertices,
`1556` faces i `4668` raw indeksow. Wszystkie wartosci POSITION/UV/NORMAL sa
skonczone; wszystkie indeksy naleza do `[0,1334)`, dokladnie zgadzaja sie z
trzema indeksami kazdej twarzy core, a dlugosci normalnych mieszcza sie w
`[0.9999999295, 1.0000001184]`. `meshType=3`, `render=1`, `shadow=1`, jedna
warstwa UV i sentinel colors `-1` sa zgodne z obserwowanymi native skin
nodes.

To odrzuca dla v19 hipotezy o braku strumienia, OOB indexach, rozjezdzie
core-face/raw-index albo zerowym draw gate. Nie dowodzi jeszcze poprawnej
semantyki runtime: pozostaja inverse-bind/deformacja i pozostalosc sciezki
renderera po przygotowaniu geometrii.

## Ponowny audyt appearance.2da i zaleznosci 2DA (2026-07-18)

### Aktywny pakiet

Zainstalowane pliki sa bitowo rowne audytowanemu wariantowi ASCII A/B:

| Plik zainstalowany | SHA-256 | Zgodnosc |
| --- | --- | --- |
| `Documents\Neverwinter Nights\hak\m2a_codex_aproof.hak` | `79f19df7391810efb2cec8ff5d2d058c3882436badcfc3107f6b4059d4ee029c` | identyczny z `package-ascii` |
| `Documents\Neverwinter Nights\modules\m2a_codex_aproof.mod` | `9af06d6a3155a1043bd0343a23164d7ba88a9e30fef3e0685b780ae666fbb1cd` | identyczny z `package-ascii` |

HAK zawiera wymagane zasoby: `appearance` type `2017`, modele type `2002` i
tekstury type `3` dla poprzedniego H1 oraz dla `incaxje`. IFO MOD-a ma jeden
`Mod_HakList/Mod_Hak=m2a_codex_aproof`.

### Wiersze appearance

Tabela ma 35 kolumn, a oba wiersze maja dokladnie 35 komorek:

| Physical row / UTC | LABEL | MODELTYPE | RACE | MOVERATE | pozostale istotne wartosci |
| --- | --- | --- | --- | --- | --- |
| `15100` / recznie utworzony `m2a_test.mod` | `M2A_M6_PROOF` | `S` | `m2a_m6p01` | `NORM` | `BLOODCOLR=R`, `WEAPONSCALE=1.0`, `PORTRAIT=****`, `SIZECATEGORY=4` |
| `15101` / aktualny `m2a_codex_aproof.mod` | `INCAXJE_NEVERBLENDER` | `S` | `incaxje` | `NORM` | te same wartosci |

Aktualny UTC ma `Appearance_Type=15101`, `Phenotype=0` i wskazuje ten
fizyczny wiersz. Recznie utworzony `m2a_test.mod` wskazuje
`Appearance_Type=15100` w tym samym HAK-u.

### Korekta znaczenia recznego MOD-a

Reczny MOD jest kontrola tylko dla kontenera i instancji: potwierdza, ze
Toolset potrafi utworzyc modul, obszar i placed creature z danym
`Appearance_Type`, a MOD laduje ten HAK. **Nie jest niezaleznym testem
`appearance.2da`**, poniewaz nie tworzy drugiej tabeli ani nie zastępuje jej
zawartosci; korzysta z dokladnie tego samego zasobu `appearance` w HAK-u.
Nie wolno na tej podstawie uznac `appearance.2da` za wykluczone runtimeowo.

Audyt strukturalny nadal potwierdza tylko: 35 komorek na wiersz, zgodny
physical row, `MODELTYPE=S` i resref obecny jako MDL w tym samym HAK-u. Nie
potwierdza niezaleznie, jak gra interpretuje ten wiersz po zaladowaniu HAK-a.

### Czy brakuje innej 2DA?

Nie ma obecnie nierozwiazanej, niestandardowej zaleznosci 2DA:

- `appearance.2da` jest jedyna tabela potrzebna do wyboru direct modelu przez
  `MODELTYPE=S` i `RACE=<model-resref>`.
- `MOVERATE=NORM`, `BLOODCOLR=R`, size `4` i phenotype `0` odwoluje sie tylko
  do wartosci bazowych gry; nie dodajemy nowego tokenu ani nowego wiersza do
  powiazanych tabel.
- `PORTRAIT=****`, `ENVMAP=****` oraz pozostale opcjonalne pola sa nullem, wiec
  nie wymagaja `portraits.2da`, dodatkowego envmapu ani innego custom resource.

`TARGETABLE=****` moze byc osobnym wyborem gameplayowym, ale nie uczestniczy
w rozwiazaniu `appearance -> RACE -> MDL`. Nie jest to jeszcze niezaleznie
potwierdzony runtimeowo argument przeciwko calej tabeli.

**Werdykt:** nie ma dowodu na brakujaca dodatkowa 2DA ani na blad skladniowy
aktualnego `appearance.2da`. Jednak reczny MOD nie eliminuje tej tabeli jako
zmiennej runtime. Rozstrzygajaca kontrola 2DA wymagalaby A/B z niezaleznie
znanym, dzialajacym modelem przy tej samej sciezce `appearance -> RACE -> MDL`,
a nie kolejnego recznego utworzenia MOD-a. Nie nalezy dodawac tabel na slepo.

## Audyt hipotezy animacji (2026-07-18)

### Read-only porownanie binarne

Odczytano tylko metadane animacji z lokalnego `cep3_core1.hak` (SHA-256
`6a8e6a64773a77fd46740cbcce19a708db6a70b4975732d0405978f3fbe8eb1a`,
bez kopiowania payloadu do repo) i bezposrednio z H1 v19 (SHA-256
`625974fda868edd9631a43341c5238669ff39e210bb50f10705ffb6cfec2f38f`).
Oba modele maja pointer array animacji przy core offset `232`, ale referencja
ma `43` klipy, a H1 tylko `7`.

| Klip | `c_squirrel`: dlugosc / transition / eventy | H1 v19: dlugosc / transition / eventy |
| --- | --- | --- |
| `cappear` | `2.0 / 0.25 / 0` | `4.033333 / 0.25 / 0` |
| `cpause1` | `2.0 / 0.25 / 0` | `4.033333 / 0.25 / 0` |
| `cwalk` | `0.5 / 0.25 / 2` | `4.033333 / 0.25 / 0` |
| `crun` | `0.5 / 0.25 / 2` | `4.033333 / 0.25 / 0` |
| `ca1slashl` | `1.0 / 0.0 / 1` | `4.033333 / 0.25 / 0` |
| `cdamagel` | `0.333333 / 0.0 / 0` | `4.033333 / 0.25 / 0` |
| `cdead` | `1.0 / 0.25 / 0` | `4.033333 / 0.25 / 0` |

Kazdy z siedmiu klipow H1 ma `24` nodow i lacznie `343` tracki, ale sa to
aliasy jednego ruchu, bez eventow. To jest rzeczywisty rozjazd semantyczny od
dzialajacego direct creature; obecnosc nazw nie jest rownowazna natywnej
animacji o tej nazwie.

### Co wyklucza v20, a co pozostawia otwarte

Bezposredni odczyt H1 v20 (RIGID, SHA-256
`6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd`)
potwierdza bitowo ten sam profil naglowkow animacji: te same 7 nazw,
dlugosci `4.033333`, transition `0.25`, zero eventow i
`animroot=m2a_m6p01`. Ten wariant zostal narysowany przez NWN jako
zdeformowana figura.

Z tego wynika, ze brak dodatkowych klipow, eventow albo natywnej dlugosci
**nie jest globalnym warunkiem zaladowania ani draw modelu**. Nie wolno
teraz przepisywac generatora animacji tylko na tej hipotezie.

Nie eliminuje to jednak animacji jako czynnika specyficznego dla skina:
v20 omija skin, podczas gdy v19 laczy animowane kosci z inverse-bind i
wagami. Dekompilacja pokazuje zapis/odczyt serializowanego pola
`AnimationState` (`FUN_00aeed68` przy 329773 i `FUN_00aefd1c` przy 329889),
ale w tym miejscu nie ustanawia mapowania liczbowego stanu na nazwe klipu
ani nie dowodzi, jaki transform trafia do palety skina.

**Werdykt:** animacja pozostaje kandydatem tylko dla sciezki
`AnimationState -> kosci -> skin`, nie dla globalnej widocznosci modelu.
Nastepny krok Aurora First to sledzenie tego resolvera w dekompilacji i
porownanie transformow base-pose/klipu przed wejsciem do
`FUN_00a932cc`; dopiero potem mozna uzasadnic zmiane writera lub nowy proof.

## Zewnetrzna kontrola hipotez animacji i HAK (2026-07-18)

To sa zrodla community/tooling, wiec potwierdzaja kierunek dochodzenia, a nie
zastepuja proofu ani dekompilacji Aurora.

1. [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf)
   stanowi, ze nazwa animacji okresla jej uzycie, `Root` nadpisuje tylko nody
   ponizej root, a eventy maja nazwe i frame. Potwierdza to, ze siedem aliasow
   H1 nie jest semantycznie tym samym co siedem natywnych klipow referencji.
2. W rozwiazanym przypadku ["Altered pause1-animation leads to wrong angles"](https://forum.neverwintervault.org/t/altered-pause1-animation-leads-to-wrong-angles-for-the-other-animations-how-so-solved/5962)
   zmiana orientacji `rootdummy` powodowala bledna orientacje innych animacji
   w grze, mimo poprawnego obrazu w Blenderze. Finalnie potwierdzona przyczyna
   byla niezamierzona zmiana rootdummy. To wzmacnia dokladnie hipoteze
   `animacja -> kosci -> skin`; nie dowodzi jeszcze jej dla H1.
3. W [dyskusji o skinmesh i keyframed bones](https://forum.neverwintervault.org/t/blender-and-rigged-mesh-to-keyframed-bones/555)
   autor NeverBlendera potwierdza, ze skinmesh deformuje sie przez armature,
   a community opisuje limit czterech bone influences na vertex dla tego
   formatu. H1 ma najwyzej cztery influence na vertex, wiec ta granica nie
   wyjasnia problemu.
4. Istnieje tez niezalezny przypadek [modeli widocznych w Toolsecie, ale nie
   renderowanych w grze](https://neverwintervault.org/comment/77468), ktory
   rozwiazal brak `cep2_add_ee.hak`. To jest wazna klasa objawu, ale nie pasuje
   bezposrednio do H1: auditowany model ma `supermodel=NULL`, a jedyny HAK
   MOD-a zawiera `appearance`, wlasny MDL i teksture. Nie ma stwierdzonej
   zewnetrznej zaleznosci CEP/EE.

**Wniosek po kontroli zewnetrznej:** nie dodawac teraz kolejnych nazw ani
eventow na slepo. Najbardziej uzasadniony pozostaje test runtime transformow
root/bones i inverse-bind w wariancie skin; rownolegle trzeba zachowac
read-only kontrole kompletnej listy zaleznosci HAK dla kazdego proofu.

## Aktualne ograniczenia NWN:EE i korekta hipotezy skin (2026-07-18)

### Fakty o konkretnym runtime

Read-only odczyt lokalnego manifestu Steam wskazuje na AppID `704450`, build
`20277208`. Historia tego builda identyfikuje go jako NWN:EE
`89.8193.37-17`, a wiec nie jako legacy 1.69.

Beamdog w oficjalnych notatkach do patcha `8193.23` podaje limit **64 kosci
na pojedynczy skinmesh**. Aktualna dokumentacja formatu NWN:EE dodatkowo
podaje maksymalnie **4 wplywy kosci na wierzcholek**. Zrodla:

- [Beamdog: patch 8193.23](https://www.beamdog.com/news/patch-819323-launches-neverwinter-nights-enhanced-edition/)
- [NWN Wiki: Models](https://nwn.wiki/spaces/NWN1/pages/38175602/Models)
- [NWN Wiki: tabela parametrow MDL](https://nwn.wiki/spaces/NWN1/pages/53671005/Model%2BTable%2Bof%2BParameters)
- [SteamDB: build 20277208](https://steamdb.info/patchnotes/20277208/)

### Zastosowanie do H1

| Wariant | Skin | Wynik wobec obecnej granicy |
| --- | --- | --- |
| v19 | `25` aktywnych slotow kosci; najwyzej `4` wplywy na vertex | miesci sie w obu udokumentowanych limitach EE |
| v20 | brak nodu skin, zwykly RIGID trimesh | limit skinu nie ma zastosowania |

**Korekta wniosku:** dawna liczba `17/21/24` z CleanModels3 opisuje
historyczny workflow 1.69 i nie jest prawidlowym kandydatem na root cause dla
tej instalacji NWN:EE. Hipoteza, ze `25` slotow H1 v19 samo w sobie powoduje
niewidocznosc, zostaje **odrzucona**. Nie oznacza to proofu poprawnej
deformacji: nadal wymagany jest runtime proof dla skina i osobno dla
wspolnej sciezki mesh/MDX.

### Co zmieniaja materialy o kompilacji i animacjach

1. Oficjalny patch 8193.23 zawiera zarowno zabezpieczenie renderera na brak
   animacji, jak i naprawy skina. Brak klipu nadal moze dawac zly stan lub
   deformacje, ale sam w sobie nie jest juz uzasadnieniem tezy, ze model musi
   zniknac.
2. Dokumentacja NWN:EE zaleca kompilacje ASCII skinmesh dopiero po zaladowaniu
   go w grze przez `compileloadedasciimodels`; kompilacja jednorazowym
   `compilemodel` nie inicjalizuje skina. To jest niezalezny, wartosciowy
   kontrolny pipeline, lecz nie jest naprawa binarnego writera Meshy2Aurora.
3. Kompilator gry zapisuje wynik do
   `Documents\\Neverwinter Nights\\modelcompiler`. Uruchomienie go wymaga
   osobnej zgody wlasciciela, bo tworzy zewnetrzny artefakt w instalacji
   uzytkownika. Nie zostal uruchomiony w ramach tego audytu.

### Priorytet po korekcie

Najmocniejszy pozostaly trop nie jest juz liczba kosci ani lista nazw klipow,
lecz **semantyka wspolnej sciezki runtime mesh/MDX oraz transformacji
base-pose/inverse-bind**, ktora Toolset toleruje, a klient gry pokazuje jako
brak lub zdeformowana figure. Kolejny krok implementacyjny ma najpierw dodac
gate porownujacy te pola z native binary MDL, a dopiero potem wykonac nowy
proof standardowym adapterem Aurora -> NWN.
