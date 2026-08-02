# `m2a_ms1_mod.mod` — Material Separation V1 ready for owner proof

Toolset module name: `Meshy2Aurora Material Separation V1`

Area name: `Material Separation Two Material Proof`

Status: `owner_failure_reported / ms1_frozen / not_ready_for_owner_proof`

> Amendment 2026-08-02: właściciel zgłosił bezpośrednio po handoffie
> „nic w tym module nie ma”. Kandydat `m2a_ms1` nie jest już uznawany za gotowy
> do proofu. Offline readback nadal potwierdza Area i jeden wpis Placeable w
> GIT, ale audyt wykazał błąd przygotowania: materializer użył małej tabeli
> testowej `placeables.2da` i przypisał Appearance `3`, podczas gdy sprawdzone
> moduły Placeable używają pełnego baseline'u i wiersza `16500`. Exact MS1
> pozostaje zamrożony. Wynik oraz warunek wznowienia zapisano w
> [material-separation-ms1-owner-empty-module-result-2026-08-02.json](material-separation-ms1-owner-empty-module-result-2026-08-02.json).

## Cel i wynik offline

To pierwszy, osobny fixture funkcji Material Separation, a nie kolejna
iteracja istniejącego modelu statku. Projektowa geometria syntetyczna zawiera
dwa rozłączne komponenty w jednym source primitive i jednym source material.
Recipe przypisuje je do dwóch authored Material IDs:

- `material:red-source` używa oryginalnej czerwonej tekstury `SOURCE`;
- `material:blue-override` używa osobnej niebieskiej tekstury `OVERRIDE`.

Pipeline wykonano end-to-end przez wspólny Core, Placeable V6, texture
authoring, binary MDL writer, HAK i MOD. Powtórne zbudowanie tych samych wejść
dało byte-identical MOD, HAK i report. Binary readback potwierdził dwa
trójkąty, dwa material slots, dwa texture resrefs, zachowane UV0 oraz brak
agresywnego czyszczenia geometrii.

## Exact lineage

- proof packet:
  `C:\Projects\meshy2aurora\proof-output\material-separation-placeable-v1-20260801`;
- source kind: `PROJECT_OWNED_SYNTHETIC_FIXTURE`;
- source bytes: `1568`;
- source SHA-256:
  `a8d28f38fed73c72d391fd194f45ea2285763fc4b9116e4c1e512adb0f885b21`;
- source/output triangles: `2 / 2`;
- connected components: `2`;
- Meshy API calls/credits: `0 / 0`;
- MOD: `m2a_ms1_mod.mod`, `13438` bajtów,
  SHA-256 `f0ebb252a86a399e7b458716dcfbc8051280ea9639a1f03a5302814fa063b287`;
- HAK: `m2a_ms1_hak.hak`, `3224` bajty,
  SHA-256 `d6dd8dcc711b9ef449be67d868126b15cea436039a56e0f5725c702b56199664`;
- MDL resref: `m2a_ms1_mdl`, SHA-256
  `9e4f8dbf1f80c53a162faf8f0199f3dad96691ad481ac5aa64c34f251dc8ffa4`;
- PWK SHA-256:
  `33788bccbc3d0fd1f1c303a5816e35ee7762b3ec95883cacbb5de6a4f60ec22d`;
- texture resrefs: `m2a_ms1_tex`, `m2a_ms1_tex_m1`;
- recipe SHA-256:
  `65a94751f6380536d4419f666de7d7595ca56802ab3ecbaef7a863d63fa7f9c5`;
- texture authoring SHA-256:
  `5486ced8a204bd9e8f9dca8fd7dc3238d367974992a088a7744579c8898be6f6`;
- binary readback SHA-256:
  `f331b88b73420267f9eac0a3c5dea394841fe905b26c8f91a41f2f1a93c4c6fd`;
- Appearance row: `3`;
- blueprint resref: `m2a_ms1_utp`;
- object tag: `m2a_ms1_two_material_panels`;
- placement: `(10.0, 14.5, 0.0)`, bearing `0.0`.

Pełna maszyna-czytelna tożsamość znajduje się w
`proof-output/material-separation-placeable-v1-20260801/ready-for-owner-proof.json`.

## Native installation

Oba cele były nieobecne przed zapisem. Użyto semantyki create-new, bez
overwrite, a następnie potwierdzono byte identity:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_ms1_mod.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_ms1_hak.hak`.

## Owner proof

1. Otwórz dokładnie `m2a_ms1_mod.mod`.
2. Potwierdź nazwę modułu `Meshy2Aurora Material Separation V1`.
3. Otwórz Area `Material Separation Two Material Proof`.
4. Obiekt `m2a_ms1_two_material_panels` stoi przed graczem. Oczekiwane są dwa
   rozdzielone panele: czerwony z tekstury source i niebieski z override.
5. Zapisz osobno dla Toolset i NWN:
   `modelVisibility=visible|not_visible|not_tested` oraz
   `proofCompleteness=verified|failed|missing`.

Agent nie uruchamiał, nie adoptował i nie kontrolował Toolsetu ani NWN.
Aktualny stan obu torów to `modelVisibility=not_tested` i
`proofCompleteness=missing`; wyłącznie właściciel zamyka werdykt wizualny.
