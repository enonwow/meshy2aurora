# TLC Meshy P20K Placeables V1 — amendment kolizji natywnego MOD-a

Data odczytu: `2026-07-31`

Status: `BLOCKED_NATIVE_COLLISION / IMMUTABLE_LINEAGE_PRESERVED / NOT_READY_FOR_OWNER_PROOF`

## Wynik kontroli

Zamrożony MOD kandydata nie może zostać zainstalowany pod wymaganą nazwą,
ponieważ ścieżka docelowa już istnieje i zawiera inne bajty. Zgodnie z polityką
fail-closed nie wykonano overwrite, delete, rename, replace ani automatycznej
alokacji nowego resrefu lub iteracji.

| Rola | Ścieżka | Bajty | SHA-256 |
|---|---|---:|---|
| kanoniczny MOD | `C:\Projects\meshy2aurora\proof-output\tlc-meshy-p20k-placeables-v1-20260725\generated\m2a_tlcm3_mod.mod` | 19652 | `60b943b3090487aecb48034667ebc6e41a9383a78577c4860da0b20b895dcdbb` |
| istniejący natywny MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_tlcm3_mod.mod` | 31186 | `aa6c1fde2684a166ccb8b526d03d28fcffe33683b1172729788f925bce74f1c2` |
| kanoniczny HAK | `C:\Projects\meshy2aurora\proof-output\tlc-meshy-p20k-placeables-v1-20260725\generated\m2a_tlcm3_hak.hak` | 45818454 | `0ce9f1097941a928861ee8ee06e44fe174420054c009fd05a16546e5282c55cb` |
| istniejący natywny HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_tlcm3_hak.hak` | 45818454 | `0ce9f1097941a928861ee8ee06e44fe174420054c009fd05a16546e5282c55cb` |

HAK ma tożsamość bajtową, ale nie kompensuje niezgodności MOD-a. Z wymaganych
22 porównań natywnych artefaktów audytowanych linii 21 ma zgodny hash; jedyną
kolizją jest wskazany wyżej `m2a_tlcm3_mod.mod`.

## Skutek

- Historyczny pakiet proof pozostaje niezmieniony i nadal opisuje stan w chwili
  zamrożenia.
- Nie uruchomiono Aurora Toolset ani NWN.
- Nie wykonano instalacji, ponieważ docelowy MOD nie był nieobecny ani
  byte-identical.
- Kandydat nie może otrzymać aktualnego `ready_for_owner_proof`.
- Kolizja instalacyjna nie jest wizualną porażką modelu i nie dopuszcza nowej
  iteracji.

## Warunek wznowienia

Wznowienie jest możliwe dopiero wtedy, gdy właściciel niezależnie doprowadzi
dokładną ścieżkę docelową MOD-a do stanu nieobecnego albo wyda osobną, bezpośrednią
instrukcję zmieniającą granicę mutacji. Następnie wolno ponowić instalację tylko
tego samego kanonicznego MOD-a o SHA-256
`60b943b3090487aecb48034667ebc6e41a9383a78577c4860da0b20b895dcdbb`, z kontrolą
hashu przed i po kopii. Inna zawartość docelowa nadal musi zatrzymać lane.
