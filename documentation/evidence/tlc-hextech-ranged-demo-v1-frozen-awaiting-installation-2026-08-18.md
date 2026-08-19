# Hextech ranged demo V1 — zamrożony kandydat oczekujący na instalację

**Test MOD:** `m2aimod35.mod`  
**Nazwa modułu w Toolset:** `Meshy2Aurora Item 113 candidate`  
**Area:** `Meshy2Aurora Item Assembly Proof`

## Stan

Kandydat jest zamrożony i przeszedł pełny offline readback, ale nie został zainstalowany w natywnych katalogach NWN. Dlatego stan pozostaje `not_ready_for_owner_proof`, `modelVisibility=not_tested`, `proofCompleteness=missing`. Agent nie uruchamiał Aurora Toolset ani NWN.

## Dokładna linia artefaktów

- MOD: `m2aimod35.mod`, SHA-256 `3b0305aaf1997a72b1efdaa58fdf86af486e438cc9709e24355c2d9425560ef2`, 18 659 bajtów.
- HAK: `m2aihak35.hak`, SHA-256 `dd4f269cfea4867490ca19c49b23f48c7632c4d0686f39db0ddc3d176c426d55`, 185 253 306 bajtów.
- UTI broni: `m2aitm35.uti`, SHA-256 `475a4b8e065b205c57c03cb225332c5151d4b0f3c5839257ab2ec9706240ba90`.
- UTI amunicji: `m2ahxammo.uti`, SHA-256 `be36ad6cb9f6bdff2df92675be83e138ebb68373b916282091aed04f104ceb9b`.
- Raport buildu: `item-build-report.json`, SHA-256 `21f82669fc69198cbd4a05578045ce91816d24f3b85d8143ceda445a5100cf80`.

## Zawartość demo potwierdzona odczytem MOD

- wejście gracza: `[10, 10, 0]`, kierunek `[0, 1]`;
- pickup broni `m2aitm35`: `[10, 14.5, 0]`;
- pickup amunicji `m2ahxammo`: `[11.25, 14.5, 0]`;
- nieruchomy cel `m2arngtarget`: `[10, 18, 0]`, `Appearance_Type=102`, `WalkRate=0`, `PerceptionRange=0`, wszystkie skrypty AI puste;
- `groundItemCount=2`, `creatureCount=1`, semantic readback `PASS`.

## Trasa broni i pocisku

`BaseItem 113 -> Bullet BaseItem 27 -> AmmunitionType 3 -> DamageRangedProjectile 6 -> ammunitiontypes row 38 -> m2ahxshell -> xbowshot`

Model pocisku ma 1536 trójkątów, używa źródła SHA-256 `f4f8c41d445036b1364a16f779b17b21baa4f49caff356d77f6d8b8e1d9727ec`, skali `0.2` i ręcznej korekty rotacji `[0, 0, 90]`, która przeszła walidację osi Aurora `+Y`.

## Oczekiwany test właścicielski po instalacji

1. Uruchomić dokładnie `m2aimod35.mod` z HAK `m2aihak35.hak`.
2. Podnieść broń i stos amunicji leżące przed punktem startowym.
3. Wyposażyć broń, załadować amunicję i zaatakować nieruchomy cel stojący dalej na osi `+Y`.
4. Potwierdzić zużycie amunicji, clip `xbowshot`, widoczny lot pocisku przodem, skalę, dźwięki i trafienie.

Nowa iteracja modelu jest niedopuszczalna bez świeżego, związanego z tym dokładnym kandydatem wyniku wizualnego `modelVisibility=not_visible`.
