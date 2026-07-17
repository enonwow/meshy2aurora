# Audyt mockupow Meshy Lab — 2026-07-16

## Werdykt

Mockupy `Meshy Lab` maja prawidlowa architekture UX: jest to opcjonalny,
prowadzony workflow proof assetu, a nie drugi glowny produkt ani surowy
explorer API. Nie sa jeszcze finalnym kontraktem wizualnym. Ekrany Configure
i Review wymagaja korekt, a ekran Ready wymaga ponownego mockupu viewportu.

| ID | Stan | Status | Decyzja |
|---|---|---|---|
| MLAB-01 | Configure / H1 | `ACCEPTED_WITH_FIXES` | Zachowac trzykolumnowy workbench i profil H1; odchudzic powtorzone informacje. |
| MLAB-02 | Review generation and credits | `ACCEPTED_WITH_FIXES` | Zachowac jawne potwierdzenie kosztu; dodac saldo i usunac kolor ostrzezenia z neutralnej ceny. |
| MLAB-03 | H1 asset ready | `REWORK_REQUIRED` | Zachowac przeplyw sukcesu; przebudowac preview i zredukowac duplikacje statusow. |

## Baza porownawcza Studio

Obowiazujacym stylem jest pakiet
`documentation/mockups/studio-v1-2026-07-14`:

- spokojny, ciemny technical-art workbench, nie SaaS dashboard;
- wyrazny, jeden glowny cel na ekran;
- cienkie obramowania i panele strukturalne, bez plywajacych kart;
- cyan oznacza aktualna akcje lub aktywny krok, zielony tylko potwierdzony
  wynik, amber tylko warning, a nie neutralna metryke;
- checkmark zastępuje numer ukonczonego kroku;
- Debug Drawer jest domyslnie zwiniety; szczegoly techniczne nie konkurują z
  akcja workflow;
- normalny produkt zachowuje workflow `Source -> Inspect -> Build -> Review
  Output -> Download`.

Meshy Lab moze miec wlasny, kontekstowy workbench, ale nie moze udawac
szostego etapu glownego workflow. Wejscie i powrot musza byc zawsze jawne:
`Open Meshy Lab from Source` oraz `Import verified GLB to Source`.

## Korekty wspolne dla wszystkich ekranow

1. **Wspolny shell.** Zachowac marke, gęstosc top baru, proporcje paneli,
   ikony i rytm pionowy Studio. Tekst `Back to Source` ma byc jednoznacznym
   wyjsciem z opcjonalnego workspace, a nie alternatywna nawigacja produktu.
2. **Jedna informacja w jednym miejscu.** Pochodzenie assetu, stan bridge i
   status runu maja miec jedno zrodlo prawdy. Nie powtarzac tych samych danych
   w lewym railu, srodku i prawym inspectorze.
3. **Progresywne ujawnianie.** API key, task ID, hash, JSON i proof packet nie
   sa trescia podstawowego ekranu. Sa dostepne tylko pod `View technical
   details`; klucz API nigdy nie trafia do UI.
4. **Status i kolor.** `Connected`, `Generated`, `Intake passed` i `Ready to
   import` sa zielone. Cena, rozmiar pliku oraz format sa neutralnymi wartosciami
   tekstowymi. Amber pozostaje zarezerwowany dla ostrzezen, ryzyka albo
   wymagajacej potwierdzenia akcji.
5. **Nazewnictwo.** Uzywac konsekwentnie `Source`, `Studio`, `GLB`, `Local
   Meshy Bridge` i `Import to Source`. Nie mieszac nazwy produktu z surowymi
   terminami API w glownej warstwie UI.
6. **Dostepnosc.** Zwiekszyc kontrast drobnych opisow, nie opierac znaczenia
   wylacznie na kolorze i utrzymac duze, jednoznaczne cele klikniecia.

## MLAB-01 — Configure / H1

### Co zachowac

- lewy rail z trzema profilami proofu;
- centralny, trzyetapowy tok `Configure -> Generate -> Import to Studio`;
- domyslnie ukryte Advanced options;
- prawy inspector z kosztem i zredukowanym podsumowaniem runu.

### Co poprawic

- Sekcje `Local Meshy Bridge` i `Runs` w railu zredukowac do jednego wiersza
  statusu oraz linku `Connection details`; opis granicy credentiali pokazac
  raz, pod tym linkiem.
- Nie powtarzac kompletnej konfiguracji w prawym inspectorze. Inspector ma
  pokazywac tylko profil, wynik, koszt i stan.
- Tekst przy H1 ma opisywac cel użytkownika (`Humanoid with animation`), a nie
  implementacyjny detal typu `no auto-rig` dla pozostalych profilow.
- Akcja glowna pozostaje jedna: `Review generation and credits`.

## MLAB-02 — Review generation and credits

### Co zachowac

- read-only podsumowanie Asset i Pipeline;
- wyrazny komunikat, ze task jeszcze nie istnieje;
- powrot do konfiguracji oraz jawna akcja `Generate H1 asset — 38 credits`.

### Co poprawic

- Dodac `Available balance` oraz `Balance after generation`; przy
  niewystarczajacym saldzie blokowac akcje i wyjasnic przyczyne.
- Cena nie powinna wygladac jak warning. Uzyc neutralnego koloru lub cyan;
  amber tylko dla informacji o ryzyku albo niepewnej wycenie.
- Zredukowac powtorzenie `Texture`, `Rigging` i `Animation` miedzy Pipeline a
  prawym panelem: w inspectorze wystarcza skrot stanu runu.
- Przed potwierdzeniem nie pokazywac technicznych krokow jako aktywnego runu;
  nazwac je `After confirmation` i pozostawic mutowane.

## MLAB-03 — H1 asset ready

### Problem wymagajacy przebudowy

Obecny preview ma mocny bialy obrys, kapitaliki i styl niezaleznego debug
narzedzia (`SOURCE GLB / LOCAL PREVIEW`). Nie odpowiada standardowemu viewportowi
Studio z ekranow Inspect i Review. Prosty manekin nie komunikuje tez wiarygodnie
roznicy miedzy wygenerowanym assetem a technicznym placeholderem.

### Wymagany nowy uklad

- Preview ma uzyc tej samej ramy, tla gridowego, osi i skali typografii co
  viewport Studio. Bez grubego bialego obramowania i bez displayowej,
  skondensowanej typografii.
- Nad viewportem: `Source GLB` oraz dyskretny badge `Intake passed`; pod nim
  tylko standardowe narzedzia preview (`Frame`, `Grid`, `Wireframe`).
- Obok preview jedna karta `Intake verification` z piecioma checkmarkami.
  `Asset summary` polaczyc z prawym inspectorem zamiast tworzyc drugi panel z
  tymi samymi danymi.
- Glowna akcja ma brzmiec `Import verified GLB to Source`.
- Drugorzedne akcje `Download original GLB` i `Start another proof asset`
  zostaja przy tej akcji, ale nie mogą wizualnie rywalizowac z importem.
- Prawy inspector pokazuje tylko `Ready to import`, finalny koszt, wynik GLB,
  Intake i target `Studio Source`; szczegoly są w zwiniętym disclosure.

## Kolejnosc prac nad mockupami

1. Zastosowac korekty wspolne do MLAB-01 i MLAB-02.
2. Wygenerowac od nowa MLAB-03 zgodnie z wymaganym ukladem preview.
3. Dopiero po zatwierdzeniu trzech stanow przygotowac stan przejsciowy
   `Generating` jako wariant MLAB-02, a nie jako osobny, rozbudowany ekran.
4. Po akceptacji zapisac eksporty mockupow w
   `documentation/mockups/meshy-lab-v1-2026-07-16` wraz z README i macierza
   statusow przed implementacja.
