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

## CLI preverjanje

```sh
cargo run -p preflight-cli -- check --profile profiles/python-workshop.yaml --format text
cargo run -p preflight-cli -- check --profile profiles/python-workshop.yaml --format json
```

Exit kode: 0 = vse obvezne zahteve izpolnjene, 1 = vsaj ena ni izpolnjena (tudi unknown), 2 = neveljaven/neberljiv profil ali napačna uporaba CLI. Neobvezna neustrezna različica vrne warning. `unknown` nikoli ne šteje kot uspešno preverjanje.

Profil uporablja `schema_version: 1`, enolične ID-je in samo `type: runtime` z `tool: python|git|node|java`. Java pomeni JDK: preverita se java in javac. Omejitve sledijo Rust semver, npr. `>=22.0.0,<23.0.0`; golo `22.0.0` pomeni združljiv razpon, za točno različico uporabi `=22.0.0`. Stare Java 1.8.0_412 različice se normalizirajo na 8.0.412. Nestabilni/neprepoznani izpisi vrnejo unknown. YAML je omejen na 64 KiB, 32 preverjanj, globino 8; neznana polja, tagi, aliasi in več dokumentov niso dovoljeni.

Vsak proces ima 5 sekund in največ 32 KiB skupnega izhoda; stdout/stderr se bereta sočasno in omejeno. Procesi se poženejo neposredno s statičnimi argumenti, brez shell-a, in se ob omejitvi ustavijo. Python: Windows python, nato py -3; Unix python3, nato python. JSON ne vsebuje surovega izhoda, poti do profila ali podatkov o računalniku. ID-ji preverjanj prihajajo iz profila: vanje ne zapisuj osebnih podatkov. Izpisov ne shranjuj v Git.

## Namizni pregled (faza 03)

`npm run dev:desktop` odpre namizno aplikacijo. Izberi Python delavnico, klikni »Zaženi preverjanje«, razširi posamezno vrstico za razlago in klikni »Ponovno preveri«. Obvezni unknown prepreči oznako pripravljenosti. Med izvajanjem sta izbira profila in ponovni zagon zaklenjena tudi v Rust sloju.

Pregleda Zasebnost in Nastavitve pojasnita lokalno obdelavo in omogočata sistemsko/svetlo/temno temo. Tipkovnica: Tab za premik, Enter/Space za gumbe in podrobnosti. Obstajajo vidni fokus, skip link in živi povzetek. Rezultati se ohranijo samo v pomnilniku; shrani se le izbira teme. Aplikacija neposredno kliče preflight-core, brez CLI podprocesa ali klicev API strežnika.

V tej fazi je na voljo en vgrajen profil. Uvoz profilov, dodatna diagnostika in izvoz so predmet faze 04. Učiteljsko deljenje še ni na voljo.
