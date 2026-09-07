# `m2a_ms3_mod.mod` — Material Separation V3 ready for owner proof

Toolset module name: `Meshy2Aurora Material Separation V3`

Area name: `Material Separation Closed Panels Proof V3`

Status: `owner_visual_proof_passed / toolset_visible / nwn_visible / ms3_frozen`

> Owner result 2026-08-02: exact `m2a_ms3_mod.mod` i Area
> `Material Separation Closed Panels Proof V3` pokazują jednocześnie dwie
> zamknięte bryły: czerwoną `SOURCE` oraz niebieską `OVERRIDE`. Toolset:
> `modelVisibility=visible`, `proofCompleteness=verified`. MS3 jest zamrożony;
> przed jakąkolwiek zmianą należy sprawdzić ten sam lineage w NWN. Zapis:
> [`material-separation-ms3-owner-toolset-visible-result-2026-08-02.json`](material-separation-ms3-owner-toolset-visible-result-2026-08-02.json).

> Owner NWN result 2026-08-02: ten sam zamrożony MS3 pokazuje przed graczem
> obie oczekiwane bryły — czerwoną `SOURCE` i niebieską `OVERRIDE`. NWN:
> `modelVisibility=visible`, `proofCompleteness=verified`. Finalny proof
> Toolset/NWN jest zamknięty; nie dopuszczono następnej iteracji.

## Minimalna zmiana po wyniku MS2

Właściciel uznał exact MS2 za niedziałający w Aurora Toolset. MS2 został
zamrożony z `modelVisibility=not_visible`, `proofCompleteness=failed`.
Porównanie z historycznie widocznym Placeable potwierdziło poprawny MOD, GIT,
Appearance `16500`, pełne `placeables.2da` i resolver MDL. Ryzykiem MS2 była
nieczytelna geometria: dwa jednostronne sections po jednym trójkącie.

MS3 zmienia wyłącznie source geometry fixture na dwa rozłączne, zamknięte
panele. Każdy ma 12 trójkątów i pozostaje przypisany do jednego authored
Material ID. Zachowano pełny baseline `placeables.2da`, skalę `2.5`, placement,
tryby tekstur `SOURCE`/`OVERRIDE` i wszystkie semantyki packagingu poza nową
tożsamością MS3.

## Exact lineage

- proof packet:
  `C:\Projects\meshy2aurora\proof-output\material-separation-placeable-v3-20260802`;
- source kind: `PROJECT_OWNED_SYNTHETIC_FIXTURE`;
- source SHA-256:
  `b5a904747f473eee779685ff56ca7bd557d1064925b531eabeb927caeb31af21`;
- source/output triangles: `24 / 24`;
- connected components: `2`;
- material slots: `2`, po `12` trójkątów;
- texture resrefs: `m2a_ms3_tex`, `m2a_ms3_tex_m1`;
- MOD: `m2a_ms3_mod.mod`, `13472` bajty, SHA-256
  `68c9f49cdc0f738250a8d7bf04f3758e79652e2988abd6d72e6ce1e43a15282b`;
- HAK: `m2a_ms3_hak.hak`, `3023768` bajtów, SHA-256
  `531f6bde6f96cb150bc3d0403fec4b64e70edba0c5c897fcfa8bd8033b1c6669`;
- MDL resref: `m2a_ms3_mdl`, SHA-256
  `f9b2f2c0cae3db47c4795deb079fecc063af33ca6b8cbb5b2fe088574d3e8086`;
- PWK SHA-256:
  `97826113af37bbe50f19655e895011b8a9f1481a032cf7f1863c1565c3e788e0`;
- production baseline `placeables.2da` SHA-256:
  `b772eafec5e6b380ad41e163e2a52585f2ddcec1c5bd7acea230b7e1a618df90`;
- generated `placeables.2da` SHA-256:
  `30139f0dbe717de305dd25e3a43a9dfc3e4f3a7a1033f12592a63f75c8c0bdf5`;
- Appearance row: `16500`;
- blueprint resref: `m2a_ms3_utp`;
- object tag: `m2a_ms3_two_material_panels`;
- placement: `(10.0, 14.5, 0.0)`, bearing `0.0`.

Pełna maszyna-czytelna tożsamość znajduje się w
`proof-output/material-separation-placeable-v3-20260802/ready-for-owner-proof.json`.

## Walidacja offline

Niezależny readback exact MOD potwierdził siedem zasobów, Area
`m2a_ms3_area`, dokładnie jeden Placeable, `Appearance 16500`,
`TemplateResRef m2a_ms3_utp`, `Static 1` i pozycję `(10.0, 14.5, 0.0)`.

HAK zawiera pełne `placeables.2da`, MDL, PWK oraz dwie TGA. Binary readback
potwierdził `24 / 24` trójkąty, dwa material slots, dwa texture resrefs,
zachowane UV0, brak agresywnego cleanupu i deterministyczny repeat.
`cargo check`, `clippy -D warnings`, testy `model_material_separation` oraz
`placeable_pipeline` zakończyły się bez błędów.

## Native installation

Cele były nieobecne. Exact pliki zapisano bez overwrite i potwierdzono byte
identity względem canonical proof packet:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_ms3_mod.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_ms3_hak.hak`.

## Owner proof

1. Otwórz dokładnie `m2a_ms3_mod.mod`.
2. Potwierdź nazwę modułu `Meshy2Aurora Material Separation V3`.
3. Otwórz Area `Material Separation Closed Panels Proof V3`.
4. Oczekiwane są dwie zamknięte bryły: jedna czerwona z tekstury source i
   druga niebieska z override.
5. Zapisz osobno wynik Toolset i NWN.

Agent nie uruchamiał, nie adoptował ani nie kontrolował Toolsetu lub NWN.
Właściciel zamknął zarówno Toolset, jak i NWN jako
`modelVisibility=visible`, `proofCompleteness=verified`. MS3 pozostaje
zamrożonym finalnym lineage.
