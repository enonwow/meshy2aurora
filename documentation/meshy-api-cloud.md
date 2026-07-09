# meshy-api-cloud.md
Data: 2026-07-08 | Status: GOTOWE | Autor: Claude
Źródła (Internet, uzupełnienie — brak lokalnego źródła): https://docs.meshy.ai/en/api/rigging, https://docs.meshy.ai/en/api/animation (stan na 2026-07-08).

## Najważniejsze fakty dla meshy2aurora

### Rigging API (`POST /openapi/v1/rigging`)

```yaml
rigging:
  input: "textured GLB (model_url lub input_task_id)"
  limits:
    max_faces: 300000          # powyżej: najpierw Remesh API
    facing: "+Z (standard glTF forward); inne osie = fail pose estimation"
    height_meters: "parametr skali, default 1.7"
  ONLY_HUMANOID: true          # KRYTYCZNE OGRANICZENIE
  not_supported:
    - "untextured meshes"
    - "non-humanoid assets (czworonogi, potwory, smoki...)"
    - "humanoidy z niejasną strukturą kończyn"
  output:
    - "rigged_character_glb_url / fbx_url"
    - "basic_animations: walking + running (GLB/FBX, wersje withSkin i armature-only)"
```

### Animation API (`POST /openapi/v1/animations`)

```yaml
animation:
  input: "rig_task_id (ukończony rigging) + action_id z biblioteki animacji"
  library: "https://docs.meshy.ai/api/animation-library (katalog gotowych akcji po ID)"
  one_clip_per_task: true      # każda animacja = osobny task i osobny plik GLB/FBX
  post_process:
    change_fps: [24, 25, 30, 60]
    extract_armature: "GLB/FBX samego szkieletu"
  note: "animacje z biblioteki meshy (mocap-style), nie custom"
asset_retention:
  expires_at: "assety taskow wygasają (~3 dni) — pobierać od razu"
```

## Konsekwencje dla kierunku projektu (WAŻNE)

1. **Humanoidy**: pełny pipeline meshy (mesh+rig+animacje) działa. Szkielet meshy = standardowy humanoid; animacje z biblioteki meshy trzeba zmapować na nazwy NWN (walk, run, pause1, 1hslashl...) — każdy klip to osobny GLB do scalenia.
2. **Nie-humanoidy** (potwory, czworonogi): meshy dostarczy TYLKO siatkę. Rig i animacje musimy zrobić sami — tu wraca strategia B (transfer wag z referencyjnego creature Aurory + jego natywne animacje przez supermodel) albo rig ręczny w Blenderze.
3. Pipeline konwertera musi więc mieć **dwie ścieżki**:
   - `humanoid`: meshy rig+anim → mapowanie kości/animacji → MDL z własnymi animacjami (lub na szkielet NWN part-based?),
   - `creature`: siatka meshy → osadzenie na szkielecie referencyjnym Aurory → animacje z supermodelu.
4. Wejście do riggingu wymaga poprawnej orientacji (+Z) i tekstury — guideline generacji musi to wymuszać.
5. Automatyzacja: całość meshy jest dostępna przez REST API (klucz API Mateusza) — konwerter może sam odpalać rigging/animation tasks i pobierać wyniki przed wygaśnięciem.

## Do decyzji (aktualizuje P2/P3 w decyzje-i-zadania-cloud.md)

- P2/P3: pierwszy model — humanoid (pełny pipeline meshy, więcej mapowania po naszej stronie) czy nie-humanoid (prostszy wariant strategii B, bez animacji meshy)?
- Czy mamy klucz API meshy i budżet kredytów na taski rigging/animation?
