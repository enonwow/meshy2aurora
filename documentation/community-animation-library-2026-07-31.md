# Repozytoryjna biblioteka animacji Meshy2Aurora — 2026-07-31

Status: `TECHNICAL MVP IMPLEMENTED / OWNER GATES OPEN`

Wykonawczy audyt, fazowy plan i mierzalne kryteria ukończenia znajdują się w
[`audyt-plan-biblioteki-animacji-2026-07-31.md`](audyt-plan-biblioteki-animacji-2026-07-31.md).
Audyt doprecyzowuje, że agregat katalogu jest generowany do `contracts`, a
repozytoryjny preset jest przenośnym formatem innym niż source-bound
`AuthoredAnimationClipV1`.

Aktualny stan wykonania, hashe i wyniki bramek są zapisane w
[`animation-library-implementation-evidence-2026-07-31.md`](animation-library-implementation-evidence-2026-07-31.md).
Otwarte pozostają decyzja właściciela o root `LICENSE`/publicznym contribution
oraz owner-reported NWN result dla dokładnego presetu startowego.

Decyzja właściciela: własne i społecznościowe animacje mają być częścią
repozytorium, aby użytkownicy mogli przygotowywać je w Meshy2Aurora, wysyłać
pull requesty, a aplikacja mogła udostępniać zaakceptowany katalog.

## Cel

Biblioteka ma zmienić pojedyncze presety animacji w wersjonowany, przeszukiwalny
i wielokrotnie używalny katalog. Contributor tworzy animację w aplikacji,
eksportuje bezpieczną paczkę danych i otwiera PR. Po walidacji i merge animacja
pojawia się w następnym wydaniu Meshy2Aurora.

Pierwszym zaimplementowanym presetem jest ruch prawego prostego wyprowadzony z
`m2a_voidcleave`. Zamrożony klip i lineage `vckcleave2` pozostają bez zmian;
osobny zasób biblioteczny ma stabilne ID `m2a_right_cross`.

## Źródła animacji w aplikacji

| Źródło | Własność | Dostępność | Edycja |
|---|---|---|---|
| `Built-in` | utrzymywane przez projekt | dołączone do wydania | przez kopię do `Custom` |
| `Community` | zaakceptowane PR-y | dołączone do wydania | przez kopię do `Custom` |
| `Custom` | użytkownik/projekt | lokalna warstwa projektu | bezpośrednia |

Preset z `Built-in` lub `Community` jest niezmiennym szablonem. Polecenie
`Use as template` tworzy edytowalną kopię w `Custom`; nie zmienia wpisu
repozytoryjnego ani źródłowego GLB.

## Proponowany układ repozytorium

```text
animation-library/
├── tags-v1.json
├── rig-profiles/
│   └── m2a-humanoid-strict-v1.json
└── presets/
    └── m2a_right_cross/
        ├── manifest.json
        ├── animation.json
        ├── README.md
        └── preview.webp
```

Deterministyczny agregat jest generowany do
`contracts/community-animation-catalog-v1.json`; nie istnieje drugi ręcznie
utrzymywany katalog.

Do repozytorium trafiają dane ruchu i mały podgląd. Paczka nie zawiera modelu
GLB, tekstur, retailowych assetów NWN, kodu wykonywalnego ani skryptów.

## Kontrakt manifestu

Minimalny manifest:

```json
{
  "schemaVersion": 1,
  "presetId": "m2a_right_cross",
  "presetVersion": 1,
  "outputName": "m2a_rightcross",
  "label": "Right cross",
  "source": "BUILT_IN",
  "authors": [{ "name": "Meshy2Aurora contributors" }],
  "license": "LicenseRef-Meshy2Aurora-Project-Generated",
  "tags": ["attack", "boxing", "humanoid", "one-shot", "right-hand", "unarmed", "upper-body"],
  "rigProfile": "M2A_HUMANOID_STRICT_V1",
  "rigSignatureSha256": "<sha256>",
  "durationSeconds": 0.85,
  "playback": "ONE_SHOT",
  "motionSha256": "<sha256>",
  "animationPath": "animation.json",
  "animationSha256": "<sha256>",
  "previewPath": "preview.webp",
  "previewSha256": "<sha256>",
  "validationStatus": "PIPELINE_VERIFIED"
}
```

Wersja manifestu, stabilne ID i `motionSha256` są obowiązkowe. Zmiana ruchu
zwiększa wersję presetu; opublikowanej pary `id + version` nie wolno po cichu
nadpisywać inną zawartością.

## Tagi i wyszukiwanie

Tagi pochodzą z wersjonowanego, kontrolowanego słownika
`animation-library/tags-v1.json`. V1 zawiera dokładnie 17 wartości:
`attack`, `boxing`, `combat`, `defense`, `dodge`, `full-body`, `guard`,
`humanoid`, `left-hand`, `locomotion`, `loop`, `one-shot`, `right-hand`,
`sword`, `unarmed`, `upper-body`, `utility`. Core wymaga dokładnie tego
posortowanego zbioru; rozszerzenie słownika wymaga nowego kontraktu wersji.

Wyszukiwanie w Studio obejmuje nazwę, ID, autora i tagi. UI pokazuje klikalne
chipy tagów oraz filtry kategorii. Nieznany tag blokuje PR do czasu dodania go
do kontrolowanego słownika albo zastąpienia istniejącym terminem.

## Zgodność z modelem

Każdy preset deklaruje `rigProfile` i wymagane kości. Aplikacja przed użyciem:

1. porównuje profil oraz sygnaturę rigu;
2. rozróżnia `COMPATIBLE` i `INCOMPATIBLE`;
3. nie stosuje po cichu animacji do niezgodnego szkieletu;
4. w V1 nie wykonuje ukrytego retargetingu; zgodny preset tworzy nową kopię
   `Custom` i zapisuje provenance;
5. zachowuje źródłową geometrię, materiały, tekstury i GLB bez zmian.

`requiredBones` nie jest deklaracją uznaniową: musi być dokładnie zbiorem root
bone i wszystkich targetów tracków. Brakująca albo nadmiarowa kość blokuje
preset.

## Workflow contributora

1. Utworzyć albo edytować animację w Animation Studio.
2. Zapisać ją do `Custom` i przejść walidację lokalną.
3. Wybrać `Export contribution`.
4. Uzupełnić nazwę, autora, licencję, tagi i opis ruchu.
5. Aplikacja generuje katalog presetu z manifestem, animacją, hashem i
   podglądem.
6. Contributor dodaje paczkę do `animation-library/presets/<id>` i otwiera PR.
7. CI weryfikuje kontrakt, deterministyczny readback i podgląd.
8. Po merge generator katalogu dodaje preset do kolejnego wydania aplikacji.

## Walidacja CI

Każdy PR z animacją musi przejść co najmniej:

- [x] walidację JSON Schema manifestu i danych animacji;
- [x] unikalność oraz format stabilnego ID;
- [x] zgodność ID, wersji, ścieżek i wpisu w katalogu;
- [x] rosnące czasy klatek mieszczące się w długości klipu;
- [x] skończone liczby oraz znormalizowane quaterniony;
- [x] istnienie wymaganych kości zadeklarowanego profilu;
- [x] deterministyczny `motionSha256` i identyczny core readback;
- [x] brak zmian geometrii i brak payloadów modelu/tekstur;
- [x] limit rozmiaru paczki i dozwolone typy plików;
- [x] deklarację autora, licencji oraz provenance;
- [x] render podglądu przez pipeline Meshy2Aurora;
- [x] materializację do binary MDL na syntetycznej fixture kompatybilnego rigu;
- [x] test katalogu w Studio, Worker/WASM i produkcyjnym buildzie.

Contribution jest wyłącznie deklaratywnymi danymi. Repozytorium nie wykonuje
skryptów dostarczonych przez autora presetu.

Contribution może otrzymać wyłącznie status `PIPELINE_VERIFIED`. Promocja do
`OWNER_NWN_VERIFIED` następuje osobno i wymaga wpisu owner proof registry
związanego z dokładnym preset ID, wersją, motion SHA i rig SHA.

## Dostarczanie katalogu do aplikacji

### Wersja podstawowa

Generator uruchamiany w buildzie czyta `animation-library`, waliduje wszystkie
presety i tworzy statyczny katalog dołączony do aplikacji. Studio działa dzięki
temu offline, a jedna wersja aplikacji zawsze widzi identyczny zestaw animacji.

Aplikacja nie pobiera arbitralnej zawartości z gałęzi GitHub przy każdym
uruchomieniu. Chroni to tryb local-first, powtarzalność buildów oraz użytkownika
przed zmianą lub usunięciem danych po publikacji wydania.

### Opcjonalna wersja późniejsza

Można dodać ręczne `Check for library updates`, ale wyłącznie dla:

- podpisanego manifestu GitHub Release;
- wersjonowanej, niezmiennej paczki;
- sprawdzonego SHA-256 każdego pliku;
- jawnej zgody użytkownika na pobranie;
- lokalnego cache z możliwością powrotu do katalogu wbudowanego.

Awaria sieci albo podpisu nie może blokować aplikacji ani usuwać wbudowanej
biblioteki.

## Pierwszy zestaw utrzymywany przez projekt

- [x] `m2a_right_cross` — prawy prosty;
- [x] `m2a_left_jab` — szybki lewy prosty;
- [x] `m2a_right_hook` — prawy sierpowy;
- [x] `m2a_uppercut` — cios podbródkowy;
- [x] `m2a_combat_guard` — zapętlona garda;
- [x] `m2a_dodge_left` — unik w lewo;
- [x] `m2a_dodge_right` — unik w prawo.

## Etapy implementacji

### Faza 1 — kontrakty repozytorium

- [x] utworzyć `animation-library` i schematy V1;
- [x] zdefiniować kontrolowany słownik tagów;
- [x] dodać parser, walidator oraz generator katalogu;
- [x] dodać negatywne testy niedozwolonych plików i błędnego provenance;
- [ ] ustalić licencję contribution przed publicznym przyjmowaniem PR-ów.

### Faza 2 — integracja produktu

- [x] dodać źródła `Built-in`, `Community` i `Custom` do biblioteki Studio;
- [x] dodać wyszukiwanie po tagach oraz filtry;
- [x] dodać kontrolę kompatybilności rigu;
- [x] dodać `Use as template` z pełnym provenance;
- [x] przekazać jeden kontrakt przez Studio → Worker → WASM → core.

### Faza 3 — eksport contribution

- [x] dodać `Export contribution`;
- [x] generować manifest, animation payload, hash i README;
- [x] generować pipeline-bound preview;
- [x] dodać szablon PR oraz checklistę autora;
- [x] dodać pełne CI contribution na pull requestach; zewnętrzny wynik czeka
  na push brancha.

### Faza 4 — biblioteka startowa

- [x] wyprowadzić `m2a_right_cross` jako nowy preset bez zmiany `vckcleave2`;
- [x] przygotować i zweryfikować offline pozostałe animacje startowe;
- [x] wykonać testy wyszukiwania, kopiowania, edycji i eksportu MDL;
- [x] udokumentować authoring nowych presetów;
- [ ] przypisać owner-reported NWN result do dokładnego startera.

### Faza 5 — podpisane aktualizacje, opcjonalnie

- [ ] zdefiniować format signed release manifest;
- [ ] dodać ręczne sprawdzanie aktualizacji;
- [ ] dodać cache, rollback i zachowanie offline;
- [ ] przetestować uszkodzone, niepodpisane oraz niekompletne paczki.

## Definition of Done

Biblioteka jest gotowa do publicznych contribution, gdy użytkownik może
utworzyć animację w Meshy2Aurora, wyeksportować paczkę PR, przejść wszystkie
bramki CI, a następne wydanie aplikacji pokaże ją jako przeszukiwalny preset
`Community`. Użycie presetu na zgodnym rigu musi dawać deterministycznie ten sam
motion hash i binary MDL readback, bez modyfikacji źródłowego GLB.
