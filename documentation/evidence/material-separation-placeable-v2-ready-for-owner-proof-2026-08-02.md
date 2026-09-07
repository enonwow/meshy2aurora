# `m2a_ms2_mod.mod` — Material Separation V2 ready for owner proof

Toolset module name: `Meshy2Aurora Material Separation V2`

Area name: `Material Separation Two Material Proof V2`

Status: `owner_toolset_not_visible / ms2_frozen / ms3_admitted`

> Amendment 2026-08-02: właściciel otworzył exact `m2a_ms2_mod.mod` i Area
> `Material Separation Two Material Proof V2`, lecz viewport nie pokazał
> geometrii. Zrzut pokazuje zielony wire bounds, ale drzewo Placeables jest
> zwinięte i nie ma czytelnego readbacku nazwy zaznaczonego obiektu. Zgodnie z
> modelem iteration gate jest to obserwacja candidate-bound, ale jeszcze nie
> formalny verdict exact-object. MS2 pozostaje zamrożony. Zapis:
> [`material-separation-ms2-owner-toolset-empty-observation-2026-08-02.json`](material-separation-ms2-owner-toolset-empty-observation-2026-08-02.json).

> Final owner decision 2026-08-02: „to znaczy że nie działa”. Dla exact MS2
> zapisano Toolset `modelVisibility=not_visible`, `proofCompleteness=failed`.
> Jedyny Placeable w MOD i widoczny wire bounds wiążą wynik z kandydatem.
> Dopuszczono jeden minimalny MS3 z zamkniętą, judgeable geometrią paneli.

## Dlaczego powstał MS2

Właściciel potwierdził, że exact MS1 nie pokazał Placeable ani w Aurora
Toolset, ani w NWN. Offline diagnoza wykazała, że MS1 używał miniaturowej
testowej bazy `placeables.2da` i wpisał custom appearance pod wierszem `3`.
To dopuściło jedną minimalną iterację MS2.

MS2 używa pełnej produkcyjnej bazy `placeables.2da` o SHA-256
`b772eafec5e6b380ad41e163e2a52585f2ddcec1c5bd7acea230b7e1a618df90`
i dopisuje custom appearance pod wierszem `16500`. Nie zmieniono source GLB,
geometrii, receptury Material Separation, texture authoringu, skali ani
placementu. Nowe są tylko resrefy MS2 i poprawna baza 2DA.

## Exact lineage

- proof packet:
  `C:\Projects\meshy2aurora\proof-output\material-separation-placeable-v2-20260802`;
- source kind: `PROJECT_OWNED_SYNTHETIC_FIXTURE`;
- source SHA-256:
  `a8d28f38fed73c72d391fd194f45ea2285763fc4b9116e4c1e512adb0f885b21`;
- source/output triangles: `2 / 2`;
- material slots / texture resources: `2 / 2`;
- recipe SHA-256:
  `65a94751f6380536d4419f666de7d7595ca56802ab3ecbaef7a863d63fa7f9c5`;
- texture authoring SHA-256:
  `5486ced8a204bd9e8f9dca8fd7dc3238d367974992a088a7744579c8898be6f6`;
- MOD: `m2a_ms2_mod.mod`, `13450` bajtów, SHA-256
  `62870a6427810a555808ae8955c4a10aa3961b54f0ea90066a21a688f6524f37`;
- HAK: `m2a_ms2_hak.hak`, `3022576` bajtów, SHA-256
  `ecd4ff727af740a44439371f9e5a200ecb4fde758a4cf80f6fa8be58118ac7a5`;
- MDL resref: `m2a_ms2_mdl`, SHA-256
  `e18949da952d7d5cf27fcff5283fe679dadb08d985155ce21ec8a53e20e7c3d6`;
- PWK SHA-256:
  `6cf51893499761c559db8002eb8c257be146ca131166d5e931e4b592d02dd205`;
- texture resrefs: `m2a_ms2_tex`, `m2a_ms2_tex_m1`;
- blueprint resref: `m2a_ms2_utp`;
- object tag: `m2a_ms2_two_material_panels`;
- Appearance row: `16500`;
- placement: `(10.0, 14.5, 0.0)`, bearing `0.0`.

Pełna maszyna-czytelna tożsamość znajduje się w
`proof-output/material-separation-placeable-v2-20260802/ready-for-owner-proof.json`.

## Niezależny readback offline

Odczyt exact MOD potwierdził siedem zasobów, Area `m2a_ms2_area`, jeden wpis
Placeable w GIT, tag `m2a_ms2_two_material_panels`, `Appearance 16500`,
`TemplateResRef m2a_ms2_utp`, `Static 1` i pozycję `(10.0, 14.5, 0.0)`.
`module.ifo` wskazuje Area `m2a_ms2_area` i ordered HAK `m2a_ms2_hak`.

Odczyt exact HAK potwierdził MDL, PWK, dwie osobne TGA i pełne
`placeables.2da`. Binary MDL readback potwierdził dwa trójkąty, dwa texture
resrefs, zachowane UV0, brak agresywnego geometry cleanup i deterministyczny
repeat. `cargo check`, `clippy -D warnings` oraz pełny
`cargo test -p m2a-core` zakończyły się bez błędów.

## Native installation

Cele były nieobecne przed zapisem. Exact pliki zapisano z semantyką
create-new, bez overwrite, a następnie potwierdzono byte identity:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_ms2_mod.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_ms2_hak.hak`.

## Owner proof

1. Otwórz dokładnie `m2a_ms2_mod.mod`.
2. Potwierdź nazwę modułu `Meshy2Aurora Material Separation V2`.
3. Otwórz Area `Material Separation Two Material Proof V2`.
4. Oczekiwany Placeable stoi przed graczem i składa się z dwóch oddzielnych
   paneli: czerwonego z tekstury source oraz niebieskiego z override.
5. Zapisz osobno wynik Toolset i NWN.

Agent nie uruchamiał, nie adoptował ani nie kontrolował Toolsetu lub NWN.
Aktualne osie MS2 to `modelVisibility=not_tested` oraz
`proofCompleteness=missing`; wyłącznie właściciel zamyka werdykt wizualny.
