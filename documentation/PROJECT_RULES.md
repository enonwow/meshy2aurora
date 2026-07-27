# Project Rules

## 0. Kanoniczny workspace — HARD STOP

Jedynym repozytorium i zapisywalnym workspace projektu jest
`C:\Projects\meshy2aurora`.

`C:\Users\enonw\Documents\meshy2aurora` jest sciezka bezwzglednie zakazana.
Nie wolno tam tworzyc, edytowac, stage'owac, kopiowac, migrowac, testowac,
budowac ani przechowywac plikow tymczasowych projektu. Nie jest to klon,
staging, scratch, mirror ani fallback. Wlasciciel nigdy nie wskazal ani nie
autoryzowal tej sciezki.

Kazdy agent i subagent przed pierwszym zapisem musi rozwiazac repo root. Jezeli
nie jest nim dokladnie `C:\Projects\meshy2aurora`, ma wykonac HARD STOP bez
tworzenia plikow i bez obchodzenia problemu przez drugi katalog. Task trzeba
wznowic z repo kanonicznym jako workspace root. Pelny kontrakt znajduje sie w
`documentation/CANONICAL_WORKSPACE.md` i root `AGENTS.md`.

Obowiazkowy preflight:

`powershell -NoProfile -ExecutionPolicy Bypass -File assert-canonical-workspace.ps1`

## 1. Dokumentacja

Cala dokumentacja projektu znajduje sie w folderze `C:\Projects\meshy2aurora\documentation` i tam ma byc dopisywana. Nie tworzymy rozproszonych notatek poza tym folderem bez rownoczesnego wpisu lub przeniesienia do `documentation`.

Foldery o podobnej nazwie poza `C:\Projects\meshy2aurora` nie sa kanoniczne
dla tego projektu i nie wolno uzywac ich nawet jako tymczasowego stagingu.

### 1.1 Rejestr istotnych problemow i wynikow

Kazdy istotny problem techniczny, blokujacy proof, zmieniajacy decyzje albo
ujawniajacy rozjazd miedzy narzedziem a runtime musi otrzymac trwaly wpis w
`documentation`. Wpis ma rozdzielac fakty od hipotez i zawierac co najmniej:

- objaw, zakres oraz warunki odtworzenia;
- zebrane dowody i odrzucone hipotezy;
- wybrane podejscie do rozwiazania wraz z uzasadnieniem;
- wynik weryfikacji: uruchomione testy/proof, ich rezultat i pozostale ryzyko
  albo jawny kolejny krok.

Nie wolno oznaczac problemu jako rozwiazanego samym udanym buildem, odczytem
Toolsetu albo niezweryfikowana hipoteza. Gdy problem dotyczy runtime NWN,
wynik musi wyraznie rozroznic proof Toolsetu od proofu w grze.

### 1.2 Kanoniczny uklad modeli Meshy

Jedynym katalogiem zrodlowym lokalnych modeli wybranych przez wlasciciela jest:

`C:\Projects\meshy2aurora\sample-3d\<asset-id>\`

Kazda probka ma sledzony `manifest.yaml` z provenance, rolami plikow,
rozmiarami i SHA-256. Binarne payloady sa lokalne i ignorowane przez Git.
`test-assets\meshy` jest wycofanym, zabronionym drugim rootem i nie moze byc
odtwarzany ani wskazywany przez kod lub testy.

`proof-output` przechowuje zamrozone lineage proof, a `artifacts` wygenerowane
wyniki. Nie sa biblioteka zrodel i nie wolno nimi po cichu zastepowac
`sample-3d`. Po kanonicznym przeniesieniu dokumentacja moze znormalizowac
sciezke zrodla tylko z datowanym amendmentem potwierdzajacym byte-identical
SHA-256. Oryginalny immutable packet proof pozostaje autorytatywny dla sciezki
obowiazujacej w chwili capture.

Przed zmiana ukladu trzeba przeczytac `MESHY_ASSET_LAYOUT.md` i uruchomic:

`powershell -NoProfile -ExecutionPolicy Bypass -File assert-meshy-asset-layout.ps1`

Gate ma pozostac zielony. Nowy konkurencyjny katalog, brak manifestu,
niezadeklarowany payload albo hash niezgodny z manifestem jest bledem struktury,
nie powodem do dodania wyjatku.

## 2. Aurora First

ZASADA NAJWAZNIEJSZA DLA IMPLEMENTACJI: AURORA FIRST.

Dekompilacja Aurory jest glownym zrodlem wiedzy. Nie ma strzelania. Jezeli dany watek, model albo agent nie zna odpowiedzi, najpierw szuka jej w dekompilacji `C:\Projects\New Folder`, potem w lokalnych zasobach gry/CEP/NWN EE jako read-only reference, potem w `C:\Projects\aurora-web` jako read-only reference, a dopiero pozniej w Internecie. Internet moze uzupelniac brakujacy kontekst, ale nie moze zastapic lokalnego dowodu z Aurory, jezeli ten dowod jest dostepny.

`C:\Projects\aurora-web` jest osobnym projektem. W `meshy2aurora` wolno czytac jego kod, dokumenty i artefakty jako material porownawczy, ale nie wolno uzywac go jako dependency, CLI/subprocess, oracle, walidatora, fixture source ani elementu runtime/testow. Implementacja `meshy2aurora` ma miec wlasny parser MDL, wlasny emiter/writer, wlasny writer 2DA i wlasny writer ERF/HAK.

Formatem docelowym modelu dla gry jest natywny binary MDL oraz polityka MDX rozstrzygnieta dla pierwszego profilu w `engine-mdl-odpowiedz-codex.md` Q2. ASCII MDL nie jest sciezka runtime/proofu; moze istniec tylko jako debug dump albo golden snapshot do czytania przez czlowieka.

Aurora First dla modelu oznacza lancuch: realny model Aurory -> own reader -> mapa potwierdzonych invariantow -> own IR/writer -> own readback -> Toolset/game. Reader nie produkuje assetu do gry; odkrywa kontrakt, ktory writer musi spelnic. Nie wymagamy identycznych bajtow ani nie kopiujemy payloadu, ale wymagamy analogicznej semantyki, layoutu profilu i zachowania akceptowanego przez engine.

Przed implementacja obszaru trzeba sprawdzic `documentation/macierz-gotowosci-wiedzy-codex.md` i jego kanoniczny kontrakt. Odlegly etap nie moze pozostac bez kierunku: dopuszczalny jest otwarty runtime proof, ale nie brak decyzji, zrodla i testu zamykajacego.

Kazde twierdzenie implementacyjne musi byc oznaczone jako jedno z:

- fakt z dekompilacji Aurory;
- fakt z retail/resource/binary/proof;
- aktualny status `aurora-web` jako reference-only;
- wniosek implementacyjny;
- hipoteza do sprawdzenia.

Hipoteza nie jest podstawa implementacji bez testu albo proofu.

## 2.1 Licencje i provenance zrodel

Zewnetrzne repozytoria, dekompilacja, retail i CEP sa materialem read-only. Fakty o publicznym formacie wolno niezaleznie zaimplementowac, ale nie kopiujemy kodu GPL, payloadow, animacji, szkieletow, tekstur ani fixture z tych zrodel do `meshy2aurora`. Kazdy zewnetrzny fragment kodu wymaga przed uzyciem jawnej decyzji licencyjnej i zapisu provenance.

Domyslny proof base to wlasne syntetyczne fixture oraz wygenerowany przez `meshy2aurora` HAK/modul/asset. Licencja samego repozytorium pozostaje decyzja wlasciciela przed publicznym wydaniem; brak tej decyzji nie daje zgody na kopiowanie materialow referencyjnych.

Jeden realny model nie wystarcza do uznania parsera albo writera za zgodny. Reguly corpusu wielomodelowego sa w `documentation/korpus-referencyjny-mdl-codex.md`: realne zasoby sa czytane in-place przez env-gated testy regresyjne, a CI i finalny proof pozostaja oparte na fixture/generated content.

Kazdy faktycznie uruchomiony model referencyjny wymaga packetu `P-REF`: hash wejscia, raport naszego readera, wyniki invariantow i - po dodaniu preview - widoczny screenshot lub motion capture naszego preview. Screenshot Toolsetu z niezmienionym assetem gry nie jest proofem naszego kodu.

## 2.2 Aurora/NWN: nie odtwarzamy workflow od zera

### Shared Aurora operator tooling exception

The `aurora-web` reference-only boundary applies to Meshy2Aurora product code,
assets, fixtures, product validators, and runtime dependencies. It does not
apply to the shared Aurora operating skills, their canonical runner, or their
verified native atoms. Those are mandatory shared operator tooling for Aurora
and NWN work. Agents use them directly and must not create, qualify, or
substitute a Meshy2Aurora-local Toolset runner or UI adapter.

### Trwala autoryzacja kontroli live Aurora/NWN — decyzja wlasciciela 2026-07-21

Dla celu proof Meshy2Aurora wlasciciel udziela agentom koordynujacym stalej
zgody na uruchamianie, inspekcje, sterowanie, testowanie, przelaczanie oraz
zamykanie sesji Aurora Toolset i NWN, ktore agent uruchomil albo jawnie przejal
do biezacego runu. Rutynowe akcje w zakresie zadania nie wymagaja kolejnych
pytan: obsluga oczekiwanych modali, nawigacja do exact modulu/Area, wybor
obiektu, capture Toolset/NWN, koordynacja Test Module i czyste zamkniecie
procesu moga przebiegac autonomicznie przez shared canonical operator.

Ta sama stala autoryzacja obejmuje bezpieczny staging i instalacje dokladnego,
wygenerowanego przez projekt MOD-a albo HAK-a z kanonicznego workspace do
natywnego katalogu uzytkownika NWN `modules` albo `hak` dla zatwierdzonego
lineage proof. Przed kopiowaniem trzeba rozwiazac i zahashowac dokladne zrodlo,
rozwiazac dokladny cel oraz wymagac, aby cel nie istnial. Po instalacji trzeba
ponownie zahashowac cel i wymagac byte-identical zgodnosci z zatwierdzonym
zrodlem przed otwarciem albo testem. Jezeli cel juz istnieje, wolno go tylko
odczytac: identyczny hash oznacza reuse bez kopiowania, a rozny hash oznacza
fail-closed — nigdy overwrite, delete, rename ani replace. Nalezy wybrac nowy
zatwierdzony resref, filename i destination path zgodny z model-iteration gate,
a nastepnie powtorzyc kontrole absent-target oraz hash before/after. Jest to
waski wyjatek instalacyjny dla MOD/HAK, nie zgoda na uzywanie natywnego
katalogu NWN jako workspace lub cache projektu.

Zgoda obejmuje zmiane otwartego modulu na inny exact zatwierdzony modul bez
kolejnego pytania. Proof-owned albo jawnie przejeta responsywna sesja Toolsetu
moze uzyc `File > Open`, gdy biezacy exact frame modulu nie ma dirty `*`,
readback potwierdza brak niezapisanych zmian, nie ma Save promptu ani innego
modala, a target jest exact zainstalowanym i hash-verified zatwierdzonym MOD-em.
Po operacji trzeba ponownie rozwiazac okna i wymagac zgodnosci frame/module
readback z targetem przed kontynuacja. Nieoczekiwany Save prompt, dirty state,
target mismatch albo brak post-switch readback oznacza fail-closed i nie wolno
go potwierdzac ani obchodzic. Clean close, potwierdzone osadzenie procesu i
swieza sesja Toolsetu pozostaja dozwolona alternatywa, gdy same-session switch
jest nieodpowiedni; nie sa obowiazkiem. Nie wolno utrzymywac dwoch
konkurencyjnych procesow Toolsetu.

Ta stala autoryzacja usuwa pytania o kazdy zwykly krok, ale nie znosi zakazu
globalnego inputu, destrukcyjnej odpowiedzi na nierozpoznany prompt, zmiany
INI/MRU lub konfiguracji uzytkownika, cichego Save/repack zamrozonego lineage
r27, nowej iteracji modelu ani bramek tozsamosci sesji/kandydata/proofu. Takie
stany pozostaja fail-closed wedlug shared Aurora standards.

### Proof koncowy wykonuje wlasciciel — decyzja wlasciciela 2026-07-24

Koncowy proof wizualny w Aurora Toolset i NWN wykonuje wlasciciel projektu.
Ta decyzja zaweza i nadpisuje powyzsza stala autoryzacje live-control w zakresie
proofu kandydata.

- Agent implementuje i diagnozuje model, uruchamia testy offline, zamraza jeden
  dokladny lineage kandydata oraz przygotowuje krotki handoff z hashami i
  oczekiwanym modulem, Area, obiektem, HAK-iem, Appearance row i placementem.
  Kazdy handoff zaczyna sie od dokladnej nazwy testowego pliku `.mod`, nastepnie
  podaje nazwe modulu widoczna w Toolsecie i dokladna nazwe Area; zadne z tych
  pol nie moze pozostac domyslne.
- Agent nie uruchamia, nie przejmuje, nie przelacza, nie kontroluje, nie
  przechwytuje, nie mierzy, nie sprzata i nie konczy sesji proofowej Aurora
  Toolset ani NWN. Nie uruchamia skilla proofowego, jego koordynatora, wrapperow
  ani zastepczego workflow.
- Agent nie instaluje ani nie stage'uje artefaktow proofowych w natywnych
  katalogach NWN, chyba ze wlasciciel osobno zleci dokladnie taka operacje.
- Wlasciciel wybiera sposob wykonania proofu. Po przekazaniu przez niego wyniku
  albo dowodu agent moze go zapisac, zastosowac bramke iteracji, zdiagnozowac
  blad i kontynuowac implementacje.
- Wyjatek wymaga nowego, bezposredniego polecenia wlasciciela, ktore jawnie
  autoryzuje proof live wykonywany przez agenta dla jednego dokladnego
  kandydata. Ogolne polecenie kontynuowania implementacji, dokonczenia modelu
  albo przygotowania go do testu nie przywraca live-control.

Granica odpowiedzialnosci agenta ma status `ready_for_owner_proof`, a nie
deklarowany sukces wizualny w Toolsecie/NWN. Dopoki ta decyzja obowiazuje,
etap proofu wizualnego zamyka wylacznie wynik przekazany przez wlasciciela.

Obsluga Aurora Toolset i klienta NWN odbywa sie tylko wedlug istniejacego,
zweryfikowanego standardu operacyjnego oraz jego adapterow i precedensow.
Przed kolejnym krokiem nalezy najpierw odczytac odpowiedni standard, poprzedni
proof i stan zywej sesji. Nie wolno tworzyc pomocniczego workflow ad hoc ani
zastepowac istniejacego mechanizmu recznymi kliknieciami, globalnym wejsciem,
nowym skryptem sterowania lub bezposrednim uruchomieniem klienta.

Gdy standard albo adapter nie jest dostepny, nalezy to zglosic jako blokade i
ustalic jego lokalizacje z wlascicielem; nie wolno improwizowac zamiennika.
Kazde odstepstwo wymaga wyraznej zgody wlasciciela oraz trwalego zapisu
zakresu, przyczyny i wyniku w `documentation`.

## 2.3 Bramka kolejnej iteracji modelu — HARD STOP

Kolejna iteracja modelu jest dozwolona dopiero po swiezym, wizualnym bledzie
aktualnego kandydata w Aurora Toolset albo w NWN. Za iteracje uwaza sie nie
tylko zmiane MDL/TGA/2DA, lecz takze nowy numer `rNN`, nowy resref, kopie lub
nazwe HAK-a, nowy modul/fixture i ponowne wygenerowanie tego samego lancucha
zasobow.

Obowiazuje nastepujaca bramka:

1. Zapisac jedna tozsamosc kandydata: SHA-256 MOD, HAK, MDL, tekstur i 2DA,
   Area, fixture, Appearance row oraz pozycje.
2. W Toolsecie potwierdzic tozsamosc i wybor dokladnego obiektu oraz zrobic
   swiezy capture zwalidowanego `TScrollBox`.
3. Jezeli model jest widoczny, nie wolno tworzyc kolejnej iteracji. Nastepny
   krok to NWN na tym samym lancuchu artefaktow.
4. Jezeli model nie jest widoczny w Toolsecie albo NWN, zachowac swiezy obraz
   lub runtime packet, zdiagnozowac przyczyne i zadeklarowac minimalna delte
   kolejnej iteracji przed jej utworzeniem.
5. `missing` proofu, zly placement, HAK/build/save, lock hasha, niedomknieta
   geometria, timeout albo blad automatyzacji nie sa dowodem awarii modelu i
   nie zezwalaja na nowy `rNN`.

Jezeli sam kadr nie pozwala stwierdzic, czy dokladny obiekt jest widoczny,
wynik brzmi `modelVisibility=not_tested` i `proofCompleteness=missing`; nie
wolno wyprowadzic z niego `modelVisibility=not_visible` ani nowej iteracji.

Wyniki Aurora i NWN maja dwie osobne osie:

- `modelVisibility = visible | not_visible | not_tested`;
- `proofCompleteness = verified | failed | missing`.

Slowa `missing` nie wolno uzywac jako wyniku widocznosci modelu. Kolejna
iteracje dopuszcza wylacznie swiezy, zwiazany z kandydatem wynik
`modelVisibility=not_visible` w Toolsecie albo NWN. Niekompletny packet ma
`proofCompleteness=missing` i wymaga dokonczenia tego samego kandydata.

Dowod jest monotoniczny: brak swiezego proofu w pozniejszym module nie
uniewaznia poprawnej obserwacji wczesniejszego, jawnie zidentyfikowanego
modulu. Gdy dokladnego kandydata nie da sie wznowic z przyczyn innych niz model,
nalezy zatrzymac lane i zapisac blocker; nie wolno po cichu tworzyc nastepnej
iteracji.

Jezeli pozniejsze bezposrednie polecenie wlasciciela przywroci agent-run proof
dla jednego dokladnego kandydata, wtedy nalezy stosowac
[`MODEL_PROOF_STANDARD.md`](MODEL_PROOF_STANDARD.md) i shared
`$aurora-model-proof-120s` skill. Jeden agent proof owner uzywa jego centralnego
koordynatora dla mierzonej sciezki Toolset-to-NWN; Meshy2Aurora nie tworzy
lokalnego runnera ani UI adaptera.

## 3. TDD

Implementujemy zgodnie z zasada TDD. Najpierw powstaje test lub gate opisujacy oczekiwane zachowanie, potem minimalna implementacja, potem refactor i proof. Dla modeli, animacji, parserow i konwersji assetow test musi byc oparty na realnym zasobie albo na minimalnej fixture opisanej wprost jako fixture.

Wyjatek harmonogramowy zatwierdzony przez wlasciciela dla aktywnej fali M7/S1
jest opisany w `documentation/suplement-implementation-first-m7-s1-codex.md`:
najpierw pierwsza implementacja wiekszosci vertical slices, potem wspolna faza
testow, review i proof gates. Wyjatek zmienia kolejnosc, nie koncowe wymagania
jakosci ani Definition of Done.

## 4. Pliki `*-cloud.md`

Jezeli w dokumentacji pojawia sie plik w formacie `[nazwa]-cloud.md`, oznacza to, ze trzeba dostarczyc suplement cloud do tresci bazowej dokumentacji. Taki plik nie zastepuje dokumentu glownego; dopisuje wymagania, ograniczenia, decyzje, instrukcje lub roznice potrzebne dla pracy w chmurze/Codex Cloud.

Format takich plikow i format odpowiedzi cloud sa opisane w `documentation/CLOUD_SUPPLEMENT_FORMAT.md`.

## 5. Pliki `*-odpowiedz-codex.md`

Pliki w formacie `[temat]-odpowiedz-codex.md` sa odpowiedziami lokalnego Codexa na pytania Cloud z plikow `[temat]-pytania-cloud.md`. Odpowiedz musi byc pod kazdym numerowanym pytaniem `Q1`, `Q2`, ... i miec status `POTWIERDZONE`, `HIPOTEZA` albo `NIE WIEM`.

Pelne reguly wymiany Cloud/Codex sa w `documentation/reguly-dokumentacji-cloud.md`. Wymagany zestaw startowy jest opisany w `documentation/wymagania-startowe-cloud.md`.

## 6. Aktualny produkt webowy

Od decyzji D12-D14 produktem jest aplikacja webowa local-first, a nie desktopowe narzedzie ani CLI jako interfejs uzytkownika. Aktywny kontrakt znajduje sie w:

- `documentation/decyzje-i-zadania-cloud.md` (D11 i D12);
- `documentation/architektura-meshy2aurora-codex.md`;
- `documentation/architektura-web-wasm-codex.md`;
- `documentation/audyt-gotowosci-startowej-2026-07-10-codex.md`;
- `documentation/macierz-gotowosci-wiedzy-codex.md`;
- `documentation/plan-implementacji-orkiestrator-codex.md`;
- `documentation/orchestrator-state.yaml`.

Starsze dokumenty zachowuja fakty i kontekst z chwili utworzenia, ale nie moga nadpisac D11-D14. Pelna klasyfikacja wszystkich plikow jest w `documentation/status-dokumentacji-web-2026-07-10-codex.md`. Nie edytujemy historycznych plikow `*-cloud.md` tylko po to, aby zmienic ich decyzje; ich status zmienia centralny indeks i nowszy suplement Codexa.
