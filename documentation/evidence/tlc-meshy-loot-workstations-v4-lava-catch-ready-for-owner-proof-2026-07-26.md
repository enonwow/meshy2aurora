# `m2a_tlcw4_mod.mod` — reaktor V4 gotowy do testu właściciela

Nazwa modułu w Toolset: `Meshy2Aurora TLC Loot Workstations V4`  
Area: `Meshy2Aurora TLC Loot Workstations V4`  
HAK: `m2a_tlcw4_hak.hak`  
Status: `ready_for_owner_proof`

## Poprawiony układ

- [x] Pionowy strumień lawy `C766` zachowuje lokalną transformację identity.
- [x] Końcowy trójkąt strumienia `C809` zachowuje lokalną transformację identity.
- [x] Właściwa dolna misa jest grupą 16 komponentów:
      `31, 33, 34, 35, 45, 66, 74, 76, 84, 92, 144, 155, 170, 600, 602, 603`.
- [x] Transformacja misy: translation `[0, 0.05, 0.14]`,
      scale `[0.54, 1, 1.50]`.
- [x] Dolny pierścień `C37` pozostaje przy nogach i ma transformację identity.
- [x] Nogi nie są elementami misy i nie zostały przesunięte.
- [x] Dwa zespoły łańcuchów pozostały bez transformacji: prawy ma 186
      komponentów, lewy 194.
- [x] Cały model zachowuje wspólną skalę `2.5`, czyli wysokość `1.50 m`.
- [x] Znormalizowana odległość punktu lawy od środka aktywnej powierzchni misy
      wynosi `0.375153`, przy limicie `0.600000`.
- [x] Stary układ V3 daje `0.872884` i jest przez nowy kontrakt odrzucany.

![Reaktor V4 — pionowy strumień kończy się na tafli dolnej misy](../../output/playwright/reactor-v4-basin-16-z014-s150.png)

SHA-256 obrazu viewportu:
`c86eadea4bb1e80ad657eb752a686445444e39b6a30b6bdd972cd94cf7aa9c87`.

Podgląd jest kontrolą authoringu Three.js. Nie jest dowodem renderera Aurora
ani NWN.

## Wspólny pipeline

V4 korzysta z tej samej ścieżki co pozostałe modele:

`GLB -> wspólny parser/IR -> PlaceableAuthoringDocumentV1 -> Profile A -> binary MDL -> shadow adjacency -> ASCII PWK -> placeables.2da -> UTP -> GIT/GIC/ITP -> HAK/MOD`

- [x] dokładnie 20 000 trójkątów i 60 000 indeksów na placeable;
- [x] limit pojedynczego mesha: 21 845 trójkątów;
- [x] binarny MDL przeszedł niezależny readback;
- [x] model ma `classification=4`, `render=1`, `shadow=1`;
- [x] model reaktora ma wysokość bounds `1.50 m`;
- [x] shadow adjacency przeszło readback;
- [x] ASCII PWK przeszło readback;
- [x] `placeables.2da`, UTP, GIT/GIC/ITP, HAK i MOD przeszły readback;
- [x] testy `placeable_authoring`: 5/5;
- [x] testy `placeable_pipeline`: 9/9 plus 1 świadomie ignorowany test zależny od env;
- [x] testy materializatora: 3/3 plus 3 dokładne testy źródłowe oznaczone jako ignored;
- [x] dokładny źródłowy test V4 został uruchomiony osobno i przeszedł;
- [x] `cargo fmt --all -- --check`.

## Artefakty

Kanoniczny MOD:

[m2a_tlcw4_mod.mod](../../proof-output/tlc-meshy-loot-workstations-v4-20260726/generated/m2a_tlcw4_mod.mod)

- SHA-256:
  `c49ae2ab0bd79cf8f100794cfe5f93e1f7713408fd75902ba5ad69d3d4c9a348`;
- zainstalowany:
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_tlcw4_mod.mod`;
- hash docelowy jest identyczny.

Kanoniczny HAK:

[m2a_tlcw4_hak.hak](../../proof-output/tlc-meshy-loot-workstations-v4-20260726/generated/m2a_tlcw4_hak.hak)

- SHA-256:
  `6b14c8ba49cb304e91039ebfa742d0ba4360474d4e21be5bbc7d3fa04cc26692`;
- zainstalowany:
  `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_tlcw4_hak.hak`;
- hash docelowy jest identyczny.

Model reaktora:

- resref: `m2a_tlcw4_upg`;
- blueprint: `m2a_tlcw4_uu`;
- Appearance row: `16502`;
- pozycja: `X=13.5`, `Y=14.5`, `Z=0.0`, bearing `0.0`;
- MDL SHA-256:
  `dfb50a33dc8dcc3266f090d2003c6f425ba49057612da1fe9a7782a79d89f5d2`;
- PWK SHA-256:
  `3b949a92ae57f665ce8b471b4b26744ccfef0ad32850ba659310e7d6b6e5bd2c`.

Dokument authoringu:

[m2a_tlcw4_upg-authoring.json](../../proof-output/tlc-meshy-loot-workstations-v4-20260726/generated/m2a_tlcw4_upg-authoring.json)

- authoring SHA-256:
  `db695e214d99709bdcb6ee16c70e8d468aa221ae2642126623ddbbe5e87f25ff`;
- SHA-256 pliku JSON:
  `737f8b30bcad10aa3f42e71e925ad3ea6a99a7b10e72862c07d93ecacd24f256`.

Maszynowy handoff:

[ready-for-owner-proof.json](../../proof-output/tlc-meshy-loot-workstations-v4-20260726/ready-for-owner-proof.json)

SHA-256 po zapisaniu potwierdzenia instalacji:
`7fb4a2483b3706e4c4bed8a3f5503bbc1b1d16b48e0dc818010db10ce55718b6`.

## Test właściciela

- [ ] Otworzyć `m2a_tlcw4_mod.mod`.
- [ ] Potwierdzić nazwę modułu `Meshy2Aurora TLC Loot Workstations V4`.
- [ ] Otworzyć Area `Meshy2Aurora TLC Loot Workstations V4`.
- [ ] Wybrać `TLC V4 Upgrading Reactor 20K`.
- [ ] Potwierdzić, że pionowy strumień kończy się wewnątrz pomarańczowej tafli
      dolnej misy, a nie na jej rancie.
- [ ] Potwierdzić, że dolny pierścień i stopy nóg nie zostały wciągnięte do misy.
- [ ] Potwierdzić oba łańcuchy korpus–podstawa.
- [ ] Sprawdzić cień w Toolset i NWN.
- [ ] Sprawdzić kolizję PWK z kilku kierunków w NWN.

Agent nie uruchamiał Toolsetu ani NWN. Do wyniku właściciela pozostaje:

- `modelVisibility = not_tested`
- `proofCompleteness = missing`
- `collisionRuntimeVerdict = not_tested`
