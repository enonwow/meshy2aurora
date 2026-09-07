# Audyt higieny Git — 7 września 2026

Zakres: klasyfikacja plików przed commitem i push, reguły `.gitignore`,
usunięcie lokalnych payloadów z indeksu oraz bramki jakości zastanego kodu.
To nie jest ponowny pełny audyt funkcjonalny ani potwierdzenie gotowości
Creature/Item/Placeable do wydania. Poprzednie ustalenia funkcjonalne pozostają
w [audycie z 5 września](audyt-projektu-2026-09-05.md).

Repozytorium: `C:\Projects\meshy2aurora`, gałąź `build-week-submission`,
początkowy HEAD `7289f2b0c8d385b2058912fb4fa6c0eb7e1990ba`.
Przed zmianami workspace guard potwierdził kanoniczne repozytorium i rejestr
worktree. Fetch potwierdził zgodność początkowego HEAD z gałęzią zdalną.

## Ustalenia i wykonane porządki

- Początkowo: 114 zmienionych plików śledzonych i 1962 nieśledzone pliki.
  Aż 1618 nieśledzonych plików w `artifacts/` zajmowało 7 844 743 959 bajtów.
  Były wśród nich wyniki diagnostyczne, obrazy, JSON-y podglądu i baza analizy
  binarnej o rozmiarze 391 036 928 bajtów.
- Dodano ignorowanie całego `/artifacts/`, lokalnych załączników
  `/.codex-remote-attachments/`, `/.worktrees/`, `/.tmp/`, literalnego
  `/%SystemDrive%/`, cache Pythona, środowisk wirtualnych oraz
  `NWNScriptCompilerTempScript.ncs`. Reguła worktree była wcześniej wyłącznie
  lokalna w `.git/info/exclude`; teraz jest częścią repozytorium.
- Rozszerzono lokalne payloady `sample-3d` o BIN i obrazy PNG/JPG/JPEG/WebP.
  Manifesty, provenance JSON i przepisy materiałów pozostają śledzone.
- Usunięto wyłącznie z indeksu 28 wygenerowanych artefaktów
  (112 610 816 bajtów) i pięć obrazów wejściowych z manifestami
  (1 261 932 bajty). Wszystkie 33 pliki pozostały na dysku; SHA-256 sprawdzono
  przed i po operacji. Hashe pięciu obrazów zgadzają się z ich manifestami.
  [Pełny wykaz i hashe](evidence/git-hygiene-untracking-2026-09-07.json).
- Nowe reguły ukryły 1631 nieśledzonych plików lokalnych. Pozostałe 331
  stanowiły kod, testy, narzędzia, dokumentacja i metadane próbek przeznaczone
  do commitów. Raport audytu i wykaz hashy dodano później.
- Kod, syntetyczne fixture, lockfile, `proof-profiles` i dokumentacja dowodowa
  pozostają wersjonowane. Nie dodano globalnego ignorowania obrazów ani JSON.
- Nie przenoszono źródeł Meshy, nie zmieniano zamrożonych kandydatów ani
  zawartości lokalnych artefaktów. Nie wykonywano sesji Toolset/NWN.

Commit higieny: `0e49c64`. Operacja nie usuwa starszych blobów z historii Git
i nie zwalnia miejsca zajmowanego przez lokalne wyniki. Historia nie była
przepisywana; świeży checkout po tym commicie nie pobiera usuniętych payloadów
do drzewa roboczego, lecz pełny klon nadal zawiera ich historyczne wersje.

## Weryfikacja

| Kontrola | Wynik |
| --- | --- |
| Canonical workspace guard | PASS dla głównego i zarejestrowanego linked worktree; odrzuca `C:\Projects` |
| Meshy asset layout guard | PASS |
| SHA-256 33 plików usuniętych z indeksu | PASS; lokalne bajty zachowane |
| `git check-ignore` — payloady i lokalne katalogi | PASS |
| `git check-ignore` — manifesty, provenance, evidence i fixture | Nie są ignorowane, zgodnie z kontraktem |
| `git ls-files -ci --exclude-standard` | Pusto po porządkach |
| Kontrola typowych formatów kluczy/tokenów w plikach do wersjonowania | Brak trafień; ograniczona kontrola wzorców, nie pełny audyt bezpieczeństwa |
| `npm run typecheck` | PASS |
| `npm test -- --reporter=dot` | 66 plików, 364/364 PASS |
| Narzędzia: registration, triangle target i trzy zestawy testów bridge | 38/38 PASS; wyłącznie testy offline/lokalne atrapy |
| `cargo fmt --all --check` | Początkowo FAIL; po `cargo fmt --all` PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | FAIL: 58 błędów dla lib i 44 dla lib test, częściowo wspólnych |
| `cargo test --workspace --no-fail-fast` | Przebieg nieukończony; potwierdzone dwa błędy core lib, pozostałe wyniki częściowe |
| `npm run build` | FAIL, exit 1: Rust/WASM release skompilowany, `wasm-opt` zakończył się kodem `0xc0000409`; etap Vite nie został wykonany |
| `git diff --check` i `git diff --cached --check` | PASS po normalizacji whitespace dokumentacji |

Clippy wskazuje m.in. nieużywane funkcje/stałe w implementacji skinningu,
format dokumentacji oraz linty pętli i operacji numerycznych. Są to ustalenia
w zastanym kodzie; zakres porządkowania Git nie obejmuje przebudowy algorytmów
ani wyciszania tych lintów. Formatowanie Rust było zmianą mechaniczną.

Dwa odtworzone błędy testów dotyczą oczekiwań provenance w
`reference_supermodel_surface_anatomy.rs`: testy
`largest_surface_component_is_the_only_authoritative_joint_fit_body` i
`disconnected_authoritative_contact_surface_contributes_ground_landmarks`.
Są opisane również w audycie z 5 września. Pełny przebieg testów został
przerwany przed końcem; nie ma końcowego kodu wyjścia i nie przypisuje się
niewykonanym zestawom wyniku PASS. Nie uruchamiano ponownie kompletnego
zestawu w celu odroczenia zleconego push.

Awaria `wasm-opt` nastąpiła po ukończeniu kompilacji release (2 min 26 s).
Nie ustalono przyczyny awarii optymalizatora i nie wyłączono optymalizacji
w konfiguracji projektu. To osobny problem narzędziowy do odtworzenia;
sam udany etap Rust nie oznacza udanego builda Studio.

W 31 nowych dokumentach znormalizowano whitespace zgłaszany przez
`git diff --cached --check`: jawne podziały linii Markdown zachowano przez
backslash, a nadmiarowe puste linie na końcu usunięto. Nie zmieniano
opisywanych wyników ani tożsamości kandydatów.

Zmiany zapisano w czterech grupach: higiena Git, Core/WASM, Studio/narzędzia
oraz dokumentacja/manifesty/reguły workspace. Publikacja jest checkpointem
z jawnymi czerwonymi bramkami, nie deklaracją gotowości do wydania.

Logi tej weryfikacji pozostają lokalnie w `.codex-tmp/git-audit-*.log`.
Wyniki pozytywne dotyczą wymienionych kontroli; czerwone bramki nie pozwalają
opisywać tego checkpointu jako wydania z kompletem zielonych testów.
