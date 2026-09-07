# Creature held item V8 — ready for owner proof — 2026-08-17

## Handoff

1. plik modułu: `m2aweapdemo8.mod`;
2. nazwa modułu w Toolsecie/NWN: `Meshy2Aurora procedural humanoid item placement`;
3. Area: `Meshy2Aurora procedural humanoid item placement area`;
4. HAK: `m2aweaphak8.hak`;
5. Creature: `Meshy procedural humanoid with held item`;
6. blueprint Creature: `m2awrhand8`;
7. modułowy przedmiot UTI: `m2aweapitem8`;
8. placement: `[10.0, 14.5, 0.0]`, kierunek `[0.0, -1.0]`.

## Decyzja właściciela i zakres

Po trzecim bezpośrednim poleceniu właściciela z 2026-08-17 przygotowano nowy
kandydat zamiast ponownie przekazywać historyczny V7. Polecenie zapisano jako
bezpośrednią zmianę wcześniejszej decyzji o oczekiwaniu na wynik V7.

V8 powstał z aktualnej produkcyjnej ścieżki
`CreatureHeldWeaponOptionsV1 -> build_procedural_creature_demo_with_held_weapon_v3`.
Delta obejmuje wyłącznie module-local item wyposażony w prawej ręce. Animacje
nie zostały zmienione.

## Exact wejścia i readback

- source GLB:
  `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`,
  SHA-256 `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`;
- base `appearance.2da`: `local-reference-assets/appearance.2da`, SHA-256
  `815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a`;
- wynik: 297 190 trójkątów, 42 animacje, Appearance row `15100`,
  `MODELTYPE=L`;
- hooki: `RightHand -> rhand`, `LeftHand -> lhand`, obecne w `42/42`
  drzewach animacji;
- UTI: `BaseItem=1`, `ModelPart1/2/3=61/11/11`;
- wyposażenie: dokładnie jeden fixture, `right_hand`, native slot `16`,
  `EquippedRes=m2aweapitem8` w GIT i UTC;
- semantic MOD readback: `PASS`, 7 zasobów.

## Exact artefakty i instalacja

- MOD:
  `proof-output/creature-held-item-v8/m2aweapdemo8.mod`, 16 247 bytes,
  SHA-256 `3e4c0a7fbae513fa6a7cb47ed07d377d071c3e9bcd5f8130aef6e7f2cbd4a065`;
- HAK:
  `proof-output/creature-held-item-v8/m2aweaphak8.hak`, 44 113 353 bytes,
  SHA-256 `b8a989fc22bef58fd6e49599de9d25d8b0d7bd4e232070f85e07d50663abc4e5`.

Przed instalacją oba cele nie istniały. Po kopii ponownie policzono SHA-256 i
potwierdzono byte-identical zgodność:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aweapdemo8.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aweaphak8.hak`.

Podczas instalacji istniał responsywny proces `nwmain`; agent go nie przejął,
nie sterował nim i nie uruchamiał sesji proofowej. Lista modułów może wymagać
powrotu do menu lub restartu gry.

## Status

- offline materialization: `verified`;
- MOD/HAK installation: `verified`;
- Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- agent-side boundary: `ready_for_owner_proof`.
