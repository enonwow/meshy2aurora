# The Last City — osobne mockupy creature

Status: zestaw v2 jest aktualnym kierunkiem koncepcyjnym do selekcji. To nie są
jeszcze wygenerowane modele 3D ani dowód rzeczywistej liczby trójkątów.

## Nadrzędny brief fabularny v2

Creature nie są zwykłym bestiariuszem dark fantasy. Mają wynikać z głównej
tajemnicy świata:

- nienaturalna biało-szara Mgła odcięła Ostatnie Miasto od reszty świata;
- Mgła zabija, wywołuje obłęd, zmienia otoczenie i mutuje ludzi oraz zwierzęta;
- część stworzeń powstała z dawnych mieszkańców, uchodźców, marynarzy,
  podróżnych i zwierząt z pochłoniętych wsi;
- potwór powinien zachowywać jeden czytelny ślad dawnej tożsamości;
- fioletowe paleniska i latarnia są ambiwalentnymi symbolami ochrony, dlatego
  fiolet może występować najwyżej jako oszczędny ślad kontaktu, a nie jako
  dominująca „magiczna” stylistyka;
- Mgła jest VFX-em lub materiałem, nie ciężką geometrią modelu.

## Aktualny zestaw v2

| Creature | Plik | Docelowy budżet | Źródło fabularne |
| --- | --- | ---: | --- |
| Powrotnik | `tlc-creature-powrotnik-v2.png` | 13–17k tris | uchodźca, który wrócił z Mgły bez dawnej tożsamości |
| Drogowy Tropiciel | `tlc-creature-drogowy-tropiciel-v2.png` | 10–14k tris | pies zagubiony na drogach zmienianych przez Mgłę |
| Białorogi Tułacz | `tlc-creature-bialorogi-tulacz-v2.png` | 11–15k tris | zwierzę gospodarskie z pochłoniętych okolicznych wsi |
| Brzegowiec | `tlc-creature-brzegowiec-v2.png` | 12–16k tris | marynarz lub rybak, który wyszedł z Mgły nad zatoką |
| Wydrążony Badacz | `tlc-creature-wydrazony-badacz-v2.png` | 13–17k tris | badacz upadłej magii zmieniony przez Mgłę |

## Źródła wejściowe dla Meshy

Mgła określa genezę i wygląd creature, ale nie może być przedstawiana na
obrazie przekazywanym do Meshy. Generator mógłby zinterpretować dym, opary,
wolumetryczne smugi lub cząsteczki jako część geometrii albo tekstury modelu.
Efekty Mgły dodajemy później w grze jako osobny VFX lub materiał.

Obraz wejściowy dla Meshy musi przedstawiać wyłącznie jedną, pełną i
niezasłoniętą sylwetkę na jednolitym neutralnym tle. Dopuszczalny jest tylko
zwarty cień kontaktowy pod stopami. Zabronione są: mgła, dym, para z oddechu,
zamglenie, cząsteczki, smugi wolumetryczne, sceneria i pedestal.

Aktualne czyste źródło:

| Creature | Źródło Meshy | Powiązany koncept |
| --- | --- | --- |
| Powrotnik | `tlc-creature-powrotnik-meshy-v1.png` | `tlc-creature-powrotnik-v2.png` |

## Twarda zasada geometrii

Każdy creature ma osobny limit całego finalnego modelu:

`triangles <= 300_000`

Limit obejmuje ciało, ubranie/pancerz, uzbrojenie i wszystkie dodatki. Drobne
łuski, sierść, korozja, blizny i zabrudzenia mają trafić do tekstur/normal maps,
a nie do osobnej geometrii.

## Reguły wspólne

- jeden obraz przedstawia dokładnie jeden creature;
- pełna sylwetka i czytelne stawy muszą umożliwiać rigowanie;
- bez pedestalów, elementów otoczenia i cech placeable;
- maksymalnie jedna broń lub jeden prosty rekwizyt będący częścią modelu;
- bez modelowanych włosów, wielu łańcuchów, mikro-kolców i drobnych warstw
  pancerza;
- realny budżet zostanie zatwierdzony dopiero na wygenerowanym mesh po
  triangulacji.

## Status poprzednich wariantów

Zestaw v1 oraz `tlc-five-creatures-concept-lineup-v1.png` są zachowane
historycznie, ale zostały odrzucone jako źródło do dalszego generowania:

- lineup był zbyt mały, aby ocenić pojedyncze sylwetki;
- v1 był generycznym bestiariuszem dungeonowym i nie wynikał dostatecznie z
  centralnej roli Mgły;
- v1 nie wiązał każdego projektu z dawnym człowiekiem, zwierzęciem lub miejscem
  utraconym po katastrofie.
