# Creature weapon manual roll/pitch/yaw - implementacja 2026-08-04

Status: **implementacja i bramki offline gotowe; bez nowej iteracji MOD/HAK**.

## Wynik

Pipeline Creature obsluguje teraz dwa jawne tryby ustawienia broni:

- `AUTO` - dotychczasowa automatyczna kalibracja dloni i natywnej osi przedmiotu;
- `AUTO_PLUS_OFFSETS` - automatyczna kalibracja plus osobne, reczne
  `roll/pitch/yaw` dla prawej i lewej dloni.

Zakres kata kazdej osi wynosi `[-180, 180]` stopni. Dane niefinitywne,
nieobslugiwany schemat, wartosc poza zakresem oraz niezerowe offsety ukryte pod
trybem `AUTO` sa odrzucane fail-closed.

## Kontrakt transformacji

Osie sa lokalnymi osiami przedmiotu po automatycznej kalibracji:

- `roll` - wokol osi ostrza `+Y`;
- `pitch` - wokol lokalnej osi `+X`;
- `yaw` - wokol lokalnej osi `+Z`.

Dla wektorow kolumnowych Core i Studio stosuja dokladnie:

```text
M_final = M_auto * Rz(yaw) * Rx(pitch) * Ry(roll)
```

Korekta zmienia wylacznie obrot hooka. Translacja wyznaczona przez automatyczna
kalibracje dloni pozostaje identyczna. Wynik musi byc sztywnym obrotem o
wyznaczniku `+1`; skala, shear i odbicie sa zabronione.

## Przeplyw danych

Opcje sa przenoszone przez caly aktywny produktowy przeplyw Creature:

1. formularz Studio dla zrodla Creature;
2. podglad Weapon Grip na binarnym readbacku MDL;
3. `Apply and rebuild`, ktory uniewaznia poprzedni build;
4. identyfikator artefaktu i cache;
5. wszystkie requesty `BUILD_MODEL_PACKAGE` Creature;
6. Worker i scisly JSON opcji WASM;
7. Core IR, authoring `rhand`/`lhand` i binary MDL writer;
8. raport `weaponAnchorAuthoring.gripAdjustment`.

Raport trybu recznego zachowuje zadane katy, macierz automatyczna, macierz
koncowa i stala nazwe kolejnosci skladania. Projektor wyniku sprawdza, czy
macierz koncowa raportu jest taka sama jak macierz zapisana dla odpowiedniego
hooka. W trybie `AUTO` pole korekty nie jest emitowane, aby zachowac semantyke
historycznego raportu.

## Studio i podglad

Studio pokazuje osobne kontrolki prawej i lewej dloni. Podglad uzywa czytelnego
proxy short sword zgodnego z baza osi NWN, a nie udaje geometrii konkretnego
przedmiotu z gry. Zmiana draftu jest nakladana jako roznica pomiedzy ustawieniem
zastosowanym w aktualnym MDL i nowym ustawieniem, dzieki czemu offset nie jest
nakladany podwojnie po przebudowie.

Podglad nie jest finalnym dowodem renderera Aurora/NWN. Jego zadaniem jest
deterministyczna kalibracja osi przed materializacja kolejnego kandydata.

## Kryteria ukonczenia - wynik

| Kryterium | Wynik |
|---|---|
| Auto zachowuje dotychczasowa kalibracje | PASS |
| Osobne R/P/Y dla prawej i lewej dloni | PASS |
| Lokalna, jawna kolejnosc obrotow | PASS |
| Translacja hooka pozostaje bez zmian | PASS |
| Walidacja finite/range/rigid/determinant | PASS |
| Opcje przechodza przez App, Worker i WASM | PASS |
| Opcje uczestnicza w identity/cache | PASS |
| Readback report zgadza sie z macierza hooka | PASS |
| Podglad draftu i jawne Apply/rebuild | PASS |
| Test wlascicielski nowego MOD/HAK w Aurora/NWN | OCZEKUJE NA GATE |

## Wykonane bramki

- `cargo test -p m2a-core --lib`: `109 passed`, `3 ignored` lokalne korpusy;
- `cargo test -p m2a-wasm --lib`: `40 passed`;
- `cargo check -p m2a-wasm`: PASS;
- `cargo fmt --all -- --check`: PASS;
- `npm test` w `apps/studio-web`: `272 passed` w `48` plikach;
- `npm run typecheck`: PASS;
- browser Worker/WASM, procedural H2 z niezerowym RPY: PASS;
- test projektora: finalna macierz `rhand` musi byc identyczna z raportem: PASS.

## Dlaczego nie powstal nowy modul

Aktualnym zamrozonym kandydatem jest V7:

- `m2aweapdemo7.mod`;
- modul `Meshy2Aurora Creature Weapon Grip V7`;
- Area `Meshy2Aurora Creature Weapon Test V7`;
- `m2aweaphak7.hak`.

Jego zapisany stan to `modelVisibility=not_tested` i
`proofCompleteness=missing`. Screenshot V6 nie jest swiezym wynikiem V7.
Zgodnie z bramka iteracji modelu nie wolno materializowac V8 ani zmieniac
lineage V7, dopoki wlasciciel nie zwroci wyniku wizualnego dokladnie dla V7.
Implementacja jest dlatego zakonczona offline, ale nowy demonstrator pozostaje
zablokowany prawidlowo, a nie pominiety technicznie.
