# `m2aborzmod6.mod`

Toolset module name: `Meshy2Aurora Borzoi c_wolf Demo V6`\
Exact Area name: `Meshy2Aurora Borzoi Test Area V6`

Date: 2026-08-21

Status: `ready_for_owner_proof`. Dokładne MOD i HAK są już zainstalowane w
natywnych katalogach NWN i zweryfikowane bajt w bajt. Agent nie uruchamiał
Aurora Toolset ani NWN; runtime pozostaje `RUNTIME_UNPROVEN` do testu
właściciela.

## Co naprawia V6

V6 izoluje zmianę, która w V5 doprowadziła do fatalnego crashu klienta. Model
zachowuje bez zmian:

- kanoniczny GLB 300 000 trójkątów;
- rig V5 `owned-borzoi-c-wolf-deformation-cage-rig-v5`;
- barycentryczny transfer wag i cztery iteracje wygładzania;
- bezpośrednie nośniki `c_wolf`, zero correction nodes i zero lokalnych
  animacji;
- dokładnie ten sam motion contract oraz wynik motion oracle `PASS`.

Zmienia się wyłącznie emisja materiału. V6 używa
`CLASSIC_DIFFUSE_TGA_SAFE_V1`: zwykłego diffuse TGA w bazowym MDL, bez
material extension, tangentów, MTR, TXI, normal i specular. Generator wybiera
te elementy przez deklaratywną receptę `rig + model + quality`, a nie przez
warunek `version == N`.

## Potwierdzona izolacja V5 → V6

| Kontrakt | V5 | V6 | Wynik |
|---|---|---|---|
| Source SHA-256 | `f96be839…75dda` | `f96be839…75dda` | identyczny |
| Rig content SHA-256 | `78bcc68d…247d2` | `78bcc68d…247d2` | identyczny |
| `rig-profile.json` SHA-256 | `9c4d1ed9…42cca` | `9c4d1ed9…42cca` | byte-identical |
| Motion contract SHA-256 | `800a8075…264c` | `800a8075…264c` | identyczny |
| Motion quality | `PASS` | `PASS` | zachowany |
| Material extension | tak | nie | usunięty |
| Tangent streams | obecne | `0` | usunięte |
| HAK | 9 zasobów | 3 zasoby | uproszczony |

HAK V6 został niezależnie odczytany przez parser ERF i zawiera dokładnie:

1. `appearance` type `2017`;
2. `m2aborzcre6` type `2002`;
3. `m2aborztex6` type `3`.

## Exact owner handoff

- Test module file: `m2aborzmod6.mod`
- Ordered HAK: `m2aborzhak6`
- Creature blueprint: `m2aborzutc6`
- Appearance row: `848`
- Model resref: `m2aborzcre6`
- Diffuse texture resref: `m2aborztex6`
- Creature placement: `(10.0, 14.5, 0.0)`
- Creature orientation: `(0.0, -1.0)`, w stronę punktu startowego

## Immutable artifact identity

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2aborzmod6.mod` | 15 205 | `5330fa8da1a8570ae0342a66c719715ec52fbd9f1bdde6cd0a8bc5fc9a134e79` |
| `m2aborzhak6.hak` | 76 222 468 | `90f06145306de60b2d4decd7167606a7f1d0ee0e2f331bdb713ea94fd91cb604` |
| `m2aborzcre6.mdl` | 25 502 368 | `eba9c0ab085ad3f2243be0b1c419d76cc71d31a31c4056c62b16454b69a18203` |
| `m2aborztex6.tga` | 50 331 692 | `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1` |

Canonical packet:
`proof-output/borzoi-c-wolf-demo-v6-20260821`.

Native installation:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aborzmod6.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aborzhak6.hak`

## Kryteria oceny właściciela

1. Moduł ładuje się bez fatalnego crashu klienta.
2. Pies jest widoczny, stoi na podłożu i jest zwrócony do gracza.
3. Tułów, szyja, głowa i sierść nie tworzą rozerwanych płatów ani dużych
   kolców.
4. Przednie łapy są rozstawione, a wszystkie cztery łapy uczestniczą w ruchu.
5. Idle, chód, bieg oraz atak zachowują psią sylwetkę.
6. Diffuse jest widoczny; brak normal/specular w V6 jest zamierzonym testem
   izolującym crash V5.

Do czasu wyniku właściciela osie pozostają:
`modelVisibility=not_tested`, `proofCompleteness=missing`,
`qualityVerdict=not_tested`.
