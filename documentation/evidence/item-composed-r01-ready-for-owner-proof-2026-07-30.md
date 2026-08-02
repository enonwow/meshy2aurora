# Item composed r01 — handoff do proofu właściciela

Test module file: `m2ait_r01.mod`

Toolset module name: `Meshy2Aurora Item composed r01`

Area name: `Meshy2Aurora Item Assembly Proof`

Status: `ready_for_owner_proof`

## Dokładna tożsamość

| Element | Resref / plik | SHA-256 |
|---|---|---|
| MOD | `m2ait_r01.mod` | `3af24e7bf57009c1244a3477d02080ecbaf28c7ddd71f21935ef729249d6b644` |
| HAK | `m2ait_h01.hak` | `feae602c30d7bb266cbf53c8ebbc6ff610c84361d0977814abb041b720181cf1` |
| UTI | `m2ait_u01.uti` | `860465dbf271b942a2bf7857a62fe8fb5ea824d768bf65e9e4d67d0813bfd356` |
| Area | `m2ait_a01` | zasób w dokładnym MOD |

Kanoniczny packet:

`C:\Projects\meshy2aurora\proof-output\item-composed-r01-20260730`

Manifest:

`C:\Projects\meshy2aurora\proof-output\item-composed-r01-20260730\ready-for-owner-proof.json`

## Instalacja do testu

Oba cele były nieobecne przed kopią, zostały utworzone jako `created_new`,
a hash źródła i celu jest identyczny:

| Typ | Natywny cel |
|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2ait_r01.mod` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2ait_h01.hak` |

Kolejność HAK: tylko `m2ait_h01`.

## Kontrakt przedmiotu

- BaseItem: `1` (`longsword`);
- ModelType: `2`;
- UTI: `m2ait_u01`;
- lokalna nazwa UTI: `Meshy composed three-part item r01`;
- suma geometrii: `22 955 / 300 000` trójkątów;
- każdy part ma własny MDL, teksturę i warstwę ikony;
- nie istnieje jeden scalony MDL.

| Pole UTI | Wariant | MDL | Translacja Aurora |
|---|---:|---|---|
| `ModelPart1` / Bottom | `251` | `wswls_b_251` | `[0, 0, 0]` |
| `ModelPart2` / Middle | `251` | `wswls_m_251` | `[0, 0, 2]` |
| `ModelPart3` / Top | `251` | `wswls_t_251` | `[0, 0, 4]` |

Źródła GLB i ich exact SHA-256:

| Pole | Źródło | SHA-256 |
|---|---|---|
| `ModelPart1` | `sample-3d\s1-placeable-ritual-pedestal-1500\source.glb` | `dad22a5c3490242458cb7a81e50c53e265abf75886f57f6bd8a770938c2f7372` |
| `ModelPart2` | `sample-3d\s1-static-prop-1500\source.glb` | `e476bfa426701bb73f75b7555bd6576a7923064ecab0f7e7afae277f373ca191` |
| `ModelPart3` | `sample-3d\tlc-aether-lamp-p20k-v1\final.glb` | `310a6597f92bfcd391f44d293585417fa53abe3f1a576d93a7e0dc2c586b1ac7` |

## Oczekiwane miejsce

- wejście gracza: `[10.0, 10.0, 0.0]`;
- kierunek wejścia: `[0.0, 1.0]`;
- jedyny umieszczony UTI: `[10.0, 14.5, 0.0]`;
- obiekt leży bezpośrednio przed kierunkiem wejścia.

## Granica werdyktu

Agent nie uruchamiał Toolset ani NWN:

- `agentStartedToolset=false`;
- `agentStartedNwn=false`;
- `modelVisibility=not_tested`;
- `proofCompleteness=missing`.

Właściciel otwiera dokładnie `m2ait_r01.mod`, sprawdza nazwę modułu i Area
podane na początku dokumentu, a następnie wydaje świeży werdykt wizualny dla
tej samej linii MOD/HAK/UTI. Dopiero ten werdykt może zamknąć proof albo
dopuścić nową iterację modelu.
