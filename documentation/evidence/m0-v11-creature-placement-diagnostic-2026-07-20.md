# M0 v11: diagnoza braku placementu creature

**Status:** `failed` dla placementu fixture; **nie jest** to ani negatywny,
ani pozytywny wynik renderera M0.

## Cel i wiarygodny stan wejściowy

Pracujący, zapisany moduł to
`C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0v11.mod`
(`SHA-256` `168241c73768c4dff572afed5aab64924f087997a019ffab14354e5d3efae112`).
Ma jeden zapisany HAK `m2a_m0v10`
(`SHA-256` `04afbb1dc1e2f5cbb601e5d005f81396a044fc7b16597022b104367c5ee74966`)
i Area `m2a_m0a11` typu Tiny/MicroSet `2 x 2`.

Centralna transakcja `bootstrap-reconcile-hak-attached-place-creature` wybrała
w palecie dokładny leaf `Dwarf Mercenary`, a następnie wysłała jedną kierowaną
parę komunikatów do zwalidowanego `TScrollBox`. Nie wolno jednak wyciągać z
tego wniosku o placementcie: po tej akcji otwarcie Properties nie znalazło
żadnego `Dwarf Mercenary` w drzewie Area.

Decydujący readback parsera GIT po zatrzymanej transakcji jest jednoznaczny:

```json
{
  "moduleSha256": "168241c73768c4dff572afed5aab64924f087997a019ffab14354e5d3efae112",
  "area": "m2a_m0a11",
  "creatures": []
}
```

Nie uruchomiono więc Properties, nie wybrano Appearance `848` i nie zapisano
żadnej zmiany modelu. **M0 nie jest jeszcze widoczne ani zweryfikowane w
Aurora v11.**

## Bezpośrednia przyczyna placementu

Deklaracja v11 podała punkt palety lokalny względem viewportu `(1050, 120)`.
Jest on formalnie wewnątrz kontrolki `TScrollBox` o rozmiarze `1275 x 839`, ale
nie leży na renderowanej powierzchni Tiny Area.

Świeży, wyłącznie odczytowy capture zwalidowanego `TScrollBox` jest zapisany w
`proof-output/m0-v11-toolset-runtime-20260720/diagnostic-empty-area-viewport.png`
oraz w jego JSON-ie. Oględziny PNG pokazują teksturowany kwadrat `2 x 2`
pośrodku szarego viewportu (w przybliżeniu lokalny zakres `x=435..839`,
`y=217..621`); `(1050,120)` trafia w szare tło poza Area.

To wyjaśnia jednocześnie oba fakty: paleta została poprawnie wybrana, ale GIT
nie otrzymał nowej creature. Jest to problem punktu placementu, nie dowód
błędu MDL, HAK-a, `appearance.2da` ani rendererów Aurora/NWN.

Capture ma potwierdzoną tożsamość `m2a_m0v11.mod`/`m2a_m0a11`, `DISPLAY1`
`primary=false`, i metodę `physical-pixel capture of validated TScrollBox
rectangle after WindowFromPoint ownership verification`. Przedstawia pusty
Area diagnostycznie, więc nie może być podany jako proof modelu.

## Następna oddzielna transakcja

Nie wolno ponowić tej samej akcji w v11. Przygotować należy nowy, własny
moduł roboczy (nowy resref i nieistniejąca ścieżka) z tym samym, już
zweryfikowanym HAK/wierszem `848`, ale z punktem placementu **wewnątrz środka
pojedynczego tile'a**, np. lokalnie `(536,318)`. Ten punkt leży wewnątrz
tekstury górnego-lewego tile'a, a nie na szarej ramce ani na granicy tile'ów.

Nowa transakcja wymaga ponownego `bootstrap-dry-run`, jednego `bootstrap-create`,
natywnego zapisu i natychmiastowego GIT readback `creatures.length == 1` przed
jakkolwiek Properties/Appearance lub proofem viewportu. Dopiero po tym można
sprawdzać M0 jako model.

## Granice procesu

W tej diagnozie ani w poprzednich krokach nie zmieniono `nwtoolset.ini`, MRU,
pozycji okna ani ustawień użytkownika. Sesja Toolsetu PID `9408` pozostaje
własnością tego zadania i może być normalnie zamknięta/zrestartowana tylko
zgodnie z regułą z
[`aurora-operator-configuration-and-ownership-2026-07-20.md`](aurora-operator-configuration-and-ownership-2026-07-20.md).
