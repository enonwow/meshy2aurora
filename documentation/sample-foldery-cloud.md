# sample-foldery-cloud.md
Data: 2026-07-08 | Status: OBOWIĄZUJĄCE
Struktura folderów próbek w repo meshy2aurora. Przepływ: **sample-2d** (obrazy koncepcyjne/referencyjne) → meshy.ai → **sample-3d** (modele GLB) → konwerter → dist (HAK).

## Struktura

```text
C:\Projects\meshy2aurora\
├── sample-2d\                      # obrazy 2D — wejście do generacji w meshy
│   ├── _reference\                 # screenshoty modeli NWN (bind pose z nwnexplorer/toolset)
│   │   └── c_kocrachn\
│   │       ├── front.png
│   │       ├── side.png
│   │       ├── quarter.png
│   │       └── manifest.yaml
│   └── <sample_id>\                # np. koc01 — jeden koncept = jeden katalog
│       ├── front.png               # wymagany
│       ├── side.png                # wymagany
│       ├── quarter.png             # zalecany
│       └── manifest.yaml
├── sample-3d\                      # modele 3D z meshy — wyjście generacji
│   └── <resref>\                   # np. m2a_koc01 — docelowy resref w Aurorze
│       ├── source.glb              # eksport z meshy (embedded textures)
│       ├── preview-front.png       # screenshot z meshy/viewera
│       ├── preview-side.png
│       └── manifest.yaml
└── dist\                           # wyniki konwertera (poza zakresem tego dokumentu)
```

## manifest.yaml — sample-2d

```yaml
sample_id: koc01
target_resref: m2a_koc01            # docelowy resref w sample-3d
reference_nwn_model: c_kocrachn     # model, którego proporcje/pozę naśladujemy
reference_images: "sample-2d/_reference/c_kocrachn/"
generator: "openai gpt-image | dall-e | screenshot | inne"
prompt: "<pełny prompt użyty do generacji obrazów>"
views: [front, side, quarter]
pose: "neutralna, zgodna z bind pose referencji"
created: 2026-07-08
author: Mateusz
```

## manifest.yaml — sample-3d

```yaml
resref: m2a_koc01
source_sample_2d: koc01             # powiązanie z sample-2d
reference_model: c_kocrachn
reference_supermodel: c_Horror
meshy:
  mode: image-to-3d
  input_images: ["sample-2d/koc01/front.png", "sample-2d/koc01/side.png"]
  textured: true
  rigging: false                    # ścieżka A
  remesh_target: 1500
  exported: GLB
  export_date: 2026-07-08
license_note: "meshy tier / prawa użycia"
checklist_passed: false             # Claude ustawia po weryfikacji (meshy-przygotowanie-modelu-cloud.md, krok 4)
```

## Źródła próbek (obowiązujące)

```yaml
zrodla:
  sample_2d: "OpenAI (model 5.5 / gpt-image) — obrazy koncepcyjne generowane wg promptów Codexa"
  sample_3d: "meshy.ai — modele generowane z obrazów sample-2d (Image-to-3D)"
  wyjatek: "_reference/ — screenshoty modeli NWN (nwnexplorer/toolset), nie generowane"
```

## Zasady

1. Nazwy katalogów: lowercase, bez spacji; `sample-3d/<resref>` zawsze z prefiksem `m2a_` i ≤16 znaków (limit ERF).
2. Każdy katalog MUSI mieć manifest.yaml — próbka bez manifestu nie istnieje dla pipeline'u.
3. `_reference` zawiera wyłącznie screenshoty do użytku lokalnego (assety NWN/CEP — nie publikować poza repo prywatnym; polityka: standalone-odpowiedz-codex.md Q5).
4. Jeden sample-2d może mieć wiele podejść w meshy — kolejne resrefy (m2a_koc01, m2a_koc02...) wskazują ten sam `source_sample_2d`.
5. GLB z meshy pobieramy natychmiast po generacji (assety wygasają ~3 dni).
