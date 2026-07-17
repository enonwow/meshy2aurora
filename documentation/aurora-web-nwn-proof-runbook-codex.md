# Aurora/NWN native proof runbook (Codex)

## Correction 2026-07-17: Toolset viewport gate

The direct writer may create a module that Toolset parses structurally without
creating a visible, playable scene.  Therefore every native attempt must pass
this gate **before** `Build -> Test Module`:

1. Toolset tree contains the exact proof Area and H1 creature entry.
2. The Area opens in the Toolset viewport without an Error modal or persistent
   Loading Area state.
3. The viewport visibly contains H1; save a viewport PNG and a tree/inspector
   PNG.

Failure at any point is `FAILED`: do not start NWN.  After five minutes without
the next gate, save the artifacts, audit `C:\Projects\aurora-web` and the
local Aurora-first contracts, record the precise delta, close Toolset/NWN, and
restart the next attempt from the preflight.  The current MOD/HAK hashes must
always come from the attempt packet, never from a historical static hash in
this document.

Status: **obowiązkowy** dla każdego native proofu w `meshy2aurora`.

To jest repo-lokalna, skrócona procedura wykonawcza wyprowadzona z
sprawdzonych live standardów w `C:\Projects\aurora-web`. Jest materiałem
procesowym, nie zależnością, kodem runtime ani fixture. Nie wolno zastępować
tej procedury skrótami typu bezpośredni `nwmain +TestNewModule`, zgadywanym
ID menu lub komunikatami wysyłanymi do niewalidowanego UI NWN.

## Źródła i zakres

Źródła read-only:

- `C:\Projects\aurora-web\backend\docs\aurora-reverse\aurora-toolset-operating-standard.md`
- `C:\Projects\aurora-web\backend\docs\aurora-reverse\aurora-toolset-npc-stage3-standard.md`
- `C:\Projects\aurora-web\docs\aurora-toolset-atomic-operations-analysis-2026-06-18.md`

Przeniesione fakty:

- status native proofu jest wyłącznie `VERIFIED`, `FAILED` albo `MISSING`;
- Toolset pracuje w jednej sesji; żywy proces odzyskuje się najpierw przez
  PID/window inspection, bez automatycznego zamykania;
- wejście jest kierowane tylko do aktualnie odczytanych kontrolek; globalny
  kursor, klawiatura i mysz są zabronione;
- każdy używany precedens live trzeba odtworzyć w aktualnej sesji, a nie
  traktować jako licencję na zgadywanie identyfikatorów;
- dowód konkretnej funkcji wymaga realnego runtime, a nie samego Toolsetu.

## Kontrakt H1 (aktualny proof)

Jedynym modułem jest `m2a_codex_aproof.mod`, a jedynym dołączonym HAK jest
`m2a_codex_aproof.hak`. Nie twórz drugiego modułu i nie używaj historycznych
`m2a_h1proof.mod` ani `m2a_m6p01.hak`.

Przed wejściem do Toolsetu zapisuj do packetu:

| Pole | Wartość |
| --- | --- |
| MOD SHA-256 | `dee6585745a6a57ce518af1443d9845333d1c7f1225b438ad9295e44b5a29d2d` |
| HAK SHA-256 | `da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756` |
| `Mod_HakList` | dokładnie `m2a_codex_aproof` |
| Area | `m2a_caproof_area` |
| Creature | `m2a_caproof_h1` |
| `Appearance_Type` | `15100` |
| Modelowa animacja | tylko `cpause1` / Idle, 4.0333333 s |

## Jedyna dozwolona trasa uruchomienia

1. **Preflight bez mutacji.** Sprawdź canonical workspace, MOD/HAK hash,
   docelowe pliki użytkownika, istniejące `nwtoolset`/`nwmain`, modale i log
   silnika. Przeprowadź PID-first recovery, jeżeli proces żyje. Fizyczny ekran
   proofu to `\\.\DISPLAY1`, `primary=false`.
2. **Toolset Open Module.** W jednej sesji Toolsetu odczytaj rzeczywiste menu
   i aktywne okno. Otwórz wyłącznie `m2a_codex_aproof.mod` przez zweryfikowaną
   ścieżkę File/Open. Precedens `aurora-web` potwierdza `WM_COMMAND 44`, ale
   identyfikator można użyć tylko po odczycie go w bieżącej sesji.
3. **Readback modułu.** Zanim nastąpi Test Module, potwierdź w Toolsecie i
   parserem: moduł, pojedynczy HAK, Area, UTC i appearance. Brak któregokolwiek
   pola kończy attempt jako `FAILED` lub `MISSING`; nie wolno przechodzić do
   NWN.
4. **Toolset Test Module.** Odczytaj bieżące menu `Build`, pozycję `Test
   Module`, jej `enabled` oraz command ID. Sprawdzony precedens to pozycja 2,
   `WM_COMMAND 121` / `miTestModule`, ale ID nie jest globalną stałą. Dopiero
   po tej weryfikacji wyślij kierowaną komendę do ramki Toolsetu.
5. **Bramka ładowania NWN.** Przed jakimkolwiek proofem animacji wymagaj
   świeżego wpisu logu wskazującego ładowanie `m2a_codex_aproof` albo konkretnego
   błędu HAK/MOD. Selekcja modułu na ekranie lub nowy PID `nwmain` nie spełniają
   tej bramki.
6. **Proof w NWN.** Dopiero po bramce ładowania wykonaj native capture obiektu
   H1: PNG przed, PNG po, MP4 z widoczną zmianą `cpause1`, analizę różnicy oraz
   niezależny readback. Toolset capture jest wyłącznie pomocniczy.
7. **Packet i wynik.** Zapisz JSON z hashami, resrefami, wersją/PID runtime,
   monitorem, timestampami, cytatem/offsetem logu, nazwami artefaktów i wynikiem
   `VERIFIED`/`FAILED`/`MISSING`. Brak MP4, obu PNG albo potwierdzonego loadu
   nigdy nie może dostać `VERIFIED`.

## Incydent 2026-07-17: ścieżka zakazana

Bezpośrednie uruchomienie `nwmain` z `+TestNewModule` dotarło tylko do
selektora modułów. Log nie potwierdził załadowania `m2a_codex_aproof`; nie
powstała scena, before/after ani MP4. Ten wariant jest `MISSING` i nie może być
ponowiony jako substytut kroku 2–5.

Jeżeli proces zniknie, jest to fakt do odnotowania w packetcie. Przy istniejącej
zgodzie na native proof można utworzyć nową pojedynczą sesję i przejść ten sam
runbook od kroku 1; nie tworzy to nowego modułu ani nowej ścieżki dowodowej.

## Definition of Done

Native proof H1 jest `VERIFIED` tylko, gdy ten sam attempt zawiera:

1. potwierdzony kanoniczny MOD/HAK i ich hash;
2. Toolset readback kontekstu modułu;
3. zweryfikowane aktualne `Build -> Test Module`;
4. log silnika potwierdzający load modułu;
5. native PNG przed/po i MP4 widocznego ruchu w NWN;
6. analizę wizualnej różnicy oraz niezależny readback;
7. fail-closed JSON i audit w `documentation/evidence`.
