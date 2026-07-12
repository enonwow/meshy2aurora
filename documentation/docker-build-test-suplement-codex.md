# Docker build/test supplement

Data: 2026-07-12 | Autor: Codex | Status: AKTYWNY SUPLEMENT IMPLEMENTACYJNY (DOCS-ONLY)

## 1. Decyzja

```yaml
status: POTWIERDZONE
product_runtime_now: "browser local-first + Rust/WASM; no backend"
docker_now: "optional reproducible build/test environment"
docker_not_now:
  - "product runtime"
  - "NWN EE Toolset/game runtime"
  - "replacement for Windows host proof"
static_web_image: "deferred to S1, after M6 DONE"
initial_web_delivery: "static files/GitHub Pages remains the active architecture"
```

Docker nie zmienia architektury produktu. Konwersja nadal wykonuje sie lokalnie w
przegladarce na plikach jawnie wybranych przez uzytkownika, a `m2a-core` pozostaje
jedynym rdzeniem formatu. Na obecnym etapie obraz ma jedynie odtwarzac toolchain i
uruchamiac te same gates, ktore istnieja na hoscie i w CI.

Nie dodajemy jeszcze obrazu serwujacego aplikacje. `apps/studio-web`, React/Vite,
Worker, viewport oraz statyczny obraz webowy naleza do S1, ktore zalezy od
`M6 DONE`. Docker nie moze przyspieszyc S1 przez stworzenie alternatywnego UI lub
pipeline'u.

## 2. Podstawa audytu

Audyt repozytorium z 2026-07-12 potwierdzil:

- `rust-toolchain.toml` przypina Rust `1.96.1`, `rustfmt`, `clippy` i target
  `wasm32-unknown-unknown`;
- lokalny toolchain: Rust/Cargo `1.96.1`, Node `24.15.0`, npm `11.12.0` oraz
  `wasm-pack 0.15.0`;
- `.github/workflows/ci.yml` uruchamia native quality na Windows i Linux oraz
  osobny gate WASM/Node na Linux;
- aktywny produkt nie ma backendu i nie ma jeszcze `package.json`; pin Node i
  package managera dla React/Vite jest gate'em `P-NODE` etapu S1;
- finalny proof pozostaje manualnym proofem w NWN EE Toolset/grze na hoscie
  Windows.

Zrodla kontraktu w repo:

- `documentation/PROJECT_RULES.md`;
- `documentation/architektura-web-wasm-codex.md`;
- `documentation/architektura-meshy2aurora-codex.md`;
- `documentation/audyt-gotowosci-startowej-2026-07-10-codex.md`;
- `documentation/plan-implementacji-orkiestrator-codex.md`;
- `.github/workflows/ci.yml`;
- `rust-toolchain.toml`.

## 3. Przypiety toolchain

Minimalny kontrakt obrazu build/test:

```yaml
rust: "1.96.1"
rust_profile: "minimal"
rust_components: ["rustfmt", "clippy"]
rust_targets: ["wasm32-unknown-unknown"]
node_major: 24
node_audited_patch: "24.15.0"
wasm_pack: "0.15.0 installed with cargo install --locked"
cargo_dependencies: "Cargo.lock with --locked gates"
host_targets: ["Linux container", "Windows CI/host remains mandatory"]
```

Implementacja Dockerfile ma przypiac obrazy bazowe do konkretnego patcha i
digestu SHA256. Sam tag `latest`, `node:24` lub `rust:bookworm` nie jest
powtarzalnym pinem. Digesty musza zostac pobrane i zapisane w momencie dodania
Dockerfile; ten dokument nie zgaduje wartosci digestow.

Node 24 jest potrzebny teraz do `wasm-pack test --node`. Repozytoryjny pin Node,
package managera i lockfile aplikacji webowej powstaje dopiero razem z
`apps/studio-web` w S1. Do tego czasu Docker nie jest zrodlem decyzji o stacku UI.

## 4. Rekomendowane etapy przyszlego Dockerfile

Pierwszy Dockerfile ma byc build/test-only i miec jawne, male etapy:

```yaml
stages:
  node_toolchain:
    base: "exact Node 24.15.0 Debian image pinned by digest"
    output: "Node binary required by wasm-pack tests"
  rust_toolchain:
    base: "exact Rust 1.96.1 Debian image pinned by digest"
    installs:
      - "wasm32-unknown-unknown"
      - "wasm-pack 0.15.0 via cargo install --version 0.15.0 --locked"
    verifies:
      - "rustc --version"
      - "cargo --version"
      - "node --version"
      - "wasm-pack --version"
  quality:
    copies: "only workspace manifests, lockfile and required crate sources/tests"
    runs:
      - "cargo fmt --all -- --check"
      - "cargo clippy --locked --workspace --all-targets -- -D warnings"
      - "cargo test --locked --workspace"
      - "cargo build --locked -p m2a-wasm --target wasm32-unknown-unknown"
      - "wasm-pack test --node crates/m2a-wasm"
```

Etap `quality` moze byc finalnym targetem build/test. Nie potrzebuje portu,
ENTRYPOINT-u, serwera ani stale dzialajacego kontenera. Sukces `docker build
--target quality` jest wynikiem gate'u, a nie proofem zgodnosci z Aurora/NWN.

Po rozpoczeciu S1 mozna dodac osobny lancuch:

```yaml
S1_stages:
  web_deps: "Node 24 + repozytoryjny packageManager + frozen lockfile"
  wasm_release: "release build publicznego adaptera WASM"
  studio_build: "Vite static bundle consuming the same WASM pipeline"
  static_runtime: "minimal non-root static file server, optional deployment image"
```

`static_runtime` serwuje wylacznie pliki statyczne. Nie przyjmuje uploadu do
backendu, nie wykonuje konwersji po stronie serwera i nie przechowuje assetow.
Preferowanym pierwszym sposobem publikacji pozostaje eksport statycznego katalogu
do GitHub Pages; obraz runtime jest opcjonalnym artefaktem wdrozeniowym S1.

## 5. `.dockerignore` i granica kontekstu

Przy implementacji nalezy dodac `.dockerignore` przed pierwszym buildem. Minimalna
polityka:

```text
.git/
.github/                    # wlaczyc tylko jesli konkretny gate jej wymaga
.codex-tmp/
target/
**/pkg/
node_modules/
proof-output/
local-reference-assets/
*.hak
*.mod
*.key
*.bif
*.mdl
*.mdx
*.tga
*.dds
*.plt
*.tmp
*.log
```

Dockerfile powinien dodatkowo uzywac jawnych `COPY`, a nie `COPY . .`.
Syntetyczne fixture sa dozwolone tylko z dedykowanego, przejrzanego katalogu w
`crates/**/tests/fixtures`; jezeli maja rozszerzenie z listy blokowanej, wymagaja
waskiego wyjatku `.dockerignore` dla konkretnej sciezki. Wyjatek nie moze
obejmowac `sample-*`, katalogu instalacji NWN, CEP ani lokalnych proof-output.

## 6. Zasoby retail/CEP i testy env-gated

Obraz, warstwa build cache, registry, CI artifact i statyczny bundle nigdy nie
moga zawierac:

- HAK/MDL/MDX/KEY/BIF ani tekstur retail NWN/Beamdog;
- HAK/MDL/MDX, animacji, szkieletow ani tekstur CEP;
- absolutnych sciezek hosta, lokalnych screenshotow proofu ani sekretow;
- extracted payloadow albo pochodnych fixture z materialu referencyjnego.

Domyslny build/test korzysta tylko z wlasnych syntetycznych fixture. Opcjonalny
test referencyjny wolno uruchomic lokalnie przez bind mount konkretnego pliku w
trybie read-only oraz env var wskazujacy sciezke wewnatrz kontenera, na przyklad:

```powershell
docker run --rm --network none `
  --mount "type=bind,source=<absolute-host-path>,target=/references/cep3_core1.hak,readonly" `
  -e M2A_REFERENCE_CEP_HAK=/references/cep3_core1.hak `
  m2a-quality:local `
  cargo test --locked -p m2a-core --test erf_reference_integration
```

Analogicznie `M2A_REFERENCE_MDL_FILE` moze wskazywac pojedynczy read-only plik.
Brak mounta/env ma dawac kontrolowany skip, tak jak poza Dockerem. Test nie moze
kopiowac payloadu do obrazu, cache, outputu ani logu. CI pozostaje niezalezne od
lokalnych zasobow referencyjnych.

## 7. Komendy operatorskie po implementacji

Docelowy minimalny interfejs powinien byc prosty i zgodny z CI:

```powershell
# Pelny gate zapisany w warstwie buildowej.
docker build --pull --target quality -t m2a-quality:local .

# Okresowy czysty gate bez cache.
docker build --pull --no-cache --target quality -t m2a-quality:clean .

# Odczyt przypietych wersji bez uruchamiania zadnego serwera.
docker run --rm --network none m2a-quality:local rustc --version
docker run --rm --network none m2a-quality:local node --version
docker run --rm --network none m2a-quality:local wasm-pack --version
```

Po przypieciu baz po digestach `--pull` sluzy odswiezeniu metadanych, nie zmianie
wersji. Polecenia kontenerowe uzupelniaja, ale nie usuwaja matrixa Windows/Linux
w GitHub Actions.

## 8. Cache, reproducibility i security gates

```yaml
reproducibility:
  - "base images pinned by exact version and sha256 digest"
  - "Cargo.lock committed; dependency commands use --locked"
  - "wasm-pack installed with exact version and --locked"
  - "tool versions printed in build log"
  - "clean --no-cache build executed before Docker checkpoint acceptance"
  - "same native/WASM gates pass on host/CI and in container"
cache:
  - "BuildKit cache mounts allowed only for Cargo registry/git/target and later npm cache"
  - "cache is an optimization, never evidence of a clean build"
  - "no reference assets, proof outputs, credentials or host paths in cache"
security:
  - "no Docker socket mount"
  - "no privileged mode and no host network"
  - "no secrets in ARG, ENV, labels, history or static bundle"
  - "network disabled for runtime/test execution after dependencies are present"
  - "S1 static runtime runs as non-root with read-only root filesystem where supported"
  - "scan final S1 image and emit SBOM/provenance in release CI"
  - "inspect docker history and exported image contents before publication"
```

Build wymagajacy pobrania crates lub obrazu bazowego oczywiscie potrzebuje sieci.
Zakaz sieci dotyczy wykonywania gotowego test targetu i statycznego runtime'u; nie
mozna deklarowac hermetycznosci, jezeli zaleznosci nie zostaly jeszcze pobrane.

## 9. Windows host proof boundary

Docker nie jest i nie bedzie oracle Aurora. NWN EE Toolset oraz gra pozostaja na
hoscie Windows. Kontener Linux nie ma zatwierdzonej sciezki do wiarygodnego
proofu GUI, renderingu, zasobow modulu ani zachowania engine'u.

Dozwolony przeplyw po M5/M6:

```text
container/host quality gates
  -> meshy2aurora generates its own HAK/module/proof manifest
  -> output exported to a neutral host proof-output directory
  -> hashes verified on host
  -> generated content installed/selected on Windows host
  -> manual NWN EE Toolset/game proof
  -> screenshots, manifest, hashes and named result recorded in stage evidence
```

Kontener nie powinien zapisywac bezposrednio do katalogow instalacji NWN ani
`Documents/Neverwinter Nights`. Nie montujemy do niego NWN/CEP tylko po to, aby
udawac finalny proof. Read-only mount jest dopuszczalny wylacznie dla jawnego,
env-gated testu readera/reference inspection.

## 10. Definition of done dla checkpointu Docker build/test

Dodanie Dockerfile jest osobnym checkpointem implementacyjnym. Moze zostac uznane
za gotowe dopiero, gdy:

- istnieja przejrzane `Dockerfile` i `.dockerignore` zgodne z tym suplementem;
- wersje Rust `1.96.1`, Node `24.x` (konkretny patch) i `wasm-pack 0.15.0` sa
  sprawdzane w logu oraz bazowe obrazy sa przypiete digestami;
- format, clippy, wszystkie workspace tests, WASM build i publiczny Node/WASM
  adapter przechodza w czystym buildzie;
- te same gates nadal przechodza na hoscie i w CI;
- skan kontekstu, warstw oraz `docker history` nie wykrywa retail/CEP assetow,
  proof payloadow, sekretow ani absolutnych sciezek hosta;
- env-gated test bez mounta konczy sie kontrolowanym skipem;
- opcjonalny test z mountem uzywa `readonly`, nie zapisuje payloadu i raportuje
  tylko dozwolone metadane/hash;
- evidence zapisuje exact commands, exit codes, wersje, digest obrazu i wynik
  skanu; sam zielony Docker build nie zamyka M6;
- niezalezny review nie ma otwartych findings.

## 11. Kolejnosc wdrozenia

```yaml
D0_docs:
  status: "this supplement"
D1_build_test:
  when: "after the active M2 quality fixes are stable"
  artifacts: ["Dockerfile", ".dockerignore", "Docker evidence"]
  product_effect: "none; developer/CI reproducibility only"
D2_static_web:
  when: "S1 after M6 DONE"
  artifacts: ["static web build", "optional minimal runtime image"]
  product_effect: "alternate static delivery only; conversion remains browser-local"
```

Ten suplement nie zmienia `orchestrator-state.yaml`, nie uruchamia S1 i nie
autoryzuje kopiowania zewnetrznych assetow do repozytorium ani obrazu.
