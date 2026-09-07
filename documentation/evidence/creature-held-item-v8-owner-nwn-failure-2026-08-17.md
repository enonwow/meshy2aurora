# Creature held item V8 — wynik właścicielski NWN — 2026-08-17

Status: `FAILED / V9_ADMITTED`.

Dokładny kandydat: `m2aweapdemo8.mod`, moduł
`Meshy2Aurora procedural humanoid item placement`, Area
`Meshy2Aurora procedural humanoid item placement area`.

## Wynik

Właściciel przekazał świeży kadr NWN związany z V8. Creature jest wyraźnie
widoczny, lecz obie dłonie są puste. Ustawienie postaci nie nadaje się też na
neutralny demonstrator trzymania itemu: V8 ponownie użył Fogbounda, który był
wcześniej podmiotem macierzy eksperymentów kierunku przodu.

- NWN Creature: `modelVisibility=visible`;
- NWN item w ręce: `heldItemVisibility=not_visible`;
- NWN facing demonstratora: `failed`;
- NWN: `proofCompleteness=verified` dla powyższych obserwacji;
- Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`.

Kadr zachowano bez zmiany jako
`proof-output/creature-held-item-v8-owner-result-2026-08-17/owner-nwn-no-weapon.png`,
574 091 bytes, SHA-256
`b4b56299e04cb171df0b6c3d011b17c570764362eaa1fc58e6d3be23383431c5`.

## Wiążąca tożsamość V8

- MOD SHA-256:
  `3e4c0a7fbae513fa6a7cb47ed07d377d071c3e9bcd5f8130aef6e7f2cbd4a065`;
- HAK SHA-256:
  `b8a989fc22bef58fd6e49599de9d25d8b0d7bd4e232070f85e07d50663abc4e5`;
- MDL SHA-256:
  `2cafad006d74460d57251c3ccefadcc3283bdc26558f06d2e59b22d06ee73d40`;
- Creature: `m2awrhand8`;
- modułowy UTI: `m2aweapitem8`;
- źródło Fogbound SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`.

## Diagnoza

Fakt z własnego readbacku: MOD zawierał UTI, a oba `Equip_ItemList` w GIT i
UTC wskazywały jego resref. Fakt z renderera NWN: przedmiot nie został
narysowany. Sam parser-visible UTI i zgodne referencje GFF nie są więc proofem
runtime resolution. Przejście produktowe na modułowy UTI było regresją wobec
V5/V6, gdzie stockowy `nw_wswss001` był widoczny przy prawej dłoni.

## Minimalna delta V9

1. Użyć innego humanoida, którego poprawny zwykły wariant był już potwierdzony
   przez właściciela, zamiast podmiotu facing-matrix.
2. Wyposażyć dokładny stockowy `nw_wswss001`; MOD nie udaje, że własny UTI jest
   runtime-resolved.
3. Zastosować przygotowany lokalny roll prawej ręki `+90°`, aby stockowe
   ostrze nie było pokazane krawędzią.
4. Nie zmieniać animacji.

Świeży, wiążący funkcjonalny wynik NWN dopuszcza V9. Pełny zapis maszynowy:
`proof-output/creature-held-item-v8-owner-result-2026-08-17/owner-result.json`.
