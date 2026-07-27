# M0 v12 — runtimeowy obraz bez widocznego modelu

**Status:** `failed` dla aktualnego kadru runtime; przyczyna renderera jest jeszcze `missing`.

## Fakt z aktualnej sesji

Prawdziwy capture `PrintWindow` procesu `nwmain` PID `25792` pokazuje
załadowany test `m2a_m0v12`, ale nie pokazuje sylwetki M0:

- PNG: `proof-output/m0-v12-toolset-runtime-20260720/m0-v12-nwn-runtime-printwindow-20260720.png`
- SHA-256: `cc8518cbc6e3fd22d7378d383ac69a75c621ee13fd7a34a3fd563549814ed2bc`
- uruchomienie: `+TestNewModule "m2a_m0v12"`
- bieżąca instancja GIT: `nw_dwarfmerc001`, `Appearance_Type=848`,
  `M2A_M0_MESHY_RIGID -> m2a_m0p01`, pozycja `[5.0642, 14.9358, 0]`.

Nie jest to dowód, że model renderuje się w NWN. Nie wolno opisywać tego
przebiegu jako sukcesu runtime.

## Co wykluczono, a czego nie

Łańcuch `MOD -> HAK -> appearance row 848 -> m2a_m0p01` jest zgodny z
aktualnym GIT i log potwierdza ładowanie właściwego modułu. Nie ma komunikatu
o brakującym `m2a_m0p01`, HAK-u ani teksturze.

To nie rozstrzyga renderu, ponieważ instancja leży poza bezpośrednią osią
startu IFO `[10, 10, 0]`, kierunek `[0, 1]`. Następny test ma najpierw
ustawić jedną zapisaną pozycję na tej osi, a potem utworzyć świeży obraz NWN.
Bez tego obraz nie odróżnia niewidoczności poza kadrem od odrzucenia modelu
przez renderer.

## Porównanie binarne

Własny readback bieżącego M0 i poprzedniego RIGID H1 v20, który był obserwowany
w NWN po powrocie postaci w okolice fixture, daje zgodne kluczowe pola mesha:

| Pole | M0 v10/v12 | H1 RIGID v20 |
| --- | ---: | ---: |
| node flags | `0x21` | `0x21` |
| `meshType` | `3` | `3` |
| `render` / `shadow` | `1` / `1` | `1` / `1` |
| vertex colors | obecne | obecne |
| model bounds / radius | `[-5,-5,-1]..[5,5,10]` / `7` | takie same |

Wniosek: sama wartość `meshType=3` nie jest uzasadnioną przyczyną zmiany
writera. Wymuszona zmiana MDL przed testem pozycji byłaby zgadywaniem.

## Granica operacyjna

Aktualny `nwmain` jest nadal responsywny. Zwykły, targetowany `WM_CLOSE`
został już wysłany do tego samego HWND i nie zamknął procesu; nie wolno
powtarzać innej sekwencji zamknięcia ani wymuszać zakończenia bez osobnej,
jednoznacznej autoryzacji. Nie zmieniono żadnego pliku INI, MRU ani ustawienia
użytkownika.
