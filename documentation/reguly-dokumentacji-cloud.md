# reguly-dokumentacji-cloud.md

Reguła współpracy dokumentacyjnej Claude ↔ Codex w projekcie meshy2aurora.
Obowiązuje w folderze: `C:\Projects\meshy2aurora\documentation\`

## 1. Nazewnictwo plików

| Typ | Wzorzec nazwy | Autor |
|---|---|---|
| Dokumentacja Claude (specyfikacje, decyzje, notatki implementacyjne) | `[temat]-cloud.md` | Claude |
| Pytania Claude do Codexa | `[temat]-pytania-cloud.md` | Claude |
| **Odpowiedź Codexa na pytania** | `[temat]-odpowiedz-codex.md` | Codex |
| Dokumentacja własna Codexa | `[temat]-codex.md` | Codex |

`[temat]` jest identyczny w parze pytania↔odpowiedź. Przykład: na `aurora-animacje-pytania-cloud.md` odpowiedź ma nazwę `aurora-animacje-odpowiedz-codex.md`. Małe litery, myślniki zamiast spacji, bez polskich znaków w nazwach.

## 2. Struktura pliku z pytaniami (`*-pytania-cloud.md`)

```markdown
# [temat]-pytania-cloud.md
Data: RRRR-MM-DD | Status: OTWARTE | Priorytet: BLOKUJĄCE / NORMALNE

## Q1: [krótki tytuł pytania]
Treść pytania. Jedno zagadnienie na jedno Q.

## Q2: ...
```

Każde pytanie ma stały identyfikator `Q<n>` — nie zmienia się nigdy.

## 3. Struktura odpowiedzi Codexa (`*-odpowiedz-codex.md`)

```markdown
# [temat]-odpowiedz-codex.md
Data: RRRR-MM-DD | Odpowiada na: [temat]-pytania-cloud.md

## Q1: [powtórzony tytuł]
Status: POTWIERDZONE (źródło: ścieżka/plik) | HIPOTEZA | NIE WIEM
Odpowiedź.

## Q2: ...
```

Wymagania dla odpowiedzi:
- Odpowiedź na każde Q osobno, pod jego identyfikatorem; bez pomijania — jeśli brak danych, wpisać `NIE WIEM`.
- Fakty konkretnie: pełne ścieżki jako kod, limity jako liczby.
- Dane maszynowe (hierarchie kości, listy animacji, mapowania nazw) w blokach ```yaml lub ```json.
- `POTWIERDZONE` tylko z odwołaniem do źródła (plik w repo / dekompilacji).

## 4. Cykl życia

1. Claude tworzy `[temat]-pytania-cloud.md` (Status: OTWARTE).
2. Codex zapisuje `[temat]-odpowiedz-codex.md` w tym folderze.
3. Claude po przeczytaniu zmienia status pytań na ZAMKNIĘTE lub dopisuje `## Q<n+1>` (doprecyzowania) do tego samego pliku pytań.
4. Pliki są append-only dla drugiej strony: Codex nie edytuje plików `*-cloud.md`, Claude nie edytuje plików `*-codex.md`.

## 5. Stan bieżący

Aktualizacja 2026-07-09: ponizszy pierwszy zestaw pytan jest historyczny. Aktualny kierunek i kolejka po D7-D8 sa w `decyzje-i-zadania-cloud.md`, `audyt-dokumentacji-plan-2026-07-09-codex.md` i `architektura-meshy2aurora-codex.md`. `aurora-web` jest reference-only.

Pierwszy zestaw pytań znajduje się w `wymagania-startowe-cloud.md` (sekcje 1–5). Oczekiwane odpowiedzi:

- `aurora-pipeline-odpowiedz-codex.md` (BLOKUJĄCE)
- `aurora-animacje-odpowiedz-codex.md` (BLOKUJĄCE)
- `meshy-input-odpowiedz-codex.md`
- `pliki-referencyjne-odpowiedz-codex.md`
- `srodowisko-zakres-odpowiedz-codex.md`
