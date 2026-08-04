# m2atgls8.mod

Toolset module name: `Meshy2Aurora TLC Guard Longsword reference profile v8`
Area: `TLC Guard Longsword Reference Profile V8`

Status: `owner_rejected`
Candidate: `tlc-guard-longsword-item-reference-profile-v8-20260804`

Owner result: `documentation/evidence/tlc-guard-longsword-v8-owner-reference-profile-result-2026-08-04.json`

## Dokładna tożsamość

- MOD: `m2atgls8.mod`
  - SHA-256: `c84acd3113e5ebeb91181b3d61335ab0de5492d43ae223a62a9fbbe09f63a4dd`
  - installed: `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2atgls8.mod`
- ordered HAK: `m2atglh8.hak`
  - SHA-256: `1791986315b4ccc4a443eea4473007bab1c14ac176c28daa2c5d76a264076778`
  - installed: `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2atglh8.hak`
- UTI/resref: `m2atglu8`
- item name: `Last City Bronze Guard Longsword`
- BaseItem: `1` (`WSwLs`, ModelType 2)
- selected values: Bottom `253`, Middle `253`, Top `253`;
- identified: `true`;
- attachment profile SHA-256:
  `877dc68ab31c14ca1d87684d86442fa9c7cef0356d6d2e426e769e2e3872a947`;
- reference fit SHA-256:
  `ac087a33ebc4949e1c19f7db20eb801f72ee5edc85d0efcd58685038fe23d3f4`.

Źródłowe i zainstalowane pliki MOD/HAK zostały ponownie zahashowane i są
byte-identical. Referencyjne payloady Vanilla nie zostały skopiowane do HAK-a
ani packetu; raport zawiera wyłącznie ich locatory, hashe i profil liczbowy.

## Co sprawdzić

1. Otworzyć dokładnie moduł `m2atgls8.mod` i Area podaną wyżej.
2. W drzewie Items otworzyć właściwości `Last City Bronze Guard Longsword`.
   Model ma być skierowany przodem, kompletny i złożony Bottom/Middle/Top bez
   rozłączenia lub przesunięcia rękojeści.
3. Porównać jego origin i układ z natywnym długim mieczem. Ostrze może mieć
   odmienną sylwetkę, lecz gardę/chwyt i wewnętrzne złącza muszą pozostawać w
   ramie referencyjnej.
4. Sprawdzić ikonę oraz ground Item. Ground Item jest prawdziwym itemem, nie
   creature.
5. W NWN sprawdzić osobnego equipped-render witness. Creature służy wyłącznie
   jako nośnik tego samego exact UTI do oceny chwytu; nie zastępuje itemu.

## Stan dowodu

- `modelVisibility=not_tested`
- `proofCompleteness=missing`
- `visualAcceptance=not_tested`

Te pola może zamknąć wyłącznie wynik wizualny przekazany przez właściciela.
Agent nie uruchamiał Toolsetu ani NWN.

## Wynik właściciela 2026-08-04

- `modelVisibility=visible` dla właściwego Itemu w Item Properties;
- `proofCompleteness=verified`;
- `visualAcceptance=failed`;
- selektory Model są puste, ponieważ V8 używa modelu `25` / bucketu `250`,
  podczas gdy retail `WSwLs` enumeruje buckety `10..100` (modele `1..10`);
- `Middle` przekracza referencyjną obwiednię poprzeczną około `2.00x` w osi X
  i `2.34x` w osi Z;
- creature/equipped-render witness jest nieważny i zostaje usunięty z zakresu
  następnego demo zgodnie z decyzją właściciela.

Packet V8 pozostaje immutable; nie wolno go przepakowywać ani podmieniać.
