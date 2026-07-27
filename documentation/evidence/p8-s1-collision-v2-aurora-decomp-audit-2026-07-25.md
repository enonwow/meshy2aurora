# P8 S1 collision V2 — potwierdzona przyczyna braku kolizji

Data: 2026-07-25

Status: `ROOT_CAUSE_CONFIRMED / V2_OWNER_RUNTIME_NOT_BLOCKING / NO_V3_CREATED`

## 1. Werdykt

Przyczyna jest jednoznaczna dla zainstalowanego NWN:EE `89.8193.37-17`:

> Runtime nie ładuje zasobu placeable PWK parserem binarnego MDL. Funkcja
> `CNWPlaceableSurfaceMesh::LoadWalkMesh` czyta PWK jako tekst dzielony na
> linie i rozpoznaje słowa `node`, `position`, `orientation`, `verts`,
> `vertices`, `faces` oraz `endnode`. Meshy2Aurora emituje PWK jako binary MDL.
> Exact V2 nie dostarcza więc parserowi runtime ani jednej linii `verts` lub
> `faces`, a placeable otrzymuje pusty walkmesh i nie blokuje ruchu.

To nie jest już hipoteza o hierarchii, adjacency albo brakującym polu
binarnego mesha. Potwierdzają ją niezależnie:

1. dekompilacja dokładnego `nwmain.exe`;
2. dekompilacja dokładnego `nwserver.exe`;
3. pełny lokalny corpus: `9 752` zewnętrzne PWK są tekstem, `0` jest binary;
4. niezależny loader xoreos, który dla PWK używa tokenizera tekstowego;
5. bezpośredni odczyt exact V2: binarny payload nie zawiera żadnej linii
   rozpoznawalnej przez loader runtime.

Pewność: **potwierdzone dla dokładnych binariów klienta i serwera
`89.8193.37-17` używanych na tej stacji**.

## 2. Exact kandydat związany z wynikiem właściciela

1. Testowy MOD: `m2a_s1_c2_mod.mod`
2. Nazwa modułu w Toolset: `Meshy2Aurora S1 Collision V2`
3. Dokładna Area: `Meshy2Aurora M0 binary vertical-slice area`

Pozostała tożsamość:

- HAK: `m2a_s1_c2_hak.hak`;
- model i PWK resref: `m2a_s1_c2_ped`;
- obiekt: `m2a_s1_c2_obj`;
- appearance row: `16500`;
- placement: `(10.0, 14.5, 0.0)`, bearing `0.0`;
- MOD SHA-256:
  `bdc2c1fc6e1f6f9e0c93d80d3e768fd60e7fe6876b9faeabb9665e83eaaaa7de`;
- HAK SHA-256:
  `25c3f064d6b040cce946adf51092478bc6e239139ccbb4a962249d2e406e8440`;
- PWK SHA-256:
  `4eef6b278925845ebe6ee17646aa0d647a38fbaa56c7e0cce6252950c2e4e6d2`.

Ponowny odczyt potwierdził byte-identical MOD i HAK w natywnych katalogach
NWN. Audyt niczego tam nie zapisał ani nie podmienił.

## 3. Dowód bezpośredni z `nwmain.exe`

Audytowany plik:

| Pole | Wartość |
|---|---|
| ścieżka | `C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\bin\win32\nwmain.exe` |
| FileVersion/ProductVersion | `89.8193.37-17` |
| rozmiar | `21 071 872` bajty |
| SHA-256 | `3b7cb1252e0edb2ce22d7971f333aade027039ae30a45b4bc64732c3e6bec73a` |
| PE image base | `0x140000000` |
| liczba eksportowanych nazw | `18 245` |

Istotne eksporty:

| Symbol | RVA |
|---|---:|
| `CResPWK::GetPWKDataPtr` | `0x2CB720` |
| `CResPWK::GetPWKSize` | `0x2CB730` |
| `CNWPlaceMeshManager::GetWalkMesh` | `0x2CD9B0` |
| `CNWPlaceMeshManager::InitializeWalkMeshes` | `0x2CDC70` |
| `CNWPlaceableSurfaceMesh::LoadWalkMesh` | `0x2CE0B0` |
| `CNWPlaceableSurfaceMesh::LoadWalkMeshString` | `0x2CEAF0` |

### 3.1. Wiązanie z placeable walkmeshem

`CNWPlaceMeshManager::GetWalkMesh` ma jedyne bezpośrednie wywołanie
`CNWPlaceableSurfaceMesh::LoadWalkMesh` pod VA `0x1402CDC04`. W
`LoadWalkMesh`:

- pod `0x1402CE1C8` ustawiany jest Resource Type `0x805`, czyli `2053`/PWK;
- pod `0x1402CE252` pobierany jest wskaźnik danych `CResPWK`;
- pod `0x1402CE25E` pobierany jest rozmiar `CResPWK`.

To jest dokładna ścieżka zasobu kolizji placeable, a nie ścieżka viewera MDL.

### 3.2. Parser jest tekstowy

Pętla od `0x1402CE2C0`:

- kopiuje kolejne bajty do bufora o pojemności `256`;
- kończy rekord na `LF` (`0x0A`) albo granicy bufora;
- dopisuje `NUL`;
- pomija początkowe spacje i tabulatory;
- wykonuje porównania tekstowe.

Wyeksportowane `LoadWalkMeshString` pod `0x1402CEAF0` ma ten sam kontrakt:
kopiuje bajty, sprawdza `0x0A`, dopisuje `NUL` i nie sprawdza nagłówka binary
MDL.

Literały odwoływane bezpośrednio przez `LoadWalkMesh`:

```text
node
 trimesh
 dummy
pwk_use
pwk_dp_use_
01
02
position
orientation
verts
vertices
faces
endnode
%f %f %f
%f %f %f %f
%d %d %d %d %d %d %d %d
```

W funkcji znajdują się również:

```text
game\sharedcore\nwplaceablesurfacemesh.cpp
LoadWalkMesh
Walkmesh with double faces tag: %s
Walkmesh with double vertices tag: %s
```

Nie ma:

- warunku „pierwsze cztery bajty są zerem”;
- odczytu nagłówka binary MDL;
- wywołania binarnego parsera node/mesh;
- alternatywnej drogi binary PWK.

Funkcja może zakończyć skanowanie powodzeniem bez utworzenia `verts` i
`faces`. To wyjaśnia cichy objaw: brak błędu zasobu, ale również brak
przeszkody.

## 4. Niezależne potwierdzenie w `nwserver.exe`

Audytowany plik:

| Pole | Wartość |
|---|---|
| ścieżka | `C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\bin\win32\nwserver.exe` |
| FileVersion/ProductVersion | `89.8193.37-17` |
| rozmiar | `12 586 496` bajtów |
| SHA-256 | `98951511ae7d06a251355f12d4f3e4b96269a2414fa3058e641afd70098690f6` |
| `LoadWalkMesh` RVA | `0x261730` |
| `LoadWalkMeshString` RVA | `0x262170` |

Serwer ma tę samą:

- ścieżkę `CResPWK`;
- pętlę tekstową;
- listę tokenów;
- ścieżkę źródłową `nwplaceablesurfacemesh.cpp`;
- nieobecność gałęzi binary PWK.

To wyklucza wyjaśnienie, że tylko klient lub tylko single-player używa
tekstowego parsera. Klient i dedicated server w tej samej wersji zgadzają się
co do kontraktu.

## 5. Co dokładnie dzieje się z V2

Exact V2 PWK:

| Pole | Wartość |
|---|---|
| rozmiar | `1 272` bajty |
| SHA-256 | `4eef6b278925845ebe6ee17646aa0d647a38fbaa56c7e0cce6252950c2e4e6d2` |
| pierwsze 32 bajty | `00 00 00 00 50 04 00 00 9c 00 00 00 00 00 00 00 00 00 00 00 6d 32 61 5f 73 31 5f 63 32 5f 70 65` |
| liczba `LF` | `0` |

Loader runtime dzieli ten payload na rekordy po maksymalnie `256` bajtów.
Początki C-stringów widzianych przez porównania tekstowe:

| Offset payloadu | Początek C-stringa |
|---:|---|
| `0` | pusty — pierwszy bajt to `NUL` |
| `256` | pusty |
| `512` | `73 E2 EB 3E` (`s` + dane binarne), bez tokenu |
| `768` | pusty |
| `1024` | pusty |

Liczba rozpoznanych rekordów `node`/`position`/`orientation`/`verts`/`faces`/
`endnode`: **0**.

Zatem dokładny przebieg jest następujący:

```text
(m2a_s1_c2_ped, resource type 2053)
  -> CResPWK
  -> CNWPlaceableSurfaceMesh::LoadWalkMesh
  -> tekstowe rekordy po LF / 256 bajtów
  -> 0 rozpoznanych node
  -> 0 verts
  -> 0 faces
  -> brak przeszkody
```

## 6. Pełny lokalny corpus

Skan wykonano read-only, bez kopiowania payloadów do repo.

### 6.1. Retail KEY/BIF

| Źródło | PWK | Tekst ASCII | Binary |
|---|---:|---:|---:|
| `nwn_base.key` i wskazane BIF-y | `1 090` | `1 090` | `0` |
| `nwn_retail.key` | `0` | `0` | `0` |

Spośród `1 090` zasobów retail:

- `1 076` zawiera tokeny geometrii;
- `14` to tekstowe, komentarzowe PWK bez geometrii;
- `0` zaczyna się od nagłówka binary MDL.

### 6.2. Natywne HAK-i

Przeskanowano `148` plików HAK. Czternaście zawiera PWK.

| HAK | PWK zewnętrzne ASCII | PWK Meshy2Aurora binary |
|---|---:|---:|
| `cep3_core0.hak` | `2 550` | `0` |
| `cep3_core1.hak` | `974` | `0` |
| `cep3_core2.hak` | `1 849` | `0` |
| `cep3_facelift.hak` | `1` | `0` |
| `lc_ccc.hak` | `20` | `0` |
| `lc_placeable.hak` | `992` | `0` |
| `lc_slms_city.hak` | `30` | `0` |
| `nwic_placeables.hak` | `410` | `0` |
| `q_placeables.hak` | `1 049` | `0` |
| `vale-c_and_m.hak` | `41` | `0` |
| `witcher.hak` | `687` | `0` |
| `witcher2.hak` | `59` | `0` |
| `m2a_s1_col_hak.hak` | `0` | `1` |
| `m2a_s1_c2_hak.hak` | `0` | `1` |

Łącznie:

```text
retail + zewnętrzne HAK-i = 9 752 ASCII PWK, 0 binary PWK
Meshy2Aurora              = 0 ASCII PWK, 2 binary PWK
```

Tylko dwa niedziałające artefakty Meshy2Aurora są binarne w całym
przeskanowanym środowisku.

## 7. Niezależne implementacje i źródła

### 7.1. xoreos

Pinned commit:
[`89c99d2a93c23f3ba2b1218759e38775e4f2bdf9`](https://github.com/xoreos/xoreos/blob/89c99d2a93c23f3ba2b1218759e38775e4f2bdf9/src/engines/nwn/walkmeshloader.cpp).

`WalkmeshLoader::load`:

- pobiera zasób wybranego typu, w tym `kFileTypePWK`;
- tworzy `StreamTokenizer`;
- ustawia separator spacji, koniec rekordu `\n` i ignoruje `\r`;
- parsuje `node`, `position`, `orientation`, `verts`, `faces`, `aabb`,
  `endnode`;
- nie ma drogi binary PWK.

`objectwalkmesh.cpp` wywołuje ten loader z `kFileTypePWK`, wiążąc parser z
kolizją obiektu. xoreos nie jest kodem Beamdog, ale jest niezależną
reimplementacją i dokładnie zgadza się z dekompilacją obu binariów runtime.

### 7.2. NwnExplorer — dlaczego wcześniejszy wniosek był błędny

Pinned commit:
[`56da6dc2fe94da6bbabe83ad18670f47fccd7dfb`](https://github.com/dunahan/nwnexplorer/blob/56da6dc2fe94da6bbabe83ad18670f47fccd7dfb/nwnexplorer/ModelView.cpp).

`ModelView.cpp`:

- kompiluje ASCII PWK do modelu pomocniczego dla własnego widoku;
- payload zaczynający się od zera przekazuje bezpośrednio do własnego model
  viewera.

To dowodzi zdolności **viewera** do pokazania binary PWK. Nie dowodzi
zdolności `CNWPlaceableSurfaceMesh` do zbudowania z niego kolizji. Dokładna
dekompilacja runtime rozstrzyga tę różnicę.

### 7.3. Dokumentacja formatu

Repozytorium
[`xoreos-docs`](https://github.com/xoreos/xoreos-docs/tree/4e1c197aa09b532ef466ff8ceccfd6221e80c3c9)
identyfikuje katalog `specs/bioware` jako archiwum oficjalnych dokumentów
BioWare. Zawiera specyfikacje GFF placeable, Area, IFO, Palette ITP i KEY/BIF,
ale nie zawiera osobnej specyfikacji serializacji PWK. Dlatego wcześniejsze
traktowanie binarnego layoutu MDL jako kontraktu PWK nie miało oparcia w
oficjalnym dokumencie.

## 8. Powiązanie przyczyny z naszym kodem

Implementacja w chwili audytu V2:

- `crates/m2a-core/src/placeable_collision.rs:1` opisuje PWK jako warstwę nad
  binary MDL writerem;
- `write_placeable_walkmesh_v1` wywołuje `write_binary_mdl` w okolicy linii
  `147`;
- test pipeline odczytuje PWK przez `inspect_binary_mdl`;
- materializer również wymaga binary-MDL readback.

Własny readback potwierdzał więc zgodność z własnym writerem, ale nie z
faktycznym parserem `CNWPlaceableSurfaceMesh`.

Aktualizacja po implementacji V3: powyższa lista opisuje historyczne V1/V2.
`placeable_collision.rs` zawiera teraz `PlaceableWalkmeshIrV1`, ASCII writer i
runtime-aligned reader. Package gate ponownie parsuje gotowy zasób `2053` i
odrzuca binary PWK. Exact V3 opisuje osobny handoff
`p8-s1-collision-v3-ascii-pwk-ready-for-owner-proof-2026-07-25.md`.

Właściwa korekta architektury nie rozdziela geometrii na dwa niezależne
pipeline'y. Wspólne pozostają:

```text
GLB/Meshy
  -> wspólny import
  -> AuroraModelIrV1
  -> wspólne transformacje i world-space bounds
  -> PlaceableWalkmeshIrV1
```

Dopiero format wyjściowy rozgałęzia się zgodnie z parserem docelowym:

```text
render MDL -> wspólny binary MDL writer/readback -> resource type 2002
PWK        -> PWK ASCII writer/readback          -> resource type 2053
```

ASCII PWK nie jest „ASCII render MDL”. Jest osobnym zasobem pomocniczym
ładowanym przez osobny, tekstowy parser runtime.

## 9. Odrzucone przyczyny

| Hipoteza | Werdykt | Powód |
|---|---|---|
| PWK nie znajduje się w HAK | odrzucona | type `2053`, exact resref i hash potwierdzone |
| MDL i PWK mają różne resrefy | odrzucona | oba `m2a_s1_c2_ped` |
| zły `placeables.2da ModelName` | odrzucona | row `16500` wskazuje exact resref |
| placeable nie jest statyczny | odrzucona | UTP/GIT/2DA: `Static=1`, `Useable=0` |
| zły surface | odrzucona | numeric `7` to `Nonwalk` |
| zły footprint lub winding | odrzucona | 4 vertices, 2 faces, dodatnie bounds, normalne `+Z` |
| brak adjacency | odrzucona | V2 ma poprawne adjacency i nadal nie blokuje |
| zła binarna hierarchia rootów | nieistotna dla tego objawu | runtime nie parsuje binarnych node'ów |
| `m_fFaceNormalSumDiv2 = 0` | nieistotna dla tego objawu | runtime nie odczytuje binarnego pola `0x268` |
| zły typ klasyfikacji binary modelu | nieistotna dla tego objawu | runtime nie odczytuje binarnego nagłówka modelu |
| podmieniony plik natywny | odrzucona | MOD/HAK source→destination są byte-identical |

Różnice hierarchy/`0x268` pozostają prawdziwymi obserwacjami formatu
binarnego, ale nie są przyczyną braku kolizji V2.

## 10. Minimalny następny krok — wykonany po audycie

W samym audycie nie powstał V3. Późniejsza implementacja wykonała następujący
zakres:

- [x] pozostawić render MDL w obecnym wspólnym binary MDL pipeline;
- [x] wyprowadzić footprint z tego samego `AuroraModelIrV1`;
- [x] dodać osobny, deterministyczny `write_ascii_placeable_walkmesh_v1`;
- [x] emitować tekstowe `node trimesh`, `parent`, `position`, `orientation`,
  `verts`, `faces`, `endnode`;
- [x] zapisywać po osiem pól na face:
  `v0 v1 v2 smoothGroup adj0 adj1 adj2 surface`;
- [x] zachować surface `7` (`Nonwalk`) i retailowe zera w trzech polach
  kompatybilności, których exact runtime nie przechowuje;
- [x] dodać parser/readback dokładnie tej gramatyki ASCII;
- [x] dodać test negatywny odrzucający binary PWK w runtime-ready package;
- [x] dodać golden oparty na samodzielnie utworzonej geometrii i potwierdzonej
  gramatyce, bez kopiowania retail payloadu do produktu;
- [x] nie zmieniać Resource Type `2053`, `Static=1`, `Useable=0`, appearance
  row ani placementu; V3 otrzymał świeży, związany z iteracją zestaw resrefów;
- [x] dopiero po gate'ach offline utworzyć jeden exact kandydat V3;
- [x] przekazać V3 właścicielowi do testu zgodnie z human-owned proof.

Nie należy tworzyć `PwkBinarySemanticProfile`. Taka iteracja powtórzyłaby
format, którego dokładny runtime w tej ścieżce nie parsuje.

## 11. Zamknięcie audytu

- [x] Związano raport właściciela z exact V2.
- [x] Sprawdzono Toolsetową ścieżkę resolvera i pola UTP/GIT.
- [x] Zdekompilowano exact `nwmain.exe`.
- [x] Zdekompilowano exact `nwserver.exe`.
- [x] Związano `GetWalkMesh` z `LoadWalkMesh`.
- [x] Potwierdzono Resource Type `0x805`/`2053`.
- [x] Potwierdzono tekstowy parser i brak gałęzi binary PWK.
- [x] Zasymulowano parser na exact V2: zero rozpoznanych rekordów.
- [x] Przeskanowano pełny lokalny corpus: `9 752` zewnętrzne ASCII, `0`
  zewnętrznych binary PWK.
- [x] Porównano niezależny loader xoreos.
- [x] Skorygowano interpretację NwnExplorera.
- [x] Odrzucono hierarchy/`0x268` jako przyczynę tego failure.
- [x] Zapisano minimalny wspólny pipeline z PWK ASCII serializerem.
- [x] W samym audycie nie utworzono ani nie zainstalowano kolejnej iteracji.
- [x] Zaimplementowano PWK ASCII writer/readback po zamknięciu audytu.
- [x] Dopiero potem utworzono jeden exact kandydat V3.
- [ ] Właściciel wykonuje finalny test kolizji V3 w NWN.

Aktualne osie:

```text
V2 collisionRuntimeVerdict = not_blocking
V2 collisionProofCompleteness = verified_by_owner_report
V2 modelVisibility = not_tested
auditStatus = root_cause_confirmed
rootCause = binary_PWK_sent_to_ASCII_only_placeable_walkmesh_loader
V3 candidateStatus = ready_for_owner_proof
V3 collisionRuntimeVerdict = not_tested
```
