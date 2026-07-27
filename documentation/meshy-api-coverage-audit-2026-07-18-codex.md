# Audyt pokrycia Meshy API i UX (2026-07-18)

**Status:** audyt wykonany; poprawki kontraktu Local Bridge zweryfikowane fake transportem; nie wykonano żadnego płatnego zadania Meshy.

## Zakres i dowody

Stan Studio porównano z publicznym workspace Meshy oraz oficjalną dokumentacją API w dniu 2026-07-18. Nie logowano się na konto i nie przesyłano materiałów do Meshy. To wystarcza do sprawdzenia formularzy publicznego workspace i kontraktów API, ale nie jest dowodem zachowania planu właściciela ani realnego taska.

- Workspace: [Image / Multi-image / Text to 3D](https://www.meshy.ai/pl/workspace) pokazuje trzy źródła modelu, bibliotekę zasobów i osobne obszary `Model`, `Drukuj` oraz `Animować`.
- Publiczny przegląd funkcji: [Meshy Features](https://www.meshy.ai/pl/features) pokazuje też 3D Agent, AI Texture, generator obrazów i animację jako osobne narzędzia jednego workspace.
- Kontrakty generowania: [Text to 3D](https://docs.meshy.ai/en/api/text-to-3d), [Image to 3D](https://docs.meshy.ai/en/api/image-to-3d), [Multi-Image to 3D](https://docs.meshy.ai/en/api/multi-image-to-3d).
- Kontrakty działań na istniejącym modelu: [Remesh](https://docs.meshy.ai/en/api/remesh), [Retexture](https://docs.meshy.ai/en/api/retexture), [Rigging](https://docs.meshy.ai/en/api/rigging), [Animation](https://docs.meshy.ai/en/api/animation) i [Animation Library](https://docs.meshy.ai/en/api/animation-library).
- Eksport i druk: [formaty eksportu](https://docs.meshy.ai/en/webapp/guides/platform/export-formats) oraz [Multi-Color Print](https://docs.meshy.ai/en/api/multi-color-print).
- Zmiany API są istotne: [changelog](https://docs.meshy.ai/en/api/changelog) z 13 lipca dodał dla Image to 3D Smart Topology (`meshy-t1`/`meshy-t2`).

## Wniosek UX

Referencje Image/Multi-image/Text pokazują układ trójkolumnowy: konfiguracja źródła po lewej, pusty lub wybrany canvas pośrodku, biblioteka modeli po prawej. Aktualny Meshy Lab zachowuje tę zasadę (źródło/canvas/historia), zamiast wracać do historycznego pełnoekranowego kroku z pionowym stepperem zaznaczonego na czwartej referencji.

Meshy rozdziela **tworzenie** (Text/Image/Multi-image do 3D, tekstowa faza preview/refine) od **operacji na modelu lub zadaniu, które już istnieje** (retexture, remesh, rigging, animation, format/druk). To rozdzielenie musi pozostać widoczne w Studio: akcja na modelu nie może udawać kolejnego pola formularza tworzenia ani ominąć osobnego review kosztu.

## Macierz pokrycia

| Endpoint / funkcja Meshy | Stan Studio / Bridge | Brak lub ryzyko | Bezpieczny kierunek |
| --- | --- | --- | --- |
| Połączenie, pairing, saldo | Zaimplementowane: loopback, origin allowlist, TTL, pairing i `GET /v1/balance`. | Nie ma logowania ani klucza w UI — celowo. | Zachować lokalny proces i pamięć procesu dla klucza. |
| Text to 3D v2: preview + refine | Zaimplementowane, z kontrolami remesh/topologii/decimation, pose, moderacji, PBR/HD texture, lighting, miniatur i auto-size. | Nie jest ogólnym klientem każdego pola lub stylu Meshy. | Dodawać pole wyłącznie z walidacją Bridge, testem requestu i review kosztu. |
| Image to 3D | Zaimplementowane: jeden PNG/JPEG Data URI; tekstura, PBR, lighting, image enhancement i generacja GLB. | Nie ma `input_task_id` z generatora obrazów Meshy. | Podłączyć tylko po osobnym, nietajnym kontrakcie dla tasków 2D; lokalny upload pozostaje podstawą. |
| Image Smart Topology | **Dodano w tym audycie**: tylko Image to 3D, `meshy-t1` lub `meshy-t2`; T2 ma limit 100–15 000 faces. | Nie należy wysyłać `smart-topology` do Text/Multi-image. | Utrzymać ograniczenie źródłowe oraz testy walidacji. |
| Multi-Image to 3D | Zaimplementowane: 1–4 PNG/JPEG Data URI, zgodnie z API. | UX publicznego workspace reklamuje większy zakres ujęć, lecz API opisuje 1–4; nie rozszerzać bez potwierdzonego API. | Utrzymać 1–4 i jasny opis. |
| Texturing w fazie refine / Image / Multi-image | Zaimplementowane dla obsługiwanych źródeł (`should_texture`, PBR, HD, prompt/obraz guidance, lighting w endpointach obrazowych). | Text/Image URL guidance jest wejściem wysyłanym po potwierdzeniu; nie może przyjmować signed URL-i z historii. | W przyszłości przyjmować tylko lokalny Data URI albo jawny publiczny URL, walidowany i pokazany w review. |
| Retexture (`/v1/retexture`) | Niezaimplementowane. | To płatna operacja na istniejącym tasku albo modelu (GLB/glTF/OBJ/FBX/STL); obecny Bridge nie ma bezpiecznej ścieżki uploadu modelu ani historii tych typów. | Osobny ekran „Edit existing model”; wejście tylko przez znormalizowany task ID lub lokalny, ograniczony Data URI; osobne preview, maksimum kosztu, nonce i GLB intake. |
| Remesh (`/v1/remesh`) | Częściowo: te same parametry są używane podczas tworzenia, ale brak operacji na ukończonym modelu. | Nie wolno przedstawiać checkboxa tworzenia jako post-processingu. | Dodać osobną akcję na zaufanym tasku: topology, target/decimation, auto-size; obsłużyć nowy task, recovery i import GLB. |
| Convert, Resize, UV Unwrap | Niezaimplementowane. | Nowe endpointy zastępują część historycznych pól Remesh; mogą stworzyć artefakt niedający się zaimportować do Studio. | Najpierw zdefiniować docelowe użycie Aurora i kontrakt artefaktu, potem pojedyncze płatne slice'y. |
| Rigging | Zaimplementowane wyłącznie w H1 po preflight (standardowy humanoid, wyraźne kończyny, bez broni). | Nie jest to ogólny rig dla lokalnych modeli; Meshy dokumentuje ograniczenia humanoidów i limit 300 000 faces. | Zachować H1 jako wąski pipeline; w przyszłości osobne wejście dla istniejącego taska i opcjonalny Remesh przed rig. |
| Animation | Zaimplementowane po rigu; domyślny action `0` to oficjalne `Idle`, można podać inne ID. | Brak odczytywalnej biblioteki akcji i opcji `post_process`; arbitralne ID może zostać odrzucone przez API. | Najpierw dodać read-only katalog akcji z oficjalnego źródła, potem walidowane opcje FPS/operacji w osobnym review. |
| Historia / recovery | Zaimplementowane dla Text to 3D: stronicowanie, status, retry od refine i pobranie GLB; signed URL-e nie przechodzą do UI. | Nie obejmuje Image/Multi-image/Remesh/Retexture/Rig/Animation ani nie jest trwałym archiwum Meshy. | Uogólnić jako znormalizowany, per-endpoint katalog metadanych (bez URL-i), dopiero po mapowaniu każdego typu artefaktu. |
| Status, cancel, SSE | Polling i cancel bieżącego runu są zaimplementowane; historia jest odczytem bezpłatnym. | SSE nie jest podłączone; brak listowania i delete dla większości tasków. | SSE można dodać jako optymalizację odczytu. Delete pozostawić poza UI, aż będzie jawny, odwracalny UX potwierdzenia. |
| Format wyjściowy | Bridge pobiera i weryfikuje wyłącznie GLB, a Studio importuje tylko GLB. **W tym audycie usunięto wybór OBJ/FBX/STL/USDZ/3MF z formularza**, aby nie naliczać niewidocznych plików. | Meshy oferuje GLB, FBX, OBJ, STL, USDZ i 3MF, ale bieżący kontrakt nie ma pobrania/provenance/importu dla pozostałych. | Najpierw dodać typowany artifact manifest, bezpieczny download i wyraźny odbiorca pliku; dopiero potem wystawić format. |
| Multi-Color Print | Niezaimplementowane. | To płatna konwersja do 3MF z paletą 1–16 kolorów i depth 3–6; Studio nie ma konsumenta 3MF. | Oddzielny moduł druku, tylko po decyzji produktu i kontrakcie downloadu 3MF; nie mieszać z Aurora intake. |
| Analyze / Repair Printability | Niezaimplementowane. | Brak potwierdzonego use-case w pipeline Aurora i brak obiektu wyniku w Studio. | Traktować jako przyszły moduł druku, po Multi-Color Print albo jako read-only raport. |
| Text to Image / Image to Image / 3D Agent / Creative Lab | Niezaimplementowane. | Rozszerzają zakres o generowanie 2D, kontynuację tasków i nowe dane użytkownika. | Najpierw wybrać osobny produktowy cel. Nie wystawiać arbitralnego proxy ani automatycznego chainingu do 3D. |

## Poprawki wykonane w tym audycie

1. Bridge akceptuje teraz wyłącznie `127.0.0.1` albo `::1`; `0.0.0.0` jest odrzucone.
2. Image/Multi-image przyjmują wyłącznie ograniczone Data URI `image/png` i `image/jpeg`, a nie dowolne `data:image/*`.
3. Parametr `image_enhancement` jest dostępny wyłącznie przy źródle obrazowym i przekazywany do Image/Multi-image to 3D razem z `remove_lighting`.
4. Smart Topology jest dostępne wyłącznie dla Image to 3D i wymaga `meshy-t1` lub `meshy-t2`; validator nie akceptuje go dla Text ani Multi-image.
5. H1 ponownie ma deterministyczny Idle (`action_id: 0`) zgodny z oficjalną biblioteką animacji.
6. Output jest jawnie GLB-only, ponieważ tylko ten plik ma działający kontrakt pobrania, SHA-256/provenance i importu Studio.

## Weryfikacja

- `npm run typecheck` w `apps/studio-web` — PASS.
- `npx vitest run src/features/meshy/bridge.test.ts src/features/meshy/MeshyLab.test.tsx` — PASS, 9 testów.
- `node --test bridge.test.mjs` w `tools/meshy-local-bridge` — PASS, w tym test loopback, MIME i Smart Topology.
- `git diff --check` — PASS po końcowym uruchomieniu.

## Pozostałe granice

- Brak realnego E2E; nie uruchamiano płatnych zadań ani nie używano klucza.
- Studio nie ujawnia key, pairing code ani signed model URLs.
- Żadna funkcja API nie może dodawać zadania bez salda, preview, maksimum kosztu i jednorazowego nonce potwierdzenia.
- Publiczny zakres Meshy jest szerszy niż cel Aurora. Pokrycie oznacza tu poprawne, bezpieczne wsparcie albo jawny plan — nie pozorną kontrolkę w UI.

## Aktualizacja kontraktu i UX — 2026-07-19

**Status: aktywne wdrożenie; nie jest to jeszcze końcowa deklaracja 90%.** Poniższa tabela zastępuje historyczne stwierdzenie, że ReTexture nie ma kontraktu. Sprawdzono ją w kodzie Bridge, testach lokalnego Bridge i w nieobciążającym proofie lokalnego fixture. Nie utworzono płatnego zadania Meshy.

| Endpoint / funkcja | Aktualny stan Studio / Bridge | Brak / granica | Bezpieczny plan |
| --- | --- | --- | --- |
| Text to 3D / Image to 3D / Multi-image to 3D | Preview, saldo przed utworzeniem, jawne potwierdzenie i odzysk zweryfikowanego GLB. UI pokazuje tylko pola rzeczywiście walidowane przez Bridge. | Nie jest ogólnym proxy wszystkich eksperymentalnych pól web Workspace. | Rozszerzać pojedynczo wraz z walidatorem, testem i ekranem kosztu. |
| Text to Image / Image to Image | Własny preview + potwierdzenie; ukończony prywatny task 2D może być przekazany do Image to 3D wyłącznie po identity taska. | Brak signed URL w UI; brak „udawanej” galerii obrazów. | Zachować task identity jako jedyne ogniwo chainingu. |
| ReTexture (`/openapi/v1/retexture`) | **Zaimplementowane.** Bridge przyjmuje tylko zaufany `REFINE` albo `PREVIEW` task ID, buduje preview z maksimum kredytów, wymaga jednorazowego nonce przy utworzeniu, pollingu/cancelu i zweryfikowanego GLB. UI ma oddzielne menu akcji modelu i ekran review. | Nie przyjmuje arbitralnego pliku ani signed model URL; nie jest przypięte do przycisku lokalnego Material Matching. | Zachować oddzielenie: Material Matching = lokalny renderer, ReTexture = płatne zadanie z jawnej akcji. |
| Material Matching Workspace | **Zaimplementowane jako lokalna funkcja viewportu.** Metaliczność/szorstkość, intensywność i kontrast zmieniają tylko aktualny materiał Three.js; kliknięcie czyści aktywny podgląd mapy, więc efekt nie jest maskowany. | Nie zmienia modelu na serwerze i nie nalicza kredytów — tak samo nie udaje ReTexture. | Utrzymać jako bezpieczne sterowanie podglądem, osobno od API ReTexture. |
| Remesh jako akcja na istniejącym modelu | Brak osobnego endpointu/UI. `should_remesh`, topology, decimation i polycount działają tylko w tworzeniu, zgodnie z kontraktem generowania. | Nie wolno opisywać checkboxa generowania jako post-processingu. | Dodać osobny paid slice dopiero z task identity, preview/nonce i recovery GLB. |
| Rigging / Animation | Wąski pipeline H1 po preflight; Bridge uruchamia rigging, następnie animation dla zweryfikowanego humanoida. | Brak swobodnego edytora istniejącego modelu i katalogu akcji w UI. | Najpierw read-only katalog akcji i kontrakt na task identity, potem osobny review. |
| History / recovery | Stronicowane metadane i odzysk zweryfikowanego GLB bez signed URL; biblioteka Studio pobiera lokalne miniatury przez Bridge. | Zakres historii nie jest pełnym archiwum wszystkich rodzajów tasków Meshy. | Rozszerzać per typ taska wraz z manifestem artefaktu. |
| Output formats | GLB-only: jedyny format z pełnym pobraniem, weryfikacją, provenance i importem Studio. | OBJ/FBX/STL/USDZ/3MF nie są eksponowane. | Najpierw typowany manifest i odbiorca pliku, potem pojedynczy format. |
| Multi-color print / analiza druku | Niezaimplementowane. | Brak konsumenta 3MF i use-case Aurora. | Oddzielny moduł druku po decyzji produktu; nie dodawać widoku „Drukuj” dla samego podobieństwa UI. |

### Korekta semantyki viewportu

Aktualny publiczny Workspace używa `Dopasowanie materiału` jako lokalnego dialogu materiału (`280×236` w punkcie `316,108` przy 1280×720), a nie jako żądania ReTexture. W Studio pierwotne podpięcie tego przycisku do płatnego ReTexture było rozjazdem semantycznym. Zostało zastąpione lokalnym panelem o tym samym układzie; płatny ReTexture pozostaje dopiero w menu akcji zweryfikowanego modelu. Browser proof wykazał również i naprawił sytuację, w której aktywny kafelek Base Color maskował zmianę Lit/Material Matching.

Pozostały dowód do końcowego gate: screenshot Meshy → screenshot Studio dla każdego scoped stanu viewportu oraz końcowa tabela procentowa. Do czasu tego gate nie należy używać wcześniejszych wpisów `PASS` jako deklaracji 1:1.
