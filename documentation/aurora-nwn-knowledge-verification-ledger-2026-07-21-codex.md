# Rejestr weryfikacji wiedzy Aurora/NWN — 2026-07-21

Status: `ACTIVE / OFFLINE KNOWLEDGE LEDGER`

## 1. Cel i granica

Ten rejestr wiąże źródło z konkretnym faktem, skutkiem dla implementacji oraz
brakującym dowodem. Nie jest proofem Toolsetu ani NWN. W tej rundzie nie
uruchamiano i nie zmieniano Toolsetu, klienta NWN, modułu, HAK-a ani modelu.

Obowiązuje kolejność autorytetu z `PROJECT_RULES.md`:

1. dekompilacja Aurory;
2. lokalne zasoby retail/CEP i zapisane artefakty runtime, wyłącznie read-only;
3. niezależne parsery/specyfikacje przypięte do dokładnej rewizji;
4. dokumentacja authoringu jako kontekst, nie jako definicja binarnego ABI;
5. własny parser, semantic readback i testy jako dowód implementacji, ale nie
   jako oracle zamkniętego renderera NWN.

Każdy wniosek wizualny nadal wymaga świeżego, związanego z kandydatem proofu.

## 2. Przypięte źródła

| Źródło | Tożsamość | Zweryfikowany zakres | Granica |
| --- | --- | --- | --- |
| Aurora decomp | `C:\Projects\New Folder\export\decompiled_all.c`, SHA-256 `36bb8b1031afe2abf23f0e18180a5ad649401d9ea5e078e5170a31a96167c572` | `FUN_00a3b874`, `FUN_00a3ed90`, `FUN_00a3ee64`; dokładny lifecycle local-animation header | Dekompilacja nie nadaje wartości `5` nazwy gameplayowej i sama nie dowodzi widoczności modelu |
| xoreos NWN reader | commit `91d5b40a463a8627adccdfa991d61ca09f18d37d`, `src/graphics/aurora/model_nwn.cpp`, `Model_NWN::readAnimBinary()` | osobny `uint8_t type`, trzy bajty paddingu, potem length/transition/animroot | Niezależny reader GPL; reference-only, nie kod do kopiowania ani oracle Beamdog |
| xoreos binary template | commit `4e1c197aa09b532ef466ff8ceccfd6221e80c3c9`, `templates/NWN1MDL.bt` | cross-check layoutu binary MDL | Specyfikacja pomocnicza; konflikt rozstrzyga Aurora/retail/proof |
| CEP corpus | `C:\Users\enonw\Documents\Neverwinter Nights\hak\cep3_core1.hak`, SHA-256 `6a8e6a64773a77fd46740cbcce19a708db6a70b4975732d0405978f3fbe8eb1a` | read-only histogram local-animation headers opisany w syntezie: `type=5` dla `24 707` osiągalnych nagłówków | Corpus nie jest fixture produktu i nie wolno kopiować payloadów |
| Własna implementacja | `crates/m2a-core/src/mdl/{write_binary_mdl.rs,parse_binary_mdl.rs,semantic_readback.rs}` oraz testy `mdl_writer`/`model_pipeline` | writer emituje `animation_type=5`; parser i semantic readback rozdzielają type od paddingu | Zielony readback dowodzi własnej spójności, nie skutku renderowego NWN |

Przypięte linki upstream:

- xoreos `model_nwn.cpp`: <https://github.com/xoreos/xoreos/blob/91d5b40a463a8627adccdfa991d61ca09f18d37d/src/graphics/aurora/model_nwn.cpp>
- xoreos-docs `NWN1MDL.bt`: <https://github.com/xoreos/xoreos-docs/blob/4e1c197aa09b532ef466ff8ceccfd6221e80c3c9/templates/NWN1MDL.bt>

## 3. Zweryfikowany fakt: `animation_type` jest pojedynczym bajtem

Dekompilacja przypięta powyższym hashem pokazuje:

- `FUN_00a3b874` tworzy obiekt local state, wywołuje `FUN_00a3ee64`, kopiuje
  events i materializuje state tree;
- `FUN_00a3ee64` kopiuje `+0x70`, `+0x74`, `+0x78..+0xb7`, po czym wywołuje
  `FUN_00a3ed90`;
- `FUN_00a3ed90` kopiuje nazwę, root pointer i dokładnie jeden bajt
  `*(undefined1 *)(... + 0x6c)`; nie kopiuje trzech kolejnych bajtów.

Przypięty xoreos commit niezależnie czyta w `readAnimBinary()` jeden bajt
`type`, omija trzy bajty paddingu i dopiero potem czyta długość, transition i
`animroot`.

Wniosek implementacyjny: reprezentacja `animation_type: u8` plus
`animation_type_padding: [u8; 3]` jest właściwą granicą readbacku. Emitowanie
`5,0,0,0` jest poparte dekompilacją, corpushowym precedensem i niezależnym
readerem. Nie wolno z tego wyprowadzać twierdzenia, że `type=5` samodzielnie
naprawia widoczność w NWN.

## 4. Tożsamość bieżącego kandydata r27

Read-only hash check rozróżnił dwa różne MOD-y:

| Artefakt | SHA-256 | Znaczenie |
| --- | --- | --- |
| `proof-output/m0-r27-animation-type5-20260721/generated/m2a_codex_aproof.mod` | `2b1b80b8c8b3f33ac5cdd4e4500d61b547072c5a481205cf5635b940b7961ac7` | wygenerowany MOD w katalogu materializacji; nie jest to moduł związany z live proofem r27 |
| `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0r25.mod` | `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257` | zapisany moduł live z Area `m2a_m0a25`, fixture `(10,14.5,0)` i ordered HAK listą r27/r26/r21 |
| `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r27.hak` | `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd` | bieżący HAK z `animation_type=5` |
| `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r26.hak` | `6f805873420b5279b45dadc26d913fa7692374df9e0582c23b42c8a4b42d6bb0` | poprzedni HAK z identity-controller delta |
| `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r21.hak` | `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6` | wcześniejszy HAK widoczny w Toolsecie |

`native-geometry-gate.json` wiąże live kandydata z modułem `4a4f...`, nie z
wygenerowanym MOD-em `2b1b...`. Raporty i przyszłe proofy muszą używać tej
tożsamości, aby nie pomieszać przygotowanego outputu z zapisanym modułem.

Aktualny stan pozostaje:

- Toolset r27: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN r27: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- r28 ani kolejna kopia MDL/HAK/MOD nie są dopuszczone.

## 5. Następny dowód zamykający

Jedyny rozstrzygający kolejny krok dla tej hipotezy to świeży proof dokładnego
r27, najpierw w Toolsecie, a po widoczności tego samego lineage w NWN. Packet
musi wiązać co najmniej:

- deployed MOD `4a4f...`;
- ordered HAK listę `[r27,r26,r21]` i HAK r27 `8714...`;
- Area `m2a_m0a25`, fixture `m0_fixture`, Appearance row `848` i pozycję
  `(10,14.5,0)`;
- MDL/TGA/`appearance.2da` z HAK-a r27;
- świeży validated `TScrollBox` dla Toolsetu oraz osobny packet NWN.

Brak takiego capture pozostaje `missing`, nie jest negatywnym wynikiem modelu
i nie dopuszcza nowej iteracji.

## 6. Weryfikacja własnej implementacji

Po zapisaniu rejestru uruchomiono trzy wąskie testy na aktualnym worktree:

```text
cargo test -p m2a-core --test mdl_writer owned_cpause1_roundtrips_exact_animation_layout_events_and_linear_keys -- --exact
cargo test -p m2a-core --test mdl_writer animation_reader_rejects_named_pointer_array_and_controller_mutations -- --exact
cargo test -p m2a-core --test model_pipeline static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice -- --exact
```

Wynik: `3 passed, 0 failed`. Zakres dowodu:

- writer emituje local-animation header z `animation_type=5` i zerowym
  paddingiem;
- parser zachowuje niezależną obserwowalność mutacji type oraz paddingu;
- własny M0 przechodzi self-contained vertical-slice readback.

Status tej poprawki: `verified` wyłącznie dla source/readback. Status live r27
pozostaje bez zmian: `modelVisibility=not_tested`,
`proofCompleteness=missing` osobno dla Toolsetu i NWN.
