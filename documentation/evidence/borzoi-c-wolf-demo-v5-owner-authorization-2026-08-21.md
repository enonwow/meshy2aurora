# Borzoi `c_wolf` V5 — bezpośrednia autoryzacja właściciela

Data: 2026-08-21

Status: `ONE_EXACT_V5_ITERATION_AUTHORIZED`

## Decyzja właściciela

Po odrzuceniu jakości V4 właściciel zakwestionował zastosowanie bramki
widoczności do tego przypadku i bezpośrednio polecił przygotować poprawne nowe
demo. Polecenie dopuszcza dokładnie jedną nową iterację: V5.

Nie jest to zgoda na uruchamianie Toolsetu lub NWN przez agenta. Końcowy proof
wizualny nadal wykonuje właściciel.

## Dowód poprzedniego problemu

- V3: `modelVisibility=visible`, animacje `c_wolf` obserwowane, jakość
  deformacji odrzucona przez właściciela;
- trwały obraz V3:
  `documentation/evidence/borzoi-c-wolf-demo-v3-owner-nwn-result-2026-08-20.png`;
- V4: właściciel przekazał świeży screen z rozerwaniem powierzchni i ocenił
  wynik jako fatalny, po czym polecił wrócić do V3 jako bazy diagnostycznej;
- rozpoznana przyczyna: V3 udowadniał jedynie pokrycie kości, a V4 zastąpił
  ciągły skin rigid assignmentem rozłącznych komponentów. Obie trasy używały
  też uproszczonego materiału i topology-only kontraktu supermodelu.

## Dokładna tożsamość dopuszczonego kandydata

- lineage: `proof-output/borzoi-c-wolf-demo-v5-20260821`;
- MOD: `m2aborzmod5.mod`;
- HAK: `m2aborzhak5.hak`;
- model: `m2aborzcre5.mdl`;
- bazowy resref materiału: `m2aborztex5`;
- module display name: `Meshy2Aurora Borzoi c_wolf Demo V5`;
- Area display name: `Meshy2Aurora Borzoi Test Area V5`;
- Appearance row: `848`;
- source: `sample-3d/borzoi-meshy-manual-p1997k-v1/source-p300k.glb`;
- source SHA-256:
  `f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda`.

## Minimalna delta V5

1. Rigid component assignment V4 nie wraca. V5 używa własnej ciągłej klatki
   deformacyjnej (54 wierzchołki, 42 trójkąty) i przenosi wagi
   barycentrycznie na dokładną siatkę renderową.
2. Eksperymentalna warstwa 30 correction nodes została odrzucona w trakcie
   implementacji: dziedziczone kontrolery `c_wolf` rozwiązują stan po nazwie,
   więc dodatkowe dzieci nie otrzymywały ruchu zgodnego z zamierzonym
   łańcuchem. Finalny V5 ma dokładnie 30 bezpośrednich, nazwanych carrier nodes.
3. Skin wskazuje te 30 źródłowo dopasowanych nośników, które zachowują
   topologię, nazwy i rodziców kontraktu `c_wolf`.
4. Offline oracle wymaga ośmiu klipów, eventów i kanałów oraz procentowych
   bramek edge/triangle/seam i kompletnych klastrów czterech łap. Metryki
   normalnych w świecie, side-crossing, kontakt i clip-start pozostają jawnie
   raportowaną diagnostyką, ponieważ absolute-zero odrzucało również retailowy
   model `c_dog` dziedziczący po `c_wolf`.
5. Materiał używa pełnego zestawu diffuse/normal/specular-gloss + TXI + MTR,
   nie pojedynczej TGA.
6. MOD/HAK powstają raz, są zamrażane i instalowane no-replace. Dalsza
   iteracja wymaga nowego wyniku właściciela.
