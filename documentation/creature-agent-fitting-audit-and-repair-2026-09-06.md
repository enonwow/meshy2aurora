# Audyt dopasowania psa Tripo do c_wolf — 2026-09-06

## Wynik i granica ukończenia

Audyt i opisane poniżej poprawki aplikacji są wdrożone lokalnie. Przygotowano **edytowalny model ze szkieletem, wagami i teksturą**. Nie przygotowano dopuszczonego produktu NWN/HAK: wybrany model przechodzi bieżącą kontrolę skinningu i bind pose, lecz 10 z 42 odziedziczonych animacji nadal nie spełnia kontroli lokalnej deformacji. Nie obniżono progów testów ruchu.

Pliki modelu: [manifest i hashe](../artifacts/creatures/borzoi-cwolf-editable-20260906/manifest.json), [Blender](../artifacts/creatures/borzoi-cwolf-editable-20260906/borzoi-cwolf-rigged.blend), [rigowany GLB](../artifacts/creatures/borzoi-cwolf-editable-20260906/borzoi-cwolf-rigged.glb), [render](../artifacts/creatures/borzoi-cwolf-editable-20260906/borzoi-cwolf-preview.png). To pliki robocze; nie są oznaczone jako game-ready ani ready_for_owner_proof.

## Dokładne wejście

- Źródło użytkownika: wolf+3d+model.glb, Tripo, 20 534 trójkąty, SHA-256 dc22ecb0561fc10773e1a156b0d126225a15cdfb67cd7c0999a965a4f1d59689.
- Kanoniczne źródło: sample-3d/borzoi-tripo-dc22ecb0/source.glb, byte-identical do wybranego pliku Downloads.
- Dopasowana bryła: source-cwolf-bind.glb, SHA-256 a2d74adb6dd36c8577c59a331ba98d0b2f41c4c544d1782a6968997b96a876ed. Jednorazowa transformacja położeń zarejestrowana w asset.extras.m2aReferenceBindV1. Indeksy, UV i osadzona tekstura źródłowa zachowane.
- Referencja: c_wolf -> NULL, binarny MDL 340 812 B, SHA-256 a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726. Odczyt z KEY/BIF w pamięci. Nie skopiowano detalicznych danych siatki ani klipów retail do dostarczonych GLB/Blender.

## Ustalenia audytu

1. Zadanie **Audit creature** (01a00f31-d2af-7d80-8043-3f5f7ed0eff8) początkowo analizowało inne źródło, później to sprostowało. Ograniczenia starego modelu high-poly nie dotyczą obecnego Tripo.
2. Poprzednia długa próba powtarzała tuning solvera bez uzyskania edytowalnego rigu. Zmieniony backend nie został wtedy przebudowany do WASM Studio.
3. Przygotowanie authoringu wymagało wcześniejszego sukcesu automatycznego skinningu. Błąd jakości usuwał drogę do ręcznej/AI korekty tego samego rigu.
4. Walidacja po korekcie kopiowała poprzednie validationViolations i nie przeliczała gradientu aktualnych wag. Werdykt mógł pozostać nieaktualny.
5. Sztywny limit zmiany wagi na jednej krawędzi uzależniał wynik od liczby podziałów siatki. Należy mierzyć zmianę względem długości krawędzi i rozpiętości kości.
6. Globalny Wolf_rootdummy był traktowany jako wymagający własnego obszaru skóry. To przodek ruchu wszystkich części; musi pozostać w hierarchii, ale nie wymaga bezpośrednich wag.
7. Pojedyncza linia przodków to ograniczenie dotychczasowego solvera, nie ogólne wymaganie LBS. Lokalne połączenia sąsiednich gałęzi powinny być legalne, a odległe kończyny odrzucane. Pasywne węzły pośrednie nie powinny sztucznie zwiększać odległości deformujących kości.
8. Indeksy wierzchołków z binarnego MDL nie są indeksami wejściowego GLB. Lokalizacja napraw po takich indeksach bez mapowania może zmieniać niewłaściwe miejsce.
9. Historyczny eksport diagnostyczny V10 nie był dowodem pełnej zgodności animacji. Jego kontrola ruchu także zwracała BLOCKED.
10. Końcowy optymalizator produktu nadal zwraca błąd bez edytowalnego wyniku ostatniego ulepszenia. Nie naprawiono jeszcze tego osobnego ograniczenia.

## Plan i wykonanie

| Zakres | Implementacja / wynik |
| --- | --- |
| Edytowalny draft po błędzie jakości | retain_editable_draft_on_quality_failure, jawne NEEDS_AUTHORING, zachowane blokady eksportu; nieprawidłowe wejścia nadal odrzucane |
| Bieżąca walidacja | Ponowne obliczenie naruszeń, gradientów, lokalności i pokrycia z aktualnych wag; historia inicjalizacji pozostaje w polach diagnostycznych |
| Geometria gradientu | Usunięty stały limit 5% na pojedynczej krawędzi; pozostawiona miara geometryczna i niezależny test ruchu |
| Klasyfikacja szkieletu | Zachowane wszystkie 30 węzłów; 22 wymagane kości powierzchni. Konserwatywna reguła wspólnego, nierenderującego przodka dla referencji z częściami rigid |
| Lokalność authoringu | Dozwolone ograniczone sąsiedztwo deformujących kości; test odległych kończyn i wstawienia pasywnego transformu. Stare ograniczenia solvera nie zostały globalnie usunięte |
| Sterowanie AI | WebMCP m2a_creature_import_authoring: URL tego samego originu, limit wielkości, SHA-256 i identyfikacja source/chain |
| Przewidywalny preview | Diagnostyczny preview pokazuje aktualne wagi bez ukrytego dodatkowego optymalizowania |
| Materiał podglądu | Oryginalny materiał GLB na rzeczywistej geometrii MDL; odrębne oznaczenie od natywnego testu materiału. Uwzględniona konwersja V między glTF/Aurora |
| Aktualny backend w Studio | Przebudowano WASM; sprawdzono load_source -> select_supermodel -> prepare -> import_authoring -> preview w przeglądarce |
| Model do edycji | GLB z JOINTS_0/WEIGHTS_0 i macierzami inverse bind; reimport do Blendera, zapis .blend i render |

Pomocnicze narzędzia authoringu są w tools: author-creature-weights-blender.py, constrain-creature-weight-proposal.mjs, regularize-creature-weight-gradients.mjs, repair-creature-weight-witnesses.mjs, repair-creature-motion-neighborhoods.mjs. Każdy wynik propozycji pozostaje NOT_VALIDATED do przejścia niezależnej kontroli Creature. Lokalna naprawa używa położeń referencyjnych; nowsza wersja sprawdza też zgodność wag świadków, aby odrzucać stary raport.

## Wybrany wynik modelu

Wybrano gradient-v8; późniejsze próby są zachowane diagnostycznie, ale nie zastępują tego wyniku.

- 25 830 wierzchołków renderera, 20 534 trójkąty, 30 węzłów hierarchii, 22 kości z wagami.
- Pusta lista bieżących naruszeń skinningu. Bind i skin bind zgodne.
- 42 klipy odziedziczone; 869/869 wymaganych par joint/clip ma pokrycie. To miara ruchu, nie brak deformacji.
- Zero naruszeń szwów. Nadal występują lokalne zapadnięcia powierzchni.
- Klipy z odrzuconą deformacją: ca1slashl, ca1stab, creach, cconjure1, ckdbck, ckdbckdie, ccwalkf, ccwalkb, ctaunt, cdead.
- Próba pełnego produktu zakończyła się M2A-SUPERMODEL-MINIMAL-MTR-MOTION-QUALITY-BLOCKED; folder produktu nie zawiera ukończonego HAK.
- LOD 7186 trójkątów zwiększył fragmentację powierzchni, pogorszył kontrolę komponentów i został odrzucony jako podstawa modelu.
- Silniejsza lokalna zmiana wag naprawiała część wcześniejszych ekstremów, lecz pogarszała inne klipy; również ją odrzucono.

Ponowny import dostarczonego GLB do Blendera: maksymalny błąd pozycji bazowej 2.39e-7 jednostki. Osobne próbne obroty lewej/prawej przedniej łapy, obu tylnych łap, obu części ogona i głowy dają mierzalny ruch skóry. Plik .blend zawiera rig i materiał, lecz nie kopiuje animacji retail. Aplikacja odtwarza je z referencji w pamięci.

## Walidacja zmian

- Testy sesji/WebMCP: 13/13 PASS; TypeScript PASS.
- Build natywny adaptera Creature i wasm-pack PASS. Aplikacja rzeczywiście załadowała nowy WASM i przyjęła authoring.
- Testy Core: 228 PASS, 2 FAIL, 3 ignored. Dwa niezmieniane testy surface-anatomy mają nieaktualne oczekiwania: largest_surface_component_is_the_only_authoritative_joint_fit_body oraz disconnected_authoritative_contact_surface_contributes_ground_landmarks. Pełna suita nie jest zielona.
- Nowe regresje obejmują bieżące przeliczanie walidacji, niezależność gradientu od liniowego podziału krawędzi, lokalne rozgałęzienie, odrzucenie odległego wsparcia, pasywny węzeł pośredni i globalny dummy bez własnej powierzchni.
- Guard kanonicznego repozytorium i layoutu sample-3d PASS.
- Procesy uruchamiane przez Node z windowsHide:true, stdout/stderr i kod zakończenia zapisane. Nie sterowano Toolset/NWN ani obcymi oknami.

Dowody robocze: artifacts/diagnostics/tripo-cwolf-agent-audit-20260906 — raporty native/WASM, testów, propozycje wag, wyniki motion, log nieudanego produktu i odrzuconego LOD. Dokładny manifest dostarczonego modelu jest oddzielony od tych eksperymentów.

## Pozostała praca

Nie wystarczy ponownie zwiększyć budżetu solvera. Potrzebna jest korekta anatomii wag i ewentualnie lokalnej topologii przy barku/pod szyją i przegubach kończyn, oceniana jednocześnie na całym zestawie klipów. Każda propozycja powinna zachowywać ostatni lepszy wynik i porównanie per clip. Osobna poprawka aplikacji powinna umożliwiać jawne, ograniczone refine z zapisem wag i świadków także po niepowodzeniu produktu. Natywna kontrola należy do właściciela zgodnie z AGENTS.md; nie przygotowano jeszcze kandydata do tego etapu.

Kontrola końcowa UI: po pełnym przeładowaniu Studio potwierdzono poprawny kierunek tekstury na rzeczywistym MDL oraz odtwarzanie cwalk. Widoczny podgląd pozostał otwarty na http://127.0.0.1:5186/?creatureAgent=1, revision 5, DIAGNOSTIC_ONLY; model SHA-256 WASM dd1296731bf11646fe7e4a4fb2772d0e2ce71b5e5362cd54e989ec28bca260a8. To obserwacja przeglądarkowego readback, nie natywny dowód NWN.
