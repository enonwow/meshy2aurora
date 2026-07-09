# konwersja-meshy-analiza-cloud.md

Status 2026-07-09: AKTYWNA ANALIZA TRANSFORMACJI z korekta D7-D9. Finalny output to binary MDL/MDX policy + 2DA + HAK; wzmianki o v13/ingest/CDP sa historyczne i nie sa gates wykonawczymi.
Data: 2026-07-08 | Status: ANALIZA (uzupełnia kierunek-implementacji-cloud.md)

Co dokładnie musi się stać z modelem meshy, żeby stał się poprawnym modelem Aurory. Inwentarz transformacji, potrzebnych danych i ryzyk. Statusy: [P] potwierdzone kotwicą, [H] hipoteza, [?] pytanie w `konwersja-meshy-pytania-cloud.md`.

## Różnice źródło → cel

```yaml
meshy_output_vs_aurora_target:
  coordinate_system:
    meshy_gltf: "Y-up, forward +Z, prawoskrętny [P: spec glTF + docs meshy]"
    aurora_mdl: "Z-up [P: mapowanie x,y,z→x,z,y w placeableThreeAssetLoader 5323-5332]; oś forward [?Q1]"
    needed: "rotacja osi przy imporcie GLB→przestrzeń MDL"
  units_scale:
    meshy: "metry (height_meters, default 1.7)"
    aurora: "jednostki modelu Aurora/NWN; wartosci z aurora-web sa tylko reference-only, 1 unit = 1 m? [?Q2]"
    needed: "normalizacja bbox do wymiarów referencji + poprawny animationscale"
  mesh_budget:
    meshy: "dziesiątki–setki tysięcy trójkątów (limit riggingu 300k)"
    aurora_retail: "creature ~1–5k trójkątów [H; ?Q3 retail budżet]; c_kocrachn: 24 mesh nodes, skin ~185/59/59 verts [P]"
    needed: "decymacja: meshy Remesh API (target polycount) i/lub decymacja lokalna; podział na mesh nodes"
  topology:
    meshy: "jeden mesh, triangulowany, może mieć luźne shelle"
    aurora: "verts/faces/tverts per node; faces niosą smoothing group i material id [P: format w aurora-mdl-format-codex.md]"
    needed: "triangulacja (już jest), scalenie duplikatów vertów, normalne→smoothing groups [?Q4]"
  textures:
    meshy: "PBR: basecolor + metallic/roughness/normal, często 1–4k"
    aurora: "1 diffuse bitmap (TGA) + opcjonalne TXI/MTR [P: Q7 poprzedniej rundy]"
    needed: "wybór/bake basecolor → TGA, rozmiar power-of-2 [?Q5 max rozmiar], resref <=16 lowercase"
  uv:
    meshy: "TEXCOORD_0 w 0-1 [P]"
    aurora: "tverts + indeksy w faces [P]"
    needed: "przepisanie 1:1; flip V? [?Q6]"
  skeleton_weights:
    sciezka_A: "brak rigu meshy → szkielet+wagi z referencji"
    aurora: "weights max 4 wpływy, suma 1.0, nazwy kości z hierarchy [P]"
    needed: "transfer wag nearest-surface Z OGRANICZENIEM do influencingBoneNames per skin node referencji [P: struktura c_kocrachn]"
  pivot_pose:
    meshy: "pivot dowolny, poza generacji zbliżona do promptu"
    aurora: "pozycje nodów hierarchy = bind pose; animacje supermodelu zakładają tę pozę [P]"
    needed: "wyrównanie pozy siatki meshy do bind pose referencji (na MVP: dyscyplina generacji + kontrola wizualna; auto-fit później)"
```

## Pipeline konwersji — ścieżka A (etapy przetwarzania)

```yaml
stages:
  s1_ingest: "parse GLB (@gltf-transform/core), walidacja: 1 mesh, TEXCOORD_0, tekstura basecolor obecna"
  s2_reference: "load referencyjny binary MDL przez wlasny parser albo fixture syntetyczna; source of truth: hierarchy, weights, bind pose, classification/metadata, animationscale jesli potwierdzone"
  s3_axes_scale: "rotacja Y-up→Z-up, orientacja forward, skala bbox→bbox referencji (per-wymiar sanity check ±20%)"
  s4_decimate: "redukcja do budżetu [?Q3]; preferencja: meshy Remesh API przed pobraniem; fallback lokalny"
  s5_align: "wyrównanie do bind pose referencji: translacja pivotu, raport odchyleń (heatmapa dystansów powierzchni)"
  s6_segment_or_skin: "przypisanie geometrii do skin nodes referencji (wg najbliższych powierzchni segmentów referencji); transfer wag max4 + normalizacja"
  s7_texture: "basecolor → TGA (resize power-of-2), resref naming"
  s8_emit_mdl: "binary MDL writer: hierarchy referencji/modelu + nowa geometria + skin + animacje; ASCII dump tylko opcjonalny debug snapshot"
  s9_package: "appearance.2da row + HAK (ErfHakWriter)"
  s10_verify: "file gates + generated HAK/module + manualny proof NWN EE Toolset/gra; aurora-web tylko opcjonalny zewnetrzny konsument HAK-a"
gates_tdd:
  - "s1: fixture z realnego GLB meshy (do dostarczenia, P5) + syntetyczny minimal GLB"
  - "s3: golden test transformacji osi na znanych punktach"
  - "s6: suma wag=1.0, max4, tylko kości z influencingBoneNames"
  - "s8: binary writer structural golden snapshot; env-gated read-only reference c_kocrachn tylko do porownan struktury"
```

## Czego potrzebujemy (skonsolidowane)

1. **Dane**: 1 realny eksport GLB/FBX z meshy (nawet darmowy/testowy — do fixture s1); retail appearance.2da albo reczny read-only export; odpowiedzi Q1–Q6 (nowe pytania).
2. **Decyzje**: budżet trójkątów i rozmiar tekstury dla MVP (po Q3/Q5); akceptacja P4.
3. **Narzędzia**: @gltf-transform/core (parse/decymacja GLB), sharp lub podobne (TGA/resize), wlasny binary MDL/MDX writer, wlasny 2DA writer, wlasny ERF/HAK writer; meshy Remesh API (klucz, P5).
4. **Ryzyka główne**: (a) niedopasowanie pozy meshy↔bind pose referencji — najsłabszy punkt ścieżki A, mitygacja: obraz referencyjny bind pose w promptcie + raport s5; (b) decymacja niszcząca UV — mitygacja: Remesh API przed teksturowaniem po stronie meshy; (c) smoothing groups z normalnych — jakość cieniowania w grze.
