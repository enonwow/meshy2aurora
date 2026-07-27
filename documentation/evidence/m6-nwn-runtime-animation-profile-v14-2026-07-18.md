# M6 runtime animation profile v14-v17 (2026-07-18)

Status: `ARTEFAKT GOTOWY DO TESTU NWN; runtime proof nadal OTWARTY.`

## Problem

Audyt `m6-nwn-runtime-model-visibility-audit-2026-07-18.md` wykazal, ze
wygenerowany model zawieral wyłącznie `cpause1`, gdy lokalny dzialajacy direct
creature ma stany spawn/idle/ruchu. Toolset potrafi pokazac statyczny model,
ale to nie dowodzi, ze gra potrafi rozwiazac stan animacji stworzenia.

## Aurora First

- `C:\Projects\New Folder\export\decompiled_all.c:886491-886499` pokazuje,
  ze parser modelu Aurory rozpoznaje `animroot` i zapisuje go do naglowka
  animacji.
- `C:\Projects\New Folder\export\decompiled_all.c:886390-886425` pokazuje,
  ze parser rozpoznaje rowniez `radius` modelu.
- Read-only `c_squirrel` z lokalnego `cep3_core1.hak` ma 43 klipy, w tym
  `cappear`, `cpause1`, `cwalk` i `crun`, wszystkie z poprawnym `animroot`.

Nie skopiowano zadnego payloadu, animacji, szkieletu ani tekstury referencyjnej.
Nazwy stanow sa tylko odczytanym kontraktem runtime.

## Zmiana

`model_pipeline.rs` materializuje minimalny profil direct creature:
`cappear`, `cpause1`, `cwalk`, `crun`. Gdy wejscie Meshy dostarcza tylko
`cpause1`, trzy brakujace stany sa clean-room aliasami tego samego, dostarczonego
przez uzytkownika klipu. Istniejacy w wejsciu stan o tej nazwie nigdy nie jest
nadpisywany. Testy pokrywaja zarowno tworzenie aliasow, jak i zachowanie
specyficznego stanu z wejscia.

## Wynik generacji v17

Artefakt: `proof-output/meshy-h1-nwn-runtime-bounds-proof-v17`.

Wariant v17 zachowuje profil czterech klipow z v14, zapisuje
sentinel braku supermodelu jako `NULL`, zgodnie z lokalnym, samodzielnym
direct-creature reference `c_squirrel`, a takze normalizuje jedyny geometry
root i kazdy `animroot` do resrefu `m2a_m6p01`. Ponadto naglowek modelu
uzywa culling envelope `[-5,-5,-1]..[5,5,10]`, `radius=7`, zaobserwowany na
szesciu lokalnych modelach direct creature. Nie jest to deklarowane jako
samodzielna przyczyna braku widocznosci; to usuniecie potwierdzonych rozjazdow
binarnych danych referencyjnych.

| Pole | Wynik |
| --- | --- |
| input GLB SHA-256 | `3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f` |
| model SHA-256 | `95b285b823b424bcbdfa949fd5e012ea683b9121b480977e7880533a9fce39b2` |
| model bytes | `397320` |
| HAK SHA-256 | `d73f4265b6d087ccaddac1d72f222fc603d0926168bafa765d4501e8d0babb8a` |
| appended appearance row | `15101` |
| animation pointer array | `used=4`, `allocated=4` |
| emitted clips | `cpause1`, `cappear`, `cwalk`, `crun` |
| geometry root i animroot | `m2a_m6p01` dla modelu i kazdego klipu; bezposredni odczyt binary MDL |
| supermodel sentinel | `NULL`, odczytany bezposrednio z wygenerowanego binary MDL |
| model bounds / radius | `[-5,-5,-1]..[5,5,10]`, `7`; bezposredni odczyt binary MDL |
| own binary readback | `PASS`, bez diagnostyk |

Weryfikacja kodu: `cargo fmt --all --check`, `cargo test -p m2a-core --lib`
(47/47), `cargo test -p m2a-core --test model_pipeline` (7/7), dwa zamrozone
testy MDL dla payloadu bez animacji oraz granica M7/WASM dla batcha ready-owned.
Wszystkie uruchomione bramki przeszly.

## Granica i kolejny krok

Artefakty zostaly wygenerowane wyłącznie w kanonicznym repozytorium. Nie
kopiowano HAK-a ani MOD-a do instalacji/uzytkowych katalogow NWN, poniewaz sa
read-only bez odrebnej zgody wlasciciela. Finalnym dowodem pozostaje uruchomienie
tego HAK-a i MOD-a w NWN oraz capture widocznego modelu. Jezeli model nadal nie
bedzie widoczny, nastepna hipoteza do testu to `bounds/radius` oraz pozostale
oznaczone `OPEN_M6` pola skin/mesh; nie nalezy oznaczac v14 jako rozwiazania
przed tym proofem. Pelne uruchomienie `cargo test -p m2a-core --test mdl_writer`
nadal ujawnia wczesniejszy, niezalezny test kolejki bledow
`M4A-TRACK-PATH-UNSUPPORTED` vs `M4A-TRACK-ARITY-INVALID`; nie wynika on z
generacji v15 i nie jest w tym wpisie maskowany.
