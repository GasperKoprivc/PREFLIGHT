# PREFLIGHT — KAKO UPORABITI CODEX (START_HERE.md)

## 1. Pripravi računalnik (Windows 11)

1. Namesti/posodobi **Git**, **GitHub CLI (`gh`)**, **Node.js LTS**, **Rust (`rustup` + MSVC toolchain)** in razvojne predpogoje za Tauri 2 (WebView2, Microsoft C++ Build Tools in druge uradno navedene zahteve). **VS Code** ali drug editor je uporaben.
2. V PowerShell preveri:

```powershell
git --version
gh --version
node --version
npm --version
rustc --version
cargo --version
```

3. Prijava na GitHub (v interaktivnem terminalu):

```powershell
gh auth login
gh auth setup-git
gh auth status
```

4. Ustvari prazno mapo `preflight`, v katero kopiraj datoteke `AGENTS.md`, `ROADMAP.md` in `START_HERE.md`. Odpri **to mapo** v VS Code/Codexu. Datoteke lahko po potrebi tudi dodaš v Git repozitorij, če ga že imaš. Ne ustvarjaj novega repozitorija z enakim imenom, če ga že imaš.

Uradna navodila:
- Tauri 2 predpogoji: https://v2.tauri.app/start/prerequisites/
- Tauri 2 create project: https://v2.tauri.app/start/create-project/
- GitHub CLI prijava: https://docs.github.com/en/github-cli/github-cli/authenticating-with-github-cli

## 2. Prvo sporočilo Codexu — kopiraj TOČNO to

> Preberi `AGENTS.md`, `ROADMAP.md` in `START_HERE.md` v korenu projekta. Ti dokumenti so obvezna projektna specifikacija. Izvedi SAMO **fazo 00** iz `ROADMAP.md`: inicializiraj lokalni Git repozitorij, README, .gitignore, prvi commit in ustvari **private** GitHub repozitorij `preflight` z uporabo GitHub CLI, če še ne obstaja. Če GitHub zahteva prijavo ali dovoljenje, mi povej točen ukaz, ne zahtevaj gesel/žetonov. Ne implementiraj aplikacije še. Na koncu preveri status Git, resnični push in URL repozitorija, poročaj v slovenščini in se ustavi.

## 3. Naslednje sporočilo za vsako fazo

Ko je trenutna faza zaključena in njen PR združen v `main`, v Codex pošlji:

> Preberi `AGENTS.md` in `ROADMAP.md`. Preveri čist `git status`, posodobi `main` z `git pull --ff-only origin main` in izvedi SAMO **fazo NN** iz `ROADMAP.md`. Ustvari predpisano feature vejo, implementiraj vse naloge, zaženi relevantne teste in build, ustvari smiselne commite, pushaj vejo na GitHub in odpri PR v `main` prek `gh pr create`. Če je kak test blokiran, to pošteno pojasni. Ne mergaj PR in ne začni naslednje faze. Na koncu mi poročaj, kaj dela, kako to zaženem, kateri testi so uspeli, ime veje in povezavo do PR.

Namesto `NN` napiši `01`, `02`, `03` in tako naprej.

**Primer za fazo 01:**

> Preberi `AGENTS.md` in `ROADMAP.md`. Izvedi SAMO fazo 01 (monorepo in ogrodje). Ustvari vejo `feat/01-project-setup`, naredi vse predvidene funkcije in teste, pushaj vejo ter odpri PR. Na koncu mi napiši točne ukaze, da lahko sam zaženem desktop aplikacijo, CLI, API in dashboard. Ne začni faze 02.

## 4. Tipični Git proces, ki ga izvaja Codex

```bash
# Po tem, ko je PR prejšnje faze združen:
git switch main
git pull --ff-only origin main
git switch -c feat/02-check-engine

# ... Codex napiše kodo in izvede teste ...

git add -A
git commit -m "feat(core): add runtime diagnostic engine"
git push -u origin feat/02-check-engine
gh pr create --base main --head feat/02-check-engine --title "feat: runtime diagnostic engine" --body "Implemented versioned profiles, runtime checks, CLI JSON output and tests."
```

Po pregledu in uspešnih testih odpreš PR v GitHubu, klikneš **Squash and merge** oziroma izbrano metodo združitve, nato Codexu naročiš naslednjo fazo. Če GitHub repo nima dovoljenj za zaščito glavne veje, še posebej pazi, da ne pushate neposredno v `main`.

## 5. Če se Codex zatakne

Uporabi sporočilo:

> Ostani v trenutni fazi. Preveri dejansko napako v terminalu in popravi najmanjši potreben del. Ne prepisuj celotnega projekta, ne briši mojih sprememb, ne uporabi `git reset --hard` ali `push --force`. Ponovno zaženi relevantne teste, nato naredi commit/push na isto vejo in posodobi PR. Če manjka nameščena sistemska odvisnost ali GitHub prijava, mi napiši točen ukaz za Windows PowerShell in se ustavi pri tem koraku.

## 6. Za katere faze je prvi rezultat uporaben?

- **Faze 00–04:** delujoča lokalna namizna aplikacija in CLI brez strežnika.
- **Faze 05–07:** soba, učitelj, rezultati v živo in vodeni popravki; primerno za prvi nadzorovan pilot.
- **Fazi 08–09:** omejeni varni samodejni popravki, pakiranje, varnostni pregled in release.

Za resnične učence/otroke, javno objavo in GDPR preveri dokumentacijo ter pravila šole, gostovanje, hrambo in ustrezno pravno podlago za obdelavo podatkov. Prostovoljna privolitev sama ni avtomatično zadostna pravna podlaga v vseh šolskih okoliščinah.
