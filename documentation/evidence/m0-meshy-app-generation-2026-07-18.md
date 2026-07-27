# M0: wygenerowanie źródłowego GLB w Meshy Lab (2026-07-18)

Status: `SOURCE READY / NATIVE PACKAGE LANE IMPLEMENTED / LIVE AURORA+NWN PROOF PENDING`.

## Cel i zakres

Ten packet dokumentuje utworzenie nowego, niezależnego wejścia Meshy dla
`M0_MESHY_RIGID_CONTROL`, zgodnie z Fazą 0 planu runtimeowego. Nie jest to
jeszcze proof binarnego MDL, Aurora Toolsetu ani klienta NWN:EE.

## Fakty z przebiegu aplikacji

- Profil w Meshy Lab: `S1-static-prop/v1` (`Static Prop`, `GLB`,
  `PREVIEW -> REFINE`).
- Prompt: `A small low-poly painted stone golem statue, upright and centered,
  one solid opaque material, no weapon, no moving parts, isolated game asset`.
- Przed uruchomieniem aplikacja pokazała limit `max 30 credits`; uruchomienie
  nastąpiło dopiero po potwierdzeniu właściciela.
- Run osiągnął `READY / 100%`. Lokalna aplikacja pobrała artefakt, sprawdziła
  nagłówek GLB, rozmiar i SHA-256 względem provenance, a następnie zaimportowała
  go do kroku `Source`.
- Zaimportowany plik ma nazwę `meshy-s1-static-prop.glb`, rozmiar `8.6 MB` i
  widoczny w UI prefiks SHA-256 `aac32ee61974...`.
- Provenance zostało pobrane lokalnie. Identyfikator zadania pozostaje wyłącznie
  w tym artefakcie i nie jest kopiowany do dokumentacji ani używany jako sekret.

## Nagranie przebiegu

Kolejne zrzuty aplikacji są przechowywane lokalnie, poza Gitem, w:

`C:\Projects\meshy2aurora\.codex-tmp\meshy-m0-app-recording-2026-07-18`

Pakiet obejmuje: wejście do Studio, połączenie z Meshy Lab, wybór profilu,
konfigurację promptu, review kosztu, stan bezpośrednio przed potwierdzeniem,
start, etap refine, stan ready oraz import zweryfikowanego GLB do Source
(`01`--`10` PNG).

## Zebrane dowody i ograniczenia

Fakt z aplikacji: import zakończył się w kroku `Source`; model jest gotowym,
zweryfikowanym wejściem GLB do własnego ingestu.

Fakt z kodu: obecna ścieżka builda Studio emituje artefakty o resrefach
`m2a_m6p01.mdl`, `m2a_codex_aproof.hak` i `m2a_codex_aproof.mod`
(`apps/studio-web/src/worker/m2a.worker.ts`). `m2a_m6p01` jest resrefem H1,
więc natychmiastowy build tego wejścia nadpisałby rozróżnienie H1/M0 zamiast
utworzyć osobny kandydat M0.

Wniosek implementacyjny: przed buildem M0 trzeba dodać jawny, testowany
kontrakt osobnego resrefu M0 oraz kompozycję pojedynczego proof HAK/MOD z
dwoma wierszami `appearance.2da`: dodatnią kontrolą `c_tortoise` i nowym M0.
Nie wolno interpretować samego sukcesu importu GLB jako proofu runtimeowego.

## Zaakceptowany kontrakt paczki M0

Implementacja test-first wprowadza oddzielny runtime lane `M0_STATIC_RIGID`:

- wygenerowany model i tekstura: `m2a_m0p01.mdl` / `m2a_m0t01.tga`;
- własny HAK i MOD: `m2a_m0_proof.hak` / `m2a_m0_proof.mod`;
- obszar proofu: `m2a_m0proof_area`;
- dwa dopisane wiersze `appearance.2da`: `M2A_CTRL_TORTOISE` wskazujący
  `c_tortoise0` oraz `M2A_M0_MESHY_RIGID` wskazujący `m2a_m0p01`;
- dwa UTC/GIT w tym samym obszarze: tortoise przy `(12, 10)` i M0 przy
  `(16, 10)`;
- `Mod_HakList` w kolejności: zewnętrzny `znd_tortoise`, następnie własny
  `m2a_m0_proof`. Własny HAK dostarcza końcowe `appearance.2da`, ale nie
  kopiuje modelu ani tekstur tortoise'a.

M0 odrzuca wejście ze skinem lub animacjami źródłowymi i materializuje jeden
RIGID segment. Siedem wymaganych klipów lifecycle jest minimalnymi,
bezruchomymi klipami utworzonymi przez Meshy2Aurora; nie są importowane z
Meshy ani z zasobu referencyjnego.

Test `static_meshy_m0_materializes_in_a_separate_two_fixture_runtime_package`
potwierdza deterministyczny MDL/TGA/2DA/HAK/MOD, własny binary readback,
osobne resrefy i obecność obu wierszy appearance.

## Następny krok

Odzyskać ten sam zweryfikowany GLB M0 do Studio, zbudować świeży packet i
wykonać obowiązkową bramkę viewportu Toolsetu. Tylko po jej przejściu można
uruchomić standardowy proof NWN na drugim ekranie i najpierw ocenić tortoise,
a następnie M0.
