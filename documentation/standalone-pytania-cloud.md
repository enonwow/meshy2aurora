# standalone-pytania-cloud.md
Data: 2026-07-08 | Status: ZAMKNIĘTE (odpowiedź: standalone-odpowiedz-codex.md; NwnMdlComp zablokowany → własny parser binary wg dostarczonego layoutu; fixtures: syntetyczne w repo + assety gry przez env paths) | Priorytet: BLOKUJĄCE dla M1

## Q1: Droga do referencyjnego ASCII MDL bez aurora-web
c_kocrachn.mdl jest binary. Opcje: (a) lokalna binarka NwnMdlComp lub inny dekompilator MDL binary→ASCII (sprawdź, czy jest na dysku / w repach; `ekosystem-narzedzia-codex.md` miał NIE WIEM), (b) nwn-lib-d — czy umie binary MDL→ASCII i czy mamy go lokalnie zbudowanego, (c) własny parser binary. Zweryfikuj (a) i (b); jeśli oba niedostępne — odpowiedz na Q2.

## Q2: Pełny layout binarnego MDL (jeśli potrzebny własny parser)
Z dekompilacji + template xoreos (NWN1MDL.bt): pełna struktura binary MDL — header (offsety, rozmiary), geometry header, node header (typ, flags, dzieci), controllery (typy, klucze, dane), mesh header (verts/faces/tverts arrays), skin header (weights, boneconstantindices, qbone_ref_inv/tbone_ref_inv), animation header. Format ```yaml z offsetami i typami pól + kotwice. To duży temat — jeśli (a)/(b) z Q1 działa, wystarczy minimalny podzbiór do odczytu geometrii+szkieletu+wag.

Uwaga (D7 doprecyzowane): wolno posiłkować się logiką binarnego parsera z aurora-web (`aurora-mdl-ascii-to-glb.converter.ts`, sekcje binary ~544–1520) jako referencją — opisz layout na podstawie tego kodu + dekompilacji, a my napiszemy własną implementację.

## Q3: Runbook weryfikacji w NWN EE (bez aurora-web)
Krok po kroku: ścieżka instalacji NWN EE (Steam), gdzie wgrać hak (Documents\Neverwinter Nights\hak?), jak przygotować minimalny moduł testowy z creature m2a_koc01 (toolset: nowy moduł + hak w properties + paleta custom → utc), jak zrobić proof (screenshot z toolset preview / z gry, test animacji). Czy da się to częściowo zautomatyzować (nwmain/toolset flagi CLI)?

## Q4: Niezależny walidator plików
Czym walidować nasze MDL/2DA/HAK niezależnie od naszego kodu i od aurora-web: nwn-lib-d (jakie komendy?), inne narzędzia w ekosystemie (nwnexplorer? nwhak?)? Per narzędzie: dostępność lokalna, komenda, co waliduje.

## Q5: Licencja/pochodzenie referencji w fixtures
Czy możemy trzymać kopie c_kocrachn.mdl (CEP) i c_horror.mdl (retail NWN) w repo meshy2aurora jako fixtures do testów (repo prywatne?), czy testy mają wskazywać pliki poza repo przez konfigurację ścieżki?
