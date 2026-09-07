# Audyt warstw animacji broni Aurora

Data: 2026-08-27

## Werdykt

Obserwacja właściciela jest poprawna: kusza, łuk, miecz i tarcza nie zastępują
podstawowych animacji lokomocji postaci. Retailowy humanoid składa pełną bazę
`pause1` / `walk` / `run` z animacją ograniczoną przez `animroot` do poddrzewa
tułowia albo ręki. Pełnocielesne `xbowrdy` i `xbowshot` są osobnymi stanami
gotowości i akcji, a nie stałym zamiennikiem chodu.

V14 złamał ten kontrakt. Lokalny `pfh0.mdl` przesłonił `pause1`, `xbowrdy` i
`xbowshot` klipami deklarującymi `animroot rootdummy`, lecz każdy klip zawierał
tylko 10 kontrolerów górnej części ciała. Taki klip zgłasza zasięg całej postaci,
ale nie dostarcza pełnych danych dla miednicy i nóg. To jest bezpośrednia
przyczyna zdeformowanego chodu oraz różnicy między podglądem aplikacji i NWN.

## Zakres i ograniczenia dowodu

Audyt wykonano offline. Nie uruchamiano ani nie kontrolowano Aurora Toolset ani
NWN. Sprawdzono:

- dokładne retailowe zasoby z instalacji NWN:EE przez lokalny odczyt KEY/BIF;
- łańcuch supermodeli `pfh0 -> a_fa -> a_ba -> a_ba_non_combat ->
  a_ba_med_weap -> a_ba_custom`;
- strukturę klipów, `animroot`, czasy przejść i listy kontrolowanych węzłów;
- parser i interpolację kontrolerów w dostępnej dekompilacji Aurora Toolset;
- dokładny eksport postaci V14 oraz kod generujący jego klipy.

Dostępna dekompilacja Toolsetu nie zawiera zamkniętego routera stanów klienta
NWN. Dowodzi ona parsowania `length`, `transtime`, `animroot` i mieszania
kontrolerów, ale nie pozwala uczciwie twierdzić, że odtworzono pełną tablicę
decyzyjną klienta. Role stanów poniżej wynikają z exact retail assets i
kanonicznego przewodnika Custom Content Guide; końcową zgodność runtime musi
potwierdzić właściciel na następnym dopuszczonym kandydacie.

## Dokładna linia retailowa

| Zasób | SHA-256 | Rola |
| --- | --- | --- |
| `pfh0.mdl` | `1da54afc5ff270c8c7decc61829df0595c6568d70cdef75026f63d8e94362f51` | model postaci, supermodel `a_fa` |
| `a_fa.mdl` | `1f8b1ac2e4ab31e1b2172c7ca208160267cdbf861cd36e544775f391b5b6978d` | żeńskie animacje bazowe i część warstw broni |
| `a_ba.mdl` | `a38c9f0de00a7e401419d1e668ad1b9d825cfc9635712cdb9ae63e840104ee2b` | wspólne humanoidalne stany walki |
| `a_ba_non_combat.mdl` | `6222498236772cb30d44d92ed4819842f9a0b17d95a0bbfe081d447c66a7f717` | stany spoczynku i warstwy bez walki |
| `a_ba_med_weap.mdl` | `9894151ab16d5439e2fedc500c9b44c4e03cad664e8e4e7e6883e1072a82419d` | wspólne animacje broni średniej |

## Jak retail składa ruch i broń

| Stan | Rozwiązane źródło | `animroot` | Kontroluje dół ciała | Znaczenie |
| --- | --- | --- | --- | --- |
| `pause1` | `a_fa` | `rootdummy` | tak | pełna baza bezczynności |
| `walk` | `a_fa` | `rootdummy` | tak | pełna baza chodu |
| `walk_bowl` | `a_fa` | `torso_g` | nie | warstwa broni na bazowym chodzie |
| `run` | `a_fa` | `rootdummy` | tak | pełna baza biegu |
| `run_bowl` | `a_fa` | `lbicep_g` | nie | warstwa lewej ręki na bazowym biegu |
| `xbowr` | `a_ba_non_combat` | `rforearm_g` | nie | kusza trzymana w spoczynku |
| `xbowrdy` | `a_ba` | `rootdummy` | tak | pełny stan gotowości kuszy |
| `xbowshot` | `a_ba` | `rootdummy` | tak | pełna akcja strzału z kuszy |

Podstawowe `pause1`, `walk` i `run` kontrolują `rootdummy`, miednicę, nogi i
ręce. Warstwy ruchowe broni mają celowo węższy zasięg:

- `walk_shieldl`, `walk_swordl`, `walk_swordr` i `walk_bowl` startują w
  `torso_g` i nie zawierają kontrolerów miednicy ani nóg;
- odpowiedniki biegu także nie sterują dołem ciała; `run_bowl` ogranicza się
  do sześciu węzłów lewej ręki;
- `xbowr` kontroluje jedynie `rforearm_g`, `rhand_g` i `rhand`;
- `torchl` również jest warstwą ręki, a nie nową animacją całej postaci.

To nie jest przypadkowy brak ścieżek. Pole `animroot` określa punkt wejścia
animacji w hierarchii i umożliwia odtwarzanie innej animacji na reszcie modelu.
Właśnie dlatego retailowa broń nie niszczy podstawowego ruchu nóg.

## Co dokładnie zrobił V14

Dokładny niezaakceptowany kandydat:

- MOD `m2aimod35d.mod`, SHA-256
  `ce81eb3a2baf8e84bc851ec820f75e0ab0200eb634576c8ff21f6e7a59e1e52e`;
- HAK `m2aihak35d.hak`, SHA-256
  `473a9b5238a290c041192037ec23c3c032a9f18591a512c8eff8cfb8c0098082`;
- lokalny carrier `pfh0.mdl`, SHA-256
  `42c5f0fadb3068c22c47851e40ed518b2dc627c0e90046b29b6b79b90253eb8d`.

`firearmRuntimeAnimation.ts` mapuje:

| Klip autorski | Lokalna nazwa runtime | Zapisany `animroot` |
| --- | --- | --- |
| `firearm_idle_custom` | `pause1` | `rootdummy` |
| `firearm_ready_custom` | `xbowrdy` | `rootdummy` |
| `firearm_shot_custom` | `xbowshot` | `rootdummy` |

Każdy z tych klipów eksportuje te same 10 węzłów:
`torso_g`, `rbicep_g`, `rforearm_g`, `rhand_g`, `rhand`, `lbicep_g`,
`lforearm_g`, `lhand_g`, `lhand`, `neck_g`. Brakuje `rootdummy`, miednicy,
ud, goleni i stóp.

| Kontrakt retail | Eksport V14 | Skutek |
| --- | --- | --- |
| `pause1` jest pełną bazą | lokalny `pause1` ma tylko górę ciała, ale `rootdummy` | niepełne przesłonięcie bazowego idle |
| `walk/run` pozostają bazą, broń jest osobną warstwą | brak jawnego kontraktu `walk + walk_bowl` i `run + run_bowl` | podgląd nie odpowiada routingowi gry |
| spoczynek kuszy używa `xbowr` o zasięgu przedramienia | V14 nie eksportuje `xbowr`, tylko zastępuje `pause1` | złe rozdzielenie postawy i ruchu |
| `xbowrdy/xbowshot` retailowo są pełnocielesne | V14 zapisuje górę ciała pod `rootdummy` | niekompletna gotowość i akcja |

`WeaponWield=6` wybiera rodzinę zachowania kuszy. Samo przypisanie tego pola nie
naprawia błędnych lokalnych klipów, ponieważ lokalny `pfh0` przesłania klipy
odziedziczone z poprawnego łańcucha supermodeli. Co więcej, `pfh0` jest
współdzielonym carrierem żeńskiego człowieka phenotype 0, więc takie
przesłonięcie nie jest własnością samego przedmiotu — wpływa na postać używającą
tego modelu.

## Poprawny kontrakt następnej implementacji

1. Nie eksportować autorskiego `firearm_idle_custom` jako lokalnego `pause1`.
   Retailowe pełne `pause1`, `walk` i `run` muszą pozostać odziedziczone i
   niezmienione.
2. Rozdzielić bazę ruchu od warstwy broni:
   `pause1 + xbowr`, `walk + walk_bowl`, `run + run_bowl`.
3. Warstwa custom grip musi mieć najwęższy poprawny `animroot` i wyłącznie
   kontrolery należące do tego poddrzewa. Nie może deklarować `rootdummy`, jeżeli
   nie jest kompletną animacją całej postaci.
4. Jeżeli `xbowrdy` albo `xbowshot` mają zostać zmienione, trzeba wygenerować
   własne kompletne klipy pełnego ciała, zachowujące potwierdzony retailowym
   świadkiem układ i pokrycie korzenia, miednicy oraz nóg. Retail służy wyłącznie
   do odczytu kontraktu; jego keyframe'ów ani payloadu animacji nie kopiujemy.
   Retailowe odpowiedniki mają `animroot rootdummy`.
5. Nie przesłaniać globalnego `pfh0`, jeśli zmiana ma być przypisana wyłącznie
   do nowej broni. Potrzebny jest dedykowany, jawnie wybrany carrier/
   phenotype/appearance albo świadomie zaakceptowany globalny zakres rodziny
   broni.
6. Podgląd aplikacji musi czytać dokładny wyeksportowany MDL i składać te same
   stany bazowe i warstwy co eksport. Kompozycja tylko z autorskich klipów
   Three.js nie może być przedstawiana jako parity proof NWN.

## Kryteria zakończenia poprawki

- Eksporter odrzuca każdy klip z `animationRoot=rootdummy`, który nie zawiera
  wymaganych kontrolerów korzenia, miednicy i nóg dla pełnocielesnego stanu.
- Eksporter potwierdza, że `pause1`, `walk` i `run` nie zostały lokalnie
  przesłonięte przez częściowe klipy broni.
- Warstwy chodu i biegu nie zawierają żadnych kontrolerów miednicy, uda, goleni
  ani stopy i mają `animroot` zgodny z kontrolowanym poddrzewem.
- Readback dokładnego MDL pokazuje jawny kontrakt stanów:
  `pause1 + xbowr`, `walk + walk_bowl`, `run + run_bowl`, `xbowrdy`,
  `xbowshot`.
- Readback `xbowrdy` i `xbowshot` potwierdza kompletne ścieżki pełnego ciała
  albo implementacja przedstawia osobny, zaakceptowany dowód, że konkretny
  częściowy zakres tych slotów działa w kliencie.
- Aplikacja nie używa osobnej, niewyeksportowanej definicji pozy do deklarowania
  zgodności; podgląd pochodzi z exact binary/semantic readback.
- Testy negatywne blokują ponowne utworzenie kontraktu V14: 10 ścieżek górnej
  części ciała pod `rootdummy` dla `pause1`, `xbowrdy` lub `xbowshot`.
- Dopiero po przejściu powyższych bramek można materializować następnego
  dopuszczonego kandydata. Końcowy werdykt wizualny należy do właściciela.

## Źródła

- retailowe MDL z lokalnej instalacji NWN:EE, hashe podane wyżej;
- `C:\Projects\New Folder\export\decompiled_all.c`: parser `length`,
  `transtime`, `animroot` w `FUN_00a5d758` oraz interpolacja kontrolerów w
  `FUN_00a1999c`;
- [NWN Wiki: MDL ASCII](https://nwn.wiki/spaces/NWN1/pages/12027273/MDL%2BASCII),
  opis znaczenia `animroot` i animacji części modelu;
- [Custom Content Guide v3.0](https://www.neverwintervault.org/project/nwn1/other/custom-content-guide-v30),
  hierarchia supermodeli i role `xbowr`, `xbowrdy`, `xbowshot`;
- `documentation/evidence/tlc-hextech-ranged-demo-v14-owner-runtime-parity-failure-2026-08-27.md`;
- `apps/studio-web/src/features/preview/firearmRuntimeAnimation.ts` w
  zarejestrowanym worktree `items-agent-remediation-final`.

## Stan po audycie

Audyt nie tworzy V15 ani żadnego nowego MOD/HAK/modelu. Zamrożona geometria
broni pozostaje bez zmian. Dopuszczony następny zakres naprawy dotyczy wyłącznie
kontraktu routingu i warstw animacji oraz zgodności podglądu z exact export
readback.
