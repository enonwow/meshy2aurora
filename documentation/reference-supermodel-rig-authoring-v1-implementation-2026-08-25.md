# Reference Supermodel Rig Authoring V1 — implementacja (2026-08-25)

## Wynik

Studio potrafi teraz edytować owned target rig modelu korzystającego z dowolnie wybranego supermodelu. Edycja nie modyfikuje referencyjnego MDL ani jego payloadu i nie zmienia nazw, numerów, liczby lub hierarchii carrierów. Preview oraz eksport korzystają z tego samego, zapieczętowanego dokumentu authoringu.

Nie utworzono nowego MOD/HAK ani iteracji modelu. Ten etap jest implementacją i walidacją offline; końcowy proof wizualny Aurora/NWN pozostaje human-owned.

## Fakty zaimplementowane

### Core

Nowy kontrakt `ReferenceSupermodelRigAuthoringDocumentV1` wiąże edycje z:

- SHA-256 źródłowego GLB,
- kierunkiem przodu źródła,
- resrefem wybranego supermodelu,
- SHA-256 exact chain,
- SHA-256 motion contract,
- SHA-256 bazowego target riga.

Dokument obsługuje:

- sparse `jointOverrides` wskazywane przez stabilny `carrierPartNumber`,
- pełną lokalną macierz bind jointa,
- opcjonalną rolę semantyczną i znormalizowaną oś jointa,
- blokadę authoringową rekordu,
- sparse `weightOverrides` dla dokładnego `(segmentId, vertexIndex)`, maksymalnie cztery wpływy.

Walidacja fail-closed odrzuca między innymi:

- zmieniony lub niezapieczętowany dokument,
- dokument pochodzący z innego źródła, supermodelu, chainu, kontraktu lub bazowego riga,
- macierze nieafiniczne, osobliwe lub zawierające wartości niefinitywne,
- zduplikowane joint/weight overrides,
- wagi niefinitywne, niedodatnie, z powtórzonym bone ID albo sumą różną od `1.0`,
- bone ID spoza `allowedBoneNodeIds`, brakujący segment, vertex lub carrier.

Po zastosowaniu zmian Core porównuje `(id, name, parentId)` wszystkich carrierów przed i po operacji. Autorowi nie udostępniono operacji rename/add/delete/reparent, ponieważ odziedziczone kontrolery animacji wiążą się z dokładnymi carrierami.

### WASM i Worker

`buildReferenceSupermodelAppliedPreviewV2` zwraca teraz również:

- `authoringJson` — pusty, zapieczętowany dokument bazowy,
- `targetRigJson` — dokładny target rig użyty do wygenerowania preview.

Dodano:

- `buildReferenceSupermodelAuthoredPreviewV1` — przyjmuje draft, waliduje go, pieczętuje i generuje preview z authored riga,
- `buildReferenceSupermodelCreatureProductV3` — przyjmuje wyłącznie zapieczętowany dokument; nie naprawia ani nie przepieczętowuje zmienionego wejścia.

Worker udostępnia request `BUILD_REFERENCE_SUPERMODEL_AUTHORED_PREVIEW`. Produktowa ścieżka `REFERENCE_SUPERMODEL_CREATURE` automatycznie wybiera V3, gdy otrzyma `rigAuthoringJson`.

### Studio

W bibliotece supermodeli dodano edytor:

- wybór jointa z listy target riga,
- wybór jointa przez kliknięcie overlayu w viewport,
- pozycja XYZ,
- rotacja XYZ w stopniach,
- skala XYZ,
- rola semantyczna,
- opcjonalna oś jointa,
- blokada override,
- przywrócenie bazowego jointa,
- sparse weight override jako `boneId:weight`,
- przywrócenie bazowego wiersza wag.

Każde zastosowanie przebudowuje diagnostyczny binary MDL przez Worker/WASM/Core. Nowy `contentSha256` authoringu trafia do stanu aplikacji, unieważnia wcześniejszy build i jest przekazywany do eksportu produktu V3.

## Zweryfikowane kryteria zakończenia offline

- `cargo test -p m2a-core --test reference_supermodel_authoring`: 3/3 PASS.
- `cargo test -p m2a-wasm --lib reference_supermodel_motion_boundary_tests::ascii_supermodel_completes_analysis_preview_and_product_offline -- --exact`: PASS; ten sam zapieczętowany authoring przechodzi przez preview i produkt, `jointOverrideCount=1`, `carrierTopologyPreserved=true`.
- pełne `cargo test -p m2a-wasm --lib`: 55/55 PASS.
- `npm test` w `apps/studio-web`: 60 plików, 332 testy PASS.
- `npm run typecheck`: PASS.
- `npm run build`: PASS wraz z ponowną kompilacją WASM i produkcyjnym buildem Vite.

## Granica dowodu

Powyższe potwierdza kontrakt, deterministyczne użycie authoringu oraz brak mutacji topologii carrierów offline. Nie jest to twierdzenie, że konkretna korekta jointów ma już poprawny wygląd w Toolset/NWN. Taką ocenę wykonuje właściciel na wybranym, później zamrożonym kandydacie zgodnie z model iteration gate.
