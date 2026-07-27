# Audyt bramek pre-push 2026-07-27

Status: `ZAMKNIETY / HISTORYCZNY DRIFT R33-R39 WYJASNIONY`

## Najwazniejsza informacja dla kolejnego agenta

Rozne hashe historycznych modeli r33-r39 i wyjscia obecnego writera nie sa
regresja migracji `sample-3d`. Sa oczekiwanym skutkiem pozniejszej, obowiazkowej
naprawy ABI SkinMesh wykonanej po wlascicielskim negatywnym wyniku r44.

Historyczne r33-r39 mialy na bazowym wezle SkinMesh:

```text
0 controller keys
0 controller data
0 odczytanych base controllerow
```

Obecny writer, zgodnie z naprawa r45, dodaje do kazdego bazowego SkinMesh:

```text
position     type 8   [0,0,0]
orientation  type 20  [0,0,0,1]
```

Dla rodziny z jednym SkinMesh daje to dokladnie:

```text
2 * 12 bajtow controller keys = 24
9 *  4 bajty controller data  = 36
razem                          = 60 bajtow core MDL
raw geometry                   = bez zmian
```

Nie wolno aktualizowac historycznych hashy, nadpisywac proof artifacts ani
tworzyc nowego `rNN` z powodu tej roznicy. Stare artefakty pozostaja
nienaruszalnym zapisem proofu z 2026-07-24. Obecny output jest osobna regresja
writera po r45, a nie nowa wersja tych kandydatow.

## Zakres migracji i bramki

Audyt wykonano przy migracji lokalnych zrodel Meshy z `test-assets/meshy` do
kanonicznego `sample-3d`. Bazowy commit brancha przed commitami migracji:

`ea2ab68b2edf94e9b0b817f25fefc4cd67be256f`

Potwierdzone bramki:

- `assert-canonical-workspace.ps1` - PASS;
- `assert-meshy-asset-layout.ps1` - PASS;
- `cargo fmt --all -- --check` i `git diff --check` - PASS;
- `cargo clippy --workspace --all-targets -- -D warnings` - PASS po
  niebehawioralnym uporzadkowaniu zastanych ostrzezen;
- `cargo test -p m2a-core --lib` - `63 passed`, `1 ignored`;
- `cargo test -p m2a-core --test model_pipeline` - `27 passed`, `1 ignored`;
- domyslny `animated_donor_retarget` - `7 passed`, `7 ignored`;
- runtime-witness `animated_donor_retarget --include-ignored` - `14 passed`;
- kandydatowe testy r33-r39 - PASS jako jawny fail-closed historycznych
  builderow po naprawie r45;
- r40 i r41 - nadal odtwarzaja historyczne hashe, poniewaz maja rigid TriMesh
  i nie maja SkinMesh;
- testy H2 r42/r43 oraz H2 owner gate - PASS;
- `cargo test -p m2a-wasm` - `30 passed`;
- Studio build, `200` testow Vitest oraz `9` browser/worker integration
  testow - PASS.

Lokalny H1 ma SHA-256
`3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`.
Jest zgodny z manifestem `sample-3d`, kodem i historycznym evidence. Migracja
zmienila sciezke odczytu, ale nie bajty zrodla.

## Macierz historyczny kandydat a obecny writer

`Post-r45 SHA-256` jest hashem regresyjnym obecnego writera. Nie zastepuje
historycznego hasha kandydata.

| Profil | Historyczny SHA-256 | Post-r45 SHA-256 | Potwierdzona roznica |
|---|---|---|---|
| r33 weighted runtime | `b82b6d7b9260a05cebb7bb93ed75f2938a210a01bd01bd397f1b63fc60fde25e` | `b4e1ac1e27e2da6f95ce192f28a90601a036d8f73ecb7c6bdbd27609cc173b46` | `+60` core, raw bez zmian, SkinMesh `0 -> [8,20]` |
| r34 rig-only state | `2fe4ad1ae4354335008916cbff3e0f724fedf0119f30f07aa8d5341c3d5b4af5` | `172f166b552cb29556e08a0235bff3f4be239e8c9d5f0938fb9ef2ca6bf1c16d` | `+60` core, raw bez zmian, SkinMesh `0 -> [8,20]` |
| r35 zero-terminated skin | `779d93fa762980ef17448762ba97d8f8775b03335b1d9561b8c2b6483772b3e0` | `eb71c15b4049caaae49ecd099e73909f1717edc8668223618484e2b6f58f2c2f` | `+60` core, raw bez zmian, SkinMesh `0 -> [8,20]` |
| r36 dedicated Aurora root | `459b9954d377c1daab9b12c73a2bf9a64507b5f3cf6d2a6a2ea7d751f680963a` | `b04bf97bf989e0021aa3f622511911cff3313cc2079a9f893d912ce8be370b6b` | `+60` core, raw bez zmian, SkinMesh `0 -> [8,20]` |
| r37 direct-root SkinMesh | `48746e6e0b19bedbdcc8a364ff96cd583848dfa38e06971706bfb69b0341f676` | `2f7e399bde73a91590ad588fc35ec177fd8b6ef4083da644a57a5cbe991a9f16` | `+60` core, raw bez zmian, SkinMesh `0 -> [8,20]` |
| r38 scale-normalized SkinMesh | `039d07cd937430d83006c7d0176aa7659440265417fb7fe53bf73b405563c248` | `6d3a56d9b18bada169da02a82ec00e181fa46810760b4e3c3c95c016d68e2c45` | `+60` core, raw bez zmian, SkinMesh `0 -> [8,20]` |
| r39 controllerless root | `fab5ab98e9225c1553947f17994441273ae4c9bbd5a1d14034721ee3be2d86db` | `144f4d7fb7b39e77869ac8dc1128e6f4c60bcb999178f6eef0dc777db4d1dac3` | `+60` core, raw bez zmian, SkinMesh `0 -> [8,20]` |
| r40 rigid triangle groups | `1809b05370e77f2c2559ec3e6fb354518bb5695ed48600954033175377fc0a40` | ten sam | brak SkinMesh, brak delty r45 |
| r41 full-state rigid | `c6a341ac8fc432d3d3f5862f67e1fcbcad72bb1ab3d2d241ce6421a976261f6b` | ten sam | brak SkinMesh, brak delty r45 |

Wazna korekta nazewnictwa: hash `172f...` jest post-r45 wynikiem r34
rig-only, sprawdzanym jako poprzednik wewnatrz testu o nazwie
`exact_r34_to_native_zero_terminated_skin_changes_only_the_unused_inline_palette_tail`.
Nie jest to wynik r35.

## Dlaczego historia Git wygladala na sprzeczna

Fakty:

- proof artifacts r33-r40 powstaly 2026-07-24;
- dokument
  `audyt-systemowy-dlaczego-generowane-creature-nie-dzialaja-w-nwn-2026-07-26.md`
  zapisuje wlascicielski `not_visible` r44, diagnoze
  `M2A-NWN-SKIN-BASE-CONTROLLERS-MISSING` oraz implementacje r45;
- commit `8813408` (`feat: add creature, placeable, and tile workflows`) dodal
  `273` pliki i `81 269` linii naraz: historyczne testy/kandydaty, pozniejsza
  naprawe writera i dokumentacje;
- `git blame` przypisuje przydzial i emisje `skinBindControllerKeys` oraz
  `skinBindControllerData` temu samemu zbiorczemu commitowi;
- commit migracji `5fee03c` zmienil w testach tylko kanoniczne sciezki assetow.

Zbiorczy commit ukryl rzeczywista chronologie prac. Hashe z 2026-07-24 sa
starsze niz poprawka r45 z 2026-07-26, mimo ze oba stany kodu trafily do Git
jednym commitem 2026-07-27.

## Zachowanie kodu i testow po korekcie

- Historyczne stale SHA-256 i proof artifacts pozostaly bez zmian.
- Buildery r33-r39 nadal porownuja z historycznym hashem i fail-closed.
  Komunikat bledu wyjasnia teraz naprawe r45, dokladna delte `24 + 36 = 60`
  bajtow i prowadzi do tego audytu.
- Ich testy env-gated uznaja tylko ten konkretny, nazwany fail-closed. Inny kod
  bledu lub brak wyjasnienia nadal obala test.
- Bezposrednie testy retargetu czytaja historyczne MDL-e read-only i wymagaja:
  historycznego hasha, jednego SkinMesh bez controllerow, osobnego przypietego
  hasha post-r45, `+60` bajtow core, niezmienionego raw oraz dokladnie
  controllerow `[8,20]`.
- r40 i r41 nadal wymagaja pelnej byte-identical zgodnosci ze starymi hashami.

Powtorzenie:

```powershell
$env:M2A_REQUIRE_RUNTIME_WITNESSES = "1"
cargo test -p m2a-core --test animated_donor_retarget -- --include-ignored
cargo test -p m2a-core --test m0_r33_animated_donor_candidate -- --include-ignored
```

Jesli przyszly wynik nie pasuje do opisanej delty, jest to nowy drift i wymaga
osobnej diagnozy. Sama znana delta r45 nie jest otwartym blockerem i nie
uzasadnia nowej iteracji modelu.
