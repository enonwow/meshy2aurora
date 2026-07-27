# P8 — statyczna kolizja placeable przez PWK

Data: 2026-07-25

Status: `V1_AND_V2_OWNER_RUNTIME_FAILED / ROOT_CAUSE_CONFIRMED /
BINARY_PWK_ROUTE_REJECTED / V3_ASCII_PWK_READY_FOR_OWNER_PROOF`

Właściciel sprawdził dokładne V1 i V2 i zgłosił brak blokowania w obu
kandydatach. V2 dowiódł, że wcześniejsza diagnoza adjacency była
niewystarczająca. Aktualny wynik i audyt dekompilacji są zapisane w:

- `p8-s1-collision-v2-owner-runtime-result-2026-07-25.json`;
- `p8-s1-collision-v2-aurora-decomp-audit-2026-07-25.md`.

Po przejściu testów offline utworzono dokładnie jeden V3. Jego osobny handoff:
`p8-s1-collision-v3-ascii-pwk-ready-for-owner-proof-2026-07-25.md`.

> **Korekta końcowa audytu:** exact `nwmain.exe` i `nwserver.exe`
> `89.8193.37-17` pokazują, że `CNWPlaceableSurfaceMesh::LoadWalkMesh` parsuje
> PWK jako tekstowe linie, a nie binary MDL. V1/V2 są binary MDL pod Resource
> Type `2053`, więc runtime nie odczytuje z nich żadnych `verts` ani `faces`.
> Hierarchia binary modelu, adjacency i pole `0x268` nie są przyczyną tego
> failure. Szczegóły zawiera audyt wskazany wyżej.

## Historyczny exact handoff V1

1. Testowy MOD: `m2a_s1_col_mod.mod`
2. Nazwa modułu w Toolset: `Meshy2Aurora S1 Collision Proof`
3. Dokładna Area: `Meshy2Aurora M0 binary vertical-slice area`

Obiekt: `m2a_s1_collision`, appearance row `16500`, placement
`(10.0, 14.5, 0.0)`, bearing `0.0`.

## Objaw i zakres

Wcześniejszy exact placeable P1 był widoczny w Aurora Toolset i NWN, ale jego
HAK nie zawierał zasobu PWK, więc nie zapewniał kolizji. Właściciel zgłosił
widoczny brak kolizji. P8 jest oddzielnym lane'em funkcjonalnym, nie ponowną
iteracją naprawiającą widoczność modelu. Poprzedni wynik
`visible + verified` pozostaje zachowany.

Zakres V1: statyczny, sztywny placeable z `Useable = 0`; kolizja blokująca
oparta na konserwatywnym prostokącie XY. Poza zakresem: use points, animowany
placeable, dokładny obrys wklęsły, tile WOK/AABB.

## Fakty Aurora First i retail

### Fakt z dekompilacji

W `C:\Projects\New Folder\export\decompiled_all.c` wspólna fabryka node'ów
modelu rozdziela rodziny mesh/AABB/skin po rozpoznaniu typu. Nie ma osobnego
parsera geometrii placeable. To utrzymuje decyzję o jednym readerze, IR i
writerze MDL.

### Fakt z retail

Źródła odczytano in-place; żadnego payloadu retail nie skopiowano do repo.

- `nwn_base.key`: SHA-256
  `09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935`;
- `os_models.bif`: SHA-256
  `f3c8d051cb4c40264b8c9743a097b0fb151f6d963878f3c66ec08b68df35c89a`;
- typ zasobu PWK: `2053`;
- `dag_applescene`: 4 vertices, 2 faces, 2 use dummy, surface `7`;
- `dag_batstatue`: 4 vertices, 2 faces, 2 use dummy, surface `7`;
- `dag_blk`: 4 vertices, 2 faces, 1 use dummy, surface `7`;
- `dag_brokenbrl`: 4 vertices, 2 faces, 2 use dummy, surface `7`.

Dodatkowy env-gated test czyta zewnętrzne HAK-i in-place produkcyjnym parserem
ASCII PWK; binary PWK jest jawnie odrzucany przez runtime-ready gate. Końcowy
audyt przeskanował cały dostępny
lokalny corpus: `1 090` retail PWK i `8 662` PWK z zewnętrznych HAK-ów.
Wszystkie `9 752` są tekstowe; zero jest binary. Jedyne dwa binary PWK w
środowisku pochodzą z niedziałających V1/V2 Meshy2Aurora.

## Wniosek implementacyjny po audycie runtime

Poprawne i przeznaczone do ponownego użycia są:

- wspólny `AuroraModelIrV1`;
- world-space XY bounds wszystkich rigid segmentów;
- walidacja hierarchii, transformacji i zdegenerowanego footprintu;
- surface `7` (`Nonwalk`) i osiem pól liczbowych face zgodnych z parserem;
- ten sam bazowy resref MDL/PWK oraz Resource Type `2053`;
- brak use dummy dla statycznego `Useable = 0`.

Błędna jest wyłącznie serializacja PWK przez `write_binary_mdl`. Render MDL
pozostaje binarny, ale PWK musi otrzymać osobny ASCII writer/readback zgodny z
`CNWPlaceableSurfaceMesh`: `node`, `parent`, `position`, `orientation`,
`verts`, `faces`, `endnode`. To zachowuje wspólny pipeline geometrii i
rozdziela jedynie engine-facing serializer wymagany przez inny parser.

## Testy

| Gate | Wynik |
|---|---|
| `cargo test -p m2a-core --test placeable_collision --test placeable_pipeline` | PASS: 11 testów zaliczonych, 1 env-gated ignored |
| produkcyjny parser przez `pwk_reference_corpus` z `cep3_core0.hak` i `*` | PASS: 8 ASCII PWK odczytanych in-place |
| `cargo fmt --all --check` | PASS |
| `cargo check -p m2a-core --example materialize_s1_collision_placeable` | PASS |
| `cargo test -p m2a-core --quiet` | PASS: pełny zestaw testów crate'a core, bez awarii |
| własny runtime-aligned readback exact PWK | PASS: ASCII, 4 vertices, 2 faces, powierzchnie `[7, 7]`, compatibility fields `0 0 0` |
| pełny `cargo test --workspace --quiet` | PARTIAL: cały `m2a-core` przeszedł; 4 niezwiązane z PWK byte-exact regresje `m2a-wasm --lib` pozostają poza lane'em P8 |

## Exact frozen artifacts

Katalog:
`C:\Projects\meshy2aurora\proof-output\s1-placeable-collision-v1-20260725`

| Artefakt | SHA-256 |
|---|---|
| MOD `m2a_s1_col_mod.mod` | `3da10b51f3f20b71cc96d35d0b6419ed458a07fd3b7755e41cd7fb8aa596afce` |
| HAK `m2a_s1_col_hak.hak` | `06b5142a136a8c021766cc5d2b8344941571bbafcefc4ccc59dfb7c70c292d4e` |
| render MDL `m2a_s1_col_ped` | `c6d39152b655fa2ded09f101b0519ed510f202946c69cf6ceaf6fd8566b790f9` |
| PWK `m2a_s1_col_ped`, type `2053` | `1ed56a207c58ad939ee34c03d121e9ecf41b5aaaa51dbc88a210edf42c38b7d1` |
| TGA `m2a_s1_col_tex` | `96a45ce0ac3b3eba8e54a56af1aeec425e2c63c441b8a87498f045bb365239dc` |
| wygenerowane `placeables.2da` | `3eb0f995593e4ef1c4aa4fe3a34e991e0e8fc98421a0bb76c902c89f29ed05ac` |

Exact PWK ma 1272 bajty, bounds
`[-0.32745302, -0.3240835, 0.0] .. [0.32745302, 0.3240835, 0.0]`,
4 wierzchołki, 2 twarze i `surfaceId = 7` na obu twarzach.

## Instalacja

Zgodnie z `AGENTS.md` exact pliki z kanonicznego outputu zostały natychmiast
zainstalowane bez overwrite i zweryfikowane byte-for-byte:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_s1_col_mod.mod`
  — SHA-256 MOD powyżej;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_s1_col_hak.hak`
  — SHA-256 HAK powyżej.

Oba cele nie istniały przed instalacją. Hash źródła i celu jest identyczny.
Aurora Toolset ani NWN nie zostały uruchomione przez agenta.

## Zamknięcie V1 przez właściciela

- [x] PWK jest w tym samym HAK-u i ma ten sam resref co MDL.
- [x] Binary readback geometrii, windingu, bounds i surface przeszedł.
- [x] Exact MOD/HAK zainstalowano i zweryfikowano byte-for-byte.
- [x] Właściciel sprawdził exact V1 w runtime.
- [x] Właściciel zgłosił, że placeable nie blokuje ruchu.

- `collisionCompleteness = pwk_emitted_readback_passed`;
- `collisionRuntimeVerdict = not_blocking`;
- `collisionProofCompleteness = verified_by_owner_report`;
- status = `failed_collision_runtime_gate`.

Machine-readable wynik:
`documentation/evidence/p8-s1-collision-v1-owner-runtime-result-2026-07-25.json`.

## Implementacja ASCII PWK i exact V3

- [x] `PlaceableWalkmeshIrV1` jest wyprowadzany ze wspólnego
  `AuroraModelIrV1`.
- [x] Render MDL nadal korzysta ze wspólnego binary MDL writera.
- [x] PWK typu `2053` korzysta z deterministycznego serializera ASCII.
- [x] Reader obsługuje `node`, `trimesh`, `dummy`, `parent`, `position`,
  `orientation`, `verts`/`vertices`, `faces` i `endnode`.
- [x] Reader ignoruje niegeometryczne metadane najwyższego poziomu tak jak
  runtime, m.in. retailowe `filedependancy`.
- [x] Writer i reader pilnują limitu linii wynikającego z 256-bajtowego bufora
  runtime.
- [x] Runtime-ready package odrzuca binary PWK, NUL, non-ASCII, błędne count,
  skrócone dane i indeksy faces poza zakresem.
- [x] HAK readback parsuje gotowy zasób `2053`, nie tylko porównuje bajty.
- [x] Materializator domyślnie tworzy V3 tylko w kanonicznym `proof-output`.
- [x] Materializator nie instaluje natywnych plików bez osobnej decyzji
  właściciela.
- [x] Dokładnie jeden V3 utworzono po zielonym pełnym `m2a-core`.
- [ ] Właściciel potwierdza kolizję w exact V3.

Exact V3:

1. MOD: `m2a_s1_c3_mod.mod`
2. Toolset module name: `Meshy2Aurora S1 Collision V3 ASCII PWK`
3. Area: `Meshy2Aurora M0 binary vertical-slice area`

Katalog:

`C:\Projects\meshy2aurora\proof-output\s1-placeable-collision-v3-ascii-pwk-20260725`

| Artefakt | SHA-256 |
|---|---|
| MOD | `3bc640836f8a5eeb0f0f383e66af4980a8c408e61e3de05a474c26e6aaf28bc4` |
| HAK | `e214d33bc7910ca8ab7626238cc8f5bd17270828146b3b88b0e527fc0332ba24` |
| render MDL | `499d7d4a80ab66a8c4bc7a3332bb195497f8f8078eeb74edc224aca17d0e4342` |
| ASCII PWK | `6299bf9fc2c60e0b8e8bceb57f170e8eff55f4c049d55031ccf6bf1f35726cbd` |

Stan handoffu:

```text
status = ready_for_owner_proof
collisionCompleteness = ascii_pwk_emitted_runtime_readback_passed
collisionRuntimeVerdict = not_tested
modelVisibility = not_tested
proofCompleteness = missing
nativeInstallation = installed_after_explicit_owner_instruction
nativeInstallationVerifiedByteIdentical = true
```

Pełne hashe i checklista właściciela:
`documentation/evidence/p8-s1-collision-v3-ascii-pwk-ready-for-owner-proof-2026-07-25.md`.
