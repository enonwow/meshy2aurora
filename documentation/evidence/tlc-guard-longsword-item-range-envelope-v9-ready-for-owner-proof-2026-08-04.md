# m2atgls9.mod

Toolset module name: `Meshy2Aurora TLC Guard Longsword range envelope v9`
Area: `TLC Guard Longsword Range Envelope V9`

Status: `ready_for_owner_proof`
Candidate: `tlc-guard-longsword-item-range-envelope-v9-20260804`

## Dokładna tożsamość

- MOD: `m2atgls9.mod`
  - SHA-256: `0d65199b43977fba57872deed2f0742a24bd6549c8a2958c0a58e6ae534a1c7e`
  - installed: `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2atgls9.mod`
- ordered HAK: `m2atglh9.hak`
  - SHA-256: `4dd051a3808bd30765579da9ee0e611c87e0b00fedee8b96c916aae3e6742d07`
  - installed: `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2atglh9.hak`
- UTI/resref: `m2atglu9`
- item name: `Last City Bronze Guard Longsword`
- BaseItem: `1` (`WSwLs`, ModelType 2)
- selected values: Bottom `253`, Middle `253`, Top `253`
- identified: `true`
- fit SHA-256:
  `f213e326161ba57d5c09728e1a44c63e498826d17d74072bcbdd6f748c778907`

Źródłowe i zainstalowane pliki MOD/HAK zostały zahashowane po instalacji i są
byte-identical. Agent nie uruchamiał Toolsetu ani NWN.

## Zakres modelu i selektory

Retail `WSwLs` dopuszcza buckety `10..100`, podczas gdy V9 używa modelu `25`,
czyli wariantów `251..254`. HAK zawiera dokładny override `baseitems.2da` typu
`2017`, który zmienia wyłącznie `BaseItem 1 / MaxRange` z `100` na `250`.

- source `baseitems.2da` SHA-256:
  `3fbdcd012b55f0b869c6324cc7d4ece44ed63a78f00e7889e7acefac28c8fdf4`
- patched `baseitems.2da` SHA-256:
  `bfe292e887cb3159b3ca6434d9c1d1da036567b515a0f4b66c135c45f4fdaf15`
- patch status: `PATCHED`
- semantic readback: zachowany prefix, suffix i wszystkie pozostałe komórki

To ma sprawić, że Aurora wyświetli model `25` w polach Top/Middle/Bottom zamiast
pozostawiać puste selektory.

## Dopasowanie partów

Każdy part zachowuje referencyjny zakres Y wybranego natywnego parta WSwLs.
Osobna skala poprzeczna ogranicza X/Z bez skracania broni:

- Bottom: `[0.8272951, 1.0, 0.8272951]`
- Middle: `[0.4277794, 1.0, 0.4277794]`
- Top: `[0.4979150, 1.0, 0.4979150]`

Middle ma po dopasowaniu span X około `0.04439` i span Z około `0.19473`, więc
mieści się w natywnej obwiedni `wswls_m_063`. Fit i oba adjacent seams mają
status `PASSED`.

## Co sprawdzić

1. Otworzyć dokładnie `m2atgls9.mod`, następnie Area podaną na początku.
2. Otworzyć Item Properties dla `Last City Bronze Guard Longsword`.
3. W zakładce Appearance potwierdzić, że Top/Middle/Bottom pokazują model `25`
   oraz kolor `3`; żadne pole Model nie może być puste.
4. W lewym viewportcie potwierdzić, że Bottom, Middle i Top są połączone,
   miecz jest skierowany przodem, a jednoręczny jelec/chwyt nie są nadmiernie
   szerokie ani przesunięte.
5. Potwierdzić, że mała ikona pokazuje ten sam złożony miecz.
6. Sprawdzić ground Item w Area. Moduł zawiera jeden prawdziwy Item i zero
   creatures; nie ma już błędnego `Equipped Item model render witness`.

## Granica dowodu

- `modelVisibility=not_tested`
- `proofCompleteness=missing`
- `visualAcceptance=not_tested`
- `candidateProfile=ITEM_ONLY_GROUND_ITEM_V1`
- `creatureCount=0`

Ta iteracja nie deklaruje obsługi chwytu przez postać. Końcowy wynik wizualny
Item Properties i ground Item może zatwierdzić wyłącznie właściciel.

## Wynik widoczności właściciela 2026-08-04

Właściciel potwierdził, że exact V9 Item jest widoczny zarówno w Aurora
Toolset, jak i w NWN runtime. Dla obu powierzchni zapisano
`modelVisibility=visible` oraz `proofCompleteness=verified`.

Oddzielna akceptacja selektorów modelu `25`, proporcji partów i ich złożenia
nie została jeszcze zgłoszona, dlatego `visualAcceptance=not_tested`.

Dowód: `documentation/evidence/tlc-guard-longsword-v9-owner-visibility-result-2026-08-04.json`.
