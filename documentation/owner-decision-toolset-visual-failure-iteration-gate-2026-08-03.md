# Owner decision: deterministyczny błąd Toolsetu dopuszcza iterację

Data: 2026-08-03

Właściciel zdecydował, że świeży, dokładnie związany z kandydatem i jednoznaczny błąd wizualny na poziomie Aurora Toolset jest wystarczający do rozpoczęcia minimalnej iteracji naprawczej, jeżeli badane kryterium należy do tej powierzchni Toolsetu.

Przykład rozstrzygający: model itemu jest widoczny, ale długi miecz leży poziomo w oknie `Item Properties`, podczas gdy referencyjny miecz stoi pionowo. Dla takiego wyniku zapisujemy:

- `modelVisibility=visible`;
- `proofCompleteness=verified`;
- `visualAcceptance=failed`.

NWN nie jest wymagany przed poprawką tego konkretnego defektu, ponieważ wynik runtime nie może unieważnić błędnej orientacji w `Item Properties`. NWN pozostaje osobnym, obowiązkowym kryterium runtime po przejściu właściwej akceptacji Toolsetu.

Decyzja nie dopuszcza iteracji na podstawie nieczytelnego kadru, braku proofu, błędu placementu, HAK/build/save, timeoutu ani problemu automatyzacji. Nadal wymagane są:

1. dokładne hashe i tożsamość kandydata;
2. świeży, judgeable dowód związany z dokładnym obiektem;
3. diagnoza przyczyny;
4. minimalna deklarowana delta następnej iteracji.

Dla TLC Guard Longsword V3 warunki te spełnia packet `tlc-guard-longsword-v3-owner-toolset-orientation-result-2026-08-03.json`: dokładne V3 jest widoczne, proof Toolsetu jest kompletny, a akceptacja orientacji `Item Properties` nie przeszła.
