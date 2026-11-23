# Preflight

Preflight pred delavnico lokalno preveri razvojno okolje. Namizna aplikacija in CLI uporabljata isto Rust jedro. Rezultati ostanejo na računalniku; učiteljske sobe in deljenje so načrtovani za poznejše faze.

## Razvoj

Zahteve: Node.js 24+, npm, stabilni Rust in [predpogoji Tauri 2](https://v2.tauri.app/start/prerequisites/). Windows potrebuje MSVC C++ Build Tools in WebView2; macOS Xcode Command Line Tools; Linux WebKitGTK 4.1 in knjižnice iz CI workflowa. Orodji cargo in npm morata biti na PATH. V PowerShell lahko ob omejitvi izvajanja skript uporabiš `npm.cmd`.

Iz korena repozitorija:

```sh
npm ci
npm run dev:desktop
npm run dev:dashboard
npm run dev:server
cargo run -p preflight-cli -- --help
```

Vsak dev ukaz zaženi v svojem terminalu. Dashboard je na http://127.0.0.1:1421, API na http://127.0.0.1:3001/health (vrne `ok`). Desktop Vite strežnik uporablja vrata 1420; Rust ukazi delujejo samo v Tauri oknu.

```sh
npm run build
npm run lint
npm test
npm run build:desktop
```

`build` zgradi oba frontenda in celotni Cargo workspace. `build:desktop` ustvari produkcijsko namizno izvršljivo datoteko brez namestitvenega paketa. `lint` preveri ESLint, TypeScript, Rust fmt in Clippy. CI preveri tri OS. Namestitveni paketi in podpisovanje pridejo pozneje.

## Moduli in načrt

- `crates/preflight-core`: skupno jedro.
- `crates/preflight-cli`: ukaz `preflight`.
- `apps/desktop`: React/TypeScript/Vite + Tauri 2; eksplicitni Rust command `get_app_info`.
- `apps/server`: Axum API, trenutno samo `/health`, vezan na loopback.
- `apps/dashboard`: jasno označen pogled »v pripravi«.

Faza 00 pripravi repozitorij; faza 01 ogrodje; faza 02 tipizirane profile in CLI preverjanja; faza 03 offline namizni pregled rezultatov. Celotni načrt je v [ROADMAP.md](ROADMAP.md), dogovor v [AGENTS.md](AGENTS.md), začetna priprava v [START_HERE.md](START_HERE.md).

Implementacija uporablja [uradni Tauri IPC](https://v2.tauri.app/develop/calling-rust/) in [Vite](https://vite.dev/guide/). JavaScript nima splošnih shell ali filesystem dovoljenj. Lock datoteki sta vključeni v Git.
