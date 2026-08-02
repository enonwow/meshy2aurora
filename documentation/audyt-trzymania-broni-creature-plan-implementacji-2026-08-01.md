# Audyt trzymania broni przez Creature: plan implementacji i kryteria ukończenia

> **Korekta właścicielska 2026-08-02 — wcześniejszy kontrakt został
> zastąpiony.** Demo V1 pokazało model, ale nie pokazało wyposażonej broni.
> Przyczynami były trzy błędne założenia audytu: `MODELTYPE=S` wyłącza
> renderowanie wyposażonych broni; `rhand_g`/`lhand_g` są animowanymi kośćmi
> dłoni, a właściwe hooki przedmiotów to ich nieważone dzieci `rhand`/`lhand`;
> oraz lokalnie syntetyzowany UTI nie został rozwiązany przez Toolset jak
> prawdziwy przedmiot wyposażenia. Aktywny kontrakt V2 to `MODELTYPE=L`,
> hierarchia `RightHand -> rhand` / `LeftHand -> lhand` i istniejący zasób
> bazowy NWN `nw_wswss001`. Wszystkie poniższe twierdzenia sprzeczne z tą
> korektą są zachowane wyłącznie jako historyczny zapis błędnego audytu V1.

## Korekta dowodowa V2

- właścicielski negatywny wynik V1: model widoczny, główny slot broni pusty;
- screenshot wyniku V1:
  `C:\Users\enonw\AppData\Local\Temp\codex-clipboard-d517cded-4d00-467f-8ca7-ab8e05c74f1d.png`,
  SHA-256 `0edc6563f30fcfe672170cbfcb4e71d1fd877b1d5af7b518a4b0d463f0d782a0`;
- screenshot właściciela pokazujący poprawnie wyposażony miecz:
  `C:\Users\enonw\AppData\Local\Temp\codex-clipboard-aaeba59a-c281-4635-a476-d441151ebc61.png`,
  SHA-256 `f570473a8fbef47a0fc96d64e92f2a2f5124b1602e0d8067f42a009290e7543b`;
- read-only kontrola retailowego `c_lich.mdl` potwierdziła hierarchie
  `rforearm_g -> rhand_g -> rhand` i `lforearm_g -> lhand_g -> lhand`;
- dokumentacja `appearance.2da` NWN potwierdza, że `S` nie pokazuje broni,
  natomiast `L` używa animacji Creature i pokazuje broń przy obecności
  `rhand`, `lhand` lub `lforearm`;
- V2 używa dokładnie tego samego oryginalnego GLB o SHA-256
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`,
  bez wariantu facing-matrix; delta dotyczy kontraktu wyposażenia.

Data: 2026-08-01
Status: V1 ODRZUCONY PRZEZ OWNER PROOF; V2 GOTOWY DO OWNER PROOF
Zakres: direct Creature `MODELTYPE=S`, humanoidalny profil Meshy H1, broń
trzymana w prawej lub lewej dłoni, animacje i demonstrator MOD/HAK.

## 1. Werdykt

Obecny pipeline Meshy2Aurora **nie implementuje kompletnego kontraktu
wyposażenia Creature**. Potwierdzoną główną przyczyną tego, że broń dodana do
Creature nie jest trzymana, jest brak natywnych punktów zaczepienia Aurory:

- `rhand_g` dla prawej dłoni;
- `lhand_g` dla lewej dłoni.

Meshy dostarcza w badanym rigu kości `RightHand` i `LeftHand`. Pipeline zachowuje
te nazwy i nie syntetyzuje zgodnych z NWN dummy nodes. Wynikowy MDL ma więc
animowane dłonie, ale nie ma węzłów, pod które runtime może podpiąć model
wyposażenia.

To jest problem systemowy profilu H1, a nie wada wyłącznie badanego modelu.
Każdy model przechodzący przez obecną ścieżkę, którego źródło nie zawiera już
`rhand_g`/`lhand_g`, ma tę samą lukę.

Jednocześnie istnieją dwa osobne braki jakościowe/testowe:

1. generowany fixture UTC ma pusty `Equip_ItemList`, więc obecne demo nie testuje
   wyposażenia od UTI do renderera;
2. źródłowe ataki badanego modelu to Meshy `Punch Combo` i `Left Hook`, a nie
   ruchy z bronią. Po dodaniu anchorów broń może być poprawnie doczepiona, ale
   nadal wyglądać źle podczas ataku.

`HASARMS=1` i `WEAPONSCALE=1` są już emitowane. Nie są rozwiązaniem brakujących
anchorów. `WEAPONSCALE` skaluje poprawnie rozwiązaną broń; nie tworzy punktu
zaczepienia.

## 2. Zakres i metoda

Audyt wykonano read-only na:

- kanonicznym źródle Meshy H1;
- wynikowym binary MDL oraz jego own readbacku;
- kodzie `m2a-core`, WASM/Worker i Meshy Local Bridge;
- generatorze fixture MOD/UTC;
- lokalnej dekompilacji Aurora Toolset;
- retailowym `nwn_base.key`, `appearance.2da` i modelach direct Creature,
  odczytywanych in-place bez kopiowania payloadów do repo;
- referencyjnym, tylko do odczytu katalogu `aurora-web`.

Nie uruchamiano i nie kontrolowano Aurora Toolset ani NWN. Screenshot właściciela
jest obserwacją objawu, nie zastępuje strukturalnych dowodów poniżej.

## 3. Dokładny badany model

| Element | Wartość |
|---|---|
| asset | `tlc-fogbound-claw-guard-h1-p300k-v1` |
| canonical source | `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb` |
| source SHA-256 | `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af` |
| source skin joints | `24` |
| source hand joints | `RightHand`, `LeftHand` |
| source `rhand_g` / `lhand_g` | brak / brak |
| wynikowy model kontrolny | `artifacts/creature-facing-matrix-v1/m2afacepz1.mdl` |
| wynikowy MDL SHA-256 | `a1b0b2046caa67953ced1b07c9e9875eb008319399ec50d852c12ff20fc81290` |
| output rig nodes | `25` = 24 jointy + root modelu |
| output `rhand_g` / `lhand_g` | brak / brak |
| trójkąty przed/po | `297 190` / `297 190` |
| animacje źródłowe / stany NWN | `9` / `42` |

Źródłowa hierarchia dłoni jest poprawna jako szkielet skinningu:

```text
LeftArm -> LeftForeArm -> LeftHand
RightArm -> RightForeArm -> RightHand
```

Brakuje warstwy semantycznej Aurory:

```text
LeftHand  -> lhand_g
RightHand -> rhand_g
```

## 4. Dowody kontraktu Aurora/NWN

### 4.1 Sloty silnika

Lokalna dekompilacja `decompiled_all.c:314206-314298` przekazuje literalne
sloty `RHAND` i `LHAND` do ścieżki wyposażenia jako osobne pozycje 4 i 5.
Referencyjny katalog `aurora-web` mapuje:

- right hand i creature right weapon -> `rhand_g`;
- left hand i creature left weapon -> `lhand_g`;
- amunicję -> `rootdummy`.

Ten katalog jest źródłem pomocniczym, ale mapowanie jest zgodne z dekompilacją
i retailowym corpusem.

### 4.2 Retailowe direct Creature `MODELTYPE=S`

Own-reader wykonany in-place na retailowych modelach potwierdził m.in.:

```text
c_curst
  -> rootdummy
     -> torso_g
        -> rbicep_g -> rforearm_g -> rhand_g
        -> lbicep_g -> lforearm_g -> lhand_g

c_troll
  -> rootdummy
     -> torso_g
        -> rbicep_g -> rforearm_g -> rhand_g
        -> lbicep_g -> lforearm_g -> lhand_g
```

Oba są bezpośrednimi modelami Creature i mają natywne hand anchors. `c_curst`
ma dodatkowo model miecza pod `rhand_g`. Dowodzi to, że `MODELTYPE=S` nie
wyłącza wyposażenia; direct Creature nadal potrzebuje właściwych węzłów dłoni.

### 4.3 Stan obecnego pipeline

`derive_meshy_h1_profile_and_mapping_*` w `crates/m2a-core/src/profile_a.rs`
przenosi nazwy jointów źródłowych do `CreatureRigNodeV1`. Nie dodaje attachment
nodes. Binary scan obu aktualnych wyników potwierdził:

| Plik | `RightHand` | `LeftHand` | `rhand_g` | `lhand_g` | `rootdummy` |
|---|---:|---:|---:|---:|---:|
| `m2afacepz1.mdl` | tak | tak | nie | nie | nie |
| `cm4cx2if4gswhd4d.mdl` | tak | tak | nie | nie | nie |

To nie jest błąd segmentacji geometrii, skinningu ani writerowego limitu
65 535 indeksów. Punkty zaczepienia nie istnieją już w IR, więc writer nie ma
czego zapisać.

## 5. Co jest, a czego nie ma

| Warstwa | Stan | Znaczenie |
|---|---|---|
| humanoidalne kości dłoni | GOTOWE | `RightHand`/`LeftHand` są animowane |
| `MODELTYPE=S` | GOTOWE | właściwy direct Creature route |
| `HASARMS=1` | GOTOWE | Appearance deklaruje ramiona |
| `WEAPONSCALE=1` | GOTOWE | skala wyposażenia jest jawna |
| `rhand_g`, `lhand_g` | BRAK P0 | runtime nie ma natywnego celu attachmentu |
| kalibracja pivotu/orientacji broni | BRAK P0 | sama obecność nazwy nie gwarantuje osi chwytu |
| audyt/readback attachmentów | BRAK P0 | brak raportu i fail-closed walidacji |
| `Equip_ItemList` w fixture | BRAK P0 | obecny generator wpisuje pustą listę |
| typed UTI fixture | BRAK P0 | brak productowego writera minimalnego itemu testowego |
| weapon-aware idle/attack | BRAK P0 | obecne ataki są bezbronne |
| grip geometry preflight | BRAK P0 | H1 nie ma kości palców, a dłoń może być otwarta |
| UI Auto/Off/Manual | BRAK P1 | użytkownik nie widzi ani nie koryguje anchorów |
| right/left owner proof | BRAK P0 | brak exact demo obu dłoni |

## 6. Rozdzielenie odpowiedzialności

### 6.1 Attachment

Attachment odpowiada za to, czy broń istnieje w dłoni i podąża za nią. Naprawa
polega na dodaniu nieobciążonych dummy nodes:

```text
RightHand (istniejący skinned joint)
└── rhand_g (nowy unweighted dummy)

LeftHand (istniejący skinned joint)
└── lhand_g (nowy unweighted dummy)
```

Nie należy zmieniać nazwy `RightHand` na `rhand_g`, bo naruszyłoby to targety
animacji, mapowanie jointów, inverse bind matrices i skinning. Nowy dummy
dziedziczy ruch dłoni bez własnych wag i bez niezależnej animacji.

### 6.2 Chwyt dłoni

Anchor może trzymać broń poprawnie, a palce nadal mogą być otwarte. Badany rig
ma kości całych dłoni, ale nie ma kości palców. Pipeline nie może automatycznie
zacisnąć palców samym przesunięciem `rhand_g`.

V1 musi więc rozróżniać:

- `attachment compatible` — broń jest poprawnie przyczepiona;
- `grip geometry compatible` — geometria dłoni wygląda jak chwyt;
- `grip geometry unsupported` — broń jest przyczepiona, ale źródłowy mesh dłoni
  nie pozwala osiągnąć naturalnego chwytu bez nowego rigu/morpha/modelu.

### 6.3 Animacja z bronią

Obecne mapowanie badanego assetu zawiera:

- Meshy action 198 `Punch Combo` -> `ca1slashl`;
- Meshy action 193 `Left Hook` -> `ca1slashr`.

Nazwy stanów NWN są poprawne, ale choreografia nie jest atakiem bronią. P0 musi
obsłużyć co najmniej jeden zatwierdzony one-handed idle/ready i jeden atak
bronią. Motion retargeting pozostaje osobnym mechanizmem; nie może być używany
do ukrycia braku attachmentu.

## 7. Docelowy kontrakt P0

Rekomendowany kontrakt domenowy:

```text
CreatureHeldEquipmentProfileV1
  mode: Off | OneHandRight | OneHandLeft
  anchorPolicy: Auto | Manual
  rightHandJoint: semantic RightHand
  leftHandJoint: semantic LeftHand
  rightAnchor: rhand_g + local transform
  leftAnchor: lhand_g + local transform
  weaponScale: finite positive scalar
  gripCompatibility: compatible | warning | blocked
  animationProfile: Unarmed | OneHandedWeaponV1
```

Raport `CreatureEquipmentCompatibilityReportV1` powinien zawierać:

- znalezione i brakujące semantic hands;
- dokładne parenty nowych anchorów;
- źródło transformacji: `AUTO_CALIBRATED` albo `MANUAL_OVERRIDE`;
- local/world transform i determinant;
- informację, że node jest dummy i nie należy do skin joint list;
- niezmienione hashe geometrii, materiałów, skinningu i klipów;
- wynik grip preflight;
- status fixture UTI/UTC i wyposażonego slotu;
- listę stanów animacji użytych do audytu trajektorii broni.

`rootdummy` nie jest wymagany do pierwszego testu broni w dłoni. Należy dodać go
dopiero w osobnym zakresie amunicji/effect attachments, aby nie przebudowywać
sprawdzonego root/animation contractu bez potrzeby.

## 8. Plan implementacji

### Etap 0 — zamrożenie kontraktu i korpusu

1. Zamrozić dwa lub więcej retailowych direct `MODELTYPE=S` z prawą/lewą dłonią
   jako hash-only P-REF: hierarchia, typ node'a, local transforms oraz przykładowy
   item hook. Retailowych payloadów nie kopiować do repo.
2. Zamrozić obecny model 297 190 trójkątów jako baseline.
3. Potwierdzić na dokładnym retailowym UTC mapowanie `Equip_ItemList.struct_id`
   dla prawej i lewej dłoni; nie wybierać wartości na podstawie domysłu.
4. Ustalić P0: one-handed right i one-handed left. Dual wield, shield,
   two-handed, bow/crossbow i ammo pozostają poza P0.

Warunek wyjścia: contract fixtures i spodziewane błędy są opisane przed zmianą
IR.

### Etap 1 — semantyczne rozpoznanie dłoni

1. Dodać mapowanie semantic bones z co najmniej `RightHand` i `LeftHand`.
2. Wymagać dokładnie jednego kandydata na każdą włączoną rękę.
3. Fail closed dla braku, duplikatu, cyklu, ręki poza skin hierarchy albo
   nieodwracalnego transformu.
4. Nie opierać mapowania wyłącznie na indeksie jointu.

### Etap 2 — generowanie anchorów w IR

1. Dodać `rhand_g` jako child `RightHand` oraz `lhand_g` jako child `LeftHand`.
2. Zachować jointy, ich ID, wagi i inverse bind matrices bez zmian.
3. Nadać nowym node'om deterministyczne ID i typ dummy.
4. Anchorów nie dopisywać do skin joint list i nie tworzyć dla nich wag.
5. Włączać je po rozstrzygnięciu basis/facing, aby osie nie rozjeżdżały się po
   zmianie kierunku przodu Creature.

### Etap 3 — kalibracja transformacji chwytu

1. Zbudować wersjonowany `CreatureWeaponAnchorCalibrationProfileV1` na wielu
   retailowych direct Creature, nie na jednym modelu.
2. Dla `Auto` obliczyć frame dłoni z rest pose i łańcucha
   forearm -> hand, z uwzględnieniem Creature basis.
3. Nie uznawać identity transformu za poprawny bez porównania z item hookiem.
4. Udostępnić `Manual` translation + rotation osobno dla każdej dłoni; skala
   pozostaje w `WEAPONSCALE`, nie w sheared anchor matrix.
5. Raportować dokładną deltę Auto/Manual i blokować NaN, reflection,
   non-uniform scale oraz shear.

### Etap 4 — writer i own readback

1. Binary MDL writer zapisuje nowe dummy nodes w modelu bazowym.
2. Animacje nie potrzebują osobnych controllerów anchorów: anchor ma dziedziczyć
   ruch parent hand. Reader musi jednak potwierdzić rozwiązanie tej hierarchii
   dla efektywnego drzewa każdego stanu.
3. Semantic readback wymaga dokładnie jednego case-insensitive `rhand_g` i
   `lhand_g`, poprawnych parentów oraz stałego local transformu.
4. Native i WASM muszą zwrócić byte-identical wynik oraz identyczny raport.

### Etap 5 — weapon-aware animation i grip gate

1. Dodać jawny profil `OneHandedWeaponV1` zamiast automatycznie traktować
   `Punch Combo`/`Left Hook` jako ataki bronią.
2. Co najmniej idle/ready, walk, run, attack, damage i death muszą zachować
   ciągłość anchorów.
3. Attack wymaga źródłowej albo retargetowanej animacji stworzonej dla broni.
4. Grip preflight klasyfikuje dłoń i nie obiecuje zaciśniętych palców przy rigu
   bez finger bones.
5. Event `hit` pozostaje przypisany do kinematycznie wyznaczonego momentu
   uderzenia.

### Etap 6 — typed UTI i fixture UTC

1. Dodać minimalny typed UTI fixture oparty o wybrany stockowy `BaseItem` i
   zgodne `ModelPart*`; nie kopiować retailowego UTI.
2. Sparametryzować generator Creature fixture tak, aby `Equip_ItemList` nie był
   zawsze pusty.
3. Utworzyć dwa obiekty kontrolne w jednym demo: broń prawa i broń lewa.
4. Own GFF readback musi potwierdzić `EquippedRes`, struct ID slotu i istnienie
   dokładnego UTI w efektywnym namespace.
5. Normalny fixture bez broni nadal pozostaje wspierany i deterministyczny.

### Etap 7 — Studio

Minimalny panel Creature:

- `Held weapon support`: `Off | Right hand | Left hand`;
- `Weapon anchor`: `Auto | Manual`;
- gizmo translation/rotation dla wybranego anchoru;
- proxy broni w podglądzie, wyraźnie oznaczone jako preview;
- status `attachment`, `grip geometry`, `weapon animation`;
- czytelne blokery Build dla brakującej/niejednoznacznej ręki;
- wynikowe nazwy `rhand_g`/`lhand_g` i transformacje w raporcie Review.

### Etap 8 — testy offline i demonstrator

1. Testy syntetyczne: seamy/duplikaty nazw, brak ręki, zły parent, cykl,
   reflection, scale/shear i deterministyczność.
2. Regresja exact badanego modelu 297 190 trójkątów.
3. Macierz wszystkich obsługiwanych Creature facing/basis.
4. Testy 42 stanów i trajektorii anchorów.
5. Test Native/WASM/Worker/Studio -> core.
6. Po zielonych bramkach przygotować exact MOD/HAK, zainstalować hash-verified do
   natywnych katalogów i zatrzymać się na `ready_for_owner_proof`.
7. Aurora Toolset/NWN testuje właściciel zgodnie z `AGENTS.md`.

## 9. Kryteria ukończenia

### 9.1 Struktura modelu

- Wynik ma dokładnie jeden case-insensitive `rhand_g` i jeden `lhand_g`.
- `rhand_g.parent == RightHand`, `lhand_g.parent == LeftHand` po semantic
  mappingu, a nie po przypadkowym indeksie.
- Oba są dummy nodes, nie skin joints i nie mają vertex weights.
- Local transform jest skończony, odwracalny, bez reflection, non-uniform scale
  i shear.
- Zmiana Creature facing nie odwraca strony dłoni ani osi broni.

### 9.2 Nienaruszalność modelu

- Dla exact baseline liczba trójkątów pozostaje `297 190`.
- Semantyczne hashe POSITION, indices, NORMAL, TANGENT, UV, materiałów,
  JOINTS, WEIGHTS i inverse bind matrices są identyczne przed/po.
- Liczba source joints pozostaje `24`, active joints pozostaje `22`.
- Output rig rośnie tylko o dwa jawnie raportowane dummy nodes: z `25` do `27`.
- Wszystkie 42 nazwy stanów, czasy, eventy i dotychczasowe lineage pozostają
  niezmienione poza jawnie wybranymi weapon-aware klipami.
- Segmentacja modelu 300 000 trójkątów i stabilizacja accessories nie zmieniają
  wyniku anchorów.

### 9.3 Zachowanie animacji

- Anchor ma stały local transform względem dłoni we wszystkich próbkach każdego
  z 42 stanów; nie odłącza się, nie skaluje i nie wykonuje axis flip.
- Broń podąża za dłonią w `cpause1`, `cwalk`, `crun`, wybranym ataku,
  damage/knockdown/death i terminalnym `cdead`.
- Co najmniej jeden atak P0 pochodzi z weapon-aware motion, nie z punch/hook
  tylko przemianowanego na `ca1slash*`.
- `hit` wypada w wyznaczonym momencie ruchu broni.
- Nie powstaje dodatkowy skok root/hips ani regresja ciągłości zgonu.

### 9.4 Equipment resources

- Typed UTI przechodzi własny writer/readback i wskazuje istniejący stockowy
  wygląd broni.
- UTC `Equip_ItemList` zawiera poprawny, potwierdzony native struct ID oraz
  `EquippedRes` dokładnie wskazujący UTI.
- HAK/MOD manifest i readback potwierdzają wszystkie wymagane zasoby.
- Fixture bez wyposażenia nadal ma pustą listę i przechodzi istniejące testy.

### 9.5 Studio i diagnostyka

- Użytkownik może wybrać Off/Right/Left oraz Auto/Manual bez edycji kodu.
- Podgląd pokazuje dokładnie tę samą transformację anchoru, którą buduje core.
- Build blokuje brak/duplikat ręki i nie stosuje cichego fallbacku do root modelu.
- Raport rozróżnia: anchor poprawny, grip geometry warning i niewłaściwą
  animację broni.
- Dwa identyczne buildy dają byte-identical MDL/HAK/MOD i raport.

### 9.6 Human-owned proof

Exact demonstrator musi pokazać dwa Creature: weapon right oraz weapon left.
Właściciel potwierdza osobno w Aurora Toolset i NWN:

- model Creature jest widoczny;
- właściwa broń jest widoczna w odpowiedniej dłoni;
- broń znajduje się w obrębie dłoni, a nie przy root/floor/origin;
- nie jest lustrzanie obrócona ani odwrócona ostrzem/uchwytem;
- podąża za dłonią w idle, walk, run, attack, reaction i death;
- nie odłącza się, nie pływa i nie rozciąga;
- `WEAPONSCALE` daje oczekiwaną zmianę skali;
- dla modelu bez finger bones wynik gripu jest oceniany zgodnie z raportem, a
  nie fałszywie uznany za pełny finger grip.

Dopiero pozytywny wynik właściciela zamyka etap wizualny. Granicą pracy agenta
pozostaje `ready_for_owner_proof`.

## 10. Priorytety

### P0 — wymagane, aby powiedzieć „Creature trzyma broń”

1. semantic hand mapping;
2. `rhand_g`/`lhand_g` w IR i writerze;
3. kalibracja Auto + readback;
4. typed UTI i wyposażony UTC fixture;
5. right/left demo;
6. co najmniej jeden weapon-aware attack;
7. pełne invarianty i owner proof.

### P1 — produkcyjne authoring UX

- manual anchor gizmo;
- zapis presetów per skeleton profile;
- grip compatibility visualization;
- dual wield i shield;
- integracja z biblioteką motion packs.

### P2 — rozszerzone typy broni

- two-handed IK;
- bow/crossbow i współpraca obu dłoni;
- ammo/quiver przez `rootdummy`;
- finger rig/morph correction;
- różne profile szkieletów poza Meshy H1.

## 11. Definition of Done w jednym zdaniu

Trzymanie broni przez direct Creature jest gotowe wtedy, gdy pipeline dodaje
audytowalne `rhand_g`/`lhand_g` bez zmiany geometrii lub skinningu, generuje
poprawne UTI/UTC wyposażenia, zachowuje anchor w pełnych 42 stanach i
weapon-aware ruchach, a właściciel potwierdza exact right/left demonstrator w
Aurora Toolset oraz NWN.

## 12. Odpowiedź praktyczna

Tak, funkcję można zaimplementować w naszej aplikacji bez ponownego generowania
modelu Meshy. Najpierw należy naprawić kontrakt anchorów. Dopiero potem ma sens
oceniać pivot, wygląd chwytu i animację ataku. Samo ustawienie broni w UTC albo
zmiana `WEAPONSCALE` nie naprawi bieżącego modelu, ponieważ punkt zaczepienia w
MDL nie istnieje.

## 13. Status implementacji 2026-08-01

Zaimplementowany został pierwszy produkcyjny pionowy wycinek attachmentów broni:

- aktywne trasy produktu Creature rozpoznają semantyczne `RightHand` i
  `LeftHand` bez polegania na indeksie jointu;
- pipeline dopisuje deterministyczne, nieobciążone wagami dummy nodes
  `RightHand -> rhand_g` i `LeftHand -> lhand_g`;
- local transform obu anchorów jest wersjonowany jako
  `MESHY_H1_HAND_BONE_IDENTITY_CHILD_V1`; API dopuszcza ręczną sztywną macierz,
  ale blokuje NaN, reflection, skalę niejednorodną i shear;
- modele bez obu semantycznych dłoni zachowują zgodność wsteczną; gdy pipeline
  wykryje tylko jedną dłoń albo niejednoznaczność, kontrakt fail-closed;
- writer MDL oraz własny reader potwierdziły dokładnych parentów w modelu
  bazowym i obecność obu anchorów w każdym z 42 drzew animacji;
- `GffFileTypeV1` obsługuje typed `UTI `;
- generator demonstratora tworzy własny longsword UTI (`BaseItem=1`,
  `ModelPart*=11`) oraz dwa Creature: prawy slot UTC/GIT `struct_id=16` i lewy
  slot `struct_id=32`;
- własny MOD readback wymaga zgodności `Equip_ItemList` pomiędzy GIT i UTC oraz
  wskazania dokładnego UTI w archiwum;
- brak-broni w istniejących fixture pozostaje obsługiwany przez niezmienioną
  ogólną trasę modułu.

Exact model `tlc-fogbound-claw-guard-h1-p300k-v1` zachował 297 190 trójkątów,
22 aktywne jointy i 42 klipy. Demonstrator ma status `ready_for_owner_proof`;
szczegóły i hashe są w
`documentation/evidence/creature-weapon-anchor-v1-ready-for-owner-proof-2026-08-01.md`.

Świadomie niezamknięte elementy pozostają oddzielne od samego attachmentu:

- wizualna kalibracja kąta/położenia identity-child wymaga testu właściciela w
  Aurora Toolset i NWN;
- obecny atak źródłowy nadal jest ruchem unarmed Meshy, więc kryterium
  weapon-aware attack nie jest jeszcze spełnione;
- brak panelu Studio `Off | Right | Left` i `Auto | Manual`;
- źródłowy rig nie ma finger bones, więc pipeline nie deklaruje pełnego gripu
  palców.
