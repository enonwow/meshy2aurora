# `m2aborzmod10.mod` — nowy borzoj `c_wolf` V10 gotowy do proofu właściciela

Moduł wyświetlany w Toolsecie: `Meshy2Aurora Borzoi c_wolf Demo V10`.

Dokładny Area: `Meshy2Aurora Borzoi Test Area V10` (`m2aborzarea10`).

Status: `ready_for_owner_proof`. Agent nie uruchamiał Toolsetu ani NWN.

## Nowe źródło Meshy

Właściciel 2026-08-26 jawnie sklasyfikował czterowidokowy zestaw bind V6 jako
nowy model, niezależny od artefaktowej iteracji V9, i zlecił jeden run Meshy,
pipeline `c_wolf` oraz demo NWN.

- asset: `sample-3d/borzoi-c-wolf-bind-v6-p300k-v1/source.glb`;
- GLB: 12 471 876 B, SHA-256
  `3efd673f4ec953de568b2a30f6f14131a62828398f3133bb89855193b5fffd93`;
- geometria: 299 783 trójkąty;
- Meshy: jeden run, 30 kredytów, bez drugiego modelu i bez auto-riga
  humanoidalnego.

## Naprawa ogólnego fittera

Pierwszy przebieg produktu został prawidłowo zablokowany: automatyczny fitter
rozpoznał tylko 2/4 kontakty łap, a motion gate zgłosił
`2 810 866 / 1 888 632` próbek rozciągniętych ponad limit. Render źródłowego
GLB potwierdził cztery fizyczne łapy. Raport ujawnił asymetrię klastra przednich
łap: 11 widocznych wierzchołków po lewej i 1830 po prawej.

Do ogólnego `reference_supermodel_generic` dodano odzyskiwanie brakujących
terminali kontaktu z punktów, dla których terminal jest już najbliższym
strukturalnym jointem. Adaptacyjne pasmo odrzuca odległe kontakty ogona i
brzucha. Nie dodano nazwy `c_wolf`, gatunku ani wyjątku dla borzoja. Test
regresyjny przed zmianą odtwarzał `2/4`, a po zmianie zwraca `4/4`.

## Wynik offline V10

- `motionQuality=PASS`;
- 30 carrierów, 23/23 aktywne kości;
- 4/4 dopasowane łańcuchy kontaktu z podłożem;
- 42/42 klipy i 907/907 par joint×clip;
- rozciągnięcia `1 678 244 / 1 888 632`, poniżej niezmienionego limitu;
- 0 naruszeń szwów, 0 błędów kontaktu łap i 0 błędów stron łap;
- `c_wolf`, 0 lokalnych animacji, semantyczna delta ogona potwierdzona;
- `shadow=0` pozostaje aktywne dla segmentów renderujących.

## Zamrożone artefakty

- MOD `m2aborzmod10.mod`: 15 295 B, SHA-256
  `0a97d8fd3429fc359b7d56bd36b3b7a3927f557af3e8f073c06233f24e234456`;
- HAK `m2aborzhak10.hak`: 41 150 702 B, SHA-256
  `8a168fdc4cdb4b3439f08940d53c3e774cf23c13979c58064aafc379abfd3186`;
- MDL `m2aborzcre10.mdl`: 21 666 072 B, SHA-256
  `764298489ca8f6e60e4eb8f29bd8249cdd11d2d31226fd5b2eac2a68e164c4fd`;
- diffuse `m2aborztex10.tga`: SHA-256
  `cdd9ac9310740952f1f1c804c6ec66283f185be0c5d15a549873ed4ad3d15cbe`;
- MTR `m2aborztex10_m0.mtr`: SHA-256
  `7dd199ba5f24144860d826528b231ef015a0c8629f793501f5dc55195aa6cb7f`.

MOD i HAK zostały zainstalowane do wcześniej nieistniejących celów w
`Documents/Neverwinter Nights/modules` oraz `hak`. Hash źródła i celu jest
byte-for-byte identyczny.

## Scena i proof właściciela

Appearance row: `15100`; template: `m2aborzutc10`; pies stoi w
`(10.0, 14.5, 0.0)` i patrzy w `(0.0, -1.0)`.

Do wyniku właściciela Toolset i NWN pozostają
`modelVisibility=not_tested`, `proofCompleteness=missing`. Należy sprawdzić
widoczność, idle, chód, bieg, walkę, ruch całego ogona i końcówki, kontakt
czterech łap oraz brak ciemnych przerywanych linii.
