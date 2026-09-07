# Creature held item V9 — ready for owner proof — 2026-08-17

1. Plik modułu: `m2aweapdemo9.mod`.
2. Nazwa modułu: `Meshy2Aurora procedural humanoid item placement`.
3. Area: `Meshy2Aurora procedural humanoid item placement area`.

Status: `ready_for_owner_proof`.

Agent nie uruchamiał, nie przejmował ani nie kontrolował Toolsetu lub NWN.
Dokładne MOD/HAK zostały przygotowane i zainstalowane dla właścicielskiego
proofu.

## Co naprawia V9

- demonstrator używa `void-crystal-knight-h1-v1/source.glb`, a nie Fogbounda
  używanego wcześniej jako podmiot macierzy facing;
- przód źródła pozostaje jawnie `POSITIVE_Z`, bez eksperymentalnego wariantu;
- prawa ręka ma natywny slot `16` i dokładny stockowy `nw_wswss001` z zasobów
  bazowych NWN (`UTI/2025`);
- MOD zawiera zero modułowych UTI; parser-visible własny UTI z V8 nie jest już
  traktowany jako proof runtime resolution;
- lokalny roll prawej ręki wynosi `+90°`, zgodnie z wcześniej przygotowaną
  korektą ostrza renderowanego krawędzią;
- authoring animacji nie został zmieniony.

## Offline readback

- triangles: `19704` przed i po;
- binary MDL animations: `42`;
- `MODELTYPE=L`, Appearance row `15100`;
- `rhand` ma rodzica `RightHand`, `lhand` ma rodzica `LeftHand`;
- oba hooki występują w `42/42` klipach;
- GIT i UTC wskazują `nw_wswss001` w prawym slocie;
- raport zasobu: `NWN_BASE_GAME`, type `2025`;
- report gripu: `AUTO_PLUS_OFFSETS`, right roll `90`, pitch `0`, yaw `0`;
- `animationChangedForHeldItem=false`.

## Exact lineage

- MOD: 15 353 bytes, SHA-256
  `da0a5872a3c82e7701696baf789c54a7b6b181e35099c1754c4ce475d2ff3d18`;
- HAK: 22 050 805 bytes, SHA-256
  `f3f8691f1a00fd5ef6798cd7e833c540d4852c9ced8ff34c0e9f9eb692a89e1b`;
- MDL: 2 566 248 bytes, SHA-256
  `d7505dbc60944e69058123c4b1dfd9f66ea4802417383f6209e7632503c19c57`;
- TGA: 12 582 956 bytes, SHA-256
  `5101483b93b6eb4e0aa39cc705dcbed9de1a70c1661f85eacffcb58793025e8a`;
- `appearance.2da`: 6 901 345 bytes, SHA-256
  `3fde7fafa7e232e1b1cd0ba0fa9383612ef6d9a5658f00909d7ccd39ed9a1f51`.

Pełny packet:
`proof-output/creature-held-item-v9/handoff.json`, SHA-256
`0446bd1b30a230df58aa27c0c238cc9f93e74c4651851318ff2ade23371015c1`.

## Bramki implementacyjne

- Core lib: `116 passed`, `3 ignored` lokalne korpusy;
- Core real product/demo integration: `1 passed`;
- WASM lib: `45 passed`;
- Studio: `295 passed`;
- real browser Worker/WASM held-item integration: `1 passed`;
- TypeScript typecheck: PASS;
- production WASM/Vite build: PASS;
- `cargo fmt --all -- --check`: PASS;
- canonical workspace i Meshy asset layout: PASS.

## Instalacja

Oba cele były nieobecne. Kopiowanie wykonano bez overwrite, a potem
potwierdzono byte-identical SHA-256:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aweapdemo9.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aweaphak9.hak`.

## Właścicielski test NWN

Uruchom dokładnie `m2aweapdemo9.mod`, Area
`Meshy2Aurora procedural humanoid item placement area`. Jedyny Creature to
`Meshy procedural humanoid with held item`, template `m2awrhand9`, ustawiony
bezpośrednio przed graczem w `[10.0, 14.5, 0.0]`.

Kryterium: zwykły Void Crystal Knight jest zwrócony prawidłowym przodem, a
stockowy krótki miecz jest widoczny w prawej dłoni i nie jest pokazany samą
krawędzią. Animacja pozostaje poza zakresem tego proofu.
