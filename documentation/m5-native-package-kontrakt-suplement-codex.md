# M5 - native texture, 2DA i HAK contract

Data: 2026-07-13 | Autor: Codex | Status: M5_CONTRACT_LOCKED_SLICE_A_NEXT

## 1. Zakres i status

M5 buduje strukturalny, local-first pakiet zasobow z wlasnych artefaktow:

1. deterministic TGA type 2;
2. preserve-and-append `appearance.2da` V2.0;
3. deterministic HAK/ERF V1.0 z own-reader readbackiem;
4. manifest laczacy model, teksture i appended row.

M5 konczy sie statusem structural DONE. Toolset/game resolution TGA, 2DA i HAK
pozostaje `OPEN_M6`; wymaganie runtime w M5 tworzyloby cykl, bo M6 zalezy od
M5. TXI oraz typed GFF/UTC/IFO/ARE/GIT nie sa objete pierwszymi slice'ami.

## 2. Primary evidence i clean-room boundary

Primary, read-only:

- oficjalny `Bioware_Aurora_2DA_Format.pdf`, strony 1-4;
- oficjalny `Bioware_Aurora_ERF_Format.pdf`, strony 1-4;
- lokalne retail TGA `t_karandas.tga` i `t_malagr.tga`, tylko inventory/hash;
- `C:\Projects\New Folder\export\decompiled_all.c` dla pozniejszych
  `Appearance_Type` i `Mod_HakList` proofow.

PDF-y zostaly wyrenderowane i wizualnie sprawdzone 2026-07-13. Radoub i
`aurora-web` pozostaja supplementary/read-only; kod ani payload nie jest
kopiowany. Testy uzywaja wylacznie synthetic/owned bytes.

## 3. Slice A - deterministic TGA type 2

### 3.1 Publiczny input

Input jest juz zdekodowanym buforem top-left RGB8 albo RGBA8. Slice A nie
dekoduje PNG/JPEG, nie resize'uje, nie wykonuje PBR bake i nie emituje TXI.
Format 24/32 bpp jest jawny, nigdy heurystyczny.

```rust
pub enum TgaPixelFormatV1 { Rgb8, Rgba8 }

pub struct TgaImageV1 {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub pixel_format: TgaPixelFormatV1,
    pub pixels: Vec<u8>,
}

pub struct TgaWriterLimitsV1 {
    pub max_output_bytes: u64,
}

pub struct TgaWriterOptionsV1 {
    pub schema_version: u32,
    pub limits: TgaWriterLimitsV1,
}

pub struct TgaWriterReportV1 {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub pixel_format: TgaPixelFormatV1,
    pub pixel_depth: u8,
    pub descriptor: u8,
    pub pixel_data_offset: u64,
    pub pixel_data_length: u64,
    pub footer_offset: u64,
    pub byte_length: u64,
    pub input_sha256: String,
    pub output_sha256: String,
}

pub struct TgaArtifactV1 {
    pub payload: Vec<u8>,
    pub report: TgaWriterReportV1,
}

pub struct TgaWriteError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

pub fn write_tga_v1(
    image: &TgaImageV1,
    options: &TgaWriterOptionsV1,
) -> Result<TgaArtifactV1, TgaWriteError>;
```

Input/options sa `camelCase`, `deny_unknown_fields`; pixel format jest
`RGB8 | RGBA8` w JSON. Default i hard `maxOutputBytes` wynosi `64 MiB`; caller
moze limit tylko obnizyc. Raport nie zawiera pixel payloadu ani host paths.

### 3.2 Exact bytes

```text
header: 18 bytes
  idLength=0, colorMapType=0, imageType=2
  colorMapSpec=0, xOrigin=0, yOrigin=0
  width=u16 LE, height=u16 LE
  pixelDepth=24|32
  descriptor=0 dla RGB8, 8 dla RGBA8
pixels:
  bottom-left row order
  BGR dla RGB8, BGRA dla RGBA8
footer: 26 bytes
  extensionOffset=0 u32 LE
  developerDirectoryOffset=0 u32 LE
  ASCII `TRUEVISION-XFILE.` + NUL
```

Wymiary musza byc `1..=65535`. Dlugosc inputu musi byc dokladnie
`width*height*channels`. Wszystkie iloczyny, rozmiary i alokacje sa checked;
input nie jest mutowany. Output nie ma ID, color map, RLE, paddingu ani trailing
bytes poza footerem.

Walidacja ma jedna kolejnosc na native i wasm32:

1. schema image/options i options `0 < maxOutputBytes <= 64 MiB`;
2. dimensions `1..=65535`;
3. checked `u64` expected pixel bytes i output bytes `18 + pixels + 26`;
4. output limit;
5. exact input pixel length;
6. `usize` conversion i fallible allocation;
7. emission oraz stage-private own readback.

Own readback pozostaje private/`pub(crate)`, nie rozszerza publicznego API.
Sprawdza exact 18-byte header, image type/depth/descriptor, zero fields, pixel
range, 26-byte footer, exact EOF i dekoduje wynik ponownie do top-left RGB(A).
Parse failure mapuje sie na `READBACK-FAILED`, semantic mismatch na
`SEMANTIC-DIFF`.

Stable taxonomy:

- `M5-TGA-SCHEMA-INVALID`;
- `M5-TGA-DIMENSIONS-INVALID`;
- `M5-TGA-PIXEL-LENGTH-INVALID`;
- `M5-TGA-OUTPUT-LIMIT-EXCEEDED`;
- `M5-TGA-ALLOCATION-FAILED`;
- `M5-TGA-READBACK-FAILED`;
- `M5-TGA-SEMANTIC-DIFF`.

TDD: 2x2 z czterema roznymi naroznikami, osobne RGB/RGBA, exact header/pixel
order/footer/EOF, determinism, immutability, dimensions/length/limit, mutation
readback i no-panic.

## 4. Slice B - preserve-and-append 2DA V2.0

Oficjalny format wymaga pierwszej linii `2DA V2.0`, drugiej linii pustej albo
`DEFAULT: <one token>`, nazw kolumn `[A-Za-z0-9_]+`, spacji zamiast tabow,
`****` jako NULL i dokladnie pelnego wiersza. String ze spacjami jest quoted i
nie moze zawierac `"`; format nie ma escape syntax. `"****"` jest zwyklym
tekstem, a `""` pustym tekstem. Kolumn i wierszy nie wolno wstawiac, usuwac
ani zmieniac w srodku.

Writer zachowuje wszystkie istniejace raw bytes 1:1 i dopisuje tylko suffix.
Akceptuje printable ASCII oraz jednolity LF albo CRLF; mixed EOL, bare CR,
BOM, NUL, tab, inne control bytes i non-ASCII sa fatal.
Jesli input nie konczy sie EOL, writer dopisuje wykryty EOL przed nowym row.
Nowy row zawsze konczy sie tym samym EOL.

Row id jest fizyczna liczba istniejacych data rows `N`, nie ostatni wydrukowany
label. Oficjalny PDF mowi, ze label jest tylko dla czlowieka, a engine sledzi
fizyczny index; dlatego istniejacy mismatch label jest warning/report, nie
fatal. Append label i przyszle UTC `Appearance_Type` sa rowne `N`, wymagane
`N <= 65535`.

Parser nie interpretuje integer/float; zachowuje lexical `NULL | TEXT`. Row
label musi byc decimal `u32`, ale mismatch z fizycznym ordinalem daje limitowany
`WARNING M5-2DA-ROW-LABEL-MISMATCH`, nie fatal. Lokalny read-only
`lc_2da.hak` ma taki realny mismatch, zgodny z regula official PDF o fizycznym
indexie.

Writer zachowuje header order, DEFAULT, rows oraz exact `****`. Nowy row jest
full-width: caller podaje named assignments case-insensitive, a niepodane
kolumny dostaja NULL. Duplicate/unknown assignment i kolizja nazw kolumn po
ASCII case-fold sa fatal. Shuffled assignments daja te same bytes. Value moze
byc tylko printable ASCII bez quote; spacje wymuszaja quoted output.

### 4.1 Publiczny kontrakt v1

`V1` ponizej wersjonuje publiczny JSON/API projektu; format payloadu pozostaje
official `2DA V2.0`.

```rust
pub enum TwoDaNewlineV1 {
    CrLf,
    Lf,
}

#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TwoDaCellValueV1 {
    Null,
    Text { value: String },
}

pub struct TwoDaCellAssignmentV1 {
    pub column_name: String,
    pub value: TwoDaCellValueV1,
}

pub struct TwoDaAppendRequestV1 {
    pub schema_version: u32,
    pub cells: Vec<TwoDaCellAssignmentV1>,
}

pub struct TwoDaLimitsV1 {
    pub max_input_bytes: u64,
    pub max_columns: u32,
    pub max_rows: u32,
    pub max_token_bytes: u32,
    pub max_diagnostics: u32,
}

pub struct TwoDaDiagnosticV1 {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub line: Option<u32>,
    pub message: String,
}

pub struct TwoDaInspectionV1 {
    pub schema_version: u32,
    pub format: String,
    pub version: String,
    pub source_sha256: String,
    pub byte_length: u64,
    pub newline: TwoDaNewlineV1,
    pub terminal_newline: bool,
    pub default_value: Option<TwoDaCellValueV1>,
    pub columns: Vec<String>,
    pub physical_row_count: u32,
    pub next_append_index: Option<u16>,
    pub row_label_mismatch_count: u32,
    pub diagnostics: Vec<TwoDaDiagnosticV1>,
}

pub struct TwoDaChangedCellV1 {
    pub column_index: u32,
    pub column_name: String,
    pub value: TwoDaCellValueV1,
}

pub struct TwoDaAppendReportV1 {
    pub schema_version: u32,
    pub source_sha256: String,
    pub output_sha256: String,
    pub source_byte_length: u64,
    pub output_byte_length: u64,
    pub source_prefix_preserved: bool,
    pub appended_row_index: u16,
    pub physical_rows_before: u32,
    pub physical_rows_after: u32,
    pub newline: TwoDaNewlineV1,
    pub inserted_separator_newline: bool,
    pub changed_cells: Vec<TwoDaChangedCellV1>,
    pub diagnostics: Vec<TwoDaDiagnosticV1>,
}

pub struct TwoDaAppendArtifactV1 {
    pub payload: Vec<u8>,
    pub report: TwoDaAppendReportV1,
}

pub struct TwoDaError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub byte_offset: u64,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub path: String,
    pub message: String,
}

pub fn inspect_two_da_v2(
    bytes: &[u8],
    limits: &TwoDaLimitsV1,
) -> Result<TwoDaInspectionV1, TwoDaError>;

pub fn append_two_da_row_v1(
    bytes: &[u8],
    request: &TwoDaAppendRequestV1,
    limits: &TwoDaLimitsV1,
) -> Result<TwoDaAppendArtifactV1, TwoDaError>;
```

Wszystkie input structs sa `camelCase`, `deny_unknown_fields`. Enum newline ma
JSON `CR_LF | LF`; cell ma jawne `{"kind":"NULL"}` albo
`{"kind":"TEXT","value":"..."}`. Dzieki temu `****`, `"****"` i pusty
tekst nie sa mylone na granicy JSON. Publiczny JSON nie zawiera host paths,
payloadu istniejacych rows ani `usize`.

Parser moze przechowywac spans do inputu, ale nie kopiuje/materializuje calej
tabeli jako drugi payload. Limity sa sprawdzane przed parser-owned allocation;
caller moze je tylko obnizyc wzgledem hard project caps. `maxRows` musi
pozwalac readerowi zwalidowac `65536` existing rows, aby append mogl zwrocic
precyzyjny `M5-2DA-APPEND-U16-OVERFLOW`.

### 4.2 Strict lexical grammar

- line 1 po usunieciu EOL jest dokladnie `2DA V2.0`;
- caly input uzywa jednego EOL: `CRLF` albo `LF`; bare CR i mixed EOL sa fatal;
- line 2 jest whitespace-only albo ma `DEFAULT:` + co najmniej jedna spacje +
  dokladnie jeden token;
- line 3 ma co najmniej jedna nazwe `[A-Za-z0-9_]+`;
- data lines nie moga byc puste; terminal EOL nie tworzy dodatkowego row;
- separator to jedna lub wiecej spacji ASCII; leading/trailing spaces sa legalne;
- quoted token zaczyna i konczy sie `"`; format nie ma escape syntax, wiec `"`
  wewnatrz value jest fatal;
- po closing quote moze wystapic tylko separator albo koniec linii;
- unquoted exact `****` jest NULL; quoted `"****"` jest TEXT;
- kazdy data row ma dokladnie `columns + 1` tokenow;
- pierwszy token row jest nieujemnym decimal `u32`; leading zero jest
  akceptowane i zachowywane w source bytes;
- mismatch jego wartosci z zero-based physical ordinalem emituje limitowany
  `WARNING M5-2DA-ROW-LABEL-MISMATCH`, nigdy nie zmienia indeksu;
- BOM, bytes `>= 0x80`, NUL, TAB i control bytes poza EOL sa fatal.

Reader nie wykonuje type inference ani numeric normalization. `DEFAULT:` jest
raportowane jako lexical `NULL | TEXT`, lecz append nie ocenia jego engine
fallback semantics i nigdy go nie zmienia.

### 4.3 Exact suffix i encoding nowego row

Dla `N = physical_row_count`:

```text
input ma terminal EOL:
  output = input || decimal(N) || " " || cells.join(" ") || EOL

input nie ma terminal EOL:
  output = input || EOL || decimal(N) || " " || cells.join(" ") || EOL
```

Writer gwarantuje `output[0..input.len()] == input`; nie poprawia alignmentu,
row labels, trailing spaces ani zadnego innego source byte. Nowy row uzywa
jednej spacji miedzy tokenami i zawsze konczy sie source EOL.

`N <= 65535` jest legalne, zatem append przy `65535` existing rows emituje row
`65535`. Przy `65536` existing rows nowy index `65536` jest fatal. Printed label,
`appendedRowIndex` i przyszle UTC `Appearance_Type` sa ta sama wartoscia `N`.

Encoding cells jest zamrozony:

```text
NULL                         -> ****
TEXT("")                    -> ""
TEXT("****")                -> "****"
TEXT zawierajacy space       -> quoted
pozostaly TEXT               -> unquoted
```

Nowy TEXT jest printable ASCII i nie moze zawierac quote, CR, LF, TAB ani NUL.
Assignments sa rozwiazywane ASCII-case-insensitive; source spelling/order
kolumn pozostaje bez zmian. Collision nazw po ASCII case-fold, duplicate
assignment i missing target column sa fatal. Output cells sa zawsze w source
column order, a wszystkie niepodane cells sa NULL.

### 4.4 Validation order i own-readback

Kolejnosc jest identyczna native/wasm32:

1. hard input byte limit i bezpieczna konwersja dlugosci;
2. encoding/control/EOL scan;
3. version, DEFAULT i columns;
4. streaming validation wszystkich rows, arity, quotes, row labels i limits;
5. request schema, column case-fold uniqueness i assignments;
6. physical append index `u16` boundary;
7. checked suffix/output length oraz fallible allocation;
8. append-only emission;
9. stage-private own-readback.

Own-readback reparsuje output i sprawdza exact source prefix, identyczne
columns/default/existing rows, `N+1` physical rows, printed/physical index `N`,
full-width appended row, kazda cell oraz source/output SHA-256. Parse failure
mapuje sie na `READBACK-FAILED`, mismatch na `SEMANTIC-DIFF`; nie ma czesciowego
artifactu.

Own-readback sprawdza exact source prefix, identyczne columns/default/existing
rows, `N+1` rows, label/fizyczny index `N`, full-width row, wszystkie nowe
cells oraz source/output SHA-256. Publiczny JSON nie zawiera `usize`.

Stable taxonomy:

- `M5-2DA-SCHEMA-INVALID`;
- `M5-2DA-LIMIT-EXCEEDED`;
- `M5-2DA-HEADER-INVALID`;
- `M5-2DA-ENCODING-UNSUPPORTED`;
- `M5-2DA-NEWLINE-INVALID`;
- `M5-2DA-TAB-FORBIDDEN`;
- `M5-2DA-NUL-FORBIDDEN`;
- `M5-2DA-DEFAULT-INVALID`;
- `M5-2DA-COLUMN-INVALID`;
- `M5-2DA-COLUMN-AMBIGUOUS`;
- `M5-2DA-ROW-LABEL-INVALID`;
- `M5-2DA-ROW-ARITY-INVALID`;
- `M5-2DA-QUOTE-INVALID`;
- `M5-2DA-ASSIGNMENT-DUPLICATE`;
- `M5-2DA-ASSIGNMENT-COLUMN-MISSING`;
- `M5-2DA-VALUE-INVALID`;
- `M5-2DA-APPEND-U16-OVERFLOW`;
- `M5-2DA-LAYOUT-OVERFLOW`;
- `M5-2DA-READBACK-FAILED`;
- `M5-2DA-SEMANTIC-DIFF`.

`M5-2DA-ROW-LABEL-MISMATCH` jest wylacznie kodem `WARNING`, nie fatal taxonomy.
WASM malformed request JSON ma osobny boundary fatal
`M5-2DA-REQUEST-JSON-INVALID` o tym samym stabilnym shape.

### 4.5 Required tests i public WASM

Happy/compatibility:

- minimal LF i CRLF; blank line i quoted DEFAULT;
- multiple/leading/trailing spaces;
- quoted label ze spacjami, `****`, `"****"` oraz `""` round-trip;
- terminal EOL present/absent i exact append suffix;
- source prefix byte-identical, source input immutable;
- mismatch/duplicate displayed row labels daje warning i nadal appenduje pod
  physical `N`;
- synthetic pattern odpowiadajacy lokalnemu `15219 rows / duplicate 15152 /
  missing 15153`;
- starred-out row nie jest reuse hole;
- shuffled assignments daja identyczne bytes/report;
- unspecified columns sa NULL, output row ma full width;
- `N=65535` success i `N=65536` fatal;
- deterministic output/report, source/output SHA i own-readback;
- synthetic owned 35-column appearance handoff do HAK writera.

Negative/mutation/no-panic:

- bad/truncated signature lub version, missing lines;
- BOM, non-ASCII, NUL, TAB, control, mixed EOL i bare CR;
- invalid DEFAULT, column name/case-fold collision;
- unclosed/embedded quote i znak bez separatora po closing quote;
- interior blank row, missing/extra cell, invalid/overflowed row label;
- duplicate/unknown assignment i invalid generated TEXT;
- every truncated prefix, limit/allocation/layout boundaries i arbitrary bytes
  pod `catch_unwind`.

Publiczny adapter:

```text
inspectTwoDaV2Json(bytes) -> String
appendTwoDaRowV2(bytes, requestJson) -> Result<Uint8Array, JsValue>
appendTwoDaRowV2ReportJson(bytes, requestJson) -> String
```

Adapter tylko strict-dekoduje JSON i deleguje do core. Nie emituje base64.
Native i wasm32 asertywnie uzywaja wspolnego frozen output length/SHA oraz
identycznego report JSON; source `Uint8Array` pozostaje niemutowany.

## 5. Slice C - deterministic HAK V1.0

Publiczny resource ma lowercase resref `[a-z0-9_]{1,16}`, dowolny `u16`
resource type i owned payload bytes. Writer sortuje po `(resref bytes, type)`,
odrzuca duplicate key i nadaje `resource_id` po sortowaniu.

```text
keyOffset      = 0xA0
resourceOffset = 0xA0 + 24*N
payloadOffset  = 0xA0 + 32*N
fileSize       = payloadOffset + sum(payload sizes)
```

Header ma `HAK `, `V1.0`, zero language/localized size, localized/key offset
`0xA0`, build year/day `0`, description strref `0xffffffff` i reserved zero.
Key ma NUL-padded resref, final id, type i unused zero. Resource ma `u32`
offset/size. Payloady sa contiguous, bez kompresji, alignmentu i paddingu.

Hard limits: `262144` entries i `256 MiB` output; caller moze je tylko obnizyc.
Kazda arytmetyka i konwersja do `u32` jest checked przed alokacja.

Stable taxonomy:

- `M5-HAK-OPTIONS-INVALID`;
- `M5-HAK-RESREF-INVALID`;
- `M5-HAK-DUPLICATE-KEY`;
- `M5-HAK-ENTRY-LIMIT-EXCEEDED`;
- `M5-HAK-OUTPUT-LIMIT-EXCEEDED`;
- `M5-HAK-U32-OVERFLOW`;
- `M5-HAK-ALLOCATION-FAILED`;
- `M5-HAK-READBACK-FAILED`;
- `M5-HAK-SEMANTIC-DIFF`.

Po emisji writer zawsze uruchamia `ErfArchive::parse_with_limits` i porownuje
file type, kolejnosc, IDs, types, resrefs, offsets, sizes, exact payload bytes,
SHA-256 per payload, exact EOF i archive SHA. Retail/CEP pozostaja in-place i
nie sa fixture writera.

## 6. Jawne odroczenia

- TXI: brak zamrozonego profilu dyrektyw/kodowania/kolejnosci; domyslnie brak.
- Full image bake: potrzebny osobny payload/decode/resize contract.
- Generic GFF V3.2: kierunek istnieje, lecz exact field/layout contract nie jest
  jeszcze zamrozony.
- Typed UTC/IFO/ARE/GIT: wymagaja exact label/type/struct-id manifests oraz
  read-only packets; nie implementowac przez zgadywanie.
- Toolset/game acceptance calego pakietu: `OPEN_M6`.
