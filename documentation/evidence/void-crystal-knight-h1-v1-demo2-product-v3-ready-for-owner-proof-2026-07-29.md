# vckdemo2.mod — Void Crystal Knight stabilized demo handoff

**Exact test module:** `vckdemo2.mod`
**Toolset module name:** `Meshy2Aurora procedural humanoid proof`
**Exact Area name:** `Meshy2Aurora M0 binary vertical-slice area`

Data: 2026-07-29
Status: `owner_visual_fix_confirmed`

## Wynik

Właściciel jawnie dopuścił nową iterację po potwierdzonym wyniku
`modelVisibility=visible`, `proofCompleteness=failed` starego `vckdemo1.mod`.
Przyczyną były odłączone kryształy rozciągane przez mieszane wagi ramion i
tułowia.

Nowy candidate został zbudowany wyłącznie przez aktywną ścieżkę:

```text
canonical source.glb
  -> Procedural Creature Product V3
  -> SkinAccessoryStabilizationV1 / AUTO
  -> Product MDL + TGA + appearance.2da + HAK
  -> Demo V2 MOD/UTC
  -> Product Demo Packet V2
```

Nie uruchomiono ani nie adoptowano Aurora Toolset/NWN. Końcowy test wizualny
należy do właściciela.

## Exact identity

```yaml
candidate_id: void-crystal-knight-h1-v1-demo2-product-v3
packet: proof-output/void-crystal-knight-h1-v1-demo2-product-v3
module_file: vckdemo2.mod
module_resref: vckdemo2
module_name: Meshy2Aurora procedural humanoid proof
area_resref: vckarea2
area_name: Meshy2Aurora M0 binary vertical-slice area
creature_resref: vckutc2
hak_file: vckhak2.hak
hak_resref: vckhak2
model_resref: vcknight_m2
texture_resref: vcknight_t2
appearance_row: 15100
placement: { x: 10.0, y: 14.5, z: 0.0 }
orientation: { x: 0.0, y: -1.0 }
```

## Immutable artifacts

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `generated/vckdemo2.mod` | 15,142 | `cb96c0a9caba04463c4b53303f43aca09ff0216efd3a3b1211af7669b3a08797` |
| `generated/vckhak2.hak` | 22,040,645 | `4b2181b922fa6fce85a312596754bda0acc5aee831265c745e0e7bc182a1f7e2` |
| `generated/vcknight_m2.mdl` | 2,556,084 | `219a51ee1c9112500332d02adfa25b093748030cc69c8ec55bbf66a25df8963e` |
| `generated/vcknight_t2.tga` | 12,582,956 | `5101483b93b6eb4e0aa39cc705dcbed9de1a70c1661f85eacffcb58793025e8a` |
| `generated/appearance.2da` | 6,901,349 | `3dc8505bb5848044659cf44f9fe60e3f7c401db772496c4571ec0fff62a761d6` |
| `generated/source.glb` | 9,951,624 | `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7` |
| `reports/materialization-manifest.json` | 9,901 | `1ae830be617e04c2df983a19ca297b656bff305598ddbdec20b57fb64493c6ee` |
| `reports/materialization-report.json` | 1,080,957 | `89ed01c239546c7fca59870f7051a09e0c576fa601a52c85edd52aa195f3f8d8` |

Packet manifest zadeklarował dziesięć plików wejściowych/wyjściowych. Każdy
został ponownie odczytany z dysku i przeszedł kontrolę długości oraz SHA-256.

## Geometria i stabilizacja

```yaml
source_triangles: 19704
geometry_triangles: 19704
written_triangles: 19704
skin_accessory_stabilization:
  mode: AUTO
  component_count: 5
  detached_component_count: 4
  risky_component_count: 4
  stabilized_component_count: 4
  changed_vertex_count: 341
  warnings: []
  lower_pair: Spine02
  upper_pair: Spine
```

Zachowane są wszystkie trójkąty, pozycje, indeksy, UV, normalne, tangenty,
materiały i klipy animacji. Zmienione zostały wyłącznie wagi 341 wierzchołków
czterech zatwierdzonych odłączonych komponentów.

Profil animacji pozostaje kompletny:

```yaml
profile: FULL_NATIVE42_PROCEDURAL_HUMANOID_V1
required_clips: 42
preserved_source_clips: [cpause1, cwalk, crun]
preserved_source_count: 3
procedural_count: 39
gameplay_event_pairs: 23
behavior_candidate_eligible: true
skin_animation_conformance_complete: true
```

## Odrzucony lane

Pierwsza próba użyła historycznego legacy bundle i zapisała 19,678 trójkątów.
Została odrzucona przed instalacją. Jej packet pozostaje wyłącznie dowodem lane
failure:

`proof-output/void-crystal-knight-h1-v1-demo2`

Nie jest kandydatem do testu. Naprawiono systemowo przykład materializatora:
domyślna ścieżka używa teraz Product V3 + Demo V2 + Product Demo Packet V2.

## Native installation

Dokładne artefakty zostały skopiowane wyłącznie do wcześniej nieobecnych celów:

```text
C:\Users\enonw\Documents\Neverwinter Nights\modules\vckdemo2.mod
C:\Users\enonw\Documents\Neverwinter Nights\hak\vckhak2.hak
```

Po instalacji:

```yaml
module_hash_verified: true
hak_hash_verified: true
```

Hashe plików natywnych są identyczne z kanonicznymi źródłami packetu.

## Owner proof

1. Otwórz dokładnie `vckdemo2.mod`.
2. Potwierdź nazwę modułu `Meshy2Aurora procedural humanoid proof`.
3. Otwórz Area `Meshy2Aurora M0 binary vertical-slice area`.
4. Sprawdź model `vcknight_m2`/creature `vckutc2` w idle, walk i run.
5. Zweryfikuj, że cztery kryształy pozostają sztywne przy tułowiu i nie tworzą
   czarnych skrzydeł ani wydłużonych odłamków.
6. Przetestuj ten sam exact MOD/HAK w NWN.

Aktualne osie przed wynikiem właściciela:

| Lane | modelVisibility | proofCompleteness |
|---|---|---|
| Aurora Toolset | `not_tested` | `missing` |
| NWN runtime | `not_tested` | `missing` |

## Owner result — 2026-07-30

Właściciel potwierdził, że model jest „ładnie poprawiony”, i zlecił publikację
brancha do merge. Wynik zamyka wadę `detached_skin_accessory_deformation` dla
exact kandydata `void-crystal-knight-h1-v1-demo2-product-v3`.

Wypowiedź nie wskazała osobno Toolsetu ani NWN, więc historyczna tabela per-lane
powyżej pozostaje niezmieniona i nie jest sztucznie uzupełniana.
