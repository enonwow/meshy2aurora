# H2 r44 procedural full-native — ready for owner proof

Test-module file: `m2a_h2r44.mod`

Toolset module name: `Meshy2Aurora procedural humanoid proof`

Area: `Meshy2Aurora M0 binary vertical-slice area`

Area resref: `m2a_h2a44`

Status: `ready_for_owner_proof`

## 1. Cel tej lineage

To jest pierwszy świeży handoff wykorzystujący aktualną produkcyjną ścieżkę
proceduralnego creature. Nie jest to stary `m2a_van02`, H1 ani rigid/type-0
r43.

Właściciel odrzucił kierowanie kolejnego testu do starego,
runtime-niepotwierdzonego modułu diagnostycznego i polecił iść w działające
rozwiązanie. R44 zamyka potwierdzone braki implementacyjne r43:

- nie używa corrupt-draw H1 jako wzorca poprawności;
- ma osobny controllerless Aurora root;
- ma jeden direct-root SkinMesh i ważony szkielet;
- usuwa dopuszczalny stały uniform scale ze wszystkich stanów;
- ma dokładnie 42 odrębne stany type 5;
- ma 23 callbacki gameplay;
- używa `ActiveMonsterBaseline` identycznie w GIT i module-local UTC;
- ma świeżą, caller-owned tożsamość we wszystkich warstwach zamiast
  kolizyjnych `m2a_m6p01`/`m2a_codex_aproof`.

Wynik właściciela r43 pozostaje związany z exact zmodyfikowanym MOD-em:

```text
m2a_h2r43.mod
39090 bytes
a444722545f81da256d6733b10c71c830609939eb31df496a3db2e9e7e3f7f77
NWN modelVisibility=not_visible
```

## 2. Exact immutable lineage

Canonical output:

`C:\Projects\meshy2aurora\proof-output\h2-r44-procedural-full-native-20260726`

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `generated/source.glb` | 8,234,708 | `f8cf0af21c8143a62b64c490a81dd2855ad3c3f9865922e3854f84b714dec3a3` |
| `generated/m2a_h2p44.mdl` | 612,692 | `41138bf8b5ce94e04607ef119752f45d6800a107a6ee6f282888317908e6eb80` |
| `generated/m2a_h2t44.tga` | 12,582,956 | `03169b1493ed4b2f5269ed6fa611aef787e219cddd08661c9135a7f82191c167` |
| `generated/appearance.2da` | 7,655,521 | `632bdc6d1d9a38f27493737d9f244e34a92773e28480100b81d670bec1c403c9` |
| `generated/m2a_h2r44.hak` | 20,851,425 | `c2a089f0fd6ee50ce186a6495f992b0fbb0a8f348dcca7406ddd670849f21ac6` |
| `generated/m2a_h2r44.mod` | 15,157 | `721f4653cc7c546fe6c2ed26df36e7643909a267c4c3f0bed061077ee578a315` |
| `h2-r44-procedural-full-native-lineage-contract-v1.json` | 23,279 | `c85b7d29dab5046fdec89b78e98b682dfcd80ba3bcb9649ab3f8541513c1fea0` |

Exact resource bindings:

```text
MOD/HAK resref: m2a_h2r44
Area resref:    m2a_h2a44
UTC resref:     m2a_h2utc44
model resref:   m2a_h2p44
texture resref: m2a_h2t44
Appearance row: 15219
MODELTYPE:      S
RACE:           m2a_h2p44
```

## 3. Exact scene

Moduł ma dokładnie jedno creature:

```text
display name: Meshy procedural humanoid
tag/id:       m2a_procedural_creature
UTC:          m2a_h2utc44
profile:      ActiveMonsterBaseline
position:     [10, 14.5, 0]
orientation:  [0, -1]
```

Player entry to `[10,10,0]`, kierunek `[0,1]`. Creature stoi 4.5 m
bezpośrednio przed graczem i jest zwrócone w jego stronę.

## 4. Offline readback

Exact MDL:

- binary MDL, `geometryType=2`, `classification=4`;
- model/root name `m2a_h2p44`;
- 26 base nodes;
- controllerless model root;
- jeden SkinMesh;
- 24 aktywne joints;
- kompletne wagi, forward/reverse map i inverse binds;
- 42 animacje, wszystkie type 5;
- 23/23 wymagane callbacki;
- bind reconstruction error `<= 1e-4`;
- mesh jest uziemiony na `Z=0`, ma około 1.7 m wysokości;
- wszystkie wymagane próbki wykazują nie-rigid deformation.

Exact MOD readback:

- `moduleResref=m2a_h2r44`;
- `areaResref=m2a_h2a44`;
- ordered HAK list dokładnie `[m2a_h2r44]`;
- dokładnie jeden GIT creature i jeden odpowiadający UTC;
- GIT i UTC klasyfikują się niezależnie jako `ActiveMonsterBaseline`;
- `PerceptionRange=11`;
- nie ma H1 ani Hook Horrora jako mylących controls.

## 5. Testy wykonane

Przeszły:

```text
cargo test -p m2a-core --example materialize_h2_r44_procedural_candidate
cargo test -p m2a-core --test model_pipeline \
  procedural_humanoid_profile_authors_a_distinct_owned_42_state_set_from_h2_idle
M2A_REFERENCE_CEP_HAK=<cep3_core1.hak> \
  cargo test -p m2a-core --test erf_reference_integration
cargo test -p m2a-core
cargo test --workspace
git diff --check
```

Stary `nwnmdlcomp`:

- z powodzeniem zdekompilował exact `m2a_h2p44.mdl` do ASCII;
- odmówił rekompilacji komunikatem
  `Too many bones in skin, max allowed is 17`.

Ten komunikat jest ograniczeniem historycznego kompilatora, nie limitem
formatu EE. Canonical native corpus przechodzi dla `c_kocrachn` z trzema
skinami `extended64` i `map count=38,38,38`; `17` i `64` są szerokościami
wariantów inline headera, nie limitem map/bind arrays. Wynik rekompilacji
oznaczono więc jako `REFERENCE_TOOL_LIMITATION`, bez mutowania r44.

Strict Clippy na aktualnym Rust 1.96 nie jest zielony dla całego zastanego
workspace: zgłasza istniejące wcześniej linty w kilku niezwiązanych plikach.
Nowy kod kompiluje się, focused tests i pełne testy workspace przechodzą.

## 6. Native installation

Instalację wykonano create-new/no-clobber. Cele były nieobecne przed kopią.
Po kopii źródło i cel są byte-identical:

| Native destination | Bytes | SHA-256 |
|---|---:|---|
| `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_h2r44.mod` | 15,157 | `721f4653cc7c546fe6c2ed26df36e7643909a267c4c3f0bed061077ee578a315` |
| `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_h2r44.hak` | 20,851,425 | `c2a089f0fd6ee50ce186a6495f992b0fbb0a8f348dcca7406ddd670849f21ac6` |

## 7. Granica wyniku

Agent nie uruchomił, nie przejął i nie kontrolował Toolsetu ani NWN.

Obie osie pozostają:

```text
Toolset modelVisibility=not_tested, proofCompleteness=missing
NWN     modelVisibility=not_tested, proofCompleteness=missing
```

Status `ready_for_owner_proof` oznacza, że exact MOD/HAK są gotowe i
zainstalowane. Widoczność i zachowanie zamyka dopiero wynik właściciela z tej
samej immutable lineage.

