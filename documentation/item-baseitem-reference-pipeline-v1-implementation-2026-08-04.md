# Implementacja pipeline BaseItem + model referencyjny V1

Data: 2026-08-04
Status: `IMPLEMENTED_VERTICAL_SLICE / READY_FOR_OWNER_PROOF`

## Wynik

Pierwszy pełny przypadek `BaseItem=1 / WSwLs / ModelType 2` nie używa już
zgadywanych długości i przesuwanego kursora jako kontraktu świata. Pozycja,
skala, wspólny origin oraz zakres każdego slotu wynikają z dokładnych modeli
referencyjnych wybranych w jednym kontekście zasobów.

## Zaimplementowane elementy

1. Read-only resolver KEY/BIF lokalizuje i odczytuje dokładny zasób na podstawie
   `nwn_base.key`, bez kopiowania payloadów retail do repozytorium lub HAK-a.
2. `ResourceContextV1` hashuje providery i rozwiązuje efektywny
   `baseitems.2da`. Jeden custom provider może zasłonić Vanilla; kolizja w
   więcej niż jednym custom providerze jest odrzucana jako nieudowodniona
   kolejność, a nie rozstrzygana przez zgadywanie.
3. `ItemAttachmentProfileV1` jest wyprowadzany przez własny binary MDL reader.
   Profil wiąże context, tabelę, BaseItem, wariant referencyjny oraz SHA-256
   każdego Bottom/Middle/Top.
4. `ITEM_REFERENCE_SLOT_FRAME_FIT_V1` obraca Meshy do ramy Aurora Y/Z/X i
   dopasowuje każdy part do zakresu oraz środka jego natywnego slotu, zachowując
   wspólny origin.
5. Writer wypala transform montażowy do child Trimesh. Root pozostaje bez
   kontrolerów, zgodnie ze sposobem składania ModelType 2 przez Aurorę.
6. Walidacja obejmuje fit/profile identity, connector overlap, geometryczne
   seamy, binary readback, kolejność append Bottom/Middle/Top, wszystkie cztery
   warianty koloru, UTI, budżet trójkątów oraz pakiet HAK/MOD.
7. Studio wymaga dla ModelType 2 dokładnych referencyjnych MDL, pokazuje hash
   profilu, origin oraz per-slot ranges i blokuje build bez profilu oraz raportu
   fit V4.
8. Kadrowanie ikony ma osobny deterministyczny presentation fit. Jego transform
   nie jest używany przez MDL, seamy, UTI ani equipped attachment, dzięki czemu
   poprawa ikony nie może przesunąć chwytu modelu świata.

## Referencja retail pierwszego profilu

- `nwn_base.key` SHA-256:
  `09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935`;
- `baseitems.2da` SHA-256:
  `3fbdcd012b55f0b869c6324cc7d4ece44ed63a78f00e7889e7acefac28c8fdf4`;
- Bottom `wswls_b_023`: Y `[-0.2025382, 0.0841520]`;
- Middle `wswls_m_063`: Y `[0.042125102, 0.1570407]`;
- Top `wswls_t_023`: Y `[0.1283190, 0.9313860]`;
- profile SHA-256:
  `877dc68ab31c14ca1d87684d86442fa9c7cef0356d6d2e426e769e2e3872a947`;
- fit SHA-256:
  `ac087a33ebc4949e1c19f7db20eb801f72ee5edc85d0efcd58685038fe23d3f4`.

## Granice

- Exact V8 jest gotowy do proofu właściciela, ale wynik wizualny nie jest
  deklarowany przez agenta.
- Kolejność dwóch custom HAK-ów zawierających ten sam resref pozostaje
  fail-closed do potwierdzenia w dekompilacji Aurory.
- ModelType 0/1, pozostałe ModelType 2, CAPART, robe i cloak wymagają własnych
  route/profile; nie dziedziczą profilu dłoni WSwLs.
- Referencyjny payload retail jest czytany in-place. W outputcie pozostają tylko
  hashe, locatory, profil liczbowy oraz własne wygenerowane zasoby.

## Weryfikacja offline

- Studio: `45` plików testowych, `241 passed`, `1 skipped`;
- Worker/WASM Item integration: `2 passed`, w tym build z profilem V4 i osobnym
  presentation fit ikon;
- Item core: `45 passed`;
- `item_reference_profile`: `4 passed`, w tym env-gated odczyt rzeczywistego
  Steam `nwn_base.key`, `base_2da.bif` i `models_02.bif`;
- `item_resource_context`: `4 passed`;
- `key_bif`: `4 passed`;
- `cargo check -p m2a-wasm`, build WebAssembly, typecheck oraz `cargo fmt
  --check`: passed;
- canonical workspace i canonical Meshy asset layout: passed.

Pełne `cargo test -p m2a-core` przechodzi wszystkie uruchomione testy przed
niezwiązanym testem Creature H2, który wymaga lokalnego, Git-ignored pliku
`sample-3d/h2-clockwork-sentinel-1500/source.glb`. Pliku nie ma w workspace;
nie jest to błąd ani zależność pipeline Item.
