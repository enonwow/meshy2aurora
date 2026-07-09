# meshy-przygotowanie-modelu-cloud.md
Data: 2026-07-08 | Status: INSTRUKCJA (wykonawca: Mateusz w meshy.ai; wsparcie: Claude)
Podstawa: meshy-api-cloud.md, konwersja-meshy-analiza-cloud.md, kierunek-implementacji-cloud.md (ścieżka A), D7.

## Cel

Pierwszy poprawnie przygotowany model z meshy: potwór o proporcjach c_kocrachn (ścieżka A — bez rigu meshy), gotowy jako wejście s1 konwertera i fixture testów.

## Krok 0 — obraz referencyjny bind pose (przed generacją!)

Najważniejszy element przygotowania: model meshy musi mieć pozę i proporcje zgodne z bind pose c_kocrachn, inaczej transfer wag da złe deformacje.

```yaml
referencja:
  model: c_kocrachn (CEP, cep3_core1.hak)
  jak_zdobyc_obraz:
    opcja_1: "nwnexplorer.exe (C:\\Program Files (x86)\\Steam\\...\\bin\\win32) → otwórz cep3_core1.hak → c_kocrachn.mdl → podgląd 3D → screenshot z 3 ujęć: przód, bok, 3/4"
    opcja_2: "Toolset → paleta creature z appearance Kocrachn → screenshot"
  wymagane_ujecia: ["front", "profil", "3/4"]
  zapis: "C:\\Projects\\meshy2aurora\\sample-2d\\_reference\\c_kocrachn\\{front|side|quarter}.png (struktura: sample-foldery-cloud.md)"
```

## Krok 1 — generacja w meshy

```yaml
generacja:
  tryb_rekomendowany: "Image to 3D z obrazami z sample-2d\\koc01\\ (concept 2D generowany w OpenAI wg sample-2d-generacja-cloud.md)"
  tryb_alternatywny: "Text to 3D z opisem pozy"
  wytyczne_promptu:
    - "creature w POZIE NEUTRALNEJ zgodnej z referencją: cztery kończyny na ziemi / rozstaw jak c_kocrachn"
    - "symetria lewo-prawo"
    - "zwarta bryła: bez cienkich odstających elementów (anteny/kolce = ryzyko przy decymacji i skinningu)"
    - "bez broni/przedmiotów trzymanych"
  wyglad: "dowolny potwór — proporcje ważniejsze niż estetyka pierwszego modelu"
  texturing: "WŁĄCZONY (textured; sama geometria bez tekstury jest bezużyteczna dla pipeline)"
  rigging: "WYŁĄCZONY (non-humanoid — auto-rig i tak nie zadziała; ścieżka A go nie używa)"
```

## Krok 2 — remesh przed eksportem

```yaml
remesh:
  target_polycount: 1500        # budżet z konwersja-meshy-odpowiedz-codex.md (retail: 424-1343 tri)
  topologia: "triangle"
  uwaga: "remesh PO teksturowaniu może zepsuć UV — jeśli webapp pozwala, ustaw target polycount przed/przy teksturowaniu; jeśli wynik ma zniszczone UV, powtórz z remesh przed texture"
  akceptowalne_widelki: [1000, 3000]   # >3000 = powtórz remesh; konwerter i tak przytnie dalej
```

## Krok 3 — eksport i zapis

```yaml
eksport:
  format: "GLB (tekstury embedded)"
  pobrac_natychmiast: true      # assety meshy wygasają (~3 dni)
  struktura_zapisu:
    katalog: "C:\\Projects\\meshy2aurora\\sample-3d\\m2a_koc01\\ (struktura i manifest: sample-foldery-cloud.md)"
    pliki:
      - "source.glb"
      - "manifest.yaml"
      - "preview-{front|side}.png (screenshoty z meshy)"
  manifest_yaml_wzor: |
    resref: m2a_koc01
    reference_model: c_kocrachn
    reference_supermodel: c_Horror
    meshy:
      mode: image-to-3d | text-to-3d
      prompt: "<pełny prompt>"
      source_image: "<plik/brak>"
      textured: true
      rigging: false
      remesh_target: 1500
      exported: GLB
      export_date: 2026-07-08
    license_note: "meshy account tier / usage rights"
```

## Krok 4 — checklist akceptacji (przed użyciem w konwerterze)

```yaml
checklist:
  - "jedna spójna siatka (bez rozsypanych fragmentów w powietrzu)"
  - "tekstura basecolor widoczna w podglądzie GLB (np. https://gltf-viewer.donmccurdy.com lub Blender 5.1)"
  - "poza zgodna z referencją: kończyny w podobnym rozstawie co c_kocrachn na screenshotach"
  - "proporcje: stosunek długości kończyn/korpusu z grubsza jak w referencji (±30%)"
  - "trójkąty w widełkach 1000-3000"
  - "manifest.yaml wypełniony"
niespelnienie: "regeneracja tańsza niż walka konwertera ze złym wejściem"
```

## Status wykonania

- [ ] Screenshoty referencyjne c_kocrachn (Mateusz, ~5 min w nwnexplorer)
- [ ] Generacja + remesh + eksport (Mateusz w meshy.ai)
- [ ] Weryfikacja checklisty (Claude po otrzymaniu plików w samples/)
