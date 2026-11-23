# PREFLIGHT — FAZNI NAČRT IMPLEMENTACIJE (ROADMAP.md)

Projekt izdeluj **fazo za fazo**. Vsaka faza ima svojo Git vejo, teste, push in PR. Nikoli ne preskoči kriterijev sprejema. Ne začni naslednje faze brez uporabnikovega naročila in preverjene/združene predhodne faze.

## Faza 00: inicializacija Git/GitHub repozitorija

**Veja:** `main` (edina izjema od pravila PR za začetni commit).

Naloge:
- Preveri, ali smo v obstoječem repozitoriju; če ne, `git init -b main`.
- Ohrani te tri dokumente: `AGENTS.md`, `ROADMAP.md`, `START_HERE.md`.
- Ustvari minimalni `README.md` (kaj je Preflight in načrt), primerno `.gitignore` za Rust, Tauri, Node, env, SQLite, macOS in IDE; ne briši ničesar uporabnikovega.
- Preveri, ali sta Git in GitHub CLI na voljo; za GitHub prijavo naj uporabnik po potrebi sam izvede `gh auth login`.
- Če ni oddaljenega repozitorija, ustvari **private** GitHub repo `preflight` prek `gh repo create preflight --private --source=. --remote=origin --push` po prvem commitu. Če obstaja, ne ustvarjaj drugega.
- Prvi commit: `docs: initialize Preflight project plan`.

**Sprejem:** Git repo deluje, `main` obstaja, dokumenti so commitani, remote GitHub repo obstaja in `git status` je čist. Poročaj URL repozitorija. Če prijava/pravice ne delujejo, dokončaj lokalne korake, točno poročaj in ustavi.

## Faza 01: monorepo in ogrodje

**Veja:** `feat/01-project-setup`.

Naloge:
- Nastavi Cargo workspace in npm workspaces.
- Ustvari `preflight-core` Rust knjižnico in `preflight-cli` binarno aplikacijo z osnovnim `--help`.
- Ustvari `apps/desktop` kot Tauri 2 + React + TypeScript + Vite aplikacijo; preveri začetni ekran in Rust command `get_app_info`.
- Dodaj `apps/server` Axum aplikacijo z `GET /health` in `apps/dashboard` React/Vite placeholder z jasnim napisom »v pripravi«.
- Ustvari enotno dokumentirane skripte za dev/build/lint/test, pravilno `.gitignore`, začetno `.github/workflows/ci.yml`.
- Preveri Cargo/Tauri workspace povezavo in lokalno gradnjo; zabeleži OS posebnosti.

**Sprejem:** Rust crate-i se zgradijo; CLI --help deluje; desktop odpre okno na razvojnem OS; API `/health` vrne `ok`; dashboard se prikaže; GitHub CI preverja vsaj Rust fmt/check/test in TS lint/build.

**Demo:** ukazi v `README.md`, dejanski screenshot UI je dobrodošel v PR, ni obvezen.

## Faza 02: osnovni preverjalnik in CLI

**Veja:** `feat/02-check-engine`.

Naloge:
- Določi tipizirane modele: Profile, Check, CheckResult, Summary, Status, Error; `schema_version` in `reason_code`.
- Implementiraj YAML parser in validacijo; le statično dovoljene vrste preverjanja.
- Implementiraj preverjanja `runtime`: Python (Windows `py` fallback, `python3` na Unix), Git, Node, Java/JDK; razčlenjevanje verzij in primerjava omejitev.
- Poseben process runner z timeoutom, omejitvijo velikosti izhoda in neposrednim spawnom brez shell-a.
- CLI `preflight check --profile ... --format json` ter `--format text`, exit kode 0,1,2.
- V JSON ne vključuj poti, surovega stdout/stderr, imen uporabnikov, strojnih ID.
- Dodaj unit/integration teste z lažnim process runnerjem, vključno z manjkajočim PATH, različnimi izhodi `java -version`, `python --version`, `py -3`, Node `v22.x`, Git različico in neveljavnim profilom.

**Sprejem:** CLI preveri pravi primer `profiles/python-workshop.yaml`; vrne razumen slovenski izpis in stabilen JSON; vsi testi s simuliranimi ukazi so zeleni, tudi ko na CI ni lokalne Jave.


## DO TU SM PRSU



## Faza 03: prava desktop aplikacija, offline diagnostika

**Veja:** `feat/03-desktop-ui`.

Naloge:
- Uporabi `preflight-core` neposredno iz Tauri Rust kode (ne zaganjaj CLI procesa iz React-a).
- Oblikuj premišljen desktop UI: pregled/prvi zagon, izbira profila, »Zaženi preverjanje«, seznam statusov, podrobnosti in priporočila, ponovno preverjanje, skupni rezultat.
- Stanja med izvajanjem, timeout, `unknown`, prikaz zakaj test ni uspel, empty state, tipkovnica in bralniki zaslona.
- Moderna vizualna podoba: umirjene barve, dosledni razmiki, svetli/temni način, brez odvečnih animacij. Ne zanašaj se samo na zeleno/rdečo, dodaj ikone in besedilo.
- Privzeto brez omrežnih klicev na strežnik; jasno napiši »Rezultati ostanejo na tvojem računalniku«.
- Dodaj primerljive UI/komponentne teste, kjer je razumno.

**Sprejem:** desktop app dejansko zažene lokalne Rust teste, prikaže rezultate in ponovno preverjanje; dela brez API/računa. UI ni mockup in nima lažnih rezultatov.

## Faza 04: razširjena diagnostika in profili

**Veja:** `feat/04-profiles`.

Naloge:
- Preveri VS Code razširitve (kolikor je zanesljivo), prost disk in eksplicitne HTTPS povezave; varno obravnavaj nedostopnost VS Code CLI in omrežne napake.
- Vključi profile `python-workshop.yaml`, `java-workshop.yaml`, `web-workshop.yaml`. Vsak ima razumljive zahteve in dokumentirano semantiko verzij.
- Dodaj lokalni pregled/uvoz/izvoz profilov ter preprost urejevalnik za učitelja; validacija in človeku prijazna napaka ob napačnem YAML-u.
- Izvozi lokalni tekstovni/JSON povzetek brez osebnih podatkov.
- Profil ne sme sprožiti poljubnega ukaza. HTTPS zaščiti pred SSRF (zasebni/loopback/metadata naslovi, preusmeritve, ponovne DNS razrešitve).
- Testi za disk, razširitve, profile, napake, timeout, validacijo URL.

**Sprejem:** možno uvoziti YAML, videti napake in uspešno izvesti vse podprte vrste testov. Neznano je prikazano kot `unknown`, ne kot izmišljeno `pass/fail`.

## Faza 05: spletni API, sobe in SQLite

**Veja:** `feat/05-classroom-api`.

Naloge:
- Axum + SQLx/SQLite in migracije. Konfiguracija preko env (z `.env.example`, nikoli pravimi skrivnostmi v Gitu).
- Sobe: naključna koda za pridružitev (8 znakov, enostavno berljiva), generirana dolga učiteljska skrivnost (zgoščeno shranjena), izbrani profil, ustvarjen čas, čas poteka.
- API (po potrebi izboljšaj imena, vendar dokumentiraj in testiraj):
  - `POST /api/v1/rooms` — ustvari sobo in vrne `room_code` + `teacher_token`;
  - `GET /api/v1/rooms/{code}/profile` — javno vrne **validiran** varni profil;
  - `POST /api/v1/rooms/{code}/sessions` — pridruži anonimnega udeleženca, vrne njegov kratkoživ token;
  - `PUT /api/v1/rooms/{code}/sessions/{session_id}/results` — posodobi le lastne minimizirane rezultate po opt-in;
  - `DELETE /api/v1/rooms/{code}/sessions/{session_id}` — izbriši lastno sejo;
  - `GET /api/v1/rooms/{code}/results` — samo učiteljska skrivnost; vrne agregate in psevdonimne posamezne rezultate;
  - `DELETE /api/v1/rooms/{code}` — samo učitelj.
- Povsem loči identiteto učitelja od javne kode sobe in identitete študenta. Rate-limit ter validiraj velikost payloadov, profilov in parametrov.
- Rezultati so minimalni: `check_id`, status, verzija, morebitni reason_code; nikoli raw logs, lokalnih poti ali sistemskih uporabniških imen.
- Privzeta hramba 24h do poteka sobe, cleanup job in test poteka/izbrisa. Brez aplikacijskih IP-logov; opozori na zunanji reverse proxy.
- Pripravi `docs/api.md`, `docs/privacy.md`, `docs/deployment.md` z razlago HTTPS, CORS, environment in SQLite backup omejitev.
- Integracijski testi za sobo, dostop s pravo/napačno skrivnostjo, drugega udeleženca, potek in brisanje.

**Sprejem:** funkcionalen lokalni end-to-end API z migracijami; neavtoriziran učitelj/udeleženec ne more prebrati tujih rezultatov; po poteku ni več podatkov.

## Faza 06: učiteljski dashboard in povezava desktop → soba

**Veja:** `feat/06-teacher-dashboard`.

Naloge:
- Dashboard: ustvari sobo z validiranim profilom, vidi kodo in povezavo, učiteljsko skrivnost varno obravnava, prikazuje agregate in psevdonimne seje. Če učitelj osveži stran, ne sme naključno izgubiti nadzora brez jasnega opozorila; ne puščaj tokenov v URL.
- Samodejno osveževanje npr. na 5 sekund (polling; brez WebSocketov v MVP), filter za težavne preverbe, indikator zadnje osvežitve, stanje prazne sobe in poteka.
- Desktop: vnos kode sobe, varen prenos profila, lokalni pregled, zaslon »To se bo delilo«, checkbox/izrecni gumb za privolitev, pošiljanje samo ob potrditvi, po potrebi izklop in brisanje seje.
- Brez privolitve in brez učiteljske sobe mora lokalna funkcionalnost ostati polna.
- Demonstracija z vsaj dvema simuliranima udeležencema in enim učiteljem; preveri varnost/nepooblaščen pregled.
- UI v slovenščini in solidni loading/error statusi.

**Sprejem:** učitelj ustvari sobo; dva odjemalca se pridružita; rezultati se prikažejo le ob privolitvi in osvežujejo; možno odstraniti sejo; učiteljski pregled je zaščiten.

## Faza 07: vodeni popravki, jasne razlage

**Veja:** `feat/07-guided-fixes`.

Naloge:
- Za vsak pogost razlog napake dodaj slovensko, kratko in točno razlago ter preverjene korake za Windows/macOS/Linux.
- Vključi »Ponovno preveri« za posamezen test, kopiranje varnega ukaza za ročno namestitev, povezavo na uradno spletno stran orodja. Navodila naj pojasnijo PATH, različice in ponovni zagon terminala/aplikacije.
- Za konkretne primere (Python ni v PATH, napačna Java, manjka Node, ni VS Code razširitve) različne rešitve. Povezave/ukaze vzdržuj v zaupanja vrednem katalogu, ne v oddaljenem YAML.
- Če platforma ali različica ni podprta, to jasno povej.
- Testi za preslikavo napak v nasvete in varen prikaz ukazov.

**Sprejem:** pri napaki se odpre pravilno lokaliziran vodič, ki je konkretno uporaben in ne izvaja ničesar brez uporabnika.

## Faza 08: omejeni samodejni popravki z izrecno potrditvijo (šele po 07)

**Veja:** `feat/08-safe-installs`.

Naloge:
- Implementiraj varen opt-in mehanizem `FixAction` z vnaprej kodiranimi podprtimi kombinacijami OS + orodje + preverjena identiteta paketa (winget / Homebrew / apt); začni z MAJHNIM seznamom preverjenih akcij.
- Nikoli ne sprejmi programa, package ID, URL ali argumentov iz YAML ali API. Vse dovoljene akcije so definirane v zaupanja vredni kodi/katalogu in preverjene pred zagonom.
- Pred vsakim popravkom pokaži ime programa, upravljalnik paketov, predviden ukaz, predvidene učinke in morebitno zahtevo po administratorskih pravicah; uporabnik mora posebej klikniti »Potrdi namestitev«.
- Preveri prisotnost upravljalnika paketov in prikaži napako, če ni na voljo; brez avtomatskega prenosa skript, brez `curl | sh`, brez prikritega `sudo` ali splošnega shell izvajanja. `apt` uporabljaj le v ustrezno podprti distribuciji, kjer je jasno, da je akcija varna.
- Po zaključku preveri status in ponudi ponovni pregled. Prikaz napredka; prepreči več sočasnih namestitev.
- V testih **nikoli dejansko ne nameščaj paketov**, ampak uporabi simuliran runner; v dokumentaciji napiši, katere platforme/akcije so potrjene, katere pa samo ročno vodene.

**Sprejem:** vsaj 1–2 res podprta in ročno preizkušena popravka na razvojnem OS, vedno z dodatno privolitvijo. Drugod varen ročni fallback. Ne trdi, da vse deluje na vseh treh OS, če ni preizkušeno.

## Faza 09: CI, pakiranje, zasebnost, pilot v razredu

**Veja:** `feat/09-release-readiness`.

Naloge:
- Celovit CI za Rust fmt/clippy/test, TS lint/test/build in osnovne API integration teste.
- GitHub Actions matrix build za Windows/macOS/Linux Tauri instalacijske artefakte. V dokumentaciji loči nepodpisane testne gradnje od podpisanih/notariziranih izdaj, opozori na SmartScreen/macOS Gatekeeper.
- Dodaj ročni release workflow `workflow_dispatch` in označevanje verzije `v0.1.0` šele ko testi in smoke test uspejo; ne objavljaj javne izdaje brez uporabnikovega soglasja.
- Produkcijska navodila za API/dashboard prek HTTPS in varno konfiguracijo CORS, shranjevanje SQLite podatkov na trajnem disku, cleanup/retention in osnovno spremljanje zdravja.
- Pripravi navodila za prvi pilot z 10–15 udeleženci: ustvarjanje sobe, zagon, preverjanje, povratne informacije. Opiši omejitve in GDPR vprašanja za šolo; brez lažne pravne garancije.
- Preglej minimalna Tauri dovoljenja, URL preverjanje, avtentikacijo in odvisnosti. Osveži `README.md`, `docs/troubleshooting.md`.

**Sprejem:** koda prestane CI na treh OS, instalacijski artefakti so na voljo iz Actions in preverjeni, E2E lokalna demonstracija uspe; poznane varnostne in distribucijske omejitve so pošteno opisane.

---

## Definition of Done (za VSAKO fazo)

- [ ] Samo predvidene funkcionalnosti faze, brez velikih nepovezanih refaktorjev.
- [ ] Kompilacija in relevantni avtomatizirani testi uspejo ali so izrecno dokumentirane blokade, ki niso prikrite.
- [ ] Ni skrivnosti, osebnih podatkov ali build artefaktov v Git.
- [ ] README/docs posodobljeni z ukazi za zagon.
- [ ] Git feature veja je na GitHub, PR je odprt (razen pri fazi 00).
- [ ] V PR so opis, testi, omejitve in kako preveriti delovanje.
- [ ] Codex poroča v slovenščini in se ustavi do naslednjega naročila.

## Opombe za določanje obsega

Za prvi zares uporabni *offline MVP* dokončaj faze **00–04**. Za učiteljski pilot z nadzorno ploščo dokončaj **00–07**. Samodejne namestitve so bistveno bolj občutljiv del in pridejo **šele po preverjanju osnovne aplikacije**. Fazi 08–09 pomenita nadgradnjo in pripravo na izdajo. Pro naročnine in plačila šele po dejanski validaciji pri učiteljih.
