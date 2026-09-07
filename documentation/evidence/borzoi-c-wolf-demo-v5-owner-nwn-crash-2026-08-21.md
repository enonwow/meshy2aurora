# Borzoi c_wolf demo V5 — wynik właściciela: crash NWN

Data: 2026-08-21\
Status: `OWNER_NWN_RUNTIME_CRASH_RECORDED / ITERATION_GATE_CLOSED`

## Tożsamość kandydata

- MOD: `m2aborzmod5.mod`, SHA-256
  `01c346d47af823bf429c6fdc8c3181aab9f5109858ee56f6ff2ad690e9a49c47`;
- nazwa modułu: `Meshy2Aurora Borzoi c_wolf Demo V5`;
- Area: `Meshy2Aurora Borzoi Test Area V5`;
- HAK: `m2aborzhak5.hak`, SHA-256
  `da24c46d22d45b14f798a32bde9b44f1464c747be9e73d29e0f2647ffacce4ad`;
- MDL: `m2aborzcre5.mdl`, SHA-256
  `1f4fe421779cf472e16336588084d92e21d0174c2b9091ef326f9373bb6d4adc`;
- Appearance row: `848`.

## Objaw i dowody

Właściciel uruchomił V5 w NWN:EE 8193.37.17. Podczas ładowania Area klient
zgłosił fatalny błąd i zakończył działanie.

- screenshot właściciela:
  `C:\Users\enonw\AppData\Local\Temp\codex-clipboard-3e9d2cf4-0d42-4df5-8d3b-568a7c536bb1.png`;
  340669 bajtów; SHA-256
  `d9e15f06bbdab4dfeefee2a86b807f54f1e8377752c1c38791d323e79c794437`;
- crash report:
  `C:\Users\enonw\Documents\Neverwinter Nights\crashreport\nwmain-crash-1787307083.nwcrash.txt`;
  655176 bajtów; SHA-256
  `974b04bf5b2b2f3b174699c4e3d67ffb44fbd99bd93de621d28f3d3453f43690`;
- crash report potwierdza `CURRENTGAME:m2aborzmod5` oraz
  `HAK:m2aborzhak5`; nie jest to brak ani pomylenie artefaktu;
- wyjątek: `0xc0000005`; renderer: NVIDIA GeForce RTX 5070; sterownik
  `32.0.16.1062`; minidump prowadzi przez stos OpenGL/NVIDIA, a finalny adres
  wyjątku leży w pamięci kodu JIT sterownika;
- crash nastąpił około cztery sekundy po rozpoczęciu ładowania dokładnego
  modułu V5.

## Klasyfikacja wyniku

NWN:

- `modelVisibility = not_tested`;
- `proofCompleteness = failed`;
- `qualityVerdict = not_tested`.

Crash podczas inicjalizacji renderowania nie jest obserwacją
`modelVisibility=not_visible`. Nie otwiera więc samodzielnie bramki nowej
iteracji modelu.

## Audyt strukturalny po crashu

Fakty:

1. Własny parser odczytuje cały MDL, 14 SkinMeshy, 44 nody, tangenty, wagi,
   materiał i animacje bez błędu.
2. Niezależny CleanModels EE `v4.0.0-rc6`, WASM SHA-256
   `786b78932f0beca95295bd7a7a608ac82cbccf8234e3f88cdec215987b74dacb`,
   dekompiluje V5 do ASCII bez ostrzeżeń:
   44 nody, 14 SkinMeshy, 14 bloków tangentów i 14 renderhintów
   `NormalAndSpecMapped`.
3. CleanModels ponownie kompiluje i dekompiluje wynik w pamięci bez błędu,
   zachowując te same liczby nodów, SkinMeshy, tangentów i renderhintów.
4. Lokalna dekompilacja Aurory potwierdza pola raw MDL:
   tangent XYZ pod offsetem common-mesh `0x258` i handedness pod `0x260`.
   Writer V5 używa właśnie tych pól. Pierwsza hipoteza o błędnym offsecie
   tangentów została odrzucona.
5. V3 i V5 mają po 44 nody i 14 SkinMeshy. V3 nie ma tangentów ani profilu
   `NormalAndSpecMapped`; V5 ma je na wszystkich 14 SkinMeshach. V5 zmienia
   równocześnie profil deformacji i ścieżkę materiałową, więc sam crash nie
   izoluje jeszcze jednej z tych delt.

Hipoteza robocza:

- najbardziej podejrzane jest połączenie wysokowierzchołkowego SkinMesha,
  ścieżki `NormalAndSpecMapped`, tangentów oraz sterownika NVIDIA OpenGL;
- sam format kontenera MDL, HAK attachment i lookup zasobów nie są obecnie
  kandydatami pierwszego rzędu.

## Decyzja i następny krok

Status `ready_for_owner_proof` V5 zostaje wycofany. V5 pozostaje zamrożony;
nie wolno go nadpisywać, przepakowywać ani po cichu zastępować V6.

Następny bezpieczny krok dotyczy tego samego kandydata: zebrać od właściciela
wynik widoczności dokładnego obiektu V5 w Toolsecie oraz, jeśli Toolset go
renderuje, zawęzić awarię NWN do ścieżki runtime shader/deformation. Dopiero
jawna decyzja właściciela zmieniająca bramkę może dopuścić izolacyjną iterację
z jedną deltą naraz.
