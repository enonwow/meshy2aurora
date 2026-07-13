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

Project caps sa zamrozone i jednoczesnie sa wartosciami `Default`:

```text
maxInputBytes   = 16_777_216   (16 MiB)
maxColumns      = 4_096
maxRows         = 65_536
maxTokenBytes   = 1_048_576    (1 MiB)
maxDiagnostics  = 2_048
```

`16 MiB` ma zapas nad najwiekszym lokalnie zinwentaryzowanym read-only 2DA
(`9_260_475` bytes). `65_536` jest celowe: reader musi przyjac tyle istniejacych
rows, aby append zwrocil precyzyjny `M5-2DA-APPEND-U16-OVERFLOW`. Kazdy limit
rowny zero albo wiekszy od odpowiedniego project cap jest odrzucany jako
`M5-2DA-SCHEMA-INVALID`; wartosci nie sa clampowane. Caller moze podac wartosc
`1..=cap`.

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

1. limits sa niezerowe i `<=` hard project caps; dla append takze request
   `schemaVersion == 1`;
2. hard/configured input byte limit i bezpieczna konwersja dlugosci;
3. encoding/control/EOL scan;
4. version, DEFAULT i columns;
5. streaming validation wszystkich rows, arity, quotes, row labels i limits;
6. column case-fold uniqueness i assignments;
7. physical append index `u16` boundary;
8. checked suffix/output length oraz fallible allocation;
9. append-only emission;
10. stage-private own-readback.

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
- synthetic owned 35-column appearance artifact gotowy do handoffu; Slice B
  sprawdza full-width 2DA i own readback, a faktyczne przekazanie jego payloadu
  do HAK writera jest bramka integracyjna Slice C (writer HAK jeszcze nie istnieje
  podczas zamykania Slice B).

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

### 5.1 Publiczne API i JSON

```rust
pub struct HakResourceInputV1 {
    pub resref: String,
    pub resource_type: u16,
    pub payload: Vec<u8>,
}

pub struct HakWriterLimitsV1 {
    pub max_entry_count: u64,
    pub max_output_bytes: u64,
}

pub struct HakWriterOptionsV1 {
    pub schema_version: u32,
    pub limits: HakWriterLimitsV1,
}

pub struct HakResourceReportV1 {
    pub resref: String,
    pub resource_id: u32,
    pub resource_type: u16,
    pub payload_offset: u32,
    pub payload_size: u32,
    pub payload_sha256: String,
}

pub struct HakWriterReportV1 {
    pub schema_version: u32,
    pub entry_count: u32,
    pub key_table_offset: u32,
    pub resource_table_offset: u32,
    pub payload_offset: u32,
    pub byte_length: u64,
    pub archive_sha256: String,
    pub resources: Vec<HakResourceReportV1>,
}

pub struct HakArtifactV1 {
    pub payload: Vec<u8>,
    pub report: HakWriterReportV1,
}

pub struct HakWriteError {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub path: String,
    pub message: String,
}

pub fn write_hak_v1(
    resources: &[HakResourceInputV1],
    options: &HakWriterOptionsV1,
) -> Result<HakArtifactV1, HakWriteError>;
```

`HakResourceInputV1`, limits i options sa strict `camelCase`,
`deny_unknown_fields`, `Serialize + Deserialize`. Reporty i blad sa
`camelCase`, `Serialize`; artifact jest binarnym carrierem i nie jest
serializowany do JSON. `severity` bledu jest exact `FATAL`.

Schema options wynosi `1`. Default i hard `maxEntryCount` wynosi `262144`;
caller moze ustawic `0..=262144`, gdzie zero dopuszcza tylko pusty HAK. Default
i hard `maxOutputBytes` wynosi `256 MiB`; caller moze ustawic
`160..=256 MiB`. Inna schema lub limit daje `M5-HAK-OPTIONS-INVALID`.

Publiczny resource ma lowercase resref `[a-z0-9_]{1,16}`, dowolny `u16`
resource type i owned payload bytes. Writer niczego nie normalizuje. Ten sam
resref z roznymi typami jest legalny; duplicate oznacza dokladnie ten sam
`(resref, resource_type)`. Pusty payload jest legalny. Pusta lista resources
jest legalna i emituje exact 160-byte HAK.

### 5.2 Deterministyczna kolejnosc i exact layout

Po walidacji writer sortuje rosnaco po `(resref.as_bytes(), resource_type)`,
gdzie type jest porownywany numerycznie jako `u16`. Input order nie trafia do
raportu. `resource_id` jest nadawany 0-based dopiero po finalnym sortowaniu;
key, resource descriptor, raport i payload maja te sama kolejnosc. Wszystkie
permutacje tego samego zbioru resources daja byte-identyczny artifact i report.

```text
keyOffset      = 0xA0
resourceOffset = 0xA0 + 24*N
payloadOffset  = 0xA0 + 32*N
fileSize       = payloadOffset + sum(payload sizes)
```

Wszystkie liczby sa little-endian. Exact 160-byte header ma:

| Offset | Wartosc |
|---:|---|
| `0x00` | `HAK ` |
| `0x04` | `V1.0` |
| `0x08` | language count `u32 = 0` |
| `0x0c` | localized string size `u32 = 0` |
| `0x10` | entry count `u32 = N` |
| `0x14` | localized string offset `u32 = 0xA0` |
| `0x18` | key offset `u32 = 0xA0` |
| `0x1c` | resource offset `u32` z planu |
| `0x20` | build year `u32 = 0` |
| `0x24` | build day `u32 = 0` |
| `0x28` | description strref `u32 = 0xffffffff` |
| `0x2c..0xa0` | 116 reserved zero bytes |

Key ma 24 bytes: NUL-padded `resref[16]` (pelne 16 bytes nie ma terminatora),
final `resource_id u32`, `resource_type u16`, `unused u16 = 0`. Resource entry
ma `payload_offset u32`, `payload_size u32`. Payloady sa contiguous w sorted
order, bez kompresji, alignmentu, gaps, paddingu, footera i trailing bytes.
Zero-length payload ma offset rowny biezacemu cursorowi i nie przesuwa cursora;
kilka pustych payloadow moze miec ten sam offset. Dla pustego HAK localized,
key, resource i EOF sa rowne `0xA0`.

### 5.3 Kolejnosc walidacji i arytmetyka

Native i wasm32 uzywaja tej samej kolejnosci:

1. options schema oraz zakresy obu limits;
2. input entry count wzgledem configured/hard `maxEntryCount`;
3. resref kazdego inputu w original input order;
4. duplicate `(resref,type)` w original input order;
5. final sort i length-only layout plan w `u64`;
6. checked key/resource/payload/file sizes i configured `maxOutputBytes`;
7. checked konwersje finalnego count, offsetow i sizes do `u32`;
8. checked konwersja output length do `usize` i fallible allocation;
9. emisja, private exact-layout verification i `ErfArchive` semantic readback.

Przekroczenie configured/hard output limit zawsze wygrywa z pozniejsza
konwersja do `u32`. Przy obecnym hard `256 MiB` publiczny input nie osiaga
`u32` overflow; `M5-HAK-U32-OVERFLOW` pozostaje defense-in-depth dla checked
plannera i jest testowany przez private length-only planner seam bez alokowania
gigabajtowych payloadow. Writer nie wykonuje infallible allocation zaleznej od
inputu.

### 5.4 Private exact verifier i own-reader gate

`ErfArchive` pozostaje publicznym, szerszym read-only readerem i nie jest
zaostrzany pod generated-only policy. Writer po emisji uruchamia dwa gate'y:

1. stage-private exact-layout verifier sprawdza wszystkie header fields,
   reserved zero, table offsets/sizes, NUL padding, key unused zero, sequential
   IDs, sorted order, contiguous descriptors/payloady, zero-payload cursor,
   brak gaps/footera/trailing bytes i exact EOF;
2. `ErfArchive::parse_with_limits` sprawdza `Hak`, liczbe i kolejnosc metadata,
   IDs/types/resrefs/offsets/sizes, po czym `find` porownuje exact payload bytes
   i SHA-256 kazdego payloadu.

Writer dodatkowo liczy SHA-256 calego artifactu i porownuje report byte length
z exact EOF. Blad private layout albo `ErfArchive` parse mapuje sie na
`M5-HAK-READBACK-FAILED`; poprawny parse z roznica metadata, bytes lub hash
mapuje sie na `M5-HAK-SEMANTIC-DIFF`. Verifier jest private/`pub(crate)` i nie
rozszerza publicznego API.

### 5.5 Stable taxonomy, paths i JSON

| Code | Stable path |
|---|---|
| `M5-HAK-OPTIONS-INVALID` | `options.schemaVersion`, `options.limits.maxEntryCount` albo `options.limits.maxOutputBytes` |
| `M5-HAK-RESREF-INVALID` | `resources[i].resref` |
| `M5-HAK-DUPLICATE-KEY` | `resources[i]` drugiego input entry |
| `M5-HAK-ENTRY-LIMIT-EXCEEDED` | `resources` |
| `M5-HAK-OUTPUT-LIMIT-EXCEEDED` | `options.limits.maxOutputBytes` |
| `M5-HAK-U32-OVERFLOW` | `layout` |
| `M5-HAK-ALLOCATION-FAILED` | `output` |
| `M5-HAK-READBACK-FAILED` | `output` |
| `M5-HAK-SEMANTIC-DIFF` | `output` |

Error JSON ma exact field order `schemaVersion`, `code`, `severity`, `path`,
`message`. Report JSON ma field order zgodny z deklaracja typow w 5.1, a
resources sa w final sorted order. Zmiana code/path/order jest zmiana
publicznego kontraktu.

### 5.6 Obowiazkowa test matrix

Happy i determinism:

- empty HAK: exact 160 bytes, header/offsets/EOF i frozen SHA;
- minimum one resource oraz trzy shuffled resources: `appearance/2017`,
  model/2002 i texture/3;
- wszystkie permutacje tego samego zbioru daja exact bytes/report;
- bytewise resref sort, numeric type sort, sequential IDs oraz payload order;
- ten sam resref z roznymi typami jest legalny; duplicate exact key jest fatal;
- resref length 1 i 16, exact NUL padding i brak terminatora dla length 16;
- jeden i kilka zero-length payloadow, w tym finalny empty payload na EOF;
- configured entry/output limit dokladnie na granicy jest inclusive; `+1` jest
  fatal, testowane niskimi configured limits bez wielkich alokacji;
- repeated run, input immutability, exact per-payload/archive SHA i frozen
  report JSON;
- strict input/options JSON, exact enum/field names, unknown field rejection i
  frozen representative error JSON.

Negative, planner i mutation:

- options schema, limits ponizej/dozwolone/powyzej hard oraz validation
  precedence z jednoczesnie niepoprawnym inputem;
- entry limit przed resref/duplicate/layout allocation;
- empty, uppercase, hyphen, whitespace, punctuation, control, non-ASCII i
  over-16 resref;
- duplicate key przy roznych input positions; same resref/different type pass;
- checked add/multiply oraz `u32` overflow przez private length-only planner;
- deterministic test-only allocation failure seam daje `ALLOCATION-FAILED`;
- mutacja kazdego identity/header field, reserved byte, count/offset, key resref
  padding/ID/type/unused, resource offset/size, payload byte i SHA;
- key/resource overlap, payload gap/overlap, metadata overlap, footer/trailing
  byte oraz kazdy truncated prefix;
- parse mutation daje `READBACK-FAILED`, poprawny parse z payload/hash roznica
  daje `SEMANTIC-DIFF`; wszystkie public invalid cases sa no-panic.

Testy writera uzywaja wylacznie synthetic/owned bytes. Retail/CEP pozostaja
env-selected i read-only in-place; nie sa fixture source, expected payload ani
golden archive writera. Evidence zapisuje tylko layout, counts, hashes i
logiczne labels, bez payloadow i prywatnych host paths.

## 6. Jawne odroczenia

- TXI: brak zamrozonego profilu dyrektyw/kodowania/kolejnosci; domyslnie brak.
- Full image bake: potrzebny osobny payload/decode/resize contract.
- Generic GFF V3.2: kierunek istnieje, lecz exact field/layout contract nie jest
  jeszcze zamrozony.
- Typed UTC/IFO/ARE/GIT: wymagaja exact label/type/struct-id manifests oraz
  read-only packets; nie implementowac przez zgadywanie.
- Toolset/game acceptance calego pakietu: `OPEN_M6`.
