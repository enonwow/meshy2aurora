# Porownanie `m2a_test.mod` z wygenerowanym proof MOD (2026-07-18)

Status: `OTWARTE — modul generatora wymaga zblizenia do Toolsetu; przyczyna
braku modelu w runtime nie zostala przez to porownanie znaleziona.`

## Objaw i zakres

Model z HAK-a `m2a_codex_aproof.hak` jest widoczny w Toolsecie/Aurorze, ale nie
pojawia sie w grze. Wlasciciel utworzyl recznie
`C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_test.mod` jako
kontrolny MOD i poprosil o porownanie go z automatycznie wygenerowanym
`m2a_codex_aproof.mod`. Porownanie dotyczy kontenera MOD, `module.ifo` i
trojki ARE/GIC/GIT; nie przypisuje roznic w samym stworze ani pozycji startowej
jako wyjasnienia.

## Badane artefakty

| Artefakt | SHA-256 |
| --- | --- |
| `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_codex_aproof.mod` | `751959179ec0003d361a80508738c36fdc9fe255ff21f4b82e1fbb7c2f53e781` |
| `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_test.mod` | `16f3d5dabd494a6ad70d1fe513dc65a481449649533e28cdf2073bfc5d6ba9a` |

Oba MOD-y zostaly odczytane in-place wlasnym `m2a-core` przez
`examples/inspect_module.rs`; nie byly zapisywane ani modyfikowane.

## Fakty

1. Oba `module.ifo` maja dokladnie jeden wpis
   `Mod_HakList[0].Mod_Hak` typu CExoString o wartosci
   `m2a_codex_aproof`. To jest ten sam HAK, ktory zawiera
   `appearance.2da`, `m2a_m6p01.mdl` i `m2a_m6t01.tga`.
2. Wygenerowany MOD zawiera: IFO, FAC (`repute`), ARE, GIC, GIT i UTC.
   Reczny MOD zawiera IFO, FAC, ARE, GIC, GIT oraz palety Toolsetu. Palety nie
   dostarczaja modelu ani wpisu appearance; roznica nie zmienia wspolnego
   lancucha `Mod_HakList -> appearance.2da -> m2a_m6p01`.
3. Oba archiwa i wszystkie wymienione GFF-y przechodza wlasny odczyt. Toolset
   uprzednio otworzyl obszar wygenerowanego MOD-a; nie jest to dowod gry.
4. Generator zapisuje w IFO wartosci odbiegajace od kontrolnego MOD-a Toolsetu:

| Pole IFO | Wygenerowany | Reczny Toolset |
| --- | --- | --- |
| `Mod_MinGameVer` | `1.69` | `1.89` |
| `Expansion_Pack` | `0` | `3` |
| `Mod_Creator_ID` | `0` | `2` |
| `Mod_MinPerHour` | `60` | `2` |
| callbacki modulu | puste | standardowe `x2_` / `x3_` / `nw_` |

5. ARE generatora jest minimalnym obszarem `2x2` z wlasnym zestawem oswietlenia
   (`LightingScheme=0`, `DayNightCycle=1`, `Flags=0`), podczas gdy reczny
   obszar Toolsetu ma `8x8`, `LightingScheme=13`, `DayNightCycle=0` i
   `Flags=3`. GIT w obu przypadkach ma ten sam komplet list instancji oraz
   `AreaProperties`; roznia sie tylko wartosciami ambientu/muzyki.

## Ocena hipotez

- **Odrzucone jako samodzielna przyczyna:** brak palet Toolsetu lub brak wpisu
  HAK. Reczny MOD ma palety, ale wskazuje identyczny HAK; wpis HAK jest
  prawidlowy w obu plikach.
- **Niezweryfikowane, ale bledne w generatorze:** `Mod_MinGameVer=1.69` i
  `Expansion_Pack=0` nie odpowiadaja kontrolnemu MOD-owi zapisanemu przez
  aktualny Toolset. Generator powinien dostac jawny profil metadanych NWN EE,
  z testem porownujacym typy i wartosci tych pol.
- **Wniosek:** skoro reczny MOD z Toolsetowymi metadanymi i tym samym HAK-em
  rowniez nie renderuje modelu w grze, roznice generatora MOD-a nie moga same
  wyjasnic braku modelu. Glowny trop pozostaje wspolny binarny model/MDX i jego
  kontrakt runtime, nie kontener MOD.

## Wynik i kolejny krok

Wynik porownania: `MOD-GENERATOR-METADATA-GAP` potwierdzony;
`RUNTIME-MODEL-ROOT-CAUSE` nadal otwarty. Przed poprawa generatora nalezy
zapisac test kontraktowy dla profilu IFO NWN EE. Rownolegle diagnoza runtime
ma porownac wygenerowany binarny MDL/MDX z odczytanym natywnym modelem,
poniewaz jest to jedyny wspolny element obu nieudanych przebiegow.
