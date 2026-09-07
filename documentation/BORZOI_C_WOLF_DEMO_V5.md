# `m2aborzmod5.mod`

> Amendment 2026-08-21: właściciel zgłosił fatalny crash NWN podczas
> ładowania dokładnego V5. Status `ready_for_owner_proof` jest wycofany;
> bieżący status to `owner_runtime_crash_recorded`. Szczegóły i hashe:
> `documentation/evidence/borzoi-c-wolf-demo-v5-owner-nwn-crash-2026-08-21.md`.

Toolset module name: `Meshy2Aurora Borzoi c_wolf Demo V5`\
Exact Area name: `Meshy2Aurora Borzoi Test Area V5`

Date: 2026-08-21

Status: `owner_runtime_crash_recorded / iteration_gate_closed`. Dokładne MOD i
HAK pozostają zainstalowane w natywnych katalogach NWN i zweryfikowane bajt w
bajt. Właściciel uruchomił proof NWN, który zakończył się fatalnym crashem
podczas ładowania Area. Agent nie uruchamiał żadnej z tych aplikacji.

## Co zmienia V5

- exact źródło zachowuje 300 000 trójkątów i wszystkie trzy obrazy Meshy;
- własna ciągła klatka deformacyjna ma 54 wierzchołki i 42 trójkąty;
- barycentryczny transfer na 233 832 wierzchołki renderowe zastępuje rigid
  assignment 1 836 rozłącznych komponentów, który rozerwał V4;
- cztery iteracje wygładzania topologicznego zmniejszają największą sąsiednią
  różnicę wag z `2.0000002` do `1.210592`;
- wszystkie 16 segmentów nóg ma niepuste regiony główne, a cztery klastry łap
  są obecne;
- model ma dokładnie 30 bezpośrednich nośników o nazwach/topologii kontraktu
  `c_wolf`, zero correction nodes i zero lokalnych animacji;
- pełny materiał NWN:EE zawiera diffuse, normal, specular/gloss, TXI oraz MTR
  z zachowanym `twosided`;
- model dziedziczy inwentarz 42 animacji `c_wolf`; osiem wymaganych klipów jest
  dokładnie próbkowanych przez offline oracle.

## Offline motion quality

Profil `REFERENCE_SUPERMODEL_SAMPLED_SURFACE_AND_PAW_COMPLETENESS_V3` zakończył
się statusem `PASS`.

| Metryka | Wynik | Limit |
|---|---:|---:|
| Krawędzie poza `[0.5, 2.0]` | 6 316 945 / 207 000 000 (3,052%) | 8 279 999 (4%) |
| Krawędzie poza `[0.25, 4.0]` | 1 949 965 / 207 000 000 (0,942%) | 2 069 999 (1%) |
| Zapadnięte trójkąty | 17 755 / 69 000 000 (0,0257%) | 20 700 (0,03%) |
| Rozszerzone trójkąty | 57 239 / 69 000 000 (0,0830%) | 69 000 (0,1%) |
| Naruszenia bliskości szwów | 820 131 / 37 056 910 (2,213%) | 926 422 (2,5%) |
| Brakujące klastry łap | 0 | 0 |

`worldNormalOppositionCount`, przechodzenie łap przez oś, kontakt w animowanym
`cpause1` i różnica pierwszej klatki względem bind pose pozostają jawnie
zapisane jako diagnostyka. Nie blokują V5, ponieważ raw world-normal dot product
oraz absolute-zero odrzucały także poprawnego retailowego `c_dog` dziedziczącego
po `c_wolf`. Właściciel ocenia finalny ruch na dokładnym kandydacie.

## Exact owner handoff

- Test module file: `m2aborzmod5.mod`
- Ordered HAK: `m2aborzhak5`
- Creature blueprint: `m2aborzutc5`
- Appearance row: `848`
- Model resref: `m2aborzcre5`
- Base material resref: `m2aborztex5`
- Creature placement: `(10.0, 14.5, 0.0)`
- Creature orientation: `(0.0, -1.0)`, w stronę punktu startowego

## Immutable artifact identity

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2aborzmod5.mod` | 15 205 | `01c346d47af823bf429c6fdc8c3181aab9f5109858ee56f6ff2ad690e9a49c47` |
| `m2aborzhak5.hak` | 197 683 232 | `da24c46d22d45b14f798a32bde9b44f1464c747be9e73d29e0f2647ffacce4ad` |
| `m2aborzcre5.mdl` | 29 522 024 | `1f4fe421779cf472e16336588084d92e21d0174c2b9091ef326f9373bb6d4adc` |
| `m2aborztex5.tga` | 50 331 692 | `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1` |
| `m2aborztex5_n0.tga` | 50 331 692 | `027b80b7a30f58db442c8705ac38e5824ae076e44db96da10665e2f8f2336927` |
| `m2aborztex5_s0.tga` | 67 108 908 | `649ea9bdc05609361a47475b0c881bf4330795260e83e9a60bd8198c676f13ff` |

Canonical packet: `proof-output/borzoi-c-wolf-demo-v5-20260821`.

Native installation:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aborzmod5.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aborzhak5.hak`

Oba hashe instalacji są identyczne z canonical source. Drugi niezależny build
porównał 26 plików i był bajtowo identyczny.

## Kryteria oceny właściciela

1. Pies jest widoczny, stoi na podłożu i jest zwrócony do gracza.
2. Tułów, szyja, głowa i sierść nie tworzą rozerwanych płatów ani dużych kolców.
3. Przednie łapy są rozstawione, wszystkie cztery łapy uczestniczą w ruchu.
4. Idle, chód, bieg oraz atak zachowują psią sylwetkę.
5. Diffuse, normal/specular i dwustronna sierść są widoczne zgodnie z modelem.

Do czasu wyniku właściciela osie pozostają:
`modelVisibility=not_tested`, `proofCompleteness=missing`,
`qualityVerdict=not_tested`.
