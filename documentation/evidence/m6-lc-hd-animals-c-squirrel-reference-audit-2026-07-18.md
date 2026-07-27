# Last City c_squirrel: audit referencyjny (2026-07-18)

## Zakres i kontrola kopii

Wybrano `c_squirrel` z `lc_hd_animals.hak`: 109 modeli type `2002` w tym HAK-u
jest ASCII MDL, wiec ten witness opisuje natywna semantyke autora, a nie
binarny layout writera. Model ma resource id `285`, `801374` B i SHA-256
`e381bd13f4f10ab9c33a9751f409c061b6579e2a042d12170d74e23f5b669767`.

Na wyrazna prosbe ownera utworzono tylko kopie do
`proof-output/lc-hd-animals-c-squirrel-reference-audit-2026-07-18/source-copies`.
Manifest zawiera source container, resource id, offset, rozmiar i hash kazdego
pliku. Nie zmieniono instalacji Last City, retail NWN, HAK-a, MOD-a ani
aktualnego proofu H1.

`c_squirrel` ma dwa bitmap resrefs: `c_squirrel` oraz `c_badger`. Pierwszy
jest DDS z `lc_hd_animals.hak`; drugi jest retailowym TGA/DDS z
`nwn_base.key` (KEY SHA-256
`09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935`).
Dlatego obie retailowe warianty `c_badger` sa skopiowane wyraznie, zamiast
ukrywac zewnetrzna zaleznosc referencji.

## appearance.2da

Skopiowana tabela pochodzi z `lc_2da.hak`, resource id `99`, type `2017`,
SHA-256 `ca0b80b74e068d8ebbd94df6005b5971e50eca5c8662fca10a40688ea2c033a2`.
Wiersz `15216` ma label `(HD-ANIMAL) Squirrel - Brown`, `RACE=c_squirrel`,
`MODELTYPE=S` i `MOVERATE=FAST`. Nie ujawnia dodatkowej custom 2DA koniecznej
do wyboru MDL: lookup pozostaje `appearance -> RACE -> c_squirrel.mdl`.

## Fakty z ASCII modelu LC

| Wlasciwosc | c_squirrel Last City |
| --- | --- |
| model / supermodel | `c_squirrel` / `NULL` |
| classification / animation scale | `Character` / `0.5` |
| nody bazowe | `30`: 6 `dummy`, 21 `trimesh`, 1 `danglymesh`, 2 `skin` |
| skinmeshe | `sqrlfront` (294 vertices), `sqrlback` (163 vertices) |
| helper dummies | `impact`, `head`, `handconjure`, `headconjure` oraz root `c_squirrel`, `Bdger_rootdummy` |
| animacje | 43 roznych nazw; wszystkie `animroot c_squirrel` |
| bitmapy | `c_squirrel`, `c_badger` |

Kluczowy odczyt weight lines obu skinmeshow: influence wskazuja na
`Badger_*` nody typu `trimesh` (np. `Badger_thorax`,
`Badger_Lfrontbicep`, `Badger_shoulders`, `Badger_head`), a nie na helper
`dummy`. Te `trimesh` maja `render 0` i pelnia role pseudo-kosci.

Read-only kontrola binarnego direct creature `c_vampire_f` z lokalnego CEP
potwierdza, ze ten wzorzec nie jest wyjatkiem tylko ASCII: jego 28 reachable
nodow to 17 `trimesh`, 9 helper `dummy` i 2 `skin`; ma m.in. `Impact`,
`head`, `handconjure` i `headconjure`.

## Porownanie z H1 v19

| Wlasciwosc | LC c_squirrel | H1 v19 |
| --- | --- | --- |
| base tree | pseudo-kosci `trimesh` plus helper dummies | 24 nody `dummy/header` plus jeden `skin` |
| skinmeshe | 2 | 1 (`1334` vertices) |
| aktywne influence skina | `trimesh` pseudo-kosci | wylacznie nody `dummy/header`, sloty `0..21` |
| helpers | `impact`, `head`, `handconjure`, `headconjure` | tylko glTF-derived joints; brak `impact`, `handconjure`, `headconjure` |
| animacje | 43 roznych klipow | 7 aliasow jednego klipu, 343 tracki, zero eventow |
| supermodel | `NULL` | `NULL` |

## Werdykt

**Potwierdzone rozjazdy:** H1 nie odtwarza pseudo-bone topology LC, helper
dummies ani indywidualnej semantyki 43 klipow. Najwazniejsza nowa hipoteza
dotyczy nie samej liczby kosci, lecz typu nodu, do ktorego trafia weight:
LC wiąze skin z niewidzialnym `trimesh`, a H1 z `dummy/header`.

### Kontrola w dekompilacji Aurora

`FUN_00a932cc` jest bezposrednia funkcja budujaca palete skina. Dla kazdego z
64 wpisow inline mappingu sprawdza tylko, czy ordynal jest nieujemny,
mniejszy od `param_2[1]` oraz od liczby q/t inverse-bind. Nie odczytuje flag
nodu (`dummy`, `trimesh`, `skin`) ani nie rozgalezia sie po node family.

To **odrzuca twierdzenie**, ze sama zamiana `dummy` na `trimesh` jest
potwierdzonym wymaganiem koncowego etapu skinowania. Pozostaje otwarte, czy
wczesniejsze budowanie tablicy transformow zasila dummy inaczej niz
pseudo-bone. Wymaga to sledzenia danych wejsciowych `param_2` do tej funkcji,
nie zgadywania formatu nodu.

**Czego nie wolno jeszcze twierdzic:** dwie warstwy skin, `danglymesh`, helper
dummies i 43 klipy sa roznicami referencyjnymi, ale nie sa jeszcze pojedynczo
udowodniona przyczyna niewidocznosci.

**Nastepny test Aurora First:** sledzic pochodzenie i wypelnienie tablicy
transformow przekazywanej jako `param_2` do `FUN_00a932cc`, a nastepnie
porownac jej base-pose i pierwszy frame H1 z LC. Dopiero wynik tego audytu
moze uzasadnic A/B non-rendered `trimesh`. Nie kopiowac geometrii, skeletonu,
wag ani animacji Last City do produktu.
