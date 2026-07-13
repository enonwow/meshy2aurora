# M6 evidence - generated native model proof

Data: 2026-07-13

```yaml
stage: M6
status: IN_PROGRESS
attempt_id: M6-20260713-01
active_slice: M6B_MODEL_PACKAGE_BRIDGE_DONE_LIVE_PROOF_NEXT
runtime_proof: NOT_RUN
retail_payload_committed: false
```

## 1. Wejscie do etapu

M6 rozpoczal sie dopiero po structural `M5 DONE`. Zamkniete wejscia obejmuja
deterministic TGA, preserve-and-append 2DA, HAK V1.0, PackageManifest oraz
publiczny boundary WASM z own-readback i frozen native/WASM byte proof.

M6 nie traktuje tych bramek jako runtime acceptance. Celem etapu jest wlasny,
wygenerowany pakiet modelu oraz rzeczywisty proof w Aurora Toolset i grze.

## 2. M6A contract-lock

Historyczne, read-only badanie Aurora First obejmowalo:

- wspolny binary layout GFF V3.2;
- minimalne typed UTC/IFO/ARE/GIT wymagane do proofu creature;
- kontener MOD/ERF V1.0 i jego minimalny resource inventory;
- exact labels, field types, struct IDs, list nesting, validation precedence i
  limity potrzebne do TDD;
- rozdzielenie structural own-readback od finalnego live Toolset/game gate.

Primary evidence: lokalna dekompilacja Aurory, oficjalne BioWare PDF-y oraz
read-only lokalne packets. Zewnetrzne implementacje i retail payloady nie sa
fixture source ani kodem do kopiowania.

## 3. Otwarte bramki

```yaml
gff_contract: LOCKED_P1_0_P2_0
typed_utc_ifo_are_git_gic_schema: LOCKED
typed_generated_gameplay_preset: DEFERRED_OUT_OF_ACTIVE_SCOPE
mod_container_contract: LOCKED_STRUCTURAL
own_writer_readback: DONE
embedded_png_jpeg_decode: DONE
model_package_compositor: DONE
generated_file_packet: NOT_STARTED
toolset_acceptance: NOT_RUN
game_acceptance: NOT_RUN
animation_runtime_proof: NOT_RUN
```

## 4. M6A contract-lock evidence

Kanoniczne suplementy:

- `documentation/m6-gff-module-kontrakt-suplement-codex.md`;
- `documentation/m6-typed-resource-manifests-codex.md`;
- skorygowany `documentation/hak-2da-gff-crosswalk-codex.md`.

Sprawdzono wizualnie official BioWare GFF pages 3, 5-14 oraz relewantne strony
Creature, IFO i AreaFile. Primary decomp anchors potwierdzaja exact 56-byte GFF
header, section chain/EOF, typy 0..15, insertion order, `Appearance_Type` WORD,
`Mod_HakList` child ID 8, GIT creature child ID 4 i AreaProperties ID 100.

Contract review znalazl i zamknal: empty CResRef, canonical Struct/Field/Label/
FieldData/FieldIndices/ListIndices ownership, graph cycle/reuse, frozen traversal
phases, LocString schema, precedence, inclusive limit scopes, official PDF vs
Aurora errata oraz provenance typed packets. Final rereview: `P1=0; P2=0`.

Typed ordered schemas UTC67, IFO55, ARE43/tile10, GIT70 i GIC pozostaja future
infrastructure. Po korekcie zakresu nie sa aktywna sciezka modelowego proofu;
Feat/Class/Equip oraz synthetic Tileset/Tile_ID nie sa blockerami produktu.

## 5. M6A GFF core implementation evidence

Zaimplementowano wersjonowany typed tree IR, wszystkie GFF type IDs `0..15`,
deterministyczny writer, public own reader, exact contiguous section layout/EOF,
canonical Struct/Field/Label/FieldData/FieldIndices/ListIndices ownership oraz
semantic readback.

Final review remediations zamknely:

- borrowed two-pass writer preflight: exact counts/layout/maxGff przed input-sized
  materialization;
- exact global field encounter order i swapped FieldIndices rejection;
- stabilne precedence raw bounds -> allocation-free value scan -> canonical
  ownership/coverage -> semantic limits/allocation;
- LocString max limit przed bounded uniqueness allocation;
- compound precedence, zero-materialization i forced-allocation seams;
- deep structured mutation/no-panic matrix.

```yaml
gff_integration: "19/19 PASS"
gff_private_seams: "3/3 PASS"
workspace_tests: "278/278 PASS"
workspace_clippy_all_targets_deny_warnings: PASS
cargo_fmt_check: PASS
git_diff_check: PASS
docker_no_cache:
  tag: "m2a-quality:m6a-final"
  duration_seconds: 165.0
  image_id: "sha256:60ea3d147de1e2f14452fd4459b9f2580978a6be113a84b66f2acdd3fffc0793"
  size_bytes: 1365006101
frozen_one_field_gff:
  byte_length: 96
  sha256: "954af919e592c1abc0a92edef52a0c2855c8940c48199db3c0bd01a62601e5f1"
contract_final_rereview: "P1=0; P2=0"
code_review_initial: "P1=1; P2=3"
code_review_final_rereview: "P1=0; P2=0"
retail_payload_committed: false
runtime_proof: NOT_RUN
```

M6A jest strukturalnie `DONE`. Live Toolset/game acceptance nie zostala
wykonana.

## 6. Korekta zakresu wlasciciela

Wlasciciel doprecyzowal, ze produkt tworzy model, a nie klasy i pozostale dane
gameplay creature. Dlatego `FeatList`, `ClassList`, `Equip_ItemList`, ITP oraz
pelny generated UTC/IFO/ARE/GIT/GIC/MOD nie sa aktywnym deliverable pierwszego
proofu modelu.

GFF core i typed schema research pozostaja poprawnym, zreviewowanym future
infrastructure, ale nie prowadza teraz dalszej implementacji. Aktywna sciezka
M6 to generated binary MDL+MDX, TGA, appended appearance.2da i HAK, sprawdzone
na Toolset-created lub istniejacym known-good test creature/module. Scaffold
nie jest outputem produktu i nie wymaga generowania klas, featow ani ekwipunku.

## 7. M6B model-package bridge

Zamknieto dwa rzeczywiste braki pomiedzy osobnymi writerami a pakietem modelu:

- `decode_embedded_image_to_tga_v1` dekoduje osadzony PNG/JPEG z GLB do
  limitowanego `TgaImageV1` RGB8/RGBA8; kontroluje MIME, zakres `bufferView`,
  wymiary, liczbe pikseli, wewnetrzna alokacje dekodera i finalny limit TGA;
- `write_model_package_v1` wykonuje jeden HAK write i buduje manifest z tego
  samego own-readback artifact;
- WASM `writeModelPackageV1` zwraca raport i manifest jako JSON, a HAK przekazuje
  jednokrotnie przez `takeHakBytes()` bez base64 i bez klonowania bufora.

```yaml
embedded_image_tests: "6/6 PASS"
package_manifest_tests: "6/6 PASS"
workspace_tests: "287/287 PASS"
workspace_clippy_all_targets_deny_warnings: PASS
cargo_fmt_check: PASS
wasm32_build: PASS
wasm_pack_node: "19/19 PASS"
node_boundary: PASS
git_diff_check: PASS
decoder_review_initial: "P1=0; P2=1"
decoder_review_final: "P1=0; P2=0"
compositor_review_initial: "P1=0; P2=1"
compositor_review_final: "P1=0; P2=0"
generated_file_packet: NOT_STARTED
runtime_proof: NOT_RUN
```

Nastepny krok nie wymaga klas ani generowania modulu: materializacja realnego
`MDL+MDX`, TGA, appended `appearance.2da`, HAK i manifestu z owned/synthetic
source, a potem proof na istniejacym `sandbox.mod` przy uzyciu test creature
utworzonego w Toolsecie.
