# Audyt V7: rama partów i punkt chwytu itemu

Data: 2026-08-04
Status: `VISUAL_ACCEPTANCE_FAILED / NEXT_ITERATION_ADMITTED`
Zakres: exact V7 `m2atgls7.mod` + `m2atglh7.hak`, Item Properties, mieszanie
partów custom/retail oraz wyposażony model w NWN. Audyt offline; agent nie
uruchamiał ani nie kontrolował Aurora Toolset lub NWN.

> Korekta implementacyjna 2026-08-04: `-0.0206093` dla Middle jest pozycją
> kontrolera child Trimesh, a nie minimalnym Y powierzchni trójkątów. Ponowny
> read-only odczyt exact `nwn_base.key -> models_02.bif -> wswls_m_063` daje
> surface bounds Y `[0.042125102, 0.1570407]`. Historyczna tabela poniżej
> połączyła pozycję kontrolera z envelope nagłówka modelu. Nowy profil przechowuje
> controller transform i world-in-model triangle-surface bounds osobno.

## Werdykt

Właściciel poprawnie wskazał brakujący kontrakt. V7 ma prawidłową orientację
pełnej ramy i poprawioną ikonę, ale **nie jest skalibrowany do natywnej ramy
lokalnej longsworda `WSwLs`**.

Obecny fitter buduje własny łańcuch:

```text
Bottom od Y=0 -> Middle po bbox Bottom -> Top po bbox Middle
```

Aurora tak nie układa partów. Każdy retailowy MDL Bottom/Middle/Top ma już
własną pozycję child mesha w jednej wspólnej przestrzeni całego itemu. Toolset
ładuje Bottom i appenduje Middle oraz Top, zachowując ich kontrolery. Ta sama
przestrzeń ma wspólne zero używane przy przyczepieniu wyposażonego itemu do
dłoni postaci.

Skutek V7 jest podwójny:

1. własne party 2/6/2 pasują tylko do siebie;
2. wspólne zero modelu wypada przy dolnym końcu własnego pommela, a nie w
   natywnej strefie chwytu `WSwLs`, więc postać trzyma miecz z przesunięciem.

## Dowód liczbowy

Exact V7 `fit-report.json` deklaruje:

| Slot | Długość docelowa | Zakres Y V7 |
|---|---:|---:|
| Bottom | `0.22` | `[0.000, 0.220]` |
| Middle | `0.08` | `[0.212, 0.292]` |
| Top | `0.90` | `[0.284, 1.184]` |

Kod zaczyna od `cursor=0`, centruje każdy part poprzecznie, przesuwa pierwszy
part tak, aby jego minimum osiowe było równe zeru, a kolejne przesuwa za
kursorem poprzedniej obwiedni. Następnie funkcja connector overlap tylko
nakłada sąsiadów o maksymalnie `0.01`. Źródła:

- `crates/m2a-core/src/item.rs`, `fit_meshy_item_parts_on_target_axis_v3`;
- `crates/m2a-core/src/item.rs`, `finish_item_fit_with_connector_overlap_v2`;
- `crates/m2a-core/examples/materialize_tlc_guard_longsword_v3.rs`, stała
  `FIT_TARGET_AXIAL_LENGTHS = [0.22, 0.08, 0.90]`.

Wcześniejszy, obowiązujący packet retailowy P-REF odczytał bezpośrednio z
`nwn_base.key/models_02.bif` dokładne `wswls_b_023`, `wswls_m_063` i
`wswls_t_023`. Po zastosowaniu ich natywnych kontrolerów child mesha zakresy
wynoszą:

| Slot | Pozycja child mesha | Zakres Y retail |
|---|---:|---:|
| Bottom | `-0.145710` | `[-0.2025382, 0.0841520]` |
| Middle | `-0.0206093` | `[0.042125102, 0.1570407]` |
| Top | `0.262925` | `[0.1283190, 0.9313860]` |

Zatem zero retailowego modelu leży wewnątrz hilt/grip assembly, a nie na jego
skraju. Transformacje są różne per slot; błędu nie naprawi jeden globalny
offset ani zmiana długości samego ostrza.

Źródło: `documentation/evidence/p-ref-item-wswls-parts-2026-07-29.md`.

## Co pokazują screeny właściciela

| Capture | Wynik |
|---|---|
| custom 2/6/2 | model jest widoczny i łączy się tylko we własnym układzie |
| custom Top/Middle + native Bottom | natywny Bottom jest odłączony poniżej |
| custom Top + native Middle/Bottom | custom Top jest odłączony powyżej |
| native 3/7/3 | wszystkie party składają się we wspólnej ramie |
| exact V7 wyposażony w NWN | model jest widoczny, lecz chwyt jest przesunięty względem dłoni |

Jest to świeży, związany z V7 wynik
`modelVisibility=visible`, `proofCompleteness=verified`,
`visualAcceptance=failed`. Trwały packet:
`documentation/evidence/tlc-guard-longsword-v7-owner-slot-frame-result-2026-08-04.json`.

## Gdzie naprawdę zdefiniowany jest chwyt

### Fakt z `baseitems.2da`

Wiersz `BaseItem=1` podaje między innymi `ModelType=2`, `ItemClass=WSwLs`,
`EquipableSlots=0x1C030`, rozmiar itemu, typ/rozmiar broni i namespace. Nie ma
kolumn `GripX/Y/Z`, `PartOffset`, `Socket`, `Pivot` ani per-part transform.

### Fakt z dekompilacji i retail MDL

- Aurora rozwiązuje resrefy z `ItemClass` i trzech wartości UTI.
- Ładuje Bottom, następnie appenduje Middle i Top.
- Nie wylicza dla nich offsetów z `baseitems.2da` lub UTI.
- Pozycja każdego partu jest kontrolerem w jego MDL.
- Ścieżka wyposażenia wybiera prawą/lewą dłoń z equip slotu; model itemu jest
  interpretowany względem wspólnego item-local origin.
- `appearance.2da.WEAPONSCALE` może dodatkowo skalować broń per wygląd
  postaci, ale nie definiuje chwytu ani pozycji trzech partów.

Wniosek: intuicja o potrzebie rozróżnienia per BaseItem jest trafna, lecz
współrzędne chwytu nie są pojedynczym polem w `baseitems.2da`. Pipeline musi
związać semantykę BaseItemu z ramą referencyjną odczytaną z natywnych MDL.

## Braki obecnego pipeline'u

1. `ItemBaseItemV1` rozwiązuje receptę i namespace, ale nie ma
   `attachmentProfile` ani `slotFrameProfile`.
2. Fitter przyjmuje tylko dowolne `target_lengths`; nie przyjmuje natywnego
   profilu Bottom/Middle/Top.
3. Gate sprawdza własne seam/overlap i readback bbox, więc potwierdza zgodność
   z własną błędną receptą zamiast z retailową ramą `WSwLs`.
4. Composer structural gate potwierdza append oraz controller ownership, ale
   nie porównuje pozycji child meshów do profilu referencyjnego.
5. Profil `ITEM_ONLY_GROUND_ITEM_V1` zabrania creature/UTC i nie może
   zweryfikować chwytu wyposażonego modelu.
6. Obecny UI nie pokazuje wspólnego originu, strefy chwytu ani obwiedni
   referencyjnych per slot.

## Plan naprawy

1. Dodać wersjonowany `ItemAttachmentProfileV1`, rozwiązywany z efektywnego
   `BaseItem + ItemClass + ModelType + render route`.
2. Dla `WSwLs` zbudować profil liczbowy z P-REF retailowych MDL: wspólny origin,
   orientacja, target bounds, child translation i connector zone osobno dla
   Bottom/Middle/Top. Profil przechowuje provenance/hash, nie retailowy payload.
3. Zastąpić fitter cursorowy fitterem `REFERENCE_SLOT_FRAME_FIT_V1`:
   orientuje źródło, dopasowuje je do obwiedni jego slotu i wypala wynik w
   natywnej przestrzeni całego itemu. Nie zeruje pierwszego partu i nie układa
   następnych po bbox poprzednika.
4. Rozszerzyć own readback o gate pozycji child controllerów, slot bounds,
   wspólnego originu i mixed-part compatibility.
5. W Studio wyświetlić origin, grip zone, referencyjne slot envelopes i
   ostrzegać, gdy geometria opuszcza kontrakt BaseItemu.
6. Zachować dwa jawnie różne przypadki proof:
   - prawdziwy ground Item do Item Properties;
   - osobny, wyraźnie nazwany equipped witness z **tym samym UTI**, który nie
     zastępuje itemu creature, tylko wymusza renderer trzymania w dłoni.
7. Nie uogólniać ramy dłoni na wszystkie itemy. ModelType 3/CAPART, robe i
   cloak korzystają z profili szkieletu/body-node; ModelType 0/1 musi dostać
   profil zależny od faktycznego attachment route.

## Kryteria ukończenia

### Offline/TDD

- test regresyjny odtwarza dokładne zakresy retail `WSwLs` z P-REF;
- test udowadnia, że stary układ `[0, .22] / [.212, .292] / [.284, 1.184]`
  jest odrzucany dla profilu `WSwLs`;
- każdy wygenerowany part ma controller/bounds zgodny z własnym natywnym
  slotem w tolerancji określonej przez profil;
- wspólny model-local origin pozostaje identyczny dla Bottom/Middle/Top i leży
  w zatwierdzonej strefie grip/hilt;
- testy mieszane custom/native dla każdego pojedynczo wymienionego slotu
  przechodzą conformance bez gapu wynikającego z różnej ramy;
- wydłużenie ostrza może być jawnie dozwolone, ale nie może przesuwać Middle,
  Bottom ani wspólnego originu;
- profil WSwLs nie może zostać użyty przez CAPART robe, cloak ani inną trasę
  attachmentu;
- brak profilu dla equipowalnego BaseItemu kończy się `MANUAL_REQUIRED`, a nie
  cichym użyciem cursor fit.

### Owner proof

- ten sam exact UTI/MDL/HAK jest widoczny i spójny w Item Properties;
- custom 2/6/2 składa się poprawnie;
- trzy próby mieszane — native Bottom, native Middle i native Top — nie
  rozdzielają modelu przez różne układy współrzędnych;
- w NWN dłoń postaci obejmuje zatwierdzoną strefę rękojeści, a guard/pommel
  mają położenie porównywalne z natywnym longswordem;
- sukces samego builda, ikony, ground placementu lub Item Properties nie
  zamyka kryterium wyposażonego chwytu.

## Granica bieżącego audytu

Audyt dopuszcza następną minimalną iterację, ale jej nie tworzy. Nie wykonano
jeszcze TDD, implementacji `ItemAttachmentProfileV1`, nowego HAK/MOD ani
owner proofu. V7 pozostaje widocznym, lecz wizualnie odrzuconym kandydatem.
