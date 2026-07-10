# przyszle-featurey-studio-codex.md

Data: 2026-07-10 | Autor: Codex | Status: BACKLOG PO MVP

## Cel i granica

Ten dokument zbiera przyszle funkcje Studio `meshy2aurora`. Nie jest to plan biezacego MVP ani upowaznienie do rozpoczecia implementacji przed pierwszym proofem end-to-end.

Aktywny priorytet pozostaje niezmieniony:

```text
GLB -> own parser/converter/writer -> binary MDL + 2DA + HAK -> NWN EE proof
```

Przed rozpoczeciem dowolnego feature'u `F1`-`F10` musza przejsc:

```yaml
entry_gates:
  - "binary MDL/MDX policy resolved"
  - "own 2DA and HAK writer works on synthetic fixtures"
  - "generated asset is visible in NWN EE Toolset/game"
  - "base color -> generated TGA path is proven"
```

## Zasady wspolne

```yaml
studio_rules:
  source_glb: "never overwrite source input"
  edit_state: "store declarative recipes in m2a.project.json"
  final_preview: "readback of our generated output, not an embellished raw GLB"
  external_references: "Aurora First; aurora-web remains read-only reference only"
  proof: "generated HAK/module plus NWN EE visual proof"
  anti_scope_creep: "Studio is a conversion/validation editor, not a replacement for Blender"
```

## Backlog funkcji

### F1. Scene Part and Material Picker

Status: PLAN.

Klikniecie czesci modelu w viewportcie wskazuje pelna trase danych:

```text
mesh node -> primitive -> material -> image -> UV -> output TGA -> emitted MDL reference
```

Funkcje:

- izolacja, ukrycie i ponowne pokazanie wybranej czesci;
- lista czesci korzystajacych z tego samego materialu;
- odczyt resrefu docelowej tekstury;
- skok z walidatora bezposrednio do problematycznej czesci.

### F2. UV Checker

Status: PLAN.

Tryb kontrolny naklada teksture siatki UV oraz zaznacza:

- odwrocony kierunek V;
- rozciagniete albo zdegenerowane UV;
- UV poza aktywnym zakresem polityki;
- brak `TEXCOORD_0`;
- roznice miedzy Source i Aurora Preview.

Gate: wynikowy TGA po readback nie moze byc pionowo odbity wzgledem fixture `uv-probe.glb`.

### F3. Model Health and Repair Plan

Status: PLAN.

Jeden raport `OK`, `WARN`, `BLOCKED` dla:

- triangles, vertices, mesh nodes i material splits;
- rozmiaru, alpha i resrefu tekstur;
- UV, pivotu, osi i skali;
- skeletonu, wag i animacji;
- gotowosci MDL, 2DA i HAK.

Raport podaje kod bledu, dotkniety element i bezpieczna proponowana akcje. Auto-repair jest dozwolony tylko dla zmian deterministycznych zapisanych w raporcie; zmiany artystyczne wymagaja akceptacji uzytkownika.

### F4. Reference Compare

Status: PLAN.

Porownanie `Source`, `Aurora Preview` i `Readback` w tych samych kontrolowanych rzutach. Opcjonalnie obok pokazuje zatwierdzona referencje techniczna lub artystyczna.

Funkcje:

- front, side, quarter i top;
- wspolna kamera, siatka podloza i bounding box;
- overlay sylwetki oraz roznicy rozmiaru;
- zapis screenshotu porownawczego do proof packet.

Nie wolno oznaczyc takiego porownania jako runtime proofu bez osobnego dowodu z NWN EE.

### F5. Material Variants

Status: PLAN.

Warianty zachowuja jeden model, ale maja rozne recipe tekstur, resrefy i generowane TGA, na przyklad `m2a_koc01_green` oraz `m2a_koc01_undead`.

Gates:

- kazdy wariant ma unikalne resrefy i manifest;
- wariant nie nadpisuje globalnie niepowiazanego asset family;
- HAK zawiera wszystkie wynikowe zasoby wymagane przez wybrany wariant.

### F6. Alpha and Emissive Inspector

Status: PLAN.

Podglad szachownicy alpha, policy alpha/cutout i jawny stan emisji. Edytor pokazuje, czy kanal alpha lub emissive jest zbakowany, emitowany zgodnie z potwierdzonym kontraktem Aurora, czy odrzucony z ostrzezeniem.

### F7. Geometry Inspector

Status: PLAN.

Widok kosztu geometrii per mesh node i primitive:

- triangles i vertices;
- material splits;
- duplicate vertices;
- zero-area faces;
- najbardziej kosztowne czesci;
- stan przed i po remesh/decymacji.

Nie wykonuje automatycznej decymacji bez zapisanej policy i raportu zmiany UV/triangles.

### F8. Skeleton and Weight Inspector

Status: PLAN.

Lista kosci, bind pose, zaznaczenie wplywow wybranej kosci i heatmapa wag. Walidator raportuje ponad cztery wplywy, wagi nienormalizowane, zero-weight vertices i niezgodnosc z wybranym skeleton strategy.

### F9. Animation Inspector

Status: PLAN.

Lista klipow, bind pose, preview czasu, loop/one-shot oraz mapa `source clip -> Aurora/NWN animation name`. Eksport jest blokowany dla animacji bez zaakceptowanego mappingu i przejscia gates skeletonu/wag.

### F10. Proof Packet Builder

Status: PLAN.

Jedna komenda przygotowuje lokalny packet dowodowy:

- generated HAK i manifest zasobow;
- `validation-report.json` i hashe;
- source, Aurora Preview i readback screenshots;
- lista manualnych krokow Toolset/game.

Nie automatyzuje ani nie udaje wizualnego proofu w NWN EE. Finalne screenshoty Toolset/grze nadal wymagaja zatwierdzonego manualnego runbooka.

## Kolejnosc po MVP

```yaml
post_mvp_sequence:
  S1_diagnostics:
    features: [F1, F2, F3, F4]
    reason: "najpierw widzimy i diagnozujemy finalny asset"
  S2_material_control:
    features: [F5, F6, F7]
    reason: "potem bezpiecznie poprawiamy wyglad i budzet"
  S3_rig_animation:
    features: [F8, F9]
    reason: "wymaga dzialajacego kontraktu skin/animation"
  S4_proof_ergonomics:
    features: [F10]
    reason: "pakiet dowodowy ma sens po ustabilizowaniu pozostalych warstw"
```

## Poza zakresem tego backlogu

```yaml
not_planned_as_studio:
  - "manual pixel painting"
  - "full mesh modeling"
  - "manual rig authoring"
  - "timeline animation authoring"
  - "direct binary MDL editing"
  - "automatic NWN Toolset GUI control without separate approval"
```

Do takich zmian sluzy Blender lub inne narzedzie artystyczne, a `meshy2aurora` ponownie importuje wynik przez kontrolowany pipeline GLB.
