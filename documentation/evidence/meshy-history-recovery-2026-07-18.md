# Meshy Lab: historia i odzyskiwanie GLB (2026-07-18)

Status: `IMPLEMENTED / FAKE-TRANSPORT VERIFIED / LIVE M0 RECOVERY PENDING`.

## Objaw i zakres

Płatne wygenerowanie M0 zakończyło się stanem `READY`, lecz Studio i Local
Bridge trzymały bieżący run, sesję oraz bajty GLB wyłącznie w pamięci procesu.
Po wygaśnięciu sesji nie istniała ścieżka UI ani kontrakt Bridge'a do ponownego
odczytania historycznego zadania Meshy. Sam podpisany URL GLB jest czasowy i
nie stanowi trwałego identyfikatora odzyskania.

## Fakty i odrzucone założenia

- Fakt z kodu przed zmianą: `runs` był mapą in-memory; `GET /v1/runs/{id}`
  nie odtwarzał zadania po restarcie Bridge'a.
- Fakt z kodu przed zmianą: `MeshyLab` trzymał `sessionToken` i `run` w stanie
  Reacta; odświeżenie lub nowa karta usuwały tę informację.
- Fakt z oficjalnego API Meshy: `GET /openapi/v2/text-to-3d` listuje strony
  zadań, a `GET /openapi/v2/text-to-3d/{id}` zwraca zadanie `preview` lub
  `refine` wraz z możliwym `model_urls.glb`.
- Fakt lokalny: pobrany provenance M0 zachował oba ID etapów oraz pełny hash i
  rozmiar GLB. Nie kopiujemy tych identyfikatorów ani URL-i do dokumentacji.
- Odrzucone założenie: że aktywny Bridge lub import w Source wystarcza jako
  trwałe archiwum artefaktu.

## Wybrane podejście

Dodano slice `MLAB-I6` bez tworzenia nowych zadań i bez dodatkowych kosztów:

1. `GET /v1/history` pobiera jedną stronicę z `GET
   /openapi/v2/text-to-3d`, maksymalnie 50 wpisów. Bridge usuwa signed URL-e
   przed przekazaniem danych do Studio.
2. Widok **Meshy history** pokazuje strony dostępnych prac. Akcja recovery jest
   dostępna wyłącznie dla ukończonego `text-to-3d-refine` z GLB.
3. Bridge ponownie odczytuje wybrane zadanie po ID, pobiera GLB we własnym
   procesie, sprawdza nagłówek `glTF`, rozmiar i SHA-256, a dopiero potem
   przekazuje bajty do Studio.
4. Odzyskany artefakt otrzymuje provenance
   `RECOVERED-text-to-3d/v1` oraz ID etapu `REFINE`; Studio używa nadal tego
   samego wejścia `File` jak dla lokalnego wyboru GLB.

## Wynik weryfikacji

- `node --test bridge.test.mjs`: 7/7 PASS, w tym lista historii bez wycieku
  signed URL-i oraz recovery zweryfikowanego GLB z fake transportu Meshy.
- `npm test` w `apps/studio-web`: 155/155 PASS.
- `npm run typecheck` w `apps/studio-web`: PASS.

## Pozostałe ryzyko i następny krok

Nie wykonano jeszcze live recovery M0: zmieniony Bridge musi zostać uruchomiony
z istniejącym kluczem właściciela i nowym jednorazowym parowaniem. Operacja
listowania/ponownego pobrania nie tworzy taska i nie powinna zużyć kredytów,
ale przed importem M0 należy porównać odzyskany SHA-256 oraz rozmiar z jego
lokalnym provenance. Dopiero potem M0 może wejść do osobnego proof HAK/MOD,
Aurora Toolsetu i NWN:EE.
