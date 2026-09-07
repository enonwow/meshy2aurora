# Audyt wzrostu rozmiaru workspace — 2026-08-02

Status: diagnoza zakończona; remediacja konfiguracji nie została jeszcze
wdrożona.

## Zakres i metoda

Audyt objął kanoniczny workspace `C:\Projects\meshy2aurora`, trzy aktywne
worktree, konfigurację Cargo, strukturę targetów zarejestrowaną bezpośrednio
przed czyszczeniem, `proof-output`, lokalne modele, artefakty, Git oraz katalogi
tymczasowe. Nie uruchamiano ponownego pełnego builda, ponieważ odtworzyłby
właśnie usunięte dziesiątki GiB danych.

Pomiar używa jednostek GiB (`bytes / 2^30`), mimo że Eksplorator Windows opisuje
je etykietą GB.

## Wynik wykonawczy

Workspace osiągnął `95.771 GiB` (`102,832,860,919` bajtów). Bezpośrednią
przyczyną było `84.9 GiB` odtwarzalnych artefaktów Cargo w trzech niezależnych
katalogach `target`. Po wykonanym `cargo clean` workspace zajmuje `11.562 GiB`
(`12,414,976,727` bajtów).

To nie modele Meshy spowodowały skok do 96 GB. Kanoniczne `sample-3d` miało
`1.125 GiB`. Dominujący mechanizm to iloczyn:

1. pełne symbole debug i incremental domyślnego profilu Cargo;
2. bardzo duża liczba osobnych targetów testowych i przykładowych;
3. natywne, WASM, release i doraźne warianty builda;
4. trzy niezależne worktree z osobnymi katalogami `target`;
5. brak limitu rozmiaru, polityki retencji i automatycznego raportu.

Drugim, znacznie mniejszym mechanizmem jest zamierzona retencja immutable
proofów. `proof-output` zajmuje `7.289 GiB` i zawiera co najmniej `2.192 GiB`
identycznych bajtowo kopii w plikach o rozmiarze co najmniej 64 KiB. Tych
pakietów nie wolno traktować jak cache Cargo.

## Stan przed czyszczeniem

| Obszar | Rozmiar | Klasyfikacja |
|---|---:|---|
| `.worktrees` łącznie | 56.948 GiB | głównie osobne targety Cargo |
| główny `target` | 28.919 GiB | odtwarzalny build/cache |
| `proof-output` | 7.289 GiB | immutable proof i narzędzia referencyjne |
| `sample-3d` | 1.125 GiB | kanoniczne modele źródłowe |
| `.git` | 0.443 GiB | historia i obiekty Git |
| `artifacts` | 0.337 GiB | wygenerowane deliverables |

Dokładny wynik `cargo clean`:

| Target | Usunięte pliki | Rozmiar zgłoszony przez Cargo |
|---|---:|---:|
| główny workspace | 37,233 | 28.9 GiB |
| worktree `animation` | 37,342 | 32.9 GiB |
| worktree `items` | 29,276 | 23.1 GiB |
| **Łącznie** | **103,851** | **około 84.9 GiB** |

## Przyczyna 1: ciężkie domyślne profile Cargo

Root `Cargo.toml` nie definiuje `[profile.dev]` ani `[profile.test]`. Nie ma też
projektowego lub użytkownikowego `.cargo/config.toml`, `CARGO_TARGET_DIR` ani
`CARGO_INCREMENTAL`.

W efekcie działa domyślny profil `dev`: pełne debug info oraz incremental.
Profil `test` dziedziczy ustawienia `dev`. Cargo dokumentuje wprost, że
incremental zapisuje dodatkowe dane w `target`, a pełne debug info jest
domyślne dla `dev`:

- <https://doc.rust-lang.org/cargo/reference/profiles.html#dev>
- <https://doc.rust-lang.org/cargo/reference/profiles.html#incremental>
- <https://doc.rust-lang.org/cargo/reference/profiles.html#test>

W snapshotcie głównego `target/debug`:

| Podkatalog | Rozmiar |
|---|---:|
| `deps` | 12.464 GiB |
| `incremental` | 7.434 GiB |
| `examples` | 6.204 GiB |
| `build` | 0.135 GiB |

Największe pojedyncze artefakty potwierdzały ten mechanizm:

- `libm2a_core*.rlib`: około 222–307 MiB za kopię;
- PDB przykładów: około 143–167 MiB za plik;
- pojedyncze drzewa incremental: około 166–170 MiB.

## Przyczyna 2: duża liczba osobnych binariów

W głównym branchu znajdują się:

- 47 przykładów Rust;
- 56 targetów/plików testów integracyjnych;
- 60 modułów źródłowych crate.

Źródła przykładów mają razem tylko `542.4 KiB`, lecz ich artefakty w
`target/debug/examples` osiągnęły `6.204 GiB`. Każdy przykład i test integracyjny
jest osobnym programem linkowanym z `m2a-core` i otrzymuje własne dane debug.
Cargo potwierdza, że każdy target testowy jest osobnym executable oraz że
zwykłe `cargo test` domyślnie buduje również przykłady, aby sprawdzić ich
kompilowalność:

- <https://doc.rust-lang.org/cargo/commands/cargo-test.html#description>
- <https://doc.rust-lang.org/cargo/commands/cargo-test.html#target-selection>

Repo dodatkowo uruchamia pełną kardynalność targetów w CI i Dockerze:

```text
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p m2a-wasm --target wasm32-unknown-unknown
```

`--all-targets` obejmuje biblioteki, binaria, testy, benchmarki i przykłady.
Jest poprawnym gate'em CI, ale jest kosztowne jako częsty lokalny inner loop.

## Przyczyna 3: mnożenie przez worktree, target i profil

Każdy Git worktree ma własny root Cargo, dlatego bez wspólnej konfiguracji ma
też własny `target`:

- główny branch: 28.9 GiB;
- `animation`: 32.9 GiB;
- `items`: 23.1 GiB.

W głównym target dodatkowo współistniały:

- `debug`: 26.238 GiB;
- doraźny `ms8-material-check`: 2.015 GiB;
- `release`: 0.364 GiB;
- `wasm32-unknown-unknown`: 0.301 GiB.

Worktree `animation` miał osobno `29.553 GiB` natywnego debug oraz `3.198 GiB`
WASM. Cargo poprawnie rozdziela artefakty dla targetów i profili, lecz projekt
nie ma mechanizmu usuwania starych kombinacji po zakończeniu zadania.

## Przyczyna 4: brak kontroli retencji

W repo nie znaleziono:

- limitu rozmiaru targetu lub workspace;
- raportu trendu rozmiaru;
- procedury `cargo clean` dla zakończonego worktree;
- rozdzielenia lokalnego szybkiego gate'a od pełnego `--all-targets`;
- polityki dla doraźnych `--target-dir`;
- ustawień ograniczających debug info lub incremental.

Ponieważ `target` jest poprawnie ignorowany przez Git, jego wzrost nie jest
widoczny w `git status`. Bez osobnego miernika problem ujawnia się dopiero w
Eksploratorze lub przy braku miejsca.

## Przyczyna 5: immutable proof i duplikacja payloadów

`proof-output` zawiera 166 katalogów najwyższego poziomu i zajmuje `7.289 GiB`.
Największe klasy payloadów:

| Rozszerzenie | Liczba | Rozmiar |
|---|---:|---:|
| `.hak` | 105 | 2.342 GiB |
| `.tga` | 109 | 1.223 GiB |
| `.glb` | 82 | 0.923 GiB |
| `.2da` | 103 | 0.506 GiB |
| `.mdl` | 131 | 0.416 GiB |
| `.zip` | 2 | 0.373 GiB |

Hashowanie kandydatów o identycznym rozmiarze, ograniczone do plików co
najmniej 64 KiB, wykazało:

- 77 grup identycznej zawartości;
- `2.192 GiB` nadmiarowych kopii — dolna granica, bo mniejsze pliki pominięto;
- między innymi 28 kopii jednego 12 MiB TGA, 24 kopie innego 12 MiB TGA,
  15 kopii jednego HAK oraz liczne kopie GLB i `appearance.2da`.

Jest to zgodne z obecną zasadą zamrożonego proof lineage: kandydaci zachowują
własne kompletne pakiety i hashe. Istniejących packetów nie wolno usuwać,
przepakowywać ani zastępować odwołaniami bez osobnej zmiany standardu.

Nietypowym przypadkiem jest
`proof-output/third-party-tools/neverblender-reference` (`1.475 GiB`):

- rozpakowany Blender 4.0.2: `1.099 GiB`;
- zachowany ZIP tej samej dystrybucji: `0.373 GiB`.

Katalog jest związany z historycznym proofem NeverBlender i ma jawne referencje
w dokumentacji. Nie jest bezpiecznym celem automatycznego czyszczenia, ale jego
klasyfikacja jako trwałej części `proof-output` wymaga osobnej decyzji
retencyjnej.

## Co nie jest główną przyczyną

- `sample-3d`: `1.125 GiB`; to wymagane kanoniczne źródła Meshy;
- kod Rust w głównym worktree: około `3.1 MiB` źródeł crate;
- `.git`: obecnie około `0.51 GiB`, w tym tylko `43.07 MiB` wykrytego garbage;
- rootowe `.playwright-cli`, `.codex-tmp` i `output`: razem około `0.22 GiB`;
- zależności webowe i kod aplikacji są małe w porównaniu z targetami Cargo.

Po czyszczeniu największe obszary to `proof-output` (`63%`), nie-targetowa
zawartość `.worktrees` (`13.8%`) i `sample-3d` (`9.7%`). To prawidłowo zmienia
charakter dalszej optymalizacji: kolejne masowe kasowanie nie może obejmować
proofów ani źródeł.

## Rekomendowany plan remediacji

### P0 — zatrzymać ponowne dojście do 96 GB

1. Zmierzyć na czystym target cztery warianty: bieżący pełny build, wyłączone
   incremental, ograniczone debug info oraz lokalny gate bez przykładów.
   Rejestrować zimny/ciepły czas builda i rozmiar targetu.
2. Jeżeli pomiar potwierdzi oszczędność, rozważyć w root `Cargo.toml`:

   ```toml
   [profile.dev]
   debug = "line-tables-only"
   incremental = false

   [profile.test]
   debug = "line-tables-only"
   incremental = false
   ```

   `line-tables-only` zachowuje lokalizacje plik/linia dla backtrace, ale nie
   pełne informacje o zmiennych. Wyłączenie incremental zmniejszy dysk kosztem
   wolniejszych częściowych rekompilacji. Decyzja wymaga benchmarku.
3. Zdefiniować dwa jawne gate'y:
   - lokalny inner loop wybierający `--lib`, konkretne `--test` lub
     `--lib --tests`, bez przykładów;
   - pełny CI/release gate z `--all-targets`.
4. Po zakończeniu pracy w worktree, po potwierdzeniu commit/push i braku
   aktywnego `cargo`/`rustc`, czyścić wyłącznie jego katalog target przez
   `cargo clean --manifest-path <worktree>/Cargo.toml`.
5. Dodać read-only raport rozmiaru targetów uruchamiany przed i po pełnym gate.
   Najpierw zebrać nowy baseline; dopiero potem ustalić warning i hard limit.

### P1 — zmniejszyć kardynalność buildów

1. Skonsolidować historyczne `materialize_*` w jedno narzędzie z subcommandami
   albo wydzielić je do jawnie uruchamianego pakietu/gate'a.
2. Alternatywnie oznaczyć historyczne przykłady tak, aby zwykły lokalny
   `cargo test` ich nie budował, zachowując osobny gate kompilowalności w CI.
3. Zabronić doraźnych `--target-dir` bez nazwanego właściciela i późniejszego
   czyszczenia. `ms8-material-check` pozostawił samodzielne `2.015 GiB`.
4. Przetestować wspólny target dla worktree jako eksperyment, nie wdrażać go
   bez pomiaru współbieżności, blokad Cargo i faktycznego reuse między różnymi
   ścieżkami źródłowymi.

### P1 — kontrolować przyszły wzrost proof-output

1. Nie modyfikować istniejących immutable packetów.
2. Zaprojektować dla przyszłych packetów content-addressed immutable blob store
   z manifestami SHA-256. Hardlink/reflink można dopuścić dopiero po testach
   read-only, backup/export i odporności na zmianę jednego linku.
3. Rozstrzygnąć, czy przyszły packet musi kopiować cały source GLB, TGA i 2DA,
   czy może wiązać część payloadu przez kanoniczny manifest i hash. Zmiana nie
   może osłabić candidate identity ani historycznej odtwarzalności.
4. Osobno zaudytować `third-party-tools/neverblender-reference`: zachować
   provenance i historyczny proof, ale ustalić, czy konieczne jest trwałe
   przechowywanie jednocześnie ZIP-a i rozpakowanej dystrybucji.

### P2 — mniejsze porządki

1. Po związaniu dowodów można przeglądać ignorowane `output`, `.playwright-cli`
   i `.codex-tmp`; ich łączny potencjał jest jednak poniżej 1 GiB także z
   worktree `animation`.
2. Git garbage ma tylko `43.07 MiB`; `git gc` nie jest priorytetem i nie należy
   go wykonywać jako substytutu rozwiązania Cargo.
3. Całych worktree nie wolno usuwać tylko dla oszczędności miejsca, dopóki
   zawierają potrzebne branche, artefakty lub lokalny stan. Czyścić ich target,
   a usunięcie worktree traktować jako osobną decyzję lifecycle.

## Weryfikacja remediacji

Remediację należy uznać za potwierdzoną dopiero po pomiarze na tym samym
commitcie i tym samym zestawie testów:

1. rozmiar target po zimnym pełnym gate;
2. rozmiar po drugim, ciepłym gate;
3. czas obu przebiegów;
4. rozmiar `deps`, `incremental`, `examples` i WASM;
5. kompletność testów, clippy i backtrace;
6. zachowanie przy dwóch równoległych worktree.

Nie należy deklarować konkretnej oszczędności profili przed tym eksperymentem.
Twardym faktem pozostaje, że obecna konfiguracja zgromadziła `84.9 GiB`
artefaktów, z czego w samym głównym target `7.434 GiB` stanowił incremental, a
`6.204 GiB` katalog przykładów.
