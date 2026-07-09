# Documentation

Ten folder jest jedynym miejscem dokumentacji projektu `meshy2aurora`.

## Aktualny kierunek

Aktualny kierunek po audycie 2026-07-09:

- `meshy2aurora` jest projektem standalone: Meshy -> natywny content Aurora/NWN (`binary MDL`, polityka `MDX`, `2DA`, `HAK`).
- `C:\Projects\aurora-web` jest tylko read-only reference. Nie jest dependency, CLI, oracle, walidatorem ani proof base dla `meshy2aurora`.
- Twardy proof podstawowy ma isc przez NWN EE Toolset/gra oraz wlasny wygenerowany HAK/modul testowy.
- `c_kocrachn` jest technicznym proxy dla creature pipeline, nie assetem The Last City.

## Dokumenty

- [PROJECT_RULES.md](PROJECT_RULES.md) - zasady projektu i implementacji.
- [reguly-dokumentacji-cloud.md](reguly-dokumentacji-cloud.md) - aktualne reguly wymiany plikow Cloud/Codex.
- [audyt-dokumentacji-plan-2026-07-09-codex.md](audyt-dokumentacji-plan-2026-07-09-codex.md) - aktualna mapa rozjazdow, luk i plan naprawczy.
- [architektura-meshy2aurora-codex.md](architektura-meshy2aurora-codex.md) - architektura standalone `meshy2aurora`.
- [neverblender-audyt-2026-07-09-codex.md](neverblender-audyt-2026-07-09-codex.md) - audyt NeverBlender jako narzedzia pomocniczego/debug dla modeli NWN/Aurora.
- [repozytoria-pomocnicze-2026-07-09-codex.md](repozytoria-pomocnicze-2026-07-09-codex.md) - mapa repozytoriow drugiej linii po Aurora First.
- [audyt-repozytoriow-pomocniczych-2026-07-10-codex.md](audyt-repozytoriow-pomocniczych-2026-07-10-codex.md) - lokalny audyt 27 repozytoriow z `C:\Projects\Claude`, z priorytetami, kluczowymi plikami i mapowaniem na pipeline.
- [wymagania-startowe-cloud.md](wymagania-startowe-cloud.md) - historyczne wymagania startowe sprzed decyzji D7; uzywac tylko jako kontekst.
- [CLOUD_SUPPLEMENT_FORMAT.md](CLOUD_SUPPLEMENT_FORMAT.md) - format odpowiedzi i suplementow dla plikow `*-cloud.md`.
- [aurora-models-animations-audit-2026-07-08.md](aurora-models-animations-audit-2026-07-08.md) - audyt modeli, animacji i systemow powiazanych z Aurora/NWN.
- [aurora-pipeline-odpowiedz-codex.md](aurora-pipeline-odpowiedz-codex.md) - odpowiedzi Codexa dla starego tematu pipeline `aurora-web`; reference-only.
- [aurora-animacje-odpowiedz-codex.md](aurora-animacje-odpowiedz-codex.md) - odpowiedzi Codexa dla tematu animacji.
- [meshy-input-odpowiedz-codex.md](meshy-input-odpowiedz-codex.md) - odpowiedzi Codexa dla tematu wejscia Meshy.
- [pliki-referencyjne-odpowiedz-codex.md](pliki-referencyjne-odpowiedz-codex.md) - odpowiedzi Codexa dla tematu plikow referencyjnych; sciezki `aurora-web` sa reference-only.
- [srodowisko-zakres-odpowiedz-codex.md](srodowisko-zakres-odpowiedz-codex.md) - odpowiedzi Codexa dla tematu srodowiska i zakresu.

## Zasada dopisywania

Nowe notatki, audyty, decyzje techniczne, wyniki proofow, listy zrodel i runbooki dopisujemy tutaj. Jezeli w innym repo istnieje dokument potrzebny temu projektowi, w `documentation` powinna powstac notatka indeksujaca albo przeniesiony/odswiezony dokument, z jasnym wskazaniem zrodla.

## Konwencja `*-cloud.md`

Pliki w formacie `[nazwa]-cloud.md` oznaczaja suplement cloud do dokumentu bazowego. Gdy taki plik istnieje albo zostanie wskazany, trzeba dopisac uzupelnienie dla pracy w chmurze, a nie traktowac go jako osobny zamiennik glownej dokumentacji.

Pelny uklad takiego suplementu opisuje [CLOUD_SUPPLEMENT_FORMAT.md](CLOUD_SUPPLEMENT_FORMAT.md).

## Konwencja Cloud/Codex

Aktualna wymiana z Cloud:

- `[temat]-cloud.md` - dokument Cloud.
- `[temat]-pytania-cloud.md` - pytania Cloud z sekcjami `Q1`, `Q2`, ...
- `[temat]-odpowiedz-codex.md` - odpowiedzi Codexa, z odpowiedzia pod kazdym `Q`.
