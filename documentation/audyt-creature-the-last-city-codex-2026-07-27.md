# Audyt creature modułu `the_last_city — codex.mod`

Data audytu: 2026-07-27
Zakres: dokładny spakowany MOD, jego 58 HAK-ów, efektywne `appearance.2da`,
zasoby modeli, blueprinty UTC, osadzenia GIT oraz skrypty NSS/NCS związane z
creature.

## 1. Werdykt

Moduł nie ma jednego globalnego problemu, przez który wszystkie creature byłyby
niewidoczne. Większość jego użytych creature ma spójny łańcuch:

`GIT/UTC -> Appearance_Type -> lc_2da/appearance.2da -> istniejący MDL`.

Są jednak cztery twarde błędy:

1. Trzy osadzone creature wskazują na puste, rezerwowane wiersze
   `appearance.2da` i nie mają z nich żadnego modelu do narysowania:

   | Area | Template | Appearance | Stan w `appearance.2da` |
   | --- | --- | ---: | --- |
   | `hx_tygiel_under` | `hx_undertyg_robo` | 2082 | `LABEL=****`, `NAME=USER_RESERVED`, `MODELTYPE=****`, `RACE=****` |
   | `koszary_new` | `wnd_test` | 7000 | `LABEL=****`, `NAME=CEP_RESERVED`, `MODELTYPE=****`, `RACE=****` |
   | `njkb_engine` | `njkb_npc_1` | 7000 | `LABEL=****`, `NAME=CEP_RESERVED`, `MODELTYPE=****`, `RACE=****` |

   To są deterministycznie błędne osadzenia, a nie przypuszczenie o rendererze.
   W `koszary_new` dodatkowo sam blueprint `wnd_test.utc` ma poprawny
   `Appearance_Type=4000` (`Warjack, red`, model `c_warjack_red` z
   `lc_ccc.hak`), lecz instancja nadpisuje go pustym wierszem 7000.

2. Loch pająków próbuje tworzyć `d_exploding_spider`, podczas gdy moduł ma
   wyłącznie `d_exploding_spid.utc`. Błędny, 18-znakowy resref występuje nie
   tylko w źródle `d_spiders.nss`, ale również w skompilowanym
   `d_spiders_c_a_hb.ncs`. Ta gałąź `CreateObject` nie znajduje lokalnego UTC,
   UTC w HAK-ach ani UTC w bazowej grze.

3. `hx_undertyg_robo` ma równocześnie dwa nierozwiązywalne skrypty instancji:
   `hx_tygiel_dzwon` na `OnSpawn` oraz `zep_arm_on_conv` na `OnConversation`.
   Nie ma ich jako NCS w module, dołączonych HAK-ach, bazowej grze ani lokalnym
   `override`.

4. Pętla `SpidersCreateAdds(int nSpiders)` używa
   `for(i=0; i<=nSpiders; i++)`, czyli tworzy `nSpiders + 1` obiektów. Dla
   wartości zero również próbuje utworzyć jednego adda. To osobny błąd liczby
   spawnów.

Poza tym:

- 46 z 48 wierszy appearance użytych przez statyczne creature jest
  nierezerwowanych i każdy ich model został odnaleziony;
- pozostałe dwa wiersze to właśnie 2082 i 7000 opisane powyżej;
- wszystkie 56 wierszy appearance użytych przez 116 lokalnych UTC istnieje,
  żaden nie jest rezerwowany, a wszystkie wymagane modele fixed/simple
  zostały odnalezione;
- wszystkie 58 HAK-ów zapisanych w `module.ifo` istnieje na tej stacji;
- jedyne `appearance.2da` w stosie HAK pochodzi z `lc_2da.hak`, więc nie ma
  konfliktu kilku wersji tej tabeli.

## 2. Tożsamość audytowanego modułu

| Pole | Wartość |
| --- | --- |
| Plik | `C:\Users\enonw\Documents\Neverwinter Nights\modules\the_last_city — codex.mod` |
| Rozmiar | 84 936 838 B |
| SHA-256 | `b774037f160a9be3fcc0e985dde880a0ae97e10af2bf868c38a3f6573cbb021e` |
| Modyfikacja | 2026-07-03 14:56:37 |
| `Mod_Name` | `the_last_city` |
| Custom TLK | `tlc` |
| HAK-i | 58/58 obecnych |

Na dysku istnieje także:

`C:\Users\enonw\Documents\Neverwinter Nights\modules\the_last_city - codex.mod`

Wariant z krótkim łącznikiem ma identyczny rozmiar, czas oraz SHA-256. Nie jest
inną wersją modułu, ale drugi alias nazwy jest źródłem niepotrzebnej
niejednoznaczności przy ręcznym wyborze.

Spakowany MOD jest wewnętrznie kompletny na poziomie Areas:

- `Mod_Area_list`: 170 unikalnych wpisów;
- ARE: 170;
- GIT: 170;
- GIC: 170;
- brak brakujących lub dodatkowych par ARE/GIT/GIC.

Starszy loose snapshot opisany w audycie `aurora-web` miał 169 Areas. Nie
powinien być traktowany jako bieżący obraz tego spakowanego MOD-u.

## 3. Liczby

| Metryka | Wynik |
| --- | ---: |
| lokalne blueprinty UTC | 116 |
| statyczne osadzenia creature w GIT | 184 |
| Areas z co najmniej jednym creature | 37 z 170 |
| Areas bez statycznego creature | 133 |
| unikalne template resref w GIT | 63 |
| unikalne lokalne UTC użyte statycznie | 40 |
| unikalne template’y bazowej gry użyte statycznie | 18 |
| custom template’y zachowane tylko jako pełna instancja GIT | 5 |
| statyczne osadzenia lokalnych UTC | 80 |
| statyczne osadzenia template’ów spoza modułu | 104 |
| standardowe Encounter instances | 0 |
| zasoby UTE | 0 |
| wywołania `CreateObject(OBJECT_TYPE_CREATURE, ...)` w NSS | 30 |
| wywołania z dosłownym resref | 9 |
| lokalne UTC użyte przez dosłowny spawn | 6 |
| lokalne UTC użyte statycznie lub przez dosłowny spawn | 46 |
| lokalne UTC bez statycznego użycia i bez dosłownego spawnu | 70 |
| lokalne UTC bez statycznego użycia i bez choćby cytowania w NSS | 29 |

W module nie ma standardowych encounterów. Populacja jest realizowana przez
osadzenia statyczne oraz własny kod spawnów, m.in. lochy pająków i nieumarłych,
system petów, questów, bestiariusza i mountów.

## 4. Zawartość testowa a właściwa

Klasyfikacja poniżej jest jawnie heurystyczna. Za obszar testowy uznano tylko
taki, którego resref/nazwa zawiera `test` albo którego nazwa zawiera `[Beta]`.
Nie klasyfikowano automatycznie wszystkich `enon_*`, `njkb_*` lub `hx_*` jako
testowych.

| Warstwa | Areas | Osadzenia | Unikalne template’y | Lokalnych | Zewnętrznych |
| --- | ---: | ---: | ---: | ---: | ---: |
| jawnie test/beta | 22 | 111 | 36 | 21 | 15 |
| pozostałe | 15 | 73 | 29 | 19 | 10 |

111 z 184 osadzeń, czyli 60,3%, znajduje się w jawnie testowych lub beta
Areas. Największy udział ma `enon_test_quest`: 53 creature. Wszystkie 19
instancji `npc_dummy_target` znajduje się w obszarach test/beta.

To nie psuje automatycznie gry, ale oznacza, że spakowany moduł produkcyjny jest
jednocześnie laboratorium i fixturem testowym. Utrudnia to audyty, przegląd
palety oraz odróżnienie gotowych NPC od placeholderów.

W warstwie nieoznaczonej jako testowa nadal są elementy robocze:

- `koszary_new` używa `wnd_test` i nadpisuje je pustym appearance 7000;
- `wnd_shop_port` zawiera `wnd_guardian_cpy`;
- `wnd_shop_port` zawiera `wnd_test_skl_prt` o nazwie
  `Pan sklepikarz co umrze po becie`;
- `njkb_engine` zawiera `njkb_npc_1` z pustym appearance 7000;
- `d_spiders` zawiera trzy hostile combat dummy `x2_golem002` o nazwie
  `Dumy`, CR 21 i bez własnych skryptów instancji.

## 5. Profile creature właściwe dla settingu

Rzeczywisty język creature w module jest przede wszystkim dark-fantasy,
miejski i dungeonowy:

- kanały: aligatory, pisklęta, szczury, `Zgniłoszczęki`;
- loch pająków: `Skakun`, `Sieciarz`, `Miecznik`, `Widmowiec`,
  `Matka pająków`, robotnik, krwawy pająk, czarna wdowa i wariant eksplodujący;
- loch nieumarłych: strzelcy, wojownicy, abominacja, inżynier, strażnik oraz
  czterech nazwanych władców;
- slumsy i miasto: urzędnicy, bankier, kowal, strażnicy, sklepikarze,
  rzemieślnicy, szpital, marynarz, paser i gospodarskie zwierzęta;
- warstwa systemowa/testowa: dummy targets, bestiary, mounty, pety,
  reputation actors i portal placeholder;
- dodatkowe rodziny tekstowo podłączone do systemów: ghule, upiory,
  szkielety, hieny cmentarne, renegaci, przemytnicy, ślimaki i pająki.

Wniosek dla nowych creature The Last City: najbliższe istniejącemu modułowi są
skażone zwierzęta kanałowe, pajęczaki, nieumarli, miejskie bestie i humanoidalni
NPC. Clockwork/warjack również istnieje (`Appearance 4000`), ale jako pojedynczy
roboczy `wnd_test`, a nie dominująca rodzina settingu.

## 6. Audyt appearance i modeli

Efektywna tabela:

`lc_2da.hak -> appearance.2da`

ma 15 218 wierszy. Moduł statycznie używa 48 różnych wierszy. Rezultat:

- 46 wierszy ma prawidłowe dane;
- dla każdego fixed/simple modelu z tych 46 wierszy istnieje wymagany MDL w
  bazie gry lub dołączonym HAK-u;
- wiersz 2082 jest pustym `USER_RESERVED`;
- wiersz 7000 jest pustym `CEP_RESERVED`.

Przykładowe poprawne łańcuchy:

| Appearance | Label | Model | Źródło modelu |
| ---: | --- | --- | --- |
| 6403 | `Alligator* (Project Q)` | `qc_alligator` | `cep3_core2.hak` |
| 1315 | `Spiderling: Phase*` | `spiderling_ph` | `cep3_core1.hak` |
| 1916 | `Spider: Sword - Gargantuan*` | `c_spidswrd06` | `cep3_core1.hak` |
| 1922 | `Spider: Wraith - Gargantuan*` | `c_spidwra06` | `cep3_core1.hak` |
| 463 | `[Unique] Maggris` | `c_maggris` | bazowa gra |
| 63 | `Skeleton_Common` | `c_skel_com01` | bazowa gra / CEP |
| 4000 | `Warjack, red` | `c_warjack_red` | `lc_ccc.hak` |
| 3022 | `Human: Gaffer 2*` | `codi_bom02` | `cep3_core1.hak` |
| 3346 | `Deer: Irish*` | `zcp_irishdeer` | `cep3_core1.hak` |
| 1806 | `Donkey: Pack*` | `c_donkey2` | `cep3_core2.hak` |

Zatem nie należy diagnozować modułu jako ogólnie pozbawionego modeli creature.
Konkretne osadzenia 2082 i 7000 są wyjątkiem i mają jednoznacznie zerwany
łańcuch appearance.

## 7. Audyt spawnów skryptowych

### 7.1 Bezpośrednie lokalne spawny

Sześć lokalnych UTC ma dosłowny resref w `CreateObject`:

| Resref | Nazwa | Appearance | Skrypt źródłowy | Model |
| --- | --- | ---: | --- | --- |
| `husky` | Husky | 1394 | `lib_c_tamer` | rozwiązany |
| `hx_test_crab` | Krab Testowy | 1430 | `hx_test_crabs` | rozwiązany |
| `ownerburglary` | Właciciel | 6506 | `lib_burglary` | rozwiązany |
| `portal_placehold` | portal placeholder | 3129 | `lib_portal` | rozwiązany |
| `sailor` | Marynarz | 6 | `lib_pc_create` | dynamic human |
| `wnd_spider11_sma` | wnd_spider11_sma | 1920 | `lib_npc_combat` | rozwiązany |

### 7.2 Dynamiczne lochy

Skompilowane NCS potwierdzają następujące runtime resref:

- pająki: `d_blackwidow`, `d_bloodyspider`, `d_guard`,
  `d_spiderworker`, błędne `d_exploding_spider` oraz `d_miner`;
- nieumarli: `d_und_b_a`, `d_und_b_k`, `d_und_b_n`, `d_und_b_w`,
  `d_und_a_w`, `d_und_a_a`, `d_und_a_ab`.

Wszystkie poza `d_exploding_spider` mają lokalny UTC. Minimalna naprawa to
zmiana stałej w `d_spiders` na `d_exploding_spid` i ponowna kompilacja
zależnych skryptów, zwłaszcza `d_spiders_c_a_hb`.

### 7.3 Nierozwiązane literalne resref

- `henchman_test` jest osadzony w skompilowanym `lib_test.ncs`, ale nie istnieje
  jako UTC w module, HAK-ach, bazowej grze ani override. To błąd fixture/dev,
  nie potwierdzona ścieżka produkcyjna.
- `jass_spelltrig` występuje w źródłowym include `j_inc_spawnin.nss`, ale nie
  ma go w żadnym skompilowanym NCS modułu. Jedyny call `SetSpellTrigger` w
  `j_ai_onspawn.nss` jest zakomentowany, a żadne lokalne UTC ani statyczna
  instancja nie używa `j_ai_onspawn`. To martwa zależność źródłowa, nie aktywny
  runtime blocker obecnego modułu.

### 7.4 Ograniczenie statycznego audytu

21 z 30 wywołań `CreateObject` otrzymuje resref przez zmienną, dane lokalne,
quest, bazę danych albo tablicę. Audyt potwierdza stałe lochów i dosłowne
wywołania, ale nie może udowodnić wszystkich wartości wstrzykiwanych przez
runtime DB bez uruchomienia serwera z jego danymi.

## 8. Skrypty i dialogi creature

Wszystkie lokalne UTC oraz statyczne instancje odwołują się łącznie do 90
unikalnych skryptów eventowych.

- 88 rozwiązuje się w module, HAK-ach albo bazowej grze;
- 2 brakujące skrypty należą do `hx_undertyg_robo`:
  `hx_tygiel_dzwon` i `zep_arm_on_conv`.

Dialogi:

- wszystkie dialogi użyte przez statyczne instancje są dostępne;
- `kds1_dead_pc` i `kds1_dead_pc_f` wskazują na brakujący dialog `dummy`;
  są tworami ciała gracza i nie są osadzone statycznie, więc jest to dług
  niskiego ryzyka, o ile nie mają być interaktywne.

## 9. Problemy spójności blueprintów i instancji

### 9.1 `d_miner` udający `d_guard`

Instancja `d_spiders[8]` ma:

- `TemplateResRef=d_miner`;
- nazwę `Strażniczka`;
- `Tag=d_guard`;
- portret strażniczki;
- płeć 0, podczas gdy właściwy `d_guard.utc` ma płeć 1.

Moduł ma osobny poprawny `d_guard.utc`, a skrypty lochu używają go dynamicznie.
Statyczna instancja wygląda na ręcznie przerobioną kopię `d_miner`, która
zachowała złą tożsamość blueprintu. Nie musi być niewidoczna, ale utrudnia
aktualizacje i może zachowywać niezamierzone pola z górnika.

### 9.2 Custom template’y zachowane tylko w GIT

Pięć custom resref nie istnieje jako osobny UTC ani w module, ani w HAK-ach,
ani w bazowej grze:

| Resref | Osadzenia |
| --- | ---: |
| `alligator_hatchl` | 21 |
| `creature` | 2 |
| `npc_quest` | 2 |
| `hx_undertyg_robo` | 1 |
| `njkb_npc_1` | 1 |

Statyczne GIT przechowuje pełną strukturę creature, więc sam brak osobnego UTC
nie powoduje zniknięcia już osadzonego obiektu. Powoduje jednak, że tych
template’ów nie można bezpiecznie ponownie tworzyć przez
`CreateObject(..., resref, ...)` ani traktować jako stabilnej biblioteki
blueprintów. Jeśli mają być reużywane, trzeba je promować do jawnych UTC.

### 9.3 Nadpisania appearance instancji

11 lokalnych instancji ma appearance inne niż ich bazowy UTC:

- 8 jest w jawnie testowych Areas i służy m.in. krabom/dummy;
- 2 `race_chicken` w `wnd_slms_gruz` zmienia appearance 31 na 6508;
- 1 `wnd_test` w `koszary_new` zmienia poprawne 4000 na puste 7000.

Pierwsze dwie grupy mogą być świadomym reuse blueprintu. Ostatnia jest
jednoznacznie błędna.

### 9.4 Nazwy i tagi robocze

- 27 lokalnych UTC ma nazwę równą resrefowi;
- 26 UTC ma tag inny niż resref; większość dungeonowych kolizji jest
  zamierzona, bo skrypty grupują obiekty wspólnym tagiem;
- `kowal` jest wspólnym tagiem `npc_bsmh1` i `npc_bsmh3`, choć oba obecnie nie
  mają statycznego ani tekstowego użycia;
- placeholdery obejmują m.in. `bestiarytest`, `npc_dummy_target`,
  `portal_placehold`, `wnd_guardian_cpy`, `wnd_test*`.

## 10. Pełna lista Areas z creature

| Area resref | Nazwa | Instancje | Template’y | Klasa | Użyte resref |
| --- | --- | ---: | ---: | --- | --- |
| `enon_test_quest` | enon_test_quest | 53 | 10 | test/beta | `npc_quest`, `nw_goblina`, `nw_goblinb`, `nw_gobwiza`, `nw_gobwizb`, `nw_oldman`, `nw_skeleton`, `nw_skelmage`, `nw_skelwarr01`, `nw_skelwarr02` |
| `d_sewers001` | d_sewers | 22 | 2 | pozostałe | `alligator_hatchl`, `dsewerboss` |
| `d_und` | d_und | 17 | 5 | pozostałe | `d_und_a_a`, `d_und_a_ab`, `d_und_a_w`, `d_und_engineer`, `d_und_guard` |
| `d_spiders` | d_spiders | 9 | 7 | pozostałe | `d_miner`, `d_spiderjumper`, `d_spidernetmaker`, `d_spiders_boss`, `d_spidersword`, `d_spiderwraith`, `x2_golem002` |
| `enon_test_bestia` | enon_test_bestiary | 7 | 3 | test/beta | `bestiarytest`, `drgblack002`, `nw_goblina` |
| `wnd_testowa` | wnd_testowa | 7 | 3 | test/beta | `npc_dummy_target`, `wnd_eff_tst`, `zep_malekid003` |
| `test_arena_pve` | [Beta] Arena PvE | 5 | 1 | test/beta | `npc_dummy_target` |
| `wnd_slms_s_gate` | Ostatnie Miasto - Slumsy: Północna Brama | 5 | 2 | pozostałe | `nw_chicken`, `x0_penguin001` |
| `wnd_testing` | [Lokacja Testowa] | 5 | 2 | test/beta | `npc_dummy_target`, `zep_malekid003` |
| `enon_test_pp` | enon_test_pp | 4 | 2 | test/beta | `nw_malekid01`, `nw_oldman` |
| `enon_test_rep` | enon_test_rep | 4 | 4 | test/beta | `rep_commoner`, `rep_defender`, `rep_hostile`, `rep_merchant` |
| `enon_test_zoo` | enon_test_zoo | 4 | 4 | test/beta | `catnocombat`, `creature`, `dognocombat`, `zookeeper` |
| `wnd_slms_gruz` | Ostatnie Miasto - Slumsy: Rozpadlina | 4 | 1 | pozostałe | `race_chicken` |
| `enon_house_6` | enon_house_6 | 3 | 2 | pozostałe | `nw_chicken`, `nw_rat001` |
| `enon_test_burgla` | enon_test_burglary | 3 | 2 | test/beta | `fence`, `guardburglary` |
| `enon_test_shop` | enon_test_shop | 3 | 1 | test/beta | `nw_innkeeper` |
| `test_arena_pvp` | [Beta] Arena PvP | 3 | 1 | test/beta | `npc_dummy_target` |
| `wnd_shop_port` | enon_shop_1 | 3 | 3 | pozostałe | `wnd_guardian`, `wnd_guardian_cpy`, `wnd_test_skl_prt` |
| `enon_gang_2` | enon_gang_2 | 2 | 1 | pozostałe | `nw_rat001` |
| `enon_test_ads` | enon_test_ads | 2 | 2 | test/beta | `herald`, `npc_noticeboard` |
| `enon_test_new_ca` | enon_test_new_card | 2 | 2 | test/beta | `npc_dummy_target`, `zep_malekid003` |
| `hx_bank` | Ostatnie Miasto - Rynek, Dom Bankowy | 2 | 2 | pozostałe | `creature`, `wnd_bank_clerk` |
| `enon_test_ah` | enon_test_ah | 1 | 1 | test/beta | `auctioneer` |
| `enon_test_build` | enon_test_build | 1 | 1 | test/beta | `kwatermistrz` |
| `enon_test_cnr` | enon_test_cnr | 1 | 1 | test/beta | `nw_deer` |
| `enon_test_dungeo` | enon_test_dungeon | 1 | 1 | test/beta | `boss` |
| `enon_test_effect` | enon_test_effects | 1 | 1 | test/beta | `nw_innkeeper` |
| `enon_test_loot` | enon_test_loot | 1 | 1 | test/beta | `nw_drowrogue010` |
| `enon_test_mount` | enon_test_mount | 1 | 1 | test/beta | `stablemaster` |
| `enon_test_pets` | enon_test_pets | 1 | 1 | test/beta | `mastiff` |
| `enon_test_talent` | enon_test_talents | 1 | 1 | test/beta | `npc_dummy_target` |
| `hx_tygiel_under` | Arkona, Kanały - zapomniany odcinek | 1 | 1 | pozostałe | `hx_undertyg_robo` |
| `koszary_new` | Arkona Zachodni Brzeg - Koszary | 1 | 1 | pozostałe | `wnd_test` |
| `njkb_engine` | njkb_engine | 1 | 1 | pozostałe | `njkb_npc_1` |
| `wnd_blacksmith` | Ostatnie Miasto - Stocznia, Kowal | 1 | 1 | pozostałe | `wnd_blacksmith_1` |
| `wnd_citymarket` | Ostatnie Miasto - Rynek | 1 | 1 | pozostałe | `npc_quest` |
| `wnd_slms_swamp` | Ostatnie Miasto - Slumsy: Bagno | 1 | 1 | pozostałe | `nw_ox` |

## 11. Pełna lista statycznie użytych template’ów

`embedded GIT only` oznacza custom template bez osobnego UTC. Jego już
osadzona instancja zawiera komplet danych, ale resref nie jest dostępny do
ponownego spawnu.

| Resref | Nazwa przykładowej instancji | Liczba | Pochodzenie | Appearance | Areas |
| --- | --- | ---: | --- | --- | --- |
| `alligator_hatchl` | Pisklę aligatora | 21 | embedded GIT only | 6403 | `d_sewers001` |
| `npc_dummy_target` | npc_dummy_target | 19 | module UTC | 201, 271, 1427, 1429 | `enon_test_new_ca`, `enon_test_talent`, `test_arena_pve`, `test_arena_pvp`, `wnd_testing`, `wnd_testowa` |
| `nw_oldman` | Old Man Inventory | 9 | NWN base | 239 | `enon_test_pp`, `enon_test_quest` |
| `d_und_a_a` | Nieumarły Strzelec | 8 | module UTC | 63 | `d_und` |
| `nw_goblina` | Goblin | 7 | NWN base | 86 | `enon_test_bestia`, `enon_test_quest` |
| `d_und_a_w` | Nieumarły Wojownik | 6 | module UTC | 63 | `d_und` |
| `nw_chicken` | strref:12416 | 6 | NWN base | 31, 368, 6508, 6509 | `enon_house_6`, `wnd_slms_s_gate` |
| `nw_goblinb` | Goblin | 6 | NWN base | 87 | `enon_test_quest` |
| `nw_gobwiza` | strref:12568 | 6 | NWN base | 84 | `enon_test_quest` |
| `nw_gobwizb` | strref:12568 | 6 | NWN base | 85 | `enon_test_quest` |
| `nw_skeleton` | strref:12701 | 6 | NWN base | 63 | `enon_test_quest` |
| `nw_skelwarr01` | Skeleton Warrior | 6 | NWN base | 70 | `enon_test_quest` |
| `bestiarytest` | bestiarytest | 5 | module UTC | 420 | `enon_test_bestia` |
| `nw_skelmage` | Skeleton Mage | 5 | NWN base | 148 | `enon_test_quest` |
| `nw_skelwarr02` | strref:12704 | 5 | NWN base | 71 | `enon_test_quest` |
| `nw_innkeeper` | Innkeeper | 4 | NWN base | 233 | `enon_test_effect`, `enon_test_shop` |
| `race_chicken` | Chicken | 4 | module UTC | 31, 6508 | `wnd_slms_gruz` |
| `nw_rat001` | strref:3103 | 3 | NWN base | 386 | `enon_gang_2`, `enon_house_6` |
| `x2_golem002` | Dumy | 3 | NWN base | 201 | `d_spiders` |
| `zep_malekid003` | Boy | 3 | module UTC | 1267 | `enon_test_new_ca`, `wnd_testing`, `wnd_testowa` |
| `creature` | Jeleń | 2 | embedded GIT only | 3022, 3346 | `enon_test_zoo`, `hx_bank` |
| `fence` | Paser | 2 | module UTC | 6 | `enon_test_burgla` |
| `npc_quest` | Ten od zadań | 2 | embedded GIT only | 212 | `enon_test_quest`, `wnd_citymarket` |
| `auctioneer` | Aukcjoner | 1 | module UTC | 6 | `enon_test_ah` |
| `boss` | Boss | 1 | module UTC | 0 | `enon_test_dungeo` |
| `catnocombat` | Cat | 1 | module UTC | 1410 | `enon_test_zoo` |
| `d_miner` | Górnik | 1 | module UTC | 6 | `d_spiders` |
| `d_spiderjumper` | Skakun | 1 | module UTC | 1315 | `d_spiders` |
| `d_spidernetmaker` | Sieciarz | 1 | module UTC | 1256 | `d_spiders` |
| `d_spiders_boss` | Matka pająków | 1 | module UTC | 463 | `d_spiders` |
| `d_spidersword` | Miecznik | 1 | module UTC | 1916 | `d_spiders` |
| `d_spiderwraith` | Widmowiec | 1 | module UTC | 1922 | `d_spiders` |
| `d_und_a_ab` | Nieumarła Abominacja | 1 | module UTC | 24 | `d_und` |
| `d_und_engineer` | Inżynier | 1 | module UTC | 232 | `d_und` |
| `d_und_guard` | Strażnik Nieumarłych | 1 | module UTC | 6 | `d_und` |
| `dognocombat` | Dog | 1 | module UTC | 1397 | `enon_test_zoo` |
| `drgblack002` | Adult Black Dragon | 1 | module UTC | 41 | `enon_test_bestia` |
| `dsewerboss` | Zgniłoszczęki | 1 | module UTC | 6403 | `d_sewers001` |
| `guardburglary` | Guard Burglary | 1 | module UTC | 6546 | `enon_test_burgla` |
| `herald` | Herold | 1 | module UTC | 450 | `enon_test_ads` |
| `hx_undertyg_robo` | Goblińska Myśl Technologiczna | 1 | embedded GIT only | 2082 | `hx_tygiel_under` |
| `kwatermistrz` | Kwatermistrz | 1 | module UTC | 6 | `enon_test_build` |
| `mastiff` | Mastif | 1 | module UTC | 1396 | `enon_test_pets` |
| `njkb_npc_1` | njkb_npc_1 | 1 | embedded GIT only | 7000 | `njkb_engine` |
| `npc_noticeboard` | Ten od ogłoszeń | 1 | module UTC | 6481 | `enon_test_ads` |
| `nw_deer` | Deer | 1 | NWN base | 35 | `enon_test_cnr` |
| `nw_drowrogue010` | strref:12503 | 1 | NWN base | 216 | `enon_test_loot` |
| `nw_malekid01` | DEFENDER FACTION | 1 | NWN base | 241 | `enon_test_pp` |
| `nw_ox` | Osiołek Oskar | 1 | NWN base | 1806 | `wnd_slms_swamp` |
| `rep_commoner` | Reputation Commoner | 1 | module UTC | 6 | `enon_test_rep` |
| `rep_defender` | Reputation Defender | 1 | module UTC | 6 | `enon_test_rep` |
| `rep_hostile` | Reputation Hostile | 1 | module UTC | 6 | `enon_test_rep` |
| `rep_merchant` | Reputation Merchant | 1 | module UTC | 6 | `enon_test_rep` |
| `stablemaster` | Stajenny | 1 | module UTC | 6 | `enon_test_mount` |
| `wnd_bank_clerk` | Albrecht | 1 | module UTC | 6 | `hx_bank` |
| `wnd_blacksmith_1` | wnd_blacksmith_1 | 1 | module UTC | 6 | `wnd_blacksmith` |
| `wnd_eff_tst` | wnd_eff_tst | 1 | module UTC | 6 | `wnd_testowa` |
| `wnd_guardian` | Guardian | 1 | module UTC | 6 | `wnd_shop_port` |
| `wnd_guardian_cpy` | Guardian2 | 1 | module UTC | 6 | `wnd_shop_port` |
| `wnd_test` | wnd_test | 1 | module UTC | 7000 | `koszary_new` |
| `wnd_test_skl_prt` | Pan sklepikarz co umrze po becie | 1 | module UTC | 6 | `wnd_shop_port` |
| `x0_penguin001` | strref:40617 | 1 | NWN base | 206 | `wnd_slms_s_gate` |
| `zookeeper` | Opiekun Zoo | 1 | module UTC | 6 | `enon_test_zoo` |

## 12. Lokalne UTC bez statycznego i bez dosłownego spawnu

`Cytowane w NSS=tak` oznacza jedynie tekstową osiągalność. Może to być stała,
konfiguracja, system DB albo komentarz; nie jest to samodzielny dowód wykonania
spawnu.

| Resref | Nazwa | Appearance | Cytowane w NSS | Skrypty |
| --- | --- | ---: | --- | --- |
| `caravan` | Karawana | 3491 | tak | `lib_build` |
| `d_blackwidow` | Czarna wdowa | 1248 | tak | `d_spiders` |
| `d_bloodyspider` | Krwawy pająk | 1168 | tak | `d_spiders` |
| `d_exploding_spid` | Eksplodujący pająk | 3951 | nie | — |
| `d_guard` | Strażniczka | 6 | tak | `d_spiders` |
| `d_spiderworker` | Pająk robotnik | 159 | tak | `d_spiders` |
| `d_und_b_a` | Nieumarła Władczyni Avri | 71 | tak | `d_und` |
| `d_und_b_k` | Nieumarły Władca Winimar | 182 | tak | `d_und` |
| `d_und_b_n` | Nieumarły Władca Salomon | 62 | tak | `d_und` |
| `d_und_b_w` | Nieumarła Władczyni Tamira | 70 | tak | `d_und` |
| `kds1_dead_pc` | Dead Body | 6 | tak | `lib_c_warrior`, `lib_death` |
| `kds1_dead_pc_f` | Dead Body | 6 | tak | `lib_death` |
| `mount_bear` | Niedwied | 3929 | tak | `test_mount` |
| `mount_griffon` | Gryf | 3913 | tak | `test_mount` |
| `mount_lizard` | Jaszczurka | 3922 | tak | `test_mount` |
| `mount_skeleton` | Szkielet Konia | 3903 | tak | `test_mount` |
| `npc_bimber1` | Bimbrownik Beczuła | 6 | nie | — |
| `npc_bimber2` | Bimbrownik Borys | 6 | nie | — |
| `npc_bimber3` | Bimbrowniczka Bożena | 6 | nie | — |
| `npc_bkr1` | Piekarz Piér | 6 | nie | — |
| `npc_bkr2` | Piekarz Buła | 6 | nie | — |
| `npc_bkr3` | Piekareczka Chałka | 6 | nie | — |
| `npc_bowm1` | Łuczkarz Łukasz | 6 | nie | — |
| `npc_bsmh1` | Kowal Klod | 6 | nie | — |
| `npc_bsmh2` | Kowalka Kunekunda | 0 | nie | — |
| `npc_bsmh3` | Kowal Kong | 3144 | nie | — |
| `npc_krawcowa1` | Krawcowa Kiki | 1 | nie | — |
| `npc_lthr1` | Rymarz Roman | 6 | nie | — |
| `npc_shopkpr1` | Sklepikarz Szymon | 232 | nie | — |
| `wnd_bat3_n` | Nietoperz | 10 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_bhood6_art` | wnd_bhood6_art | 6 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_bhood6_min` | wnd_bhood6_min | 6 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_bhood6_rat` | wnd_bhood6_rat | 387 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_doc_icu_exit` | wnd_doc_icu_exit | 6 | nie | — |
| `wnd_gator16_n` | Aligator | 6403 | tak | `lib_custom_exp` |
| `wnd_gator16_y` | Aligator, pisklę | 6403 | tak | `lib_custom_exp` |
| `wnd_ghou6_charla` | Ghoul, oprych | 2977 | tak | `lib_custom_exp`, `lib_quest`, `x2_def_spawn` |
| `wnd_ghou6_poison` | Ghoul, zatruty | 199 | tak | `lib_custom_exp`, `lib_quest`, `x2_def_spawn` |
| `wnd_ghou6_war` | Ghoul, wojownik | 197 | tak | `lib_custom_exp`, `lib_quest`, `x2_def_spawn` |
| `wnd_guardsman` | wnd_guardsman | 6 | nie | — |
| `wnd_hiena16_ar` | Hiena Cmentarna, łucznik | 6 | tak | `lib_custom_exp` |
| `wnd_hiena16_n` | Hiena Cmentarna, nożownik | 6 | tak | `lib_custom_exp` |
| `wnd_hiena16_war` | Hiena Cmentarna, wojownik | 6 | tak | `lib_custom_exp`, `lib_npc_combat` |
| `wnd_hospital_doc` | wnd_hsptl_doc_m | 6 | nie | — |
| `wnd_hosptl_doc_f` | wnd_hosptl_doc_f | 6 | nie | — |
| `wnd_hsptl_apoth` | wnd_hsptl_apoth | 6 | nie | — |
| `wnd_hsptl_inj_f` | wnd_hsptl_inj_f | 6 | nie | — |
| `wnd_hsptl_inj_m` | wnd_hsptl_inj_m | 6 | nie | — |
| `wnd_hsptl_scribe` | wnd_hsptl_scribe | 6 | nie | — |
| `wnd_marta6_war` | wnd_marta6_war | 6 | nie | — |
| `wnd_marta6_whip` | wnd_marta6_whip | 6 | nie | — |
| `wnd_rat3_n` | Szczur, wygłodniały | 15047 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_rene11_kni` | Renegat, rycerz | 6 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_rene11_rze` | Renegat, rzemielnik | 6 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_rene11_war` | Renegat, tarczownik | 6 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_slug3_n` | Ślimiak | 3475 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_smg3_n` | Przemytnik, nożownik | 6 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_spider11_coc` | wnd_spider11_coc | 2997 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_spider11_t` | wnd_spider11_t | 2998 | tak | `lib_custom_exp`, `lib_quest`, `x2_def_ondeath` |
| `wnd_test_dng_1` | wnd_test_dng_1 | 186 | nie | — |
| `wnd_test_dng_5` | wnd_test_dng_5 | 186 | nie | — |
| `wnd_test_guard` | Vine | 6 | tak | `lib_rest` |
| `wnd_u16_n` | Szkielet, skrytobójca | 3295 | tak | `lib_custom_exp` |
| `wnd_u16_war` | Szkielet, wojownik | 3292 | tak | `lib_custom_exp` |
| `wnd_u16_xbow` | Szkielet, kusznik | 3293 | tak | `lib_custom_exp` |
| `wnd_viper11_arch` | wnd_viper11_arch | 6 | nie | — |
| `wnd_viper11_ass` | wnd_viper11_ass | 6 | nie | — |
| `wnd_wright11_ass` | Upiór, strażnik | 186 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_wright11_war` | Upiór, wojownik | 186 | tak | `lib_custom_exp`, `lib_quest` |
| `wnd_zlodupiec` | Złodupiec | 269 | nie | — |

Szczególna uwaga: `d_exploding_spid` nie jest faktycznym orphanem projektowym.
Jest zamierzonym dynamicznym addem, ale skrypt cytuje błędny, dłuższy resref
`d_exploding_spider`, dlatego prosty audyt tekstowy go nie łączy.

## 13. Kolejność napraw

1. Naprawić trzy puste appearance:

   - `wnd_test` w `koszary_new`: usunąć override 7000 albo przywrócić 4000,
     jeśli zamiarem jest istniejący czerwony warjack;
   - `njkb_npc_1`: wybrać istniejący appearance i zachować pełny standalone UTC,
     jeśli NPC ma pozostać;
   - `hx_undertyg_robo`: odtworzyć poprawny appearance i brakujące skrypty albo
     usunąć legacy instancję z gotowego modułu.

2. Zmienić `SPIDER_EXPLODING_RESREF` z `d_exploding_spider` na
   `d_exploding_spid`, skompilować wszystkie zależne eventy i potwierdzić, że
   długi literal nie występuje już w żadnym NCS.

3. Zmienić warunek pętli addów z `<= nSpiders` na `< nSpiders`, jeżeli parametr
   oznacza liczbę addów zgodnie z nazwą.

4. Zastąpić statyczną instancję `d_miner`/`d_guard` właściwym blueprintem
   `d_guard` albo udokumentować świadome odstępstwo.

5. Przenieść jawne Areas test/beta do osobnego modułu deweloperskiego albo
   wprowadzić jednoznaczną granicę pakowania. Obecnie ponad połowa creature w
   MOD-ie to fixture’y.

6. Promować pięć `embedded GIT only` do standalone UTC tylko wtedy, gdy mają
   być reużywane lub spawnowane skryptem.

7. Po naprawach wykonać osobny, właścicielski proof w Toolset/NWN dla dokładnie
   tego samego hasha nowego modułu. Ten dokument jest audytem offline i nie
   ustanawia `modelVisibility=visible`.

## 14. Metoda i ograniczenia

Audyt nie uruchamiał Toolset ani NWN, zgodnie z obowiązującą decyzją o
właścicielskim finalnym proof.

Sprawdzono:

- tablice ERF MOD-u;
- wszystkie 116 UTC;
- `Creature List` we wszystkich 170 GIT;
- `module.ifo`, HAK order i obecność 58 HAK-ów;
- 263 399 wpisów zasobów HAK;
- bazowe indeksy KEY NWN;
- efektywne `appearance.2da`;
- rozwiązywanie fixed/simple MDL;
- źródłowe NSS oraz skompilowane NCS;
- skrypty i dialogi przypisane do UTC i instancji;
- lokalny `override` dla wskazanych braków.

Ścisły parser produktu `inspect_module` odrzucił ten legacy MOD błędem:

`M6-GFF-LAYOUT-INVALID at input.fieldIndices: FieldIndices records overlap or leave gaps`

Dlatego dane zostały potwierdzone niezależnym bezpiecznym odczytem tabel
ERF/GFF bez zapisu. Wszystkie 116 UTC i 170 GIT zostały w ten sposób
zdekodowane bez błędu. Niezgodność `inspect_module` jest osobnym problemem
kompatybilności naszego narzędzia z realnym legacy GFF i nie jest dowodem
uszkodzenia modułu.
