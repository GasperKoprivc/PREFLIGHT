# PRE-FLIGHT — NAVODILA ZA CODEX (AGENTS.md)

Ta datoteka je trajni projektni dogovor. Preberi jo na začetku vsake Codex seje skupaj z `ROADMAP.md` in `START_HERE.md`.

## 1. Vloga in način dela

Deluj kot izkušen senior full-stack/Rust/Tauri razvijalec in skrben vodja projekta. Cilj je delujoča, varna, preprosta in lepo oblikovana namizna aplikacija **Preflight** z učiteljskim spletnim nadzornim pogledom.

**Pomembno: IZVAJAJ SAMO ENO FAZO na posamezno uporabnikovo zahtevo.** Če naročilo ne navaja faze, začni pri prvi nedokončani fazi v `ROADMAP.md`. Ne preskakuj korakov, ne generiraj vse aplikacije hkrati. Po koncu faze ustavi delo, poročaj in počakaj na zahtevo za naslednjo fazo. Ne obljubljaj, da boš delo nadaljeval v ozadju.

Komunikacija z uporabnikom, navodila za aplikacijo, napake in osnovni UI naj bodo **v slovenščini**, razumljivo in jedrnato. Imena datotek, razredov, API poti, Git vej, commitov in komentarjev v kodi naj bodo v razumljivi angleščini. Ne pozivaj uporabnika, naj ročno kodira stvari, ki jih lahko narediš sam.

## 2. Namen izdelka

Študent pred delavnico zažene namizno aplikacijo (Windows, macOS, Linux), izbere ali prejme profil zahtev in izvede lokalno preverjanje. UI mu jasno pove, kaj je v redu, kaj manjka in kako popraviti. Učitelj lahko ustvari sobo in spremlja **prostovoljno deljene** anonimizirane/psevdonimne rezultate.

MVP mora omogočati:
- lokalno diagnostiko brez računa, povezave s strežnikom ali internetom, razen kadar profil posebej zahteva spletni test;
- preverjanje Python, Java/JDK, Git, Node.js, VS Code razširitev, prostega prostora in HTTPS dosegljivosti;
- lokalne profile YAML z validacijo in vgrajenimi primeri;
- pregledno namizno aplikacijo in samostojen CLI z JSON izhodom;
- učiteljeve sobe, kodo sobe, učiteljsko nadzorno ploščo, pregledne podatke v živo (polling je dovolj);
- izrecno soglasje za deljenje rezultatov, brez surovih logov, uporabniških imen, domačih poti ali strojnih ID-jev;
- vodene, preverjene popravke, pozneje omejeno podprte samodejne namestitve z dodatno potrditvijo;
- GitHub CI in gradnje za tri OS.

Ne vključuj plačil, naročnin, šolskih SSO prijav, arbitrarnosti `custom scripts`, samodejnega oddaljenega izvajanja ukazov, telemetrije ali kompleksnih funkcij pred zaključkom MVP.

## 3. Tehnična arhitektura (ne spreminjaj brez tehtnega razloga)

- **Rust**: samostojna knjižnica `preflight-core`, ki izvaja vse preverjalnike; uporablja jo `preflight-cli` in Tauri namizna aplikacija brez podvajanja logike.
- **Tauri 2**: namizna aplikacija, React + TypeScript + Vite, dostop do preverjanj preko eksplicitnih Rust `tauri::command` funkcij.
- **Axum + Tokio + SQLx SQLite**: majhen spletni API za sobe, z učiteljskimi skrivnostmi in časovno omejenim shranjevanjem minimalnih podatkov.
- **React + TypeScript + Vite**: ločen, odziven učiteljski dashboard; ob produkcijski namestitvi se servira iz iste domene kot API, če je mogoče.
- **npm workspaces**: JS/TS paketi v `apps/desktop` in `apps/dashboard`.
- **Cargo workspace**: Rust paketi v `crates/preflight-core`, `crates/preflight-cli`, `apps/desktop/src-tauri`, `apps/server`.
- **SQLite**: lokalna strežniška podatkovna zbirka, migracije v Git repozitoriju; brez zunanjega DB strežnika v MVP.
- **Preprost, kakovosten UI**: odzivnost, svetla in temna tema, dostopnost (ne zanašaj se samo na barvo), slovenska lokalizacija, koristna prazna/stanja napak/napredka.

Predlagana končna struktura:

```text
preflight/
├─ AGENTS.md
├─ ROADMAP.md
├─ START_HERE.md
├─ README.md
├─ Cargo.toml
├─ Cargo.lock
├─ package.json
├─ package-lock.json
├─ .gitignore
├─ .github/
│  ├─ workflows/ci.yml
│  └─ workflows/release.yml
├─ crates/
│  ├─ preflight-core/
│  └─ preflight-cli/
├─ apps/
│  ├─ desktop/
│  │  ├─ src/
│  │  └─ src-tauri/
│  ├─ dashboard/
│  └─ server/
├─ profiles/
│  ├─ python-workshop.yaml
│  ├─ java-workshop.yaml
│  └─ web-workshop.yaml
├─ docs/
│  ├─ architecture.md
│  ├─ api.md
│  ├─ privacy.md
│  ├─ troubleshooting.md
│  └─ deployment.md
└─ tests/
```

Prilagodi strukturo samo, kadar to zahteva dejanski Cargo/Tauri tooling. Ohrani jasne meje modulov in zagotovljeno delujoče gradnje. Uporabi trenutno podprte stabilne različice odvisnosti in uradno dokumentacijo, ne izmišljenih CLI argumentov.

## 4. Kontrakti profilov in rezultatov

Uporabi dokumentirano in verzionirano YAML shemo (`schema_version: 1`). Profil lahko vsebuje **samo vnaprej dovoljene vrste preverjanj**; nikoli ne sme biti poljubnih shell ukazov, skript ali parametrov za izvajanje.

Predlagan primer:

```yaml
schema_version: 1
id: python-workshop
name: "Python delavnica"
checks:
  - id: python
    type: runtime
    tool: python
    version: ">=3.11.0"
    required: true
  - id: git
    type: runtime
    tool: git
    version: ">=2.40.0"
    required: true
  - id: node
    type: runtime
    tool: node
    version: ">=22.0.0,<23.0.0"
    required: false
  - id: vscode-python
    type: vscode_extension
    extension: ms-python.python
    required: true
  - id: disk
    type: disk_space
    min_free_gb: 5
    required: true
  - id: pypi
    type: https
    url: https://pypi.org
    required: false
```

`runtime.tool` je dovoljen ID iz registra (`python`, `java`, `git`, `node` itd.), ne pot, izvršljiva datoteka ali sestavljen ukaz iz YAML. Še posebej prepreči možnost `sh -c`, `cmd /C`, PowerShell eval, URL-download-and-execute ipd. HTTPS naj dovoljuje samo `https://` in preprečuje dostop do loopback/zasebnih/metadata naslovov, kadar profil izvira iz oddaljenega vira (SSRF zaščita; upoštevaj tudi preusmeritve/DNS).

Kanonični statusi: `pass`, `fail`, `warning`, `unknown`; poleg tega strojno berljiv `reason_code`. Vsak rezultat vsebuje `check_id`, `status`, `required`, `installed_version` (kadar smiselno), `message_sl`, `suggestion_sl`. Ne vračaj surovega stdout/stderr ali absolutnih lokalnih poti v deljivih rezultatih. `unknown` ni isto kot `pass`.

CLI oblika: `preflight check --profile profiles/python-workshop.yaml --format json`.
- Exit 0: vsi obvezni preverjalniki so `pass` (opozorila dovoljena le za neobvezne).
- Exit 1: obvezni preverjalnik ni `pass`.
- Exit 2: neveljaven profil ali programska napaka.
- Podpri še človeku prijazen izpis v terminalu.

## 5. Implementacija preverjanj

- Vse izvršljive procese poženi s `std::process::Command` ali ustrezno varno abstrahiranjem **brez lupine** in s statično določenimi argumenti iz registra. Poskrbi za timeout, mejo izhodnih podatkov in robustno obravnavo napak.
- Išči programe na PATH; za Python na Windows preveri `py` launcher, za Java pravilno preberi tudi stderr. Podpri dejanske verzijske izpise na treh OS.
- Če ni mogoče zanesljivo preveriti namestitve (na primer VS Code je nameščen, njegov CLI pa ni v PATH), uporabi `unknown` z uporabnim navodilom, ne lažnega `fail`.
- Za VS Code razširitve uporabi dovoljene mehanizme (npr. `code --list-extensions`) in pametne, varne fallbacke; razumno loči »CLI ni na voljo« od »razširitev manjka«.
- Preverjanje diska naj uporablja ustrezne sistemske API-je/knjižnice, brez branja uporabnikovih datotek.
- Spletna preverjanja morajo imeti timeout, omejitve preusmeritev, jasno razlago in ne smejo izvajati vsebin s spleta.
- Preverjanja morajo biti testabilna s fake process runnerjem in fake filesystem/network slojem; testov ne pogojuj z nameščeno Javo/Pythonom na CI.

## 6. Zasebnost in varnost

- **Offline-first**. Lokalno preverjanje ne potrebuje strežnika; rezultati se nikoli ne pošljejo, dokler uporabnik ne izbere prostovoljnega deljenja.
- Pred deljenjem pokaži, katere podatke bomo poslali. Privzeto je deljenje izključeno; omogoči preklic deljenja in odstranjevanje seje, kjer je izvedljivo.
- Pri sobah uporabljaj naključne začasne ID-je in vzdevke, ne strojnih fingerprintov. Učitelju prikaži število udeležencev in njihove psevdonimne statuse. Imena samo če jih oseba sama izrecno vnese; za MVP jih ne zahtevaj.
- Ne shranjuj e-pošte, uporabniških imen OS, IP naslovov v aplikacijske tabele, vsebine okolskih spremenljivk, nameščenih paketov zunaj profila, domačih poti, celih logov ali osebnih podatkov brez potrebe. Zavedaj se, da IP v obratovalnih logih/proxyju in spletni identifikatorji še vedno lahko predstavljajo osebne podatke; **ne zatrjuj, da je projekt samodejno GDPR compliant**.
- Sobe naj potečejo (npr. 24 ur) in rezultati naj se samodejno odstranijo ob poteku; pripravi kratek razumljiv opis obdelave podatkov v `docs/privacy.md` in nastavitve hrambe.
- Učiteljeva skrivnost je dolga, naključna in ni enaka kratki kodi sobe; shrani samo hash. Učiteljske API-je zavaruj; koda sobe ne sme omogočati branja tujih rezultatov. Vsak študent sme posodobiti/izbrisati le svojo sejo. Dodaj omejitve hitrosti in validacijo vseh vhodov.
- V produkciji uporabi HTTPS, strogo CORS politiko, varno ravnanje s piškotki/žetoni in brez izpisovanja skrivnosti v dnevnikih. `teacher_token` ne daj v query string ali URL.
- Tauri dovoljenja naj bodo minimalna. Ne omogoči splošnega `shell:allow-execute` ali poljubnih sistemskih ukazov iz JavaScripta. Vsa preverjanja se izvedejo v Rust in preverijo dovoljeni seznam.
- Ne izvajaj ukazov iz oddaljenega YAML. **Custom check skripte, oddaljeni plugini, integracije z `curl | sh` so izven MVP.**
- Samodejni popravki so ločena, poznejša faza. Dovoljena je le vnaprej kodirana matrika preverjenih akcij/paketov za posamezni OS, z jasnim prikazom ukaza, posledic, potrebnih dovoljenj in dodatno eksplicitno potrditvijo. Nikoli brez vednosti uporabnika; če ni mogoče varno avtomatizirati, pokaži ročna navodila.

## 7. Git/GitHub — obvezen proces

Repozitorij: `preflight`, prvotno **private**, dokler uporabnik ne odloči drugače. Glavna veja `main` je stabilna. Uporabljaj GitHub Flow in **eno vejo na fazo**.

Pri vsaki novi fazi:
1. Preveri `git status`, `git branch --show-current`, `git remote -v`, `gh auth status`. Ne povozi obstoječih uporabnikovih sprememb, ne uporabljaj `reset --hard`, `push --force`, `clean -fd` ali `rebase` za skrivanje težav.
2. Začni iz svežega `main`: `git switch main`, `git pull --ff-only origin main`, nato `git switch -c feat/NN-ime-faze` (ali `fix/...` za popravke). Če obstaja odprt PR prejšnje faze, najprej poročaj in ne nadgrajuj na nezdruženo fazo brez uporabnikovega navodila.
3. Izvedi **samo naloge iz trenutne faze**. Ustvari razumljive manjše commite (`feat(core): ...`, `test(core): ...`, `docs: ...`, `ci: ...`, `fix: ...`). Nikoli ne commitaj skrivnosti, `.env`, API žetonov, osebnih rezultatov, SQLite baze, `target`, `node_modules`, `dist`.
4. Zaženi formatiranje, lint, teste in build, ki so relevantni za trenutno fazo. Odpravi napake, kjer je mogoče. Dokumentiraj, če kaj zaradi OS ali manjkajočih odvisnosti ni bilo preverjeno; ne izmisli si uspešnosti.
5. `git push -u origin <branch>` in `gh pr create --base main --head <branch> --title "..." --body "..."`. V PR opiši izvedeno, testiranje, znane omejitve in kako preveriti UI.
6. **Ne združuj PR samodejno** in ne pushaj neposredno na `main`, razen začetnega zagona repozitorija ali če uporabnik to izrecno zahteva. Uporabniku povej, naj pregleda/združi PR, nato naroči naslednjo fazo. Za naslednjo fazo vedno izhajaj iz posodobljenega `main`.
7. Če `gh` ni nameščen/prijavljen ali pravice niso odobrene, razloži točen ukaz za odpravo, vendar ne zahtevaj, da uporabnik prilepi skrivnost v chat. Ohrani lokalno delo in poskusi znova po prijavi. Nikoli ne trdi, da je push uspel, če ni.

`gh auth login`, `gh repo create preflight --private --source=. --remote=origin --push` sta dovoljena v začetnem koraku, če uporabnik še nima repozitorija. Če repozitorij ali remote že obstaja, preveri in uporabi obstoječega, brez podvajanja.

## 8. Standard kakovosti

- Tipi, modularnost, pregledna imena, malo odvisnosti; uporabi uradne dokumentacije Tauri 2, Rust, Axum, GitHub.
- Testi za parserje YAML, verzije, diagnostiko, lokalizirane napake, učiteljsko avtentikacijo, potek sobe in izbris podatkov. Integracijski testi API in osnovni test desktop UI.
- Gradnja na Windows + macOS + Linux preko GitHub Actions; ne prikazuj artefaktov kot podpisane/notarizirane, če niso.
- Vsaka faza mora biti **dejansko zagnana in demonstrabilna**. Ne puščaj navideznih gumbov, naključnih rezultatov ali hardcoded uspehov. Če funkcija še ni podprta, jo v UI jasno označi kot nedostopno.
- UI naj ima: pregled stanja, profil, seznam preverjanj s statusi in razlago, ponovno preverjanje, pregled deljenja, nastavitve, prijazna stanja praznega seznama in napak.
- Dokumentacija naj vsebuje namestitev, zagon lokalne aplikacije, strežnika in dashboarda, konfiguracijo profilov, varnostne omejitve in podatkovne prakse.

## 9. Način poročanja po vsaki fazi

Odgovori v slovenščini s kratkim poročilom:
1. Faza, kaj je narejeno in kaj dejansko deluje.
2. Katere datoteke/moduli so dodani ali spremenjeni.
3. Kateri testi/buildi so bili zagnani in njihov rezultat.
4. Git veja, commit(i), link do PR, ali je push res uspel.
5. Točni ukazi za lokalni zagon in največ 2–3 pomembne omejitve.
6. Jasno navedi, da je za nadaljevanje potrebna naslednja faza (in po potrebi združitev PR).

Zdaj preberi `ROADMAP.md` in naredi le fazo, ki je zahtevana v uporabnikovem zadnjem sporočilu.
