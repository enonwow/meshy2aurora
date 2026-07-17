# Jeden kanoniczny modul proof animacji Codexa

Status: `IMPLEMENTED_STRUCTURAL / NATIVE_RUNTIME_OPEN`
Data: 2026-07-17
Wlasciciel decyzji: owner projektu
Wykonawca generatora i modułu: Codex

## Decyzja wiążąca

Projekt ma **dokładnie jeden** kanoniczny modul do testów runtime Aurora/NWN.
Nie tworzymy osobnych `.mod` na model, klip, błąd, screen ani kolejny proof.
Każdy wątek implementacyjny, Studio lub runtime-worker używa tego samego
modułu; zmienia się wyłącznie zawartość jego towarzyszącego HAK i udokumentowany
plan capture.

| Rola | Jedyna dozwolona nazwa | Znaczenie |
|---|---|---|
| MOD | `m2a_codex_aproof.mod` | `Meshy2Aurora Codex Animation Proof`; jeden moduł testowy |
| resref/tag MOD | `m2a_codex_aproof` | 16-bajtowy resref określający Cel: Codex animation proof |
| HAK | `m2a_codex_aproof.hak` | jedyny HAK podpinany przez ten MOD |
| obszar | `m2a_caproof_area` | obszar proof modułu |
| UTC H1 | `m2a_caproof_h1` | instancja stworzenia dla aktualnego proofu H1 |

`module.ifo` ma jedną pozycję `Mod_HakList`, wskazującą dokładnie
`m2a_codex_aproof`. Widoczne nazwy modułu, obszaru, stworzenia i komentarz GIT
oznaczają go jako wygenerowany przez Codex do proofu animacji. To jest
świadoma identyfikacja pochodzenia, nie deklaracja zgodności runtime.

## Zawartość aktualnego HAK

HAK jest jedynym pakietem assetów przypiętym do modułu. Aktualny H1 packet ma:

| Resource | NWN type | Cel |
|---|---:|---|
| `appearance` | 2017 | wejściowe `appearance.2da` zachowane i rozszerzone o rekord proof |
| `m2a_m6p01` | 2002 | binary MDL realnego modelu Meshy H1 |
| `m2a_m6t01` | 3 | TGA użyta przez ten model |

`appearance.2da` wskazuje physical row `15100` na `m2a_m6p01`; UTC modułu ma
`Appearance_Type=15100`. Przy kolejnym modelu można rozszerzyć **ten sam** HAK
o kolejne model/texture resources i rekordy `appearance.2da`, ale nie wolno
tworzyć drugiego modułu ani podłączać dodatkowego HAK bez osobnej decyzji
udokumentowanej w tym pliku.

## Generowanie od zera i aktualny packet

Jedyną trasą generacji jest własny pipeline `m2a-core`; MOD powstaje od zera z
`module.ifo`, `ARE/GIC/GIT`, UTC i własnego readbacku. Nie jest kopią modułu
Aurory ani retail payloadu.

Aktualny packet lokalny (ignorowany przez Git):

`C:\Projects\meshy2aurora\proof-output\meshy-h1-codex-animation-proof-v1`

| Artefakt | SHA-256 | Stan |
|---|---|---|
| `generated/m2a_codex_aproof.hak` | `da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756` | own HAK readback PASS |
| `generated/m2a_codex_aproof.mod` | `dee6585745a6a57ce518af1443d9845333d1c7f1225b438ad9295e44b5a29d2d` | own MOD/GFF readback PASS |

Wejście H1 ma SHA-256
`3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`.
Jego jedyny obecny clip `Armature|Idle|baselayer` jest mapowany do `cpause1`.

## Handoff do runtime worker

1. Worker sprawdza hashe aktualnego packetu przez
   `tools/install-nwn-h1-runtime-proof.ps1 -Mode Plan`.
2. Dopiero po jawnej zgodzie ownera `-Mode Install` kopiuje dokładnie ten MOD
   i HAK, bez nadpisywania istniejących plików.
3. W Toolsecie otwiera wyłącznie `m2a_codex_aproof.mod`; nie tworzy pomocniczego
   modułu. Najpierw read-only potwierdza, czy lokalny Toolset wymaga zapisu lub
   builda projektu przed uruchomieniem.
4. Nie zakłada, że Toolset przebudowuje HAK. HAK pozostaje naszym wygenerowanym
   archiwum i jest rozwiązywany przez `Mod_HakList`; ewentualne wymaganie
   Toolsetu musi zostać udowodnione lokalnym przebiegiem.
5. Uruchamia moduł w **NWN**, a nie w Toolsecie, i dopiero tam zapisuje MP4 oraz
   PNG pokazujące zmianę animacji. Toolset screenshot jest tylko pomocniczy.
6. Zapisuje werdykt w `documentation/evidence`; raw artefakty mogą pozostać w
   ignorowanym `proof-output`.

Historyczne lokalne pakiety o nazwach `m2a_h1proof.mod` lub
`m2a_m6p01.hak` są dowodem wcześniejszej pracy, ale **nie są** aktywnym modułem
testowym i nie wolno ich instalować ani używać dla kolejnych proofów.

## Fail-closed native proof packet

Proces dowodowy korzysta z przeanalizowanego standardu live proof projektu
`aurora-web`, lecz nie używa jego kodu, zależności ani artefaktów. Dla tego
projektu obowiązują następujące odpowiedniki:

- status końcowy może być wyłącznie `VERIFIED`, `FAILED` lub `MISSING`;
  brak artefaktu z NWN oznacza `MISSING`, nigdy sukces;
- preflight zapisuje hashe HAK i MOD, ścieżki instalacji, resref modułu/area,
  `appearance` row i stan docelowych plików przed instalacją;
- Toolset działa w jednej żywej sesji. Po błędzie worker najpierw sprawdza PID
  i aktualne okno; nie zabija ani nie zamyka procesu automatycznie;
- proof packet zawiera `before`/`after` PNG z **NWN**, MP4 widocznej zmiany
  `cpause1`, `summary.json` z kontekstem runtime oraz analizę różnicy obrazu;
- screenshot bez widocznej zmiany, MP4 bez właściwego modelu albo poprawny
  packet bez capture z NWN daje `FAILED` lub `MISSING`;
- nie wolno modyfikować `nwtoolset.ini`, MRU ani traktować screenshotu Toolsetu
  jako runtime acceptance;
- jeżeli Toolset przepisze kontener MOD, raport opisuje dokładnie sprawdzoną
  semantykę; nie deklaruje byte-identical rollbacku bez osobnego dowodu.

## Co jeszcze nie jest dowiedzione

Pakiet jest poprawny strukturalnie, ale nie był jeszcze odtworzony w realnym
NWN. Natywny proof ma osobno wykazać widoczne `cpause1`, w tym
zachowanie tracku `SCALE` na `Hips`. H1 ma jedynie Idle; brak `cwalk`, `crun`,
ataku, obrażeń i śmierci pozostaje osobnym zakresem, nie powodem do utworzenia
drugiego MOD.
