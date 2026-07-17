# Polityka lokalnych modeli testowych Meshy

Status: AKTYWNA. Katalogi opisane tutaj sa jedynym miejscem w projekcie na
lokalne, wybrane przez wlasciciela modele Meshy do testow Studio i Local Bridge.

## Katalogi

```text
C:\Projects\meshy2aurora\test-assets\meshy\
├── incoming\  # nowy model przed decyzja o przydatnosci
└── active\    # model aktualnie potrzebny przez zatwierdzony test/proof
```

Pliki binarne w obu katalogach sa ignorowane przez Git. Do repozytorium nie
trafiajace GLB, tekstury, signed URL-e, klucze Meshy ani kopie assetow. Git
sledzi wyłącznie pliki `.gitkeep`, aby zachowac strukture katalogow.

## Przebieg kwalifikacji

1. Nowy GLB trafia najpierw do `test-assets/meshy/incoming`.
2. Uruchamiamy odpowiedni test/intake Studio i zapisujemy tylko redagowane
   evidence: profil, SHA-256, rozmiar, wynik walidacji i screenshot.
3. Jezeli model jest potrzebny dla aktualnego H1/N1/S1 proof, przenosimy go do
   `test-assets/meshy/active` i wskazujemy go w odpowiednim evidence packet.
4. Jezeli model nie jest wykorzystywany albo nie przechodzi intake, usuwamy go
   z `incoming`. Nie tworzymy katalogu archiwalnego dla odrzuconych modeli.
5. Gdy aktywny model przestaje byc wskazany przez test/proof, najpierw
   zachowujemy hash i wynik proof w `documentation/evidence`, potem usuwamy
   lokalny plik z `active`.

## Reguly bezpieczenstwa i provenance

- Nie dodawaj plikow z tych katalogow przez `git add -f`.
- Nie umieszczaj tu retail/CEP/Aurora reference payloadow; obowiazuje Aurora
  First i zasady provenance projektu.
- Przy kazdym realnym E2E Meshy sprawdz SHA-256 w `MeshyProvenanceV1` przed
  przeniesieniem modelu do `active`.
- Pliki sa lokalne i nie sa automatycznie odczytywane przez Studio. Uzytkownik
  musi je jawnie zaimportowac albo wybrac w przegladarce.

Ta polityka nie zmienia bramek M6/M7/S1: zachowany model nie jest dowodem
Toolsetu ani gry, dopoki wymagany evidence packet nie zostanie kompletny.
