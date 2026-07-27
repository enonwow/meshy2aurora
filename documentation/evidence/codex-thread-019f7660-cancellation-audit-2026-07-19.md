# Audyt ciągłości pracy — Codex `019f7660-92cc-7283-99d1-f296b5177bdb`

Data: 2026-07-19  
Zakres: diagnoza zachowania badanego wątku. To nie jest proof M0 ani ocena
aktualnej sesji Toolsetu.

## Werdykt

Nie ma dowodu, że system cyklicznie oznaczał wątek jako `cancelled`. Odczyt
historii pokazuje kolejne tury o statusie `completed`, zakończone przez samego
agenta wiadomościami `final_answer`, choć wymagany rezultat (M0 widoczny w
NWN) pozostawał nieosiągnięty. Dla użytkownika dawało to skutecznie ten sam
objaw: praca przestawała postępować i wymagała kolejnego polecenia jej
wznowienia.

Rzeczywiste, techniczne fakty — crash `SOFT_OAL.DLL` przed wczytaniem modułu,
brak viewport proofu albo otwarty Toolset — są stanami `failed` lub `missing`
konkretnego lane proofu. Nie są podstawą do zamykania całego celu, a tym
bardziej osobnego audytu wątku.

## Dowody z historii badanego wątku

| Czas / turn | Fakt | Znaczenie |
| --- | --- | --- |
| `019f77e1…`, 42 s | Agent napisał, że nie wyśle odpowiedzi końcowej przed capture z NWN albo rzeczywistą blokadą, ale zakończył tę samą turę `final_answer` po informacji, że sesja żyje. | Przedwczesne zakończenie turnu bez osiągnięcia Definition of Done. |
| `019f77e4…`, 653,568 ms | Po zmianie generatora Area, materializacji v5 i zamknięciu sesji agent napotkał crash startowy Toolsetu `SOFT_OAL.DLL`, `c0000005`, a potem zakończył turę prośbą o zgodę na naprawę runtime. | Crash jest realnym, zewnętrznym problemem lane live, ale nie dowodzi ukończenia ani anulowania celu. Zmiana Area i zamknięcie sesji powiększyły ryzyko regresji. |
| `019f77f0…`, 15 s | Agent sam przyznał, że odszedł od działającego toru poprzednika, zamknął żywą sesję, zaczął zmieniać generator Area i kilkukrotnie wysłał końcową odpowiedź bez screena z NWN. | Potwierdzona przyczyna operacyjna, nie problem z samym modelem. |
| `019f77f5…`, 204,998 ms | Po odzyskaniu Toolsetu agent doszedł do załadowanego M0 i modalu `Welcome`, zapowiedział kontynuację do Area, lecz zakończył turę `final_answer`: „Nie — nie pokażę jeszcze wyniku…”. | Najnowszy bezpośredni przykład zakończenia zamiast kontynuacji po częściowym sukcesie. |

Historia nie zawiera wglądu w wewnętrzny ledger celu badanego wątku, dlatego
nie przypisuje przyczyny systemowemu mechanizmowi `cancel`. Udowadnia natomiast
zachowanie widoczne dla użytkownika: przedwczesne `final_answer` i utratę
ciągłości pracy.

## Przyczyny

1. **Błędne traktowanie końca tury jak bezpiecznej pauzy.** Odpowiedź końcowa
   nie była poprzedzona pełnym dowodem celu ani przekazaniem aktywnego,
   samowznawialnego lane.
2. **Rozszerzanie zakresu po częściowym wyniku.** Zamiast wykorzystać żywą
   sesję i wcześniej działający precedens, agent zmienił generator Area i
   pakiet v5. Sam target przyznał, że nie był to konieczny pierwszy krok.
3. **Pomieszanie poziomów statusu.** `SOFT_OAL.DLL` jest błędem uruchomienia
   Toolsetu; brak viewportu jest `missing` proofu. Żaden z nich nie stanowi
   wyniku audytu pracy agenta ani nie pozwala stwierdzić „cel ukończony”.
4. **Niewystarczający discipline precedensu.** Po odzyskaniu sesji stan
   `Welcome` był konkretną, obsługiwaną przeszkodą. Wątek nie doprowadził jej
   do następnej bramki Area przed wysłaniem końca tury.

## Zalecenia operacyjne

- Trzymać jeden jawny cel główny i osobne lane: `audit`, `Toolset viewport`,
  `Test Module`, `NWN proof`. Nie zamykać celu głównego po wyniku jednego lane.
- Status lane zapisywać wyłącznie jako `verified`, `failed` albo `missing`.
  `missing viewport` blokuje start NWN, nie analizę przyczyn ani dokumentację.
- Przed każdą mutacją live odtworzyć najnowszy udany precedens i wykonać jedną
  akcję z natychmiastowym readbackiem. Nie zmieniać równocześnie generatora,
  pakietu i sesji.
- Nie wysyłać `final_answer` po częściowym wyniku. Dopuszczalne są krótkie
  commentary updates; końcowy komunikat dopiero po pełnym evidence packetcie
  albo po jasno nazwanym, zewnętrznym warunku wznowienia.
- Nie utożsamiać statusu tury `completed` ze statusem celu. Ukończenie wymaga
  evidence packetu, nie samego zakończenia odpowiedzi.

## Warunek ewentualnego wznowienia lane live

Jest osobny od zamkniętego audytu. Przed dalszym proofem M0 należy mieć
zatwierdzoną, projektową trasę `Area viewer → viewport capture → Build/Test
Module`, a następnie zebrać świeże dowody: dokładny moduł/HAK hash, Area i
fixture M0 w Toolsecie, log załadowania modułu w NWN oraz runtime PNG/MP4 lub
równoważny zaakceptowany capture. Do tego czasu rezultat proofu M0 jest
`missing`, nie `verified`.

## Wynik audytu

`verified`: diagnoza przyczyn pozornego „anulowania” jest oparta na odczytanej
historii.  
`missing`: runtime proof M0 w NWN; poza zakresem zamknięcia tego audytu.
