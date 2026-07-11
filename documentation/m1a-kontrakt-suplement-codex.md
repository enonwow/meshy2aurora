# M1A - suplement kontraktu parsera i API

Data: 2026-07-11 | Autor: Codex | Status: AKTYWNY SUPLEMENT M1A

## 1. Cel

Ten dokument zamyka niejasnosci wykryte przy starcie M1A. Nie rozszerza zakresu etapu i nie promuje syntetycznej fixture do rangi dowodu zachowania silnika Aurora.

## 2. Kontrakt typow i granicy WASM

- `m2a-core` zwraca typowany `Result<InspectionReport, ParseError>` albo rownowazne typy o tej samej semantyce.
- `m2a-core` nie zwraca gotowego lancucha JSON i nie zna JavaScriptu, DOM, filesystemu ani sieci.
- `m2a-wasm` jest jedynym miejscem serializacji publicznej odpowiedzi do JSON.
- Publiczne pola JSON uzywaja `camelCase`, zgodnie z przykladem aktywnego promptu. Pola Rust moga pozostac w `snake_case` z jawna konfiguracja serializacji.

## 3. Wersjonowanie i odpowiedz bledu

Poprawny raport zawiera top-level `schemaVersion: 1` oraz pola wymagane przez prompt M1A.

Fatalny blad wejscia zwraca stabilny obiekt bledu, a nie czesciowo poprawny raport:

```json
{
  "schemaVersion": 1,
  "code": "M2A-MDL-HEADER-INVALID",
  "severity": "error",
  "offset": 0,
  "context": "file header"
}
```

Brakujacy lub niepoprawny header, zakres MDX, wskaznik, tablica, overflow, cykl albo przekroczenie limitu sa bledami fatalnymi dla M1A. `unsupported` sluzy tylko do jawnego raportowania rozpoznanej, ale celowo nieparsowanej semantyki poza zakresem M1A; nie moze maskowac uszkodzonego wejscia.

Kazda diagnostyka lub publiczny blad zawiera semantycznie: `schemaVersion`, `code`, `severity`, `offset` oraz `context`.

## 4. Offsety i limity

Zakaz wpisywania offsetow na sztywno oznacza zakaz wartosci specyficznych dla `c_kocrachn` albo innego pojedynczego zasobu. Parser fixed-layout moze i powinien uzywac nazwanych stalych potwierdzonego profilu formatu, na przyklad rozmiaru file header oraz offsetow pol geometry/node header opisanych w aktywnym crosswalku.

Kazda taka stala musi miec komentarz provenance albo test kontraktu. Niepotwierdzone pole pozostaje niewczytane i trafia do `unsupported` lub blokuje rozszerzenie zakresu.

`ParserLimits` sa guardrailami produktu. Nazwy i wartosci domyslne musza byc jawne oraz testowane na granicy, ale nie wolno opisywac ich jako limitow silnika Aurora bez osobnego dowodu.

## 5. TDD, evidence i publikacja

- Fixture M1A jest generowana programowo w testach i nie zawiera retail/CEP payloadu.
- Subagenci implementacyjni nie wykonuja commitow ani pushy.
- Orkiestrator wykonuje checkpoint commit i push dopiero po przejsciu calego DoD M1A i zapisaniu wynikow w `documentation/evidence/M1A-evidence.md`.
- Ten suplement oraz koncowy raport `documentation/prototyp-parsera-m1a-claude.md` stanowia dwa dokumentacyjne artefakty M1A; evidence pozostaje append-only dziennikiem dowodow.

## 6. Poza zakresem

Suplement nie zezwala na mesh/skin/controllers/animations, HAK/ERF, 2DA, GLB, writer, React/Three.js ani automatyzacje Toolsetu w M1A.
