# Audyt bramek pre-push 2026-07-27

Status: `OTWARTY / BAZOWY DRIFT HASHY I CLIPPY`

## Zakres

Audyt wykonano przed publikacja migracji lokalnych zrodel Meshy z
`test-assets/meshy` do kanonicznego `sample-3d`. Bazowy commit brancha przed
commitami migracji:

`ea2ab68b2edf94e9b0b817f25fefc4cd67be256f`

## Fakty

- `assert-canonical-workspace.ps1` przeszedl.
- `assert-meshy-asset-layout.ps1` przeszedl.
- `cargo fmt --all -- --check` oraz `git diff --check` przeszly.
- `cargo test -p m2a-core --lib` przeszedl: `63 passed`, `1 ignored`.
- `cargo test -p m2a-core --test model_pipeline` przeszedl:
  `27 passed`, `1 ignored`.
- Domyslny `animated_donor_retarget` przeszedl:
  `7 passed`, `7 ignored`.
- H2 r42/r43 testy kandydata przeszly, a H2 owner gate zakonczyl sie
  wynikiem `7 passed`, `1 ignored`.
- `cargo test -p m2a-wasm` przeszedl: `30 passed`.
- Studio build, `200` testow Vitest oraz `9` browser/worker integration
  testow przeszly.

Jawne uruchomienie lokalnych testow runtime-witness:

```powershell
$env:M2A_REQUIRE_RUNTIME_WITNESSES = "1"
cargo test -p m2a-core --test animated_donor_retarget -- --include-ignored
```

wykrylo piec rozjazdow SHA-256 modelu:

| Profil | Aktualny wynik | Zamrozony oczekiwany hash |
|---|---|---|
| r34 zero-terminated skin | `172f166b552cb29556e08a0235bff3f4be239e8c9d5f0938fb9ef2ca6bf1c16d` | `2fe4ad1ae4354335008916cbff3e0f724fedf0119f30f07aa8d5341c3d5b4af5` |
| r36 dedicated Aurora root | `b04bf97bf989e0021aa3f622511911cff3313cc2079a9f893d912ce8be370b6b` | `459b9954d377c1daab9b12c73a2bf9a64507b5f3cf6d2a6a2ea7d751f680963a` |
| r37 reparented skin | `2f7e399bde73a91590ad588fc35ec177fd8b6ef4083da644a57a5cbe991a9f16` | `48746e6e0b19bedbdcc8a364ff96cd583848dfa38e06971706bfb69b0341f676` |
| r39 controllerless root | `144f4d7fb7b39e77869ac8dc1128e6f4c60bcb999178f6eef0dc777db4d1dac3` | `fab5ab98e9225c1553947f17994441273ae4c9bbd5a1d14034721ee3be2d86db` |
| r38 scale-normalized skin | `6d3a56d9b18bada169da02a82ec00e181fa46810760b4e3c3c95c016d68e2c45` | `039d07cd937430d83006c7d0176aa7659440265417fb7fe53bf73b405563c248` |

Indywidualne powtorzenie testu r34 z jednym watkiem odtworzylo ten sam
rozjazd. Test r33 uruchomiony z runtime-witness zakonczyl sie kodem
`M2A-R33-RETARGET-CONTRACT`.

Lokalny H1 ma SHA-256
`3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`,
zgodny z manifestem, kodem i historycznym evidence. Biezacy diff migracji
zmienia sciezke odczytu, ale nie kod generujacy wymienione profile ani ich
zamrozone hashe.

`cargo clippy --workspace --all-targets -- -D warnings` zatrzymal sie na
dziesieciu ostrzezeniach w istniejacym kodzie poza diffem migracji:
`len_zero`, `too_many_arguments`, `field_reassign_with_default`,
`needless_range_loop`, `filter_map_bool_then`, `needless_lifetimes` oraz
`type_complexity`.

## Ocena

Rozjazdy runtime-witness i Clippy sa osobnymi bazowymi bramkami jakosci.
Nie sa wizualnym wynikiem modelu ani zgoda na nowa iteracje. Nie wolno
aktualizowac zamrozonych hashy lub tworzyc kolejnego `rNN` bez osobnej
diagnozy zgodnej z model iteration gate.

Migracja `sample-3d` pozostaje zweryfikowana na poziomie layoutu, identycznosci
wejscia, kompilacji, domyslnych testow Rust/WASM oraz Studio. Pozostale ryzyko:
biezacy workflow CI z `-D warnings` pozostanie czerwony do czasu osobnej,
zakresowo jawnej naprawy Clippy, a runtime-witness wymaga osobnej diagnozy
driftu wzgledem zamrozonych lineage.
