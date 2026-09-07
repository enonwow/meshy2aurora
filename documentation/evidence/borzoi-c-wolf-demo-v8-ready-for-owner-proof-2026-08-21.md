# `m2aborzmod8.mod` — Borzoi `c_wolf` V8 gotowy do proofu właściciela

Moduł wyświetlany w Toolsecie: `Meshy2Aurora Borzoi c_wolf Demo V8`.

Dokładny Area: `Meshy2Aurora Borzoi Test Area V8`
(`m2aborzarea8`).

Status: `ready_for_owner_proof`. Agent nie uruchamiał Toolsetu ani NWN.

## Zamrożone artefakty

- MOD `m2aborzmod8.mod`: 15 205 B, SHA-256
  `a2b43eb8432b757ec9c46d28bc471c477b6763976c673c3bd06ce5c5acdb2853`;
- HAK `m2aborzhak8.hak`: 76 224 077 B, SHA-256
  `63f1840e1ee1128d75031e547990d6b560834af334abd75c32b5719c041b2e23`;
- MDL `m2aborzcre8.mdl`: 25 503 880 B, SHA-256
  `a2749e97a35dd6c41d9dd0cefbb3c20301927d453271b6d90c10ece44bcdfa5f`;
- diffuse `m2aborztex8.tga`: SHA-256
  `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1`;
- minimalny MTR `m2aborztex8_m0.mtr`: SHA-256
  `8abe7b15878889f1c1303ab665b48142e1d2c2e3b6a134c2a122c92a952197c0`.

Źródłem pozostaje kanoniczny `source-p300k.glb`, SHA-256
`f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda`,
z dokładnie 300 000 trójkątów.

## Dlaczego V8 nie kopiuje neutralnego bindu wilka

Pełny audit wykazał, że neutralne pivoty retailowego `c_wolf` nie są zgodne z
proporcjami borzoja. Bezpośrednie zastąpienie bindu i wariant z pojedynczą
warstwą korekcyjną zostały zatrzymane przez niezmienioną bramkę deformacji.

V8 używa poprawnego retargetingu supermodelu:

- immutable `c_wolf` weryfikuje 30 nazw i hierarchię carrierów, 42 animacje,
  osiem wymaganych klipów oraz sześć zdarzeń;
- kontrakt exact-inspection ma SHA-256
  `1142432ba999a5a027d25bce7a97df3e314b4a51612c2c2dd3e78ae37ed758c8`;
- carrier zachowuje neutralne pivoty i wagi własnego psa;
- model nie zawiera lokalnych animacji i odwołuje się do `c_wolf` jako
  supermodelu;
- wszystkie dziewięć ważonych regionów anatomicznych, w tym `tail_tip`, jest
  obecnych w każdym badanym klipie.

`referenceBindVerified=false` w raporcie korekcji jest zamierzone: oznacza, że
retailowy bind nie został skopiowany do proporcjonalnie innego celu. Jednocześnie
`bindPoseCompatible=true` oznacza zweryfikowaną strategię retargetowanego bindu
targetu, a nie fałszywe twierdzenie o identyczności macierzy.

## Wynik offline

- `motionCompatible=true`, `bindPoseCompatible=true`,
  `skinBindCompatible=true`;
- profil jakości:
  `REFERENCE_SUPERMODEL_SAMPLED_SURFACE_AND_WEIGHTED_ANCHORS_V4` — `PASS`;
- hard edges `1 949 965 / 2 069 999`;
- soft edges `6 316 945 / 8 279 999`;
- collapsed triangles `17 755 / 20 700`;
- expanded triangles `57 239 / 69 000`;
- seam violations `820 131 / 926 422`;
- minimalny two-sided MTR: `twosided=1`, bez tangentów, normal/specular i TXI;
- deterministyczny replay: 21 plików, 0 różnic bajtowych.

## Instalacja i scena

Dokładne pliki zostały skopiowane do wcześniej nieistniejących celów i po
kopii zweryfikowane SHA-256:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aborzmod8.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aborzhak8.hak`.

Moduł ma jeden HAK `m2aborzhak8`, Appearance row `848` i creature template
`m2aborzutc8`. Pies stoi w `(10.0, 14.5, 0.0)` naprzeciw gracza.

## Kryteria proofu właściciela

1. Otworzyć `m2aborzmod8.mod` i Area
   `Meshy2Aurora Borzoi Test Area V8`.
2. Potwierdzić widoczność psa oraz brak crasha.
3. W NWN sprawdzić idle, chód/bieg i walkę.
4. Szczególnie potwierdzić widoczny ruch `Wolf_tail` i `Wolf_tailend`.
5. Zgłosić osobno wynik geometrii/tekstury oraz animacji ogona.

Do czasu wyniku właściciela osie pozostają:
`modelVisibility=not_tested`, `proofCompleteness=missing`.
