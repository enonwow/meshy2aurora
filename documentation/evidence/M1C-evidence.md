# M1C Evidence

Ten plik jest append-only dziennikiem dowodow dla read-only ERF/HAK V1.0 resource locatora. Wczesniejsze wpisy zachowuja stan z chwili zapisu; najnowszy checkpoint ponizej jest rozstrzygajacy, a biezacy status etapu pozostaje w `documentation/orchestrator-state.yaml`.

## M1C-20260711-01 - 2026-07-11

Status: IN_PROGRESS
Owner: Codex orchestrator + M1C implementation/review agents
Stage: M1C

### Cel proby

Zaimplementowac i niezaleznie zweryfikowac wlasny read-only locator ERF/HAK V1.0, a nastepnie wykonac canonical CEP lookup in-place i przekazac znaleziony type-2002 slice do M1B bez zapisania zewnetrznego payloadu.

### Aurora First / provenance

- `C:\Projects\New Folder\export\decompiled_all.c:8477` - literal `HAK V1.0`.
- `C:\Projects\New Folder\export\decompiled_all.c:122308-122313` - Aurora porownuje pierwsze osiem bajtow z `HAK V1.0`.
- `C:\Projects\New Folder\export\decompiled_all.c:15430` - literal `ERF V1.0`.
- `C:\Projects\Claude\Radoub\Documentation\BioWare_Original_PDFs\Bioware_Aurora_ERF_Format.pdf` - official exact-layout final review gate; wynik jeszcze nie wpisany.
- `documentation/m1c-kontrakt-suplement-codex.md` - REQUIRED/DEFERRED/OPEN, diagnostyki i negative matrix.
- Syntetyczne fixture sa generowane w testach. CEP jest czytany w miejscu wskazanym przez `M2A_REFERENCE_CEP_HAK` i nie jest kopiowany do repo.

### Zmienione pliki

- `documentation/m1c-kontrakt-suplement-codex.md` - suplement kontraktu M1C.
- `documentation/evidence/M1C-evidence.md` - szkielet i pozniejsze append-only wyniki.
- `<UZUPELNIC po review: pliki implementacji i testow>`

### Gate 1 - official PDF exact-layout cross-check

Status: PENDING

| PDF page/section | Field or rule | Implemented anchor | Result |
|---|---|---|---|
| `<page/section>` | 160-byte header and exact fields | `<file:line>` | PENDING |
| `<page/section>` | key V1.0: 16+4+2+2 bytes | `<file:line>` | PENDING |
| `<page/section>` | resource entry: offset+size | `<file:line>` | PENDING |
| `<page/section>` | resource ID indexes resource list | `<file:line>` | PENDING |
| `<page/section>` | zero-length/offset behavior, if specified | `<file:line or NOT_SPECIFIED>` | PENDING |

Gate zalicza dopiero reviewer, ktory faktycznie otworzyl PDF i podal strony/sekcje. `documentation/aurora-hak-erf-codex.md` ani supplementary implementation nie zastepuja tego kroku.

### Gate 2 - syntetyczna macierz pozytywna i negatywna

Status: PENDING

| Command or action | Expected | Actual | Status |
|---|---|---|---|
| `cargo test -p m2a-core --test erf` | wszystkie layout, lookup, ID, zero-size, overlap i no-panic cases przechodza | `<UZUPELNIC>` | PENDING |
| valid `ERF V1.0` fixture | resource lookup zwraca expected bytes | `<UZUPELNIC>` | PENDING |
| valid `HAK V1.0` fixture | case-insensitive resref + exact type | `<UZUPELNIC>` | PENDING |
| shuffled resource IDs | lookup idzie przez resource ID | `<UZUPELNIC>` | PENDING |
| duplicate normalized key | stable duplicate diagnostic | `<UZUPELNIC>` | PENDING |
| zero-length payload | znaleziony pusty slice, nie missing | `<UZUPELNIC>` | PENDING |
| truncated prefixes/random bytes | zero panic | `<UZUPELNIC>` | PENDING |

### Gate 3 - public diagnostics/API review

Status: PENDING

| Check | Expected | Actual | Status |
|---|---|---|---|
| error schema | `schemaVersion`, stable `code`, `offset`, `context` | `<UZUPELNIC>` | PENDING |
| missing resource | `M2A-ERF-RESOURCE-MISSING` | `<UZUPELNIC>` | PENDING |
| internal ownership | validated archive borrows input bytes | `<UZUPELNIC>` | PENDING |
| M1B handoff | located type-2002 slice trafia bezposrednio do own MDL reader | `<UZUPELNIC>` | PENDING |
| filesystem side effects | brak extraction/write API w M1C | `<UZUPELNIC>` | PENDING |

### Gate 4 - env-gated canonical CEP lookup

Status: PENDING

#### Clean skip

| Command | Expected | Actual | Status |
|---|---|---|---|
| `Remove-Item Env:M2A_REFERENCE_CEP_HAK -ErrorAction SilentlyContinue; cargo test -p m2a-core --test erf_reference_integration -- --nocapture` | test pomija cleanly, bez FAIL i bez zapisu payloadu | `<UZUPELNIC>` | PENDING |

#### Read-only lookup in place

Uruchomienie lokalne nie zapisuje host path do committable raportu:

```powershell
$env:M2A_REFERENCE_CEP_HAK = '<local cep3_core1.hak path>'
cargo test -p m2a-core --test erf_reference_integration -- --nocapture
```

| Reference | Resref | Type | Expected invariant | Actual | Status |
|---|---|---:|---|---|---|
| R1 | `c_kocrachn` | 2002 | found by own locator; stable ID/offset/size/hash metadata | `<UZUPELNIC>` | PENDING |
| R3a | `c_phod_horror_b` | 2002 | found by own locator | `<UZUPELNIC>` | PENDING |
| R3b | `c_phod_horror_p` | 2002 | found by own locator | `<UZUPELNIC>` | PENDING |

Committable identity zapisuje tylko source kind, logical container label, resref, type, resource ID, offset, size i SHA-256. Nie zapisuje host path ani payloadu.

### Gate 5 - pelne bramki jakosci

Status: PENDING

| Command or action | Expected | Actual | Status |
|---|---|---|---|
| `cargo fmt --all --check` | exit 0 | `<UZUPELNIC>` | PENDING |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 | `<UZUPELNIC>` | PENDING |
| `cargo test --workspace` | exit 0 | `<UZUPELNIC>` | PENDING |
| `cargo build -p m2a-wasm --target wasm32-unknown-unknown` | exit 0 | `<UZUPELNIC>` | PENDING |
| `<public WASM adapter test command>` | public boundary executes successfully | `<UZUPELNIC>` | PENDING |
| `git diff --check` | no whitespace errors | `<UZUPELNIC>` | PENDING |
| `git status --short` + tracked-file audit | no retail/CEP payload, extracted model or private path | `<UZUPELNIC>` | PENDING |

### Problemy i bledy

```yaml
current_problems:
  - id: "M1C-PDF-GATE"
    status: "OPEN"
    detail: "Official BioWare ERF PDF exact page/field cross-check nie zostal jeszcze zapisany."
    next_action: "Reviewer otwiera PDF, wpisuje page/section mapping i porownuje implementacje pole po polu."
  - id: "M1C-CANONICAL-CEP"
    status: "OPEN"
    detail: "Canonical env-gated CEP lookup i metadata R1/R3 nie maja jeszcze wpisanego actual result."
    next_action: "Najpierw clean-skip, potem read-only test z M2A_REFERENCE_CEP_HAK bez kopiowania payloadu."
  - id: "M1C-M1B-HANDBACK"
    status: "OPEN"
    detail: "Locator musi przekazac canonical type-2002 slice do M1B i odblokowac P-REF R1/R3."
    next_action: "Po zielonym locatorze uruchomic own-reader P-REF i zapisac tylko strukturalne metadata/hash."
bugs: []
```

Brak implementacji lub niewykonana bramka jest `current_problem`, nie bugiem. Bugs dopisujemy dopiero po obserwowalnym odchyleniu dzialajacej funkcji od kontraktu.

### Evidence artifacts

- `documentation/m1c-kontrakt-suplement-codex.md` - kontrakt review i negative matrix.
- `<UZUPELNIC: exact native test output>`
- `<UZUPELNIC: exact clean-skip output>`
- `<UZUPELNIC: canonical metadata/hash report without payload or host path>`
- `<UZUPELNIC: M1B P-REF handback evidence>`

### Nastepny krok

Wykonac official PDF exact-layout review, uzupelnic actual results wszystkich gate, naprawic kazdy finding i dopiero wtedy przekazac M1C do niezaleznego final review. Nie zmieniac statusu na `DONE` na podstawie samego istnienia kodu lub zielonego pojedynczego testu.

## M1C-20260711-01 - final checkpoint

Status: DONE
Owner: Codex orchestrator + M1C implementation/review agents
Stage: M1C

### Cel checkpointu

Zamknac wlasny read-only ERF/HAK V1.0 locator po official-format review, pelnej macierzy syntetycznej, canonical CEP/P-REF runie i niezaleznym re-review bez findings, a nastepnie oddac jedyny aktywny etap z powrotem do M1B.

### Aurora First i official PDF mapping

| Source | Confirmed contract | Implementation/test anchor | Status |
|---|---|---|---|
| `decompiled_all.c:8477`, `122308-122313` | Aurora rozpoznaje exact `HAK V1.0` w pierwszych 8 bajtach | `crates/m2a-core/src/erf.rs`, signature/version tests | PASS |
| BioWare ERF PDF s.1-2 | header 160 B: file type/version, language/localized fields, entry count, table offsets, build fields, description strref i reserved | header constants/reads oraz OOB tests | PASS |
| BioWare ERF PDF s.3 | key 24 B: resref[16], sequential ResID rowny key index, type i unused | strict resource-ID tests | PASS |
| BioWare ERF PDF s.4 | resource entry 8 B: payload offset+size, one-to-one z key | resource-table/range tests | PASS |
| canonical shipped `cep3_core1` audit | uppercase i `-` rozszerzaja PDF lowercase do waskiego `[A-Za-z0-9_-]`; lookup pozostaje ASCII case-insensitive | uppercase, hyphen i invalid-punctuation regressions | PASS |

Canonical alphabet audit objal 6402 keys, w tym 3517 type-2002: `-` wystapil w 66 resrefach ogolem i 36 type-2002, uppercase w 47 resrefach/145 znakach, empty `0`, bad NUL padding `0`, pelne 16 B `81`.

### Weryfikacja

| Command or action | Actual | Status |
|---|---|---|
| `cargo test -p m2a-core --test erf` | 18 synthetic ERF tests; 0 failed | PASS |
| canonical `M2A_REFERENCE_CEP_HAK` integration | 1 env-gated canonical test; R1/R3 packets z own locator+MDL reader; 0 failed | PASS |
| `cargo test -p m2a-core --test mdl` | 34 MDL tests; 0 failed | PASS |
| `cargo test -p m2a-core --test reference_proof` | 13 P-REF tests; 0 failed | PASS |
| `cargo test -p m2a-core --lib` | 2 unit tests; 0 failed | PASS |
| `cargo test --workspace` | pelna native macierz zielona | PASS |
| `cargo fmt --all -- --check` | brak roznic | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | brak warnings | PASS |
| `cargo build -p m2a-wasm --target wasm32-unknown-unknown` | build zakonczony | PASS |
| `wasm-pack test --node crates/m2a-wasm` | 4 publiczne Node/WASM tests; 0 failed | PASS |
| `git diff --check` | brak whitespace errors | PASS |
| independent final re-review | brak findings | PASS |

### Canonical R1/R3 locator i P-REF

Wszystkie zakresy sa polotwarte `[start,end)`. Container label jest logiczny; prywatna sciezka hosta nie trafia do packetu ani evidence.

| Ref | Resref/type | Resource ID | Container payload range / bytes | MDL core range | MDL raw range | SHA-256 | Status |
|---|---|---:|---|---|---|---|---|
| R1 | `c_kocrachn` / 2002 | 724 | `[179725952,179889144)` / 163192 | `[12,76060)` / 76048 | `[76060,163192)` / 87132 | `f16426310f826ae2ab15034ac979c65f812ee8bda0d13ee459bf2b293d7db270` | PASS |
| R3a | `c_phod_horror_b` / 2002 | 1026 | `[264142176,264988240)` / 846064 | `[12,788428)` / 788416 | `[788428,846064)` / 57636 | `62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a` | PASS |
| R3b | `c_phod_horror_p` / 2002 | 1027 | `[264988240,265834304)` / 846064 | `[12,788428)` / 788416 | `[788428,846064)` / 57636 | `09e43ee9493d2fe2bbf9cbeb44f24dcb999e5f38e651bdc79eefdd5e1f19722f` | PASS |

Own locator zwrocil borrowed subrange kontenera, own MDL reader potwierdzil exact payload/core/raw invariants, a P-REF builder zwiazal raport i SHA-256 z tym samym `&[u8]`.

### Review findings - final status

Wszystkie piec findings ma status `FIXED` i regresje:

1. `ResID` jest strict sequential i rowny key index; permutacje/powtorzenia sa odrzucane.
2. `ErfLimits::default().max_entry_count = 262144` blokuje nadmierna parser-owned allocation przed odczytem tabel.
3. Zero-size payload przy exact EOF jest legalny, a EOF+1 jest odrzucany.
4. Stored/query alphabet odzwierciedla canonical shipped `[A-Za-z0-9_-]`, zachowuje case-insensitive lookup i odrzuca pozostala interpunkcje/control.
5. Canonical test wiaze exact ID/offset/size/hash/core/raw ranges, sprawdza borrowed slice oraz blokuje private path i embedded `payload`/`bytes` w P-REF.

Finalny code/test re-review po poprawkach nie znalazl nowych findings.

### Payload, path i binary boundary

- Brak retail/CEP HAK, MDL, MDX, tekstur, animacji i szkieletow w Git.
- Brak extracted payloadow, prywatnych host paths i embedded `payload`/`bytes` w committable P-REF/evidence.
- Brak skompilowanych binaries/build output w checkpoint files; test korzysta z lokalnego HAK tylko read-only przez env.
- Repo przechowuje jedynie kod, synthetic builders, logiczne identities, hashe, rozmiary, ID, offsety i invariant results.

### Zmienione pliki

- `crates/m2a-core/src/lib.rs` - publiczny eksport readera ERF.
- `crates/m2a-core/src/erf.rs` - validated read-only ERF/HAK V1.0 view, limits, lookup i diagnostyki.
- `crates/m2a-core/tests/erf.rs` - 18-testowa macierz syntetyczna i negatywna.
- `crates/m2a-core/tests/erf_reference_integration.rs` - canonical env-gated R1/R3 locator, P-REF i no-payload/path assertions.
- `documentation/m1c-kontrakt-suplement-codex.md` - finalny REQUIRED/DEFERRED/override contract.
- `documentation/evidence/M1C-evidence.md` - ten append-only checkpoint.
- `documentation/evidence/M1B-evidence.md` - handback canonical R1/R3 do aktywnego M1B.
- `documentation/raport-m1b-deep-reader-codex.md` - aktualny pozostaly zakres M1B.
- `documentation/orchestrator-state.yaml` - M1C `DONE`, M1B jedynym aktywnym etapem.

### Problemy i bledy

```yaml
current_problems: []
bugs: []
review_findings:
  fixed: 5
  open: 0
  final_re_review: "NO_FINDINGS"
```

### Nastepny krok

M1C pozostaje zamknietym read-only foundation. M1B jest ponownie `IN_PROGRESS`: wybrac canonical R4-R6 i zamknac albo jawnie sklasyfikowac GB-001-SKIN. Nie oznaczac M1B jako `DONE` przed tym evidence.
