# HAK, 2DA i GFF crosswalk

Data: 2026-07-10 | Status: AKTYWNY KONTRAKT WIEDZY DLA M1C/M5/M6

## 1. Werdykt

Kierunek jest zamkniety: aplikacja ma wlasny writer HAK/ERF V1.0, writer 2DA oraz writer GFF V3.2 w Rust. Uzytkownik jawnie dostarcza base `appearance.2da` albo HAK, z ktorego tabela jest odczytywana. Nie bundlujemy retail/CEP danych.

## 2. Primary i supplementary evidence

Primary format references:

- `C:\Projects\Claude\Radoub\Documentation\BioWare_Original_PDFs\Bioware_Aurora_ERF_Format.pdf`;
- `C:\Projects\Claude\Radoub\Documentation\BioWare_Original_PDFs\Bioware_Aurora_2DA_Format.pdf`;
- `C:\Projects\Claude\Radoub\Documentation\BioWare_Original_PDFs\Bioware_Aurora_GFF_Format.pdf`;
- `C:\Projects\New Folder\export\decompiled_all.c:262516-262552` - odczyt `Mod_HakList/Mod_Hak`;
- `C:\Projects\New Folder\export\decompiled_all.c:264456-264519` - zapis listy z zachowaniem kolejnosci.

Supplementary read-only cross-check:

- `Radoub.Formats/Erf/ErfWriter.cs:35-124`;
- `Radoub.Formats/Ifo/IfoWriter.cs:145-160`;
- `Radoub.Formats/Resolver/ModuleHakResolver.cs:13-71`.

Radoub jest GPL-3 i pozostaje reference-only. Fakty o formacie moga byc niezaleznie zaimplementowane; kod nie jest kopiowany.

## 3. HAK/ERF V1.0

```yaml
hak_v10:
  header_size: 160
  file_type: "HAK "
  version: "V1.0"
  key_entry_size: 24
  resource_entry_size: 8
  resref:
    width: 16
    encoding: "ASCII"
    policy: "lowercase [a-z0-9_] for generated content; NUL padded"
  payload_compression: "none"
```

Header zawiera count i offsety localized strings, key list i resource list. Kazdy key ma `resref[16]`, `resource_id`, `resource_type`, unused. Resource list ma offset i size payloadu. Payloady sa pakowane kolejno.

Writer policy:

- odrzuc duplikat `(lowercase resref, resource type)`;
- odrzuc resref pusty, dluzszy niz 16 albo spoza generowanego charsetu;
- sortuj entries deterministycznie po `(resref, type)`, a `resource_id` nadaj po finalnym sortowaniu;
- wykonaj checked arithmetic dla kazdego rozmiaru i offsetu `u32`;
- po zapisie natychmiast wykonaj readback i porownaj SHA-256 kazdego payloadu.

## 4. 2DA V2.0

Official format rules, ktore staja sie kontraktem writera:

- pierwsza linia `2DA V2.0`, druga pusta albo `DEFAULT:`;
- wartosci sa rozdzielane spacjami; tabulatory sa zabronione;
- brak wartosci to dokladnie `****`;
- quoted string moze zawierac spacje;
- kazdy wiersz ma index i dokladnie tyle wartosci, ile jest kolumn;
- kolumn nie usuwamy, nie zmieniamy nazw i nie wstawiamy w srodku;
- nowych wierszy nie wstawiamy pomiedzy istniejace; dopisujemy je na koncu;
- usuniecie logiczne oznacza wiersz wypelniony `****`, nie fizyczne przesuniecie indeksow.

### Lokalny `appearance.2da`

Read-only resource w `lc_2da.hak`:

```yaml
appearance_reference:
  hak_signature: "HAK V1.0"
  hak_entries: 151
  resource: { resref: "appearance", type: 2017, id: 99 }
  bytes: 7655336
  sha256: "ca0b80b74e068d8ebbd94df6005b5971e50eca5c8662fca10a40688ea2c033a2"
  columns: 35
  rows: 15219
  first_index: 0
  last_index: 15218
  tabs: 0
  observed_direct_row_15216:
    LABEL: "(HD-ANIMAL) Squirrel - Brown"
    NAME: "Squirrel"
    RACE: "c_squirrel"
    MODELTYPE: "S"
    MOVERATE: "VSLOW"
    TARGETABLE: 1
```

Payloadu nie kopiujemy do repo. Zachowujemy tylko strukturalne fakty i hash.

### Algorytm dopisania appearance row

```yaml
appearance_append:
  input: "jawnie wybrany appearance.2da albo HAK zawierajacy appearance/type 2017"
  parse: "preserve header order, DEFAULT value, rows and exact **** semantics"
  validation:
    - "existing physical row count and displayed indices are internally consistent"
    - "new physical index <= 65535 because UTC Appearance_Type is uint16"
    - "required columns LABEL, NAME, RACE, MODELTYPE and MOVERATE exist"
  new_row_index: "existing physical row count; append only"
  write: "preserve every existing row; add one full-width row at EOF"
  utc: "Appearance_Type equals new physical row index"
  manifest: "record source hash, output hash, appended index and changed cells"
```

Nie wybieramy dziury po `****` i nie hard-code'ujemy `9000`. Jezeli base table ma niespojny index albo append przekracza 65535, konwersja zatrzymuje sie z diagnostyka.

## 5. GFF V3.2 oraz modul proof

Wlasny writer GFF ma obslugiwac wspolny header i tablice struct/field/label/field-data/field-indices/list-indices z little-endian checked offsets. Dla MVP wdrazamy tylko typy pol wymagane przez wygenerowany UTC, IFO, ARE/GIT; nie probujemy od razu obslugiwac wszystkich formatow gry.

```yaml
minimum_generated_proof:
  hak:
    - "appearance.2da type 2017"
    - "binary MDL type 2002 with appended MDX"
    - "own texture resource"
  utc:
    - "generated creature template"
    - "Appearance_Type = appended row"
  module_ifo:
    - "Mod_HakList list"
    - "each element contains Mod_Hak string"
  module_area:
    - "minimal generated ARE/GIT placing the generated UTC"
```

## 6. HAK precedence

Dekompilacja potwierdza serializacje `Mod_HakList` w kolejnosci. Lokalny Radoub opisuje pierwszy element jako najwyzszy priorytet, ale to jest secondary reference, nie zamkniety proof engine'u.

Dlatego pierwszy proof uzywa jednego wygenerowanego HAK. Produkt nie musi znac kolejnosci konfliktu wielu HAK, aby bezpiecznie udowodnic M6. Jesli pozniej dodamy merge wielu HAK, powstanie osobny test z dwoma zasobami o tym samym `(resref,type)` i obserwacja Toolset/game.

## 7. TDD

```yaml
gates:
  hak:
    - "deterministic bytes and exact offsets"
    - "resource readback hashes equal input hashes"
  two_da:
    - "quoted label with spaces round-trips"
    - "**** round-trips"
    - "tabs are rejected"
    - "row is appended, never inserted into a hole"
    - "append above 65535 is rejected"
  gff:
    - "own reader reads own UTC/IFO/ARE/GIT"
    - "Mod_HakList order round-trips"
    - "UTC Appearance_Type equals appended 2DA index"
  runtime:
    - "Toolset opens generated module"
    - "game resolves generated appearance row, MDL and texture"
```
