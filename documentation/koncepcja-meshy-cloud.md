# koncepcja-meshy-cloud.md

Status 2026-07-09: HISTORYCZNE / CZESCIOWO SUPERSEDED. Ten dokument opisuje stary wariant MVP oparty o `aurora-web`/GLB/CDP. Aktualny cel to natywny content Aurora/NWN: `MDL + 2DA + HAK`, projekt standalone, proof przez NWN EE.
Data: 2026-07-08 | Status: PROPOZYCJA (do akceptacji: Mateusz + weryfikacja Codex)

Podejście do generowania modeli w meshy.ai tak, żeby były konwertowalne do Aurory. Decyzje z 2026-07-08: MVP = creature (direct model), brak plików testowych — projektujemy pipeline od strony generacji.

## Cel MVP

Jeden creature wygenerowany w meshy.ai, osadzony na istniejącym szkielecie Aurory (wzorzec: `c_kocrachn` + supermodel `c_horror`), odtwarzający natywne klipy NWN w aurora-web, z przechodzącym proofem CDP (`summary failedCount == 0`).

## Kluczowy problem projektowy

Meshy generuje siatkę (i opcjonalnie własny auto-rig), a Aurora wymaga konkretnych nazw nodów i hierarchii z supermodel chain. Animacji meshy nie używamy. Trzy możliwe strategie osadzenia siatki na szkielecie Aurory:

### Strategia A — auto-rig meshy + mapowanie kości (retarget bind pose)
Meshy rig (humanoid standard) → tabela mapowań `meshy_bone → aurora_node` → przepięcie skin weights na docelowe nazwy → korekta bind pose.
Plusy: automatyzacja skinningu. Minusy: rig meshy jest humanoidalny — nie pasuje do creature typu c_horror (4 łapy, pazury); mapowanie 1:1 często niemożliwe.

### Strategia B — surowa siatka + transfer wag z modelu referencyjnego (REKOMENDOWANA dla MVP)
Generujemy w meshy TYLKO siatkę, w pozie zgodnej z bind pose docelowego szkieletu (kontrola pozy promptem/obrazem referencyjnym). Konwerter: wczytuje szkielet z referencyjnego GLB (np. `c_kocrachn.glb` z mirrora), dopasowuje wymiary (bbox → skala), transferuje skin weights metodą nearest-surface z siatki referencyjnej na siatkę meshy (limit 4 wpływy/wierzchołek — zgodnie z potwierdzonym limitem konwertera).
Plusy: zero zależności od rigu meshy, pełna zgodność nazw nodów od razu. Minusy: wymaga zbliżonych proporcji i pozy generowanego modelu do referencyjnego.

### Strategia C — rig ręczny w Blenderze
Fallback, poza zakresem automatyzacji MVP.

## Wymagania dla generacji w meshy (draft guideline)

```yaml
meshy_generation_guideline:
  export_format: "GLB"
  rigging: false           # strategia B: tylko siatka
  pose: "zgodna z bind pose docelowego supermodelu (obraz referencyjny z renderu modelu bazowego)"
  topology: "quad/tri bez przecinających się shelli; jeden mesh lub łatwe do scalenia"
  textures: "embedded w GLB; diffuse wystarczy na MVP"
  scale_hint: "dowolna — konwerter normalizuje do bbox referencji"
  per_model_metadata:
    - "docelowy resref (np. zcp_meshy_horror01)"
    - "docelowy szkielet referencyjny (resref, np. c_kocrachn)"
    - "prompt/obraz źródłowy"
```

## Szkic pipeline konwertera (Node/TS, CLI — zgodnie z decyzją Codexa)

1. `meshy2aurora convert --input meshy.glb --type creature --reference c_kocrachn --resref zcp_x --out dist/zcp_x`
2. Wczytaj GLB meshy + referencyjny GLB z mirrora.
3. Normalizacja: osie/skala do przestrzeni referencji (mapowanie x,y,z→x,z,y po stronie renderera jest znane — konwerter trzyma konwencję źródła Aurora).
4. Transfer wag: nearest-surface z referencji, max 4 wpływy, nazwy nodów = nazwy Aurora.
5. Emisja: `zcp_x.glb` (+ `manifest.json` z metadanymi supermodel/animationscale zgodnymi z extras konwertera v13).
6. Wrzut do local mirror (`__aurora/derived/models/...`) i proof CDP klipów: `cpause1, cwalk, crun, ca1slashl, cdamagel, cdead`.

## Pytania otwarte

Wysłane do Codexa w `koncepcja-meshy-pytania-cloud.md` (weryfikacja wykonalności B względem formatu extras GLB v13 i loadera).
Do Mateusza: akceptacja strategii B i pierwszego celu (jaki creature generujemy jako pierwszy — sugestia: wariant potwora zbliżony proporcjami do c_kocrachn).
