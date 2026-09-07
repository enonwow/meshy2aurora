# Kanoniczny uklad modeli Meshy

Status: `AKTYWNY / HARD STOP`

Data decyzji naprawczej: 2026-07-27

## Decyzja

Jedynym katalogiem zrodlowym lokalnych modeli Meshy wybranych przez
wlasciciela projektu jest:

```text
C:\Projects\meshy2aurora\sample-3d\
└── <asset-id>\
    ├── manifest.yaml
    ├── source.glb       # lokalny, Git-ignored
    └── <wariant>.glb    # opcjonalny, lokalny, Git-ignored
```

`asset-id` identyfikuje probke wejsciowa i nie jest resrefem Aurory. Resrefy,
Appearance rows i nazwy kandydatow powstaja dopiero w materializacji albo
lineage proof.

Katalog `test-assets\meshy` jest wycofany i zabroniony. Kod produktu, testy,
narzedzia i nowe dokumenty nie moga go odtwarzac ani wskazywac.

## Klasy danych

| Katalog | Rola | Czy jest biblioteka zrodel Meshy |
|---|---|---|
| `sample-3d/<asset-id>` | lokalny model zrodlowy i jego jawne warianty | tak |
| `proof-output/<lineage>` | zamrozony kandydat i packet proof | nie |
| `artifacts/<deliverable>` | wygenerowany wynik lub materializacja | nie |
| `documentation/evidence` | append-only zapis wyniku i tozsamosci | nie |

Payloadow nie kopiujemy miedzy tymi klasami tylko po to, aby kod znalazl plik.
Nowy proof moze utrwalic dokladne wejscie w swoim lineage, ale nie zmienia to
kanonicznej lokalizacji probki.

## Manifest

Kazdy katalog probki musi miec sledzony `manifest.yaml`, nawet gdy lokalnego
GLB nie ma w danym checkoutcie. Manifest zapisuje co najmniej:

- `asset_id`;
- provider i lokalny charakter pliku;
- role i nazwy payloadow;
- rozmiar w bajtach oraz SHA-256 kazdego obecnego payloadu;
- stan kwalifikacji oraz uwagi o provenance.

GLB, GLTF, FBX i archiwa w `sample-3d` sa ignorowane przez Git. Nie wolno
uzywac `git add -f` do ich publikacji.

Amendment 2026-09-07: lokalne payloady obejmuja rowniez bufory BIN oraz
obrazy wejsciowe PNG/JPG/JPEG/WebP. Manifesty, provenance JSON i przepisy
materialow pozostaja sledzone. Piec wczesniej sledzonych obrazow usunieto
wylacznie z indeksu Git, bez przenoszenia plikow i bez zmiany bajtow;
ich SHA-256 potwierdzono z manifestami. Szczegoly:
[`audyt-higieny-git-2026-09-07.md`](audyt-higieny-git-2026-09-07.md).

## Przeplyw

1. Nowy model Meshy otrzymuje stabilny `asset-id`.
2. Powstaje `sample-3d/<asset-id>/manifest.yaml`.
3. Payload trafia do tego samego katalogu jako `source.glb`; warianty dostaja
   jawne role i osobne hashe w tym samym manifeście.
4. Kod i testy wskazuja sciezke pod `sample-3d`.
5. Po kazdej zmianie uruchamiany jest gate:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File assert-meshy-asset-layout.ps1
```

Nie tworzymy osobnego `incoming` lub `active` rootu. Stan kwalifikacji jest
metadana w manifeście i evidence, a nie powodem do zmiany kanonicznego korzenia.

## Przyczyna migracji 2026-07-27

Pierwotny `sample-foldery-cloud.md` opisywal `sample-3d`, lecz jego pierwszy
KROK 3 nigdy nie zostal wykonany. 17 lipca dodano druga aktywna polityke
`test-assets/meshy`, po czym testy zostaly podlaczone bezposrednio do nowego
katalogu. Nie wycofano starej koncepcji, nie wykonano migracji i nie dodano
automatycznego gate'u. W efekcie oba dokumenty wygladaly na obowiazujace, a
realne modele ominely pusty `sample-3d`.

Naprawa:

- `sample-3d` zostal jedynym rootem zrodel Meshy;
- lokalne payloady zostaly przeniesione bez zmiany bajtow;
- testy i narzedzia wskazuja nowy root;
- druga polityka i drugi katalog zostaly usuniete;
- root `AGENTS.md`, `PROJECT_RULES.md` oraz wykonywalny gate blokuja regresje.

Odwolania do konkretnych przeniesionych GLB w dokumentacji zostaly
znormalizowane do `sample-3d`. Historyczne evidence otrzymalo datowane
amendmenty potwierdzajace, ze zmienila sie tylko lokalizacja, a bajty i
SHA-256 pozostaly bez zmian. Oryginalne immutable packety proof zachowuja
autorytatywna sciezke z chwili capture.
