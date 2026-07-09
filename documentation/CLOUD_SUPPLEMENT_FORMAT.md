# Cloud Supplement Format

Ten format obowiazuje dla kazdego pliku `[nazwa]-cloud.md`. Aktualne reguly wymiany Cloud/Codex sa w `documentation/reguly-dokumentacji-cloud.md` i maja pierwszenstwo, jezeli pojawi sie konflikt.

Cel: plik `*-cloud.md` ma byc suplementem dla Codex Cloud albo innego watku cloud. Ma powiedziec, co cloud ma zrobic, czego nie wolno mu zgadywac, jakie lokalne zrodla sa autorytatywne i jaki artefakt ma oddac.

## Nazwa pliku

Format:

```text
[nazwa-dokumentu-bazowego]-cloud.md
```

Przyklady:

```text
wymagania-startowe-cloud.md
aurora-models-animations-audit-cloud.md
mdl-import-pipeline-cloud.md
```

## Wymagany uklad

Kazdy `*-cloud.md` powinien miec ponizsze sekcje.

```markdown
# [Tytul] - Cloud Supplement

Status: draft|ready|blocked|done
Dokument bazowy: [sciezka albo nazwa dokumentu]
Data: YYYY-MM-DD

## 1. Cel Dla Cloud

Jednoznacznie opisz, co cloud ma dostarczyc.

## 2. Kontekst Bazowy

Wypisz dokumenty, ktore cloud musi przeczytac przed praca.

- `documentation/PROJECT_RULES.md`
- `documentation/[dokument-bazowy].md`
- inne wymagane pliki

## 3. Aurora First Anchors

Wypisz lokalne zrodla, od ktorych cloud musi zaczac. Jezeli nie ma dostepu do lokalnej sciezki, ma oznaczyc blocker zamiast zgadywac.

- `C:\Projects\New Folder\export\decompiled_all.c`
- `C:\Projects\New Folder\export\strings.tsv`
- `C:\Projects\New Folder\export\functions.tsv`
- `C:\Projects\New Folder\[inne-zrodlo]`

## 4. Zakres Pracy

Co cloud ma zrobic.

- punkt 1
- punkt 2
- punkt 3

## 5. Poza Zakresem

Czego cloud nie ma robic.

- nie edytowac `C:\Projects\aurora-web`, jezeli nie jest to jawnie wskazane
- nie promowac hipotez jako faktow
- nie uznawac testow za proof wizualny

## 6. Wymagany Format Odpowiedzi Cloud

Cloud ma odpowiedziec w takim formacie:

### Wynik

Krotki status: `done`, `partial`, `blocked` albo `needs-local-proof`.

### Co Dostarczono

Lista artefaktow, plikow, decyzji lub zmian.

### Dowody

Tylko konkretne dowody: sciezki, komendy, wyniki testow, screenshoty, JSON, MP4, PNG, albo cytowane kotwice dekompilacji.

### Blokery

Jezeli czegos nie da sie potwierdzic, wpisac blocker i brakujace zrodlo.

### Nastepny Krok Lokalny

Jedna konkretna akcja dla lokalnego watku/Codex Desktop.

## 7. TDD / Gate

Wypisz test lub gate, ktory musi powstac przed implementacja albo zostac odpalony po zmianie.

```text
command:
expected:
proof artifact:
```

## 8. Acceptance Criteria

Warunki akceptacji.

- kryterium 1
- kryterium 2
- kryterium 3

## 9. Notatki Dla Przekazania

Krotki prompt, ktory mozna wkleic do nowego watku cloud.
```

## Zasady Odpowiedzi Cloud

Cloud ma odpowiadac krotko i dowodowo. Bez rozwleklej narracji.

Preferowany ksztalt odpowiedzi:

```markdown
## Wynik

Status: partial

## Co Dostarczono

- ...

## Dowody

- `path`: ...
- `command`: ...
- `result`: ...

## Blokery

- ...

## Nastepny Krok Lokalny

- ...
```

## Reguly Krytyczne

- `*-cloud.md` zawsze jest suplementem do dokumentu bazowego.
- Aurora First nadal obowiazuje.
- Jezeli cloud nie ma dostepu do `C:\Projects\New Folder`, ma oznaczyc `blocked: missing-local-decompilation-access`.
- Jezeli cloud nie ma proofu runtime/wizualnego, ma oznaczyc `needs-local-proof`.
- Odpowiedzi lokalnego Codexa na pytania Cloud musza isc do `[temat]-odpowiedz-codex.md`, nie do `[temat]-codex.md`.
- Nie wolno zgadywac brakujacych pol GFF, 2DA, MDL ani runtime behavior.
- Nie wolno uznac samego `build/test passed` za proof wizualnej zgodnosci.
