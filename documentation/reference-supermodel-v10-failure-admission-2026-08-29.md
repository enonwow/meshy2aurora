# Reference-supermodel V10 — zapis awarii i dopuszczenie minimalnej iteracji

Data: 2026-08-29

## Odrzucony exact candidate

- diagnostyka: `artifacts/diagnostics/borzoi-v10-skeleton-repair-motion-three-joint-seam-v9`
- source GLB SHA-256: `3efd673f4ec953de568b2a30f6f14131a62828398f3133bb89855193b5fffd93`
- exact retail `c_wolf` SHA-256: `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`
- `base-preview.mdl`: `4bb1e2b4ccd71768b44f241d52bec1d03a5e6ce995e25601e4ae54310558101c`
- `base-preview-report.json`: `aebcdeec8fd9e2fea3775506fff9600c184e05b456f59010da2046320f115b08`
- `base-target-rig.json`: `8ff0c979606284cf25b967059bf00451132c50562e2af6318f52efe702e38656`
- `base-authoring.json`: `93681fcd77df7150e5660b6e098dd2b344b6541ffa2cadbff76ec1b4ad451d05`

Raport zachowuje pełną strukturę 30 carrierów, 42 wymagane klipy i 907/907
joint×clip, ale kończy się `motionQuality=BLOCKED` oraz 2328 odrzuconymi
próbkami component×clip.

## Świeży werdykt właściciela

Właściciel otworzył dokładną zapisaną diagnostykę w Studio i stwierdził, że
fitted target skeleton i jointy są wizualnie niepoprawne względem retail
`c_wolf`. To jest candidate-bound wizualna awaria istniejącego V10, a nie
awaria samej ścieżki proofu.

## Zdiagnozowana przyczyna

Stary kontrakt przyznawał `surfaceAnatomy=READY` i `jointFit=READY` na podstawie
zbyt słabych confidence/coverage. Karty i duplicate seams wpływały na analizę,
authoring nie przechodził ponownej walidacji anatomicznej, a bind pose nie miał
osobnej bramki. Dodatkowo fitter osadzał terminale łap dokładnie 1,5% wysokości
ciała nad płaszczyzną, jednocześnie wymagając później maksimum 1,0%.

## Minimalny dozwolony delta

Następna diagnostyka może zmienić wyłącznie wynik ogólnego pipeline’u:

- virtual-weld/body-proxy do analizy bez zmiany render mesh/UV;
- topology-derived constraints joint-fit i per-joint residual/verdict;
- semantyczną projekcję auxiliary surfaces oraz ścisłą walidację wag;
- osobną bramkę bind pose;
- ponowną walidację sealed authoringu;
- centralny `ReferenceSupermodelAdmissionV3`;
- osadzenie ground terminal na 0,5% wysokości nad zmierzonym minimum.

Source GLB, exact `c_wolf`, nazwy carrierów, parenty i retail payload pozostają
niezmienione. Przed pełnym motion audit nowy kod osiągnął na tych exact danych:
`surfaceAnatomy=READY`, `jointFit=READY` (10/10), `skinning=READY`,
`bindPose=PASS` (10/10), zero cross-side/cross-branch leakage. Nie wolno tworzyć
MOD/HAK, dopóki pełny motion-quality i centralny admission nie uzyskają `PASS`.
