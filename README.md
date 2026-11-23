"# PREFLIGHT" 

Preflight je načrtovana namizna aplikacija in CLI za preverjanje razvojnega okolja pred delavnico. Lokalna diagnostika bo delovala brez računa; deljenje minimalnih rezultatov z učiteljem bo prostovoljno.

## Načrt

- Faza 00: Git/GitHub repozitorij in projektna dokumentacija.
- Faza 01: Rust in npm monorepo, Tauri namizna aplikacija, Axum API in spletni dashboard.
- Faza 02: validirani YAML profili, preverjanje Python/Git/Node/Java in CLI s tekstovnim ter JSON izpisom.
- Nadaljevanje: namizni pregled rezultatov, dodatna preverjanja, učiteljske sobe in vodeni popravki.

Podrobnosti so v [ROADMAP.md](ROADMAP.md), projektni dogovor v [AGENTS.md](AGENTS.md), priprava okolja pa v [START_HERE.md](START_HERE.md).

## Trenutno stanje

Projekt vsebuje začetno dokumentacijo; aplikacija še ni implementirana. Ukazi za dev/build/lint/test bodo dodani z ogrodjem v fazi 01.

Pred razvojem preveri Git, GitHub CLI, Rust (na Windows MSVC), Node.js in predpogoje Tauri 2 po START_HERE.md. Cargo.lock in package-lock.json bosta vključena v Git zaradi ponovljivih gradenj.
