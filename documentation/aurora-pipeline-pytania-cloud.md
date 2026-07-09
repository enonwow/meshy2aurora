# aurora-pipeline-pytania-cloud.md
Data: 2026-07-08 | Status: ZAMKNIĘTE (odpowiedź: aurora-pipeline-odpowiedz-codex.md, 2026-07-08) | Priorytet: BLOKUJĄCE

## Q1: Format konsumowany przez aurora-web
Jaki dokładnie format modeli konsumuje aurora-web: gotowe GLB w blob storage (SeaweedFS / `module-blob-mirror`), czy MDL konwertowane w locie przez backend? Podaj ścieżkę do kodu, który ładuje model do renderera.

## Q2: Punkt wpięcia meshy2aurora
Gdzie w pipeline ma się wpinać meshy2aurora: meshy → (co?) → blob storage → renderer? Które etapy już istnieją i można je reużyć (np. istniejący konwerter MDL→GLB)?

## Q3: Konwencje nazewnicze assetów
Jakie są konwencje nazewnicze assetów w blob storage (np. `zcp_horse_cart`) i struktura katalogów mirrora?

## Q4: Układ współrzędnych i skala
Jaki układ współrzędnych (która oś w górę), skala i jednostki oczekuje renderer aurora-web? Podaj źródło w kodzie.
