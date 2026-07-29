# Item Authoring V1 — mockup

Status: `DESIGN_MOCKUP_COMPLETE_NOT_IMPLEMENTATION`

Data: 2026-07-29

Mockup przedstawia rekomendowaną opcję `Item` w webowym Studio po audycie
możliwości Meshy oraz kontraktu przedmiotów Aurora.

## Zakres

Prototyp obejmuje trzy stany:

1. `Source` — jeden target Item, wybór `BaseItem` oraz przypisanie źródła do
   każdego wymaganego slotu partu;
2. `Prepare Item` — osobne zasoby partów, ich transformy, szwy i podgląd
   złożonego przedmiotu;
3. `Review Output` — readback każdego MDL partu, UTI oraz handoff do owner-run
   proof złożonego przedmiotu.

Najważniejsze decyzje:

- nie ma osobnych trybów `Shield`, `Weapon` ani innych kategorii użytkowych;
- użytkownik wybiera rekord `BaseItem`, a Studio odczytuje z `baseitems.2da`
  `ItemClass`, `ModelType`, slot wyposażenia i rozmiar ikony;
- `ModelType` wyznacza schemat wymaganych slotów, ale nie tworzy jednego
  zbiorczego modelu;
- każdy part ma własne źródło, numer wariantu, resref MDL, teksturę oraz
  transform w przestrzeni złożenia;
- UTI zapisuje numery partów, a runtime składa odpowiadające im zasoby MDL;
- złożony podgląd i ikona powstają z całego zestawu partów; podgląd nie jest
  dodatkowym zasobem MDL;
- wybrany w mockupie `BaseItem 1 · Longsword · ModelType 2` jest scenariuszem
  demonstracyjnym pokazującym trzy sloty `B/M/T`;
- sposób składania modelu wynika z `ModelType`, nie z ręcznie wybranej rodziny;
- MDL, ikona i UTI mają osobne tożsamości zasobów;
- tryby `Composed`, `Exploded` oraz `Icon` pokazują odpowiednio wynik złożenia,
  osobne zasoby partów i ikonę całego przedmiotu;
- transform i pivot są rozwiązywane per part, a nie przez Meshy `auto_size`;
- limit produktu wynosi 300 000 trójkątów;
- budżet geometrii jest liczony dla sumy partów jednego przedmiotu;
- interfejs nie deklaruje sukcesu wizualnego Toolset/NWN;
- końcowy proof pozostaje po stronie właściciela.

Nazwy i numery wariantów pokazane w mockupie są danymi demonstracyjnymi, nie
zarezerwowanymi wartościami produkcyjnymi. Implementacja musi wykonać kontrolę
kolizji zasobów przed ich przydzieleniem.

## Pliki

- `index.html` — trzy stany workflow;
- `styles.css` — wygląd zgodny z aktualnym Studio;
- `app.js` — lokalna nawigacja, zakładki i tryby podglądu;
- `01-item-target.png` — wybór opcji Item;
- `02-prepare-item.png` — przygotowanie przykładowego przedmiotu;
- `03-review-package.png` — przegląd paczki.

Mockup jest samodzielnym prototypem dokumentacyjnym. Nie uruchamia Meshy API,
nie tworzy MDL/UTI/HAK/MOD i nie jest dowodem Aurora Toolset ani NWN.

## Uruchomienie

```powershell
python -m http.server 43129 --bind 127.0.0.1
```

Następnie otwórz:

`http://127.0.0.1:43129/index.html`

## Zweryfikowane widoki

### Wybór targetu Item

![Wybór targetu Item](01-item-target.png)

### Przygotowanie przedmiotu

![Prepare Item](02-prepare-item.png)

### Przegląd paczki

![Review Item package](03-review-package.png)

Kontrola 2026-07-29:

- [x] trzy stany wyrenderowane i sprawdzone w lokalnej przeglądarce;
- [x] nawigacja Source → Prepare Item → Review Output działa;
- [x] widoki Composed, Exploded oraz Icon przełączają się;
- [x] zakładki Parts, Transform, Icon i Checks przełączają się;
- [x] konsola przeglądarki bez błędów i ostrzeżeń;
- [x] mock rozdziela MDL, ikonę, UTI i proof package;
- [x] mock nie deklaruje sukcesu wizualnego Toolset/NWN.
