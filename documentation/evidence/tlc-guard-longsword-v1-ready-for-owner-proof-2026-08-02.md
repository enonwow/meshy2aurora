# TLC Guard Longsword v1 — handoff do proofu właściciela

Test module file: `m2atgls1.mod`

Toolset module name: `Meshy2Aurora TLC Guard Longsword v1`

Area name: `TLC Guard Longsword Assembly Proof`

Status: `ready_for_owner_proof`

## Dokładna tożsamość

| Element | Resref / plik | SHA-256 |
|---|---|---|
| MOD | `m2atgls1.mod` | `08bbebc18905d4b1ed6e6d9f3b9bb50f02afa08e12e86709551274013ae53dc0` |
| HAK | `m2atglh1.hak` | `9dba2282a123177df29acabc6c543ececf135d63aed38871e01d8e7dab9fae22` |
| UTI | `m2atglu1.uti` | `b018aca195a5bf7cb4f8d1993bf1957f089ae590c332b3d9aae112971c4d5087` |
| fit report | `fit-report.json` | `e4447473220704f0854decd637a5bad9a3baa0529e480d38eb6f792631e75bdd` |
| Area | `m2atgla1` | zasób w dokładnym MOD |

Kanoniczny packet:

`C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-v1-20260802`

Manifest kandydata:

`C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-v1-20260802\ready-for-owner-proof.json`

Ten asset jest odrębny od technicznego kontrolnego `item-composed-r01-20260730`.
Nie nadpisano ani nie iterowano starego kandydata.

## Instalacja do testu

Oba cele były nieobecne przed kopiowaniem i zostały utworzone jako
`created_new`. Hash źródła i natywnej kopii jest identyczny:

| Typ | Natywny cel |
|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2atgls1.mod` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2atglh1.hak` |

Kolejność HAK: tylko `m2atglh1`.

## Kontrakt przedmiotu

- BaseItem: `1` (`longsword`), istniejący w retail `baseitems.2da`;
- ModelType: `2`, czyli trzy niezależne party Aurora;
- UTI: `m2atglu1`, nazwa `Last City Guard Longsword`;
- `Identified=1`, potwierdzone własnym readbackiem GFF;
- brak nowego wiersza 2DA: dla istniejącego BaseItem nazwy modeli wynikają z
  `ItemClass=WSwLs` i wartości `ModelPart1/2/3`;
- `16 356 / 300 000` trójkątów, `0` usuniętych;
- każdy part ma osobny MDL, teksturę TGA i warstwę ikony TGA;
- nie istnieje jeden scalony MDL.

| Pole UTI | Rola | Wariant | MDL | SHA-256 MDL |
|---|---|---:|---|---|
| `ModelPart1` | Bottom | `252` | `wswls_b_252` | `bcc154133167aa64f620a90b25b658c02cdb77f02995587adfc1fc6c74ed3c0f` |
| `ModelPart2` | Middle | `253` | `wswls_m_253` | `c2329ad0590ee51fb2491acf83adca44d3bc8a95cb7c5f5d1855c30421560b4d` |
| `ModelPart3` | Top | `254` | `wswls_t_254` | `2981ae9209dc64ecd248fb4bcee0521b1d14cbbf9a9564587fb567768c66eee1` |

Auto-fit:

- algorytm: `ORDERED_SLOT_AXIAL_ENVELOPE_SEAM_GATE_V1`;
- tolerance: `0.01`;
- solution SHA-256:
  `e88556ddcd24d665f3f9d36ae1d6e75a0bcc973d9d7d7fe9e494a43c08f49bde`;
- Bottom/Middle oraz Middle/Top: `TOUCHING`;
- Bottom/Top: bez overlap.

## Oczekiwane miejsce

- wejście gracza: `[10.0, 10.0, 0.0]`;
- kierunek wejścia: `[0.0, 1.0]`;
- jedyny umieszczony UTI: `[10.0, 14.5, 0.0]`;
- przedmiot leży bezpośrednio przed kierunkiem wejścia.

## Granica werdyktu

Agent nie uruchamiał Aurora Toolset ani NWN:

- `agentStartedToolset=false`;
- `agentStartedNwn=false`;
- `modelVisibility=not_tested`;
- `proofCompleteness=missing`.

Właściciel otwiera dokładnie `m2atgls1.mod`, sprawdza nazwę modułu i Area podane
na początku dokumentu, a następnie ocenia złożenie miecza najpierw w Toolset,
potem w NWN dla tej samej linii MOD/HAK/UTI. Dopiero świeży werdykt właściciela
może zamknąć proof albo dopuścić następną iterację modelu.
