# Evidence Template

Ten folder przechowuje append-only dowody dla jednego etapu implementacji. Tworzymy jeden plik na etap, na przyklad `M1A-evidence.md`, i dopisujemy do niego kolejne proby w porzadku czasu.

Nie kasuj historii ani zamknietych problemow z evidence. Biezacy stan nalezy do `documentation/orchestrator-state.yaml`.

## Szablon wpisu

````markdown
## <attempt-id> - <YYYY-MM-DD>

Status: IN_PROGRESS | VERIFYING | DONE | BLOCKED
Owner: <person-or-agent>
Stage: <stage-id>

### Cel proby

<jedno sprawdzalne zdanie>

### Aurora First / provenance

- <exact local documentation or decompilation path>
- <synthetic fixture or runtime proof path>

### Zmienione pliki

- `<path>` - <purpose>

### Weryfikacja

| Command or action | Expected | Actual | Status |
|---|---|---|---|
| `<command>` | <expected> | <actual> | PASS/FAIL |

### Problemy i bledy

```yaml
current_problems: []
bugs:
  - id: "<bug-id>"
    severity: "P0 | P1 | P2 | P3"
    status: "OPEN | FIXED | QUARANTINED"
    repro: "<exact reproducible action>"
    expected: "<expected behavior>"
    actual: "<observed behavior>"
    next_action: "<smallest safe action>"
```

### Evidence artifacts

- `<absolute-or-repo-relative-path>` - <what it proves>

### Nastepny krok

<one smallest safe action, or exact blocker/decision needed>
````

## Reguly

- Dowod ma wskazywac dokladna sciezke, komende albo screenshot, nie ogolny opis.
- `BUG` oznacza dzialajaca funkcje zachowujaca sie niezgodnie z kontraktem. Brak implementacji jest `current_problem`.
- Status `DONE` wymaga przejscia Definition of Done z planu i aktualizacji `orchestrator-state.yaml`.
- Nie wpisuj retail/CEP payloadow do repo ani do evidence. Dopuszczalne sa hashe, resrefy, rozmiary, offsety i raporty strukturalne.
