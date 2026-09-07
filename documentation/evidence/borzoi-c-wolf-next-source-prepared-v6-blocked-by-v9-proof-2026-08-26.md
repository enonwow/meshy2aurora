# Następne źródło borzoja `c_wolf` — wejścia przygotowane, generacja Meshy wstrzymana

Data: 2026-08-26

Status: `INPUTS_PREPARED / ZERO_MESHY_CREDITS_SPENT / V9_OWNER_PROOF_PENDING`

Przygotowano cztery spójne obrazy modelarskie pod rzeczywistą pozycję bind/rest
potomka `c_dog`, który pokrywa 23/23 aktywne kości `c_wolf`. Odrzucono tylny
widok end-on, ponieważ redukował ogon do krótkiej, niejednoznacznej kępki.
Zastąpił go widok tylny 3/4 pokazujący pełną długość i dwa segmenty ogona.

Pierwszy widok przedni również odrzucono: centralna, zwisająca masa sierści
mogła zostać zinterpretowana przez Meshy jako ogon skierowany w dół. W wersji
`borzoi-cwolf-bind-front-v2-no-tail.png` ogon jest całkowicie zasłonięty przez
tułów, zgodnie z projekcją od przodu dla poziomego bindu `c_wolf`.

Czwarty obraz jest ścisłym rzutem z góry. Kończyny pozostają w pozycji stojącej
pod tułowiem zamiast rozchodzić się na boki, a pełny ogon biegnie po osi
kręgosłupa. Widok opisuje szerokość grzbietu, talii i miednicy bez wprowadzania
sprzecznej pozycji kończyn.

Wybrane wejścia i ich hashe zapisuje:
`artifacts/concept-art/c-wolf-compatible-borzoi-bind-v6/generation-manifest.json`.

Nie utworzono zadania Meshy i nie pobrano kredytów. Płatny run oraz nowy GLB
byłyby kolejną iteracją tego samego kandydata. Dokładny V9 nadal ma
`modelVisibility=not_tested` i `proofCompleteness=missing`; wymagany jest świeży
wynik właściciela dla `m2aborzmod9.mod` oraz odpowiadającego mu dokładnego HAK-a.
Po tym wyniku można zastosować model-iteration gate i wznowić przygotowany run
z limitem 30 kredytów.

## Aneks 2026-08-26 — właściciel zaklasyfikował źródło jako nową wersję

Później tego samego dnia właściciel jednoznacznie wskazał, że komplet `bind-v6`
jest nową wersją źródła, a nie powtórzeniem kandydata V9. Na tej podstawie
wykonano jeden run Meshy, zapisano nowy model w kanonicznym `sample-3d`,
przepuszczono go przez ogólny pipeline dziedziczenia supermodelu `c_wolf` i
utworzono niezależne demo V10.

Aktualny wynik i dokładne hashe opisuje
`documentation/evidence/borzoi-c-wolf-demo-v10-ready-for-owner-proof-2026-08-26.md`.
Historyczny status powyżej pozostaje zapisem stanu sprzed tej decyzji
właściciela i nie opisuje już bieżącego kandydata V10.
