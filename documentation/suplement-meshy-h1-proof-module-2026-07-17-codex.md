# Suplement: realny Meshy H1 i modul proof — 2026-07-17

Status: `ZAIMPLEMENTOWANE I ZWERYFIKOWANE STRUKTURALNIE`.

Ten suplement doprecyzowuje aktualny vertical slice H1. Nie zmienia
historycznych decyzji w starszych dokumentach cloud. Nie jest rowniez
deklaracja runtime PASS w Toolsecie lub grze.

## Zakres zamknietego testu

Studio przeszlo lokalny workflow dla wlascicielskiego pliku Meshy H1 i lokalnej
`appearance.2da`:

| Wejscie / wynik | Potwierdzenie |
| --- | --- |
| GLB H1 | 1 334 vertices, 1 556 triangles, 24 jointy, 1 clip; SHA-256 `3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f` |
| Tabela appearance | SHA-256 `815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a`; wynikowy physical row `15100` |
| MDL | `m2a_m6p01.mdl`, 204 312 B; binary own-readback PASS |
| HAK | `m2a_m6p01.hak`, 19 688 878 B; SHA-256 `c116611095f30bfc8fde9eeaa914375ba74612416193a39c939506b9eb16d553` |
| Modul proof | `m2a_h1proof.mod`, 2 033 B; SHA-256 `52732975d10bb3718a86dc657a9ead71775eb65fbfe3fe2ed1a727845445a4f7` |

Dowod wizualny lokalnego przebiegu: ignorowany przez Git
`proof-output/meshy-h1-e2e-2026-07-17/08-review-real-h1-proof-module.png`.
Ekran pokazuje rzeczywiste wejsciowe hashe, readback MDL `PASS`, HAK oraz
osobny artefakt `m2a_h1proof.mod` z przyciskiem Download.

## Kontrakt modulu

Fakty Aurora First (read-only dekompilacja i lokalny referencyjny MOD):

- kontener ma sygnature `MOD V1.0`;
- `module.ifo` ma resource type `2014`;
- area to potwierdzona trojka `ARE` / `GIC` / `GIT`: `2012` / `2046` / `2023`;
- dekompilacja potwierdza `Mod_HakList`, element `Mod_Hak`, `Creature List`,
  `TemplateResRef`, polozenia i orientacje GIT oraz `Appearance_Type` UTC.

Wygenerowany MOD jest samodzielnym structural proof archive: zawiera
`module.ifo`, `m2a_h1area.are`, `m2a_h1area.gic`, `m2a_h1area.git` i
`m2a_h1.utc`. IFO wskazuje HAK `m2a_m6p01`; UTC wskazuje dokladnie physical
row `15100`; GIT ma instancje w `Creature List`. Po zapisie `m2a-core` odczytuje
ponownie MOD oraz kazdy GFF, sprawdzajac sygnature, zasoby, typy GFF, liste HAK
i `Appearance_Type` UTC. Test unitowy tej bramki jest w
`crates/m2a-core/src/proof_module.rs`.

MOD oraz HAK sa celowo oddzielnymi pobraniami. Uzytkownik instaluje HAK pod
nazwa zgodna z resrefem `m2a_m6p01.hak`, a modul odnosi sie do niego przez
`Mod_HakList`; aplikacja nie zapisuje nic do katalogu gry ani Toolsetu.

## Aktualizacja budzetu dokladnego mapowania odleglosci

Domyslny hard limit `ProfileALimitsV1.max_distance_evaluations` wynosi teraz
`10_000_000`. Realny H1 przechodzi dokladne, exhaustywne przypisanie do
powierzchni w tym budzecie. Zmiana jest implementacyjna i celowa: poprzednie
`3_000_000` z historycznego kontraktu M3 nie wystarczalo dla realnej,
wielokosciowej konfiguracji H1. Nadal obowiazuje deterministyczne zliczanie
przed kazda ewaluacja i fatal przed przekroczeniem limitu; nie ma fallbacku
heurystycznego ani cichego obnizania geometrii.

## Granica, ktora pozostaje otwarta

To nie jest dowod, ze konkretny Toolset/NWN EE zaladowal modul lub wyswietlil
creature. Takie uruchomienie wymaga oddzielnej, kontrolowanej przez wlasciciela
sesji Toolset/game i jest oznaczone w Studio jako `Runtime Proof: OPEN_M6`.
Zamkniety tutaj cel to realny lokalny H1 -> Studio -> zweryfikowany MDL/HAK/
MOD, wraz z widocznymi screenshotami i pobieralnymi artefaktami.
