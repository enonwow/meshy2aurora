# c-kocrachn-kontrakt-wizualny-codex.md

Status 2026-07-09: TECHNICAL PROXY ONLY. `c_kocrachn` sluzy do testowania creature pipeline i referencji proporcji. Nie jest assetem The Last City, nie jest finalnym designem i nie jest dowodem dzialania `meshy2aurora`.

Data: 2026-07-09  
Status: WSTEPNY KONTRAKT WIZUALNY  
Cel: opisac, jak ma wygladac wzorzec `c_kocrachn` / `m2a_koc01`, zanim powstana prompty 2D i model z Meshy.

## Zasada

Status: POTWIERDZONE.

Najwazniejsze informacje dla sample to wyglad, sylwetka i proporcje. Format MDL/HAK/2DA jest tylko sposobem dostarczenia potwora do gry. Prompty 2D i model Meshy nie moga powstac z samego resrefa `c_kocrachn`; musza wynikac z kontraktu wizualnego i screenshotow referencji.

`C:\Projects\aurora-web` moze byc uzyty tylko jako material porownawczy do odczytu/obejrzenia. Docelowe potwierdzenie wizualne musi pochodzic z NWN EE Toolset/gry albo z lokalnych screenshotow referencyjnych zapisanych w `C:\Projects\meshy2aurora\sample-2d\_reference\c_kocrachn\`.

## Potwierdzona referencja techniczna

Status: POTWIERDZONE z lokalnego proofu referencyjnego.

```yaml
reference:
  resref: "c_kocrachn"
  appearance: "Kocrachon"
  model_type: "S"
  race_model: "c_kocrachn"
  supermodel: "c_Horror"
  size: 4
  natural_equipment:
    right_weapon: "Kocrachon Claw1d4 + 1"
    left_weapon: "Kocrachon Claw1d4 + 1"
    bite: "Kocrachon Bite1d6 + 3"
    armor_hide: "Kocrachon Hide"
  source_skin_meshes:
    - "Lshinmesh01"
    - "Rshinmesh01"
    - "bodymesh01"
  reference_view_only:
    frame: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-source-skin-bindpose-v193\\c_kocrachn-cpause1-frame-00000ms.png"
    state: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-source-skin-bindpose-v193\\c_kocrachn-cpause1-state.json"
```

## Kontrakt wygladu

Status: POTWIERDZONE WSTEPNE z obrazu referencyjnego; wymaga docelowych screenshotow NWN/Toolset.

`c_kocrachn` ma wygladac jak owadzi, chitynowy potwor typu Kocrachon, nie jak humanoid i nie jak zwykly pies/wilk/szczur.

```yaml
visual_contract:
  archetype: "owadzi/chitynowy potwor fantasy, Kocrachon"
  posture:
    - "neutralna poza stworzenia, nie hero pose"
    - "kilka cienkich odnozy opartych o ziemie"
    - "korpus uniesiony nad podlozem"
    - "sylwetka asymetrycznie pochylona w gore przez szyje/glowe, ale anatomia lewo-prawo ma byc symetryczna"
  silhouette:
    must_have:
      - "duzy segmentowany jasnoszary/beżowy korpus lub odwłok"
      - "ciemniejsza chitynowa glowa/gorny segment"
      - "dlugie cienkie kosci/odnoza z ostrymi koncowkami"
      - "widoczny aparat gryzacy albo glowa sugerujaca bite attack"
      - "przednie konczyny/klawy sugerujace left/right claw attacks"
    must_not_have:
      - "humanoidalna stojaca sylwetka"
      - "ludzka twarz"
      - "ludzkie rece lub dlonie"
      - "trzymana bron"
      - "zbroja jako ubranie"
      - "skrzydla rozpostarte"
      - "dlugie cienkie anteny jako kluczowy element, bo Meshy moze je zepsuc"
  materials:
    palette:
      - "ciemny braz/czern na glowie i chitynie"
      - "brudny beż/szary na segmentowanym korpusie"
      - "kościany/tan na odnogach i pazurach"
    surface:
      - "matowa chityna"
      - "segmenty/pancerz"
      - "brudny organiczny fantasy look"
  scale_and_readability:
    - "czytelna sylwetka z kamery izometrycznej NWN"
    - "proporcje wazniejsze niz mikrodetaile"
    - "konczyny nie moga byc zbyt cienkie, aby nie znikaly po remesh/decymacji"
    - "glowa, korpus i odnoza musza byc rozroznialne w 1-2k tri"
```

## Co Mateusz powinien dostarczyc

Status: POTWIERDZONE jako wymaganie wejscia dla promptow.

```yaml
required_reference_screenshots:
  target_dir: "C:\\Projects\\meshy2aurora\\sample-2d\\_reference\\c_kocrachn\\"
  required:
    - "front.png"
    - "side.png"
    - "quarter.png"
    - "manifest.yaml"
  recommended:
    - "back.png"
    - "top.png"
    - "closeup-head.png"
    - "closeup-body-segments.png"
    - "closeup-legs-claws.png"
  manifest_fields:
    reference_model: "c_kocrachn"
    appearance: "Kocrachon"
    source_tool: "nwnexplorer | NWN EE Toolset | NWN EE game | aurora-web reference-only"
    pose: "bind pose | cpause1 | unknown"
    view_names: ["front", "side", "quarter"]
    notes: "czy poza jest animowana, czy czysta bind pose"
```

## Kryteria akceptacji sample 2D

Status: HIPOTEZA WDROZENIOWA; do uzycia przy ocenie obrazow z OpenAI.

```yaml
acceptance:
  pass:
    - "na pierwszy rzut oka widac owadziego Kocrachona"
    - "sylwetka pasuje do referencji: niski/sredni potwor, uniesiony korpus, dlugie odnoza"
    - "front/side/quarter pokazuja tego samego potwora"
    - "poza neutralna, wszystkie glowne odnoza widoczne"
    - "materialy: chityna, segmenty, brudny organiczny pancerz"
  reject:
    - "wyglada jak humanoid, jaszczur, wilk, demon dwunozny albo zwykly robak bez fantasy formy"
    - "ma bron, ubranie lub zbroje zamiast naturalnego pancerza"
    - "ma zbyt duzo cienkich anten/kolcow, ktore Meshy zgubi"
    - "pozy w widokach roznia sie tak mocno, ze Meshy zrobi inny model"
    - "brakuje czytelnej glowy, korpusu albo konczyn"
```

## Informacja dla Cloud

Status: POTWIERDZONE.

Przed `sample-2d-prompty-codex.md` potrzebny jest opis wygladu plus screenshoty referencyjne. Prompty maja najpierw utrzymac proporcje i czytelna sylwetke `c_kocrachn`, a dopiero potem dodawac wariant artystyczny. Dla pierwszego sample nie robimy kreatywnego redesignu; robimy technicznie uzyteczny potwor testowy zgodny z Kocrachonem.
