# `m2a_ship1_mod.mod` — średniowieczny statek gotowy do testu właściciela

1. Dokładny plik modułu testowego: `m2a_ship1_mod.mod`.
2. Nazwa modułu w Toolset: `Meshy2Aurora Medieval Ship Demo`.
3. Dokładna nazwa Area: `Meshy2Aurora Medieval Ship Harbor`.

Status: `READY_FOR_OWNER_PROOF / TOOLSET_NOT_TESTED / NWN_NOT_TESTED`

## Tożsamość demo

| Pole | Wartość |
|---|---|
| HAK | `m2a_ship1_hak.hak` |
| HAK resref | `m2a_ship1_hak` |
| Area resref | `m2a_ship1_ar` |
| Model resref | `m2a_ship1_mdl` |
| Texture resref | `m2a_ship1_tex` |
| Blueprint resref | `m2a_ship1_utp` |
| Object tag | `m2a_ship1_medieval_ship` |
| Appearance row | `16500` |
| Placement | `(10.0, 14.5, 0.0)`, bearing `0.0` |
| Uniform scale | `8.0` |
| Rozmiar w demo | `15.1361 × 10.6752 × 8.2813 m` |

## Meshy — dokładnie jeden przebieg

Do Meshy przekazano dokładne bajty PNG dostarczonego przez właściciela, bez
cropu, rozdzielania arkusza, image enhancement, baked-light removal, auto-size,
texture promptu albo ręcznej korekty. Jedynym wejściem był obraz o SHA-256:

`a04ea16b36ff098e276604e96b5d873be176f9c79ef0c9fb4ed4be68dbe2ea33`

| Pole | Wartość |
|---|---|
| Profil | `S1-static-prop/v1` |
| Tryb | Meshy 6 Image-to-3D |
| Local run ID | `d6ffe618-e08b-4c41-ad25-a73b941efbe3` |
| Meshy task ID | `019fb8fb-896a-7219-bb1d-b566c1c8f1d7` |
| Target | `100000` polygonów, triangle remesh |
| Wynik Meshy | `99432` trójkąty |
| GLB | `17880760` bajtów |
| GLB SHA-256 | `9ed47edca7765888ca0e034455ba4302a88fa700a72004cbdbee29f311e58ced` |
| Struktura | 1 mesh, 1 primitive, 1 material, 4 images, 0 skins, 0 animations |
| Koszt dokładnego tasku wg Meshy | `30` kredytów |

Account-wide saldo zmieniło się z `626` na `553`, ale w historii w tym samym
oknie pojawił się drugi, niezależny task Image-to-3D
`019fb8fe-4614-77e7-b22f-63eb5e460110`, który również raportuje koszt `30`.
Dlatego różnica całego salda nie jest przypisana temu kandydatowi. Dla jego
dokładnego task ID autorytatywne pole `consumed_credits` ma wartość `30`.
Nie wykonano retry ani drugiej generacji dla tego modelu.

Kanoniczne źródło:

`C:\Projects\meshy2aurora\sample-3d\concept-medieval-ship-s1-p100k-v1\source.glb`

## Pipeline Placeable

Model nie był ręcznie naprawiany. Pipeline wykonał własne obowiązkowe gate'y:

- usunął `1739` degeneratów, pozostawiając `97693` Aurora-safe trójkąty;
- zastosował wyłącznie jednolity scale `8×` i matematyczne przesunięcie podstawy
  do `Z=0`, zgodnie z wymaganiem dużej skali statku;
- podzielił render bez utraty geometrii na pięć strumieni:
  `10313, 21845, 21845, 21845, 21845` trójkątów;
- każdy strumień mieści się w binary MDL gate `21845` trójkątów;
- wygenerował i odczytał MDL, TGA, PWK, `placeables.2da`, UTP, ITP, GIT/GIC,
  ARE, IFO, HAK i MOD;
- potwierdził dokładną nazwę Area oraz
  `collisionCompleteness=ascii_pwk_emitted_offline_readback_passed`.

Komponenty offline: MDL, PWK, 2DA, UTP, GIT/GIC, palette i package mają status
`passed`. Widoczność pozostaje `not_tested`, a kompletność proof `missing`.

## Zamrożone artefakty i instalacja

Kanoniczny packet:

`C:\Projects\meshy2aurora\proof-output\concept-medieval-ship-placeable-v1-20260731`

| Artefakt | Bajty | SHA-256 | Natywna instalacja |
|---|---:|---|---|
| `m2a_ship1_mod.mod` | 13379 | `3d722a3f8ebe2fc646d8993580228ce7512acfb33db43793ec3ad87ced387d22` | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_ship1_mod.mod` |
| `m2a_ship1_hak.hak` | 23107323 | `c1f5d1a0027c301d9425b680849b8e072d598d23dd5e517540260bde28a0186b` | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_ship1_hak.hak` |

Oba cele natywne były nieobecne przed kopią. Po instalacji ich długości i
SHA-256 są byte-identical z kanonicznymi źródłami. Nie nadpisano, nie usunięto
ani nie przemianowano żadnego wcześniejszego pliku.

Pozostałe hashe:

| Zasób | SHA-256 |
|---|---|
| MDL | `3117df90b5573e457efb6fc9de7afa4fbfcb65846b7ac8a825b8c5b4b8bfb6bb` |
| PWK | `e417c67eaec7ba04eb26172c3bf0ac639e8bfb5856e4a9b413f360202ca9ec50` |
| TGA | `626db791f21c7095b2cab7a63601d66a8c64365ddd6b4b17d589452d3577b1e5` |
| `placeables.2da` | `0a4d0a57b4b33d1888d5a0b009a223afa2d72c47c337ff53a0dd2182884301bf` |
| UTP | `880d4f13865a22db7ea15f53169048f371696b90c41010c109ca8fc449d6aa24` |

## Weryfikacja

- Local Bridge: `21/21` testów PASS.
- Kanoniczny GLB intake: PASS.
- Placeable pipeline: `12 passed`, `1` oczekiwany env-gated ignored.
- Materializator demo: Clippy `-D warnings` PASS.
- HAK/MOD/resource readback: PASS.
- Kanoniczny asset-layout gate: PASS.
- Agent nie uruchamiał ani nie adoptował Aurora Toolset lub NWN.

## Kroki owner proof

1. Otworzyć `m2a_ship1_mod.mod`.
2. Potwierdzić nazwę modułu `Meshy2Aurora Medieval Ship Demo`.
3. Otworzyć Area `Meshy2Aurora Medieval Ship Harbor`.
4. Ocenić umieszczony obiekt `m2a_ship1_medieval_ship` w Toolset.
5. Jeśli model jest widoczny, przetestować ten sam MOD/HAK w NWN bez zmiany
   artefaktów.
6. Zgłosić osobno dla Toolset i NWN:
   `modelVisibility=visible|not_visible` oraz
   `proofCompleteness=verified|failed|missing`.

Nowa iteracja modelu jest niedozwolona bez świeżego, candidate-bound wyniku
`modelVisibility=not_visible`. Brak lub nieczytelny proof pozostawia ten sam
kandydat do ponownego sprawdzenia.
