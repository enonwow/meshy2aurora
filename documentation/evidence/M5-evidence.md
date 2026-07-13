# M5 evidence - native package resources

Data: 2026-07-13

```yaml
stage: M5
status: IN_PROGRESS
attempt_id: M5-20260713-01
contract: M5_CONTRACT_LOCKED_SLICE_A_PASS_SLICE_B_NEXT
runtime_proof: OPEN_M6
retail_payload_committed: false
```

## 1. Contract-lock evidence

Sprawdzono official BioWare 2DA i ERF PDF-y (po 4 strony) przez ekstrakcje
tekstu oraz wizualny render wszystkich stron. Potwierdzono:

- 2DA V2.0 physical row indexing, append-at-EOF, full-width rows, spaces bez
  tabow, exact `****`, quoted strings bez escape quote;
- ERF/HAK V1.0 160-byte header, 24-byte key, 8-byte resource descriptor,
  contiguous raw payload oraz 16-byte lowercase resref;
- lokalne read-only TGA 24/32 bpp kotwice pozwalaja zamrozic structural type-2
  encoder bez kopiowania payloadu.

Autorytatywny kontrakt wykonawczy:

- `documentation/m5-native-package-kontrakt-suplement-codex.md`.

## 2. Skorygowany blocker dependency

Stary plan przypisywal runtime proof jednoczesnie do M5 DoD i M6, mimo ze M6
zalezy od M5 DONE. To byl cykl. M5 zamyka deterministic bytes, own readback,
native/WASM i synthetic package evidence. Toolset/game resolution pozostaje
`OPEN_M6`.

## 3. Gotowosc slice'ow

```yaml
tga_type2_encoder: IMPLEMENTATION_READY
two_da_preserve_append: IMPLEMENTATION_READY
hak_v10_writer: IMPLEMENTATION_READY
full_image_decode_resize_bake: CONTRACT_REQUIRED
txi: NOT_READY
generic_gff_v32: CONTRACT_REQUIRED
typed_utc_ifo_are_git: CONTRACT_AND_READ_ONLY_PACKETS_REQUIRED
```

## 4. Slice A - deterministic TGA type 2

Zaimplementowano publiczny `write_tga_v1` dla RGB8/RGBA8 z exact 18-byte
headerem, bottom-left BGR(A), 26-byte TGA 2.0 footerem, limitem 64 MiB,
checked arithmetic, fallible allocation, SHA-256 reportem i prywatnym own
readbackiem dekodujacym ponownie do top-left RGB(A).

Powtorzone przez orkiestratora bramki:

```yaml
targeted_tga_integration: "7/7 PASS"
private_readback_mutation_truncation_trailing_eof: "1/1 PASS"
core_clippy_all_targets_deny_warnings: PASS
cargo_fmt_check: PASS
git_diff_check: PASS
independent_review_initial: "P1=0; three P2 test gaps"
independent_review_remediation:
  - "inclusive maxOutputBytes and dimension 65535 boundary"
  - "independent width=65535 and height=65535 positive boundaries"
  - "frozen public report and error JSON"
  - "trailing byte rejected after exact EOF"
independent_final_rereview: "P1=0; P2=0"
frozen_rgba_output_sha256: "ab5365a3...360c79"
```

Slice A jest strukturalnie zamkniety. Caly M5 pozostaje `IN_PROGRESS`; nastepny
jest preserve-and-append 2DA Slice B, a potem deterministic HAK Slice C.

## 5. Slice B - preserve-and-append 2DA V2.0

Zaimplementowano strict `inspect_two_da_v2` i `append_two_da_row_v1` z fizycznym
indeksem wiersza, zachowaniem wszystkich source bytes jako exact prefix,
deterministycznym canonical suffixem, limitami i fallible allocation, stabilna
taksonomia, SHA-256 reportem oraz stage-private own readbackiem.

Najwazniejsze regresje zamkniete podczas niezaleznych reviews:

- delimiter-dense line/token collections sa bounded i fallible;
- `DEFAULT:` raportuje fizyczny column i empty value nie panikuje;
- rows i odroczony `maxRows` maja precedence przed case-fold collision;
- own readback sprawdza exact canonical suffix, nie tylko semantyczne tokens;
- lokalny pattern 15 219 rows / duplicate 15152 / missing 15153 appenduje pod
  physical index 15219 i mismatch pozostaje warningiem;
- `N=65535` append PASS, `N=65536` stable overflow fatal;
- owned full-width 35-column artifact jest gotowy do handoffu w Slice C.

```yaml
two_da_integration: "15/15 PASS"
two_da_private: "19/19 PASS"
workspace_tests: "221/221 PASS"
workspace_clippy_all_targets_deny_warnings: PASS
cargo_fmt_check: PASS
scoped_diff_check: PASS
independent_final_rereview: "P1=0; P2=0; new regressions=0"
```

Slice B jest strukturalnie zamkniety. M5 pozostaje `IN_PROGRESS`; nastepny jest
deterministic HAK V1.0 Slice C. Faktyczny 35-column 2DA -> HAK writer handoff jest
bramka integracyjna Slice C, poniewaz writer HAK nie istnial podczas Slice B.
