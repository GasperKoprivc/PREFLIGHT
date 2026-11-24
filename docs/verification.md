# Preverjanje faze 01

Windows: Rust 1.100.0 MSVC, Node 24.15.0, Visual Studio C++ in WebView2.

Uspešno: npm install (0 ranljivosti), TS/ESLint, obe Vite gradnji, cargo check/build/test, Clippy. Ogrodje še nima vsebinskih unit testov (cargo test: 0 testov). CLI --help je izveden; dejanski GET /health vrne ok. Dashboard strežnik vrne HTTP 200. Tauri dev odpre okno; vidni Preflight 0.1.0 potrdi klic get_app_info.

TypeScript 6 je izbran zaradi podprtega razpona typescript-eslint. V tej seji je bilo treba dodati cargo bin na PATH in namestiti rustup komponento clippy. CI vsebuje Windows/macOS/Linux; rezultati oddaljenega CI se preverjajo po pushu.

Faze se po izrecnem uporabnikovem naročilu razvijajo zaporedno do 03, na ločenih vejah. PR-ji niso samodejno združeni.

# Preverjanje faze 02

26 Rust testov uspešnih (5 CLI, 6 unit, 15 engine). Vključeni so simulirani manjkajoči programi, Python launcher, Java/JDK, neveljavni profili, zasebnost in resnična testa timeouta/omejitve izhoda s testnim podprocesom. Testi ne potrebujejo lokalne Jave. Cargo fmt, Clippy in lokalna gradnja so preverjeni.

Pravi profil python-workshop.yaml na razvojnem Windows računalniku vrne dva pass in eno neobvezno opozorilo (Node je zunaj zahtevanega razpona 22); text in JSON vrneta exit 0. Izpis ne vsebuje poti ali surovega stdout/stderr. Podrobne lokalne rezultate namenoma hranimo zunaj Gita.

# Preverjanje faze 03

28 Rust testov in 7 React komponentnih testov uspešnih. Preverjeni so IPC klic, prikaz rezultatov in ponovni zagon, zaklep med preverjanjem, unknown/timeout, varna napaka, prazni profili ter tema/zasebnost. ESLint/TypeScript, Rust fmt/Clippy in celoten npm/Cargo build so uspešni.

Tauri dev se zgradi in zažene. Ponovni vizualni pregled faze 03 je omejen: orodje za nadzor oken javlja minimizirano okno oziroma zaznan uporabnikov vnos in ne omogoči zanesljivega zajema. Funkcionalni preizkus začetnega Tauri okna v fazi 01 je uspel; dejanskega klika končne diagnostike ne označujemo kot ročno potrjenega. Dashboard je bil pregledan tudi v brskalniku, z vidnim napisom »v pripravi«.

# Phase 04 verification

Windows: 42 Rust tests, 9 component tests, Rust fmt/Clippy, TypeScript/ESLint and both frontend builds passed. The real Python workshop profile was run locally in offline mode. Fake host/network tests cover disk thresholds, unavailable VS Code, redirects, mixed DNS, DNS rebinding, private/metadata IPs and profile roundtrips. Native file dialogs require an interactive desktop smoke test; no claim of a completed manual export is made. CI is not awaited at the user's request.

# Phase 05 verification

Windows: 49 Rust tests total (including 7 API integration tests), 9 UI component tests, TypeScript/ESLint, Rust fmt and Clippy passed. The complete workspace build was run. API tests use real SQLite migrations in an isolated in-memory database and cover authorization, two sessions, cross-room access, opt-in validation, invalid results, expiry, deletion, payload/rate limits and CORS.

A real HTTP smoke test on a loopback listener with an in-memory database successfully created a room, joined a participant, uploaded consented results, read teacher aggregates and deleted the room. Tokens were held only in local variables and not printed or saved. Production hosting and interactive native profile save dialogs remain unverified. GitHub CI is not awaited at the user's request.

# Phase 06 verification

51 Rust tests and 14 React tests passed, alongside full workspace build, fmt/Clippy and TS/ESLint. New tests cover endpoint restrictions, minimal share payloads, opt-in UI, deletion failure/retry, teacher recovery, two participants and filtering. Browser smoke test: created a real room through the dashboard, joined two simulated participants through HTTP, uploaded results for only one, and confirmed the other stayed unshared. Browser refresh retained teacher access. Native Tauri sharing was built and component-tested, but not clicked end-to-end in a native window. GitHub Actions are not awaited.

# Phase 07 verification

53 Rust tests and 20 React tests passed, together with Rust fmt/Clippy, TS/ESLint and the complete workspace build. Tests cover single-check execution/rejection of arbitrary IDs, reason/OS guide mapping, static extension command allowlists, command safety, clipboard copying, official links and the selected-check IPC request. No software was installed by guide tests. Windows is the development OS; macOS/Linux instructions are documented manual procedures, not physically verified installations. GitHub Actions are not awaited.

# Phase 08 verification

58 Rust tests and 22 UI tests passed; Rust fmt/Clippy, TS/ESLint and full workspace build passed locally. Installer tests exclusively use fake runners. Verified winget version and live Git.Git 2.56.1 package metadata/source on Windows. Actual package installation/UAC through the UI was not executed; that phase 08 manual acceptance check remains open for an explicit user smoke test. macOS/Linux/ARM and other tools use manual fallback. CI intentionally not awaited.
