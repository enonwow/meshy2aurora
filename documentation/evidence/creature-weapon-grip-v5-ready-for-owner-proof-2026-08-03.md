# Creature weapon grip V5 — ready for owner proof — 2026-08-03

> **Owner result 2026-08-03: GRIP ROTATION FAILED.** Exact
> `m2aweapdemo5.mod` pokazuje Creature i stockowy miecz przy prawej dłoni, lecz
> broń ma nieprawidłowy skręt/orientację. Toolset zachowuje
> `modelVisibility=visible`, natomiast funkcjonalny proof chwytu ma
> `proofCompleteness=failed`. Screenshot:
> `C:\Users\enonw\AppData\Local\Temp\codex-clipboard-5c8e9bc4-3d90-4f5f-ab86-7dee26da203a.png`,
> SHA-256
> `dfb2c48174ea0b567d44946ebc5ebdf723686cb43b93cf4f6734d9fe4e13d91e`.
>
> Potwierdzona przyczyna implementacyjna: V5 wyznacza oś podłużną broni z
> kierunku wrist-to-palm, ale pozostały roll stabilizuje globalnym `Z`, a nie
> płaszczyzną dłoni ani pełnym natywnym basisem item hooka. Ponadto ścieżka V5
> dla modelu z wystarczającą geometrią dłoni nie stosuje jawnej korekty V4;
> wcześniejszy opis „V4 retained” był zbyt mocny. Studio renderuje zgodną z MDL
> macierz, ale na uproszczonym `CALIBRATION_PROXY`, nie na dokładnym stockowym
> modelu NWN, więc jego pozytywny obraz nie był proofem rotacji rzeczywistego
> itemu.

1. Exact test module: `m2aweapdemo5.mod`
2. Module name in Toolset: `Meshy2Aurora Creature Weapon Grip V5`
3. Exact Area: `Meshy2Aurora Creature Weapon Test V5`
4. Exact HAK: `m2aweaphak5.hak`

Status po wyniku właściciela: `grip_rotation_failed`. Agent nie uruchamiał i nie kontrolował Aurora
Toolset ani NWN. Właściciel bezpośrednio zlecił przygotowanie nowego modułu po
zaakceptowaniu implementacji V5; jest to autoryzacja tej materializacji mimo
historycznej blokady kolejnej iteracji zapisanej przy V3.

## Zakres kandydata

- source GLB:
  `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`;
- source SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`;
- model: `m2aweapcre5.mdl`;
- texture: `m2aweaptex5.tga`;
- Appearance row: `15104`, `MODELTYPE=L`;
- Creature template: `m2awrhand5`;
- Creature label: `V5 RIGHT HAND - native stock sword - palm calibrated`;
- weapon: stockowy NWN `nw_wswss001`, prawy slot o struct id `16`;
- kalibracja: `MESHY_H1_SKIN_WEIGHTED_PALM_CENTER_V5`;
- orientacja modelu: kanoniczne źródłowe `+Z`, bez testowego obrotu.

V5 wyznacza pivot hooka z przestrzennego, skin-weighted środka dłoni w bind
pose: próg sumy wpływów kości dłoni `>= 0.5`, weld seamów `1e-5 m`, mediana
współrzędnych w hand-local. Poprzedni V4 oparty na stałym stosunku długości
przedramienia pozostaje wyłącznie fallbackiem dla fixture bez wystarczającej
geometrii dłoni.

## Offline gates i readback

- triangles source/output: `297190 / 297190`;
- animations: `42`;
- `rhand` parent: `RightHand`;
- `lhand` parent: `LeftHand`;
- hook coverage: `42/42` klipów;
- wyposażenie UTC/GIT: `nw_wswss001`, prawa ręka;
- test nazw IFO/ARE V3: pass;
- historyczna regresja stock-weapon V2: pass;
- procedural product/module integration: pass;
- `cargo fmt --all -- --check`: pass.

## Exact lineage i hashe

- MOD: `proof-output/creature-weapon-grip-v5/m2aweapdemo5.mod`
  — SHA-256
  `f2ac29db33fa516358ed42ff41ee9acba1a7733bfda081706a9d2b9170812fad`;
- HAK: `proof-output/creature-weapon-grip-v5/m2aweaphak5.hak`
  — SHA-256
  `8cc88c3050f11a9590999bd43c6d4f5fc651c97d9ce381878aa7edce8efd4c9c`;
- MDL: `proof-output/creature-weapon-grip-v5/m2aweapcre5.mdl`
  — SHA-256
  `681ddc321f831dcd859f7153a88ce175b925ea157d01ec0c1f12679982da05d4`;
- TGA: `proof-output/creature-weapon-grip-v5/m2aweaptex5.tga`
  — SHA-256
  `7e24d51726355bf3a4ee2f1429ea85f43a604bc2ee3e079c9fd5a306d4b1426a`;
- appearance.2da SHA-256:
  `a986346ea77946dc977e9aea4ff3863fe5c21723227ad2381aea3e92991c1fc7`;
- machine-readable handoff:
  `proof-output/creature-weapon-grip-v5/handoff.json`.

## Native installation

Oba cele były nieobecne przed kopiowaniem. Zainstalowano exact canonical
artefakty i zweryfikowano je byte-for-byte:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aweapdemo5.mod`
  — SHA-256
  `f2ac29db33fa516358ed42ff41ee9acba1a7733bfda081706a9d2b9170812fad`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aweaphak5.hak`
  — SHA-256
  `8cc88c3050f11a9590999bd43c6d4f5fc651c97d9ce381878aa7edce8efd4c9c`.

## Owner proof checklist

1. Otwórz `m2aweapdemo5.mod`.
2. Otwórz Area `Meshy2Aurora Creature Weapon Test V5`.
3. Wybierz jedyną postać
   `V5 RIGHT HAND - native stock sword - palm calibrated`.
4. Potwierdź w Inventory stockowy miecz w głównym slocie prawej ręki.
5. Oceń pozycję rękojeści i kierunek ostrza w idle, walk/run oraz atakach,
   najpierw w Toolsecie, potem na tym samym lineage w NWN.

Stan po proofie właściciela:

- Toolset: `modelVisibility=visible`, `proofCompleteness=failed` dla chwytu;
- NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`.
