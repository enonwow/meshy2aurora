# Suplement implementacji: opcjonalne wsparcie Meshy API przez Local Bridge

**Status:** IMPLEMENTED (I0-I7 runner) — realne, platne E2E I7 pozostaje recznym gate'em wlasciciela.
**Data:** 2026-07-16
**Zakres:** opcjonalny Meshy Lab do pozyskiwania kontrolowanych assetow proof dla Studio.

**Stan implementacji 2026-07-16:** kontrakt UI-Bridge, feature flag,
fake Bridge do testow, Meshy Lab, loopback Bridge, H1/N1/S1, weryfikacja
hash/GLB, provenance/import oraz budzetowany runner I7 sa obecne w worktree.
Nie wykonano platnego taska na prawdziwym koncie Meshy; taki test wymaga
osobnego, jawnego uruchomienia zgodnie z I7.

## 1. Decyzja architektoniczna i granice

Ten suplement rozszerza, ale nie zmienia decyzji D12-D14 z `architektura-web-wasm-codex.md`: Studio pozostaje statyczna aplikacja browser-first/local-first, a pipeline konwersji dziala lokalnie w React + WASM.

Meshy API nie moze byc wolane bezposrednio z Reacta. Oficjalna dokumentacja Meshy opisuje blokade CORS dla wywolan z przegladarki oraz zaleca serwerowy proxy. Tutaj takim proxy jest **opcjonalny, lokalny i uruchamiany przez wlasciciela Meshy Local Bridge**. Nie jest to backend produktu, usluga publiczna ani nowy etap glownego workflow Studio.

```text
Studio (przegladarka) ── loopback, sesja parowana ──> Meshy Local Bridge
       │                                                   │
       │  tylko lokalny GLB po jawnym imporcie             │ Bearer API key
       └───────────────────────────────────────────────────┴──> Meshy API
       │
       └── m2a-core / WASM: bez sieci, bez klucza Meshy, bez zmian kontraktu
```

### Niezmienialne reguly

- Bez Bridge Studio dziala tak jak dzis: uzytkownik wybiera lokalne GLB i `appearance.2da` w kroku Source.
- Klucz `MESHY_API_KEY` istnieje tylko w pamieci procesu Bridge, pobrany z jego srodowiska. Nie trafia do Reacta, localStorage, URL, manifestu proof, telemetryki ani logow.
- Bridge nasluchuje tylko na `127.0.0.1` i `::1`; nigdy na `0.0.0.0` ani w sieci LAN.
- Bridge udostepnia waski kontrakt dla zdefiniowanych profili. Nie jest ogolnym proxy HTTP do Meshy i nie przyjmuje arbitralnych URL-i ani endpointow.
- Kazde potencjalnie platne utworzenie taska wymaga osobnego ekranu review i jawnego potwierdzenia uzytkownika.
- Asset po imporcie jest zwyklym lokalnym GLB. Studio nie otrzymuje dostepu do konta Meshy, nie wysyla danych do Meshy i nie zmienia wymagan M6/M7/S1.

## 2. Cel produktu i poza zakresem

### Cel

Meshy Lab ma umozliwic wlascicielowi przygotowanie odtwarzalnego assetu proof, zweryfikowanie go, a nastepnie jawne wstawienie pobranego GLB do pola **Meshy GLB model** w Source. Ten sam asset przechodzi pozniej niezmieniony workflow: Inspect -> Build -> Review Output -> Download.

### Poza zakresem MVP

- logowanie kontem Meshy, przechowywanie kluczy albo platnosci w Studio;
- publiczny backend, webhooki wymagajace publicznego HTTPS i wielouzytkownikowosc;
- pelny klient wszystkich endpointow Meshy;
- automatyczne wydawanie kredytow, seryjne generowanie lub ukryte retrie platnych taskow;
- generowanie plikow Aurora/NWN, modyfikacja Toolsetu albo instalacji gry przez Bridge;
- obrazowe/multi-image workflow w pierwszej wersji.

## 3. Kontrakt UI i przebieg uzytkownika

Meshy Lab jest kontekstowym miejscem pomocniczym, nie szostym krokiem naglowka Studio. Wejscie to kompaktowe CTA **Open Meshy Lab** na Source, a powrot to **Back to Source**. Projekt wizualny musi stosowac aktywny kontrakt `documentation/mockups/studio-v1-2026-07-14` oraz zalecenia z `AUDYT_MOCKOW_MESHY_LAB_2026-07-16.md`.

| Ekran | Dzialanie | Warunek przejscia |
| --- | --- | --- |
| Connect | Wlasciciel uruchamia Bridge i paruje lokalna sesje. | Health, wersja protokolu i origin sa poprawne. |
| Configure | Wybiera H1, N1 lub S1 i uzupelnia ograniczony formularz profilu. | Lokalna walidacja profilu. |
| Review generation | Widzi parametry, estymacje, saldo oraz maksymalny koszt. | Jawne **Generate**. |
| Generating | Widzi etap, status taska, anulowanie i bezpieczne odswiezanie. | Artefakt pobrany i zweryfikowany. |
| Ready to import | Widzi lokalny preview, intake i provenance. | Jawne **Import verified GLB to Source**. |
| Source | GLB jest podstawiony jako lokalny input. | Standardowy Inspect jest jedynym kolejnym krokiem. |

Jedno zrodlo prawdy dla profilu, kosztu, taska i wyniku nalezy trzymac w `MeshyRun` po stronie UI. Lewy panel pokazuje tylko polaczenie i wybor profilu, a prawy tylko aktualny status runu. Nie wolno powielac tych samych informacji w trzech panelach.

### Stan maszyny

`OFFLINE -> PAIRING -> CONNECTED -> CONFIGURE -> AWAITING_CONFIRMATION -> QUEUED -> GENERATING -> DOWNLOADING -> VERIFYING -> READY -> IMPORTED`

Sciezki boczne: `FAILED`, `CANCELED`, `EXPIRED`. Przejscie z `AWAITING_CONFIRMATION` do `QUEUED` moze nastapic tylko raz dla danego `confirmationNonce`; ponowienie po niepewnym bledzie najpierw odczytuje status istniejacego taska, a nie tworzy kolejny.

## 4. Profile proof i obslugiwane operacje

Profil jest wersjonowanym ograniczeniem, a nie dowolnym kreatorem requestow. Kazdy ma identyfikator, wersje, walidator danych, dozwolone etapy, budzet, oczekiwane cechy wynikowe i kryteria intake.

| Profil | Cel | Dozwolony pipeline MVP | Oczekiwany wynik |
| --- | --- | --- | --- |
| `H1-humanoid-animated/v1` | Teksturowany standardowy humanoid do proof rig/animacji. | Text-to-3D preview -> refine -> rigging -> animation Idle -> GLB download. | GLB z teksturami, szkieletem i idle clipem. |
| `N1-quadruped/v1` | Teksturowany niehumanoid bez automatycznego rigu. | Text-to-3D preview -> refine -> GLB download. | GLB z teksturami, bez deklaracji rigu/animacji. |
| `S1-static-prop/v1` | Statyczny obiekt z tekstura. | Text-to-3D preview -> refine -> GLB download. | GLB z teksturami, bez szkieletu i animacji. |

H1 musi przed riggingiem przejsc preflight: teksturowany standardowy biped, brak deklarowanej broni, czytelne konczyny, zgodna pozowana sylwetka i limit geometrii wymagany przez Meshy. Nieudany preflight nie moze przejsc do platnego etapu rig/animacja. N1 i S1 nie wolno wysylac do riggingu ani animacji.

Pierwsza wersja obsluguje tylko endpointy potrzebne do tych profili: balance, Text-to-3D, odczyt statusu taska/strumienia, rigging, animation, anulowanie, pobranie artefaktu oraz weryfikacje wyniku. Remesh, Retexture, Convert, Image-to-3D i Multi-image-to-3D pozostaja przyszlymi propozycjami.

## 5. Kontrakt Local Bridge

Bridge powinien byc oddzielnym, malym procesem developersko-proofowym, np. `tools/meshy-local-bridge`. Nie jest czescia `m2a-core`, nie jest Workerem WASM i nie ma prawa przetwarzac Aurora/NWN.

### Polaczenie i parowanie

1. Bridge startuje lokalnie z `MESHY_API_KEY` w srodowisku procesu.
2. Generuje jednorazowy kod parowania oraz ograniczona czasowo sesje.
3. UI wysyla kod tylko po akcji uzytkownika.
4. Bridge sprawdza `Origin`, sesje, wersje protokolu i rate limit; zwraca token zwiazany z tym originem.
5. Kazda kolejna operacja wymaga tokenu. Restart Bridge uniewaznia sesje.

Nie wolno akceptowac wildcard originow, przekazywac tokenu w query stringu ani zapisywac go jako trwalej preferencji.

### Waskie endpointy aplikacyjne

| Endpoint | Cel | Efekt finansowy |
| --- | --- | --- |
| `GET /v1/health` | Dostepnosc, wersja protokolu, stan parowania. | Brak |
| `GET /v1/balance` | Dostepne kredyty bez ujawniania konta/klucza. | Brak |
| `GET /v1/profiles` | Lista lokalnie wspieranych profili i wersji. | Brak |
| `POST /v1/runs/preview` | Waliduje request, wylicza plan i limit kosztu. | Brak |
| `POST /v1/runs` | Tworzy run po nonce potwierdzenia. | Mozliwy koszt |
| `GET /v1/runs/{runId}` | Znormalizowany status oraz postep. | Brak |
| `GET /v1/runs/{runId}/events` | Opcjonalny SSE; polling jako fallback. | Brak |
| `POST /v1/runs/{runId}/cancel` | Anuluje jeszcze anulowalny etap. | Brak gwarancji zwrotu |
| `GET /v1/runs/{runId}/artifact` | Strumieniuje zweryfikowany GLB po akcji uzytkownika. | Brak |
| `GET /v1/runs/{runId}/provenance` | Techniczne szczegoly i hashe bez sekretow. | Brak |

Kazda odpowiedz wykorzystuje wersjonowany typ `MeshyBridgeEnvelopeV1`. Bledy musza miec stabilny kod: `BRIDGE_UNAVAILABLE`, `PAIRING_REQUIRED`, `INSUFFICIENT_CREDITS`, `TASK_REJECTED`, `TASK_FAILED`, `ASSET_EXPIRED`, `ARTIFACT_INVALID`, `CANCELED`. Surowe odpowiedzi Meshy i naglowki autoryzacji moga istniec tylko w zredagowanym logu diagnostycznym Bridge.

## 6. Import do Studio i provenance

Po kliknieciu importu Bridge strumieniuje bajty artefaktu do przegladarki. Studio tworzy obiekt `File`/`Blob` w pamieci, sprawdza MIME, rozmiar i hash, a nastepnie wywoluje **ten sam** kod wyboru inputu co lokalny file picker. Nie ma ukrytego folderu, dostepu do dysku ani alternatywnej sciezki Build.

Do UI mozna dolaczyc nietajne `MeshyProvenanceV1`:

- `profileId` i `profileVersion`;
- czasy etapow, znormalizowane ID taskow i wersja Bridge;
- SHA-256 pobranego GLB, rozmiar oraz wynik intake;
- parametry profilu zatwierdzone przez uzytkownika, bez klucza i bez podpisanych URL-i.

Provenance jest dodatkiem do proof packet. M6/M7/S1 nadal wymagaja wlasciwych dowodow Studio, Toolsetu i gry; wygenerowany GLB nie zamyka automatycznie zadnego milestone'u.

## 7. Bezpieczenstwo, prywatnosc i kredyty

| Ryzyko | Kontrola wymagana do Definition of Done |
| --- | --- |
| Ujawnienie klucza API | Tylko environment procesu Bridge; redakcja logow; testy zakazu wycieku do odpowiedzi/UI. |
| Dostep z LAN | Bind tylko loopback, bez konfiguracji publicznego hosta/port-forward. |
| Obca strona steruje Bridge | Allowlista originu, parowanie, token sesji, TTL, rate limit i restart uniewazniajacy sesje. |
| Niespodziewany koszt | Preview, saldo, maksymalny koszt, jawne potwierdzenie, idempotency nonce i brak auto-retry create. |
| Wyciek signed URL lub assetu | Nie wyswietlac ani logowac URL-i; pobierac tylko przez dozwolony run; artefakt domyslnie w pamieci. |
| Uszkodzony plik | Limit rozmiaru, typ GLB, hash, intake Studio przed importem. |
| Utrata sledzalnosci | Zredagowany provenance, hash, wersja profilu/Bridge i screenshot proof. |

Webhooki sa wylaczone w MVP, bo wymagaja publicznego endpointu. Bridge korzysta z ograniczonego polling lub SSE przez swoje polaczenie wychodzace do Meshy.

## 8. Szczegolowy podzial implementacji

Kazdy slice zaczyna sie od testow kontraktu i mock transportu; prawdziwe Meshy nie jest warunkiem codziennego CI.

### MLAB-I0 — decyzje, feature flag i kontrakty

- Dodac decyzje do dokumentacji architektury: optional local companion, `mvp_backend: false` pozostaje prawdziwe.
- Zdefiniowac `MeshyProfileV1`, `MeshyRunV1`, `MeshyBridgeEnvelopeV1`, bledy i serializacje; UI na fake Bridge nie ma jeszcze polaczenia produkcyjnego.
- Dodac feature flag domyslnie wylaczona; brak flagi nie zmienia Source.
- **DoD:** walidatory, testy zgodnosci wersji i brak importu bibliotek Meshy w `m2a-core`/Workerze.

### MLAB-I1 — Local Bridge: start i granica bezpieczenstwa

- Utworzyc minimalny proces loopback z `/health`, parowaniem, origin check, sesja TTL i zredagowanym loggerem.
- Odczyt `MESHY_API_KEY` tylko przy starcie; fail-fast gdy nie istnieje; endpoint health nigdy nie potwierdza wartosci klucza.
- Dodac testy: bind loopback, odrzucenie obcego originu, wygasniecie sesji, redakcja `Authorization` i signed URL.
- **DoD:** Bridge nie ma endpointu generycznego proxy i nie uruchamia requestu platnego.

### MLAB-I2 — katalog profili oraz bezplatny preview planu

- Zakodowac H1/N1/S1 jako dane wersjonowane i walidowac tylko ich pola.
- Zaimplementowac balance i `runs/preview`; pokazac saldo, koszt maksymalny, etapy i warunki profilu.
- UI: Connect, Configure oraz Review generation zgodne z poprawionymi mockami.
- **DoD:** nawigacja, offline i walidacja sa pokryte testami UI; klikniecie review nie wywoluje `POST /runs`.

### MLAB-I3 — orkiestracja Text-to-3D/refine

- Dla H1/N1/S1 stworzyc task preview, obserwowac stan, a po sukcesie wykonac refine tylko z jednego zatwierdzonego runu.
- Zapisac mapowanie wewnetrznego runu do taskow Meshy i idempotency nonce.
- Obsluzyc timeout, anulowanie, wygasly artefakt i restart Bridge bez duplikacji platnego requestu.
- **DoD:** test fake Meshy pokrywa sukces, bledy i wznowienie od statusu taska.

### MLAB-I4 — H1 rig/animation oraz ograniczenia N1/S1

- Dodac preflight H1 przed riggingiem, potem rigging i wybrany Idle action.
- Zablokowac rig/animation dla N1 i S1 na poziomie kontraktu Bridge oraz UI.
- Pokazac postep jako jeden etapowy timeline; koszt koncowy to dane Bridge, nie wartosc wpisana w makiecie.
- **DoD:** H1 zawiera kontraktowe oczekiwanie rig + Idle; N1/S1 nie potrafia zbudowac requestu do tych endpointow.

### MLAB-I5 — pobranie, intake i import do Source

- Strumieniowac artefakt tylko po jawnym kliknieciu, obliczyc SHA-256, uruchomic weryfikacje typu/rozmiaru i Source intake.
- Zaimplementowac ekran Ready zgodnie z audytem: viewport ma uzywac tej samej ramy, osi i hierarchii co Studio; bez bialej ramki i duplikatow statusu.
- Wywolac istniejaca sciezke wyboru lokalnego GLB; zachowac Source jako jedyne miejsce decydujace o gotowosci do Inspect.
- **DoD:** test browserowy potwierdza, ze importowany Blob trafia do tej samej sciezki co lokalny plik, bez klucza i bez sieci Workera.

### MLAB-I6 — proof packet, diagnostyka i odzyskiwanie

- Wprowadzic niejawny manifest provenance oraz zwijany panel techniczny.
- Dodac UI bledow, retry dla bezplatnego odczytu statusu, cancel i akcje recovery; zadne retry nie moze samodzielnie tworzyc nowego taska.
- Dodac eksport metadanych proof bez assetu i bez danych uwierzytelniajacych.
- **DoD:** dowod zawiera hash, profil, task IDs, wersje, etap intake i screenshot; nie zawiera kluczy ani signed URL-i.

### MLAB-I7 — testy realne i akceptacja wlasciciela

- Real E2E jest osobnym, recznie zatwierdzanym skryptem, nie standardowym CI.
- Wymaga jednoczesnie: `MESHY_REAL_E2E=1`, klucza w srodowisku oraz jawnego limitu budzetu `MESHY_MAX_CREDITS`; brak ktoregokolwiek warunku konczy test przed utworzeniem taska.
- Wlasciciel zatwierdza trzy oryginalne proof assets (H1, N1, S1), ekran review i faktycznie pobrane GLB. Artefakty binarne nie trafiaja do repozytorium; do dokumentacji trafiaja tylko uzgodnione screenshoty i hashe.
- **DoD:** wynik real E2E oraz proof packet sa zarchiwizowane zgodnie z polityka projektu; wyniki M6/M7/S1 sa aktualizowane osobno, tylko gdy ich wlasne kryteria sa spelnione.

## 9. Strategia testow i dowodow

| Warstwa | Testy wymagane |
| --- | --- |
| Kontrakt | Schematy request/response, wersjonowanie, nieznane pola i normalizacja bledow. |
| Bridge | Origin, sesja, TTL, loopback, redakcja, idempotencja, cancel i fake Meshy transport. |
| UI | Offline, parowanie, review, koszt/saldo, generowanie, blad, ready, import i accessibility klawiatury. |
| Studio | Importowany Blob przechodzi ten sam Inspect/Build path co lokalny GLB; Worker pozostaje offline. |
| Real E2E | Jedna jawnie zatwierdzona probka na profil, limit budzetu, hash artefaktu, provenance i screenshot. |

Do testow jednostkowych i integracyjnych uzywamy fake Meshy transportu. Nie wolno umieszczac prawdziwego klucza, ID konta, signed URL-i ani pobranych GLB w fixture'ach, logach CI czy repozytorium.

## 10. Kolejnosc realizacji i bramki

1. Wlasciciel akceptuje ta decyzje architektoniczna, profile i budzet real E2E.
2. Najpierw zaakceptowac poprawione mocki MLAB-01, MLAB-02 i przebudowany MLAB-03; zapisac je jako wersjonowane artefakty w `documentation/mockups`.
3. Zrealizowac I0-I2 z fake Bridge i testami, bez kredytow.
4. Zrealizowac I3-I6 w malych, testowalnych commitach.
5. Dopiero potem uruchomic I7 z kluczem wlasciciela i jawnym limitem.
6. M6/M7/S1 zamykac w `orchestrator-state.yaml` niezaleznie od tego suplementu, tylko na podstawie ich kompletnych kryteriow dowodowych.

**Bramka przed pierwszym platnym taskiem:** zaakceptowany mock review, dzialajacy loopback/parowanie, widoczne saldo i koszt maksymalny, test idempotencji oraz ustalony przez wlasciciela limit kredytow.

## 11. Zrodla referencyjne

- [Meshy API — authentication](https://docs.meshy.ai/en/api/authentication)
- [Meshy API — errors and CORS guidance](https://docs.meshy.ai/en/api/errors)
- [Meshy API — Text to 3D](https://docs.meshy.ai/en/api/text-to-3d)
- [Meshy API — rigging](https://docs.meshy.ai/en/api/rigging)
- [Meshy API — animation](https://docs.meshy.ai/en/api/animation)
- [Meshy API — pricing](https://docs.meshy.ai/en/api/pricing)
- [Audyt mockow Meshy Lab](AUDYT_MOCKOW_MESHY_LAB_2026-07-16.md)
- [Kontrakt mockow Studio v1](mockups/studio-v1-2026-07-14/README.md)
