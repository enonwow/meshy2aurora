# P20K placeable stress V1 — `ready_for_owner_proof`

1. Exact test-module `.mod` filename: `m2a_p20k_mod.mod`
2. Module name shown in Toolset: `Meshy2Aurora P20K Placeable Stress Test`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Data zamrożenia: `2026-07-25`

Status: `ready_for_owner_proof`

Cel tego niezależnego lane'u to sprawdzenie jednego statycznego placeable'a
z dokładnie `20 000` trójkątów w jednym meshu — blisko granicy `21 845`
trójkątów na mesh w aktualnym NWN:EE. Nie jest to rewizja ani zastępstwo
widocznego S1 Ritual Pedestal. Pozytywny dowód S1 pozostaje zachowany.

## 1. Zakres i klasyfikacja faktów

- **Fakt z dekompilacji/formatu lokalnego:** własny binary reader odczytuje
  vertex count jako `u16`, tablicę liczników indeksów jako `u32` oraz surowe
  wartości indeksów jako `u16`.
- **Fakt z własnego readbacku:** wygenerowany MDL ma jeden mesh, `10 102`
  wierzchołki, `20 000` faces oraz jeden stream `60 000` indeksów; każdy
  indeks mieści się w vertex buffer.
- **Fakt ze źródła Beamdog:** patch `87.8193.35` poprawił obsługę danych
  wierzchołków i dopuścił do trzech razy więcej wierzchołków na mesh.
- **Źródło społecznościowe uzupełniające:** aktualna dokumentacja NWN Wiki
  podaje `65 535 / 3 = 21 845` trójkątów na pojedynczy mesh/node w
  NWN:EE `1.87.8193.35`; większą geometrię należy dzielić.
- **Wniosek implementacyjny:** wspólny writer odrzuca teraz segment mający
  więcej niż `65 535` wpisów indeksowych. Dla listy trójkątów daje to
  dokładnie `21 845` trójkątów.

Źródła zewnętrzne:

- [Beamdog NWN:EE changelog](https://nwn.beamdog.net/docs/)
- [NWN Wiki — Models / Model Triangle Limit](https://nwn.wiki/spaces/NWN1/pages/38175602/Models)
- [xoreos NWN1 binary MDL template](https://github.com/xoreos/xoreos-docs/blob/master/templates/NWN1MDL.bt)

## 2. Exact candidate identity

| Pole | Wartość |
|---|---|
| root lineage | `proof-output/p20k-placeable-stress-v1-20260725` |
| lane | `P20K_SINGLE_MESH_STRESS_V1` |
| Area resref | `m2a_p20k_ar` |
| HAK filename | `m2a_p20k_hak.hak` |
| HAK resref | `m2a_p20k_hak` |
| model resref | `m2a_p20k_rel` |
| texture resref | `m2a_p20k_tex` |
| UTP/archive resref | `m2a_p20k_utp` |
| object Tag | `m2a_p20k_stress_reliquary` |
| display name | `M2A 20K Stress Reliquary` |
| physical `placeables.2da` row | `16501` |
| placement | `X=10.0, Y=14.5, Z=0.0, Bearing=0.0` |
| profile | `STATIC_PLACEABLE` |
| materialization count | `1` |

Machine-readable handoff:
`proof-output/p20k-placeable-stress-v1-20260725/ready-for-owner-proof.json`.

## 3. Geometria

| Własność | Wartość |
|---|---:|
| mesh/segment count | 1 |
| vertex count | 10 102 |
| triangle/face count | 20 000 |
| index count | 60 000 |
| limit indeksów na mesh | 65 535 |
| limit trójkątów na mesh | 21 845 |
| zapas do granicy | 1 845 trójkątów |
| deformation | `RIGID` |
| mesh type po readbacku | `3` |
| render po readbacku | `1` |

Model jest syntetycznym, zamkniętym `Stress Reliquary`: bryłą obrotową o
stu pierścieniach profilu i stu sektorach, z własnymi normalnymi i UV.
Jego payload nie kopiuje geometrii, tekstur ani innych danych retail/CEP.

Ważne ograniczenie zakresu: fixture rozpoczyna się w wspólnym
`AuroraModelIrV1`, przechodzi przez wspólny binary MDL writer oraz pełny
placeable packaging. Nie dowodzi jeszcze, że domyślna ścieżka importu
`Meshy GLB -> Profile A` przyjmuje 20 000 trójkątów — jej konserwatywny
guardrail nadal wynosi `10 000`. P20K testuje tutaj writer, resolver i runtime
Aurory/NWN, nie wysokopoligonowy ingest GLB.

## 4. Exact outputs

| Zasób | Bytes | SHA-256 |
|---|---:|---|
| `m2a_p20k_mod.mod` | 13 408 | `ac2f74d4f5a8453a65faa1c3744d5e358d69cb91ab1d1fe55b239ab783f48b55` |
| `m2a_p20k_hak.hak` | 4 341 510 | `7fae061c527fbe2a4d42d4205f37f4b71c2e9bd8631f67bcd2496d3fac3631dc` |
| `m2a_p20k_rel.mdl` | 1 124 724 | `fdea2ec5b51a54806533533ba2bb711107f078b7f434cd37442a80f063a129a4` |
| `m2a_p20k_tex.tga` | 196 652 | `5457e9c5109ac2c841bb6c1837bf8bb34852d1b561a2875bf20ebeb00518ddbf` |
| `placeables.2da` | 3 019 878 | `70243dc9ce08e705587089512364f65a16501a565369f4d29ae8ff73e0a28c85` |
| `m2a_p20k_utp.utp` | 1 905 | `faf7db43c4c4192233b3c0bb604042cfe81461445d0707788bc11ef937a7d25f` |
| `m2a_p20k_ar.are` | 2 483 | `1937ff9971757779522353c137f7b89c280a4423042c54db4921816ce3ad511f` |
| `m2a_p20k_ar.git` | 2 585 | `ad57e3a165df3fbe013736b4700e7afbd1dd6259f8aa92739c9dd2e7c8053bcc` |
| `m2a_p20k_ar.gic` | 503 | `447dd89f827a7f559e24b250bf21856d6a0eb938b52973499d2e02b02ecce0c8` |
| `placeablepalcus.itp` | 1 465 | `19f0bcc4222ea409e7d429d8e3a072d6e2bfb354cc20facc9bf0c76af295cb29` |
| `module.ifo` | 2 194 | `5e8ec7d87291d2dbc200e912e994ad7cd50fc4f5e71f07b154819fbedcd45899` |
| `repute.fac` | 1 889 | `558f119cc9d39c5ddd5c93a3e8d88954ac83417fd7539d25278e9f8cf7b2017d` |
| `placeable-report.json` | 4 262 | `b0ce5127ab4ea813459009ba0e1b7aea44435616ca7b25e8483a3b2b32818225` |
| `geometry-readback.json` | 563 | `c8703ee88852fb0579875655d6a3cb969740456e68a19a608fb8564ae6224ae2` |
| `source-model-ir.json` | 3 149 141 | `b160198727f89d13fbe7d468bbf51eeb461c473d67c177c5c1708f74ea9e43c5` |
| `source-spec.json` | 544 | `8c7f9ce378d272a8e8d4275477403448dc02240923908a2fd3f3dad4981f546b` |
| `ready-for-owner-proof.json` | 3 515 | `4ca1097ee5ef81d1af89424ec259967d3914e5d605f0d7d2bc37a68dc240207e` |

Resolver chain:

`GIT/UTP Appearance=16501 -> placeables.2da[16501].ModelName=m2a_p20k_rel -> m2a_p20k_rel:2002 -> m2a_p20k_tex:3`.

## 5. Verification checklist

- [x] Kanoniczny workspace potwierdzony.
- [x] Dodany czerwony test TDD: `21 846` trójkątów wcześniej przechodziło.
- [x] Wspólny writer przyjmuje `20 000` trójkątów.
- [x] Wspólny writer odrzuca `21 846` trójkątów kodem `M4-MESH-LIMIT`.
- [x] Model ma dokładnie jeden segment/mesh.
- [x] Własny MDL readback potwierdza `10 102 / 20 000 / 60 000`.
- [x] HAK i MOD przechodzą własny readback.
- [x] Resolver 2DA/UTP/GIT/GIC/MDL/TGA jest spójny.
- [x] `cargo fmt --all -- --check` przechodzi.
- [x] Pełne `cargo test -p m2a-core` przechodzi.
- [x] Kandydat został zmaterializowany dokładnie jeden raz.
- [x] Exact MOD/HAK zostały natychmiast zainstalowane w natywnych katalogach
  NWN zgodnie z `AGENTS.md`.
- [x] Cele były nieobecne przed kopiowaniem.
- [x] Kopia użyła trybu no-clobber.
- [x] Hashe obu celów są byte-identical ze źródłami.
- [ ] Aurora Toolset: `modelVisibility=visible`, `proofCompleteness=verified`.
- [ ] NWN runtime: `modelVisibility=visible`, `proofCompleteness=verified`.

## 6. Stan natywnych katalogów

Oba cele były nieobecne przed kopiowaniem, a po instalacji mają następującą
tożsamość:

| Rola | Exact destination | SHA-256 |
|---|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_p20k_mod.mod` | `ac2f74d4f5a8453a65faa1c3744d5e358d69cb91ab1d1fe55b239ab783f48b55` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_p20k_hak.hak` | `7fae061c527fbe2a4d42d4205f37f4b71c2e9bd8631f67bcd2496d3fac3631dc` |

Instalacja nie uruchomiła ani nie kontrolowała Aurora Toolset/NWN. Handoff
otrzymał status `ready_for_owner_proof` dopiero po weryfikacji obu kopii.

## 7. Handoff do testu właściciela

Exact, byte-identical MOD/HAK są już zainstalowane:

1. Otworzyć `m2a_p20k_mod.mod`.
2. Potwierdzić nazwę modułu
   `Meshy2Aurora P20K Placeable Stress Test`.
3. Otworzyć Area `Meshy2Aurora M0 binary vertical-slice area`
   (`m2a_p20k_ar`).
4. Zaznaczyć `M2A 20K Stress Reliquary`.
5. Potwierdzić widoczność modelu w Toolsecie.
6. Uruchomić ten sam moduł w NWN i potwierdzić widoczność tego samego obiektu
   przed graczem.

Brak PWK pozostaje osobnym statusem
`collisionCompleteness=pwk_not_implemented` i nie wpływa na werdykt renderingu.
