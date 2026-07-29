# Animation Studio — mockupy

Zestaw pokazuje rozwój `Create & edit` jako sub-mode kroku
`Animation Mapping`. Nie powstaje osobny krok workflow ani osobny shell.

## Bazowe ekrany

1. [`01-animation-studio-create-edit.png`](01-animation-studio-create-edit.png)
   — historyczny pierwszy kierunek edytora.
2. [`02-animation-studio-within-mapping.png`](02-animation-studio-within-mapping.png)
   — zaakceptowana hierarchia edytora wewnątrz `Animation Mapping`.
3. [`03-custom-animation-picker.png`](03-custom-animation-picker.png)
   — wybór zapisanego klipu Custom dla slotu Base 42.

## Kopiowanie animacji z innego modelu

4. [`04-copy-animation-from-model-compatible.png`](04-copy-animation-from-model-compatible.png)
   — lokalny GLB został sprawdzony; output rig jest zgodny, klip `cwalk`
   jest wybrany, a `Copy to Custom` jest dostępne.
5. [`05-copy-animation-rig-mismatch.png`](05-copy-animation-rig-mismatch.png)
   — donor ma inny rig; pierwsza konkretna różnica jest widoczna, a
   `Copy to Custom` pozostaje zablokowane.

## Decyzje UX

- wejście znajduje się w `+ New animation`;
- modal nie zasłania kontekstu biblioteki, viewportu i timeline;
- plik dawcy jest inspektowany lokalnie i nie staje się zależnością projektu;
- użytkownik widzi liczbę animacji, kości, tracków i skrót SHA-256;
- zgodność rigu jest decyzją przed wyborem akcji;
- inny rig nie oferuje pozornego sukcesu — automatyczny retarget jest osobną
  przyszłą funkcją;
- skopiowany klip trafia do biblioteki `Custom`.
