# sample-2d-generacja-cloud.md
Data: 2026-07-08 | Status: OTWARTE | Wykonawca: Codex (przygotowanie promptów) + Mateusz (generacja w OpenAI)
Podstawa: sample-foldery-cloud.md, meshy-przygotowanie-modelu-cloud.md, konwersja-meshy-analiza-cloud.md.

## Zadanie dla Codexa

Przygotuj prompty do wygenerowania obrazów 2D (OpenAI: gpt-image/DALL·E) dla pierwszej próbki `koc01` i poproś Mateusza o ich wygenerowanie. Wynik trafia do `C:\Projects\meshy2aurora\sample-2d\koc01\`.

### Krok 1 — referencja (przed promptami)
Poproś Mateusza o screenshoty c_kocrachn z nwnexplorer (front/side/quarter) → `sample-2d\_reference\c_kocrachn\`. Na ich podstawie opisz w promptach faktyczne proporcje (długość kończyn vs korpus, wysokość, sylwetka) — nie zgaduj z nazwy.

### Krok 2 — wymagania wobec obrazów 2D (kontrakt dla promptów)

```yaml
image_requirements:
  views: [front, side, quarter]        # 3 osobne obrazy, TEN SAM potwór
  pose: "neutralna A-pose zwierzęca — wszystkie kończyny na ziemi, rozstaw jak na screenshotach referencji; ZERO dynamicznych póz"
  consistency: "identyczny design na wszystkich ujęciach (użyj w promptcie tego samego opisu postaci słowo w słowo)"
  background: "jednolite jasnoszare/białe, bez sceny, bez cieni rzucanych na otoczenie"
  framing: "cała sylwetka w kadrze, kamera na wysokości korpusu, obiektyw ~50mm (bez dystorsji)"
  style: "concept art / creature sheet, realistyczne bryły i materiały (meshy lepiej czyta formy niż płaskie ilustracje)"
  avoid:
    - "cienkie odstające elementy (anteny, długie kolce, wąsy)"
    - "trzymane przedmioty/broń"
    - "skrzydła rozpostarte / elementy przesłaniające korpus"
    - "asymetria lewo-prawo"
    - "tekst, znaki wodne, ramki"
  resolution: ">=1024x1024"
```

### Krok 3 — format dostawy promptów
Zapisz prompty w `sample-2d-prompty-codex.md`: po jednym promptcie na ujęcie (front/side/quarter) + wspólny opis potwora, gotowe do wklejenia przez Mateusza w OpenAI. Dodaj instrukcję zapisu plików wg `sample-foldery-cloud.md` (nazwy: front.png, side.png, quarter.png + manifest.yaml z wypełnionym polem prompt).

### Krok 4 — po wygenerowaniu
Mateusz wrzuca obrazy do `sample-2d\koc01\`. Claude weryfikuje zgodność z kontraktem (poza/spójność/tło) i dopiero wtedy obrazy idą do meshy (Image-to-3D, wg meshy-przygotowanie-modelu-cloud.md).

## Kryterium ukończenia
`sample-2d\_reference\c_kocrachn\` (3 screenshoty + manifest) oraz `sample-2d\koc01\` (3 obrazy + manifest z promptem) istnieją i przechodzą weryfikację.
