# M1C - kontrakt read-only ERF/HAK V1.0

Data: 2026-07-11 | Status: AKTYWNY SUPLEMENT IMPLEMENTACYJNY, NIE JEST DOWODEM `DONE`

## 1. Cel i granica etapu

M1C dostarcza wlasny, bezpieczny locator zasobu w kontenerze ERF/HAK V1.0. Locator przyjmuje bajty jawnie wybranego pliku, odnajduje zasob po `(resref, resource_type)` i przekazuje jego read-only slice do M1B. Nie wypakowuje zasobu do repo ani na dysk.

W zakresie M1C sa:

- sygnatury `ERF ` i `HAK ` oraz wersja `V1.0`;
- header, key list i resource list ERF V1.0;
- case-insensitive lookup resrefu i exact lookup typu;
- bounds checks, checked arithmetic, limity i stabilne diagnostyki;
- syntetyczne fixture generowane w testach;
- opcjonalny, env-gated odczyt lokalnego CEP in-place.

Poza zakresem M1C sa writer HAK, `MOD `, `NWM `, `V1.1`, KEY/BIF, localized-string interpretation, kompresja, merge i precedence wielu HAK oraz kopiowanie retail/CEP payloadow.

## 2. Hierarchia dowodow

1. Aurora First:
   - `C:\Projects\New Folder\export\decompiled_all.c:8477` - literal `HAK V1.0`;
   - `C:\Projects\New Folder\export\decompiled_all.c:122308-122313` - porownanie pierwszych osmiu bajtow z `HAK V1.0`;
   - `C:\Projects\New Folder\export\decompiled_all.c:15430` - literal `ERF V1.0`.
2. Oficjalna specyfikacja formatu:
   - `C:\Projects\Claude\Radoub\Documentation\BioWare_Original_PDFs\Bioware_Aurora_ERF_Format.pdf`.
   - strony 1-2: header ma 160 bajtow i pola opisane w sekcji 3.1;
   - strona 3: key ma 24 bajty, dokument opisuje stored resref jako lowercase `[a-z0-9_]`, a `ResID` jako sekwencyjny i rowny indeksowi key;
   - strona 4: resource entry ma 8 bajtow i pozostaje w relacji one-to-one z key.
3. Aktywne dokumenty repo:
   - `documentation/hak-2da-gff-crosswalk-codex.md:27-51`;
   - `documentation/aurora-hak-erf-codex.md:44-99`;
   - `documentation/plan-implementacji-orkiestrator-codex.md:219-245`.
4. Lokalny CEP jest wylacznie read-only reference przez `M2A_REFERENCE_CEP_HAK`; kod i payloady projektow pomocniczych nie sa dependency ani fixture source.

Dekompilacja potwierdza rozpoznawanie kontenera przez Aurorę, ale nie wystarcza do udowodnienia calego layoutu tabel. Oficjalny PDF domyka layout tabel, natomiast canonical shipped CEP moze ujawnic szersze zachowanie realnych danych. Przed finalnym review M1C reviewer musi zapisac w evidence zgodnosc kazdego pola header/key/resource z numerem strony lub sekcji oraz kazde jawne runtime override. Samo powolanie sie na plik PDF nie zalicza gate.

## 3. REQUIRED

### 3.1 Layout

Wszystkie liczby sa little-endian.

| Offset | Pole | Typ/rozmiar |
|---:|---|---:|
| `0x00` | file type | 4 bytes, exact `ERF ` albo `HAK ` |
| `0x04` | version | 4 bytes, exact `V1.0` |
| `0x08` | language count | `u32` |
| `0x0c` | localized string size | `u32` |
| `0x10` | entry count | `u32` |
| `0x14` | offset to localized strings | `u32` |
| `0x18` | offset to key list | `u32` |
| `0x1c` | offset to resource list | `u32` |
| `0x20` | build year | `u32` |
| `0x24` | build day | `u32` |
| `0x28` | description string ref | `u32` |
| `0x2c` | reserved | 116 bytes; header konczy sie na `0xa0` |

Key V1.0 ma 24 bajty: `resref[16]`, `resource_id u32`, `resource_type u16`, `unused u16`. Resource entry ma 8 bajtow: `payload_offset u32`, `payload_size u32`.

`resource_id` jest sekwencyjny, 0-based i musi byc rowny pozycji key. Key index `i` pozostaje w relacji one-to-one z resource entry `i`; resource list ma dokladnie `entry_count` wpisow i nie zawiera osobnego ID. In-range, ale przestawiony lub powtorzony `resource_id` jest bledem strukturalnym, nie aliasem.

### 3.2 Resref i lookup

- Stored resref jest ASCII o szerokosci 16 bajtow. PDF na stronie 3 opisuje lowercase `[a-z0-9_]`, ale zamkniety canonical alphabet audit shipped `cep3_core1.hak` potwierdza uppercase oraz `-`. Produkcyjny reader akceptuje zatem `[A-Za-z0-9_-]` jako jawny spec-vs-shipped override; inne punctuation, whitespace i control bytes odrzuca.
- Pelne 16 znakow bez NUL jest legalne.
- Dla krotszej nazwy pierwszy NUL konczy resref, a pozostale bajty paddingu musza byc NUL.
- Query musi byc niepuste, miec najwyzej 16 bajtow i nalezec do `[A-Za-z0-9_-]`.
- Porownanie resref jest ASCII case-insensitive; `resource_type` jest porownywany dokladnie.
- Ten sam resref z roznymi typami jest legalny.
- Dwa keys o tym samym `(ASCII-lowercase resref, type)` sa odrzucane jako niejednoznaczne. M1C nie zgaduje first/last-wins bez engine proof.

### 3.3 Bezpieczenstwo strukturalne

- Kazde `count * entry_size` i `offset + size` uzywa checked arithmetic przed slice lub alokacja.
- Reader ma publiczny `ErfLimits { max_entry_count }` stosowany przed parser-owned allocation. `ErfLimits::default()` ustawia `max_entry_count = 262_144`; zwykle `parse(bytes)` uzywa tego defaultu, a `parse_with_limits(bytes, limits)` pozwala hostowi ustawic nizszy limit. Przekroczenie limitu zwraca stabilny blad przed alokacja.
- Header, localized-string region, key list, resource list i payloady musza miescic sie w pliku.
- Niepuste regiony metadata nie moga nachodzic na header ani na siebie.
- Niepusty payload nie moze nachodzic na header/metadata ani na inny niepusty payload.
- Kazdy `resource_id` musi byc dokladnie rowny indeksowi key, nie tylko mniejszy od `entry_count`.
- Payload `size == 0` jest istniejacym zasobem i zwraca pusty slice; jego offset musi byc `<= file_len`. W szczegolnosci `offset == EOF` jest wymaganym przypadkiem pozytywnym, a `offset == EOF + 1` wymaganym przypadkiem negatywnym.
- Dowolny prefix poprawnego pliku i dowolne male losowe bytes nie moga wywolac panic.

### 3.4 Stabilne diagnostyki

Publiczny blad ma co najmniej `schemaVersion`, stabilny `code`, `offset` i krotki `context`. Dla aktualnego kontraktu wymagane sa rozroznialne kody:

- `M2A-ERF-SIGNATURE-UNSUPPORTED`;
- `M2A-ERF-VERSION-UNSUPPORTED`;
- `M2A-ERF-HEADER-OOB`;
- `M2A-ERF-KEY-TABLE-OOB`;
- `M2A-ERF-RESOURCE-TABLE-OOB`;
- `M2A-ERF-PAYLOAD-OOB`;
- `M2A-ERF-DUPLICATE-KEY`;
- `M2A-ERF-RESOURCE-ID-INVALID`;
- `M2A-ERF-RESREF-INVALID`;
- `M2A-ERF-RESOURCE-MISSING`.

Kody OOB moga rowniez opisywac arithmetic overflow lub overlap w `context`, ale test musi ustalac kod i reprezentatywny offset. Zmiana kodu jest zmiana publicznego kontraktu.

### 3.5 Public API

Rdzen Rust powinien utrzymywac validated read-only view pozyczajacy bajty kontenera i zwracac `&[u8]` bez filesystem side effect. Metadata zasobu powinna zachowywac canonical resref, typ, resource ID, offset i size. Adapter integracyjny M1B przekazuje znaleziony type-2002 slice bezposrednio do istniejacego parsera binary MDL.

WASM moze zwracac owned bytes lub raport wymagany przez UI, ale musi zachowac te same stabilne kody. M1C nie eksportuje funkcji zapisujacej extracted payload na dysk.

## 4. DEFERRED

- interpretacja language IDs i localized strings;
- semantyka build year/day i description string ref;
- `V1.1`, 32-byte resrefs, `MOD ` i `NWM `;
- writer ERF/HAK, kompresja i deterministyczne pakowanie;
- precedence oraz kolizje pomiedzy wieloma HAK;
- KEY/BIF i base-retail locator;
- runtime/game acceptance, ktora nalezy do M5/M6.

## 5. OPEN i stop conditions

- `OPEN-PDF-EVIDENCE`: layout zostal porownany z PDF s.1-4, ale finalne evidence nadal musi zawierac page/field mapping oraz wynik review implementacji. Do tego czasu nie wolno deklarowac finalnego review ani `DONE`.
- `OPEN-DUPLICATE-ENGINE`: Aurora First nie zamyka first/last-wins dla duplikatow case-insensitive w jednym kontenerze. Reader bezpiecznie je odrzuca; inna polityka wymaga osobnego engine proof.
- `OPEN-ZERO-LENGTH-ENGINE`: strukturalna polityka M1C akceptuje zero-length resource. Twierdzenie, ze Toolset/gra konsumuje taki zasob, pozostaje poza M1C.
- `OPEN-CEP`: canonical CEP lookup musi zostac wykonany przez wlasny reader in-place albo jawnie oznaczony jako clean skip, jesli env nie ustawiono. Nie wolno kopiowac HAK ani MDL do repo.

Stop: jezeli PDF przeczy ktoremukolwiek polu lub lokalny HAK wymaga luzniejszej polityki strukturalnej, zatrzymac merge i zapisac minimalny repro/evidence. Nie dopasowywac parsera przez zgadywanie.

### Zamkniety canonical alphabet audit

Read-only scan `cep3_core1.hak` zamyka alfabet resrefow dla canonical M1C reference:

```yaml
canonical_cep_alphabet_audit:
  total_keys: 6402
  type_2002_keys: 3517
  extra_character_beyond_pdf_lowercase_set: "-"
  hyphen:
    all_keys_resrefs: 66
    type_2002_resrefs: 36
  uppercase:
    resrefs: 47
    characters: 145
  empty_resrefs: 0
  bad_nul_padding: 0
  full_width_16_byte_resrefs: 81
  accepted_stored_and_query_alphabet: "[A-Za-z0-9_-]"
  lookup: "ASCII case-insensitive"
```

PDF lowercase `[a-z0-9_]` pozostaje regula specyfikacji, a `[A-Za-z0-9_-]` jest waskim runtime override udowodnionym przez shipped canonical corpus. Audit alfabetu jest zamkniety; nie oznacza to zamkniecia osobnej polityki duplicate resolution.

## 6. Obowiazkowa macierz negatywna

| Przypadek | Wymagany wynik |
|---|---|
| input krotszy niz 160 bajtow | `M2A-ERF-HEADER-OOB` |
| nieobslugiwana sygnatura | `M2A-ERF-SIGNATURE-UNSUPPORTED` |
| `HAK V1.1` | `M2A-ERF-VERSION-UNSUPPORTED` |
| key table albo resource table poza EOF | odpowiedni table OOB |
| mnozenie count lub dodanie offset+size overflow | stabilny odpowiedni OOB, bez panic |
| `resource_id >= entry_count` | `M2A-ERF-RESOURCE-ID-INVALID` |
| in-range `resource_id != key index`, permutacja albo powtorzenie | `M2A-ERF-RESOURCE-ID-INVALID` |
| payload poza EOF | `M2A-ERF-PAYLOAD-OOB` |
| metadata overlap | stabilny odpowiedni table OOB |
| payload nachodzi na metadata lub inny payload | `M2A-ERF-PAYLOAD-OOB` |
| stored punctuation poza `_-`, whitespace/control/non-ASCII albo dane po pierwszym NUL | `M2A-ERF-RESREF-INVALID` |
| stored uppercase `[A-Z]` z canonical CEP | sukces; case-insensitive lookup |
| stored/query z `-` | sukces; case-insensitive lookup |
| query pusty, spoza `[A-Za-z0-9_-]` lub dluzszy niz 16 | `M2A-ERF-RESREF-INVALID` |
| `Foo` i `foo` tego samego typu | `M2A-ERF-DUPLICATE-KEY` |
| ten sam resref, rozne typy | oba lookupy poprawne |
| key/resource count one-to-one i `resource_id == key index` | sukces |
| dokladnie 16 znakow bez NUL | sukces |
| zero-length payload z offsetem rownym EOF | sukces i pusty slice |
| zero-length payload z offsetem rownym EOF + 1 | `M2A-ERF-PAYLOAD-OOB` |
| brak `(resref,type)` | `M2A-ERF-RESOURCE-MISSING` |
| kazdy prefix poprawnego fixture | wynik lub stabilny blad, nigdy panic |

## 7. Definition of Done supplement

M1C nie jest `DONE`, dopoki evidence nie zawiera lacznie:

1. finalnego PDF cross-checku z exact page/section mapping;
2. zielonej syntetycznej macierzy pozytywnej i negatywnej;
3. stable missing-resource assertion;
4. clean env-missing skip;
5. canonical `M2A_REFERENCE_CEP_HAK` lookup dla co najmniej R1 `c_kocrachn` type 2002 oraz record hash/size/ID/offset bez payloadu;
6. bezposredniego przekazania znalezionego slice do M1B i canonical P-REF handback;
7. `fmt`, `clippy`, native tests, wymaganej granicy WASM oraz `git diff --check`;
8. audytu Git potwierdzajacego brak retail/CEP payloadow i prywatnych host paths w committable evidence.
