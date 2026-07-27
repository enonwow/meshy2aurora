# M0 — blokada live Toolset po recovery (2026-07-20)

## Zakres

Aktualny cel to uzyskanie widocznego modelu Meshy M0 w Aurora Toolsecie i NWN.
Dotyczy to świeżej, wersjonowanej deklaracji `m2a_m0v10` i aktywnego HAK-a
`m2a_m0_v6` (SHA-256 `61A07A903C43C6AF32C5180AFB4C7678FF6EAE09F1D87A296522081F63969EF4`).

## Fakty z live Toolsetu

- Centralny `bootstrap-dry-run` przeszedł dla deklaracji
  `proof-output/m0-v10-central-bootstrap/bootstrap-declaration.json`.
- `bootstrap-create` uruchomił dokładnie jeden Toolset, ale przed kreatorem
  modułu pojawił się modal `#32770 / Confirmation` z pełnym tekstem:

  ```text
  The toolset did not close properly last time it was used.
  A module was found, would you like to try to recover it?
  Pressing 'No' will destroy the backup.
  ```

- Odczyt wykazał pojedyncze aktywne przyciski `&Tak`, `&Nie`, `Anuluj` na
  `\\.\DISPLAY1`. Wybrano wyłącznie dokładny, niedestrukcyjny `&Tak` przez
  ukierunkowany `BM_CLICK`; `&Nie` nie został użyty.
- Modal zniknął, lecz po dwóch obserwacjach przez łącznie co najmniej 20 s
  proces `nwtoolset.exe` PID `36888` pozostał responsywny z pustym tytułem
  `BioWare Aurora Neverwinter Nights Toolset v89.8193.37-17`: brak nazwy
  modułu, Area, kreatora i modalu.
- Żaden moduł, HAK, plik INI ani ustawienie użytkownika nie zostały zmienione.
  Proces nie został zamknięty ani zrestartowany.

## Przyczyna transportowa

Fakt: centralny atom tworzenia czekał wyłącznie na `TdlgWelcome`. Standardowy
modal recovery nie był rozpoznawany, więc atom zakończył się
`expected_one_exact_dialog:0` przed wejściem do Module Wizard. To jest
blokada trasy Toolsetu, nie dowód błędu binarnego modelu.

## Naprawa centralna i test

W `C:\Projects\aurora-web\backend\scripts\create-aurora-toolset-module-and-area-no-global-input.mjs`
dodano wąskie rozpoznanie wyłącznie powyższego tekstu recovery. Wymaga ono
jednego dokładnego modalu, jednego przycisku afirmatywnego (`Tak`/`Yes`) i
jednego negatywnego (`Nie`/`No`), po czym wysyła jedynie ukierunkowany `post`
do afirmatywnego przycisku. Każdy inny tekst lub układ jest odrzucony.

Testy:

- `node backend/scripts/test-create-aurora-toolset-module-and-area-recovery-contract.mjs` — passed.
- `node backend/scripts/test-aurora-toolset-vertical-slice-contract.mjs` — exit code 0;
  wypisuje istniejący negatywny przypadek `bootstrap_git_module_index_missing`.

## Status i następny krok

`failed` dla konkretnego bootstrapu `m2a_m0v10`; proof widoczności modelu w
Toolsecie oraz NWN jest nadal `missing`.

Następny live przebieg może rozpocząć się wyłącznie od czystej, pojedynczej
sesji Toolsetu i nowego centralnego `bootstrap-create`; obecny PID `36888`
pozostaje otwarty, ponieważ zamknięcie nie zostało zlecone przez właściciela.
Po prawidłowym zapisie modułu należy wykonać natywny readback GIT oraz capture
Area `TScrollBox`, a dopiero potem profil AUR-S07 dla NWN.

## Kontynuacja: pozycja fixture M0 w centrum Area

### Zweryfikowane fakty

- Bieżący Area `m2a_m0a6` ma cztery tile'e (`2 x 2`), zatem jego zakres
  światowy wynosi `0..20` na osiach X i Y. Matematyczny środek viewportu
  Area to `X=10,00`, `Y=10,00`.
- W aktywnym `TdlgLocation` pozycja fixture'a pozostaje `X=5,57`, `Y=14,53`,
  `Z=3,20`. HAK, Appearance, konfiguracja Toolsetu i plik modułu nie zostały
  w tej kontynuacji zmienione ani zapisane.
- Dekompilacja retailowego `nwtoolset.exe` (`FUN_004e5f3c`) pokazuje, że
  `Apply/OK` przenosi X/Y/Z tylko dla osi, które mają ustawione flagi dirty.
  Handlery górnej/dolnej strzałki `TUpDown` (`FUN_004e6fdc` i
  `FUN_004e709c`) aktualizują tekst i odpowiednio ustawiają flagi X/Y.
- Handler edycji tekstowej `FUN_004e6d80` parsuje `TEdit`, synchronizuje
  odpowiadający `TUpDown` i ustawia przekazaną flagę dirty. Jest to
  wymagany mechanizm dla natywnej zmiany wartości wpisanej ręcznie.

### Odrzucone transporty

- Ukierunkowane `WM_VSCROLL/SB_LINEUP` do poprawnego rodzica `Position` nie
  wywołało handlera: spinery pozostały na `9,99`, pola na `5,57/14,53`.
  Runner anulował modal i ponownie potwierdził stan źródłowy.
- Wcześniejsze `WM_SETTEXT`, `EN_CHANGE`, `EM_REPLACESEL`, klawisze,
  `UDM_SETPOS32`, `CN_NOTIFY`, `WM_HSCROLL` i akcje dostępności nie
  utworzyły trwałej flagi dirty; nie są powtarzane bez zmiany kontraktu.

### Następna wąska hipoteza

Następny adapter musi wywołać zwykłe VCL-owe `CN_COMMAND` dla samego
`TEdit` po zmianie tekstu, z natychmiastowym odczytem tekstu, pozycji `TUpDown`
i flagi pośrednio przez zachowanie `Apply`/ponowne otwarcie. To jest inna
ścieżka niż wysłany do dialogu Win32 `WM_COMMAND`. Dokumentacja Win32
potwierdza, że zwykły edit wysyła `EN_CHANGE` przez `WM_COMMAND`, a
`WM_CHAR` i `EM_REPLACESEL` generują jego powiadomienia; dekompilacja
potwierdza dodatkowe odbicie VCL `CN_COMMAND` (`0xBD11`) wewnątrz kontrolki.
Przed akcją live adapter wymaga testu kontraktowego i fail-closed rollbacku
modalu; nie wolno wpisywać flag do pamięci ani wywoływać kodu w procesie.

### Korekta sekwencji natywnej od właściciela (live)

Właściciel doprecyzował zachowanie `TdlgLocation`: `Apply` przyjmuje wartości
widoczne w polach, a `OK` akceptuje je i zamyka dialog. Dlatego synchronizacja
pozycji `TUpDown` przed `Apply` nie jest bramką poprawności. Ostatni adapter
bezpiecznie pokazał `10,00 / 10,00`, lecz został anulowany przed `Apply` tylko
dlatego, że up/down pozostały `557 / 1453`. Następna trasa ma wykonać dokładnie:
ustawienie X/Y, `Apply`, odczyt pól, `OK`, ponowne otwarcie `Adjust Location`
i dopiero wtedy decyzję o zapisie oraz proofie viewportu.

### Wynik sekwencji `Apply -> OK` i kolejna ograniczona trasa

Fakt z live readbacku: wykonano dokładnie `X=10,00 / Y=10,00` w polach,
następnie `Apply`, odczyt pól, `OK` oraz ponowne otwarcie `Adjust Location`.
Ponownie otwarty dialog nadal podał `5,57 / 14,53 / 3,20`. Zatem sam
programowy tekst, nawet razem z VCL `CN_COMMAND/EN_CHANGE`, nie uruchomił
tej samej flagi dirty co natywne kliknięcie strzałki. Moduł nie został zapisany.

Nowy, jeszcze niewykonany adapter centralny ma jednorazową autoryzację
wyłącznie od tego dokładnego failure: dla każdej osi ustawia natywny
`TUpDown` na wartość kandydata minus jego przyrost, po czym wysyła
synchroniczne, ukierunkowane `WM_LBUTTONDOWN/UP` do górnej strzałki tego
samego HWND. Dekompilacja potwierdza, że ta zdarzeniowa metoda odczytuje
bieżącą pozycję, formatuje `TEdit` i ustawia flagę dirty dla X/Y. Adapter
wymaga przed `Apply` zgodności zarówno tekstu, jak i natywnej pozycji;
następnie stosuje dokładnie `Apply -> OK -> reopen`. Nie zmienia HAK-a,
Appearance, pliku INI ani nie korzysta z globalnej myszy/klawiatury.

Weryfikacja offline przed live: `node --check` dla atomu i runnera,
PowerShell compile-only transportu oraz
`test-aurora-toolset-vertical-slice-contract.mjs` (exit `0`; istniejący
negatywny przypadek wypisuje `bootstrap_git_module_index_missing`).

## Korekta bieżącego stanu po odczycie native — 2026-07-20

### Fakty

- Read-only parser aktualnie otwartego
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0v6.mod`
  odczytał fixture `nw_dwarfmerc001` z `Appearance_Type=848` na
  `X=10.0`, `Y=10.0`, `Z=3.200000047683716`.
- Area `m2a_m0a6` jest obszarem `2 x 2`; punkt `(10.0, 10.0)` jest jego
  geometrycznym środkiem w siatce tile'i `10 x 10`.
- HAK `m2a_m0_v6.hak` ma bieżący SHA-256
  `61A07A903C43C6AF32C5180AFB4C7678FF6EAE09F1D87A296522081F63969EF4`.
- Jedyny responsywny Toolset PID `36888` nadal ma dokładny tytuł
  `m2a_m0v6.mod`; nie ma modalu ani aktywnego buildu.

### Co to znaczy

Stare twierdzenie o niezatwierdzonej pozycji `5.57 / 14.53` jest historyczne
i nie może kierować dalszymi akcjami. Łańcuch GIT → Appearance 848 → HAK jest
potwierdzony, ale **nie** jest to dowód renderera. Fresh capture `TScrollBox`
pozostaje `missing`: poprzedni hook odrzucił go jako
`toolset_viewport_obscured_by_foreground_window`. Nie wolno ponawiać tej samej
próby bez zmienionego stanu widoczności ani twierdzić, że canvas pokazuje
krasnoluda lub M0 bez zaakceptowanego obrazu.

### Najbliższa bezpieczna akcja

Gdy `TScrollBox` będzie rzeczywiście odsłonięty, uruchomić raz dokładny
read-only capture, obejrzeć wynik i pokazać ten sam PNG użytkownikowi w
rozmowie. Dopiero werdykt z tego obrazu wybierze następny lane: brak obiektu
→ pozycja; niewłaściwy model → diagnostyka resolution; M0 → runtime NWN.

## Aktualizacja live: Focus on Object i widoczność M0 — 2026-07-20

Ten wpis koryguje historyczny stan `M0-VIEWPORT-PROOF-MISSING` powyżej. Nie
usuwa historii wcześniejszego, zasłoniętego capture'u.

1. Na aktualnie zaznaczonym fixture `Dwarf Mercenary` otwarto jego widoczne
   menu kontekstowe i wywołano natywny element `&Focus on Object` (`ID=2`,
   pozycja `0`). Wykonanie zostało potwierdzone przez zamknięcie tego samego
   popupu; nie użyto globalnej myszy ani klawiatury.
2. Pozycję zmieniono przez natywne pola `TdlgLocation`, z readbackiem po
   `Apply` i `OK`, na `X=7,50`, `Y=7,45`, `Z=3,20`. Następny read-only parser
   zapisanego GIT odczytał odpowiednio
   `7.5 / 7.449999809265137 / 3.200000047683716`.
3. Dobór X/Y nie był zgadywaniem z miniatury. Area `m2a_m0a6` ma `2 x 2`
   tile'e po `10 x 10`; lokalnie odczytane WOK dla `tcn01_c01_03` oraz
   `surfacemat.2da` wskazały punkt `7.503 / 7.450 / 0.200` wewnątrz
   największego trójkąta `Stone` (`Walk=1`) tile'a `(0,0)`. Ustawiono
   bezpiecznie zaokrąglony X/Y; Z nie został zmieniony.
4. Pierwszy świeży, dokładny `TScrollBox` po zmianie pozycji nie pokazał
   M0, lecz był to kadr miasta/zasłoniętego obiektu, a nie dowód krasnoluda.
   Artefakt zachowano jako negatywny wynik tej konfiguracji kamery:
   `proof-output/m0-v6-central-bootstrap-city-exterior-20260719/viewport-after-7-50-7-45-20260720/tscrollbox.png`.
5. Wykonano pojedynczą, uzasadnioną zmianę kamery `pitch-up`. Kontroler
   zwrócił `toolset_camera_pose_unavailable`, więc nie jest to kalibrowany
   pomiar stopni; kontroler zdążył jednak wykonać akcję i zapisać obraz po
   niej. Niezależny, następny capture właściwego `TScrollBox` jednoznacznie
   pokazuje docelowy, duży siatkowy model M0 w Area — nie fallbackowy model
   `Dwarf Mercenary`:
   `proof-output/m0-v6-central-bootstrap-city-exterior-20260719/viewport-m0-visible-after-pitch-up-20260720/tscrollbox.png`.
   Ten sam PNG został obejrzany przez operatora i pokazany właścicielowi w
   rozmowie.

### Bieżący werdykt

- **Aurora viewport:** `verified` dla widoczności docelowego M0, powiązany z
  `m2a_m0v6.mod`, Area `m2a_m0a6`, fixture `nw_dwarfmerc001`,
  `Appearance_Type=848`, HAK `[m2a_m0_v6]`, modelem `m2a_m0p01` i teksturą
  `m2a_m0t01`.
- **NWN runtime:** nadal `missing`. Nie uruchomiono NWN ani nie złożono
  twierdzenia o runtime.
- To nie jest jeszcze formalna końcowa paczka profilu M0: istniejący manifest
  bootstrapu ma hash sprzed zapisu pozycji, a AUR-S07 wymaga nowego,
  niemutowalnego modułu oraz zweryfikowanej bramki geometrii/entry surface.
  Brak tej bramki nie upoważnia do tworzenia własnego launchera ani do
  deklarowania sukcesu runtime.

### Obserwacja runtime bez twierdzenia — 2026-07-20 16:34

- Read-only lista procesów wykazała jedną responsywną instancję `nwmain`
  `PID=43064`, z tytułem `Neverwinter Nights: Enhanced Edition
  v89.8193.37-17 [26c6e573]`.
- Dokładna próba capture jej HWND została odrzucona: wynikowy obraz zawierał
  już okno Codex, a ponowny odczyt nie znalazł procesu `43064`. Nie zachowano
  go jako proof i nie wyciągnięto żadnego wniosku o module, Area ani modelu w
  runtime.
- Nie uruchomiono kolejnego `nwmain`, nie zamknięto Toolsetu i nie wykonano
  żadnej zmiany w module/HAK/MDL/INI. Runtime pozostaje `missing`.
