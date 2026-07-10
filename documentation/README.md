# Documentation

Ten folder jest jedynym miejscem dokumentacji projektu `meshy2aurora`.

Przed uzyciem starszego dokumentu sprawdz jego klase w [status-dokumentacji-web-2026-07-10-codex.md](status-dokumentacji-web-2026-07-10-codex.md). D11-D14 i aktywne dokumenty webowe maja pierwszenstwo przed historycznymi rekomendacjami CLI/Node/aurora-web.

## Aktualny kierunek

Aktualny kierunek po audycie 2026-07-09:

- `meshy2aurora` jest projektem standalone: Meshy -> natywny content Aurora/NWN (`binary MDL`, polityka `MDX`, `2DA`, `HAK`).
- `C:\Projects\aurora-web` jest tylko read-only reference. Nie jest dependency, CLI, oracle, walidatorem ani proof base dla `meshy2aurora`.
- Twardy proof podstawowy ma isc przez NWN EE Toolset/gra oraz wlasny wygenerowany HAK/modul testowy.
- `c_kocrachn` jest technicznym proxy dla creature pipeline, nie assetem The Last City.
- Produkt jest aplikacja webowa local-first: UI w przegladarce, Rust 1.96.1 skompilowany do WebAssembly oraz pobieranie wygenerowanych HAK/raportow jako plikow. Studio bedzie osobnym etapem po proofie M6.

## Dokumenty

- [PROJECT_RULES.md](PROJECT_RULES.md) - zasady projektu i implementacji.
- [audyt-gotowosci-startowej-2026-07-10-codex.md](audyt-gotowosci-startowej-2026-07-10-codex.md) - kanoniczny gate przed implementacja: stan repo, toolchain, bootstrap, CI, M1A DoD i otwarte decyzje.
- [macierz-gotowosci-wiedzy-codex.md](macierz-gotowosci-wiedzy-codex.md) - centralny stan wiedzy dla calego pipeline; oddziela ustalony kierunek, otwarte evidence i runtime proof.
- [mdl-binary-crosswalk-codex.md](mdl-binary-crosswalk-codex.md) - wspolny layout binary MDL, zakres profilu A i jawny konflikt wariantow skin header.
- [mdx-polityka-codex.md](mdx-polityka-codex.md) - aktywna polityka appended volatile/MDX dla profilu A.
- [animacje-kontrakt-profil-a-codex.md](animacje-kontrakt-profil-a-codex.md) - self-contained kierunek animacji oraz korekta faktow o `c_kocrachn`/`c_Horror`.
- [hak-2da-gff-crosswalk-codex.md](hak-2da-gff-crosswalk-codex.md) - kontrakt writerow HAK/ERF, 2DA i GFF oraz generated module proof.
- [reguly-dokumentacji-cloud.md](reguly-dokumentacji-cloud.md) - aktualne reguly wymiany plikow Cloud/Codex.
- [audyt-dokumentacji-plan-2026-07-09-codex.md](audyt-dokumentacji-plan-2026-07-09-codex.md) - aktualna mapa rozjazdow, luk i plan naprawczy.
- [architektura-meshy2aurora-codex.md](architektura-meshy2aurora-codex.md) - architektura standalone `meshy2aurora`.
- [architektura-web-wasm-codex.md](architektura-web-wasm-codex.md) - architektura webowa: Rust/WASM, React, Three.js, lokalne pliki i granice opcjonalnego backendu.
- [engine-mdl-odpowiedz-codex.md](engine-mdl-odpowiedz-codex.md) - aktualny stan odpowiedzi o binary writerze, polityce MDX i bind pose.
- [status-dokumentacji-web-2026-07-10-codex.md](status-dokumentacji-web-2026-07-10-codex.md) - klasyfikacja wszystkich dokumentow po decyzji D12.
- [plan-implementacji-orkiestrator-codex.md](plan-implementacji-orkiestrator-codex.md) - aktywny plan etapow, Definition of Done i kontrakt dla orkiestratora.
- [orchestrator-state.yaml](orchestrator-state.yaml) - maszynowy aktualny stan etapow, blockerow, problemow, bledow i evidence.
- [evidence/README.md](evidence/README.md) - append-only szablon dowodow dla jednego etapu.
- [prompt-dla-claude-prototyp-parsera.md](prompt-dla-claude-prototyp-parsera.md) - gotowy prompt dla Claude: prototype M1a parsera binary MDL.
- [przyszle-featurey-studio-codex.md](przyszle-featurey-studio-codex.md) - backlog Studio po MVP: diagnostyka, materialy, geometria, rig, animacje i proof packet.
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
