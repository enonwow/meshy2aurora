# M0 r30 — Retail runtime-conformance integration candidate — 2026-07-21

## Wynik

Jednorazowo zmaterializowano exact `r30` jako pierwszy kandydat integracyjny po
code-quality pass. Nie uruchomiono Aurora Toolset ani NWN i nie wykonano Save.
Kandydat używa wyłącznie
`RetailDirectCreatureType5DummyV1`; profil/provenance CEP nie jest obecny.
Każdy z siedmiu stanów type `5` odwzorowuje pełną dwuwęzłową topologię bazy
(`root -> m2a_seg_1`) jako `0x01` dummy nodes, bez mesh/skin/raw-MDX payloadu w
drzewie animacji.

Produkcyjny verifier ponownie odczytał wygenerowany MDL i związał summary:

- base nodes `2`, base mesh nodes `1`, base skin nodes `0`, max depth `1`;
- animations `7`, type histogram `{ "5": 7 }`;
- full topology projections `7`, all-generic-dummy projections `7`;
- CEP rigid-placeholder projections `0`;
- canonical summary SHA-256
  `a7285967b142ea9ce7d4365a501496e0f8949568c436b1698aabe2954e425d31`.

## Admission i exact delta

Admission jest exact failed runtime proof r29:

| Evidence | SHA-256 | Wynik |
|---|---|---|
| `proof-output\m0-r29-all-state-identity-20260721\live\runtime\aur-s07-runtime-packet.json` | `4539b88024fb1b8977cc2a2f92554f6a78cdf0234570bbc01423bc55488b6fb6` | `modelVisibility=not_visible`, `proofCompleteness=failed` |
| `proof-output\m0-r29-all-state-identity-20260721\live\runtime\r29-nwn-runtime.png` | `5836a5e43dd9d29c57522cbbd7a7d1f87808f476e5d4da2dda64d0456634e10d` | fresh candidate-bound capture |
| `proof-output\m0-r29-all-state-identity-20260721\live\runtime\r29-close-rollback-evidence.json` | `7bb44a4d0b1458b9d600ff81ebdc9ccae1e31a4854ff6a908d19f496c8e10925` | cleanup `nwmain=0`, `nwtoolset=0` |
| `documentation\evidence\m0-runtime-conformance-state-projection-fix-2026-07-21.md` | `dd16cc4b113d578a70b0bdc4a38e5ff75b8865fe673c6888ebb33a26004a9f40` | admitted code-fix evidence |

R30 zmienia state projection z historycznego placeholderowego drzewa r29 na
profil Retail zgodny z witnessami. Bazowa geometria i materiał mają nadal
semantic digest
`f4172130a98ff6b3b4f3c9e67e9b915d0b9da72f7dff6293eabed51e369e7b6c`.
Raw MDX pozostaje byte-identical z r29: `95096` B, SHA-256
`731749c2e501305356a12939e519ee27a6bf6221bfbb925477d28e8f7fe58507`.
TGA i resrefy modelu/tekstury również są zachowane.

Druga konieczna część delty to produkcyjny `FullRuntimeAppendV1`: exact pełne
wejście `C:\Projects\meshy2aurora\local-reference-assets\appearance.2da`
(`6901169` B, SHA-256
`815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a`)
ma `15100` physical rows. Output zachowuje byte-identical source prefix, dodaje
dokładnie jeden physical row `15100` i ma `15101` rows. Fixture wiąże właśnie
ten physical row, a nie historyczny izolowany row `848`.

## Exact artifacts i instalacja

| Element | Absolute path | Bytes | SHA-256 |
|---|---|---:|---|
| MOD | `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\generated\m2a_m0r30.mod` | 13173 | `8c87317956053cecdf40d939c9061a174943b982a8db2c94d35aa2fba44043df` |
| HAK | `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\generated\m2a_m0r30.hak` | 19634596 | `6694c0fb7f16665025d312dfd35c4bcbb293fccdb0760c110492a5eb2d576d09` |
| MDL | `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\generated\m2a_m0p01.mdl` | 150024 | `43a5cbfa1a20146ec0990ce7ee980d70a54d7f72689781a58c7d9614bc35b8c7` |
| TGA | `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\generated\m2a_m0t01.tga` | 12582956 | `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` |
| output `appearance.2da` | `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\generated\appearance.2da` | 6901360 | `48d313b75761809e2231c99ad51e17d67632c1ce10e70b87fa8b1ccdfbc474a6` |

Fresh identity to MOD `m2a_m0r30`, Area `m2a_m0a30` i ordered HAK list
dokładnie `[m2a_m0r30]`. Targety native były absent; zapisano je przez atomic
`CreateNew`, po czym source/destination SHA musiały być równe:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0r30.mod` —
  `8c87317956053cecdf40d939c9061a174943b982a8db2c94d35aa2fba44043df`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r30.hak` —
  `6694c0fb7f16665025d312dfd35c4bcbb293fccdb0760c110492a5eb2d576d09`.

Kandydat został zmaterializowany dokładnie raz. Żaden artefakt wcześniejszej
lineage nie został nadpisany ani skopiowany jako r30.

## Profile i strong contract

- binary bootstrap:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r30-retail-runtime-conformance-binary-bootstrap-v1.json`,
  SHA-256 `54684a357749797ed075f01f9a04027f0c86d3fdd6cb1da1a34e2b3880d72eb4`;
- no-Save Gate B:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r30-retail-runtime-conformance-toolset-proof-v1.json`,
  SHA-256 `64ec57c20824274152b50edcf9ba094a5fb52063a797383b1647f23d370ea43b`;
- machine-readable lineage contract:
  `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\m0-r30-retail-runtime-conformance-lineage-contract-v1.json`,
  SHA-256 `d97b47d47805c685db64e3e0d03ef3a410ac9b2b51dedc08a53c735600ebda8d`.

Kontrakt ponownie hash-verifyuje admission, evidence, source/installed
MOD+HAK, profile, HAK resources, raw MDX, TGA, full appearance prefix/binding,
state-projection summary/digest i osiem negatywnych mutacji. Runtime profile
nie istnieje i nie może powstać przed zaakceptowanym, fresh Gate B r30.

## Offline validation

PASS:

```powershell
$env:M2A_REQUIRE_RUNTIME_WITNESSES='1'
cargo test -p m2a-core --test runtime_witness_conformance -- --ignored
cargo test -p m2a-core --example materialize_m0_runtime_candidate
cargo test -p m2a-core --test mdl_writer --test model_pipeline --test runtime_witness_conformance
cargo test -p m2a-core --tests -- --skip swapped_field_indices_are_in_bounds_but_violate_canonical_encounter_order --skip phase_nine_field_indices_layout_precedes_phase_ten_oversized_locstring_limit
cargo check --workspace
cargo test -p m2a-wasm
node tools\m0-r30-retail-runtime-conformance-lineage.contract.test.mjs
```

Forced witness suite wykonał `2 passed`; nie był to zwykły run ze skipem.
Targeted suite: example `1 passed`, writer `32 passed`, pipeline `17 passed`,
witness-policy `1 passed` i dwa jawnie ignored bez forced env. Szeroki suite
core oraz workspace/wasm są zielone; dwa wcześniej znane, niezależne testy GFF
zostały jawnie pominięte w tej kwalifikacji.

Central shared-tooling offline PASS:

- `binary_bootstrap_profile_schema_valid`;
- `binary_bootstrap_structural_readback_valid`;
- `binary_native_geometry_plan_valid`;
- każdy wynik ma `startsToolset=false`, `startsNwn=false`,
  `usesGlobalInput=false`, `noLocalToolsetAdapter=true`.

## Jedyny proof handoff

Następny dopuszczony krok to jedno no-Save Gate B exact r30 przez shared
`aurora-toolset-operate` + `aurora-toolset-prove`, z profilem:

```powershell
$toolsetProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r30-retail-runtime-conformance-toolset-proof-v1.json'
```

Proof owner ma otworzyć exact installed/hash-verified MOD `m2a_m0r30`, Area
`m2a_m0a30`, wybrać occurrence `0` exact fixture
`Meshy M0 binary vertical-slice fixture`, wykonać readback i fresh validated
`TScrollBox`, a następnie zapisać profile-bound Gate B packet/capture. Profil ma
`noSave=true` i `noLocalToolsetAdapter=true`. Dopiero accepted Gate B packet i
jego SHA odblokują materializację runtime profile tej samej lineage.

## Gate B accepted i runtime handoff

Powyższa sekcja zachowuje historyczny stan przed live proofem. Fresh Gate B
exact r30 został następnie zapisany jako `modelVisibility=visible` oraz
`proofCompleteness=verified`, bez Save/Build/repacku i bez uruchamiania NWN:

- packet
  `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\live\toolset\r30-gate-b-packet.json`,
  SHA-256 `c2ebbf98c5cc7c8b431bbab58c868c2d2956c55befb6d385816268fade3506b5`;
- fresh validated `TScrollBox` PNG
  `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\live\toolset\r30-after-selection.png`,
  SHA-256 `aeb49f6cb4335f1e91a1f33aa06675b94ec829e006cbb2d905fa0cf19b0fc01b`;
- capture result
  `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\live\toolset\r30-after-selection.json`,
  SHA-256 `bf2b252fb65f367a05f3ba2acf6722a36b91e34b0820f2f7a7a175b3b340df5f`.

Po tym gate zmaterializowano dwuwarstwowy runtime handoff:

- central-compatible AUR-S07 v1 profile
  `C:\Projects\meshy2aurora\proof-profiles\m0-r30-retail-runtime-conformance-runtime-v1.json`,
  SHA-256 `39b0eaa64f5bd2a49a3f92e47036c5687cc12c84c69491b164034d75faa9fab2`;
- immutable project-owned binding sidecar
  `C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\m0-r30-runtime-v2-binding-sidecar-v1.json`,
  SHA-256 `10c82232d820758ef408bb0b296500ae97ed94c23977ec50e1ce3fe3bda9f7f7`.

Sidecar zachowuje exact Gate B packet/capture, MOD/Area/ordered `[m2a_m0r30]`,
fixture physical row `15100`, MDL/TGA/full appearance output oraz produkcyjny
`RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1` summary/digest. Wiąże również exact
full input table, `FULL_RUNTIME_APPEND_V1`, jeden append, output row count
`15101` i byte-identical source prefix. Profil CEP ani CEP provenance nie są
dopuszczone.

`node tools\m0-r30-runtime-profile.contract.test.mjs` przechodzi z `14`
negatywnymi przypadkami, w tym wrong projection profile/provenance, wrong table
scope/source hash/source path/prefix, złym Gate B hashem, row, Area, MOD i HAK.
Publiczny centralny dry-run:

```powershell
Set-Location C:\Projects\aurora-web
$profile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r30-retail-runtime-conformance-runtime-v1.json'
$outDir = 'C:\Projects\meshy2aurora\proof-output\m0-r30-retail-runtime-conformance-20260721\live\runtime'
node backend\scripts\aur-s07-runtime-execution.mjs dry-run --profile $profile --outDir $outDir
```

zwraca PASS dla exact module/Area/entry i deklaruje `startsNwn=false` oraz
`usesGlobalInput=false`. Nie wykonano `arm`, `observe`, `validate`, Test Module
ani żadnej akcji w pozostawionej proof-owned sesji Toolset.
